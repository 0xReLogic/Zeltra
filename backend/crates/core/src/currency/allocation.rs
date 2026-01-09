//! Amount allocation utilities using Largest Remainder Method.
//!
//! This module provides functions for allocating amounts fairly while
//! ensuring the sum exactly equals the original total (no cents lost).
//!
//! The Largest Remainder Method works by:
//! 1. Calculate exact allocations
//! 2. Round down each allocation
//! 3. Calculate the remainder (total - sum of rounded)
//! 4. Distribute remainder units to items with largest fractional parts

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Allocation utility for distributing amounts.
///
/// Uses the Largest Remainder Method to ensure:
/// - Fair distribution of amounts
/// - Sum of allocations EXACTLY equals the original total
/// - No cents are lost or gained
pub struct AllocationUtil;

impl AllocationUtil {
    /// Allocate amount equally across N recipients using Largest Remainder Method.
    ///
    /// Ensures sum of allocations EXACTLY equals total.
    ///
    /// # Arguments
    ///
    /// * `total` - The total amount to allocate
    /// * `count` - Number of recipients
    /// * `decimal_places` - Number of decimal places for each allocation
    ///
    /// # Returns
    ///
    /// A vector of allocated amounts where sum == total.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal_macros::dec;
    /// use zeltra_core::currency::AllocationUtil;
    ///
    /// // 100 / 3 = [33.34, 33.33, 33.33], sum = 100.00
    /// let result = AllocationUtil::allocate_equal(dec!(100), 3, 2);
    /// assert_eq!(result.iter().sum::<rust_decimal::Decimal>(), dec!(100));
    /// ```
    #[must_use]
    pub fn allocate_equal(total: Decimal, count: usize, decimal_places: u32) -> Vec<Decimal> {
        if count == 0 {
            return vec![];
        }
        if count == 1 {
            return vec![
                total.round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointNearestEven),
            ];
        }

        let count_dec = Decimal::from(count as u64);
        let unit = Decimal::new(1, decimal_places);

        // Round total to target precision first
        let total_rounded =
            total.round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointNearestEven);

        // Calculate exact allocation per recipient
        let exact_per_recipient = total_rounded / count_dec;

        // Round down to get base allocation
        let base =
            exact_per_recipient.round_dp_with_strategy(decimal_places, RoundingStrategy::ToZero);

        // Calculate how much we've allocated so far
        let allocated = base * count_dec;

        // Calculate remainder to distribute
        let remainder = total_rounded - allocated;

        // How many recipients get an extra unit
        let extra_count = (remainder / unit)
            .round_dp_with_strategy(0, RoundingStrategy::ToZero)
            .to_u64()
            .unwrap_or(0);
        let extra_count = usize::try_from(extra_count).unwrap_or(0);

        // Distribute: first N items get extra unit
        (0..count)
            .map(|i| if i < extra_count { base + unit } else { base })
            .collect()
    }

    /// Allocate by percentages using Largest Remainder Method.
    ///
    /// Ensures sum of allocations EXACTLY equals total.
    ///
    /// # Arguments
    ///
    /// * `total` - The total amount to allocate
    /// * `percentages` - Slice of percentages (should sum to 100)
    /// * `decimal_places` - Number of decimal places for each allocation
    ///
    /// # Returns
    ///
    /// A vector of allocated amounts where sum == total.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal_macros::dec;
    /// use zeltra_core::currency::AllocationUtil;
    ///
    /// // 100 split 50%/30%/20%
    /// let percentages = vec![dec!(50), dec!(30), dec!(20)];
    /// let result = AllocationUtil::allocate_by_percentages(dec!(100), &percentages, 2);
    /// assert_eq!(result.iter().sum::<rust_decimal::Decimal>(), dec!(100));
    /// ```
    #[must_use]
    pub fn allocate_by_percentages(
        total: Decimal,
        percentages: &[Decimal],
        decimal_places: u32,
    ) -> Vec<Decimal> {
        if percentages.is_empty() {
            return vec![];
        }

        let hundred = Decimal::from(100);
        let unit = Decimal::new(1, decimal_places);

        // Round total to target precision first
        let total_rounded =
            total.round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointNearestEven);

        // Calculate exact allocations
        let exact: Vec<Decimal> = percentages
            .iter()
            .map(|p| total_rounded * *p / hundred)
            .collect();

        // Round down each
        let mut rounded: Vec<Decimal> = exact
            .iter()
            .map(|a| a.round_dp_with_strategy(decimal_places, RoundingStrategy::ToZero))
            .collect();

        // Calculate remainder to distribute
        let sum_rounded: Decimal = rounded.iter().copied().sum();
        let remainder = total_rounded - sum_rounded;

        // How many units to distribute
        let units_to_distribute = (remainder / unit)
            .round_dp_with_strategy(0, RoundingStrategy::ToZero)
            .to_u64()
            .unwrap_or(0);
        let units_to_distribute = usize::try_from(units_to_distribute).unwrap_or(0);

        if units_to_distribute == 0 {
            return rounded;
        }

        // Calculate fractional remainders for each allocation
        let mut remainders: Vec<(usize, Decimal)> = exact
            .iter()
            .zip(rounded.iter())
            .enumerate()
            .map(|(i, (e, r))| (i, *e - *r))
            .collect();

        // Sort by fractional remainder (largest first)
        remainders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Give +1 unit to items with largest remainders
        for (idx, _) in remainders.iter().take(units_to_distribute) {
            rounded[*idx] += unit;
        }

        rounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // =========================================================================
    // allocate_equal tests
    // =========================================================================

    #[test]
    fn test_allocate_equal_empty() {
        let result = AllocationUtil::allocate_equal(dec!(100), 0, 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_allocate_equal_single() {
        let result = AllocationUtil::allocate_equal(dec!(100), 1, 2);
        assert_eq!(result, vec![dec!(100)]);
    }

    #[test]
    fn test_allocate_equal_even_split() {
        // 100 / 2 = 50 each
        let result = AllocationUtil::allocate_equal(dec!(100), 2, 2);
        assert_eq!(result, vec![dec!(50), dec!(50)]);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(100));
    }

    #[test]
    fn test_allocate_equal_thirds() {
        // 100 / 3 = 33.33... → [33.34, 33.33, 33.33]
        let result = AllocationUtil::allocate_equal(dec!(100), 3, 2);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(100));
        // First recipient gets the extra cent
        assert_eq!(result[0], dec!(33.34));
        assert_eq!(result[1], dec!(33.33));
        assert_eq!(result[2], dec!(33.33));
    }

    #[test]
    fn test_allocate_equal_sum_invariant() {
        // Various amounts and counts - sum must always equal total
        let test_cases = [
            (dec!(100), 3),
            (dec!(100), 7),
            (dec!(1000), 3),
            (dec!(1), 3),
            (dec!(0.01), 3),
            (dec!(999.99), 7),
        ];

        for (total, count) in test_cases {
            let result = AllocationUtil::allocate_equal(total, count, 2);
            assert_eq!(
                result.iter().sum::<Decimal>(),
                total,
                "Sum invariant failed for total={total}, count={count}"
            );
        }
    }

    // =========================================================================
    // allocate_by_percentages tests
    // =========================================================================

    #[test]
    fn test_allocate_by_percentages_empty() {
        let result = AllocationUtil::allocate_by_percentages(dec!(100), &[], 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_allocate_by_percentages_single() {
        let result = AllocationUtil::allocate_by_percentages(dec!(100), &[dec!(100)], 2);
        assert_eq!(result, vec![dec!(100)]);
    }

    #[test]
    fn test_allocate_by_percentages_even() {
        // 50% / 50%
        let percentages = vec![dec!(50), dec!(50)];
        let result = AllocationUtil::allocate_by_percentages(dec!(100), &percentages, 2);
        assert_eq!(result, vec![dec!(50), dec!(50)]);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(100));
    }

    #[test]
    fn test_allocate_by_percentages_thirds() {
        // 33.33% / 33.33% / 33.34%
        let percentages = vec![dec!(33.33), dec!(33.33), dec!(33.34)];
        let result = AllocationUtil::allocate_by_percentages(dec!(100), &percentages, 2);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(100));
    }

    #[test]
    fn test_allocate_by_percentages_uneven() {
        // 50% / 30% / 20%
        let percentages = vec![dec!(50), dec!(30), dec!(20)];
        let result = AllocationUtil::allocate_by_percentages(dec!(100), &percentages, 2);
        assert_eq!(result, vec![dec!(50), dec!(30), dec!(20)]);
        assert_eq!(result.iter().sum::<Decimal>(), dec!(100));
    }

    #[test]
    fn test_allocate_by_percentages_sum_invariant() {
        // Various percentages - sum must always equal total
        let test_cases = [
            (dec!(100), vec![dec!(33.33), dec!(33.33), dec!(33.34)]),
            (dec!(1000), vec![dec!(25), dec!(25), dec!(25), dec!(25)]),
            (dec!(99.99), vec![dec!(10), dec!(20), dec!(30), dec!(40)]),
        ];

        for (total, percentages) in test_cases {
            let result = AllocationUtil::allocate_by_percentages(total, &percentages, 2);
            assert_eq!(
                result.iter().sum::<Decimal>(),
                total,
                "Sum invariant failed for total={total}, percentages={percentages:?}"
            );
        }
    }
}
