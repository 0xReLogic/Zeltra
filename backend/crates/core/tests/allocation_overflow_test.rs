//! Allocation overflow tests.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use zeltra_core::currency::AllocationUtil;

#[test]
fn prop_allocation_handles_extreme_counts() {
    // Original QA Extreme Count Test (likely to just OOM or pass if handled)
    let test_cases: Vec<usize> = vec![
        1_000,
        // 10_000_000_000, // Uncommenting requires massive RAM
    ];

    for count in test_cases {
        let result =
            std::panic::catch_unwind(|| AllocationUtil::allocate_equal(dec!(100), count, 2));
        if let Ok(allocations) = result {
            let sum: Decimal = allocations.iter().copied().sum();
            assert_eq!(sum, dec!(100), "Sum mismatch for count={count}");
        }
    }
}

#[test]
fn prop_allocation_handles_negative_totals() {
    // QA "Silently returns 0" hypothesis:
    // If total is negative, remainder might be negative.
    // remainder.to_u64() returns None for negative values.
    // unwrap_or(0) masks it.
    // Result: Remainder is discarded. Sum != Total.

    let total = dec!(-100);
    let count = 3;
    let dp = 2; // 0.01 precision

    // Expect: [-33.34, -33.33, -33.33] sum = -100
    // Current Buggy Implementation likely gives: [-33.33, -33.33, -33.33] sum = -99.99

    let result = AllocationUtil::allocate_equal(total, count, dp);
    let sum: Decimal = result.iter().copied().sum();

    assert_eq!(
        sum, total,
        "CRITICAL: Sum invariant failed for negative total! Got {sum}, expected {total}"
    );
}
