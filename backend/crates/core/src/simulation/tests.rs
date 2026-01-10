//! Property-based tests for simulation module.

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use uuid::Uuid;

use super::engine::SimulationEngine;
use super::types::{HistoricalAccountData, SimulationParams};

fn create_base_params(projection_months: u32) -> SimulationParams {
    SimulationParams {
        base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        projection_months,
        revenue_growth_rate: dec!(0.10),
        expense_growth_rate: dec!(0.05),
        account_adjustments: HashMap::new(),
        dimension_filters: vec![],
    }
}

proptest! {
    /// Feature: reports-simulation, Property 18: Simulation Baseline Calculation
    /// For any account with monthly amounts [A1, A2, ..., An], baseline = sum / n
    #[test]
    fn test_baseline_calculation(
        amounts in prop::collection::vec(1i64..1_000_000, 1..12),
    ) {
        let decimal_amounts: Vec<Decimal> = amounts.iter().map(|&a| Decimal::from(a)).collect();
        let sum: Decimal = decimal_amounts.iter().copied().sum();
        let count = Decimal::from(decimal_amounts.len() as u64);
        let expected = (sum / count).round_dp(4);

        let result = SimulationEngine::calculate_baseline(&decimal_amounts);

        prop_assert_eq!(result, expected);
    }

    /// Feature: reports-simulation, Property 18: Simulation Baseline Calculation
    /// Empty amounts should return zero baseline
    #[test]
    fn test_baseline_empty_returns_zero(_dummy in 0..1) {
        let result = SimulationEngine::calculate_baseline(&[]);
        prop_assert_eq!(result, Decimal::ZERO);
    }

    /// Feature: reports-simulation, Property 9: Simulation Projection Count
    /// For any simulation with projection_months = N, result contains N projections per account
    #[test]
    fn test_projection_count(
        projection_months in 1u32..=60,
        num_accounts in 1usize..=5,
    ) {
        let historical_data: Vec<HistoricalAccountData> = (0..num_accounts)
            .map(|i| HistoricalAccountData {
                account_id: Uuid::new_v4(),
                account_code: format!("ACC{}", i),
                account_name: format!("Account {}", i),
                account_type: "expense".to_string(),
                monthly_amounts: vec![dec!(1000)],
            })
            .collect();

        let params = create_base_params(projection_months);
        let result = SimulationEngine::run(&historical_data, &params);

        // Each account should have exactly projection_months projections
        prop_assert_eq!(result.projections.len(), (projection_months as usize) * num_accounts);
    }

    /// Feature: reports-simulation, Property 10: Simulation Compound Growth Formula
    /// Projected amount for month M = baseline * (1 + rate)^M
    #[test]
    fn test_compound_growth_formula(
        baseline_cents in 10000i64..10_000_000,  // $100 to $100,000
        growth_rate_bps in -5000i32..10000,      // -50% to +100% in basis points
        month in 1u32..=12,
    ) {
        let baseline = Decimal::from(baseline_cents) / dec!(100);
        let growth_rate = Decimal::from(growth_rate_bps) / dec!(10000);

        let account_id = Uuid::new_v4();
        let historical_data = vec![HistoricalAccountData {
            account_id,
            account_code: "TEST".to_string(),
            account_name: "Test Account".to_string(),
            account_type: "expense".to_string(),
            monthly_amounts: vec![baseline],
        }];

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: month,
            revenue_growth_rate: dec!(0),
            expense_growth_rate: growth_rate,
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(&historical_data, &params);

        // Get the last projection (month M)
        let last_projection = result.projections.last().unwrap();

        // Expected: baseline * (1 + rate)^month
        let mut expected = baseline;
        let factor = Decimal::ONE + growth_rate;
        for _ in 0..month {
            expected *= factor;
        }
        expected = expected.round_dp(4);

        prop_assert_eq!(last_projection.projected_amount, expected);
    }

    /// Feature: reports-simulation, Property 11: Simulation Growth Rate Override
    /// Account-specific rate overrides global rate
    #[test]
    fn test_growth_rate_override(
        global_rate_bps in 0i32..1000,
        override_rate_bps in 1000i32..2000,
    ) {
        let global_rate = Decimal::from(global_rate_bps) / dec!(10000);
        let override_rate = Decimal::from(override_rate_bps) / dec!(10000);

        let account_id = Uuid::new_v4();
        let baseline = dec!(1000);

        let historical_data = vec![HistoricalAccountData {
            account_id,
            account_code: "TEST".to_string(),
            account_name: "Test Account".to_string(),
            account_type: "expense".to_string(),
            monthly_amounts: vec![baseline],
        }];

        let mut account_adjustments = HashMap::new();
        account_adjustments.insert(account_id, override_rate);

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: 1,
            revenue_growth_rate: global_rate,
            expense_growth_rate: global_rate,
            account_adjustments,
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(&historical_data, &params);
        let projection = &result.projections[0];

        // Should use override rate, not global rate
        let expected_with_override = (baseline * (Decimal::ONE + override_rate)).round_dp(4);
        let expected_with_global = (baseline * (Decimal::ONE + global_rate)).round_dp(4);

        prop_assert_eq!(projection.projected_amount, expected_with_override);
        prop_assert_ne!(projection.projected_amount, expected_with_global);
    }

    /// Feature: reports-simulation, Property 12: Simulation Summary Totals
    /// total_projected_revenue = sum of all revenue projections
    /// total_projected_expenses = sum of all expense projections
    #[test]
    fn test_summary_totals(
        num_revenue in 1usize..=3,
        num_expense in 1usize..=3,
    ) {
        let mut historical_data = Vec::new();

        // Add revenue accounts
        for i in 0..num_revenue {
            historical_data.push(HistoricalAccountData {
                account_id: Uuid::new_v4(),
                account_code: format!("REV{}", i),
                account_name: format!("Revenue {}", i),
                account_type: "revenue".to_string(),
                monthly_amounts: vec![dec!(1000)],
            });
        }

        // Add expense accounts
        for i in 0..num_expense {
            historical_data.push(HistoricalAccountData {
                account_id: Uuid::new_v4(),
                account_code: format!("EXP{}", i),
                account_name: format!("Expense {}", i),
                account_type: "expense".to_string(),
                monthly_amounts: vec![dec!(500)],
            });
        }

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: 1,
            revenue_growth_rate: dec!(0),
            expense_growth_rate: dec!(0),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(&historical_data, &params);

        // Calculate expected totals from projections
        let expected_revenue: Decimal = result
            .projections
            .iter()
            .filter(|p| p.account_type == "revenue")
            .map(|p| p.projected_amount)
            .sum();

        let expected_expenses: Decimal = result
            .projections
            .iter()
            .filter(|p| p.account_type == "expense")
            .map(|p| p.projected_amount)
            .sum();

        prop_assert_eq!(result.annual_summary.total_projected_revenue, expected_revenue);
        prop_assert_eq!(result.annual_summary.total_projected_expenses, expected_expenses);
        prop_assert_eq!(
            result.annual_summary.projected_net_income,
            expected_revenue - expected_expenses
        );
    }

    /// Hash should be deterministic for same parameters
    #[test]
    fn test_hash_deterministic(
        projection_months in 1u32..=60,
    ) {
        let params = create_base_params(projection_months);
        let hash1 = SimulationEngine::hash_params(&params);
        let hash2 = SimulationEngine::hash_params(&params);

        prop_assert_eq!(hash1, hash2);
    }

    /// Hash should differ for different parameters
    #[test]
    fn test_hash_differs_for_different_params(
        months1 in 1u32..30,
        months2 in 31u32..60,
    ) {
        let params1 = create_base_params(months1);
        let params2 = create_base_params(months2);

        let hash1 = SimulationEngine::hash_params(&params1);
        let hash2 = SimulationEngine::hash_params(&params2);

        prop_assert_ne!(hash1, hash2);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::simulation::error::SimulationError;

    #[test]
    fn test_validate_params_boundary_months() {
        // 1 month should be valid
        let mut params = create_base_params(1);
        assert!(SimulationEngine::validate_params(&params).is_ok());

        // 60 months should be valid
        params.projection_months = 60;
        assert!(SimulationEngine::validate_params(&params).is_ok());
    }

    #[test]
    fn test_validate_params_boundary_growth_rate() {
        let mut params = create_base_params(12);

        // -100% should be valid
        params.revenue_growth_rate = dec!(-1);
        assert!(SimulationEngine::validate_params(&params).is_ok());

        // +1000% should be valid
        params.revenue_growth_rate = dec!(10);
        assert!(SimulationEngine::validate_params(&params).is_ok());

        // -101% should be invalid
        params.revenue_growth_rate = dec!(-1.01);
        assert!(matches!(
            SimulationEngine::validate_params(&params),
            Err(SimulationError::InvalidGrowthRate)
        ));
    }

    #[test]
    fn test_revenue_uses_revenue_growth_rate() {
        let historical_data = vec![HistoricalAccountData {
            account_id: Uuid::new_v4(),
            account_code: "REV".to_string(),
            account_name: "Revenue".to_string(),
            account_type: "revenue".to_string(),
            monthly_amounts: vec![dec!(1000)],
        }];

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: 1,
            revenue_growth_rate: dec!(0.10), // 10%
            expense_growth_rate: dec!(0.05), // 5%
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(&historical_data, &params);

        // Revenue should use 10% growth rate
        assert_eq!(result.projections[0].projected_amount, dec!(1100));
    }

    #[test]
    fn test_expense_uses_expense_growth_rate() {
        let historical_data = vec![HistoricalAccountData {
            account_id: Uuid::new_v4(),
            account_code: "EXP".to_string(),
            account_name: "Expense".to_string(),
            account_type: "expense".to_string(),
            monthly_amounts: vec![dec!(1000)],
        }];

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: 1,
            revenue_growth_rate: dec!(0.10), // 10%
            expense_growth_rate: dec!(0.05), // 5%
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(&historical_data, &params);

        // Expense should use 5% growth rate
        assert_eq!(result.projections[0].projected_amount, dec!(1050));
    }
}
