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
#[derive(Debug, Deserialize)]
pub struct CreateApprovalRuleRequest {
    /// Name of the approval rule.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Minimum amount threshold (inclusive).
    pub min_amount: Option<String>,
    /// Maximum amount threshold (inclusive).
    pub max_amount: Option<String>,
    /// Transaction types this rule applies to.
    pub transaction_types: Vec<String>,
    /// Required role to approve (viewer, submitter, approver, accountant, admin, owner).
    pub required_role: String,
    /// Priority (lower = higher priority).
    pub priority: i16,
}

/// Request body for updating an approval rule.
#[derive(Debug, Deserialize)]
pub struct UpdateApprovalRuleRequest {
    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New minimum amount.
    pub min_amount: Option<String>,
    /// New maximum amount.
    pub max_amount: Option<String>,
    /// New transaction types.
    pub transaction_types: Option<Vec<String>>,
    /// New required role.
    pub required_role: Option<String>,
    /// New priority.
    pub priority: Option<i16>,
    /// Active status.
    pub is_active: Option<bool>,
}

/// Response for an approval rule.
#[derive(Debug, Serialize)]
pub struct ApprovalRuleResponse {
    /// Rule ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Minimum amount threshold.
    pub min_amount: Option<String>,
    /// Maximum amount threshold.
    pub max_amount: Option<String>,
    /// Transaction types.
    pub transaction_types: Vec<String>,
    /// Required role.
    pub required_role: String,
    /// Priority.
    pub priority: i16,
    /// Active status.
    pub is_active: bool,
    /// Created at timestamp.
    pub created_at: String,
    /// Updated at timestamp.
    pub updated_at: String,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET `/organizations/{org_id}/approval-rules` - List approval rules.
///
/// Requirements: 6.9
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
///
/// Requirements: 6.8
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
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "name_required",
                "message": "Name is required"
            })),
        )
            .into_response();
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
        name: payload.name,
        description: payload.description,
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
        name: payload.name,
        description: payload.description.map(Some),
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
        Some(s) if !s.is_empty() => match Decimal::from_str(s) {
            Ok(d) if d >= Decimal::ZERO => Ok(Some(d)),
            Ok(_) => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_amount",
                    "message": "Amount must be non-negative"
                })),
            )
                .into_response()),
            Err(_) => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_amount",
                    "message": "Invalid amount format"
                })),
            )
                .into_response()),
        },
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
