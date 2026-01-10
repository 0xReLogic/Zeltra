//! Dashboard data types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Dashboard metrics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    /// Current period info.
    pub period: PeriodInfo,
    /// Cash position.
    pub cash_position: CashPosition,
    /// Burn rate.
    pub burn_rate: BurnRate,
    /// Runway in days.
    pub runway_days: i32,
    /// Pending approvals.
    pub pending_approvals: PendingApprovals,
    /// Budget status.
    pub budget_status: BudgetStatus,
    /// Top expenses by department.
    pub top_expenses_by_department: Vec<DepartmentExpense>,
    /// Currency exposure.
    pub currency_exposure: Vec<CurrencyExposure>,
    /// Cash flow chart data.
    pub cash_flow_chart: CashFlowChart,
    /// Utilization chart data.
    pub utilization_chart: UtilizationChart,
}

/// Period information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInfo {
    /// Period ID.
    pub id: Uuid,
    /// Period name.
    pub name: String,
}

/// Cash position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPosition {
    /// Current balance.
    pub balance: Decimal,
    /// Currency code.
    pub currency: String,
    /// Change from last period.
    pub change_from_last_period: Decimal,
    /// Change percentage.
    pub change_percent: Decimal,
}

/// Burn rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRate {
    /// Daily burn rate.
    pub daily: Decimal,
    /// Monthly burn rate.
    pub monthly: Decimal,
}

/// Pending approvals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApprovals {
    /// Number of pending approvals.
    pub count: i32,
    /// Total amount pending.
    pub total_amount: Decimal,
}

/// Budget status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
    /// Total spent amount.
    pub total_spent: Decimal,
    /// Utilization percentage.
    pub utilization_percent: Decimal,
    /// Days remaining in period.
    pub days_remaining: i32,
    /// Projected end of period spending.
    pub projected_end_of_period: Decimal,
}

/// Department expense.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentExpense {
    /// Department name.
    pub department: String,
    /// Expense amount.
    pub amount: Decimal,
    /// Percentage of total.
    pub percent: Decimal,
}

/// Currency exposure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExposure {
    /// Currency code.
    pub currency: String,
    /// Balance in currency.
    pub balance: Decimal,
    /// Functional value.
    pub functional_value: Decimal,
    /// Percentage of total.
    pub percent: Decimal,
}

/// Cash flow chart data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowChart {
    /// Labels (period names).
    pub labels: Vec<String>,
    /// Inflow amounts.
    pub inflow: Vec<Decimal>,
    /// Outflow amounts.
    pub outflow: Vec<Decimal>,
}

/// Utilization chart data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationChart {
    /// Labels (period names).
    pub labels: Vec<String>,
    /// Budgeted amounts.
    pub budgeted: Vec<Decimal>,
    /// Actual amounts.
    pub actual: Vec<Decimal>,
}

/// Activity event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// Event ID.
    pub id: Uuid,
    /// Event type.
    pub event_type: String,
    /// Action performed.
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: Uuid,
    /// Description.
    pub description: String,
    /// Amount (if applicable).
    pub amount: Option<Decimal>,
    /// Currency (if applicable).
    pub currency: Option<String>,
    /// User who performed the action.
    pub user: UserInfo,
    /// Additional metadata.
    pub metadata: serde_json::Value,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// User information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID.
    pub id: Uuid,
    /// Full name.
    pub full_name: String,
}

/// Recent activity response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivityResponse {
    /// Activity events.
    pub activities: Vec<ActivityEvent>,
    /// Pagination info.
    pub pagination: ActivityPagination,
}

/// Activity pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPagination {
    /// Limit.
    pub limit: i32,
    /// Whether there are more results.
    pub has_more: bool,
    /// Next cursor.
    pub next_cursor: Option<String>,
}
