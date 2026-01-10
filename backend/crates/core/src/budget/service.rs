//! Budget service for variance calculation and validation.

use rust_decimal::Decimal;

use super::error::BudgetError;
use super::types::{Budget, VarianceResult, VarianceStatus};

/// Budget service for business logic.
pub struct BudgetService;

impl BudgetService {
    /// Calculate variance between budgeted and actual amounts.
    ///
    /// For expense accounts: variance = budgeted - actual
    ///   - Positive variance (under budget) is favorable
    ///   - Negative variance (over budget) is unfavorable
    ///
    /// For revenue accounts: variance = actual - budgeted
    ///   - Positive variance (over target) is favorable
    ///   - Negative variance (under target) is unfavorable
    #[must_use]
    pub fn calculate_variance(
        budgeted: Decimal,
        actual: Decimal,
        account_type: &str,
    ) -> VarianceResult {
        let variance = match account_type {
            "revenue" => actual - budgeted,
            // expense, asset, liability, equity - all use budgeted - actual
            _ => budgeted - actual,
        };

        let status = match variance.cmp(&Decimal::ZERO) {
            std::cmp::Ordering::Greater => VarianceStatus::Favorable,
            std::cmp::Ordering::Less => VarianceStatus::Unfavorable,
            std::cmp::Ordering::Equal => VarianceStatus::OnBudget,
        };

        let variance_percent = if budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (variance / budgeted * Decimal::ONE_HUNDRED).round_dp(2)
        };

        let utilization_percent = if budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (actual / budgeted * Decimal::ONE_HUNDRED).round_dp(2)
        };

        VarianceResult {
            budgeted,
            actual,
            variance,
            variance_percent,
            utilization_percent,
            status,
        }
    }

    /// Validate budget line creation.
    ///
    /// # Errors
    ///
    /// Returns `BudgetError::BudgetLocked` if the budget is locked.
    /// Returns `BudgetError::NegativeAmount` if the amount is negative.
    pub fn validate_budget_line(budget: &Budget, amount: Decimal) -> Result<(), BudgetError> {
        if budget.is_locked {
            return Err(BudgetError::BudgetLocked);
        }

        if amount < Decimal::ZERO {
            return Err(BudgetError::NegativeAmount);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_expense_variance_favorable() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(800), "expense");

        assert_eq!(result.budgeted, dec!(1000));
        assert_eq!(result.actual, dec!(800));
        assert_eq!(result.variance, dec!(200));
        assert_eq!(result.variance_percent, dec!(20.00));
        assert_eq!(result.utilization_percent, dec!(80.00));
        assert_eq!(result.status, VarianceStatus::Favorable);
    }

    #[test]
    fn test_expense_variance_unfavorable() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1200), "expense");

        assert_eq!(result.variance, dec!(-200));
        assert_eq!(result.variance_percent, dec!(-20.00));
        assert_eq!(result.utilization_percent, dec!(120.00));
        assert_eq!(result.status, VarianceStatus::Unfavorable);
    }

    #[test]
    fn test_revenue_variance_favorable() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1200), "revenue");

        assert_eq!(result.variance, dec!(200));
        assert_eq!(result.variance_percent, dec!(20.00));
        assert_eq!(result.status, VarianceStatus::Favorable);
    }

    #[test]
    fn test_revenue_variance_unfavorable() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(800), "revenue");

        assert_eq!(result.variance, dec!(-200));
        assert_eq!(result.variance_percent, dec!(-20.00));
        assert_eq!(result.status, VarianceStatus::Unfavorable);
    }

    #[test]
    fn test_variance_on_budget() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1000), "expense");

        assert_eq!(result.variance, dec!(0));
        assert_eq!(result.status, VarianceStatus::OnBudget);
    }

    #[test]
    fn test_zero_budget_utilization() {
        let result = BudgetService::calculate_variance(dec!(0), dec!(500), "expense");

        assert_eq!(result.utilization_percent, dec!(0));
        assert_eq!(result.variance_percent, dec!(0));
    }
}
