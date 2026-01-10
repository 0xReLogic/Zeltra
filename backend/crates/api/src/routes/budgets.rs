//! Budget management routes.
//!
//! Implements Requirements 13.1-13.7 for Budget API endpoints.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::UserRole,
    repositories::budget::{
        BudgetError, BudgetRepository, CreateBudgetInput, CreateBudgetLineInput, UpdateBudgetInput,
    },
};

/// Creates the budget routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/budgets", get(list_budgets))
        .route("/organizations/{org_id}/budgets", post(create_budget))
        .route(
            "/organizations/{org_id}/budgets/{budget_id}",
            get(get_budget),
        )
        .route(
            "/organizations/{org_id}/budgets/{budget_id}",
            put(update_budget),
        )
        .route(
            "/organizations/{org_id}/budgets/{budget_id}/lines",
            get(list_budget_lines),
        )
        .route(
            "/organizations/{org_id}/budgets/{budget_id}/lines",
            post(create_budget_lines),
        )
        .route(
            "/organizations/{org_id}/budgets/{budget_id}/lock",
            post(lock_budget),
        )
        .route(
            "/organizations/{org_id}/budgets/{budget_id}/vs-actual",
            get(get_budget_vs_actual),
        )
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for creating a budget.
#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    /// Budget name.
    pub name: String,
    /// Budget description.
    pub description: Option<String>,
    /// Fiscal year ID.
    pub fiscal_year_id: Uuid,
    /// Budget type: annual, quarterly, monthly, project.
    pub budget_type: String,
}

/// Request body for updating a budget.
#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    /// Budget name.
    pub name: Option<String>,
    /// Budget description.
    pub description: Option<String>,
    /// Whether the budget is active.
    pub is_active: Option<bool>,
}

/// Request body for creating budget lines in bulk.
#[derive(Debug, Deserialize)]
pub struct CreateBudgetLinesRequest {
    /// Budget lines to create.
    pub lines: Vec<BudgetLineInput>,
}

/// Input for a single budget line.
#[derive(Debug, Deserialize)]
pub struct BudgetLineInput {
    /// Account ID.
    pub account_id: Uuid,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Budgeted amount.
    pub amount: Decimal,
    /// Notes.
    pub notes: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Option<Vec<Uuid>>,
}

/// Response for a budget.
#[derive(Debug, Serialize)]
pub struct BudgetResponse {
    /// Budget ID.
    pub id: Uuid,
    /// Budget name.
    pub name: String,
    /// Budget description.
    pub description: Option<String>,
    /// Fiscal year ID.
    pub fiscal_year_id: Uuid,
    /// Fiscal year name.
    pub fiscal_year_name: String,
    /// Budget type.
    pub budget_type: String,
    /// Currency.
    pub currency: String,
    /// Whether the budget is active.
    pub is_active: bool,
    /// Whether the budget is locked.
    pub is_locked: bool,
    /// Total budgeted amount.
    pub total_budgeted: String,
    /// Created at timestamp.
    pub created_at: String,
    /// Updated at timestamp.
    pub updated_at: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Checks if user is a member of the organization.
async fn check_membership(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.is_member(org_id, user_id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You are not a member of this organization"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Failed to check membership");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response())
        }
    }
}

/// Checks if user has admin or owner role.
async fn check_admin_role(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.has_role(org_id, user_id, UserRole::Admin).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin or owner role required"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Failed to check role");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response())
        }
    }
}

/// Converts budget type string to enum value.
fn parse_budget_type(s: &str) -> Option<zeltra_db::entities::sea_orm_active_enums::BudgetType> {
    use zeltra_db::entities::sea_orm_active_enums::BudgetType;
    match s.to_lowercase().as_str() {
        "annual" => Some(BudgetType::Annual),
        "quarterly" => Some(BudgetType::Quarterly),
        "monthly" => Some(BudgetType::Monthly),
        "project" => Some(BudgetType::Project),
        _ => None,
    }
}

/// Converts budget type enum to string.
fn budget_type_to_string(bt: &zeltra_db::entities::sea_orm_active_enums::BudgetType) -> String {
    use zeltra_db::entities::sea_orm_active_enums::BudgetType;
    match bt {
        BudgetType::Annual => "annual".to_string(),
        BudgetType::Quarterly => "quarterly".to_string(),
        BudgetType::Monthly => "monthly".to_string(),
        BudgetType::Project => "project".to_string(),
    }
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET `/organizations/{org_id}/budgets` - List budgets with summary totals.
///
/// Requirements: 13.2
async fn list_budgets(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    match budget_repo.list_budgets(org_id).await {
        Ok(budgets) => {
            let response: Vec<BudgetResponse> = budgets
                .into_iter()
                .map(|b| BudgetResponse {
                    id: b.budget.id,
                    name: b.budget.name,
                    description: b.budget.description,
                    fiscal_year_id: b.budget.fiscal_year_id,
                    fiscal_year_name: b.fiscal_year_name,
                    budget_type: budget_type_to_string(&b.budget.budget_type),
                    currency: b.budget.currency,
                    is_active: b.budget.is_active,
                    is_locked: b.budget.is_locked,
                    total_budgeted: b.total_budgeted.to_string(),
                    created_at: b.budget.created_at.to_rfc3339(),
                    updated_at: b.budget.updated_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(json!({ "budgets": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list budgets");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response()
        }
    }
}

/// POST `/organizations/{org_id}/budgets` - Create a new budget.
///
/// Requirements: 13.1
async fn create_budget(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateBudgetRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Get organization for currency
    let org = match org_repo.find_by_id(org_id).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to find organization");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    };

    // Parse budget type
    let Some(budget_type) = parse_budget_type(&payload.budget_type) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_budget_type",
                "message": "Invalid budget type. Must be one of: annual, quarterly, monthly, project"
            })),
        )
            .into_response();
    };

    let budget_repo = BudgetRepository::new((*state.db).clone());

    let input = CreateBudgetInput {
        organization_id: org_id,
        fiscal_year_id: payload.fiscal_year_id,
        name: payload.name,
        description: payload.description,
        budget_type,
        currency: org.base_currency,
        created_by: auth.user_id(),
    };

    match budget_repo.create_budget(input).await {
        Ok(budget) => {
            info!(
                org_id = %org_id,
                budget_id = %budget.id,
                name = %budget.name,
                "Budget created"
            );

            (
                StatusCode::CREATED,
                Json(json!({
                    "id": budget.id,
                    "name": budget.name,
                    "description": budget.description,
                    "fiscal_year_id": budget.fiscal_year_id,
                    "budget_type": budget_type_to_string(&budget.budget_type),
                    "currency": budget.currency,
                    "is_active": budget.is_active,
                    "is_locked": budget.is_locked,
                    "created_at": budget.created_at,
                    "updated_at": budget.updated_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create budget");
            map_budget_error(&e)
        }
    }
}

/// GET `/organizations/{org_id}/budgets/{budget_id}` - Get budget with all lines.
///
/// Requirements: 13.3
async fn get_budget(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, budget_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    match budget_repo.get_budget(org_id, budget_id).await {
        Ok(budget) => {
            // Get budget lines
            let lines = match budget_repo.get_budget_lines(budget_id).await {
                Ok(lines) => lines,
                Err(e) => {
                    error!(error = %e, "Failed to get budget lines");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "internal_error",
                            "message": "An error occurred"
                        })),
                    )
                        .into_response();
                }
            };

            let line_responses: Vec<serde_json::Value> = lines
                .into_iter()
                .map(|l| {
                    json!({
                        "id": l.line.id,
                        "account_id": l.line.account_id,
                        "fiscal_period_id": l.line.fiscal_period_id,
                        "amount": l.line.amount.to_string(),
                        "notes": l.line.notes,
                        "dimensions": l.dimensions
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "id": budget.id,
                    "name": budget.name,
                    "description": budget.description,
                    "fiscal_year_id": budget.fiscal_year_id,
                    "budget_type": budget_type_to_string(&budget.budget_type),
                    "currency": budget.currency,
                    "is_active": budget.is_active,
                    "is_locked": budget.is_locked,
                    "created_at": budget.created_at,
                    "updated_at": budget.updated_at,
                    "lines": line_responses
                })),
            )
                .into_response()
        }
        Err(BudgetError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Budget not found"
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get budget");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response()
        }
    }
}

/// PUT `/organizations/{org_id}/budgets/{budget_id}` - Update budget.
async fn update_budget(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, budget_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateBudgetRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    let input = UpdateBudgetInput {
        name: payload.name,
        description: payload.description.map(Some),
        is_active: payload.is_active,
    };

    match budget_repo.update_budget(org_id, budget_id, input).await {
        Ok(budget) => {
            info!(
                org_id = %org_id,
                budget_id = %budget_id,
                "Budget updated"
            );

            (
                StatusCode::OK,
                Json(json!({
                    "id": budget.id,
                    "name": budget.name,
                    "description": budget.description,
                    "fiscal_year_id": budget.fiscal_year_id,
                    "budget_type": budget_type_to_string(&budget.budget_type),
                    "currency": budget.currency,
                    "is_active": budget.is_active,
                    "is_locked": budget.is_locked,
                    "updated_at": budget.updated_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to update budget");
            map_budget_error(&e)
        }
    }
}

/// GET `/organizations/{org_id}/budgets/{budget_id}/lines` - Get budget lines.
///
/// Requirements: 13.5
async fn list_budget_lines(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, budget_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    // Verify budget belongs to this organization
    if let Err(e) = budget_repo.get_budget(org_id, budget_id).await {
        if let BudgetError::NotFound(_) = e {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Budget not found"
                })),
            )
                .into_response();
        }
        error!(error = %e, "Failed to find budget");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "internal_error",
                "message": "An error occurred"
            })),
        )
            .into_response();
    }

    match budget_repo.get_budget_lines(budget_id).await {
        Ok(lines) => {
            let response: Vec<serde_json::Value> = lines
                .into_iter()
                .map(|l| {
                    json!({
                        "id": l.line.id,
                        "account_id": l.line.account_id,
                        "fiscal_period_id": l.line.fiscal_period_id,
                        "amount": l.line.amount.to_string(),
                        "notes": l.line.notes,
                        "dimensions": l.dimensions
                    })
                })
                .collect();

            (StatusCode::OK, Json(json!({ "lines": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list budget lines");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response()
        }
    }
}

/// POST `/organizations/{org_id}/budgets/{budget_id}/lines` - Create budget lines in bulk.
///
/// Requirements: 13.4
async fn create_budget_lines(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, budget_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateBudgetLinesRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    // Convert input
    let inputs: Vec<CreateBudgetLineInput> = payload
        .lines
        .into_iter()
        .map(|l| CreateBudgetLineInput {
            account_id: l.account_id,
            fiscal_period_id: l.fiscal_period_id,
            amount: l.amount,
            notes: l.notes,
            dimensions: l.dimensions.unwrap_or_default(),
        })
        .collect();

    match budget_repo
        .create_budget_lines(org_id, budget_id, inputs)
        .await
    {
        Ok(lines) => {
            info!(
                org_id = %org_id,
                budget_id = %budget_id,
                count = lines.len(),
                "Budget lines created"
            );

            let response: Vec<serde_json::Value> = lines
                .into_iter()
                .map(|l| {
                    json!({
                        "id": l.line.id,
                        "account_id": l.line.account_id,
                        "fiscal_period_id": l.line.fiscal_period_id,
                        "amount": l.line.amount.to_string(),
                        "notes": l.line.notes,
                        "dimensions": l.dimensions
                    })
                })
                .collect();

            (StatusCode::CREATED, Json(json!({ "lines": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create budget lines");
            map_budget_error(&e)
        }
    }
}

/// POST `/organizations/{org_id}/budgets/{budget_id}/lock` - Lock budget.
///
/// Requirements: 13.6
async fn lock_budget(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, budget_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    match budget_repo.lock_budget(org_id, budget_id).await {
        Ok(budget) => {
            info!(
                org_id = %org_id,
                budget_id = %budget_id,
                "Budget locked"
            );

            (
                StatusCode::OK,
                Json(json!({
                    "id": budget.id,
                    "name": budget.name,
                    "is_locked": budget.is_locked,
                    "updated_at": budget.updated_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to lock budget");
            map_budget_error(&e)
        }
    }
}

/// Query parameters for budget vs actual.
#[derive(Debug, serde::Deserialize)]
pub struct BudgetVsActualQuery {
    /// Filter by fiscal period ID.
    pub fiscal_period_id: Option<Uuid>,
    /// Filter by dimension value IDs (comma-separated).
    pub dimensions: Option<String>,
}

/// GET `/organizations/{org_id}/budgets/{budget_id}/vs-actual` - Get budget vs actual comparison.
///
/// Requirements: 13.7
async fn get_budget_vs_actual(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, budget_id)): Path<(Uuid, Uuid)>,
    axum::extract::Query(query): axum::extract::Query<BudgetVsActualQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let budget_repo = BudgetRepository::new((*state.db).clone());

    // Parse dimension filters
    let dimension_filters: Vec<Uuid> = query
        .dimensions
        .as_ref()
        .map(|s| {
            s.split(',')
                .filter_map(|id| Uuid::parse_str(id.trim()).ok())
                .collect()
        })
        .unwrap_or_default();

    match budget_repo
        .get_budget_vs_actual(
            org_id,
            budget_id,
            query.fiscal_period_id,
            &dimension_filters,
        )
        .await
    {
        Ok((lines, summary)) => {
            let line_responses: Vec<serde_json::Value> = lines
                .into_iter()
                .map(|l| {
                    json!({
                        "id": l.line.id,
                        "account_id": l.line.account_id,
                        "account_code": l.account_code,
                        "account_name": l.account_name,
                        "fiscal_period_id": l.line.fiscal_period_id,
                        "period_name": l.period_name,
                        "budgeted": l.line.amount.to_string(),
                        "actual": l.actual.to_string(),
                        "variance": l.variance.to_string(),
                        "utilization_percent": l.utilization_percent.to_string(),
                        "status": l.status,
                        "dimensions": l.dimensions.iter().map(|d| json!({
                            "dimension_type": d.dimension_type,
                            "code": d.code,
                            "name": d.name
                        })).collect::<Vec<_>>()
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "budget_id": budget_id,
                    "lines": line_responses,
                    "summary": {
                        "total_budgeted": summary.total_budgeted.to_string(),
                        "total_actual": summary.total_actual.to_string(),
                        "total_variance": summary.total_variance.to_string(),
                        "overall_utilization": summary.overall_utilization.to_string()
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get budget vs actual");
            map_budget_error(&e)
        }
    }
}

// ============================================================================
// Error Mapping
// ============================================================================

/// Maps budget errors to HTTP responses.
fn map_budget_error(e: &BudgetError) -> axum::response::Response {
    match e {
        BudgetError::NotFound(id) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Budget not found: {}", id)
            })),
        )
            .into_response(),
        BudgetError::BudgetLocked => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "budget_locked",
                "message": "Budget is locked and cannot be modified"
            })),
        )
            .into_response(),
        BudgetError::DuplicateName => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "duplicate_name",
                "message": "Budget name already exists for this fiscal year"
            })),
        )
            .into_response(),
        BudgetError::FiscalYearNotFound(id) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "fiscal_year_not_found",
                "message": format!("Fiscal year not found: {}", id)
            })),
        )
            .into_response(),
        BudgetError::FiscalPeriodNotFound(id) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "fiscal_period_not_found",
                "message": format!("Fiscal period not found: {}", id)
            })),
        )
            .into_response(),
        BudgetError::PeriodNotInFiscalYear => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "period_not_in_fiscal_year",
                "message": "Fiscal period does not belong to budget's fiscal year"
            })),
        )
            .into_response(),
        BudgetError::AccountNotFound(id) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "account_not_found",
                "message": format!("Account not found: {}", id)
            })),
        )
            .into_response(),
        BudgetError::DuplicateBudgetLine => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "duplicate_budget_line",
                "message": "Budget line already exists for this account and period"
            })),
        )
            .into_response(),
        BudgetError::NegativeAmount => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "negative_amount",
                "message": "Amount cannot be negative"
            })),
        )
            .into_response(),
        BudgetError::InvalidDimension(id) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_dimension",
                "message": format!("Invalid dimension value: {}", id)
            })),
        )
            .into_response(),
        BudgetError::BudgetLineNotFound(id) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "budget_line_not_found",
                "message": format!("Budget line not found: {}", id)
            })),
        )
            .into_response(),
        BudgetError::Database(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "internal_error",
                "message": "An error occurred"
            })),
        )
            .into_response(),
    }
}
