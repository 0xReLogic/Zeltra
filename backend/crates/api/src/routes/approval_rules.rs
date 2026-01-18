//! Approval Rules management routes.
//!
//! Implements Requirements 6.8, 6.9 for approval rules API endpoints.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    repositories::approval_rule::{
        ApprovalRuleError, ApprovalRuleRepository, CreateApprovalRuleInput, UpdateApprovalRuleInput,
    },
};

/// Creates the approval rules routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/approval-rules",
            get(list_approval_rules),
        )
        .route(
            "/organizations/{org_id}/approval-rules",
            post(create_approval_rule),
        )
        .route(
            "/organizations/{org_id}/approval-rules/{rule_id}",
            get(get_approval_rule),
        )
        .route(
            "/organizations/{org_id}/approval-rules/{rule_id}",
            patch(update_approval_rule),
        )
        .route(
            "/organizations/{org_id}/approval-rules/{rule_id}",
            delete(delete_approval_rule),
        )
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for creating an approval rule.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateApprovalRuleRequest {
    /// Name of the approval rule (1-255 characters).
    #[schema(min_length = 1, max_length = 255, example = "High Value Bills")]
    pub name: String,
    /// Optional description (max 1000 characters).
    #[schema(max_length = 1000)]
    pub description: Option<String>,
    /// Minimum amount threshold (inclusive, must be a valid decimal with up to 2 decimal places).
    #[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "1000.00")]
    pub min_amount: Option<String>,
    /// Maximum amount threshold (inclusive, must be a valid decimal with up to 2 decimal places).
    #[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "5000.00")]
    pub max_amount: Option<String>,
    /// Transaction types this rule applies to (valid values: journal, invoice, bill, payment, expense, transfer, adjustment, opening_balance, reversal, accrual, revaluation, intercompany).
    #[schema(inline, example = json!(["bill", "invoice"]))]
    pub transaction_types: Vec<String>,
    /// Required role to approve (valid values: viewer, submitter, approver, accountant, admin, owner).
    #[schema(inline, example = "approver")]
    pub required_role: String,
    /// Priority (lower = higher priority, valid range: 1-100).
    #[schema(minimum = 1, maximum = 100, example = 1)]
    pub priority: i16,
}

/// Request body for updating an approval rule.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateApprovalRuleRequest {
    /// New name (1-255 characters).
    #[schema(min_length = 1, max_length = 255)]
    pub name: Option<String>,
    /// New description (max 1000 characters).
    #[schema(max_length = 1000)]
    pub description: Option<String>,
    /// New minimum amount (must be a valid decimal with up to 2 decimal places).
    #[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "1000.00")]
    pub min_amount: Option<String>,
    /// New maximum amount (must be a valid decimal with up to 2 decimal places).
    #[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "5000.00")]
    pub max_amount: Option<String>,
    /// New transaction types (valid values: journal, invoice, bill, payment, expense, transfer, adjustment, opening_balance, reversal, accrual, revaluation, intercompany).
    #[schema(inline, example = json!(["bill", "invoice"]))]
    pub transaction_types: Option<Vec<String>>,
    /// New required role (valid values: viewer, submitter, approver, accountant, admin, owner).
    #[schema(inline, example = "approver")]
    pub required_role: Option<String>,
    /// New priority (valid range: 1-100).
    #[schema(minimum = 1, maximum = 100)]
    pub priority: Option<i16>,
    /// Active status.
    pub is_active: Option<bool>,
}

/// Response for an approval rule.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApprovalRuleResponse {
    /// Rule ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Minimum amount threshold (must be a valid decimal with up to 2 decimal places).
    #[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "1000.00")]
    pub min_amount: Option<String>,
    /// Maximum amount threshold (must be a valid decimal with up to 2 decimal places).
    #[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "5000.00")]
    pub max_amount: Option<String>,
    /// Transaction types (valid values: journal, invoice, bill, payment, expense, transfer, adjustment, opening_balance, reversal, accrual, revaluation, intercompany).
    #[schema(inline, example = json!(["bill", "invoice"]))]
    pub transaction_types: Vec<String>,
    /// Required role (valid values: viewer, submitter, approver, accountant, admin, owner).
    #[schema(inline, example = "approver")]
    pub required_role: String,
    /// Priority.
    pub priority: i16,
    /// Active status.
    pub is_active: bool,
    /// Created at timestamp.
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:30:00Z")]
    pub created_at: String,
    /// Updated at timestamp.
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:30:00Z")]
    pub updated_at: String,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET `/organizations/{org_id}/approval-rules` - List approval rules.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/approval-rules",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("page" = Option<u32>, Query, description = "Page number (default: 1, min: 1)"),
        ("per_page" = Option<u32>, Query, description = "Items per page (default: 20, min: 1, max: 100)"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("transaction_type" = Option<String>, Query, description = "Filter by transaction type"),
        ("sort_by" = Option<String>, Query, description = "Sort by field (priority, created_at, name)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc, desc)")
    ),
    responses(
        (status = 200, description = "Paginated list of approval rules", body = inline(Object), example = json!({
            "data": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "organization_id": "550e8400-e29b-41d4-a716-446655440001",
                    "name": "High Value Bills",
                    "description": "Requires approval for bills over $5000",
                    "min_amount": "5000.00",
                    "max_amount": null,
                    "transaction_types": ["bill"],
                    "required_role": "approver",
                    "priority": 1,
                    "is_active": true,
                    "created_at": "2024-01-15T10:30:00Z",
                    "updated_at": "2024-01-15T10:30:00Z"
                }
            ],
            "meta": {
                "page": 1,
                "per_page": 20,
                "total": 150,
                "total_pages": 8
            }
        })),
        (status = 400, description = "Invalid query parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Approval Rules",
    security(("bearerAuth" = []))
)]
async fn list_approval_rules(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let rule_repo = ApprovalRuleRepository::new((*state.db).clone());

    match rule_repo.list_rules(org_id).await {
        Ok(rules) => {
            let items: Vec<ApprovalRuleResponse> =
                rules.into_iter().map(rule_to_response).collect();

            (StatusCode::OK, Json(json!({ "data": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list approval rules");
            approval_rule_error_response(e)
        }
    }
}

/// POST `/organizations/{org_id}/approval-rules` - Create approval rule.
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/approval-rules",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateApprovalRuleRequest,
    responses(
        (status = 201, description = "Approval rule created successfully", body = ApprovalRuleResponse),
        (status = 400, description = "Invalid input or amount format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Approval Rules",
    security(("bearerAuth" = []))
)]
async fn create_approval_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateApprovalRuleRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership and admin role
    if let Err(response) = check_admin_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Validate name
    let name = payload.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "name_required",
                "message": "Name is required"
            })),
        )
            .into_response();
    }

    if name.len() > 255 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "name_too_long",
                "message": "Name must not exceed 255 characters"
            })),
        )
            .into_response();
    }

    // Sanitize and validate description
    let description = payload.description.as_ref().map(|d| d.trim().to_string());
    if let Some(ref desc) = description {
        if desc.len() > 1000 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "description_too_long",
                    "message": "Description must not exceed 1000 characters"
                })),
            )
                .into_response();
        }
    }

    // Validate transaction types
    if payload.transaction_types.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "transaction_types_required",
                "message": "At least one transaction type is required"
            })),
        )
            .into_response();
    }

    // Validate priority range
    if payload.priority < 1 || payload.priority > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_priority",
                "message": "Priority must be between 1 and 100"
            })),
        )
            .into_response();
    }

    // Parse amounts
    let min_amount = match parse_optional_decimal(payload.min_amount.as_deref()) {
        Ok(a) => a,
        Err(e) => return e,
    };

    let max_amount = match parse_optional_decimal(payload.max_amount.as_deref()) {
        Ok(a) => a,
        Err(e) => return e,
    };

    // Validate amount range
    if let (Some(min), Some(max)) = (min_amount, max_amount)
        && min > max
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_amount_range",
                "message": "min_amount cannot be greater than max_amount"
            })),
        )
            .into_response();
    }

    let rule_repo = ApprovalRuleRepository::new((*state.db).clone());

    let input = CreateApprovalRuleInput {
        name: name.to_string(),
        description,
        min_amount,
        max_amount,
        transaction_types: payload.transaction_types,
        required_role: payload.required_role,
        priority: payload.priority,
    };

    match rule_repo.create_rule(org_id, input).await {
        Ok(rule) => {
            info!(
                org_id = %org_id,
                rule_id = %rule.id,
                "Approval rule created"
            );

            (StatusCode::CREATED, Json(rule_to_response(rule))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create approval rule");
            approval_rule_error_response(e)
        }
    }
}

/// GET `/organizations/{org_id}/approval-rules/{rule_id}` - Get approval rule.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/approval-rules/{rule_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("rule_id" = Uuid, Path, description = "Approval Rule ID")
    ),
    responses(
        (status = 200, description = "Approval rule details", body = ApprovalRuleResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Approval rule not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Approval Rules",
    security(("bearerAuth" = []))
)]
async fn get_approval_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, rule_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let rule_repo = ApprovalRuleRepository::new((*state.db).clone());

    match rule_repo.get_rule(org_id, rule_id).await {
        Ok(rule) => (StatusCode::OK, Json(rule_to_response(rule))).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get approval rule");
            approval_rule_error_response(e)
        }
    }
}

/// PATCH `/organizations/{org_id}/approval-rules/{rule_id}` - Update approval rule.
#[utoipa::path(
    patch,
    path = "/organizations/{org_id}/approval-rules/{rule_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("rule_id" = Uuid, Path, description = "Approval Rule ID")
    ),
    request_body = UpdateApprovalRuleRequest,
    responses(
        (status = 200, description = "Approval rule updated successfully", body = ApprovalRuleResponse),
        (status = 400, description = "Invalid input or amount format"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Approval rule not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Approval Rules",
    security(("bearerAuth" = []))
)]
async fn update_approval_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, rule_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateApprovalRuleRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership and admin role
    if let Err(response) = check_admin_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Validate name length if provided
    let name = payload.name.as_ref().map(|n| n.trim().to_string());
    if let Some(ref n) = name {
        if n.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "name_required",
                    "message": "Name is required"
                })),
            )
                .into_response();
        }

        if n.len() > 255 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "name_too_long",
                    "message": "Name must not exceed 255 characters"
                })),
            )
                .into_response();
        }
    }

    // Sanitize and validate description length if provided
    let description = payload.description.as_ref().map(|d| Some(d.trim().to_string()));
    if let Some(Some(ref desc)) = description {
        if desc.len() > 1000 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "description_too_long",
                    "message": "Description must not exceed 1000 characters"
                })),
            )
                .into_response();
        }
    }

    // Validate priority range if provided
    if let Some(priority) = payload.priority {
        if priority < 1 || priority > 100 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_priority",
                    "message": "Priority must be between 1 and 100"
                })),
            )
                .into_response();
        }
    }

    // Parse amounts if provided
    let min_amount = match payload.min_amount.as_deref() {
        Some(s) => match parse_optional_decimal(Some(s)) {
            Ok(a) => Some(a),
            Err(e) => return e,
        },
        None => None,
    };

    let max_amount = match payload.max_amount.as_deref() {
        Some(s) => match parse_optional_decimal(Some(s)) {
            Ok(a) => Some(a),
            Err(e) => return e,
        },
        None => None,
    };

    let rule_repo = ApprovalRuleRepository::new((*state.db).clone());

    let input = UpdateApprovalRuleInput {
        name,
        description,
        min_amount,
        max_amount,
        transaction_types: payload.transaction_types,
        required_role: payload.required_role,
        priority: payload.priority,
        is_active: payload.is_active,
    };

    match rule_repo.update_rule(org_id, rule_id, input).await {
        Ok(rule) => {
            info!(
                org_id = %org_id,
                rule_id = %rule_id,
                "Approval rule updated"
            );

            (StatusCode::OK, Json(rule_to_response(rule))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to update approval rule");
            approval_rule_error_response(e)
        }
    }
}

/// DELETE `/organizations/{org_id}/approval-rules/{rule_id}` - Delete approval rule.
#[utoipa::path(
    delete,
    path = "/organizations/{org_id}/approval-rules/{rule_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("rule_id" = Uuid, Path, description = "Approval Rule ID")
    ),
    responses(
        (status = 204, description = "Approval rule deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Approval rule not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Approval Rules",
    security(("bearerAuth" = []))
)]
async fn delete_approval_rule(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, rule_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership and admin role
    if let Err(response) = check_admin_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let rule_repo = ApprovalRuleRepository::new((*state.db).clone());

    match rule_repo.delete_rule(org_id, rule_id).await {
        Ok(()) => {
            info!(
                org_id = %org_id,
                rule_id = %rule_id,
                "Approval rule deleted"
            );

            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to delete approval rule");
            approval_rule_error_response(e)
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn rule_to_response(rule: zeltra_db::entities::approval_rules::Model) -> ApprovalRuleResponse {
    use zeltra_db::entities::sea_orm_active_enums::{TransactionType, UserRole};

    let transaction_types: Vec<String> = rule
        .transaction_types
        .iter()
        .map(|t| match t {
            TransactionType::Journal => "journal".to_string(),
            TransactionType::Invoice => "invoice".to_string(),
            TransactionType::Bill => "bill".to_string(),
            TransactionType::Payment => "payment".to_string(),
            TransactionType::Expense => "expense".to_string(),
            TransactionType::Transfer => "transfer".to_string(),
            TransactionType::Adjustment => "adjustment".to_string(),
            TransactionType::OpeningBalance => "opening_balance".to_string(),
            TransactionType::Reversal => "reversal".to_string(),
            TransactionType::Accrual => "accrual".to_string(),
            TransactionType::Revaluation => "revaluation".to_string(),
            TransactionType::Intercompany => "intercompany".to_string(),
        })
        .collect();

    let required_role = match rule.required_role {
        UserRole::Viewer => "viewer".to_string(),
        UserRole::Submitter => "submitter".to_string(),
        UserRole::Approver => "approver".to_string(),
        UserRole::Accountant => "accountant".to_string(),
        UserRole::Admin => "admin".to_string(),
        UserRole::Owner => "owner".to_string(),
    };

    ApprovalRuleResponse {
        id: rule.id,
        organization_id: rule.organization_id,
        name: rule.name,
        description: rule.description,
        min_amount: rule.min_amount.map(|a| a.to_string()),
        max_amount: rule.max_amount.map(|a| a.to_string()),
        transaction_types,
        required_role,
        priority: rule.priority,
        is_active: rule.is_active,
        created_at: rule.created_at.to_rfc3339(),
        updated_at: rule.updated_at.to_rfc3339(),
    }
}

#[allow(clippy::result_large_err)]
fn parse_optional_decimal(s: Option<&str>) -> Result<Option<Decimal>, axum::response::Response> {
    match s {
        Some(s) if !s.is_empty() => {
            // Validate pattern: must be digits with optional 2 decimal places
            let pattern = regex::Regex::new(r"^[0-9]+(\.[0-9]{1,2})?$").unwrap();
            if !pattern.is_match(s) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_amount_format",
                        "message": "Amount must be a valid decimal with up to 2 decimal places (e.g., 1000.00)"
                    })),
                )
                    .into_response());
            }

            match Decimal::from_str(s) {
                Ok(d) if d < Decimal::ZERO => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_amount",
                        "message": "Amount must be non-negative"
                    })),
                )
                    .into_response()),
                Ok(d) if d > Decimal::from(999_999_999) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "amount_too_large",
                        "message": "Amount must not exceed 999,999,999"
                    })),
                )
                    .into_response()),
                Ok(d) => Ok(Some(d)),
                Err(_) => Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_amount",
                        "message": "Invalid amount format"
                    })),
                )
                    .into_response()),
            }
        }
        _ => Ok(None),
    }
}

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
            error!(error = %e, "Database error checking membership");
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

async fn check_admin_membership(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    // First check membership and get role
    match org_repo.get_user_membership(org_id, user_id).await {
        Ok(Some(membership)) => {
            use zeltra_db::entities::sea_orm_active_enums::UserRole;
            match membership.role {
                UserRole::Admin | UserRole::Owner => Ok(()),
                _ => Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "admin_required",
                        "message": "Admin or Owner role required for this operation"
                    })),
                )
                    .into_response()),
            }
        }
        Ok(None) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You are not a member of this organization"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking membership");
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

fn approval_rule_error_response(e: ApprovalRuleError) -> axum::response::Response {
    match e {
        ApprovalRuleError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Approval rule not found"
            })),
        )
            .into_response(),
        ApprovalRuleError::InvalidTransactionType(t) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_transaction_type",
                "message": format!("Invalid transaction type: {}", t)
            })),
        )
            .into_response(),
        ApprovalRuleError::InvalidRole(r) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_role",
                "message": format!("Invalid role: {}", r)
            })),
        )
            .into_response(),
        ApprovalRuleError::Database(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "internal_error",
                "message": "An error occurred"
            })),
        )
            .into_response(),
    }
}
