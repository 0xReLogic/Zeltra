//! Transaction management routes.
//!
//! Implements Requirements 10.1-10.7 for transaction API endpoints.
//! Implements Requirements 6.1-6.7 for workflow API endpoints.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::{TransactionStatus, TransactionType},
    repositories::WorkflowRepository,
    repositories::transaction::{
        CreateLedgerEntryInput, CreateTransactionInput, TransactionFilter, TransactionRepository,
    },
};

/// Creates the transaction routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/transactions",
            get(list_transactions),
        )
        .route(
            "/organizations/{org_id}/transactions",
            post(create_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/pending",
            get(get_pending_transactions),
        )
        .route(
            "/organizations/{org_id}/transactions/bulk-approve",
            post(bulk_approve_transactions),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}",
            get(get_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}",
            patch(update_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}",
            delete(delete_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/submit",
            post(submit_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/approve",
            post(approve_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/reject",
            post(reject_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/post",
            post(post_transaction),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/void",
            post(void_transaction),
        )
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for listing transactions.
#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    /// Filter by status.
    pub status: Option<String>,
    /// Filter by transaction type.
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    /// Filter by date range start (YYYY-MM-DD).
    pub from: Option<NaiveDate>,
    /// Filter by date range end (YYYY-MM-DD).
    pub to: Option<NaiveDate>,
    /// Filter by dimension value ID.
    pub dimension: Option<Uuid>,
    /// Page number (1-indexed).
    pub page: Option<u64>,
    /// Page size (default: 50, max: 100).
    pub limit: Option<u64>,
}

/// Request body for creating a transaction.
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    /// Transaction type.
    #[serde(rename = "type")]
    pub transaction_type: String,
    /// Transaction date (YYYY-MM-DD).
    pub transaction_date: NaiveDate,
    /// Description.
    pub description: String,
    /// Optional reference number.
    pub reference_number: Option<String>,
    /// Optional memo.
    pub memo: Option<String>,
    /// Ledger entries.
    pub entries: Vec<CreateEntryRequest>,
}

/// Request body for a single ledger entry.
#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    /// Account ID.
    pub account_id: Uuid,
    /// Source currency code.
    pub source_currency: String,
    /// Source amount (positive).
    pub source_amount: String,
    /// Entry type: "debit" or "credit".
    pub entry_type: String,
    /// Optional memo.
    pub memo: Option<String>,
    /// Dimension value IDs.
    #[serde(default)]
    pub dimensions: Vec<Uuid>,
}

/// Request body for updating a transaction.
#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    /// Description.
    pub description: Option<String>,
    /// Memo.
    pub memo: Option<String>,
    /// Reference number.
    pub reference_number: Option<String>,
}

/// Response for a transaction.
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    /// Transaction ID.
    pub id: Uuid,
    /// Reference number.
    pub reference_number: Option<String>,
    /// Transaction type.
    #[serde(rename = "type")]
    pub transaction_type: String,
    /// Transaction date.
    pub transaction_date: String,
    /// Description.
    pub description: String,
    /// Memo.
    pub memo: Option<String>,
    /// Status.
    pub status: String,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Created by user ID.
    pub created_by: Uuid,
    /// Created at timestamp.
    pub created_at: String,
    /// Updated at timestamp.
    pub updated_at: String,
    /// Ledger entries.
    pub entries: Vec<EntryResponse>,
    /// Total debits in functional currency.
    pub total_debit: String,
    /// Total credits in functional currency.
    pub total_credit: String,
}

/// Response for a ledger entry.
#[derive(Debug, Serialize)]
pub struct EntryResponse {
    /// Entry ID.
    pub id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
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
    /// Memo.
    pub memo: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Response for transaction list item (without entries).
#[derive(Debug, Serialize)]
pub struct TransactionListItem {
    /// Transaction ID.
    pub id: Uuid,
    /// Reference number.
    pub reference_number: Option<String>,
    /// Transaction type.
    #[serde(rename = "type")]
    pub transaction_type: String,
    /// Transaction date.
    pub transaction_date: String,
    /// Description.
    pub description: String,
    /// Status.
    pub status: String,
    /// Created at timestamp.
    pub created_at: String,
}

// ============================================================================
// Workflow Request/Response Types
// ============================================================================

/// Request body for approving a transaction.
#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    /// Optional approval notes.
    pub approval_notes: Option<String>,
}

/// Request body for rejecting a transaction.
#[derive(Debug, Deserialize)]
pub struct RejectRequest {
    /// Rejection reason (required).
    pub reason: String,
}

/// Request body for voiding a transaction.
#[derive(Debug, Deserialize)]
pub struct VoidRequest {
    /// Void reason (required).
    pub reason: String,
}

/// Request body for bulk approval.
#[derive(Debug, Deserialize)]
pub struct BulkApproveRequest {
    /// Transaction IDs to approve.
    pub transaction_ids: Vec<Uuid>,
    /// Optional approval notes.
    pub approval_notes: Option<String>,
}

/// Response for void operation.
#[derive(Debug, Serialize)]
pub struct VoidResponse {
    /// Original transaction (now voided).
    pub original_transaction: TransactionResponse,
    /// Reversing transaction (posted).
    pub reversing_transaction: TransactionResponse,
}

/// Response for bulk approval.
#[derive(Debug, Serialize)]
pub struct BulkApproveResponse {
    /// Results for each transaction.
    pub results: Vec<BulkApproveItemResponse>,
    /// Number of successful approvals.
    pub success_count: usize,
    /// Number of failed approvals.
    pub failure_count: usize,
}

/// Response for a single bulk approval item.
#[derive(Debug, Serialize)]
pub struct BulkApproveItemResponse {
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Whether the approval succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Response for pending transaction in approval queue.
#[derive(Debug, Serialize)]
pub struct PendingTransactionResponse {
    /// Transaction ID.
    pub id: Uuid,
    /// Reference number.
    pub reference_number: Option<String>,
    /// Transaction type.
    #[serde(rename = "type")]
    pub transaction_type: String,
    /// Transaction date.
    pub transaction_date: String,
    /// Description.
    pub description: String,
    /// Status.
    pub status: String,
    /// Total amount.
    pub total_amount: String,
    /// Submitted at timestamp.
    pub submitted_at: Option<String>,
    /// Whether the current user can approve this transaction.
    pub can_approve: bool,
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET `/organizations/{org_id}/transactions` - List transactions with filters.
///
/// Requirements: 10.2
async fn list_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Query(query): Query<ListTransactionsQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let tx_repo = TransactionRepository::new((*state.db).clone());

    // Build filter
    let filter = TransactionFilter {
        status: query.status.as_ref().and_then(|s| string_to_status(s)),
        transaction_type: query
            .transaction_type
            .as_ref()
            .and_then(|t| string_to_tx_type(t)),
        date_from: query.from,
        date_to: query.to,
        dimension_value_id: query.dimension,
    };

    match tx_repo.list_transactions(org_id, filter).await {
        Ok(transactions) => {
            let items: Vec<TransactionListItem> = transactions
                .into_iter()
                .map(|t| TransactionListItem {
                    id: t.id,
                    reference_number: t.reference_number,
                    transaction_type: tx_type_to_string(&t.transaction_type),
                    transaction_date: t.transaction_date.to_string(),
                    description: t.description,
                    status: status_to_string(&t.status),
                    created_at: t.created_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(json!({ "transactions": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list transactions");
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

/// POST `/organizations/{org_id}/transactions` - Create a new transaction.
///
/// Requirements: 10.1
#[allow(clippy::too_many_lines)]
async fn create_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Parse transaction type
    let Some(transaction_type) = string_to_tx_type(&payload.transaction_type) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_transaction_type",
                "message": "Invalid transaction type"
            })),
        )
            .into_response();
    };

    // Validate minimum entries
    if payload.entries.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "insufficient_entries",
                "message": "Transaction must have at least 2 entries"
            })),
        )
            .into_response();
    }

    // Get organization's base currency
    let org = match org_repo.find_by_id(org_id).await {
        Ok(Some(o)) => o,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "organization_not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to get organization");
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

    let functional_currency = org.base_currency;

    // Parse and resolve entries
    let mut entries = Vec::with_capacity(payload.entries.len());
    let mut total_debit = Decimal::ZERO;
    let mut total_credit = Decimal::ZERO;

    for entry_req in &payload.entries {
        // Parse source amount
        let source_amount = match Decimal::from_str(&entry_req.source_amount) {
            Ok(a) if a > Decimal::ZERO => a,
            Ok(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_amount",
                        "message": "Amount must be positive"
                    })),
                )
                    .into_response();
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_amount",
                        "message": "Invalid amount format"
                    })),
                )
                    .into_response();
            }
        };

        // Get exchange rate (simplified - same currency = 1.0)
        // TODO: Lookup from exchange_rates table for different currencies
        let exchange_rate = if entry_req.source_currency == functional_currency {
            Decimal::ONE
        } else {
            // For now, require same currency or return error
            // In production, lookup from ExchangeRateRepository
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "no_exchange_rate",
                    "message": format!("No exchange rate found for {} to {}", entry_req.source_currency, functional_currency)
                })),
            )
                .into_response();
        };

        let functional_amount = source_amount * exchange_rate;

        // Determine debit/credit
        let (debit, credit) = match entry_req.entry_type.to_lowercase().as_str() {
            "debit" => (functional_amount, Decimal::ZERO),
            "credit" => (Decimal::ZERO, functional_amount),
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_entry_type",
                        "message": "Entry type must be 'debit' or 'credit'"
                    })),
                )
                    .into_response();
            }
        };

        total_debit += debit;
        total_credit += credit;

        entries.push(CreateLedgerEntryInput {
            account_id: entry_req.account_id,
            source_currency: entry_req.source_currency.clone(),
            source_amount,
            exchange_rate,
            functional_currency: functional_currency.clone(),
            functional_amount,
            debit,
            credit,
            memo: entry_req.memo.clone(),
            dimensions: entry_req.dimensions.clone(),
        });
    }

    // Validate balance (debits must equal credits)
    if total_debit != total_credit {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "unbalanced_transaction",
                "message": format!("Transaction is not balanced. Debit: {}, Credit: {}", total_debit, total_credit)
            })),
        )
            .into_response();
    }

    let tx_repo = TransactionRepository::new((*state.db).clone());

    let input = CreateTransactionInput {
        organization_id: org_id,
        transaction_type,
        transaction_date: payload.transaction_date,
        description: payload.description,
        reference_number: payload.reference_number,
        memo: payload.memo,
        entries,
        created_by: auth.user_id(),
    };

    match tx_repo.create_transaction(input).await {
        Ok(result) => {
            info!(
                org_id = %org_id,
                transaction_id = %result.transaction.id,
                "Transaction created"
            );

            let entry_responses: Vec<EntryResponse> = result
                .entries
                .into_iter()
                .map(|e| EntryResponse {
                    id: e.entry.id,
                    account_id: e.entry.account_id,
                    source_currency: e.entry.source_currency,
                    source_amount: e.entry.source_amount.to_string(),
                    exchange_rate: e.entry.exchange_rate.to_string(),
                    functional_currency: e.entry.functional_currency,
                    functional_amount: e.entry.functional_amount.to_string(),
                    debit: e.entry.debit.to_string(),
                    credit: e.entry.credit.to_string(),
                    memo: e.entry.memo,
                    dimensions: e.dimensions,
                })
                .collect();

            let response = TransactionResponse {
                id: result.transaction.id,
                reference_number: result.transaction.reference_number,
                transaction_type: tx_type_to_string(&result.transaction.transaction_type),
                transaction_date: result.transaction.transaction_date.to_string(),
                description: result.transaction.description,
                memo: result.transaction.memo,
                status: status_to_string(&result.transaction.status),
                fiscal_period_id: result.transaction.fiscal_period_id,
                created_by: result.transaction.created_by,
                created_at: result.transaction.created_at.to_rfc3339(),
                updated_at: result.transaction.updated_at.to_rfc3339(),
                entries: entry_responses,
                total_debit: total_debit.to_string(),
                total_credit: total_credit.to_string(),
            };

            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create transaction");
            match e {
                zeltra_db::repositories::transaction::TransactionError::NoFiscalPeriod(date) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "no_fiscal_period",
                        "message": format!("No fiscal period found for date {}", date)
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::transaction::TransactionError::PeriodClosed => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "period_closed",
                        "message": "Fiscal period is closed, no posting allowed"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::transaction::TransactionError::AccountNotFound(id) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "account_not_found",
                        "message": format!("Account not found: {}", id)
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

/// GET `/organizations/{org_id}/transactions/{transaction_id}` - Get transaction with entries.
///
/// Requirements: 10.3
async fn get_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let tx_repo = TransactionRepository::new((*state.db).clone());

    match tx_repo.get_transaction(org_id, transaction_id).await {
        Ok(result) => {
            // Calculate totals
            let total_debit: Decimal = result.entries.iter().map(|e| e.entry.debit).sum();
            let total_credit: Decimal = result.entries.iter().map(|e| e.entry.credit).sum();

            let entry_responses: Vec<EntryResponse> = result
                .entries
                .into_iter()
                .map(|e| EntryResponse {
                    id: e.entry.id,
                    account_id: e.entry.account_id,
                    source_currency: e.entry.source_currency,
                    source_amount: e.entry.source_amount.to_string(),
                    exchange_rate: e.entry.exchange_rate.to_string(),
                    functional_currency: e.entry.functional_currency,
                    functional_amount: e.entry.functional_amount.to_string(),
                    debit: e.entry.debit.to_string(),
                    credit: e.entry.credit.to_string(),
                    memo: e.entry.memo,
                    dimensions: e.dimensions,
                })
                .collect();

            let response = TransactionResponse {
                id: result.transaction.id,
                reference_number: result.transaction.reference_number,
                transaction_type: tx_type_to_string(&result.transaction.transaction_type),
                transaction_date: result.transaction.transaction_date.to_string(),
                description: result.transaction.description,
                memo: result.transaction.memo,
                status: status_to_string(&result.transaction.status),
                fiscal_period_id: result.transaction.fiscal_period_id,
                created_by: result.transaction.created_by,
                created_at: result.transaction.created_at.to_rfc3339(),
                updated_at: result.transaction.updated_at.to_rfc3339(),
                entries: entry_responses,
                total_debit: total_debit.to_string(),
                total_credit: total_credit.to_string(),
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get transaction");
            match e {
                zeltra_db::repositories::transaction::TransactionError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "not_found",
                        "message": "Transaction not found"
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

/// PATCH `/organizations/{org_id}/transactions/{transaction_id}` - Update draft transaction.
///
/// Requirements: 10.4, 10.5
async fn update_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let tx_repo = TransactionRepository::new((*state.db).clone());

    match tx_repo
        .update_transaction(
            org_id,
            transaction_id,
            payload.description,
            payload.memo,
            payload.reference_number,
        )
        .await
    {
        Ok(transaction) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                "Transaction updated"
            );

            (
                StatusCode::OK,
                Json(json!({
                    "id": transaction.id,
                    "reference_number": transaction.reference_number,
                    "type": tx_type_to_string(&transaction.transaction_type),
                    "transaction_date": transaction.transaction_date.to_string(),
                    "description": transaction.description,
                    "memo": transaction.memo,
                    "status": status_to_string(&transaction.status),
                    "updated_at": transaction.updated_at.to_rfc3339()
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to update transaction");
            match e {
                zeltra_db::repositories::transaction::TransactionError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "not_found",
                        "message": "Transaction not found"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::transaction::TransactionError::CannotModifyPosted => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "cannot_modify_posted",
                        "message": "Cannot modify posted transaction"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::transaction::TransactionError::CannotModifyVoided => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "cannot_modify_voided",
                        "message": "Cannot modify voided transaction"
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

/// DELETE `/organizations/{org_id}/transactions/{transaction_id}` - Delete draft transaction.
///
/// Requirements: 10.6, 10.7
async fn delete_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let tx_repo = TransactionRepository::new((*state.db).clone());

    match tx_repo.delete_transaction(org_id, transaction_id).await {
        Ok(()) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                "Transaction deleted"
            );

            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to delete transaction");
            match e {
                zeltra_db::repositories::transaction::TransactionError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "not_found",
                        "message": "Transaction not found"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::transaction::TransactionError::CanOnlyDeleteDraft => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "can_only_delete_draft",
                        "message": "Can only delete draft transactions"
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

// ============================================================================
// Workflow Route Handlers
// ============================================================================

/// POST `/organizations/{org_id}/transactions/{transaction_id}/submit` - Submit for approval.
///
/// Requirements: 6.1
async fn submit_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .submit_transaction(org_id, transaction_id, auth.user_id())
        .await
    {
        Ok(transaction) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                "Transaction submitted for approval"
            );

            let submitted_at = transaction
                .submitted_at
                .as_ref()
                .map(chrono::DateTime::to_rfc3339);

            (
                StatusCode::OK,
                Json(json!({
                    "id": transaction.id,
                    "status": status_to_string(&transaction.status),
                    "submitted_at": submitted_at,
                    "submitted_by": transaction.submitted_by
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to submit transaction");
            workflow_error_response(e)
        }
    }
}

/// POST `/organizations/{org_id}/transactions/{transaction_id}/approve` - Approve transaction.
///
/// Requirements: 6.2
async fn approve_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
    payload: Option<Json<ApproveRequest>>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let approval_notes = payload.and_then(|p| p.approval_notes.clone());
    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .approve_transaction(org_id, transaction_id, auth.user_id(), approval_notes)
        .await
    {
        Ok(transaction) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                "Transaction approved"
            );

            let approved_at = transaction
                .approved_at
                .as_ref()
                .map(chrono::DateTime::to_rfc3339);

            (
                StatusCode::OK,
                Json(json!({
                    "id": transaction.id,
                    "status": status_to_string(&transaction.status),
                    "approved_at": approved_at,
                    "approved_by": transaction.approved_by,
                    "approval_notes": transaction.approval_notes
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to approve transaction");
            workflow_error_response(e)
        }
    }
}

/// POST `/organizations/{org_id}/transactions/{transaction_id}/reject` - Reject transaction.
///
/// Requirements: 6.3
async fn reject_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<RejectRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if payload.reason.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "rejection_reason_required",
                "message": "Rejection reason is required"
            })),
        )
            .into_response();
    }

    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .reject_transaction(org_id, transaction_id, payload.reason)
        .await
    {
        Ok(transaction) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                "Transaction rejected"
            );

            (
                StatusCode::OK,
                Json(json!({
                    "id": transaction.id,
                    "status": status_to_string(&transaction.status),
                    "approval_notes": transaction.approval_notes
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to reject transaction");
            workflow_error_response(e)
        }
    }
}

/// POST `/organizations/{org_id}/transactions/{transaction_id}/post` - Post to ledger.
///
/// Requirements: 6.4
async fn post_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .post_transaction(org_id, transaction_id, auth.user_id())
        .await
    {
        Ok(transaction) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                "Transaction posted"
            );

            let posted_at = transaction
                .posted_at
                .as_ref()
                .map(chrono::DateTime::to_rfc3339);

            (
                StatusCode::OK,
                Json(json!({
                    "id": transaction.id,
                    "status": status_to_string(&transaction.status),
                    "posted_at": posted_at,
                    "posted_by": transaction.posted_by
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to post transaction");
            workflow_error_response(e)
        }
    }
}

/// POST `/organizations/{org_id}/transactions/{transaction_id}/void` - Void transaction.
///
/// Requirements: 6.5
async fn void_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<VoidRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if payload.reason.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "void_reason_required",
                "message": "Void reason is required"
            })),
        )
            .into_response();
    }

    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .void_transaction(org_id, transaction_id, auth.user_id(), payload.reason)
        .await
    {
        Ok(result) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                reversing_id = %result.reversing_transaction.id,
                "Transaction voided"
            );

            let voided_at = result
                .original_transaction
                .voided_at
                .as_ref()
                .map(chrono::DateTime::to_rfc3339);

            (
                StatusCode::OK,
                Json(json!({
                    "original_transaction": {
                        "id": result.original_transaction.id,
                        "status": status_to_string(&result.original_transaction.status),
                        "voided_at": voided_at,
                        "voided_by": result.original_transaction.voided_by,
                        "void_reason": result.original_transaction.void_reason,
                        "reversed_by_transaction_id": result.original_transaction.reversed_by_transaction_id
                    },
                    "reversing_transaction": {
                        "id": result.reversing_transaction.id,
                        "status": status_to_string(&result.reversing_transaction.status),
                        "transaction_type": tx_type_to_string(&result.reversing_transaction.transaction_type),
                        "reverses_transaction_id": result.reversing_transaction.reverses_transaction_id
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to void transaction");
            workflow_error_response(e)
        }
    }
}

/// GET `/organizations/{org_id}/transactions/pending` - Get pending transactions.
///
/// Requirements: 6.6
async fn get_pending_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .get_pending_transactions(org_id, auth.user_id())
        .await
    {
        Ok(pending) => {
            let items: Vec<PendingTransactionResponse> = pending
                .into_iter()
                .map(|p| {
                    let submitted_at = p
                        .transaction
                        .submitted_at
                        .as_ref()
                        .map(chrono::DateTime::to_rfc3339);
                    PendingTransactionResponse {
                        id: p.transaction.id,
                        reference_number: p.transaction.reference_number,
                        transaction_type: tx_type_to_string(&p.transaction.transaction_type),
                        transaction_date: p.transaction.transaction_date.to_string(),
                        description: p.transaction.description,
                        status: status_to_string(&p.transaction.status),
                        total_amount: "0.0000".to_string(), // TODO: Calculate from entries
                        submitted_at,
                        can_approve: p.can_approve,
                    }
                })
                .collect();

            (StatusCode::OK, Json(json!({ "data": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get pending transactions");
            workflow_error_response(e)
        }
    }
}

/// POST `/organizations/{org_id}/transactions/bulk-approve` - Bulk approve transactions.
///
/// Requirements: 6.7
async fn bulk_approve_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<BulkApproveRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if payload.transaction_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "empty_transaction_ids",
                "message": "At least one transaction ID is required"
            })),
        )
            .into_response();
    }

    if payload.transaction_ids.len() > 50 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "too_many_transactions",
                "message": "Maximum 50 transactions per bulk approval"
            })),
        )
            .into_response();
    }

    let workflow_repo = WorkflowRepository::new((*state.db).clone());

    match workflow_repo
        .bulk_approve(
            org_id,
            payload.transaction_ids,
            auth.user_id(),
            payload.approval_notes,
        )
        .await
    {
        Ok(result) => {
            info!(
                org_id = %org_id,
                success_count = result.success_count,
                failure_count = result.failure_count,
                "Bulk approval completed"
            );

            let response = BulkApproveResponse {
                results: result
                    .results
                    .into_iter()
                    .map(|r| BulkApproveItemResponse {
                        transaction_id: r.transaction_id,
                        success: r.success,
                        error: r.error,
                    })
                    .collect(),
                success_count: result.success_count,
                failure_count: result.failure_count,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to bulk approve transactions");
            workflow_error_response(e)
        }
    }
}

/// Convert WorkflowError to HTTP response.
fn workflow_error_response(e: zeltra_core::workflow::WorkflowError) -> axum::response::Response {
    use zeltra_core::workflow::WorkflowError;

    match e {
        WorkflowError::InvalidTransition { from, to } => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_transition",
                "message": format!("Invalid status transition from {:?} to {:?}", from, to)
            })),
        )
            .into_response(),
        WorkflowError::TransactionNotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Transaction not found"
            })),
        )
            .into_response(),
        WorkflowError::NotAuthorizedToApprove => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "not_authorized",
                "message": "Not authorized to approve this transaction"
            })),
        )
            .into_response(),
        WorkflowError::ExceedsApprovalLimit { amount, limit } => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "exceeds_approval_limit",
                "message": format!("Transaction amount {} exceeds approval limit {}", amount, limit)
            })),
        )
            .into_response(),
        WorkflowError::InsufficientRole {
            user_role,
            required_role,
        } => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "insufficient_role",
                "message": format!("Role {} does not meet required role {}", user_role, required_role)
            })),
        )
            .into_response(),
        WorkflowError::VoidReasonRequired => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "void_reason_required",
                "message": "Void reason is required"
            })),
        )
            .into_response(),
        WorkflowError::RejectionReasonRequired => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "rejection_reason_required",
                "message": "Rejection reason is required"
            })),
        )
            .into_response(),
        WorkflowError::CannotModifyPosted => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "cannot_modify_posted",
                "message": "Cannot modify posted transaction"
            })),
        )
            .into_response(),
        WorkflowError::CannotModifyVoided => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "cannot_modify_voided",
                "message": "Cannot modify voided transaction"
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

// ============================================================================
// Helper Functions
// ============================================================================

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

fn status_to_string(status: &TransactionStatus) -> String {
    match status {
        TransactionStatus::Draft => "draft".to_string(),
        TransactionStatus::Pending => "pending".to_string(),
        TransactionStatus::Approved => "approved".to_string(),
        TransactionStatus::Posted => "posted".to_string(),
        TransactionStatus::Voided => "voided".to_string(),
    }
}

fn string_to_status(s: &str) -> Option<TransactionStatus> {
    match s.to_lowercase().as_str() {
        "draft" => Some(TransactionStatus::Draft),
        "pending" => Some(TransactionStatus::Pending),
        "approved" => Some(TransactionStatus::Approved),
        "posted" => Some(TransactionStatus::Posted),
        "voided" => Some(TransactionStatus::Voided),
        _ => None,
    }
}

fn tx_type_to_string(tx_type: &TransactionType) -> String {
    match tx_type {
        TransactionType::Journal => "journal".to_string(),
        TransactionType::Expense => "expense".to_string(),
        TransactionType::Invoice => "invoice".to_string(),
        TransactionType::Bill => "bill".to_string(),
        TransactionType::Payment => "payment".to_string(),
        TransactionType::Transfer => "transfer".to_string(),
        TransactionType::Adjustment => "adjustment".to_string(),
        TransactionType::OpeningBalance => "opening_balance".to_string(),
        TransactionType::Reversal => "reversal".to_string(),
    }
}

fn string_to_tx_type(s: &str) -> Option<TransactionType> {
    match s.to_lowercase().as_str() {
        "journal" => Some(TransactionType::Journal),
        "expense" => Some(TransactionType::Expense),
        "invoice" => Some(TransactionType::Invoice),
        "bill" => Some(TransactionType::Bill),
        "payment" => Some(TransactionType::Payment),
        "transfer" => Some(TransactionType::Transfer),
        "adjustment" => Some(TransactionType::Adjustment),
        "opening_balance" => Some(TransactionType::OpeningBalance),
        "reversal" => Some(TransactionType::Reversal),
        _ => None,
    }
}
