//! Property-based tests for budget module.

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::service::BudgetService;
use super::types::VarianceStatus;

proptest! {
    /// Feature: reports-simulation, Property 4: Budget Variance Calculation by Account Type
    /// For expense accounts: variance = budgeted - actual, favorable if variance > 0
    #[test]
    fn test_variance_calculation_expense(
        budgeted in 0i64..1_000_000_000,
        actual in 0i64..1_000_000_000,
    ) {
        let budgeted = Decimal::from(budgeted);
        let actual = Decimal::from(actual);

        let result = BudgetService::calculate_variance(budgeted, actual, "expense");

        // For expenses: variance = budgeted - actual
        prop_assert_eq!(result.variance, budgeted - actual);

        // Favorable if under budget (variance > 0)
        if result.variance > Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Favorable);
        } else if result.variance < Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Unfavorable);
        } else {
            prop_assert_eq!(result.status, VarianceStatus::OnBudget);
        }
    }

    /// Feature: reports-simulation, Property 4: Budget Variance Calculation by Account Type
    /// For revenue accounts: variance = actual - budgeted, favorable if variance > 0
    #[test]
    fn test_variance_calculation_revenue(
        budgeted in 0i64..1_000_000_000,
        actual in 0i64..1_000_000_000,
    ) {
        let budgeted = Decimal::from(budgeted);
        let actual = Decimal::from(actual);

        let result = BudgetService::calculate_variance(budgeted, actual, "revenue");

        // For revenue: variance = actual - budgeted
        prop_assert_eq!(result.variance, actual - budgeted);

        // Favorable if over target (variance > 0)
        if result.variance > Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Favorable);
        } else if result.variance < Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Unfavorable);
        } else {
            prop_assert_eq!(result.status, VarianceStatus::OnBudget);
        }
    }

    /// Feature: reports-simulation, Property 6: Utilization Percent Calculation
    /// If B > 0: utilization_percent = (A / B) * 100
    #[test]
    fn test_utilization_percent_nonzero_budget(
        budgeted in 1i64..1_000_000_000,  // Avoid zero to test normal case
        actual in 0i64..1_000_000_000,
    ) {
        let budgeted = Decimal::from(budgeted);
        let actual = Decimal::from(actual);

        let result = BudgetService::calculate_variance(budgeted, actual, "expense");

        let expected = (actual / budgeted * dec!(100)).round_dp(2);
        prop_assert_eq!(result.utilization_percent, expected);
    }

    /// Feature: reports-simulation, Property 6: Utilization Percent Calculation
    /// If B = 0: utilization_percent = 0
    #[test]
    fn test_utilization_percent_zero_budget(
        actual in 0i64..1_000_000_000,
    ) {
        let budgeted = Decimal::ZERO;
        let actual = Decimal::from(actual);

        let result = BudgetService::calculate_variance(budgeted, actual, "expense");

        // Zero budget should result in zero utilization
        prop_assert_eq!(result.utilization_percent, Decimal::ZERO);
    }

    /// Feature: reports-simulation, Property 6: Utilization Percent Calculation
    /// Variance percent should also be zero when budget is zero
    #[test]
    fn test_variance_percent_zero_budget(
        actual in 0i64..1_000_000_000,
    ) {
        let budgeted = Decimal::ZERO;
        let actual = Decimal::from(actual);

        let result = BudgetService::calculate_variance(budgeted, actual, "expense");

        // Zero budget should result in zero variance percent
        prop_assert_eq!(result.variance_percent, Decimal::ZERO);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::budget::error::BudgetError;
    use crate::budget::types::{Budget, BudgetType};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_budget(is_locked: bool) -> Budget {
        Budget {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            fiscal_year_id: Uuid::new_v4(),
            name: "Test Budget".to_string(),
            description: None,
            budget_type: BudgetType::Annual,
            currency: "USD".to_string(),
            is_active: true,
            is_locked,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Feature: reports-simulation, Property 7: Budget Lock Prevents Modification
    #[test]
    fn test_locked_budget_rejects_modification() {
        let budget = create_test_budget(true);
        let result = BudgetService::validate_budget_line(&budget, dec!(1000));

        assert!(matches!(result, Err(BudgetError::BudgetLocked)));
    }

    /// Feature: reports-simulation, Property 7: Budget Lock Prevents Modification
    #[test]
    fn test_unlocked_budget_allows_modification() {
        let budget = create_test_budget(false);
        let result = BudgetService::validate_budget_line(&budget, dec!(1000));

        assert!(result.is_ok());
    }

    #[test]
    fn test_negative_amount_rejected() {
        let budget = create_test_budget(false);
        let result = BudgetService::validate_budget_line(&budget, dec!(-100));

        assert!(matches!(result, Err(BudgetError::NegativeAmount)));
    }

    #[test]
    fn test_zero_amount_allowed() {
        let budget = create_test_budget(false);
        let result = BudgetService::validate_budget_line(&budget, dec!(0));

        assert!(result.is_ok());
    }
}
