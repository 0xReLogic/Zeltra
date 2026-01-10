//! Report routes.
//!
//! Implements Requirements 14.1-14.6 for Report API endpoints.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_core::reports::{BalanceSheetSection, IncomeStatementSection, ReportService};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::AccountType,
    repositories::report::{AccountBalance, ReportRepository},
};

/// Creates the report routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/reports/trial-balance",
            get(get_trial_balance),
        )
        .route(
            "/organizations/{org_id}/reports/balance-sheet",
            get(get_balance_sheet),
        )
        .route(
            "/organizations/{org_id}/reports/income-statement",
            get(get_income_statement),
        )
        .route(
            "/organizations/{org_id}/reports/dimensional",
            get(get_dimensional_report),
        )
        .route(
            "/organizations/{org_id}/accounts/{account_id}/ledger",
            get(get_account_ledger),
        )
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for trial balance report.
#[derive(Debug, Deserialize)]
pub struct TrialBalanceQuery {
    /// As of date (defaults to today).
    pub as_of: Option<NaiveDate>,
    /// Dimension value IDs to filter by (comma-separated).
    pub dimensions: Option<String>,
}

/// Query parameters for balance sheet report.
#[derive(Debug, Deserialize)]
pub struct BalanceSheetQuery {
    /// As of date (defaults to today).
    pub as_of: Option<NaiveDate>,
}

/// Query parameters for income statement report.
#[derive(Debug, Deserialize)]
pub struct IncomeStatementQuery {
    /// Start date.
    pub from: Option<NaiveDate>,
    /// End date.
    pub to: Option<NaiveDate>,
    /// Dimension value IDs to filter by (comma-separated).
    pub dimensions: Option<String>,
}

/// Query parameters for dimensional report.
#[derive(Debug, Deserialize)]
pub struct DimensionalReportQuery {
    /// Start date.
    pub from: Option<NaiveDate>,
    /// End date.
    pub to: Option<NaiveDate>,
    /// Dimension types to group by (comma-separated).
    pub group_by: String,
    /// Account type filter.
    pub account_type: Option<String>,
    /// Dimension value IDs to filter by (comma-separated).
    pub dimensions: Option<String>,
}

/// Query parameters for account ledger.
#[derive(Debug, Deserialize)]
pub struct AccountLedgerQuery {
    /// Start date.
    pub from: Option<NaiveDate>,
    /// End date.
    pub to: Option<NaiveDate>,
    /// Page number (0-indexed).
    pub page: Option<u64>,
    /// Items per page.
    pub limit: Option<u64>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Response for trial balance report.
#[derive(Debug, Serialize)]
pub struct TrialBalanceResponse {
    /// Report type.
    pub report_type: String,
    /// As of date.
    pub as_of: String,
    /// Currency.
    pub currency: String,
    /// Account balances.
    pub accounts: Vec<AccountBalanceResponse>,
    /// Totals.
    pub totals: TrialBalanceTotals,
}

/// Account balance in response.
#[derive(Debug, Serialize)]
pub struct AccountBalanceResponse {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub code: String,
    /// Account name.
    pub name: String,
    /// Account type.
    pub account_type: String,
    /// Debit amount.
    pub debit: String,
    /// Credit amount.
    pub credit: String,
    /// Balance.
    pub balance: String,
}

/// Trial balance totals.
#[derive(Debug, Serialize)]
pub struct TrialBalanceTotals {
    /// Total debit.
    pub total_debit: String,
    /// Total credit.
    pub total_credit: String,
    /// Whether balanced.
    pub is_balanced: bool,
}

/// Response for balance sheet report.
#[derive(Debug, Serialize)]
pub struct BalanceSheetResponse {
    /// Report type.
    pub report_type: String,
    /// As of date.
    pub as_of: String,
    /// Currency.
    pub currency: String,
    /// Assets section.
    pub assets: BalanceSheetSectionResponse,
    /// Liabilities section.
    pub liabilities: BalanceSheetSectionResponse,
    /// Equity section.
    pub equity: BalanceSheetSectionResponse,
    /// Total assets.
    pub total_assets: String,
    /// Total liabilities and equity.
    pub total_liabilities_and_equity: String,
    /// Whether balanced.
    pub is_balanced: bool,
}

/// Balance sheet section response.
#[derive(Debug, Serialize)]
pub struct BalanceSheetSectionResponse {
    /// Section accounts.
    pub accounts: Vec<AccountBalanceResponse>,
    /// Section total.
    pub total: String,
}

/// Response for income statement report.
#[derive(Debug, Serialize)]
pub struct IncomeStatementResponse {
    /// Report type.
    pub report_type: String,
    /// Period start.
    pub period_start: String,
    /// Period end.
    pub period_end: String,
    /// Currency.
    pub currency: String,
    /// Revenue section.
    pub revenue: IncomeStatementSectionResponse,
    /// Cost of goods sold section.
    pub cost_of_goods_sold: IncomeStatementSectionResponse,
    /// Gross profit.
    pub gross_profit: String,
    /// Operating expenses section.
    pub operating_expenses: IncomeStatementSectionResponse,
    /// Operating income.
    pub operating_income: String,
    /// Other income/expenses section.
    pub other_income_expenses: IncomeStatementSectionResponse,
    /// Net income.
    pub net_income: String,
}

/// Income statement section response.
#[derive(Debug, Serialize)]
pub struct IncomeStatementSectionResponse {
    /// Section accounts.
    pub accounts: Vec<AccountBalanceResponse>,
    /// Section total.
    pub total: String,
}

/// Response for dimensional report.
#[derive(Debug, Serialize)]
pub struct DimensionalReportResponse {
    /// Report type.
    pub report_type: String,
    /// Period start.
    pub period_start: String,
    /// Period end.
    pub period_end: String,
    /// Currency.
    pub currency: String,
    /// Grouped by dimensions.
    pub group_by: Vec<String>,
    /// Report rows.
    pub rows: Vec<DimensionalReportRowResponse>,
    /// Grand total.
    pub grand_total: String,
}

/// Dimensional report row response.
#[derive(Debug, Serialize)]
pub struct DimensionalReportRowResponse {
    /// Dimension values.
    pub dimensions: Vec<DimensionValueResponse>,
    /// Total debit.
    pub total_debit: String,
    /// Total credit.
    pub total_credit: String,
    /// Balance.
    pub balance: String,
}

/// Dimension value in response.
#[derive(Debug, Serialize)]
pub struct DimensionValueResponse {
    /// Dimension type.
    pub dimension_type: String,
    /// Code.
    pub code: String,
    /// Name.
    pub name: String,
}

/// Response for account ledger.
#[derive(Debug, Serialize)]
pub struct AccountLedgerResponse {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub code: String,
    /// Account name.
    pub name: String,
    /// Entries.
    pub entries: Vec<LedgerEntryResponse>,
    /// Pagination.
    pub pagination: PaginationResponse,
}

/// Ledger entry response.
#[derive(Debug, Serialize)]
pub struct LedgerEntryResponse {
    /// Entry ID.
    pub id: Uuid,
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Transaction date.
    pub transaction_date: String,
    /// Description.
    pub description: String,
    /// Source currency.
    pub source_currency: String,
    /// Source amount.
    pub source_amount: String,
    /// Exchange rate.
    pub exchange_rate: String,
    /// Functional amount.
    pub functional_amount: String,
    /// Debit.
    pub debit: String,
    /// Credit.
    pub credit: String,
    /// Running balance.
    pub running_balance: String,
    /// Dimensions.
    pub dimensions: Vec<DimensionValueResponse>,
}

/// Pagination response.
#[derive(Debug, Serialize)]
pub struct PaginationResponse {
    /// Current page.
    pub page: u64,
    /// Items per page.
    pub limit: u64,
    /// Total items.
    pub total: u64,
    /// Total pages.
    pub total_pages: u64,
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

/// Parses comma-separated UUIDs from a string.
fn parse_uuid_list(s: &str) -> Vec<Uuid> {
    s.split(',')
        .filter_map(|part| Uuid::parse_str(part.trim()).ok())
        .collect()
}

/// Formats a Decimal as a string with 4 decimal places.
fn format_money(amount: Decimal) -> String {
    format!("{amount:.4}")
}

/// Converts AccountType enum to string.
fn account_type_to_string(account_type: &AccountType) -> String {
    match account_type {
        AccountType::Asset => "asset".to_string(),
        AccountType::Liability => "liability".to_string(),
        AccountType::Equity => "equity".to_string(),
        AccountType::Revenue => "revenue".to_string(),
        AccountType::Expense => "expense".to_string(),
    }
}

/// Parses account type from string.
fn parse_account_type(s: &str) -> Option<AccountType> {
    match s.to_lowercase().as_str() {
        "asset" => Some(AccountType::Asset),
        "liability" => Some(AccountType::Liability),
        "equity" => Some(AccountType::Equity),
        "revenue" => Some(AccountType::Revenue),
        "expense" => Some(AccountType::Expense),
        _ => None,
    }
}

/// Converts AccountBalance to response.
fn account_balance_to_response(ab: &AccountBalance) -> AccountBalanceResponse {
    AccountBalanceResponse {
        account_id: ab.account_id,
        code: ab.code.clone(),
        name: ab.name.clone(),
        account_type: account_type_to_string(&ab.account_type),
        debit: format_money(ab.total_debit),
        credit: format_money(ab.total_credit),
        balance: format_money(ab.balance),
    }
}

/// Converts BalanceSheetSection to response.
fn balance_sheet_section_to_response(section: &BalanceSheetSection) -> BalanceSheetSectionResponse {
    BalanceSheetSectionResponse {
        accounts: section
            .accounts
            .iter()
            .map(|a| AccountBalanceResponse {
                account_id: a.account_id,
                code: a.code.clone(),
                name: a.name.clone(),
                account_type: a.account_type.clone(),
                debit: format_money(a.total_debit),
                credit: format_money(a.total_credit),
                balance: format_money(a.balance),
            })
            .collect(),
        total: format_money(section.total),
    }
}

/// Converts IncomeStatementSection to response.
fn income_statement_section_to_response(
    section: &IncomeStatementSection,
) -> IncomeStatementSectionResponse {
    IncomeStatementSectionResponse {
        accounts: section
            .accounts
            .iter()
            .map(|a| AccountBalanceResponse {
                account_id: a.account_id,
                code: a.code.clone(),
                name: a.name.clone(),
                account_type: a.account_type.clone(),
                debit: format_money(a.total_debit),
                credit: format_money(a.total_credit),
                balance: format_money(a.balance),
            })
            .collect(),
        total: format_money(section.total),
    }
}

// ============================================================================
// Route Handlers
// ============================================================================

/// GET /organizations/{org_id}/reports/trial-balance
///
/// Requirement 14.1: Trial balance report endpoint
#[axum::debug_handler]
async fn get_trial_balance(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<TrialBalanceQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
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

    let as_of = query
        .as_of
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let dimension_filters = query
        .dimensions
        .as_ref()
        .map(|s| parse_uuid_list(s))
        .unwrap_or_default();

    let report_repo = ReportRepository::new((*state.db).clone());

    // Query account balances
    let balances = match report_repo
        .query_trial_balance(org_id, as_of, &dimension_filters)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            error!(error = %e, "Failed to query trial balance");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to generate trial balance"
                })),
            )
                .into_response();
        }
    };

    // Generate trial balance report using core service
    let report = ReportService::generate_trial_balance(
        balances
            .iter()
            .map(|ab| zeltra_core::reports::AccountBalance {
                account_id: ab.account_id,
                code: ab.code.clone(),
                name: ab.name.clone(),
                account_type: account_type_to_string(&ab.account_type),
                account_subtype: ab.account_subtype.as_ref().map(account_subtype_to_string),
                total_debit: ab.total_debit,
                total_credit: ab.total_credit,
                balance: ab.balance,
            })
            .collect(),
    );

    let response = TrialBalanceResponse {
        report_type: "trial_balance".to_string(),
        as_of: as_of.to_string(),
        currency: org.base_currency,
        accounts: balances.iter().map(account_balance_to_response).collect(),
        totals: TrialBalanceTotals {
            total_debit: format_money(report.totals.total_debit),
            total_credit: format_money(report.totals.total_credit),
            is_balanced: report.totals.is_balanced,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /organizations/{org_id}/reports/balance-sheet
///
/// Requirement 14.2: Balance sheet report endpoint
#[axum::debug_handler]
async fn get_balance_sheet(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<BalanceSheetQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
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

    let as_of = query
        .as_of
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let report_repo = ReportRepository::new((*state.db).clone());

    // Query account balances
    let balances = match report_repo.query_balance_sheet(org_id, as_of).await {
        Ok(b) => b,
        Err(e) => {
            error!(error = %e, "Failed to query balance sheet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to generate balance sheet"
                })),
            )
                .into_response();
        }
    };

    // Generate balance sheet report using core service
    let report = ReportService::generate_balance_sheet(
        balances
            .iter()
            .map(|ab| zeltra_core::reports::AccountBalance {
                account_id: ab.account_id,
                code: ab.code.clone(),
                name: ab.name.clone(),
                account_type: account_type_to_string(&ab.account_type),
                account_subtype: ab.account_subtype.as_ref().map(account_subtype_to_string),
                total_debit: ab.total_debit,
                total_credit: ab.total_credit,
                balance: ab.balance,
            })
            .collect(),
    );

    let response = BalanceSheetResponse {
        report_type: "balance_sheet".to_string(),
        as_of: as_of.to_string(),
        currency: org.base_currency,
        assets: balance_sheet_section_to_response(&report.assets),
        liabilities: balance_sheet_section_to_response(&report.liabilities),
        equity: balance_sheet_section_to_response(&report.equity),
        total_assets: format_money(report.total_assets),
        total_liabilities_and_equity: format_money(report.liabilities_and_equity),
        is_balanced: report.is_balanced,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /organizations/{org_id}/reports/income-statement
///
/// Requirement 14.3: Income statement report endpoint
#[axum::debug_handler]
async fn get_income_statement(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<IncomeStatementQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
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

    // Default to current month if not specified
    let today = chrono::Utc::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    // Validate date range
    if from > to {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date_range",
                "message": "Start date must be before or equal to end date"
            })),
        )
            .into_response();
    }

    let dimension_filters = query
        .dimensions
        .as_ref()
        .map(|s| parse_uuid_list(s))
        .unwrap_or_default();

    let report_repo = ReportRepository::new((*state.db).clone());

    // Query account balances
    let balances = match report_repo
        .query_income_statement(org_id, from, to, &dimension_filters)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            error!(error = %e, "Failed to query income statement");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to generate income statement"
                })),
            )
                .into_response();
        }
    };

    // Generate income statement report using core service
    let report = ReportService::generate_income_statement(
        balances
            .iter()
            .map(|ab| zeltra_core::reports::AccountBalance {
                account_id: ab.account_id,
                code: ab.code.clone(),
                name: ab.name.clone(),
                account_type: account_type_to_string(&ab.account_type),
                account_subtype: ab.account_subtype.as_ref().map(account_subtype_to_string),
                total_debit: ab.total_debit,
                total_credit: ab.total_credit,
                balance: ab.balance,
            })
            .collect(),
    );

    let response = IncomeStatementResponse {
        report_type: "income_statement".to_string(),
        period_start: from.to_string(),
        period_end: to.to_string(),
        currency: org.base_currency,
        revenue: income_statement_section_to_response(&report.revenue),
        cost_of_goods_sold: income_statement_section_to_response(&report.cost_of_goods_sold),
        gross_profit: format_money(report.gross_profit),
        operating_expenses: income_statement_section_to_response(&report.operating_expenses),
        operating_income: format_money(report.operating_income),
        other_income_expenses: income_statement_section_to_response(&report.other_income_expense),
        net_income: format_money(report.net_income),
    };

    (StatusCode::OK, Json(response)).into_response()
}

use chrono::Datelike;

/// GET /organizations/{org_id}/reports/dimensional
///
/// Requirement 14.5: Dimensional report endpoint
#[allow(clippy::too_many_lines)]
#[axum::debug_handler]
async fn get_dimensional_report(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<DimensionalReportQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
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

    // Default to current month if not specified
    let today = chrono::Utc::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    // Validate date range
    if from > to {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date_range",
                "message": "Start date must be before or equal to end date"
            })),
        )
            .into_response();
    }

    // Parse group_by dimensions
    let group_by: Vec<String> = query
        .group_by
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if group_by.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_group_by",
                "message": "At least one group_by dimension is required"
            })),
        )
            .into_response();
    }

    let account_type_filter = query
        .account_type
        .as_ref()
        .and_then(|s| parse_account_type(s));
    let dimension_filters = query
        .dimensions
        .as_ref()
        .map(|s| parse_uuid_list(s))
        .unwrap_or_default();

    let report_repo = ReportRepository::new((*state.db).clone());

    // Query dimensional report
    let (rows, grand_total) = match report_repo
        .query_dimensional_report(
            org_id,
            from,
            to,
            &group_by,
            account_type_filter,
            &dimension_filters,
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, "Failed to query dimensional report");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to generate dimensional report"
                })),
            )
                .into_response();
        }
    };

    let response = DimensionalReportResponse {
        report_type: "dimensional".to_string(),
        period_start: from.to_string(),
        period_end: to.to_string(),
        currency: org.base_currency,
        group_by: group_by.clone(),
        rows: rows
            .iter()
            .map(|row| DimensionalReportRowResponse {
                dimensions: row
                    .dimensions
                    .iter()
                    .map(|d| DimensionValueResponse {
                        dimension_type: d.dimension_type.clone(),
                        code: d.code.clone(),
                        name: d.name.clone(),
                    })
                    .collect(),
                total_debit: format_money(row.total_debit),
                total_credit: format_money(row.total_credit),
                balance: format_money(row.balance),
            })
            .collect(),
        grand_total: format_money(grand_total),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /organizations/{org_id}/accounts/{account_id}/ledger
///
/// Requirement 14.4: Account ledger endpoint
#[allow(clippy::too_many_lines)]
#[axum::debug_handler]
async fn get_account_ledger(
    State(state): State<AppState>,
    Path((org_id, account_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<AccountLedgerQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
        return response;
    }

    // Default to current month if not specified
    let today = chrono::Utc::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    // Validate date range
    if from > to {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date_range",
                "message": "Start date must be before or equal to end date"
            })),
        )
            .into_response();
    }

    let page = query.page.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).min(100);

    let report_repo = ReportRepository::new((*state.db).clone());

    // Query account ledger
    let (entries, total) = match report_repo
        .query_account_ledger(org_id, account_id, from, to, page, limit)
        .await
    {
        Ok(r) => r,
        Err(zeltra_db::repositories::report::ReportError::AccountNotFound(_)) => {
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
            error!(error = %e, "Failed to query account ledger");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get account ledger"
                })),
            )
                .into_response();
        }
    };

    // Get account info
    let account = match zeltra_db::entities::chart_of_accounts::Entity::find_by_id(account_id)
        .one(&*state.db)
        .await
    {
        Ok(Some(a)) => a,
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
            error!(error = %e, "Failed to get account");
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

    let total_pages = if total == 0 { 0 } else { total.div_ceil(limit) };

    let response = AccountLedgerResponse {
        account_id,
        code: account.code,
        name: account.name,
        entries: entries
            .iter()
            .map(|e| LedgerEntryResponse {
                id: e.id,
                transaction_id: e.transaction_id,
                transaction_date: e.transaction_date.to_string(),
                description: e.description.clone(),
                source_currency: e.source_currency.clone(),
                source_amount: format_money(e.source_amount),
                exchange_rate: format!("{:.10}", e.exchange_rate),
                functional_amount: format_money(e.functional_amount),
                debit: format_money(e.debit),
                credit: format_money(e.credit),
                running_balance: format_money(e.running_balance),
                dimensions: e
                    .dimensions
                    .iter()
                    .map(|d| DimensionValueResponse {
                        dimension_type: d.dimension_type.clone(),
                        code: d.code.clone(),
                        name: d.name.clone(),
                    })
                    .collect(),
            })
            .collect(),
        pagination: PaginationResponse {
            page,
            limit,
            total,
            total_pages,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

// ============================================================================
// Type Conversion Helpers
// ============================================================================

/// Converts DB AccountSubtype to string.
fn account_subtype_to_string(
    db_subtype: &zeltra_db::entities::sea_orm_active_enums::AccountSubtype,
) -> String {
    use zeltra_db::entities::sea_orm_active_enums::AccountSubtype as DbSubtype;

    match db_subtype {
        DbSubtype::Cash => "cash".to_string(),
        DbSubtype::Bank => "bank".to_string(),
        DbSubtype::AccountsReceivable => "accounts_receivable".to_string(),
        DbSubtype::Inventory => "inventory".to_string(),
        DbSubtype::Prepaid => "prepaid".to_string(),
        DbSubtype::FixedAsset => "fixed_asset".to_string(),
        DbSubtype::AccumulatedDepreciation => "accumulated_depreciation".to_string(),
        DbSubtype::OtherAsset => "other_asset".to_string(),
        DbSubtype::AccountsPayable => "accounts_payable".to_string(),
        DbSubtype::CreditCard => "credit_card".to_string(),
        DbSubtype::AccruedLiability => "accrued_liability".to_string(),
        DbSubtype::ShortTermDebt => "short_term_debt".to_string(),
        DbSubtype::LongTermDebt => "long_term_debt".to_string(),
        DbSubtype::OtherLiability => "other_liability".to_string(),
        DbSubtype::OwnerEquity => "owner_equity".to_string(),
        DbSubtype::RetainedEarnings => "retained_earnings".to_string(),
        DbSubtype::CommonStock => "common_stock".to_string(),
        DbSubtype::OtherEquity => "other_equity".to_string(),
        DbSubtype::OperatingRevenue => "operating_revenue".to_string(),
        DbSubtype::OtherRevenue => "other_revenue".to_string(),
        DbSubtype::CostOfGoodsSold => "cost_of_goods_sold".to_string(),
        DbSubtype::OperatingExpense => "operating_expense".to_string(),
        DbSubtype::PayrollExpense => "payroll_expense".to_string(),
        DbSubtype::DepreciationExpense => "depreciation_expense".to_string(),
        DbSubtype::InterestExpense => "interest_expense".to_string(),
        DbSubtype::TaxExpense => "tax_expense".to_string(),
        DbSubtype::OtherExpense => "other_expense".to_string(),
    }
}

use sea_orm::EntityTrait;
