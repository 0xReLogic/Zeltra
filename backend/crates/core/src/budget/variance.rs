//! Budget variance calculations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of variance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VarianceType {
    /// Actual is under budget (favorable for expenses).
    Favorable,
    /// Actual is over budget (unfavorable for expenses).
    Unfavorable,
    /// No variance.
    None,
}

/// Budget vs actual variance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVariance {
    /// Budgeted amount.
    pub budget_amount: Decimal,
    /// Actual amount.
    pub actual_amount: Decimal,
    /// Variance amount (budget - actual).
    pub variance_amount: Decimal,
    /// Variance percentage.
    pub variance_percentage: Decimal,
    /// Type of variance.
    pub variance_type: VarianceType,
}

impl BudgetVariance {
    /// Calculates variance for an expense account.
    ///
    /// For expenses: under budget is favorable, over budget is unfavorable.
    #[must_use]
    pub fn for_expense(budget: Decimal, actual: Decimal) -> Self {
        let variance = budget - actual;
        let percentage = if budget.is_zero() {
            Decimal::ZERO
        } else {
            (variance / budget) * Decimal::ONE_HUNDRED
        };

        let variance_type = if variance.is_zero() {
            VarianceType::None
        } else if variance.is_sign_positive() {
            VarianceType::Favorable
        } else {
            VarianceType::Unfavorable
        };

        Self {
            budget_amount: budget,
            actual_amount: actual,
            variance_amount: variance,
            variance_percentage: percentage,
            variance_type,
        }
    }

    /// Calculates variance for a revenue account.
    ///
    /// For revenue: over budget is favorable, under budget is unfavorable.
    #[must_use]
    pub fn for_revenue(budget: Decimal, actual: Decimal) -> Self {
        let variance = actual - budget;
        let percentage = if budget.is_zero() {
            Decimal::ZERO
        } else {
            (variance / budget) * Decimal::ONE_HUNDRED
        };

        let variance_type = if variance.is_zero() {
            VarianceType::None
        } else if variance.is_sign_positive() {
            VarianceType::Favorable
        } else {
            VarianceType::Unfavorable
        };

        Self {
            budget_amount: budget,
            actual_amount: actual,
            variance_amount: variance,
            variance_percentage: percentage,
            variance_type,
        }
    }
}
