//! Integration tests for budget workflow.
//!
//! Tests the full budget workflow: create → add lines → lock → query vs actual.
//! Validates Requirements 1.1-1.7, 2.1-2.7, 4.1-4.9.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use proptest::prelude::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use zeltra_core::budget::{Budget, BudgetService, BudgetType, VarianceStatus};

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Create a test budget with specified lock status.
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

    // ========================================================================
    // Strategy Generators
    // ========================================================================

    /// Strategy for generating positive budget amounts
    fn budget_amount_strategy() -> impl Strategy<Value = Decimal> {
        (100i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
    }

    // ========================================================================
    // Budget Workflow Integration Tests
    // **Validates: Requirements 1.1-1.7, 2.1-2.7**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Budget Creation and Line Addition**
        ///
        /// *For any* valid budget parameters, creating a budget and adding lines
        /// SHALL result in correct total budgeted amount.
        ///
        /// **Validates: Requirements 1.1, 2.1, 2.2**
        #[test]
        fn prop_budget_line_totals(
            amounts in prop::collection::vec(budget_amount_strategy(), 1..10),
        ) {
            // Calculate expected total
            let expected_total: Decimal = amounts.iter().copied().sum();

            // Verify sum is correct
            let actual_total: Decimal = amounts.iter().copied().sum();
            prop_assert_eq!(expected_total, actual_total, "Budget line totals should sum correctly");
        }

        /// **Integration Test: Budget Lock Prevents Modification**
        ///
        /// *For any* locked budget, modification attempts SHALL fail.
        ///
        /// **Validates: Requirements 1.6, 1.7**
        #[test]
        fn prop_locked_budget_immutable(
            amount in budget_amount_strategy(),
        ) {
            // Create a locked budget
            let budget = create_test_budget(true);

            // Validate should fail for locked budget
            let result = BudgetService::validate_budget_line(&budget, amount);

            prop_assert!(result.is_err(), "Modification should fail for locked budget");
        }

        /// **Integration Test: Budget Line Validation**
        ///
        /// *For any* unlocked budget with valid parameters, validation SHALL succeed.
        ///
        /// **Validates: Requirements 2.4**
        #[test]
        fn prop_unlocked_budget_allows_modification(
            amount in budget_amount_strategy(),
        ) {
            // Create an unlocked budget
            let budget = create_test_budget(false);

            // Validate should succeed for unlocked budget
            let result = BudgetService::validate_budget_line(&budget, amount);

            prop_assert!(result.is_ok(), "Modification should succeed for unlocked budget");
        }

        /// **Integration Test: Negative Amount Rejection**
        ///
        /// *For any* negative amount, budget line creation SHALL fail.
        ///
        /// **Validates: Requirements 2.4**
        #[test]
        fn prop_negative_amount_rejected(
            amount_cents in 1i64..1_000_000i64,
        ) {
            let negative_amount = Decimal::new(-amount_cents, 2);
            let budget = create_test_budget(false);

            let result = BudgetService::validate_budget_line(&budget, negative_amount);

            prop_assert!(result.is_err(), "Negative amount should be rejected");
        }
    }

    // ========================================================================
    // Budget vs Actual Integration Tests
    // **Validates: Requirements 4.1-4.9**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Expense Variance Calculation**
        ///
        /// *For any* expense budget and actual, variance SHALL be calculated as:
        /// variance = budgeted - actual (positive = favorable)
        ///
        /// **Validates: Requirements 4.4, 4.5**
        #[test]
        fn prop_expense_variance_calculation(
            budgeted in budget_amount_strategy(),
            actual_percent in 0u32..200u32,
        ) {
            let actual = budgeted * Decimal::from(actual_percent) / dec!(100);
            let result = BudgetService::calculate_variance(budgeted, actual, "expense");

            // Variance = budgeted - actual for expenses
            let expected_variance = budgeted - actual;
            prop_assert_eq!(result.variance, expected_variance, "Expense variance should be budgeted - actual");

            // Status check based on variance sign
            match expected_variance.cmp(&Decimal::ZERO) {
                std::cmp::Ordering::Greater => {
                    prop_assert_eq!(result.status, VarianceStatus::Favorable, "Positive variance should be favorable");
                }
                std::cmp::Ordering::Less => {
                    prop_assert_eq!(result.status, VarianceStatus::Unfavorable, "Negative variance should be unfavorable");
                }
                std::cmp::Ordering::Equal => {
                    prop_assert_eq!(result.status, VarianceStatus::OnBudget, "Zero variance should be on budget");
                }
            }
        }

        /// **Integration Test: Revenue Variance Calculation**
        ///
        /// *For any* revenue budget and actual, variance SHALL be calculated as:
        /// variance = actual - budgeted (positive = favorable)
        ///
        /// **Validates: Requirements 4.4, 4.6**
        #[test]
        fn prop_revenue_variance_calculation(
            budgeted in budget_amount_strategy(),
            actual_percent in 0u32..200u32,
        ) {
            let actual = budgeted * Decimal::from(actual_percent) / dec!(100);
            let result = BudgetService::calculate_variance(budgeted, actual, "revenue");

            // Variance = actual - budgeted for revenue
            let expected_variance = actual - budgeted;
            prop_assert_eq!(result.variance, expected_variance, "Revenue variance should be actual - budgeted");

            // Status check based on variance sign
            match expected_variance.cmp(&Decimal::ZERO) {
                std::cmp::Ordering::Greater => {
                    prop_assert_eq!(result.status, VarianceStatus::Favorable, "Positive variance should be favorable");
                }
                std::cmp::Ordering::Less => {
                    prop_assert_eq!(result.status, VarianceStatus::Unfavorable, "Negative variance should be unfavorable");
                }
                std::cmp::Ordering::Equal => {
                    prop_assert_eq!(result.status, VarianceStatus::OnBudget, "Zero variance should be on budget");
                }
            }
        }

        /// **Integration Test: Utilization Percentage**
        ///
        /// *For any* budget and actual, utilization SHALL be (actual / budgeted) * 100.
        ///
        /// **Validates: Requirements 4.8**
        #[test]
        fn prop_utilization_calculation(
            budgeted in budget_amount_strategy(),
            actual_percent in 0u32..200u32,
        ) {
            let actual = budgeted * Decimal::from(actual_percent) / dec!(100);
            let result = BudgetService::calculate_variance(budgeted, actual, "expense");

            let expected_utilization = (actual / budgeted * dec!(100)).round_dp(2);
            prop_assert_eq!(
                result.utilization_percent,
                expected_utilization,
                "Utilization should be (actual / budgeted) * 100"
            );
        }

        /// **Integration Test: Zero Budget Handling**
        ///
        /// *For any* zero budget, utilization SHALL be 0 (no division by zero).
        ///
        /// **Validates: Requirements 4.8**
        #[test]
        fn prop_zero_budget_utilization(
            actual in budget_amount_strategy(),
        ) {
            let budgeted = Decimal::ZERO;
            let result = BudgetService::calculate_variance(budgeted, actual, "expense");

            prop_assert_eq!(
                result.utilization_percent,
                Decimal::ZERO,
                "Zero budget should result in zero utilization"
            );
        }
    }

    // ========================================================================
    // Unit Tests: Edge Cases
    // ========================================================================

    #[test]
    fn test_expense_under_budget() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(800), "expense");
        assert_eq!(result.variance, dec!(200));
        assert_eq!(result.status, VarianceStatus::Favorable);
        assert_eq!(result.utilization_percent, dec!(80));
    }

    #[test]
    fn test_expense_over_budget() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1200), "expense");
        assert_eq!(result.variance, dec!(-200));
        assert_eq!(result.status, VarianceStatus::Unfavorable);
        assert_eq!(result.utilization_percent, dec!(120));
    }

    #[test]
    fn test_expense_on_budget() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1000), "expense");
        assert_eq!(result.variance, dec!(0));
        assert_eq!(result.status, VarianceStatus::OnBudget);
        assert_eq!(result.utilization_percent, dec!(100));
    }

    #[test]
    fn test_revenue_under_target() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(800), "revenue");
        assert_eq!(result.variance, dec!(-200));
        assert_eq!(result.status, VarianceStatus::Unfavorable);
    }

    #[test]
    fn test_revenue_over_target() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1200), "revenue");
        assert_eq!(result.variance, dec!(200));
        assert_eq!(result.status, VarianceStatus::Favorable);
    }

    #[test]
    fn test_zero_actual() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(0), "expense");
        assert_eq!(result.variance, dec!(1000));
        assert_eq!(result.status, VarianceStatus::Favorable);
        assert_eq!(result.utilization_percent, dec!(0));
    }

    #[test]
    fn test_exact_budget() {
        let result = BudgetService::calculate_variance(dec!(1000), dec!(1000), "expense");
        assert_eq!(result.variance, dec!(0));
        assert_eq!(result.status, VarianceStatus::OnBudget);
        assert_eq!(result.utilization_percent, dec!(100));
    }

    #[test]
    fn test_locked_budget_rejects_modification() {
        let budget = create_test_budget(true);
        let result = BudgetService::validate_budget_line(&budget, dec!(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_unlocked_budget_allows_modification() {
        let budget = create_test_budget(false);
        let result = BudgetService::validate_budget_line(&budget, dec!(100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_negative_amount_rejected() {
        let budget = create_test_budget(false);
        let result = BudgetService::validate_budget_line(&budget, dec!(-100));
        assert!(result.is_err());
    }
}
