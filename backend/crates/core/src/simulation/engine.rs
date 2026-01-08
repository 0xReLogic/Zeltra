//! Simulation engine for running projections.

use super::scenario::{AdjustmentType, Scenario, ScenarioResult, SimulationSummary};
use rust_decimal::Decimal;

/// Engine for running what-if simulations.
pub struct SimulationEngine;

impl SimulationEngine {
    /// Creates a new simulation engine.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Applies an adjustment to a value.
    #[must_use]
    pub fn apply_adjustment(
        value: Decimal,
        adjustment_type: AdjustmentType,
        adjustment_value: Decimal,
    ) -> Decimal {
        match adjustment_type {
            AdjustmentType::PercentageIncrease => {
                value + (value * adjustment_value / Decimal::ONE_HUNDRED)
            }
            AdjustmentType::PercentageDecrease => {
                value - (value * adjustment_value / Decimal::ONE_HUNDRED)
            }
            AdjustmentType::FixedAmount => adjustment_value,
            AdjustmentType::FixedIncrease => value + adjustment_value,
            AdjustmentType::FixedDecrease => value - adjustment_value,
        }
    }

    /// Runs a simulation scenario.
    ///
    /// Note: This is a placeholder. The actual implementation will need
    /// account balances from the database.
    #[must_use]
    pub fn run(&self, scenario: &Scenario) -> ScenarioResult {
        // Placeholder implementation
        ScenarioResult {
            scenario_name: scenario.name.clone(),
            projected_balances: Vec::new(),
            summary: SimulationSummary {
                total_revenue: Decimal::ZERO,
                total_expenses: Decimal::ZERO,
                net_income: Decimal::ZERO,
                cash_position: Decimal::ZERO,
            },
        }
    }
}

impl Default for SimulationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentage_increase() {
        let result = SimulationEngine::apply_adjustment(
            Decimal::new(100, 0),
            AdjustmentType::PercentageIncrease,
            Decimal::new(10, 0),
        );
        assert_eq!(result, Decimal::new(110, 0));
    }

    #[test]
    fn test_percentage_decrease() {
        let result = SimulationEngine::apply_adjustment(
            Decimal::new(100, 0),
            AdjustmentType::PercentageDecrease,
            Decimal::new(10, 0),
        );
        assert_eq!(result, Decimal::new(90, 0));
    }

    #[test]
    fn test_fixed_amount() {
        let result = SimulationEngine::apply_adjustment(
            Decimal::new(100, 0),
            AdjustmentType::FixedAmount,
            Decimal::new(50, 0),
        );
        assert_eq!(result, Decimal::new(50, 0));
    }
}
