//! Dashboard routes.
//!
//! Implements Requirements 16.1, 17.1 for Dashboard API endpoints.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{OrganizationRepository, repositories::dashboard::DashboardRepository};

/// Creates the dashboard routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/dashboard/metrics",
            get(get_dashboard_metrics),
        )
        .route(
            "/organizations/{org_id}/dashboard/cash-flow",
            get(get_cash_flow),
        )
        .route(
            "/organizations/{org_id}/dashboard/recent-activity",
            get(get_recent_activity),
        )
        .route(
            "/organizations/{org_id}/dashboard/budget-vs-actual",
            get(get_dashboard_budget_vs_actual),
        )
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for dashboard metrics.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct DashboardMetricsQuery {
    /// Fiscal period ID for budget status.
    pub period_id: Option<Uuid>,
    /// Entity ID to filter by (optional).
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub entity_id: Option<Uuid>,
    /// Generate consolidated metrics for all entities (optional).
    #[param(example = false)]
    pub consolidated: Option<bool>,
}

/// Query parameters for recent activity.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct RecentActivityQuery {
    /// Maximum number of items to return.
    #[param(example = 10)]
    pub limit: Option<u64>,
    /// Activity type filter.
    #[serde(rename = "type")]
    #[param(example = "transaction")]
    pub activity_type: Option<String>,
    /// Cursor for pagination.
    pub cursor: Option<String>,
    /// Entity ID to filter by (optional).
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub entity_id: Option<Uuid>,
}

/// Query parameters for cash flow.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CashFlowQuery {
    /// Number of months to include.
    #[param(example = 6)]
    pub months: Option<u32>,
    /// Fiscal period ID.
    pub period_id: Option<Uuid>,
    /// Entity ID to filter by (optional).
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub entity_id: Option<Uuid>,
    /// Generate consolidated data for all entities (optional).
    #[param(example = false)]
    pub consolidated: Option<bool>,
}

/// Query parameters for budget vs actual.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct BudgetVsActualQuery {
    /// Budget ID (optional, uses first active budget if not provided).
    pub budget_id: Option<Uuid>,
    /// Entity ID to filter by (optional).
    #[param(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub entity_id: Option<Uuid>,
    /// Generate consolidated data for all entities (optional).
    #[param(example = false)]
    pub consolidated: Option<bool>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Response for dashboard metrics.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DashboardMetricsResponse {
    /// Period info.
    pub period: Option<PeriodInfo>,
    /// Cash position.
    pub cash_position: CashPositionResponse,
    /// Burn rate.
    pub burn_rate: BurnRateResponse,
    /// Runway days.
    #[schema(example = 180)]
    pub runway_days: i32,
    /// Pending approvals.
    pub pending_approvals: PendingApprovalsResponse,
}

/// Period info.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PeriodInfo {
    /// Period ID.
    pub id: Uuid,
    /// Period name.
    #[schema(example = "December 2023")]
    pub name: String,
}

/// Cash position response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CashPositionResponse {
    /// Current balance.
    #[schema(example = "50000.0000")]
    pub balance: String,
    /// Currency.
    #[schema(example = "USD")]
    pub currency: String,
    /// Change from last period.
    #[schema(example = "5000.0000")]
    pub change_from_last_period: String,
    /// Change percentage.
    #[schema(example = 11.11)]
    pub change_percent: f64,
}

/// Burn rate response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BurnRateResponse {
    /// Daily burn rate.
    #[schema(example = "100.0000")]
    pub daily: String,
    /// Monthly burn rate.
    #[schema(example = "3000.0000")]
    pub monthly: String,
}

/// Pending approvals response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PendingApprovalsResponse {
    /// Number of pending items.
    #[schema(example = 5)]
    pub count: i32,
    /// Total amount.
    #[schema(example = "1250.0000")]
    pub total_amount: String,
}

/// Response for recent activity.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RecentActivityResponse {
    /// Activity items.
    pub activities: Vec<ActivityItemResponse>,
    /// Pagination info.
    pub pagination: PaginationInfo,
}

/// Activity item response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ActivityItemResponse {
    /// Activity ID.
    pub id: Uuid,
    /// Activity type.
    #[serde(rename = "type")]
    #[schema(example = "transaction_create")]
    pub activity_type: String,
    /// Action performed.
    #[schema(example = "create")]
    pub action: String,
    /// Entity type.
    #[schema(example = "transaction")]
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: Uuid,
    /// Description.
    #[schema(example = "New transaction created for 120.00 USD")]
    pub description: String,
    /// Amount (if applicable).
    pub amount: Option<String>,
    /// Currency (if applicable).
    pub currency: Option<String>,
    /// User info.
    pub user: DashboardUserInfo,
    /// Timestamp.
    pub timestamp: String,
}

/// Dashboard user info (simplified).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DashboardUserInfo {
    /// User ID.
    pub id: Uuid,
    /// Full name.
    #[schema(example = "John Doe")]
    pub full_name: String,
}

/// Pagination info.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationInfo {
    /// Limit.
    pub limit: u64,
    /// Has more results.
    pub has_more: bool,
    /// Next cursor.
    pub next_cursor: Option<String>,
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

// ============================================================================
// Route Handlers
// ============================================================================

/// GET /organizations/{org_id}/dashboard/metrics
///
/// Requirement 16.1: Dashboard metrics endpoint
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/dashboard/metrics",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        DashboardMetricsQuery
    ),
    responses(
        (status = 200, description = "Dashboard metrics", body = DashboardMetricsResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Dashboard",
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
#[allow(clippy::cast_precision_loss)] // UI percentage display
async fn get_dashboard_metrics(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<DashboardMetricsQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
        return response;
    }

    let dashboard_repo = DashboardRepository::new((*state.db).clone());
    let today = chrono::Utc::now().date_naive();

    // Query cash position
    let cash_position = match dashboard_repo.query_cash_position(org_id, today).await {
        Ok(cp) => cp,
        Err(e) => {
            error!(error = %e, "Failed to query cash position");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get dashboard metrics"
                })),
            )
                .into_response();
        }
    };

    // Query pending approvals
    let pending_approvals = match dashboard_repo.query_pending_approvals(org_id).await {
        Ok(pa) => pa,
        Err(e) => {
            error!(error = %e, "Failed to query pending approvals");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get dashboard metrics"
                })),
            )
                .into_response();
        }
    };

    // Calculate burn rate from actual expenses (last 30 days)
    let total_expenses_30d = match dashboard_repo.query_burn_rate(org_id, 30).await {
        Ok(total) => total,
        Err(e) => {
            error!(error = %e, "Failed to query burn rate");
            Decimal::ZERO
        }
    };
    let daily_burn = (total_expenses_30d / Decimal::from(30)).round_dp(4);
    let monthly_burn = (daily_burn * Decimal::from(30)).round_dp(4);

    // Calculate runway (cash / daily burn)
    let runway_days = if daily_burn.is_zero() {
        999 // Infinite runway if no burn
    } else {
        let runway = cash_position.balance / daily_burn;
        i32::try_from(runway.to_string().parse::<i64>().unwrap_or(999))
            .unwrap_or(999)
            .min(999)
    };

    // Get period info if provided
    let period_info = query.period_id.map(|id| PeriodInfo {
        id,
        name: "Current Period".to_string(), // TODO: Query actual period name
    });

    let response = DashboardMetricsResponse {
        period: period_info,
        cash_position: CashPositionResponse {
            balance: format_money(cash_position.balance),
            currency: cash_position.currency,
            change_from_last_period: format_money(cash_position.change_from_last_period),
            change_percent: cash_position
                .change_percent
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0),
        },
        burn_rate: BurnRateResponse {
            daily: format_money(daily_burn),
            monthly: format_money(monthly_burn),
        },
        runway_days,
        pending_approvals: PendingApprovalsResponse {
            count: pending_approvals.count,
            total_amount: format_money(pending_approvals.total_amount),
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /organizations/{org_id}/dashboard/recent-activity
///
/// Requirement 17.1: Recent activity endpoint
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/dashboard/recent-activity",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        RecentActivityQuery
    ),
    responses(
        (status = 200, description = "Recent activity logs", body = RecentActivityResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Dashboard",
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
async fn get_recent_activity(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<RecentActivityQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
        return response;
    }

    let dashboard_repo = DashboardRepository::new((*state.db).clone());
    let limit = query.limit.unwrap_or(10).min(50);

    // Query recent activity
    let (activities, pagination) = match dashboard_repo
        .query_recent_activity(org_id, limit, query.cursor)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            error!(error = %e, "Failed to query recent activity");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get recent activity"
                })),
            )
                .into_response();
        }
    };

    // Filter by type if specified
    let filtered_activities: Vec<_> = if let Some(ref activity_type) = query.activity_type {
        if activity_type == "all" {
            activities
        } else {
            activities
                .into_iter()
                .filter(|a| a.event_type == *activity_type)
                .collect()
        }
    } else {
        activities
    };

    let response = RecentActivityResponse {
        activities: filtered_activities
            .iter()
            .map(|a| ActivityItemResponse {
                id: a.id,
                activity_type: format!("{}_{}", a.entity_type, a.action),
                action: a.action.clone(),
                entity_type: a.entity_type.clone(),
                entity_id: a.entity_id,
                description: a.description.clone(),
                amount: a.amount.map(format_money),
                currency: a.currency.clone(),
                user: DashboardUserInfo {
                    id: a.user_id,
                    full_name: a.user_full_name.clone(),
                },
                timestamp: a.timestamp.to_rfc3339(),
            })
            .collect(),
        pagination: PaginationInfo {
            limit: pagination.limit,
            has_more: pagination.has_more,
            next_cursor: pagination.next_cursor,
        },
    };

    (StatusCode::OK, Json(response)).into_response()
}

// ============================================================================
// Cash Flow Response Types
// ============================================================================

/// Response for cash flow data.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CashFlowResponse {
    /// Cash flow data points.
    pub data: Vec<CashFlowDataPoint>,
}

/// Cash flow data point.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CashFlowDataPoint {
    /// Month label (e.g., "Jan").
    #[schema(example = "Jan")]
    pub month: String,
    /// Period name (e.g., "2026-01").
    #[schema(example = "2026-01")]
    pub period_name: String,
    /// Total inflow.
    pub inflow: String,
    /// Total outflow.
    pub outflow: String,
    /// Net cash flow.
    pub net: String,
}

// ============================================================================
// Budget vs Actual Response Types
// ============================================================================

/// Response for budget vs actual data.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BudgetVsActualResponse {
    /// Budget ID.
    pub budget_id: Option<Uuid>,
    /// Budget name.
    #[schema(example = "Annual Budget 2023")]
    pub budget_name: Option<String>,
    /// Summary.
    pub summary: BudgetSummary,
    /// Line items.
    pub line_items: Vec<BudgetLineItemResponse>,
}

/// Budget summary.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BudgetSummary {
    /// Total budgeted.
    pub total_budgeted: String,
    /// Total actual.
    pub total_actual: String,
    /// Variance (budgeted - actual).
    pub variance: String,
    /// Variance percentage.
    pub variance_percent: f64,
}

/// Budget line item response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BudgetLineItemResponse {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    #[schema(example = "6001")]
    pub account_code: String,
    /// Account name.
    #[schema(example = "Office Expenses")]
    pub account_name: String,
    /// Budgeted amount.
    pub budgeted: String,
    /// Actual amount.
    pub actual: String,
    /// Variance.
    pub variance: String,
    /// Variance percentage.
    pub variance_percent: f64,
}

// ============================================================================
// Cash Flow Handler
// ============================================================================

/// GET /organizations/{org_id}/dashboard/cash-flow
///
/// Requirement 4.5: Cash flow chart data
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/dashboard/cash-flow",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        CashFlowQuery
    ),
    responses(
        (status = 200, description = "Cash flow historical data", body = CashFlowResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Dashboard",
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
async fn get_cash_flow(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<CashFlowQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
        return response;
    }

    let dashboard_repo = DashboardRepository::new((*state.db).clone());
    let months = query.months.unwrap_or(6).min(12);

    // Query cash flow by month
    let cash_flow_data = match dashboard_repo
        .query_cash_flow_by_month(org_id, months)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            error!(error = %e, "Failed to query cash flow");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get cash flow data"
                })),
            )
                .into_response();
        }
    };

    let data: Vec<CashFlowDataPoint> = cash_flow_data
        .into_iter()
        .map(|d| {
            let net = d.inflow - d.outflow;
            CashFlowDataPoint {
                month: d.month,
                period_name: d.period_name,
                inflow: format_money(d.inflow),
                outflow: format_money(d.outflow),
                net: format_money(net),
            }
        })
        .collect();

    let response = CashFlowResponse { data };

    (StatusCode::OK, Json(response)).into_response()
}

// ============================================================================
// Budget vs Actual Handler
// ============================================================================

/// GET /organizations/{org_id}/dashboard/budget-vs-actual
///
/// Requirement 4.7: Budget vs actual summary
#[utoipa::path(
    get,
    path = "/organizations/{org_id}/dashboard/budget-vs-actual",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        BudgetVsActualQuery
    ),
    responses(
        (status = 200, description = "Budget vs actual performance data", body = BudgetVsActualResponse),
        (status = 403, description = "Forbidden")
    ),
    tag = "Dashboard",
    security(("bearerAuth" = []))
)]
#[axum::debug_handler]
#[allow(clippy::cast_precision_loss)] // UI percentage display
async fn get_dashboard_budget_vs_actual(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<BudgetVsActualQuery>,
    auth_user: AuthUser,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth_user.user_id()).await {
        return response;
    }

    let dashboard_repo = DashboardRepository::new((*state.db).clone());

    // Query budget vs actual
    let budget_data = match dashboard_repo
        .query_budget_vs_actual(org_id, query.budget_id)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            error!(error = %e, "Failed to query budget vs actual");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Failed to get budget vs actual data"
                })),
            )
                .into_response();
        }
    };

    let variance = budget_data.total_budgeted - budget_data.total_actual;
    let variance_percent = if budget_data.total_budgeted.is_zero() {
        0.0
    } else {
        (variance / budget_data.total_budgeted * Decimal::from(100))
            .round_dp(2)
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0)
    };

    let line_items: Vec<BudgetLineItemResponse> = budget_data
        .line_items
        .into_iter()
        .map(|l| {
            let line_variance = l.budgeted - l.actual;
            let line_variance_percent = if l.budgeted.is_zero() {
                0.0
            } else {
                (line_variance / l.budgeted * Decimal::from(100))
                    .round_dp(2)
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            };

            BudgetLineItemResponse {
                account_id: l.account_id,
                account_code: l.account_code,
                account_name: l.account_name,
                budgeted: format_money(l.budgeted),
                actual: format_money(l.actual),
                variance: format_money(line_variance),
                variance_percent: line_variance_percent,
            }
        })
        .collect();

    let response = BudgetVsActualResponse {
        budget_id: budget_data.budget_id,
        budget_name: budget_data.budget_name,
        summary: BudgetSummary {
            total_budgeted: format_money(budget_data.total_budgeted),
            total_actual: format_money(budget_data.total_actual),
            variance: format_money(variance),
            variance_percent,
        },
        line_items,
    };

    (StatusCode::OK, Json(response)).into_response()
}
