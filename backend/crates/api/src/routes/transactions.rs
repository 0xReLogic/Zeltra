//! Transaction management routes.
//!
//! Implements Requirements 10.1-10.7 for transaction API endpoints.

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
    repositories::transaction::{
        CreateLedgerEntryInput, CreateTransactionInput, TransactionFilter, TransactionRepository,
    },
};

/// Creates the transaction routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/transactions", get(list_transactions))
        .route("/organizations/{org_id}/transactions", post(create_transaction))
        .route("/organizations/{org_id}/transactions/{transaction_id}", get(get_transaction))
        .route("/organizations/{org_id}/transactions/{transaction_id}", patch(update_transaction))
        .route("/organizations/{org_id}/transactions/{transaction_id}", delete(delete_transaction))
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
        transaction_type: query.transaction_type.as_ref().and_then(|t| string_to_tx_type(t)),
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
