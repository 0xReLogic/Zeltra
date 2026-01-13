//! Dashboard service for metrics and activity.
//!
//! Implements Requirements 4.1-4.7 for dashboard analytics.

use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use super::types::{
    ActivityEvent, ActivityPagination, BudgetStatus, BurnRate, CashFlowChart, CashPosition,
    CurrencyExposure, DashboardMetrics, DepartmentExpense, PendingApprovals, PeriodInfo,
    RecentActivityResponse, UserInfo, UtilizationChart,
};

/// Error types for dashboard operations.
#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    /// Organization not found.
    #[error("Organization not found: {0}")]
    OrganizationNotFound(Uuid),

    /// Fiscal period not found.
    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    /// Repository error.
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Cash flow data point for charts.
#[derive(Debug, Clone)]
pub struct CashFlowDataPoint {
    /// Month label (e.g., "Jan", "Feb").
    pub month: String,
    /// Period name (e.g., "2026-01").
    pub period_name: String,
    /// Total inflow.
    pub inflow: Decimal,
    /// Total outflow.
    pub outflow: Decimal,
}

/// Budget vs actual summary.
#[derive(Debug, Clone)]
pub struct BudgetVsActualSummary {
    /// Budget ID.
    pub budget_id: Option<Uuid>,
    /// Budget name.
    pub budget_name: Option<String>,
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
    /// Total actual spent.
    pub total_actual: Decimal,
    /// Variance (budgeted - actual).
    pub variance: Decimal,
    /// Variance percentage.
    pub variance_percent: Decimal,
    /// Line items.
    pub line_items: Vec<BudgetLineVariance>,
}

/// Budget line variance.
#[derive(Debug, Clone)]
pub struct BudgetLineVariance {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Budgeted amount.
    pub budgeted: Decimal,
    /// Actual amount.
    pub actual: Decimal,
    /// Variance.
    pub variance: Decimal,
    /// Variance percentage.
    pub variance_percent: Decimal,
}

/// Dashboard repository trait for dependency injection.
#[async_trait]
pub trait DashboardRepository: Send + Sync {
    /// Query cash position.
    async fn query_cash_position(
        &self,
        organization_id: Uuid,
        as_of: NaiveDate,
    ) -> Result<CashPositionData, DashboardError>;

    /// Query pending approvals.
    async fn query_pending_approvals(
        &self,
        organization_id: Uuid,
    ) -> Result<PendingApprovalsData, DashboardError>;

    /// Query burn rate (expenses in last N days).
    async fn query_burn_rate(
        &self,
        organization_id: Uuid,
        days: i32,
    ) -> Result<Decimal, DashboardError>;

    /// Query cash flow by month.
    async fn query_cash_flow(
        &self,
        organization_id: Uuid,
        months: u32,
    ) -> Result<Vec<CashFlowDataPoint>, DashboardError>;

    /// Query recent activity.
    async fn query_recent_activity(
        &self,
        organization_id: Uuid,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<(Vec<ActivityEventData>, Option<String>), DashboardError>;

    /// Query budget vs actual.
    async fn query_budget_vs_actual(
        &self,
        organization_id: Uuid,
        budget_id: Option<Uuid>,
    ) -> Result<BudgetVsActualData, DashboardError>;

    /// Query current fiscal period.
    async fn query_current_period(
        &self,
        organization_id: Uuid,
    ) -> Result<Option<PeriodData>, DashboardError>;
}

/// Cash position data from repository.
#[derive(Debug, Clone)]
pub struct CashPositionData {
    /// Current balance.
    pub balance: Decimal,
    /// Currency code.
    pub currency: String,
    /// Previous period balance.
    pub previous_balance: Decimal,
}

/// Pending approvals data from repository.
#[derive(Debug, Clone)]
pub struct PendingApprovalsData {
    /// Count of pending transactions.
    pub count: i32,
    /// Total amount.
    pub total_amount: Decimal,
}

/// Activity event data from repository.
#[derive(Debug, Clone)]
pub struct ActivityEventData {
    /// Event ID.
    pub id: Uuid,
    /// Event type.
    pub event_type: String,
    /// Action.
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: Uuid,
    /// Description.
    pub description: String,
    /// Amount.
    pub amount: Option<Decimal>,
    /// Currency.
    pub currency: Option<String>,
    /// User ID.
    pub user_id: Uuid,
    /// User full name.
    pub user_full_name: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<Utc>,
}

/// Budget vs actual data from repository.
#[derive(Debug, Clone)]
pub struct BudgetVsActualData {
    /// Budget ID.
    pub budget_id: Option<Uuid>,
    /// Budget name.
    pub budget_name: Option<String>,
    /// Total budgeted.
    pub total_budgeted: Decimal,
    /// Total actual.
    pub total_actual: Decimal,
    /// Line items.
    pub line_items: Vec<BudgetLineData>,
}

/// Budget line data from repository.
#[derive(Debug, Clone)]
pub struct BudgetLineData {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Budgeted amount.
    pub budgeted: Decimal,
    /// Actual amount.
    pub actual: Decimal,
}

/// Period data from repository.
#[derive(Debug, Clone)]
pub struct PeriodData {
    /// Period ID.
    pub id: Uuid,
    /// Period name.
    pub name: String,
}

/// Dashboard service for metrics and activity.
pub struct DashboardService<R: DashboardRepository> {
    repo: Arc<R>,
}

impl<R: DashboardRepository> DashboardService<R> {
    /// Creates a new dashboard service.
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Get dashboard metrics.
    ///
    /// Requirements: 4.1, 4.2, 4.3, 4.4
    ///
    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    pub async fn get_metrics(
        &self,
        org_id: Uuid,
        _period_id: Option<Uuid>,
    ) -> Result<DashboardMetrics, DashboardError> {
        let today = Utc::now().date_naive();

        // Get current period
        let period = self.repo.query_current_period(org_id).await?;
        let period_info = period.map_or_else(
            || PeriodInfo {
                id: Uuid::nil(),
                name: "Current".to_string(),
            },
            |p| PeriodInfo {
                id: p.id,
                name: p.name,
            },
        );

        // Get cash position (Property 10)
        let cash_data = self.repo.query_cash_position(org_id, today).await?;
        let change = cash_data.balance - cash_data.previous_balance;
        let change_percent = if cash_data.previous_balance.is_zero() {
            Decimal::ZERO
        } else {
            (change / cash_data.previous_balance * Decimal::from(100)).round_dp(2)
        };

        let cash_position = CashPosition {
            balance: cash_data.balance,
            currency: cash_data.currency,
            change_from_last_period: change,
            change_percent,
        };

        // Get burn rate (Property 11)
        let total_expenses_30d = self.repo.query_burn_rate(org_id, 30).await?;
        let daily_burn = (total_expenses_30d / Decimal::from(30)).round_dp(2);
        let monthly_burn = (daily_burn * Decimal::from(30)).round_dp(2);

        let burn_rate = BurnRate {
            daily: daily_burn,
            monthly: monthly_burn,
        };

        // Calculate runway (Property 12)
        let runway_days = if daily_burn.is_zero() {
            999 // Infinite runway
        } else {
            (cash_data.balance / daily_burn)
                .to_string()
                .parse::<i32>()
                .unwrap_or(999)
        };

        // Get pending approvals
        let pending_data = self.repo.query_pending_approvals(org_id).await?;
        let pending_approvals = PendingApprovals {
            count: pending_data.count,
            total_amount: pending_data.total_amount,
        };

        // Default budget status (will be populated if budget exists)
        let budget_status = BudgetStatus {
            total_budgeted: Decimal::ZERO,
            total_spent: Decimal::ZERO,
            utilization_percent: Decimal::ZERO,
            days_remaining: 0,
            projected_end_of_period: Decimal::ZERO,
        };

        Ok(DashboardMetrics {
            period: period_info,
            cash_position,
            burn_rate,
            runway_days,
            pending_approvals,
            budget_status,
            top_expenses_by_department: vec![],
            currency_exposure: vec![],
            cash_flow_chart: CashFlowChart {
                labels: vec![],
                inflow: vec![],
                outflow: vec![],
            },
            utilization_chart: UtilizationChart {
                labels: vec![],
                budgeted: vec![],
                actual: vec![],
            },
        })
    }

    /// Get cash flow data for charts.
    ///
    /// Requirements: 4.5
    ///
    /// Property 13: Cash Flow Aggregation
    ///
    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    pub async fn get_cash_flow(
        &self,
        org_id: Uuid,
        _period_id: Option<Uuid>,
        months: u32,
    ) -> Result<Vec<CashFlowDataPoint>, DashboardError> {
        self.repo.query_cash_flow(org_id, months).await
    }

    /// Get recent activity with cursor pagination.
    ///
    /// Requirements: 4.6
    ///
    /// Property 14: Cursor-Based Pagination Consistency
    ///
    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    pub async fn get_recent_activity(
        &self,
        org_id: Uuid,
        limit: u32,
        _activity_type: Option<String>,
        cursor: Option<String>,
    ) -> Result<RecentActivityResponse, DashboardError> {
        let (events_data, next_cursor) = self
            .repo
            .query_recent_activity(org_id, limit, cursor)
            .await?;

        let activities: Vec<ActivityEvent> = events_data
            .into_iter()
            .map(|e| ActivityEvent {
                id: e.id,
                event_type: e.event_type,
                action: e.action,
                entity_type: e.entity_type,
                entity_id: e.entity_id,
                description: e.description,
                amount: e.amount,
                currency: e.currency,
                user: UserInfo {
                    id: e.user_id,
                    full_name: e.user_full_name,
                },
                metadata: serde_json::Value::Null,
                timestamp: e.timestamp,
            })
            .collect();

        let has_more = next_cursor.is_some();

        Ok(RecentActivityResponse {
            activities,
            pagination: ActivityPagination {
                limit: i32::try_from(limit).unwrap_or(i32::MAX),
                has_more,
                next_cursor,
            },
        })
    }

    /// Get budget vs actual summary.
    ///
    /// Requirements: 4.7
    ///
    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    pub async fn get_budget_vs_actual(
        &self,
        org_id: Uuid,
        budget_id: Option<Uuid>,
    ) -> Result<BudgetVsActualSummary, DashboardError> {
        let data = self.repo.query_budget_vs_actual(org_id, budget_id).await?;

        let variance = data.total_budgeted - data.total_actual;
        let variance_percent = if data.total_budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (variance / data.total_budgeted * Decimal::from(100)).round_dp(2)
        };

        let line_items: Vec<BudgetLineVariance> = data
            .line_items
            .into_iter()
            .map(|l| {
                let line_variance = l.budgeted - l.actual;
                let line_variance_percent = if l.budgeted.is_zero() {
                    Decimal::ZERO
                } else {
                    (line_variance / l.budgeted * Decimal::from(100)).round_dp(2)
                };

                BudgetLineVariance {
                    account_id: l.account_id,
                    account_code: l.account_code,
                    account_name: l.account_name,
                    budgeted: l.budgeted,
                    actual: l.actual,
                    variance: line_variance,
                    variance_percent: line_variance_percent,
                }
            })
            .collect();

        Ok(BudgetVsActualSummary {
            budget_id: data.budget_id,
            budget_name: data.budget_name,
            total_budgeted: data.total_budgeted,
            total_actual: data.total_actual,
            variance,
            variance_percent,
            line_items,
        })
    }
}
