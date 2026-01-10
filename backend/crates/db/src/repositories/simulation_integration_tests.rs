//! Integration tests for simulation engine.
//!
//! Tests the simulation workflow: parameter validation, projection calculation, caching.
//! Validates Requirements 5.1-5.9.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use proptest::prelude::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use uuid::Uuid;

    use zeltra_core::simulation::{HistoricalAccountData, SimulationEngine, SimulationParams};

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Create test simulation parameters.
    fn create_test_params(
        projection_months: u32,
        revenue_rate: Decimal,
        expense_rate: Decimal,
    ) -> SimulationParams {
        SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months,
            revenue_growth_rate: revenue_rate,
            expense_growth_rate: expense_rate,
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        }
    }

    /// Create test historical account data.
    fn create_test_account(
        account_type: &str,
        monthly_amounts: Vec<Decimal>,
    ) -> HistoricalAccountData {
        HistoricalAccountData {
            account_id: Uuid::new_v4(),
            account_code: format!("{}-001", account_type.to_uppercase()),
            account_name: format!("Test {} Account", account_type),
            account_type: account_type.to_string(),
            monthly_amounts,
        }
    }

    // ========================================================================
    // Strategy Generators
    // ========================================================================

    /// Strategy for generating valid projection months (1-60)
    fn projection_months_strategy() -> impl Strategy<Value = u32> {
        1u32..=60u32
    }

    /// Strategy for generating valid growth rates (-100% to 1000%)
    fn growth_rate_strategy() -> impl Strategy<Value = Decimal> {
        (-100i64..=1000i64).prop_map(|n| Decimal::new(n, 2))
    }

    /// Strategy for generating positive amounts
    fn amount_strategy() -> impl Strategy<Value = Decimal> {
        (100i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
    }

    // ========================================================================
    // Parameter Validation Tests
    // **Validates: Requirements 5.1, 5.2**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Valid Projection Months**
        ///
        /// *For any* projection months in range 1-60, validation SHALL succeed.
        ///
        /// **Validates: Requirements 5.1**
        #[test]
        fn prop_valid_projection_months(
            months in projection_months_strategy(),
        ) {
            let params = create_test_params(months, dec!(0.10), dec!(0.05));
            let result = SimulationEngine::validate_params(&params);
            prop_assert!(result.is_ok(), "Valid projection months should pass validation");
        }

        /// **Integration Test: Invalid Projection Months**
        ///
        /// *For any* projection months outside 1-60, validation SHALL fail.
        ///
        /// **Validates: Requirements 5.1**
        #[test]
        fn prop_invalid_projection_months(
            months in prop::num::u32::ANY.prop_filter("outside valid range", |m| *m == 0 || *m > 60),
        ) {
            let params = create_test_params(months, dec!(0.10), dec!(0.05));
            let result = SimulationEngine::validate_params(&params);
            prop_assert!(result.is_err(), "Invalid projection months should fail validation");
        }

        /// **Integration Test: Valid Growth Rates**
        ///
        /// *For any* growth rate in range -100% to 1000%, validation SHALL succeed.
        ///
        /// **Validates: Requirements 5.2**
        #[test]
        fn prop_valid_growth_rates(
            revenue_rate in growth_rate_strategy(),
            expense_rate in growth_rate_strategy(),
        ) {
            let params = create_test_params(12, revenue_rate, expense_rate);
            let result = SimulationEngine::validate_params(&params);
            prop_assert!(result.is_ok(), "Valid growth rates should pass validation");
        }
    }

    // ========================================================================
    // Baseline Calculation Tests
    // **Validates: Requirements 5.3, 5.4**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Baseline Calculation**
        ///
        /// *For any* set of monthly amounts, baseline SHALL be the average.
        ///
        /// **Validates: Requirements 5.3**
        #[test]
        fn prop_baseline_is_average(
            amounts in prop::collection::vec(amount_strategy(), 1..12),
        ) {
            let sum: Decimal = amounts.iter().copied().sum();
            let count = Decimal::from(amounts.len() as u64);
            let expected_baseline = (sum / count).round_dp(4);

            let result = SimulationEngine::calculate_baseline(&amounts);

            prop_assert_eq!(result, expected_baseline, "Baseline should be average of monthly amounts");
        }

        /// **Integration Test: Empty Baseline**
        ///
        /// *For any* empty monthly amounts, baseline SHALL be zero.
        ///
        /// **Validates: Requirements 5.4**
        #[test]
        fn prop_empty_baseline_is_zero(_dummy in Just(())) {
            let result = SimulationEngine::calculate_baseline(&[]);
            prop_assert_eq!(result, Decimal::ZERO, "Empty baseline should be zero");
        }
    }

    // ========================================================================
    // Projection Calculation Tests
    // **Validates: Requirements 5.5, 5.6, 5.7**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Integration Test: Projection Count**
        ///
        /// *For any* projection months, result SHALL have correct number of projections.
        ///
        /// **Validates: Requirements 5.5**
        #[test]
        fn prop_projection_count(
            months in projection_months_strategy(),
        ) {
            let params = create_test_params(months, dec!(0), dec!(0));
            let data = vec![create_test_account("revenue", vec![dec!(1000)])];

            let result = SimulationEngine::run(&data, &params);

            prop_assert_eq!(
                result.projections.len() as u32,
                months,
                "Should have one projection per month"
            );
        }

        /// **Integration Test: Zero Growth Projection**
        ///
        /// *For any* baseline with 0% growth, all projections SHALL equal baseline.
        ///
        /// **Validates: Requirements 5.6**
        #[test]
        fn prop_zero_growth_equals_baseline(
            baseline in amount_strategy(),
            months in 1u32..=12u32,
        ) {
            let params = create_test_params(months, dec!(0), dec!(0));
            let data = vec![create_test_account("revenue", vec![baseline])];

            let result = SimulationEngine::run(&data, &params);

            for projection in &result.projections {
                prop_assert_eq!(
                    projection.projected_amount,
                    baseline,
                    "Zero growth should maintain baseline"
                );
            }
        }

        /// **Integration Test: Positive Growth Increases**
        ///
        /// *For any* positive growth rate, projections SHALL increase over time.
        ///
        /// **Validates: Requirements 5.7**
        #[test]
        fn prop_positive_growth_increases(
            baseline in amount_strategy(),
            growth_percent in 1i64..=100i64,
        ) {
            let growth_rate = Decimal::new(growth_percent, 2);
            let params = create_test_params(3, growth_rate, growth_rate);
            let data = vec![create_test_account("revenue", vec![baseline])];

            let result = SimulationEngine::run(&data, &params);

            // Each projection should be greater than the previous
            let mut prev_amount = baseline;
            for projection in &result.projections {
                prop_assert!(
                    projection.projected_amount >= prev_amount,
                    "Positive growth should increase projections"
                );
                prev_amount = projection.projected_amount;
            }
        }
    }

    // ========================================================================
    // Summary Calculation Tests
    // **Validates: Requirements 5.8, 5.9**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Integration Test: Revenue Summary**
        ///
        /// *For any* revenue accounts, summary SHALL sum all revenue projections.
        ///
        /// **Validates: Requirements 5.8**
        #[test]
        fn prop_revenue_summary(
            amounts in prop::collection::vec(amount_strategy(), 1..5),
        ) {
            let params = create_test_params(1, dec!(0), dec!(0));
            let data: Vec<HistoricalAccountData> = amounts
                .iter()
                .map(|a| create_test_account("revenue", vec![*a]))
                .collect();

            let result = SimulationEngine::run(&data, &params);

            let expected_revenue: Decimal = amounts.iter().copied().sum();
            prop_assert_eq!(
                result.annual_summary.total_projected_revenue,
                expected_revenue,
                "Revenue summary should sum all revenue projections"
            );
        }

        /// **Integration Test: Net Income Calculation**
        ///
        /// *For any* revenue and expenses, net income SHALL be revenue - expenses.
        ///
        /// **Validates: Requirements 5.9**
        #[test]
        fn prop_net_income_calculation(
            revenue in amount_strategy(),
            expense in amount_strategy(),
        ) {
            let params = create_test_params(1, dec!(0), dec!(0));
            let data = vec![
                create_test_account("revenue", vec![revenue]),
                create_test_account("expense", vec![expense]),
            ];

            let result = SimulationEngine::run(&data, &params);

            let expected_net = revenue - expense;
            prop_assert_eq!(
                result.annual_summary.projected_net_income,
                expected_net,
                "Net income should be revenue - expenses"
            );
        }
    }

    // ========================================================================
    // Hash/Caching Tests
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Integration Test: Hash Determinism**
        ///
        /// *For any* parameters, hash SHALL be deterministic.
        #[test]
        fn prop_hash_deterministic(
            months in projection_months_strategy(),
            revenue_rate in growth_rate_strategy(),
            expense_rate in growth_rate_strategy(),
        ) {
            let params = create_test_params(months, revenue_rate, expense_rate);
            let hash1 = SimulationEngine::hash_params(&params);
            let hash2 = SimulationEngine::hash_params(&params);

            prop_assert_eq!(hash1, hash2, "Hash should be deterministic");
        }

        /// **Integration Test: Hash Uniqueness**
        ///
        /// *For any* different parameters, hash SHALL be different.
        #[test]
        fn prop_hash_unique(
            months1 in projection_months_strategy(),
            months2 in projection_months_strategy(),
        ) {
            prop_assume!(months1 != months2);

            let params1 = create_test_params(months1, dec!(0.10), dec!(0.05));
            let params2 = create_test_params(months2, dec!(0.10), dec!(0.05));

            let hash1 = SimulationEngine::hash_params(&params1);
            let hash2 = SimulationEngine::hash_params(&params2);

            prop_assert_ne!(hash1, hash2, "Different params should have different hash");
        }
    }

    // ========================================================================
    // Unit Tests: Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_historical_data() {
        let params = create_test_params(12, dec!(0.10), dec!(0.05));
        let result = SimulationEngine::run(&[], &params);

        assert!(result.projections.is_empty());
        assert_eq!(result.annual_summary.total_projected_revenue, dec!(0));
        assert_eq!(result.annual_summary.total_projected_expenses, dec!(0));
        assert_eq!(result.annual_summary.projected_net_income, dec!(0));
    }

    #[test]
    fn test_single_month_projection() {
        let params = create_test_params(1, dec!(0), dec!(0));
        let data = vec![create_test_account("revenue", vec![dec!(1000)])];

        let result = SimulationEngine::run(&data, &params);

        assert_eq!(result.projections.len(), 1);
        assert_eq!(result.projections[0].projected_amount, dec!(1000));
    }

    #[test]
    fn test_compound_growth() {
        // 10% growth for 2 months: 1000 * 1.1 = 1100, 1000 * 1.1^2 = 1210
        let params = create_test_params(2, dec!(0.10), dec!(0));
        let data = vec![create_test_account("revenue", vec![dec!(1000)])];

        let result = SimulationEngine::run(&data, &params);

        assert_eq!(result.projections.len(), 2);
        assert_eq!(result.projections[0].projected_amount, dec!(1100));
        assert_eq!(result.projections[1].projected_amount, dec!(1210));
    }

    #[test]
    fn test_negative_growth() {
        // -10% growth: 1000 * 0.9 = 900
        let params = create_test_params(1, dec!(-0.10), dec!(0));
        let data = vec![create_test_account("revenue", vec![dec!(1000)])];

        let result = SimulationEngine::run(&data, &params);

        assert_eq!(result.projections[0].projected_amount, dec!(900));
    }

    #[test]
    fn test_account_specific_adjustment() {
        let account_id = Uuid::new_v4();
        let mut params = create_test_params(1, dec!(0.10), dec!(0.05));
        params.account_adjustments.insert(account_id, dec!(0.20));

        let data = vec![HistoricalAccountData {
            account_id,
            account_code: "4000".to_string(),
            account_name: "Revenue".to_string(),
            account_type: "revenue".to_string(),
            monthly_amounts: vec![dec!(1000)],
        }];

        let result = SimulationEngine::run(&data, &params);

        // Should use 20% override instead of 10% global rate
        assert_eq!(result.projections[0].projected_amount, dec!(1200));
    }

    #[test]
    fn test_invalid_base_period() {
        let mut params = create_test_params(12, dec!(0.10), dec!(0.05));
        params.base_period_start = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        params.base_period_end = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

        let result = SimulationEngine::validate_params(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_projection_months() {
        let params = create_test_params(60, dec!(0), dec!(0));
        let result = SimulationEngine::validate_params(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_over_max_projection_months() {
        let params = create_test_params(61, dec!(0), dec!(0));
        let result = SimulationEngine::validate_params(&params);
        assert!(result.is_err());
    }
}
