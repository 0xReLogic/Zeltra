//! Simulation engine for running projections.

use chrono::{Datelike, NaiveDate};
use rayon::prelude::*;
use rust_decimal::Decimal;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

use super::error::SimulationError;
use super::types::{
    AccountProjection, AnnualSummary, HistoricalAccountData, SimulationParams, SimulationResult,
};

/// Engine for running what-if simulations.
pub struct SimulationEngine;

impl SimulationEngine {
    /// Creates a new simulation engine.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates simulation parameters.
    ///
    /// # Errors
    ///
    /// Returns error if parameters are invalid.
    pub fn validate_params(params: &SimulationParams) -> Result<(), SimulationError> {
        if params.base_period_start > params.base_period_end {
            return Err(SimulationError::InvalidBasePeriod {
                start: params.base_period_start,
                end: params.base_period_end,
            });
        }

        if params.projection_months == 0 || params.projection_months > 60 {
            return Err(SimulationError::InvalidProjectionMonths);
        }

        let min_rate = Decimal::new(-1, 0);
        let max_rate = Decimal::new(10, 0);

        if params.revenue_growth_rate < min_rate || params.revenue_growth_rate > max_rate {
            return Err(SimulationError::InvalidGrowthRate);
        }

        if params.expense_growth_rate < min_rate || params.expense_growth_rate > max_rate {
            return Err(SimulationError::InvalidGrowthRate);
        }

        for rate in params.account_adjustments.values() {
            if *rate < min_rate || *rate > max_rate {
                return Err(SimulationError::InvalidGrowthRate);
            }
        }

        Ok(())
    }

    /// Runs a simulation with parallel processing.
    ///
    /// Uses Rayon for parallel computation across accounts.
    #[must_use]
    pub fn run(
        historical_data: &[HistoricalAccountData],
        params: &SimulationParams,
    ) -> SimulationResult {
        // Use Rayon for parallel computation across accounts
        let projections: Vec<AccountProjection> = historical_data
            .par_iter()
            .flat_map(|account| Self::project_account(account, params))
            .collect();

        // Calculate summary totals
        let mut total_revenue = Decimal::ZERO;
        let mut total_expenses = Decimal::ZERO;

        for projection in &projections {
            match projection.account_type.as_str() {
                "revenue" => total_revenue += projection.projected_amount,
                "expense" => total_expenses += projection.projected_amount,
                _ => {}
            }
        }

        SimulationResult {
            simulation_id: Uuid::new_v4(),
            parameters_hash: Self::hash_params(params),
            projections,
            annual_summary: AnnualSummary {
                total_projected_revenue: total_revenue,
                total_projected_expenses: total_expenses,
                projected_net_income: total_revenue - total_expenses,
            },
            cached: false,
        }
    }

    /// Projects a single account into the future.
    fn project_account(
        data: &HistoricalAccountData,
        params: &SimulationParams,
    ) -> Vec<AccountProjection> {
        let baseline = Self::calculate_baseline(&data.monthly_amounts);

        // Get growth rate (account-specific or global)
        let growth_rate = params
            .account_adjustments
            .get(&data.account_id)
            .copied()
            .unwrap_or_else(|| {
                if data.account_type == "revenue" {
                    params.revenue_growth_rate
                } else {
                    params.expense_growth_rate
                }
            });

        let mut projections = Vec::with_capacity(params.projection_months as usize);
        let mut current_date = params.base_period_end;

        for month in 1..=params.projection_months {
            current_date = Self::add_months(current_date, 1);

            // Compound growth: baseline * (1 + rate)^month
            let growth_factor = Self::pow_decimal(Decimal::ONE + growth_rate, month);
            let projected = (baseline * growth_factor).round_dp(4);

            projections.push(AccountProjection {
                period_name: current_date.format("%Y-%m").to_string(),
                period_start: Self::month_start(current_date),
                period_end: Self::month_end(current_date),
                account_id: data.account_id,
                account_code: data.account_code.clone(),
                account_name: data.account_name.clone(),
                account_type: data.account_type.clone(),
                baseline_amount: baseline,
                projected_amount: projected,
                change_percent: if baseline.is_zero() {
                    Decimal::ZERO
                } else {
                    ((projected - baseline) / baseline * Decimal::ONE_HUNDRED).round_dp(2)
                },
            });
        }

        projections
    }

    /// Calculates the baseline (average) from monthly amounts.
    ///
    /// Returns zero if the input is empty.
    #[must_use]
    pub fn calculate_baseline(monthly_amounts: &[Decimal]) -> Decimal {
        if monthly_amounts.is_empty() {
            return Decimal::ZERO;
        }

        let sum: Decimal = monthly_amounts.iter().copied().sum();
        let count = Decimal::from(monthly_amounts.len() as u64);
        (sum / count).round_dp(4)
    }

    /// Computes base^exponent for Decimal using repeated multiplication.
    fn pow_decimal(base: Decimal, exponent: u32) -> Decimal {
        if exponent == 0 {
            return Decimal::ONE;
        }

        let mut result = Decimal::ONE;
        for _ in 0..exponent {
            result *= base;
        }
        result
    }

    /// Hashes simulation parameters for caching.
    #[must_use]
    pub fn hash_params(params: &SimulationParams) -> String {
        let mut hasher = DefaultHasher::new();

        // Hash key parameters
        params.base_period_start.hash(&mut hasher);
        params.base_period_end.hash(&mut hasher);
        params.projection_months.hash(&mut hasher);

        // Hash growth rates as strings to avoid float issues
        params.revenue_growth_rate.to_string().hash(&mut hasher);
        params.expense_growth_rate.to_string().hash(&mut hasher);

        // Hash account adjustments (sorted for consistency)
        let mut adjustments: Vec<_> = params.account_adjustments.iter().collect();
        adjustments.sort_by_key(|(k, _)| *k);
        for (account_id, rate) in adjustments {
            account_id.hash(&mut hasher);
            rate.to_string().hash(&mut hasher);
        }

        // Hash dimension filters (sorted for consistency)
        let mut filters = params.dimension_filters.clone();
        filters.sort();
        for filter in filters {
            filter.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Adds months to a date.
    fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
        date.checked_add_months(chrono::Months::new(months))
            .unwrap_or(date)
    }

    /// Gets the first day of the month.
    fn month_start(date: NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
    }

    /// Gets the last day of the month.
    fn month_end(date: NaiveDate) -> NaiveDate {
        Self::month_start(Self::add_months(date, 1))
            .pred_opt()
            .unwrap_or(date)
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
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    fn create_test_params() -> SimulationParams {
        SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months: 12,
            revenue_growth_rate: dec!(0.10),
            expense_growth_rate: dec!(0.05),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        }
    }

    #[test]
    fn test_calculate_baseline_empty() {
        let result = SimulationEngine::calculate_baseline(&[]);
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_calculate_baseline_single() {
        let result = SimulationEngine::calculate_baseline(&[dec!(1000)]);
        assert_eq!(result, dec!(1000));
    }

    #[test]
    fn test_calculate_baseline_multiple() {
        let amounts = vec![dec!(1000), dec!(2000), dec!(3000)];
        let result = SimulationEngine::calculate_baseline(&amounts);
        assert_eq!(result, dec!(2000));
    }

    #[test]
    fn test_validate_params_valid() {
        let params = create_test_params();
        assert!(SimulationEngine::validate_params(&params).is_ok());
    }

    #[test]
    fn test_validate_params_invalid_period() {
        let mut params = create_test_params();
        params.base_period_start = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        params.base_period_end = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

        assert!(matches!(
            SimulationEngine::validate_params(&params),
            Err(SimulationError::InvalidBasePeriod { .. })
        ));
    }

    #[test]
    fn test_validate_params_zero_months() {
        let mut params = create_test_params();
        params.projection_months = 0;

        assert!(matches!(
            SimulationEngine::validate_params(&params),
            Err(SimulationError::InvalidProjectionMonths)
        ));
    }

    #[test]
    fn test_validate_params_too_many_months() {
        let mut params = create_test_params();
        params.projection_months = 61;

        assert!(matches!(
            SimulationEngine::validate_params(&params),
            Err(SimulationError::InvalidProjectionMonths)
        ));
    }

    #[test]
    fn test_validate_params_invalid_growth_rate() {
        let mut params = create_test_params();
        params.revenue_growth_rate = dec!(11);

        assert!(matches!(
            SimulationEngine::validate_params(&params),
            Err(SimulationError::InvalidGrowthRate)
        ));
    }

    #[test]
    fn test_run_empty_data() {
        let params = create_test_params();
        let result = SimulationEngine::run(&[], &params);

        assert!(result.projections.is_empty());
        assert_eq!(result.annual_summary.total_projected_revenue, Decimal::ZERO);
        assert_eq!(
            result.annual_summary.total_projected_expenses,
            Decimal::ZERO
        );
    }

    #[test]
    fn test_run_single_account() {
        let params = SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: 3,
            revenue_growth_rate: dec!(0),
            expense_growth_rate: dec!(0),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        let data = vec![HistoricalAccountData {
            account_id: Uuid::new_v4(),
            account_code: "4000".to_string(),
            account_name: "Revenue".to_string(),
            account_type: "revenue".to_string(),
            monthly_amounts: vec![dec!(1000)],
        }];

        let result = SimulationEngine::run(&data, &params);

        assert_eq!(result.projections.len(), 3);
        // With 0% growth, all projections should equal baseline
        for projection in &result.projections {
            assert_eq!(projection.projected_amount, dec!(1000));
        }
    }

    #[test]
    fn test_hash_params_deterministic() {
        let params = create_test_params();
        let hash1 = SimulationEngine::hash_params(&params);
        let hash2 = SimulationEngine::hash_params(&params);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_params_different_for_different_params() {
        let params1 = create_test_params();
        let mut params2 = create_test_params();
        params2.projection_months = 24;

        let hash1 = SimulationEngine::hash_params(&params1);
        let hash2 = SimulationEngine::hash_params(&params2);

        assert_ne!(hash1, hash2);
    }
}
