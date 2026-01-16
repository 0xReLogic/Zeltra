//! Transaction management routes.
//!
//! Implements Requirements 10.1-10.7 for transaction API endpoints.
//! Implements Requirements 6.1-6.7 for workflow API endpoints.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
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
use zeltra_core::currency::calculate_forex_variance;
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::{TransactionStatus, TransactionType},
    repositories::WorkflowRepository,
    repositories::transaction::{
        CreateLedgerEntryInput, CreateTransactionInput, TransactionFilter, TransactionRepository,
        TransactionWithEntries,
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
        .route(
            "/organizations/{org_id}/transactions/pay-invoice",
            post(pay_invoice),
        )
}

/// POST `/organizations/{org_id}/transactions/pay-invoice` - Pay an invoice with auto-forex variance.
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/pay-invoice",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = PayInvoiceRequest,
    responses(
        (status = 201, description = "Payment transaction created", body = TransactionResponse),
        (status = 400, description = "Invalid input or processing error"),
        (status = 404, description = "Invoice not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
async fn pay_invoice(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<PayInvoiceRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());
    let tx_repo = TransactionRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Get Organization for functional currency
    let Ok(Some(org)) = org_repo.find_by_id(org_id).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "org_not_found"})),
        )
            .into_response();
    };
    let functional_currency = org.base_currency;

    // 1. Fetch Invoice Details
    let (invoice, original_rate, original_currency, ar_ap_account_id): (
        TransactionWithEntries,
        Decimal,
        String,
        Uuid,
    ) = match fetch_invoice_details(&tx_repo, org_id, payload.invoice_id, &functional_currency)
        .await
    {
        Ok(details) => details,
        Err(e) => return e.into_response(),
    };

    // 2. Calculate Variance
    let variance = calculate_forex_variance(payload.amount, original_rate, payload.exchange_rate);

    // 3. Construct Entries
    let entries = construct_payment_entries(
        &payload,
        original_rate,
        &original_currency,
        ar_ap_account_id,
        &functional_currency,
        variance,
    );

    // 4. Create Transaction
    let input = CreateTransactionInput {
        organization_id: org_id,
        transaction_type: TransactionType::Payment,
        transaction_date: payload.payment_date,
        description: payload
            .description
            .unwrap_or_else(|| "Invoice Payment".to_string()),
        reference_number: invoice
            .transaction
            .reference_number
            .map(|r| format!("{r}-PAY")),
        memo: None,
        entries,
        created_by: auth.user_id(),
        timezone: payload.timezone.clone(),
        idempotency_key: payload.idempotency_key,
        iso_metadata: payload.iso_metadata.clone(),
    };

    match tx_repo.create_transaction(input).await {
        Ok(result) => {
            info!(
                org_id = %org_id,
                transaction_id = %result.transaction.id,
                "Invoice payment transaction created"
            );

            let response = map_transaction_to_response(result);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create payment transaction");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "creation_failed"})),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for listing transactions.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
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
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTransactionRequest {
    /// Transaction type.
    #[serde(rename = "type")]
    #[schema(example = "journal")]
    pub transaction_type: String,
    /// Transaction date (YYYY-MM-DD).
    pub transaction_date: NaiveDate,
    /// Description.
    #[schema(example = "Monthly rent payment")]
    pub description: String,
    /// Optional reference number.
    #[schema(example = "TX-1001")]
    pub reference_number: Option<String>,
    /// Optional memo.
    pub memo: Option<String>,
    /// Ledger entries.
    pub entries: Vec<CreateEntryRequest>,
    /// Timezone for the transaction (e.g., "UTC", "Asia/Jakarta").
    #[schema(example = "UTC")]
    pub timezone: String,
    /// Optional idempotency key for preventing duplicate transactions.
    pub idempotency_key: Option<Uuid>,
    /// Optional ISO 20022 metadata.
    pub iso_metadata: Option<serde_json::Value>,
}

/// Request body for a single ledger entry.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEntryRequest {
    /// Account ID.
    pub account_id: Uuid,
    /// Source currency code.
    #[schema(example = "USD")]
    pub source_currency: String,
    /// Source amount (positive).
    #[schema(example = "100.00")]
    pub source_amount: String,
    /// Entry type: "debit" or "credit".
    #[schema(example = "debit")]
    pub entry_type: String,
    /// Optional memo.
    pub memo: Option<String>,
    /// Dimension value IDs.
    #[serde(default)]
    pub dimensions: Vec<Uuid>,
    /// Optional exchange rate override.
    #[schema(example = "1.0850")]
    pub exchange_rate: Option<String>,
    /// Optional compliance/ESG metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Request body for updating a transaction.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTransactionRequest {
    /// Description.
    pub description: Option<String>,
    /// Memo.
    pub memo: Option<String>,
    /// Reference number.
    pub reference_number: Option<String>,
}

/// Request body for paying an invoice.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct PayInvoiceRequest {
    /// ID of the invoice (transaction) being paid.
    pub invoice_id: Uuid,
    /// Account ID to pay from (e.g., Bank).
    pub payment_account_id: Uuid,
    /// Amount to pay (in source currency).
    #[schema(example = "100.00")]
    pub amount: Decimal,
    /// Exchange rate for the payment.
    #[schema(example = "1.0850")]
    pub exchange_rate: Decimal,
    /// Payment date.
    pub payment_date: NaiveDate,
    /// Account ID for Realized Gain/Loss.
    pub gain_loss_account_id: Uuid,
    /// Optional description.
    pub description: Option<String>,
    /// Timezone for the transaction.
    #[schema(example = "UTC")]
    pub timezone: String,
    /// Optional idempotency key.
    pub idempotency_key: Option<Uuid>,
    /// Optional ISO 20022 metadata.
    pub iso_metadata: Option<serde_json::Value>,
}

/// Pagination metadata.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationMeta {
    /// Current page number (0-indexed).
    pub page: u64,
    /// Items per page.
    pub limit: u64,
    /// Total number of items.
    pub total: u64,
}

/// Transaction list item (lightweight).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TransactionListItem {
    /// Transaction ID.
    pub id: Uuid,
    /// Reference number.
    pub reference_number: Option<String>,
    /// Transaction type.
    pub transaction_type: String,
    /// Transaction date.
    pub transaction_date: String,
    /// Description.
    pub description: String,
    /// Status.
    pub status: String,
    /// Created at timestamp.
    pub created_at: String,
    /// Total amount.
    pub total_amount: String,
}

/// Response for a paginated list of transactions.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginatedTransactionsResponse {
    /// List of transaction items.
    pub transactions: Vec<TransactionListItem>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

/// Response for a transaction.
#[derive(Debug, Serialize, utoipa::ToSchema)]
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
    /// Timezone for the transaction.
    #[schema(example = "UTC")]
    pub timezone: String,
    /// Optional idempotency key.
    pub idempotency_key: Option<Uuid>,
    /// Optional ISO 20022 metadata.
    pub iso_metadata: Option<serde_json::Value>,
}

/// Response for a ledger entry.
#[derive(Debug, Serialize, utoipa::ToSchema)]
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


// ============================================================================
// Workflow Request/Response Types
// ============================================================================

/// Request body for approving a transaction.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveRequest {
    /// Optional approval notes.
    pub approval_notes: Option<String>,
}

/// Request body for rejecting a transaction.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RejectRequest {
    /// Rejection reason (required).
    pub reason: String,
}

/// Request body for voiding a transaction.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct VoidRequest {
    /// Void reason (required).
    pub reason: String,
}

/// Request body for bulk approval.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BulkApproveRequest {
    /// Transaction IDs to approve.
    pub transaction_ids: Vec<Uuid>,
    /// Optional approval notes.
    pub approval_notes: Option<String>,
}

/// Response for void operation.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VoidResponse {
    /// Original transaction (now voided).
    pub original_transaction: TransactionResponse,
    /// Reversing transaction (posted).
    pub reversing_transaction: TransactionResponse,
}

/// Response for bulk approval.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BulkApproveResponse {
    /// Results for each transaction.
    pub results: Vec<BulkApproveItemResponse>,
    /// Number of successful approvals.
    pub success_count: usize,
    /// Number of failed approvals.
    pub failure_count: usize,
}

/// Response for a single bulk approval item.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BulkApproveItemResponse {
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Whether the approval succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Response for pending transaction in approval queue.
#[derive(Debug, Serialize, utoipa::ToSchema)]
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

/// GET `/organizations/{org_id}/transactions` - List transactions.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/transactions",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ListTransactionsQuery
    ),
    responses(
        (status = 200, description = "Transactions retrieved successfully", body = PaginatedTransactionsResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Transactions",
    security(
        ("bearerAuth" = [])
    )
)]
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

    let page = query.page.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    match tx_repo.list_transactions(org_id, filter, page, limit).await {
        Ok((transactions, total)) => {
            let items: Vec<TransactionListItem> = transactions
                .into_iter()
                .map(|t| {
                    let total_debit: Decimal = t.entries.iter().map(|e| e.entry.debit).sum();

                    TransactionListItem {
                        id: t.transaction.id,
                        reference_number: t.transaction.reference_number,
                        transaction_type: tx_type_to_string(&t.transaction.transaction_type),
                        transaction_date: t.transaction.transaction_date.to_string(),
                        description: t.transaction.description,
                        status: status_to_string(&t.transaction.status),
                        created_at: t.transaction.created_at.to_rfc3339(),
                        total_amount: total_debit.to_string(),
                    }
                })
                .collect();

            let response = PaginatedTransactionsResponse {
                transactions: items,
                pagination: PaginationMeta {
                    page,
                    limit,
                    total,
                },
            };

            (StatusCode::OK, Json(response)).into_response()
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

/// Checks if the organization has reached its monthly transaction limit.
fn check_monthly_transaction_limit(
    max_tx: Option<i32>,
    current_count: u64,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if let Some(max) = max_tx {
        let max_u64 = u64::try_from(max).unwrap_or(u64::MAX);
        if current_count >= max_u64 {
            return Err((
                StatusCode::PAYMENT_REQUIRED,
                Json(json!({
                    "error": "tier_limit_reached",
                    "message": format!("You have reached the maximum transactions ({}) for this month. Please upgrade your plan.", max),
                    "limit": max,
                    "current": current_count
                })),
            ));
        }
    }
    Ok(())
}

/// POST `/organizations/{org_id}/transactions` - Create a new transaction.
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateTransactionRequest,
    responses(
        (status = 201, description = "Transaction created successfully", body = TransactionResponse),
        (status = 400, description = "Invalid input, unbalanced transaction, or missing required budget dimensions"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
#[allow(clippy::too_many_lines)]
async fn create_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateTransactionRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Check tier limits - Max Transactions
    if let Ok(Some(limits)) = org_repo.get_tier_limits(org_id).await {
        let txn_repo = TransactionRepository::new((*state.db).clone());
        let current_tx_count = match txn_repo.count_monthly_usage(org_id).await {
            Ok(count) => count,
            Err(e) => {
                error!(error = %e, "Database error counting transactions");
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

        if let Err(e) =
            check_monthly_transaction_limit(limits.max_transactions_per_month, current_tx_count)
        {
            return e.into_response();
        }
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

        // Get exchange rate logic with Override support
        // Item 43: Advanced Transaction Fields (Manual Rate Override)
        let exchange_rate = if let Some(rate_str) = &entry_req.exchange_rate {
             match Decimal::from_str(rate_str) {
                Ok(rate) => rate,
                Err(_) => return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_exchange_rate",
                        "message": "Invalid exchange rate format"
                    })),
                ).into_response()
             }
        } else if entry_req.source_currency == functional_currency {
            Decimal::ONE
        } else {
            // Lookup rate from database
            use zeltra_db::repositories::exchange_rate::ExchangeRateRepository;
            
            let rate_repo = ExchangeRateRepository::new((*state.db).clone());
            let tx_date = payload.transaction_date;
            
            match rate_repo.find_rate(
                org_id,
                &entry_req.source_currency,
                &functional_currency,
                tx_date,
            ).await {
                Ok(lookup) => lookup.rate,
                Err(e) => {
                    info!(
                        from = %entry_req.source_currency,
                        to = %functional_currency,
                        error = %e,
                        "Exchange rate not found in database, using fallback 1.0"
                    );
                    // Fallback to 1.0 if rate not found (user should override)
                    Decimal::ONE
                }
            }
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
            compliance_metadata: entry_req.metadata.clone(),
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

    // Extract idempotency key from header or body (header takes precedence)
    let idempotency_key = if let Some(key_str) = headers
        .get("Idempotency-Key")
        .or_else(|| headers.get("idempotency-key"))
    {
        match key_str.to_str().ok().and_then(|k| Uuid::parse_str(k).ok()) {
            Some(key) => Some(key),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_idempotency_key",
                        "message": "Invalid Idempotency-Key header format (must be UUID)"
                    })),
                )
                    .into_response();
            }
        }
    } else {
        payload.idempotency_key
    };

    let input = CreateTransactionInput {
        organization_id: org_id,
        transaction_type,
        transaction_date: payload.transaction_date,
        description: payload.description,
        reference_number: payload.reference_number,
        memo: payload.memo,
        entries,
        created_by: auth.user_id(),
        timezone: payload.timezone,
        idempotency_key,
        iso_metadata: payload.iso_metadata,
    };

    match tx_repo.create_transaction(input).await {
        Ok(result) => {
            info!(
                org_id = %org_id,
                transaction_id = %result.transaction.id,
                "Transaction created"
            );

            let response = map_transaction_to_response(result);
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
                zeltra_db::repositories::transaction::TransactionError::BudgetConstraintViolation {
                    account_id,
                    missing_dimensions,
                } => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "budget_constraint_violation",
                        "message": format!("Budget constraint violation for account {}", account_id),
                        "details": {
                            "missing_dimensions": missing_dimensions,
                        }
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
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/transactions/{transaction_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Transaction details", body = TransactionResponse),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
            let response = map_transaction_to_response(result);
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
#[utoipa::path(
    patch,
    path = "/organizations/{org_id}/transactions/{transaction_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    request_body = UpdateTransactionRequest,
    responses(
        (status = 200, description = "Transaction updated successfully"),
        (status = 400, description = "Cannot modify posted transaction"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
#[utoipa::path(
    delete,
    path = "/organizations/{org_id}/transactions/{transaction_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 204, description = "Transaction deleted successfully"),
        (status = 400, description = "Cannot delete posted transaction"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/{transaction_id}/submit",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Transaction submitted successfully"),
        (status = 400, description = "Invalid status for submission"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/{transaction_id}/approve",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    request_body = ApproveRequest,
    responses(
        (status = 200, description = "Transaction approved successfully"),
        (status = 400, description = "Invalid status for approval"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/{transaction_id}/reject",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    request_body = RejectRequest,
    responses(
        (status = 200, description = "Transaction rejected successfully"),
        (status = 400, description = "Invalid status for rejection"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/{transaction_id}/post",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    responses(
        (status = 200, description = "Transaction posted successfully"),
        (status = 400, description = "Invalid status for posting"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/{transaction_id}/void",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("transaction_id" = Uuid, Path, description = "Transaction ID")
    ),
    request_body = VoidRequest,
    responses(
        (status = 200, description = "Transaction voided successfully", body = VoidResponse),
        (status = 400, description = "Invalid status for voiding"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Transaction not found")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
    let tx_repo = TransactionRepository::new((*state.db).clone());

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

            // Fetch full transaction data for both original and reversing transactions
            let original_full = match tx_repo
                .get_transaction(org_id, result.original_transaction.id)
                .await
            {
                Ok(tx) => tx,
                Err(e) => {
                    error!(error = %e, "Failed to fetch voided transaction details");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "internal_error",
                            "message": "Failed to fetch transaction details"
                        })),
                    )
                        .into_response();
                }
            };

            let reversing_full = match tx_repo
                .get_transaction(org_id, result.reversing_transaction.id)
                .await
            {
                Ok(tx) => tx,
                Err(e) => {
                    error!(error = %e, "Failed to fetch reversing transaction details");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "internal_error",
                            "message": "Failed to fetch transaction details"
                        })),
                    )
                        .into_response();
                }
            };

            // Return proper VoidResponse with full TransactionResponse for both
            let response = VoidResponse {
                original_transaction: map_transaction_to_response(original_full),
                reversing_transaction: map_transaction_to_response(reversing_full),
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to void transaction");
            workflow_error_response(e)
        }
    }
}

/// GET `/organizations/{org_id}/transactions/pending` - Get pending transactions.
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/transactions/pending",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of pending transactions", body = [PendingTransactionResponse]),
        (status = 403, description = "Forbidden")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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
                        total_amount: p.total_amount.to_string(),
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
#[utoipa::path(
    post,
    path = "/organizations/{org_id}/transactions/bulk-approve",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = BulkApproveRequest,
    responses(
        (status = 200, description = "Bulk approval results", body = BulkApproveResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Transactions",
    security(("bearerAuth" = []))
)]
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

fn map_transaction_to_response(result: TransactionWithEntries) -> TransactionResponse {
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

    TransactionResponse {
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
        timezone: result.transaction.timezone,
        idempotency_key: result.transaction.idempotency_key,
        iso_metadata: result.transaction.iso_metadata,
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
        TransactionType::Accrual => "accrual".to_string(),
        TransactionType::Revaluation => "revaluation".to_string(),
        TransactionType::Intercompany => "intercompany".to_string(),
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
        "accrual" => Some(TransactionType::Accrual),
        "revaluation" => Some(TransactionType::Revaluation),
        "intercompany" => Some(TransactionType::Intercompany),
        _ => None,
    }
}

/// Helper to construct payment ledger entries.
fn construct_payment_entries(
    payload: &PayInvoiceRequest,
    original_rate: Decimal,
    original_currency: &str,
    ar_ap_account_id: Uuid,
    functional_currency: &str,
    variance: Decimal,
) -> Vec<CreateLedgerEntryInput> {
    let mut entries = Vec::new();

    // A. Bank Entry (Payment)
    let bank_functional = payload.amount * payload.exchange_rate;
    entries.push(CreateLedgerEntryInput {
        account_id: payload.payment_account_id,
        source_currency: original_currency.to_string(),
        source_amount: payload.amount,
        exchange_rate: payload.exchange_rate,
        functional_currency: functional_currency.to_string(),
        functional_amount: bank_functional,
        debit: Decimal::ZERO,
        credit: bank_functional,
        memo: Some("Payment".to_string()),
        dimensions: vec![],
        compliance_metadata: None,
    });

    // B. AP/AR Clearing Entry
    let clearing_functional = payload.amount * original_rate;
    entries.push(CreateLedgerEntryInput {
        account_id: ar_ap_account_id,
        source_currency: original_currency.to_string(),
        source_amount: payload.amount,
        exchange_rate: original_rate,
        functional_currency: functional_currency.to_string(),
        functional_amount: clearing_functional,
        debit: clearing_functional,
        credit: Decimal::ZERO,
        memo: Some("Clearing".to_string()),
        dimensions: vec![],
        compliance_metadata: None,
    });

    // C. Variance Entry
    if variance.abs() > Decimal::ZERO {
        let (v_debit, v_credit) = if variance > Decimal::ZERO {
            // Rate increased -> Loss -> Debit
            (variance.abs(), Decimal::ZERO)
        } else {
            // Rate decreased -> Gain -> Credit
            (Decimal::ZERO, variance.abs())
        };

        entries.push(CreateLedgerEntryInput {
            account_id: payload.gain_loss_account_id,
            source_currency: functional_currency.to_string(),
            source_amount: variance.abs(),
            exchange_rate: Decimal::ONE,
            functional_currency: functional_currency.to_string(),
            functional_amount: variance.abs(),
            debit: v_debit,
            credit: v_credit,
            memo: Some("Realized Forex Gain/Loss".to_string()),
            dimensions: vec![],
            compliance_metadata: None,
        });
    }

    entries
}

/// Helper to fetch invoice and determine original rate details.
async fn fetch_invoice_details(
    tx_repo: &TransactionRepository,
    org_id: Uuid,
    invoice_id: Uuid,
    functional_currency: &str,
) -> Result<(TransactionWithEntries, Decimal, String, Uuid), (StatusCode, Json<serde_json::Value>)>
{
    let Ok(invoice) = tx_repo.get_transaction(org_id, invoice_id).await else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invoice_not_found"})),
        ));
    };

    let original_entry = invoice
        .entries
        .iter()
        .find(|e| e.entry.source_currency != functional_currency)
        .or_else(|| invoice.entries.first())
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "invalid_invoice_entries"})),
            )
        })?;

    Ok((
        invoice.clone(),
        original_entry.entry.exchange_rate,
        original_entry.entry.source_currency.clone(),
        original_entry.entry.account_id,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_limit_enforcement() {
        // Case 1: No limit
        assert!(check_monthly_transaction_limit(None, 100).is_ok());

        // Case 2: Limit not reached
        assert!(check_monthly_transaction_limit(Some(5), 4).is_ok());

        // Case 3: Limit reached (exactly)
        let result = check_monthly_transaction_limit(Some(5), 5);
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED);

        // Case 4: Limit exceeded
        let result = check_monthly_transaction_limit(Some(5), 6);
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    }
}
