//! Budget data types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Budget type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetType {
    /// Annual budget covering full fiscal year.
    Annual,
    /// Quarterly budget for a specific quarter.
    Quarterly,
    /// Monthly budget for a specific month.
    Monthly,
    /// Project-based budget.
    Project,
}

/// A budget record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Budget ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Fiscal year ID.
    pub fiscal_year_id: Uuid,
    /// Budget name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Budget type.
    pub budget_type: BudgetType,
    /// Currency code.
    pub currency: String,
    /// Whether the budget is active.
    pub is_active: bool,
    /// Whether the budget is locked (no modifications allowed).
    pub is_locked: bool,
    /// User who created the budget.
    pub created_by: Uuid,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// A budget line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLine {
    /// Budget line ID.
    pub id: Uuid,
    /// Parent budget ID.
    pub budget_id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Budgeted amount.
    pub amount: Decimal,
    /// Optional notes.
    pub notes: Option<String>,
    /// Associated dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Dimension information for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionInfo {
    /// Dimension type (e.g., "DEPARTMENT", "PROJECT").
    pub dimension_type: String,
    /// Dimension value code.
    pub code: String,
    /// Dimension value name.
    pub name: String,
}

/// Budget line with calculated actual amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLineWithActual {
    /// Budget line ID.
    pub id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Period name.
    pub period_name: String,
    /// Budgeted amount.
    pub budgeted: Decimal,
    /// Actual amount from ledger entries.
    pub actual: Decimal,
    /// Variance (budgeted - actual for expenses, actual - budgeted for revenue).
    pub variance: Decimal,
    /// Utilization percentage (actual / budgeted * 100).
    pub utilization_percent: Decimal,
    /// Variance status.
    pub status: VarianceStatus,
    /// Associated dimensions.
    pub dimensions: Vec<DimensionInfo>,
}

/// Budget summary for list view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    /// Budget ID.
    pub id: Uuid,
    /// Budget name.
    pub name: String,
    /// Fiscal year name.
    pub fiscal_year: String,
    /// Budget type.
    pub budget_type: BudgetType,
    /// Currency code.
    pub currency: String,
    /// Whether the budget is locked.
    pub is_locked: bool,
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
    /// Total actual amount.
    pub total_actual: Decimal,
    /// Total variance.
    pub total_variance: Decimal,
}

/// Budget vs actual report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVsActualReport {
    /// Budget ID.
    pub budget_id: Uuid,
    /// Budget name.
    pub budget_name: String,
    /// Period description.
    pub period: String,
    /// Budget lines with actual amounts.
    pub lines: Vec<BudgetLineWithActual>,
    /// Summary totals.
    pub summary: BudgetVsActualSummary,
}

/// Budget vs actual summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVsActualSummary {
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
    /// Total actual amount.
    pub total_actual: Decimal,
    /// Total variance.
    pub total_variance: Decimal,
    /// Overall utilization percentage.
    pub overall_utilization: Decimal,
}

/// Variance calculation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceResult {
    /// Budgeted amount.
    pub budgeted: Decimal,
    /// Actual amount.
    pub actual: Decimal,
    /// Variance amount.
    pub variance: Decimal,
    /// Variance percentage.
    pub variance_percent: Decimal,
    /// Utilization percentage.
    pub utilization_percent: Decimal,
    /// Variance status.
    pub status: VarianceStatus,
}

/// Variance status classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarianceStatus {
    /// Favorable variance (under budget for expenses, over target for revenue).
    Favorable,
    /// Unfavorable variance (over budget for expenses, under target for revenue).
    Unfavorable,
    /// On budget (no variance).
    OnBudget,
}

/// Input for creating a new budget.
#[derive(Debug, Clone)]
pub struct CreateBudgetInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Fiscal year ID.
    pub fiscal_year_id: Uuid,
    /// Budget name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Budget type.
    pub budget_type: BudgetType,
    /// Currency code.
    pub currency: String,
    /// User creating the budget.
    pub created_by: Uuid,
}

/// Input for creating a budget line.
#[derive(Debug, Clone)]
pub struct CreateBudgetLineInput {
    /// Parent budget ID.
    pub budget_id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Budgeted amount.
    pub amount: Decimal,
    /// Optional notes.
    pub notes: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}
