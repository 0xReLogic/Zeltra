//! Account management routes.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::{AccountSubtype, AccountType, UserRole},
    repositories::account::{AccountFilter, AccountRepository, CreateAccountInput, UpdateAccountInput},
};

/// Creates the account routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/accounts", get(list_accounts))
        .route("/organizations/{org_id}/accounts", post(create_account))
        .route("/organizations/{org_id}/accounts/{account_id}", get(get_account))
        .route("/organizations/{org_id}/accounts/{account_id}", put(update_account))
        .route("/organizations/{org_id}/accounts/{account_id}", delete(delete_account))
        .route("/organizations/{org_id}/accounts/{account_id}/balance", get(get_account_balance))
        .route("/organizations/{org_id}/accounts/{account_id}/ledger", get(get_account_ledger))
}

/// Query parameters for listing accounts.
#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    /// Filter by account type.
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    /// Filter by active status.
    pub active: Option<bool>,
    /// Filter by currency.
    pub currency: Option<String>,
}

/// Request body for creating an account.
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    /// Account code (must be unique within organization).
    pub code: String,
    /// Account name.
    pub name: String,
    /// Account description.
    pub description: Option<String>,
    /// Account type: asset, liability, equity, revenue, expense.
    #[serde(rename = "type")]
    pub account_type: String,
    /// Account subtype for more specific categorization.
    pub subtype: Option<String>,
    /// Parent account ID for hierarchical structure.
    pub parent_id: Option<Uuid>,
    /// Currency code.
    pub currency: String,
    /// Whether the account is active (default: true).
    pub is_active: Option<bool>,
    /// Whether direct posting is allowed (default: true).
    pub allow_direct_posting: Option<bool>,
}

/// Request body for updating an account.
#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    /// Account code.
    pub code: Option<String>,
    /// Account name.
    pub name: Option<String>,
    /// Account description.
    pub description: Option<String>,
    /// Account type (only if no ledger entries).
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    /// Account subtype.
    pub subtype: Option<String>,
    /// Parent account ID.
    pub parent_id: Option<Uuid>,
    /// Whether the account is active.
    pub is_active: Option<bool>,
    /// Whether direct posting is allowed.
    pub allow_direct_posting: Option<bool>,
}

/// Response for an account.
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    /// Account ID.
    pub id: Uuid,
    /// Account code.
    pub code: String,
    /// Account name.
    pub name: String,
    /// Account description.
    pub description: Option<String>,
    /// Account type.
    #[serde(rename = "type")]
    pub account_type: String,
    /// Account subtype.
    pub subtype: Option<String>,
    /// Parent account ID.
    pub parent_id: Option<Uuid>,
    /// Currency code.
    pub currency: String,
    /// Current balance.
    pub balance: String,
    /// Whether the account is active.
    pub is_active: bool,
    /// Whether direct posting is allowed.
    pub allow_direct_posting: bool,
}

/// Query parameters for getting account balance at a specific date.
#[derive(Debug, Deserialize)]
pub struct BalanceQuery {
    /// Date to get balance as of (YYYY-MM-DD format). Defaults to today.
    pub as_of: Option<NaiveDate>,
}

/// Query parameters for listing ledger entries.
#[derive(Debug, Deserialize)]
pub struct LedgerQuery {
    /// Start date filter (inclusive, YYYY-MM-DD format).
    pub from: Option<NaiveDate>,
    /// End date filter (inclusive, YYYY-MM-DD format).
    pub to: Option<NaiveDate>,
    /// Page number (1-indexed, default: 1).
    pub page: Option<u64>,
    /// Number of entries per page (default: 50, max: 100).
    pub limit: Option<u64>,
}

/// Response for a ledger entry.
#[derive(Debug, Serialize)]
pub struct LedgerEntryResponse {
    /// Entry ID.
    pub id: Uuid,
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Transaction date.
    pub transaction_date: String,
    /// Transaction reference number.
    pub reference_number: Option<String>,
    /// Transaction description.
    pub description: String,
    /// Transaction status.
    pub status: String,
    /// Source currency.
    pub source_currency: String,
    /// Source amount.
    pub source_amount: String,
    /// Exchange rate.
    pub exchange_rate: String,
    /// Functional currency.
    pub functional_currency: String,
    /// Functional amount.
    pub functional_amount: String,
    /// Debit amount.
    pub debit: String,
    /// Credit amount.
    pub credit: String,
    /// Entry memo.
    pub memo: Option<String>,
    /// Running balance before this entry.
    pub previous_balance: String,
    /// Running balance after this entry.
    pub current_balance: String,
    /// Entry timestamp.
    pub created_at: String,
}


/// GET `/organizations/{org_id}/accounts` - List accounts with balances.
async fn list_accounts(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListAccountsQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let account_repo = AccountRepository::new((*state.db).clone());

    // Build filter
    let filter = AccountFilter {
        account_type: query.account_type.as_ref().and_then(|t| string_to_account_type(t)),
        is_active: query.active,
        parent_id: None,
    };

    match account_repo.list_accounts(org_id, filter).await {
        Ok(accounts) => {
            let response: Vec<AccountResponse> = accounts
                .into_iter()
                .map(|a| AccountResponse {
                    id: a.account.id,
                    code: a.account.code,
                    name: a.account.name,
                    description: a.account.description,
                    account_type: account_type_to_string(&a.account.account_type),
                    subtype: a.account.account_subtype.map(|s| account_subtype_to_string(&s)),
                    parent_id: a.account.parent_id,
                    currency: a.account.currency,
                    balance: a.balance.to_string(),
                    is_active: a.account.is_active,
                    allow_direct_posting: a.account.allow_direct_posting,
                })
                .collect();

            (StatusCode::OK, Json(json!({ "accounts": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list accounts");
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

/// POST `/organizations/{org_id}/accounts` - Create an account.
async fn create_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Parse account type
    let Some(account_type) = string_to_account_type(&payload.account_type) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_account_type",
                "message": "Invalid account type. Must be one of: asset, liability, equity, revenue, expense"
            })),
        )
            .into_response();
    };

    // Parse subtype if provided
    let account_subtype = payload.subtype.as_ref().and_then(|s| string_to_account_subtype(s));

    let account_repo = AccountRepository::new((*state.db).clone());

    let input = CreateAccountInput {
        organization_id: org_id,
        code: payload.code,
        name: payload.name,
        description: payload.description,
        account_type,
        account_subtype,
        parent_id: payload.parent_id,
        currency: payload.currency,
        is_active: payload.is_active.unwrap_or(true),
        allow_direct_posting: payload.allow_direct_posting.unwrap_or(true),
    };

    match account_repo.create_account(input).await {
        Ok(account) => {
            info!(
                org_id = %org_id,
                account_id = %account.id,
                code = %account.code,
                "Account created"
            );

            (
                StatusCode::CREATED,
                Json(json!({
                    "id": account.id,
                    "code": account.code,
                    "name": account.name,
                    "description": account.description,
                    "type": account_type_to_string(&account.account_type),
                    "subtype": account.account_subtype.map(|s| account_subtype_to_string(&s)),
                    "parent_id": account.parent_id,
                    "currency": account.currency,
                    "balance": "0",
                    "is_active": account.is_active,
                    "allow_direct_posting": account.allow_direct_posting,
                    "created_at": account.created_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create account");
            match e {
                zeltra_db::repositories::account::AccountError::DuplicateCode(code) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "duplicate_code",
                        "message": format!("Account code '{}' already exists", code)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::account::AccountError::CurrencyNotFound(currency) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "currency_not_found",
                        "message": format!("Currency '{}' not found", currency)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::account::AccountError::ParentNotFound(id) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "parent_not_found",
                        "message": format!("Parent account not found: {}", id)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::account::AccountError::ParentWrongOrganization => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "parent_wrong_organization",
                        "message": "Parent account belongs to different organization"
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}


/// GET `/organizations/{org_id}/accounts/{account_id}` - Get account detail.
async fn get_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, account_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let account_repo = AccountRepository::new((*state.db).clone());

    match account_repo.find_account_by_id(account_id).await {
        Ok(Some(a)) if a.account.organization_id == org_id => {
            (
                StatusCode::OK,
                Json(json!({
                    "id": a.account.id,
                    "code": a.account.code,
                    "name": a.account.name,
                    "description": a.account.description,
                    "type": account_type_to_string(&a.account.account_type),
                    "subtype": a.account.account_subtype.map(|s| account_subtype_to_string(&s)),
                    "parent_id": a.account.parent_id,
                    "currency": a.account.currency,
                    "balance": a.balance.to_string(),
                    "is_active": a.account.is_active,
                    "allow_direct_posting": a.account.allow_direct_posting,
                    "is_system_account": a.account.is_system_account,
                    "is_bank_account": a.account.is_bank_account,
                    "bank_account_number": a.account.bank_account_number,
                    "created_at": a.account.created_at,
                    "updated_at": a.account.updated_at
                })),
            )
                .into_response()
        }
        Ok(Some(_)) => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Account does not belong to this organization"
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Account not found"
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to get account");
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

/// PUT `/organizations/{org_id}/accounts/{account_id}` - Update account.
async fn update_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, account_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateAccountRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let account_repo = AccountRepository::new((*state.db).clone());

    // Verify account belongs to this organization
    match account_repo.find_account_by_id(account_id).await {
        Ok(Some(a)) if a.account.organization_id == org_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Account does not belong to this organization"
                })),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Account not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to find account");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    }

    // Parse account type if provided
    let account_type = payload.account_type.as_ref().and_then(|t| string_to_account_type(t));
    if payload.account_type.is_some() && account_type.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_account_type",
                "message": "Invalid account type. Must be one of: asset, liability, equity, revenue, expense"
            })),
        )
            .into_response();
    }

    // Parse subtype if provided
    let account_subtype = payload.subtype.as_ref().map(|s| string_to_account_subtype(s));

    let input = UpdateAccountInput {
        code: payload.code,
        name: payload.name,
        description: payload.description.map(Some),
        account_type,
        account_subtype,
        parent_id: payload.parent_id.map(Some),
        is_active: payload.is_active,
        allow_direct_posting: payload.allow_direct_posting,
    };

    match account_repo.update_account(account_id, input).await {
        Ok(account) => {
            info!(
                org_id = %org_id,
                account_id = %account_id,
                "Account updated"
            );

            // Get updated balance
            let balance = match account_repo.find_account_by_id(account_id).await {
                Ok(Some(a)) => a.balance.to_string(),
                _ => "0".to_string(),
            };

            (
                StatusCode::OK,
                Json(json!({
                    "id": account.id,
                    "code": account.code,
                    "name": account.name,
                    "description": account.description,
                    "type": account_type_to_string(&account.account_type),
                    "subtype": account.account_subtype.map(|s| account_subtype_to_string(&s)),
                    "parent_id": account.parent_id,
                    "currency": account.currency,
                    "balance": balance,
                    "is_active": account.is_active,
                    "allow_direct_posting": account.allow_direct_posting,
                    "updated_at": account.updated_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to update account");
            match e {
                zeltra_db::repositories::account::AccountError::DuplicateCode(code) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "duplicate_code",
                        "message": format!("Account code '{}' already exists", code)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::account::AccountError::HasLedgerEntries(count) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "has_ledger_entries",
                        "message": format!("Cannot change account type: account has {} ledger entries", count)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::account::AccountError::ParentNotFound(id) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "parent_not_found",
                        "message": format!("Parent account not found: {}", id)
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}


/// DELETE `/organizations/{org_id}/accounts/{account_id}` - Delete (deactivate) account.
async fn delete_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, account_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let account_repo = AccountRepository::new((*state.db).clone());

    // Verify account belongs to this organization
    match account_repo.find_account_by_id(account_id).await {
        Ok(Some(a)) if a.account.organization_id == org_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Account does not belong to this organization"
                })),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Account not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to find account");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    }

    match account_repo.delete_account(account_id).await {
        Ok(()) => {
            info!(
                org_id = %org_id,
                account_id = %account_id,
                "Account deleted (deactivated)"
            );

            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to delete account");
            match e {
                zeltra_db::repositories::account::AccountError::CannotDeleteWithEntries(count) => (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "error": "has_ledger_entries",
                        "message": format!("Cannot delete account: account has {} ledger entries", count)
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}

/// GET `/organizations/{org_id}/accounts/{account_id}/balance` - Get account balance at a specific date.
async fn get_account_balance(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, account_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<BalanceQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let account_repo = AccountRepository::new((*state.db).clone());

    // Verify account belongs to this organization
    let account = match account_repo.find_account_by_id(account_id).await {
        Ok(Some(a)) if a.account.organization_id == org_id => a,
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Account does not belong to this organization"
                })),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Account not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to find account");
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

    // Use provided date or default to today
    let as_of = query.as_of.unwrap_or_else(|| chrono::Utc::now().date_naive());

    match account_repo.get_balance_at_date(account_id, as_of).await {
        Ok(balance) => {
            (
                StatusCode::OK,
                Json(json!({
                    "account_id": account_id,
                    "account_code": account.account.code,
                    "account_name": account.account.name,
                    "currency": account.account.currency,
                    "as_of": as_of.to_string(),
                    "balance": balance.to_string()
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get account balance");
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

/// GET `/organizations/{org_id}/accounts/{account_id}/ledger` - Get ledger entries for an account.
async fn get_account_ledger(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, account_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<LedgerQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let account_repo = AccountRepository::new((*state.db).clone());

    // Verify account belongs to this organization
    match account_repo.find_account_by_id(account_id).await {
        Ok(Some(a)) if a.account.organization_id == org_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "Account does not belong to this organization"
                })),
            )
                .into_response();
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Account not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to find account");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    }

    // Parse pagination with defaults and limits
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).min(100).max(1);

    match account_repo
        .get_ledger_entries(account_id, query.from, query.to, page, limit)
        .await
    {
        Ok(result) => {
            let entries: Vec<LedgerEntryResponse> = result
                .entries
                .into_iter()
                .map(|e| LedgerEntryResponse {
                    id: e.entry.id,
                    transaction_id: e.entry.transaction_id,
                    transaction_date: e.transaction_date.to_string(),
                    reference_number: e.reference_number,
                    description: e.description,
                    status: format!("{:?}", e.status).to_lowercase(),
                    source_currency: e.entry.source_currency,
                    source_amount: e.entry.source_amount.to_string(),
                    exchange_rate: e.entry.exchange_rate.to_string(),
                    functional_currency: e.entry.functional_currency,
                    functional_amount: e.entry.functional_amount.to_string(),
                    debit: e.entry.debit.to_string(),
                    credit: e.entry.credit.to_string(),
                    memo: e.entry.memo,
                    previous_balance: e.entry.account_previous_balance.to_string(),
                    current_balance: e.entry.account_current_balance.to_string(),
                    created_at: e.entry.created_at.to_rfc3339(),
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "entries": entries,
                    "pagination": {
                        "total": result.total,
                        "page": result.page,
                        "limit": result.limit,
                        "total_pages": result.total_pages
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get ledger entries");
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

// Helper functions

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
                "message": "You need admin or owner role to perform this action"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking role");
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

fn account_type_to_string(t: &AccountType) -> String {
    match t {
        AccountType::Asset => "asset".to_string(),
        AccountType::Liability => "liability".to_string(),
        AccountType::Equity => "equity".to_string(),
        AccountType::Revenue => "revenue".to_string(),
        AccountType::Expense => "expense".to_string(),
    }
}

fn string_to_account_type(s: &str) -> Option<AccountType> {
    match s.to_lowercase().as_str() {
        "asset" => Some(AccountType::Asset),
        "liability" => Some(AccountType::Liability),
        "equity" => Some(AccountType::Equity),
        "revenue" => Some(AccountType::Revenue),
        "expense" => Some(AccountType::Expense),
        _ => None,
    }
}

fn account_subtype_to_string(s: &AccountSubtype) -> String {
    match s {
        AccountSubtype::Cash => "cash".to_string(),
        AccountSubtype::Bank => "bank".to_string(),
        AccountSubtype::AccountsReceivable => "accounts_receivable".to_string(),
        AccountSubtype::Inventory => "inventory".to_string(),
        AccountSubtype::Prepaid => "prepaid".to_string(),
        AccountSubtype::FixedAsset => "fixed_asset".to_string(),
        AccountSubtype::AccumulatedDepreciation => "accumulated_depreciation".to_string(),
        AccountSubtype::OtherAsset => "other_asset".to_string(),
        AccountSubtype::AccountsPayable => "accounts_payable".to_string(),
        AccountSubtype::CreditCard => "credit_card".to_string(),
        AccountSubtype::AccruedLiability => "accrued_liability".to_string(),
        AccountSubtype::ShortTermDebt => "short_term_debt".to_string(),
        AccountSubtype::LongTermDebt => "long_term_debt".to_string(),
        AccountSubtype::OtherLiability => "other_liability".to_string(),
        AccountSubtype::OwnerEquity => "owner_equity".to_string(),
        AccountSubtype::RetainedEarnings => "retained_earnings".to_string(),
        AccountSubtype::CommonStock => "common_stock".to_string(),
        AccountSubtype::OtherEquity => "other_equity".to_string(),
        AccountSubtype::OperatingRevenue => "operating_revenue".to_string(),
        AccountSubtype::OtherRevenue => "other_revenue".to_string(),
        AccountSubtype::CostOfGoodsSold => "cost_of_goods_sold".to_string(),
        AccountSubtype::OperatingExpense => "operating_expense".to_string(),
        AccountSubtype::PayrollExpense => "payroll_expense".to_string(),
        AccountSubtype::DepreciationExpense => "depreciation_expense".to_string(),
        AccountSubtype::InterestExpense => "interest_expense".to_string(),
        AccountSubtype::TaxExpense => "tax_expense".to_string(),
        AccountSubtype::OtherExpense => "other_expense".to_string(),
    }
}

fn string_to_account_subtype(s: &str) -> Option<AccountSubtype> {
    match s.to_lowercase().as_str() {
        "cash" => Some(AccountSubtype::Cash),
        "bank" => Some(AccountSubtype::Bank),
        "accounts_receivable" => Some(AccountSubtype::AccountsReceivable),
        "inventory" => Some(AccountSubtype::Inventory),
        "prepaid" => Some(AccountSubtype::Prepaid),
        "fixed_asset" => Some(AccountSubtype::FixedAsset),
        "accumulated_depreciation" => Some(AccountSubtype::AccumulatedDepreciation),
        "other_asset" => Some(AccountSubtype::OtherAsset),
        "accounts_payable" => Some(AccountSubtype::AccountsPayable),
        "credit_card" => Some(AccountSubtype::CreditCard),
        "accrued_liability" => Some(AccountSubtype::AccruedLiability),
        "short_term_debt" => Some(AccountSubtype::ShortTermDebt),
        "long_term_debt" => Some(AccountSubtype::LongTermDebt),
        "other_liability" => Some(AccountSubtype::OtherLiability),
        "owner_equity" => Some(AccountSubtype::OwnerEquity),
        "retained_earnings" => Some(AccountSubtype::RetainedEarnings),
        "common_stock" => Some(AccountSubtype::CommonStock),
        "other_equity" => Some(AccountSubtype::OtherEquity),
        "operating_revenue" => Some(AccountSubtype::OperatingRevenue),
        "other_revenue" => Some(AccountSubtype::OtherRevenue),
        "cost_of_goods_sold" => Some(AccountSubtype::CostOfGoodsSold),
        "operating_expense" => Some(AccountSubtype::OperatingExpense),
        "payroll_expense" => Some(AccountSubtype::PayrollExpense),
        "depreciation_expense" => Some(AccountSubtype::DepreciationExpense),
        "interest_expense" => Some(AccountSubtype::InterestExpense),
        "tax_expense" => Some(AccountSubtype::TaxExpense),
        "other_expense" => Some(AccountSubtype::OtherExpense),
        _ => None,
    }
}
