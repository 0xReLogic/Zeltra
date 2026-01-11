//! Benchmark test for simulation performance.

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::time::Instant;
    use uuid::Uuid;

    use crate::simulation::{HistoricalAccountData, SimulationEngine, SimulationParams};

    /// Generate realistic test data with many accounts and historical months.
    fn generate_test_data(num_accounts: usize, num_months: usize) -> Vec<HistoricalAccountData> {
        let mut data = Vec::with_capacity(num_accounts);

        for i in 0..num_accounts {
            let account_type = if i % 2 == 0 { "revenue" } else { "expense" };
            let base_amount = Decimal::from((i + 1) * 1000);

            // Generate monthly amounts with some variance
            let monthly_amounts: Vec<Decimal> = (0..num_months)
                .map(|m| base_amount + Decimal::from(m as i64 * 100))
                .collect();

            data.push(HistoricalAccountData {
                account_id: Uuid::new_v4(),
                account_code: format!("{:04}", 4000 + i),
                account_name: format!("Account {}", i),
                account_type: account_type.to_string(),
                monthly_amounts,
            });
        }

        data
    }

    #[test]
    fn benchmark_simulation_12_months_100_accounts() {
        let params = SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months: 12,
            revenue_growth_rate: dec!(0.10),
            expense_growth_rate: dec!(0.05),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        // 100 accounts with 12 months of historical data
        let data = generate_test_data(100, 12);

        let start = Instant::now();
        let result = SimulationEngine::run(&data, &params);
        let duration = start.elapsed();

        println!("\n=== BENCHMARK: 100 accounts, 12-month projection ===");
        println!("Duration: {:?}", duration);
        println!("Projections generated: {}", result.projections.len());
        println!("Expected projections: {}", 100 * 12);

        assert_eq!(result.projections.len(), 100 * 12);
        assert!(
            duration.as_millis() < 2000,
            "Simulation took {}ms, expected <2000ms",
            duration.as_millis()
        );
    }

    #[test]
    fn benchmark_simulation_12_months_500_accounts() {
        let params = SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months: 12,
            revenue_growth_rate: dec!(0.10),
            expense_growth_rate: dec!(0.05),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        // 500 accounts - stress test
        let data = generate_test_data(500, 12);

        let start = Instant::now();
        let result = SimulationEngine::run(&data, &params);
        let duration = start.elapsed();

        println!("\n=== BENCHMARK: 500 accounts, 12-month projection ===");
        println!("Duration: {:?}", duration);
        println!("Projections generated: {}", result.projections.len());

        assert_eq!(result.projections.len(), 500 * 12);
        assert!(
            duration.as_millis() < 2000,
            "Simulation took {}ms, expected <2000ms",
            duration.as_millis()
        );
    }

    #[test]
    fn benchmark_simulation_60_months_100_accounts() {
        let params = SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months: 60, // 5 years - max allowed
            revenue_growth_rate: dec!(0.10),
            expense_growth_rate: dec!(0.05),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        let data = generate_test_data(100, 12);

        let start = Instant::now();
        let result = SimulationEngine::run(&data, &params);
        let duration = start.elapsed();

        println!("\n=== BENCHMARK: 100 accounts, 60-month projection (5 years) ===");
        println!("Duration: {:?}", duration);
        println!("Projections generated: {}", result.projections.len());

        assert_eq!(result.projections.len(), 100 * 60);
        assert!(
            duration.as_millis() < 2000,
            "Simulation took {}ms, expected <2000ms",
            duration.as_millis()
        );
    }

    #[test]
    fn benchmark_simulation_worst_case() {
        let params = SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months: 60,
            revenue_growth_rate: dec!(0.10),
            expense_growth_rate: dec!(0.05),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        };

        // Worst case: 1000 accounts, 60 months projection
        let data = generate_test_data(1000, 12);

        let start = Instant::now();
        let result = SimulationEngine::run(&data, &params);
        let duration = start.elapsed();

        println!("\n=== BENCHMARK: WORST CASE - 1000 accounts, 60-month projection ===");
        println!("Duration: {:?}", duration);
        println!("Projections generated: {}", result.projections.len());
        println!(
            "Throughput: {:.0} projections/sec",
            result.projections.len() as f64 / duration.as_secs_f64()
        );

        assert_eq!(result.projections.len(), 1000 * 60);
        // Worst case should still be under 5 seconds
        assert!(
            duration.as_millis() < 5000,
            "Worst case simulation took {}ms, expected <5000ms",
            duration.as_millis()
        );
    }
}
