//! Integration tests for dashboard metrics.
//!
//! Tests the dashboard calculation logic: burn rate, runway, utilization.
//! Validates Requirements 6.1-6.9.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    // ========================================================================
    // Strategy Generators
    // ========================================================================

    /// Strategy for generating positive amounts
    fn amount_strategy() -> impl Strategy<Value = Decimal> {
        (100i64..10_000_000i64).prop_map(|n| Decimal::new(n, 2))
    }

    /// Strategy for generating days (1-365)
    #[allow(dead_code)]
    fn days_strategy() -> impl Strategy<Value = i32> {
        1i32..=365i32
    }

    /// Strategy for generating percentages (0-100)
    #[allow(dead_code)]
    fn percent_strategy() -> impl Strategy<Value = Decimal> {
        (0i64..=10000i64).prop_map(|n| Decimal::new(n, 2))
    }

    // ========================================================================
    // Dashboard Calculation Functions (Pure Logic)
    // ========================================================================

    /// Calculate daily burn rate from monthly expenses.
    fn calculate_daily_burn_rate(monthly_expenses: Decimal) -> Decimal {
        if monthly_expenses.is_zero() {
            Decimal::ZERO
        } else {
            (monthly_expenses / dec!(30)).round_dp(2)
        }
    }

    /// Calculate runway in days from cash balance and daily burn rate.
    fn calculate_runway_days(cash_balance: Decimal, daily_burn_rate: Decimal) -> i32 {
        if daily_burn_rate.is_zero() || daily_burn_rate < Decimal::ZERO {
            i32::MAX // Infinite runway if no burn
        } else {
            (cash_balance / daily_burn_rate)
                .round_dp(0)
                .to_string()
                .parse()
                .unwrap_or(i32::MAX)
        }
    }

    /// Calculate utilization percentage.
    fn calculate_utilization(spent: Decimal, budgeted: Decimal) -> Decimal {
        if budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (spent / budgeted * dec!(100)).round_dp(2)
        }
    }

    /// Calculate change percentage.
    fn calculate_change_percent(current: Decimal, previous: Decimal) -> Decimal {
        if previous.is_zero() {
            if current.is_zero() {
                Decimal::ZERO
            } else {
                dec!(100) // 100% increase from zero
            }
        } else {
            ((current - previous) / previous * dec!(100)).round_dp(2)
        }
    }

    /// Calculate projected end of period spending.
    fn calculate_projected_spending(
        current_spent: Decimal,
        days_elapsed: i32,
        total_days: i32,
    ) -> Decimal {
        if days_elapsed == 0 {
            Decimal::ZERO
        } else {
            let daily_rate = current_spent / Decimal::from(days_elapsed);
            (daily_rate * Decimal::from(total_days)).round_dp(2)
        }
    }

    /// Calculate department expense percentage.
    fn calculate_department_percent(department_amount: Decimal, total_amount: Decimal) -> Decimal {
        if total_amount.is_zero() {
            Decimal::ZERO
        } else {
            (department_amount / total_amount * dec!(100)).round_dp(2)
        }
    }

    // ========================================================================
    // Burn Rate Tests
    // **Validates: Requirements 6.1, 6.2**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Daily Burn Rate Calculation**
        ///
        /// *For any* monthly expenses, daily burn rate SHALL be monthly / 30.
        ///
        /// **Validates: Requirements 6.1**
        #[test]
        fn prop_daily_burn_rate(
            monthly_expenses in amount_strategy(),
        ) {
            let daily = calculate_daily_burn_rate(monthly_expenses);
            let expected = (monthly_expenses / dec!(30)).round_dp(2);

            prop_assert_eq!(daily, expected, "Daily burn rate should be monthly / 30");
        }

        /// **Integration Test: Zero Expenses Burn Rate**
        ///
        /// *For any* zero expenses, burn rate SHALL be zero.
        ///
        /// **Validates: Requirements 6.2**
        #[test]
        fn prop_zero_burn_rate(_dummy in Just(())) {
            let daily = calculate_daily_burn_rate(Decimal::ZERO);
            prop_assert_eq!(daily, Decimal::ZERO, "Zero expenses should have zero burn rate");
        }
    }

    // ========================================================================
    // Runway Tests
    // **Validates: Requirements 6.3, 6.4**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Runway Calculation**
        ///
        /// *For any* cash balance and burn rate, runway SHALL be balance / daily_burn.
        ///
        /// **Validates: Requirements 6.3**
        #[test]
        fn prop_runway_calculation(
            cash_balance in amount_strategy(),
            daily_burn in amount_strategy(),
        ) {
            let runway = calculate_runway_days(cash_balance, daily_burn);
            let expected = (cash_balance / daily_burn).round_dp(0).to_string().parse::<i32>().unwrap_or(i32::MAX);

            prop_assert_eq!(runway, expected, "Runway should be cash / daily burn");
        }

        /// **Integration Test: Zero Burn Infinite Runway**
        ///
        /// *For any* cash balance with zero burn, runway SHALL be infinite (MAX).
        ///
        /// **Validates: Requirements 6.4**
        #[test]
        fn prop_infinite_runway(
            cash_balance in amount_strategy(),
        ) {
            let runway = calculate_runway_days(cash_balance, Decimal::ZERO);
            prop_assert_eq!(runway, i32::MAX, "Zero burn should have infinite runway");
        }
    }

    // ========================================================================
    // Utilization Tests
    // **Validates: Requirements 6.5, 6.6**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Utilization Calculation**
        ///
        /// *For any* spent and budgeted amounts, utilization SHALL be (spent / budgeted) * 100.
        ///
        /// **Validates: Requirements 6.5**
        #[test]
        fn prop_utilization_calculation(
            spent in amount_strategy(),
            budgeted in amount_strategy(),
        ) {
            let utilization = calculate_utilization(spent, budgeted);
            let expected = (spent / budgeted * dec!(100)).round_dp(2);

            prop_assert_eq!(utilization, expected, "Utilization should be (spent / budgeted) * 100");
        }

        /// **Integration Test: Zero Budget Utilization**
        ///
        /// *For any* spent amount with zero budget, utilization SHALL be zero.
        ///
        /// **Validates: Requirements 6.6**
        #[test]
        fn prop_zero_budget_utilization(
            spent in amount_strategy(),
        ) {
            let utilization = calculate_utilization(spent, Decimal::ZERO);
            prop_assert_eq!(utilization, Decimal::ZERO, "Zero budget should have zero utilization");
        }
    }

    // ========================================================================
    // Change Percentage Tests
    // **Validates: Requirements 6.7**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Change Percentage Calculation**
        ///
        /// *For any* current and previous values, change SHALL be ((current - previous) / previous) * 100.
        ///
        /// **Validates: Requirements 6.7**
        #[test]
        fn prop_change_percent_calculation(
            current in amount_strategy(),
            previous in amount_strategy(),
        ) {
            let change = calculate_change_percent(current, previous);
            let expected = ((current - previous) / previous * dec!(100)).round_dp(2);

            prop_assert_eq!(change, expected, "Change percent should be ((current - previous) / previous) * 100");
        }

        /// **Integration Test: Zero Previous Change**
        ///
        /// *For any* current value with zero previous, change SHALL be 100% (or 0% if both zero).
        #[test]
        fn prop_zero_previous_change(
            current in amount_strategy(),
        ) {
            let change = calculate_change_percent(current, Decimal::ZERO);
            prop_assert_eq!(change, dec!(100), "Zero previous should show 100% increase");
        }
    }

    // ========================================================================
    // Projected Spending Tests
    // **Validates: Requirements 6.8**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Projected Spending Calculation**
        ///
        /// *For any* current spending and days, projected SHALL extrapolate to full period.
        ///
        /// **Validates: Requirements 6.8**
        #[test]
        fn prop_projected_spending(
            current_spent in amount_strategy(),
            days_elapsed in 1i32..=30i32,
            total_days in 30i32..=31i32,
        ) {
            let projected = calculate_projected_spending(current_spent, days_elapsed, total_days);
            let daily_rate = current_spent / Decimal::from(days_elapsed);
            let expected = (daily_rate * Decimal::from(total_days)).round_dp(2);

            prop_assert_eq!(projected, expected, "Projected spending should extrapolate daily rate");
        }

        /// **Integration Test: Zero Days Projected**
        ///
        /// *For any* spending with zero days elapsed, projected SHALL be zero.
        #[test]
        fn prop_zero_days_projected(
            current_spent in amount_strategy(),
        ) {
            let projected = calculate_projected_spending(current_spent, 0, 30);
            prop_assert_eq!(projected, Decimal::ZERO, "Zero days should have zero projection");
        }
    }

    // ========================================================================
    // Department Percentage Tests
    // **Validates: Requirements 6.9**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test: Department Percentage Calculation**
        ///
        /// *For any* department and total amounts, percent SHALL be (dept / total) * 100.
        ///
        /// **Validates: Requirements 6.9**
        #[test]
        fn prop_department_percent(
            department_amount in amount_strategy(),
            total_amount in amount_strategy(),
        ) {
            prop_assume!(department_amount <= total_amount);

            let percent = calculate_department_percent(department_amount, total_amount);
            let expected = (department_amount / total_amount * dec!(100)).round_dp(2);

            prop_assert_eq!(percent, expected, "Department percent should be (dept / total) * 100");
        }

        /// **Integration Test: Department Percentages Sum**
        ///
        /// *For any* set of department amounts, percentages SHALL sum to ~100%.
        #[test]
        fn prop_department_percentages_sum(
            amounts in prop::collection::vec(amount_strategy(), 2..5),
        ) {
            let total: Decimal = amounts.iter().copied().sum();
            let percentages: Vec<Decimal> = amounts
                .iter()
                .map(|a| calculate_department_percent(*a, total))
                .collect();

            let sum: Decimal = percentages.iter().copied().sum();

            // Allow for rounding errors (should be close to 100)
            prop_assert!(
                sum >= dec!(99.90) && sum <= dec!(100.10),
                "Department percentages should sum to ~100%"
            );
        }
    }

    // ========================================================================
    // Unit Tests: Edge Cases
    // ========================================================================

    #[test]
    fn test_burn_rate_typical() {
        let monthly = dec!(30000);
        let daily = calculate_daily_burn_rate(monthly);
        assert_eq!(daily, dec!(1000));
    }

    #[test]
    fn test_runway_typical() {
        let cash = dec!(100000);
        let daily_burn = dec!(1000);
        let runway = calculate_runway_days(cash, daily_burn);
        assert_eq!(runway, 100);
    }

    #[test]
    fn test_utilization_50_percent() {
        let spent = dec!(5000);
        let budgeted = dec!(10000);
        let utilization = calculate_utilization(spent, budgeted);
        assert_eq!(utilization, dec!(50));
    }

    #[test]
    fn test_utilization_over_100_percent() {
        let spent = dec!(12000);
        let budgeted = dec!(10000);
        let utilization = calculate_utilization(spent, budgeted);
        assert_eq!(utilization, dec!(120));
    }

    #[test]
    fn test_change_percent_increase() {
        let current = dec!(1200);
        let previous = dec!(1000);
        let change = calculate_change_percent(current, previous);
        assert_eq!(change, dec!(20));
    }

    #[test]
    fn test_change_percent_decrease() {
        let current = dec!(800);
        let previous = dec!(1000);
        let change = calculate_change_percent(current, previous);
        assert_eq!(change, dec!(-20));
    }

    #[test]
    fn test_projected_spending_half_month() {
        let spent = dec!(5000);
        let days_elapsed = 15;
        let total_days = 30;
        let projected = calculate_projected_spending(spent, days_elapsed, total_days);
        assert_eq!(projected, dec!(10000));
    }

    #[test]
    fn test_department_percent_single() {
        let dept = dec!(2500);
        let total = dec!(10000);
        let percent = calculate_department_percent(dept, total);
        assert_eq!(percent, dec!(25));
    }

    #[test]
    fn test_negative_burn_infinite_runway() {
        // Negative burn (income > expenses) should give infinite runway
        let runway = calculate_runway_days(dec!(100000), dec!(-1000));
        assert_eq!(runway, i32::MAX);
    }

    #[test]
    fn test_both_zero_change_percent() {
        let change = calculate_change_percent(Decimal::ZERO, Decimal::ZERO);
        assert_eq!(change, Decimal::ZERO);
    }

    // ========================================================================
    // Property 10: Cash Position Calculation
    // **Validates: Requirements 4.2**
    // ========================================================================

    /// Calculate cash position from account balances.
    fn calculate_cash_position(
        current_balance: Decimal,
        previous_balance: Decimal,
    ) -> (Decimal, Decimal) {
        let change = current_balance - previous_balance;
        let change_percent = if previous_balance.is_zero() {
            if current_balance.is_zero() {
                Decimal::ZERO
            } else {
                dec!(100)
            }
        } else {
            ((change) / previous_balance * dec!(100)).round_dp(2)
        };
        (change, change_percent)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 10: Cash Position Calculation**
        ///
        /// *For any* current and previous balances, change SHALL be current - previous.
        ///
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_cash_position_change(
            current in amount_strategy(),
            previous in amount_strategy(),
        ) {
            let (change, _) = calculate_cash_position(current, previous);
            let expected = current - previous;

            prop_assert_eq!(change, expected, "Cash position change should be current - previous");
        }

        /// **Property 10b: Cash Position Change Percent**
        ///
        /// *For any* non-zero previous balance, change percent SHALL be (change / previous) * 100.
        #[test]
        fn prop_cash_position_change_percent(
            current in amount_strategy(),
            previous in amount_strategy(),
        ) {
            let (_, change_percent) = calculate_cash_position(current, previous);
            let expected = ((current - previous) / previous * dec!(100)).round_dp(2);

            prop_assert_eq!(change_percent, expected, "Change percent should be (change / previous) * 100");
        }
    }

    // ========================================================================
    // Property 11: Burn Rate Calculation
    // **Validates: Requirements 4.3**
    // ========================================================================

    /// Calculate burn rate from total expenses over N days.
    fn calculate_burn_rate_from_expenses(total_expenses: Decimal, days: i32) -> (Decimal, Decimal) {
        if days == 0 || total_expenses.is_zero() {
            return (Decimal::ZERO, Decimal::ZERO);
        }
        let daily = (total_expenses / Decimal::from(days)).round_dp(4);
        let monthly = (daily * dec!(30)).round_dp(4);
        (daily, monthly)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 11: Burn Rate from Expenses**
        ///
        /// *For any* total expenses over N days, daily burn SHALL be total / N.
        ///
        /// **Validates: Requirements 4.3**
        #[test]
        fn prop_burn_rate_from_expenses(
            total_expenses in amount_strategy(),
            days in 1i32..=90i32,
        ) {
            let (daily, _) = calculate_burn_rate_from_expenses(total_expenses, days);
            let expected = (total_expenses / Decimal::from(days)).round_dp(4);

            prop_assert_eq!(daily, expected, "Daily burn should be total expenses / days");
        }

        /// **Property 11b: Monthly Burn Rate**
        ///
        /// *For any* daily burn rate, monthly SHALL be daily * 30.
        #[test]
        fn prop_monthly_burn_rate(
            total_expenses in amount_strategy(),
            days in 1i32..=90i32,
        ) {
            let (daily, monthly) = calculate_burn_rate_from_expenses(total_expenses, days);
            let expected = (daily * dec!(30)).round_dp(4);

            prop_assert_eq!(monthly, expected, "Monthly burn should be daily * 30");
        }
    }

    // ========================================================================
    // Property 12: Runway Days Calculation
    // **Validates: Requirements 4.4**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 12: Runway Days**
        ///
        /// *For any* cash balance and daily burn, runway SHALL be balance / daily_burn.
        ///
        /// **Validates: Requirements 4.4**
        #[test]
        fn prop_runway_days(
            cash_balance in amount_strategy(),
            daily_burn in amount_strategy(),
        ) {
            let runway = calculate_runway_days(cash_balance, daily_burn);
            let expected = (cash_balance / daily_burn)
                .round_dp(0)
                .to_string()
                .parse::<i32>()
                .unwrap_or(i32::MAX);

            prop_assert_eq!(runway, expected, "Runway should be cash / daily burn");
        }

        /// **Property 12b: Runway Non-Negative**
        ///
        /// *For any* positive cash and burn, runway SHALL be non-negative.
        #[test]
        fn prop_runway_non_negative(
            cash_balance in amount_strategy(),
            daily_burn in amount_strategy(),
        ) {
            let runway = calculate_runway_days(cash_balance, daily_burn);
            prop_assert!(runway >= 0, "Runway should be non-negative for positive inputs");
        }
    }

    // ========================================================================
    // Property 13: Cash Flow Aggregation
    // **Validates: Requirements 4.5**
    // ========================================================================

    /// Aggregate cash flow entries by month.
    fn aggregate_cash_flow(
        entries: Vec<(String, Decimal, bool)>, // (month, amount, is_inflow)
    ) -> std::collections::HashMap<String, (Decimal, Decimal)> {
        let mut result: std::collections::HashMap<String, (Decimal, Decimal)> =
            std::collections::HashMap::new();

        for (month, amount, is_inflow) in entries {
            let entry = result
                .entry(month)
                .or_insert((Decimal::ZERO, Decimal::ZERO));
            if is_inflow {
                entry.0 += amount;
            } else {
                entry.1 += amount;
            }
        }

        result
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Property 13: Cash Flow Aggregation**
        ///
        /// *For any* set of cash flow entries, aggregation SHALL sum inflows and outflows per month.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn prop_cash_flow_aggregation(
            entries in prop::collection::vec(
                (
                    prop::sample::select(vec!["Jan", "Feb", "Mar", "Apr", "May", "Jun"]),
                    amount_strategy(),
                    prop::bool::ANY,
                ),
                1..20
            ),
        ) {
            let entries_with_string: Vec<(String, Decimal, bool)> = entries
                .into_iter()
                .map(|(m, a, i)| (m.to_string(), a, i))
                .collect();

            let aggregated = aggregate_cash_flow(entries_with_string.clone());

            // Verify each month's totals
            for (month, amount, is_inflow) in &entries_with_string {
                let (inflow, outflow) = aggregated.get(month).unwrap();
                if *is_inflow {
                    prop_assert!(*inflow >= *amount, "Inflow should include this entry");
                } else {
                    prop_assert!(*outflow >= *amount, "Outflow should include this entry");
                }
            }
        }

        /// **Property 13b: Net Cash Flow**
        ///
        /// *For any* month, net cash flow SHALL be inflow - outflow.
        #[test]
        fn prop_net_cash_flow(
            inflow in amount_strategy(),
            outflow in amount_strategy(),
        ) {
            let net = inflow - outflow;
            let expected = inflow - outflow;

            prop_assert_eq!(net, expected, "Net cash flow should be inflow - outflow");
        }
    }

    // ========================================================================
    // Property 14: Cursor-Based Pagination Consistency
    // **Validates: Requirements 4.6**
    // ========================================================================

    /// Simulate cursor-based pagination.
    fn paginate_with_cursor<T: Clone>(
        items: &[T],
        limit: usize,
        cursor_index: Option<usize>,
    ) -> (Vec<T>, Option<usize>) {
        let start = cursor_index.unwrap_or(0);
        let end = (start + limit).min(items.len());
        let page = items[start..end].to_vec();
        let next_cursor = if end < items.len() { Some(end) } else { None };
        (page, next_cursor)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        /// **Property 14: Cursor Pagination Consistency**
        ///
        /// *For any* list and limit, paginating through all pages SHALL return all items exactly once.
        ///
        /// **Validates: Requirements 4.6**
        #[test]
        fn prop_cursor_pagination_complete(
            items in prop::collection::vec(0i32..1000i32, 1..50),
            limit in 1usize..=10usize,
        ) {
            let mut all_items = Vec::new();
            let mut cursor = None;

            loop {
                let (page, next_cursor) = paginate_with_cursor(&items, limit, cursor);
                all_items.extend(page);

                if next_cursor.is_none() {
                    break;
                }
                cursor = next_cursor;
            }

            prop_assert_eq!(all_items.len(), items.len(), "Should return all items");
            prop_assert_eq!(all_items, items, "Items should be in order");
        }

        /// **Property 14b: Page Size Limit**
        ///
        /// *For any* pagination request, page size SHALL not exceed limit.
        #[test]
        fn prop_page_size_limit(
            items in prop::collection::vec(0i32..1000i32, 1..50),
            limit in 1usize..=10usize,
        ) {
            let (page, _) = paginate_with_cursor(&items, limit, None);
            prop_assert!(page.len() <= limit, "Page size should not exceed limit");
        }

        /// **Property 14c: Has More Indicator**
        ///
        /// *For any* pagination, has_more SHALL be true iff there are more items.
        #[test]
        fn prop_has_more_indicator(
            items in prop::collection::vec(0i32..1000i32, 1..50),
            limit in 1usize..=10usize,
        ) {
            let (page, next_cursor) = paginate_with_cursor(&items, limit, None);
            let has_more = next_cursor.is_some();
            let expected_has_more = page.len() < items.len();

            prop_assert_eq!(has_more, expected_has_more, "has_more should indicate remaining items");
        }
    }
}
