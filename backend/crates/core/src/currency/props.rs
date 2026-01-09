//! Property-based tests for currency operations.
//!
//! Feature: ledger-core
//! - Property 6: Banker's Rounding Correctness
//! - Property 7: Allocation Sum Invariant

use proptest::prelude::*;
use rust_decimal::Decimal;

use super::allocation::AllocationUtil;
use super::service::CurrencyService;

/// Strategy to generate positive decimal amounts (0.01 to 1,000,000.00).
fn positive_amount() -> impl Strategy<Value = Decimal> {
    (1i64..100_000_000i64).prop_map(|cents| Decimal::new(cents, 2))
}

/// Strategy to generate positive exchange rates (0.0001 to 10000.0000).
fn positive_rate() -> impl Strategy<Value = Decimal> {
    (1i64..100_000_000i64).prop_map(|v| Decimal::new(v, 4))
}

/// Strategy to generate allocation count (1 to 100).
fn allocation_count() -> impl Strategy<Value = usize> {
    1usize..100
}

/// Strategy to generate decimal places (0 to 4).
fn decimal_places() -> impl Strategy<Value = u32> {
    0u32..=4
}

/// Strategy to generate percentages that sum to 100.
fn percentages_summing_to_100() -> impl Strategy<Value = Vec<Decimal>> {
    // Generate 2-10 random values, then normalize to sum to 100
    prop::collection::vec(1u32..100, 2..10).prop_map(|values| {
        let sum: u32 = values.iter().sum();
        let hundred = Decimal::from(100);
        values
            .iter()
            .map(|v| hundred * Decimal::from(*v) / Decimal::from(sum))
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 6: Banker's Rounding Correctness
    // Validates: Requirements 12.1, 12.2
    // =========================================================================

    /// Property 6.1: Conversion result is always rounded to 4 decimal places.
    ///
    /// *For any* amount and rate, the result of convert() SHALL have at most
    /// 4 decimal places.
    #[test]
    fn prop_convert_rounds_to_4_decimals(
        amount in positive_amount(),
        rate in positive_rate(),
    ) {
        let result = CurrencyService::convert(amount, rate);
        // Check that result has at most 4 decimal places
        let scaled = result * Decimal::from(10000);
        let rounded = scaled.round();
        prop_assert_eq!(
            scaled, rounded,
            "Result {} should have at most 4 decimal places",
            result
        );
    }

    /// Property 6.2: Rounding is deterministic.
    ///
    /// *For any* amount and rate, calling convert() twice with the same inputs
    /// SHALL produce the same result.
    #[test]
    fn prop_convert_is_deterministic(
        amount in positive_amount(),
        rate in positive_rate(),
    ) {
        let result1 = CurrencyService::convert(amount, rate);
        let result2 = CurrencyService::convert(amount, rate);
        prop_assert_eq!(result1, result2, "Conversion should be deterministic");
    }

    /// Property 6.3: Same currency conversion preserves amount.
    ///
    /// *For any* amount, converting with rate=1 SHALL return the original amount
    /// (rounded to 4 decimals).
    #[test]
    fn prop_same_currency_preserves_amount(
        amount in positive_amount(),
    ) {
        let result = CurrencyService::convert(amount, Decimal::ONE);
        // Amount should be preserved (just rounded to 4 decimals)
        let expected = CurrencyService::round(amount, 4);
        prop_assert_eq!(result, expected, "Same currency should preserve amount");
    }

    /// Property 6.4: Conversion result is positive for positive inputs.
    ///
    /// *For any* positive amount and positive rate, the result SHALL be positive.
    #[test]
    fn prop_positive_inputs_positive_output(
        amount in positive_amount(),
        rate in positive_rate(),
    ) {
        let result = CurrencyService::convert(amount, rate);
        prop_assert!(result > Decimal::ZERO, "Result should be positive");
    }

    // =========================================================================
    // Property 7: Allocation Sum Invariant
    // Validates: Requirements 12.3, 12.4, 12.5
    // =========================================================================

    /// Property 7.1: Equal allocation sum equals original (rounded).
    ///
    /// *For any* amount and count, the sum of allocate_equal() results
    /// SHALL exactly equal the original amount rounded to the target precision.
    #[test]
    fn prop_allocate_equal_sum_invariant(
        total in positive_amount(),
        count in allocation_count(),
        decimal_places in decimal_places(),
    ) {
        let result = AllocationUtil::allocate_equal(total, count, decimal_places);

        // Sum must exactly equal total rounded to target precision
        let sum: Decimal = result.iter().copied().sum();
        let expected = CurrencyService::round(total, decimal_places);
        prop_assert_eq!(
            sum, expected,
            "Sum of allocations ({}) must equal total rounded ({})",
            sum, expected
        );
    }

    /// Property 7.2: Equal allocation produces correct count.
    ///
    /// *For any* amount and count, allocate_equal() SHALL return exactly
    /// `count` allocations.
    #[test]
    fn prop_allocate_equal_correct_count(
        total in positive_amount(),
        count in allocation_count(),
        decimal_places in decimal_places(),
    ) {
        let result = AllocationUtil::allocate_equal(total, count, decimal_places);
        prop_assert_eq!(
            result.len(), count,
            "Should return exactly {} allocations",
            count
        );
    }

    /// Property 7.3: Equal allocation produces non-negative values.
    ///
    /// *For any* positive amount and count, all allocations SHALL be non-negative.
    #[test]
    fn prop_allocate_equal_non_negative(
        total in positive_amount(),
        count in allocation_count(),
        decimal_places in decimal_places(),
    ) {
        let result = AllocationUtil::allocate_equal(total, count, decimal_places);
        for (i, alloc) in result.iter().enumerate() {
            prop_assert!(
                *alloc >= Decimal::ZERO,
                "Allocation {} should be non-negative, got {}",
                i, alloc
            );
        }
    }

    /// Property 7.4: Percentage allocation sum equals original (rounded).
    ///
    /// *For any* amount and percentages summing to 100, the sum of
    /// allocate_by_percentages() results SHALL exactly equal the original amount
    /// rounded to the target precision.
    #[test]
    fn prop_allocate_by_percentages_sum_invariant(
        total in positive_amount(),
        percentages in percentages_summing_to_100(),
        decimal_places in decimal_places(),
    ) {
        let result = AllocationUtil::allocate_by_percentages(total, &percentages, decimal_places);

        // Sum must exactly equal total rounded to target precision
        let sum: Decimal = result.iter().copied().sum();
        let expected = CurrencyService::round(total, decimal_places);
        prop_assert_eq!(
            sum, expected,
            "Sum of allocations ({}) must equal total rounded ({})",
            sum, expected
        );
    }

    /// Property 7.5: Percentage allocation produces correct count.
    ///
    /// *For any* percentages, allocate_by_percentages() SHALL return exactly
    /// `percentages.len()` allocations.
    #[test]
    fn prop_allocate_by_percentages_correct_count(
        total in positive_amount(),
        percentages in percentages_summing_to_100(),
        decimal_places in decimal_places(),
    ) {
        let result = AllocationUtil::allocate_by_percentages(total, &percentages, decimal_places);
        prop_assert_eq!(
            result.len(), percentages.len(),
            "Should return exactly {} allocations",
            percentages.len()
        );
    }

    /// Property 7.6: Percentage allocation produces non-negative values.
    ///
    /// *For any* positive amount and positive percentages, all allocations
    /// SHALL be non-negative.
    #[test]
    fn prop_allocate_by_percentages_non_negative(
        total in positive_amount(),
        percentages in percentages_summing_to_100(),
        decimal_places in decimal_places(),
    ) {
        let result = AllocationUtil::allocate_by_percentages(total, &percentages, decimal_places);
        for (i, alloc) in result.iter().enumerate() {
            prop_assert!(
                *alloc >= Decimal::ZERO,
                "Allocation {} should be non-negative, got {}",
                i, alloc
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use rust_decimal_macros::dec;

    // =========================================================================
    // Property 6: Banker's Rounding - Specific Examples
    // =========================================================================

    /// Specific example: 2.5 rounds to 2 (nearest even).
    #[test]
    fn test_bankers_rounding_2_5() {
        let result = CurrencyService::round(dec!(2.5), 0);
        assert_eq!(result, dec!(2));
    }

    /// Specific example: 3.5 rounds to 4 (nearest even).
    #[test]
    fn test_bankers_rounding_3_5() {
        let result = CurrencyService::round(dec!(3.5), 0);
        assert_eq!(result, dec!(4));
    }

    /// Specific example: 2.25 rounds to 2.2 (nearest even at 1 decimal).
    #[test]
    fn test_bankers_rounding_2_25() {
        let result = CurrencyService::round(dec!(2.25), 1);
        assert_eq!(result, dec!(2.2));
    }

    /// Specific example: 2.35 rounds to 2.4 (nearest even at 1 decimal).
    #[test]
    fn test_bankers_rounding_2_35() {
        let result = CurrencyService::round(dec!(2.35), 1);
        assert_eq!(result, dec!(2.4));
    }

    // =========================================================================
    // Property 7: Allocation Sum Invariant - Specific Examples
    // =========================================================================

    /// Specific example: 100/3 = [33.34, 33.33, 33.33], sum = 100.00.
    #[test]
    fn test_allocate_equal_100_by_3() {
        let result = AllocationUtil::allocate_equal(dec!(100), 3, 2);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(100));
        assert_eq!(result[0], dec!(33.34));
        assert_eq!(result[1], dec!(33.33));
        assert_eq!(result[2], dec!(33.33));
    }

    /// Specific example: 1/3 = [0.34, 0.33, 0.33], sum = 1.00.
    #[test]
    fn test_allocate_equal_1_by_3() {
        let result = AllocationUtil::allocate_equal(dec!(1), 3, 2);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(1));
    }

    /// Specific example: 0.01/3 with 2 decimals.
    #[test]
    fn test_allocate_equal_penny_by_3() {
        let result = AllocationUtil::allocate_equal(dec!(0.01), 3, 2);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(0.01));
    }
}
