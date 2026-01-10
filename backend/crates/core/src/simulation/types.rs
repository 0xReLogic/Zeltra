//! Simulation data types.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Parameters for running a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Start date of the base period for historical data.
    pub base_period_start: NaiveDate,
    /// End date of the base period for historical data.
    pub base_period_end: NaiveDate,
    /// Number of months to project into the future.
    pub projection_months: u32,
    /// Growth rate for revenue accounts (e.g., 0.10 for 10%).
    pub revenue_growth_rate: Decimal,
    /// Growth rate for expense accounts (e.g., 0.05 for 5%).
    pub expense_growth_rate: Decimal,
    /// Account-specific growth rate overrides.
    pub account_adjustments: HashMap<Uuid, Decimal>,
    /// Dimension value IDs to filter by.
    pub dimension_filters: Vec<Uuid>,
}

/// Historical account data for baseline calculation.
#[derive(Debug, Clone)]
pub struct HistoricalAccountData {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Account type (revenue, expense, etc.).
    pub account_type: String,
    /// Monthly amounts from the base period.
    pub monthly_amounts: Vec<Decimal>,
}

/// Projected amount for a single account and period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProjection {
    /// Period name (e.g., "2026-01").
    pub period_name: String,
    /// Period start date.
    pub period_start: NaiveDate,
    /// Period end date.
    pub period_end: NaiveDate,
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Account type.
    pub account_type: String,
    /// Baseline amount (average from historical data).
    pub baseline_amount: Decimal,
    /// Projected amount after applying growth.
    pub projected_amount: Decimal,
    /// Change percentage from baseline.
    pub change_percent: Decimal,
}

/// Annual summary of simulation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualSummary {
    /// Total projected revenue.
    pub total_projected_revenue: Decimal,
    /// Total projected expenses.
    pub total_projected_expenses: Decimal,
    /// Projected net income (revenue - expenses).
    pub projected_net_income: Decimal,
}

/// Result of a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Unique simulation ID.
    pub simulation_id: Uuid,
    /// Hash of the parameters (for caching).
    pub parameters_hash: String,
    /// Account projections.
    pub projections: Vec<AccountProjection>,
    /// Annual summary totals.
    pub annual_summary: AnnualSummary,
    /// Whether this result was returned from cache.
    pub cached: bool,
}
