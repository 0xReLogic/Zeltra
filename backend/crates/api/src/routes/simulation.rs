//! Simulation routes.
//!
//! Implements Requirements 15.1-15.4 for Simulation API endpoints.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::error;
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_core::simulation::{
    HistoricalAccountData as CoreHistoricalData, SimulationEngine, SimulationParams,
};
use zeltra_db::{OrganizationRepository, repositories::simulation::SimulationRepository};

/// Creates the simulation routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/organizations/{org_id}/simulation/run",
        post(run_simulation),
    )
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for running a simulation.
#[derive(Debug, Deserialize)]
pub struct RunSimulationRequest {
    /// Start date of historical period for baseline calculation.
    pub base_period_start: NaiveDate,
    /// End date of historical period for baseline calculation.
    pub base_period_end: NaiveDate,
    /// Number of months to project into the future (1-60).
    pub projection_months: u32,
    /// Global growth rate for revenue accounts (decimal, e.g. "0.10" for 10%).
    #[serde(default)]
    pub revenue_growth_rate: Option<String>,
    /// Global growth rate for expense accounts (decimal, e.g. "0.05" for 5%).
    #[serde(default)]
    pub expense_growth_rate: Option<String>,
    /// Per-account growth rate overrides. Key is account_id (UUID), value is growth rate.
    #[serde(default)]
    pub account_adjustments: Option<HashMap<String, String>>,
    /// Filter historical data by dimension value IDs.
    #[serde(default)]
    pub dimension_filters: Option<Vec<Uuid>>,
}

/// Response for simulation result.
#[derive(Debug, Serialize)]
pub struct SimulationResponse {
    /// Unique identifier for this simulation run.
    pub simulation_id: Uuid,
    /// Hash of simulation parameters for caching.
    pub parameters_hash: String,
    /// Whether this result was returned from cache.
    pub cached: bool,
    /// Account projections.
    pub projections: Vec<AccountProjectionResponse>,
    /// Annual summary.
    pub annual_summary: AnnualSummaryResponse,
    /// Monthly summary for charts.
    pub monthly_summary: Vec<MonthlySummaryResponse>,
}

/// Account projection response.
#[derive(Debug, Serialize)]
pub struct AccountProjectionResponse {
    /// Period identifier (YYYY-MM format).
    pub period_name: String,
    /// Period start date.
    pub period_start: String,
    /// Period end date.
    pub period_end: String,
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Account type.
    pub account_type: String,
    /// Average monthly amount from historical period.
    pub baseline_amount: String,
    /// Projected amount after applying growth rate.
    pub projected_amount: String,
    /// Percentage change from baseline.
    pub change_percent: String,
}

/// Annual summary response.
#[derive(Debug, Serialize)]
pub struct AnnualSummaryResponse {
    /// Sum of all projected revenue amounts.
    pub total_projected_revenue: String,
    /// Sum of all projected expense amounts.
    pub total_projected_expenses: String,
    /// Revenue minus expenses.
    pub projected_net_income: String,
    /// Net income / Revenue * 100.
    pub net_profit_margin: String,
}

/// Monthly summary response for charts.
#[derive(Debug, Serialize)]
pub struct MonthlySummaryResponse {
    /// Month label for chart.
    pub month: String,
    /// Full period identifier.
    pub period_name: String,
    /// Total revenue.
    pub revenue: String,
    /// Total expenses.
    pub expenses: String,
    /// Net income.
    pub net_income: String,
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

/// Formats a Decimal as a string with 4 decimal places.
fn format_money(amount: Decimal) -> String {
    format!("{amount:.4}")
}

/// Formats a Decimal as a string with 2 decimal places.
fn format_percent(amount: Decimal) -> String {
    format!("{amount:.2}")
}

/// Parses a decimal from string, defaulting to zero.
fn parse_decimal(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or(Decimal::ZERO)
}

/// Parses account adjustments from string map to UUID/Decimal map.
fn parse_account_adjustments(
    adjustments: Option<HashMap<String, String>>,
) -> HashMap<Uuid, Decimal> {
    adjustments
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(k, v)| {
            let uuid = Uuid::parse_str(&k).ok()?;
            let rate = Decimal::from_str(&v).ok()?;
            Some((uuid, rate))
        })
        .collect()
}

/// Gets month abbreviation from period name (YYYY-MM).
fn get_month_abbrev(period_name: &str) -> String {
    let parts: Vec<&str> = period_name.split('-').collect();
    if parts.len() >= 2 {
        match parts[1] {
            "01" => "Jan".to_string(),
            "02" => "Feb".to_string(),
            "03" => "Mar".to_string(),
            "04" => "Apr".to_string(),
            "05" => "May".to_string(),
            "06" => "Jun".to_string(),
            "07" => "Jul".to_string(),
            "08" => "Aug".to_string(),
            "09" => "Sep".to_string(),
            "10" => "Oct".to_string(),
            "11" => "Nov".to_string(),
            "12" => "Dec".to_string(),
            _ => period_name.to_string(),
        }
    } else {
        period_name.to_string()
    }
}

// ============================================================================
// Route Handlers
// ============================================================================

/// POST /organizations/{org_id}/simulation/run
///
/// Requirements 15.1-15.4: Run simulation endpoint
#[allow(clippy::too_many_lines)]
#[axum::debug_handler]
async fn run_simulation(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    auth_user: AuthUser,
    Json(request): Json<RunSimulationRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
        return response;
    }

    // Validate date range
    if request.base_period_start > request.base_period_end {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_date_range",
                "message": "Base period start must be before or equal to end"
            })),
        )
            .into_response();
    }

    // Validate projection months
    if request.projection_months == 0 || request.projection_months > 60 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_projection_months",
                "message": "Projection months must be between 1 and 60"
            })),
        )
            .into_response();
    }

    // Parse growth rates
    let revenue_growth_rate = request
        .revenue_growth_rate
        .as_ref()
        .map_or(Decimal::ZERO, |s| parse_decimal(s));

    let expense_growth_rate = request
        .expense_growth_rate
        .as_ref()
        .map_or(Decimal::ZERO, |s| parse_decimal(s));

    // Validate growth rates (-100% to 1000%)
    let min_rate = Decimal::new(-1, 0);
    let max_rate = Decimal::new(10, 0);

    if revenue_growth_rate < min_rate || revenue_growth_rate > max_rate {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_growth_rate",
                "message": "Revenue growth rate must be between -1 (−100%) and 10 (1000%)"
            })),
        )
            .into_response();
    }

    if expense_growth_rate < min_rate || expense_growth_rate > max_rate {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_growth_rate",
                "message": "Expense growth rate must be between -1 (−100%) and 10 (1000%)"
            })),
        )
            .into_response();
    }

    // Parse account adjustments
    let account_adjustments = parse_account_adjustments(request.account_adjustments);

    // Validate account adjustment rates
    for rate in account_adjustments.values() {
        if *rate < min_rate || *rate > max_rate {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_growth_rate",
                    "message": "Account adjustment rates must be between -1 (−100%) and 10 (1000%)"
                })),
            )
                .into_response();
        }
    }

    let dimension_filters = request.dimension_filters.unwrap_or_default();

    // Build simulation params
    let params = SimulationParams {
        base_period_start: request.base_period_start,
        base_period_end: request.base_period_end,
        projection_months: request.projection_months,
        revenue_growth_rate,
        expense_growth_rate,
        account_adjustments,
        dimension_filters: dimension_filters.clone(),
    };

    // Query historical data
    let sim_repo = SimulationRepository::new((*state.db).clone());
    let historical_data = match sim_repo
        .query_historical_data(
            org_id,
            request.base_period_start,
            request.base_period_end,
            &dimension_filters,
        )
        .await
    {
        Ok(data) => data,
        Err(e) => {
            error!(error = %e, "Failed to query historical data");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to query historical data"
                })),
            )
                .into_response();
        }
    };

    // Convert to core types
    let core_historical_data: Vec<CoreHistoricalData> = historical_data
        .into_iter()
        .map(|h| CoreHistoricalData {
            account_id: h.account_id,
            account_code: h.account_code,
            account_name: h.account_name,
            account_type: h.account_type,
            monthly_amounts: h.monthly_amounts,
        })
        .collect();

    // Run simulation
    let result = SimulationEngine::run(&core_historical_data, &params);

    // Build monthly summary
    let mut monthly_totals: HashMap<String, (Decimal, Decimal)> = HashMap::new();
    for projection in &result.projections {
        let entry = monthly_totals
            .entry(projection.period_name.clone())
            .or_insert((Decimal::ZERO, Decimal::ZERO));

        match projection.account_type.as_str() {
            "revenue" => entry.0 += projection.projected_amount,
            "expense" => entry.1 += projection.projected_amount,
            _ => {}
        }
    }

    let mut monthly_summary: Vec<MonthlySummaryResponse> = monthly_totals
        .into_iter()
        .map(
            |(period_name, (revenue, expenses))| MonthlySummaryResponse {
                month: get_month_abbrev(&period_name),
                period_name: period_name.clone(),
                revenue: format_money(revenue),
                expenses: format_money(expenses),
                net_income: format_money(revenue - expenses),
            },
        )
        .collect();

    // Sort by period name
    monthly_summary.sort_by(|a, b| a.period_name.cmp(&b.period_name));

    // Calculate net profit margin
    let net_profit_margin = if result.annual_summary.total_projected_revenue.is_zero() {
        Decimal::ZERO
    } else {
        (result.annual_summary.projected_net_income / result.annual_summary.total_projected_revenue
            * Decimal::ONE_HUNDRED)
            .round_dp(2)
    };

    let response = SimulationResponse {
        simulation_id: result.simulation_id,
        parameters_hash: result.parameters_hash,
        cached: result.cached,
        projections: result
            .projections
            .iter()
            .map(|p| AccountProjectionResponse {
                period_name: p.period_name.clone(),
                period_start: p.period_start.to_string(),
                period_end: p.period_end.to_string(),
                account_id: p.account_id,
                account_code: p.account_code.clone(),
                account_name: p.account_name.clone(),
                account_type: p.account_type.clone(),
                baseline_amount: format_money(p.baseline_amount),
                projected_amount: format_money(p.projected_amount),
                change_percent: format_percent(p.change_percent),
            })
            .collect(),
        annual_summary: AnnualSummaryResponse {
            total_projected_revenue: format_money(result.annual_summary.total_projected_revenue),
            total_projected_expenses: format_money(result.annual_summary.total_projected_expenses),
            projected_net_income: format_money(result.annual_summary.projected_net_income),
            net_profit_margin: format_percent(net_profit_margin),
        },
        monthly_summary,
    };

    (StatusCode::OK, Json(response)).into_response()
}
