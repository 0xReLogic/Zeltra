//! Simulation scenario types.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use zeltra_shared::types::AccountId;

/// A what-if scenario for simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario name.
    pub name: String,
    /// Scenario description.
    pub description: Option<String>,
    /// Start date for the projection.
    pub start_date: NaiveDate,
    /// End date for the projection.
    pub end_date: NaiveDate,
    /// Adjustments to apply.
    pub adjustments: Vec<ScenarioAdjustment>,
}

/// An adjustment in a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAdjustment {
    /// Account to adjust.
    pub account_id: AccountId,
    /// Type of adjustment.
    pub adjustment_type: AdjustmentType,
    /// Adjustment value.
    pub value: Decimal,
}

/// Type of scenario adjustment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustmentType {
    /// Increase by a percentage.
    PercentageIncrease,
    /// Decrease by a percentage.
    PercentageDecrease,
    /// Set to a fixed amount.
    FixedAmount,
    /// Add a fixed amount.
    FixedIncrease,
    /// Subtract a fixed amount.
    FixedDecrease,
}

/// Result of a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// Scenario that was run.
    pub scenario_name: String,
    /// Projected balances by account.
    pub projected_balances: Vec<ProjectedBalance>,
    /// Summary metrics.
    pub summary: SimulationSummary,
}

/// Projected balance for an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedBalance {
    /// Account ID.
    pub account_id: AccountId,
    /// Account name.
    pub account_name: String,
    /// Current balance.
    pub current_balance: Decimal,
    /// Projected balance.
    pub projected_balance: Decimal,
    /// Change amount.
    pub change: Decimal,
    /// Change percentage.
    pub change_percentage: Decimal,
}

/// Summary of simulation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    /// Total projected revenue.
    pub total_revenue: Decimal,
    /// Total projected expenses.
    pub total_expenses: Decimal,
    /// Projected net income.
    pub net_income: Decimal,
    /// Projected cash position.
    pub cash_position: Decimal,
}
