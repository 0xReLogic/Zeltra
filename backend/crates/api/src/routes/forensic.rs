//! Forensic Suite API routes.
//!
//! Implements endpoints for "AI-driven" forensic analysis (Benford's Law, Altman Z-Score, Beneish M-Score).
//! Locked to Enterprise Tier.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, RelationTrait};
use zeltra_core::forensic::{AltmanDetails, BeneishDetails, BenfordRecord, ForensicService};
use zeltra_core::reports::ReportService;
use zeltra_db::{
    OrganizationRepository,
    entities::{ledger_entries, sea_orm_active_enums::SubscriptionTier},
    repositories::report::ReportRepository,
};

/// Creates the forensic routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/forensic/benford", get(get_benford))
        .route(
            "/organizations/{org_id}/forensic/health-score",
            get(get_health_score),
        )
}

// ============================================================================
// Response Types
// ============================================================================

/// Response for Advanced Benford's Law analysis.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BenfordResponse {
    /// 1st Digit Distribution.
    pub distribution_1st_digit: Vec<BenfordRecord>,
    /// 2nd Digit Distribution (New).
    pub distribution_2nd_digit: Vec<BenfordRecord>,
    /// Mean Absolute Deviation Score.
    #[schema(example = 0.005)]
    pub mad_score: f64,
    /// MAD Verdict (Conform/Nonconform).
    #[schema(example = "Close Conformity")]
    pub mad_verdict: String,
}

/// Combined Health Score Response (Altman Z + Beneish M).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct HealthScoreResponse {
    /// Altman Z-Score Result.
    pub z_score: f64,
    /// Altman Zone (Safe, Grey, Distress).
    pub z_zone: String,
    /// Detailed Altman Z-Score Factors.
    pub z_details: AltmanDetails,

    /// Beneish M-Score Result (New).
    pub m_score: f64,
    /// Beneish Risk Level (Safe, Possible Manipulation).
    pub m_risk_level: String,
    /// Manipulation Probability (Standard Normal CDF approx).
    pub m_prob: f64,
    /// Detailed Beneish M-Score Components.
    pub m_details: BeneishDetails,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Checks if organization is on Enterprise tier.
async fn check_tier(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.find_by_id(org_id).await {
        Ok(Some(org)) => {
            if org.subscription_tier != SubscriptionTier::Enterprise {
                return Err((
                    StatusCode::PAYMENT_REQUIRED,
                    Json(json!({
                        "error": "tier_limit_exceeded",
                        "message": "This feature requires an Enterprise subscription."
                    })),
                )
                    .into_response());
            }
            Ok(())
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Organization not found"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Failed to check tier");
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

// ============================================================================
// Route Handlers
// ============================================================================

/// GET /organizations/{org_id}/forensic/benford
///
/// Performs Advanced Benford's Law analysis (1st & 2nd Digit, MAD).
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/forensic/benford",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Advanced Benford analysis", body = BenfordResponse),
        (status = 402, description = "Payment Required (Enterprise Only)"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "Forensic",
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
async fn get_benford(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // 1. Check Membership
    match org_repo.is_member(org_id, auth_user.user_id()).await {
        Ok(true) => {}
        Ok(false) => {
            return (StatusCode::FORBIDDEN, Json(json!({"error": "forbidden"}))).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal"})),
            )
                .into_response();
        }
    }

    // 2. Check Tier (Enterprise Only)
    if let Err(resp) = check_tier(&org_repo, org_id).await {
        return resp;
    }

    // 3. Fetch Data (Ledger Entry Amounts)
    let amounts: Vec<Decimal> = match ledger_entries::Entity::find()
        .join(
            sea_orm::JoinType::InnerJoin,
            zeltra_db::entities::ledger_entries::Relation::ChartOfAccounts.def(),
        )
        .filter(zeltra_db::entities::chart_of_accounts::Column::OrganizationId.eq(org_id))
        .select_only()
        .column(ledger_entries::Column::FunctionalAmount)
        .into_tuple()
        .all(&(*state.db))
        .await
    {
        Ok(a) => a,
        Err(e) => {
            error!(error = %e, "Failed to fetch ledger entries");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
                .into_response();
        }
    };

    // 4. Run Core Logic (Advanced)
    let analysis = ForensicService::calculate_benford_law(amounts);

    let response = BenfordResponse {
        distribution_1st_digit: analysis.first_digit_distribution,
        distribution_2nd_digit: analysis.second_digit_distribution,
        mad_score: analysis.mad_score,
        mad_verdict: analysis.mad_verdict,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /organizations/{org_id}/forensic/health-score
///
/// Calculates Financial Health Metrics (Altman Z-Score & Beneish M-Score).
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/forensic/health-score",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Health Score Results", body = HealthScoreResponse),
        (status = 402, description = "Payment Required (Enterprise Only)"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found")
    ),
    tag = "Forensic",
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
#[allow(clippy::too_many_lines)] // Orchestration logic is lengthy
async fn get_health_score(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // 1. Check Membership
    match org_repo.is_member(org_id, auth_user.user_id()).await {
        Ok(true) => {}
        Ok(false) => {
            return (StatusCode::FORBIDDEN, Json(json!({"error": "forbidden"}))).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal"})),
            )
                .into_response();
        }
    }

    // 2. Check Tier
    if let Err(resp) = check_tier(&org_repo, org_id).await {
        return resp;
    }

    // 3. Fetch Balance Sheet (Current Year)
    let report_repo = ReportRepository::new((*state.db).clone());
    let as_of = chrono::Utc::now().date_naive();

    let balances = match report_repo.query_balance_sheet(org_id, as_of).await {
        Ok(b) => b,
        Err(e) => {
            error!(error = %e, "Failed to query balance sheet");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "db_error"})),
            )
                .into_response();
        }
    };

    // Map to Core Types
    let core_balances: Vec<zeltra_core::reports::AccountBalance> = balances
        .iter()
        .map(|ab| zeltra_core::reports::AccountBalance {
            account_id: ab.account_id,
            code: ab.code.clone(),
            name: ab.name.clone(),
            account_type: account_type_to_string(&ab.account_type),
            account_subtype: None,
            total_debit: ab.total_debit,
            total_credit: ab.total_credit,
            balance: ab.balance,
        })
        .collect();

    let bs_report = ReportService::generate_balance_sheet(core_balances);

    // 4. Extract Altman Inputs
    let current_assets =
        find_subsection_total(&bs_report.assets, "Current Asset").unwrap_or_default();
    let current_liabilities =
        find_subsection_total(&bs_report.liabilities, "Current Liability").unwrap_or_default();
    let working_capital = current_assets - current_liabilities;
    let retained_earnings = bs_report
        .equity
        .accounts
        .iter()
        .find(|a| a.name.to_lowercase().contains("retained"))
        .map(|a| a.balance)
        .unwrap_or_default();

    // TODO: Fetch Income Statement for Sales/EBIT. Punting for now (0).
    let sales = Decimal::ZERO;
    let ebit = Decimal::ZERO;

    // 5. Calculate Altman Z-Score
    let z_result = ForensicService::calculate_altman_z_score(
        &bs_report,
        bs_report.total_assets,
        working_capital,
        retained_earnings,
        ebit,
        bs_report.total_equity,
        bs_report.total_liabilities,
        sales,
    );

    // 6. Calculate Beneish M-Score
    // Requires Prior Year data. We'll use dummy 0 values for t-1 for now to prevent crash.
    // In real prod, we'd query query_balance_sheet(as_of - 1 year).
    let d0 = Decimal::ZERO;

    // Approximations for Beneish Inputs based on what we have (BS):
    // Receivables -> Asset subsection?
    let receivables = Decimal::ZERO; // Need to find 'Accounts Receivable'
    let cogs = Decimal::ZERO; // Need IS
    let ppe = find_subsection_total(&bs_report.assets, "Fixed Asset").unwrap_or_default(); // Approx for PPE
    let dep = Decimal::ZERO; // Need IS
    let sga = Decimal::ZERO; // Need IS
    let ni = Decimal::ZERO; // Need IS
    let cfo = Decimal::ZERO; // Need Cash Flow Stmt
    let ltd = find_subsection_total(&bs_report.liabilities, "Long Term").unwrap_or_default();

    let m_result = ForensicService::calculate_beneish_m_score(
        receivables,
        d0, // receivables t, t-1
        sales,
        d0, // sales t, t-1
        cogs,
        d0, // cogs t, t-1
        bs_report.total_assets,
        d0, // assets t, t-1
        ppe,
        d0, // ppe t, t-1
        dep,
        d0, // dep t, t-1
        sga,
        d0,  // sga t, t-1
        ni,  // ni t
        cfo, // cfo t
        ltd,
        current_liabilities,
        d0,
        d0, // debt structure t, t-1
    );

    let response = HealthScoreResponse {
        z_score: z_result.score,
        z_zone: format!("{:?}", z_result.zone),
        z_details: z_result.details,

        m_score: m_result.score,
        m_risk_level: m_result.risk_level,
        m_prob: m_result.manipulation_probability,
        m_details: m_result.details,
    };

    (StatusCode::OK, Json(response)).into_response()
}

fn find_subsection_total(
    section: &zeltra_core::reports::BalanceSheetSection,
    name_part: &str,
) -> Option<Decimal> {
    section
        .subsections
        .iter()
        .find(|s| s.name.contains(name_part))
        .map(|s| s.total)
}

fn account_type_to_string(t: &zeltra_db::entities::sea_orm_active_enums::AccountType) -> String {
    match t {
        zeltra_db::entities::sea_orm_active_enums::AccountType::Asset => "asset".to_string(),
        zeltra_db::entities::sea_orm_active_enums::AccountType::Liability => {
            "liability".to_string()
        }
        zeltra_db::entities::sea_orm_active_enums::AccountType::Equity => "equity".to_string(),
        zeltra_db::entities::sea_orm_active_enums::AccountType::Revenue => "revenue".to_string(),
        zeltra_db::entities::sea_orm_active_enums::AccountType::Expense => "expense".to_string(),
    }
}
