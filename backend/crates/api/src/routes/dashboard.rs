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
            "/organizations/{org_id}/dashboard/recent-activity",
            get(get_recent_activity),
        )
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Query parameters for dashboard metrics.
#[derive(Debug, Deserialize)]
pub struct DashboardMetricsQuery {
    /// Fiscal period ID for budget status.
    pub period_id: Option<Uuid>,
}

/// Query parameters for recent activity.
#[derive(Debug, Deserialize)]
pub struct RecentActivityQuery {
    /// Maximum number of items to return.
    pub limit: Option<u64>,
    /// Activity type filter.
    #[serde(rename = "type")]
    pub activity_type: Option<String>,
    /// Cursor for pagination.
    pub cursor: Option<String>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Response for dashboard metrics.
#[derive(Debug, Serialize)]
pub struct DashboardMetricsResponse {
    /// Period info.
    pub period: Option<PeriodInfo>,
    /// Cash position.
    pub cash_position: CashPositionResponse,
    /// Burn rate.
    pub burn_rate: BurnRateResponse,
    /// Runway days.
    pub runway_days: i32,
    /// Pending approvals.
    pub pending_approvals: PendingApprovalsResponse,
}

/// Period info.
#[derive(Debug, Serialize)]
pub struct PeriodInfo {
    /// Period ID.
    pub id: Uuid,
    /// Period name.
    pub name: String,
}

/// Cash position response.
#[derive(Debug, Serialize)]
pub struct CashPositionResponse {
    /// Current balance.
    pub balance: String,
    /// Currency.
    pub currency: String,
    /// Change from last period.
    pub change_from_last_period: String,
    /// Change percentage.
    pub change_percent: f64,
}

/// Burn rate response.
#[derive(Debug, Serialize)]
pub struct BurnRateResponse {
    /// Daily burn rate.
    pub daily: String,
    /// Monthly burn rate.
    pub monthly: String,
}

/// Pending approvals response.
#[derive(Debug, Serialize)]
pub struct PendingApprovalsResponse {
    /// Number of pending items.
    pub count: i32,
    /// Total amount.
    pub total_amount: String,
}

/// Response for recent activity.
#[derive(Debug, Serialize)]
pub struct RecentActivityResponse {
    /// Activity items.
    pub activities: Vec<ActivityItemResponse>,
    /// Pagination info.
    pub pagination: PaginationInfo,
}

/// Activity item response.
#[derive(Debug, Serialize)]
pub struct ActivityItemResponse {
    /// Activity ID.
    pub id: Uuid,
    /// Activity type.
    #[serde(rename = "type")]
    pub activity_type: String,
    /// Action performed.
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: Uuid,
    /// Description.
    pub description: String,
    /// Amount (if applicable).
    pub amount: Option<String>,
    /// Currency (if applicable).
    pub currency: Option<String>,
    /// User info.
    pub user: UserInfo,
    /// Timestamp.
    pub timestamp: String,
}

/// User info.
#[derive(Debug, Serialize)]
pub struct UserInfo {
    /// User ID.
    pub id: Uuid,
    /// Full name.
    pub full_name: String,
}

/// Pagination info.
#[derive(Debug, Serialize)]
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
#[axum::debug_handler]
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

    // Calculate burn rate (simplified: monthly expenses / 30)
    // In a real implementation, this would query actual expense data
    let monthly_burn = Decimal::ZERO; // TODO: Calculate from actual expenses
    let daily_burn = monthly_burn / Decimal::from(30);

    // Calculate runway (cash / daily burn)
    let runway_days = if daily_burn.is_zero() {
        999 // Infinite runway if no burn
    } else {
        (cash_position.balance / daily_burn)
            .to_string()
            .parse::<i32>()
            .unwrap_or(999)
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
                user: UserInfo {
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
