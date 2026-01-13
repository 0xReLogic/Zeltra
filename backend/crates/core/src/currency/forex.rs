//! Forex Gain/Loss calculations.
//!
//! Provides utilities for calculating realized and unrealized foreign exchange gains and losses.

use rust_decimal::Decimal;

/// Calculates the variance in functional currency between two exchange rates.
///
/// # Arguments
///
/// * `source_amount` - The amount in source currency.
/// * `original_rate` - The exchange rate at the time of the original transaction (e.g., Invoice).
/// * `settlement_rate` - The exchange rate at the time of settlement (e.g., Payment).
///
/// # Returns
///
/// The variance in functional currency.
///
/// * Positive value indicates an increase in functional value (Gain for Assets, Loss for Liabilities).
/// * Negative value indicates a decrease in functional value (Loss for Assets, Gain for Liabilities).
#[must_use]
pub fn calculate_forex_variance(
    source_amount: Decimal,
    original_rate: Decimal,
    settlement_rate: Decimal,
) -> Decimal {
    let original_functional = source_amount * original_rate;
    let settlement_functional = source_amount * settlement_rate;

    settlement_functional - original_functional
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_variance_gain() {
        // Invoice: 100 USD @ 15000 = 1,500,000 IDR
        // Payment: 100 USD @ 15200 = 1,520,000 IDR
        // Variance: +20,000 IDR
        let variance = calculate_forex_variance(dec!(100), dec!(15000), dec!(15200));
        assert_eq!(variance, dec!(20000));
    }

    #[test]
    fn test_calculate_variance_loss() {
        // Invoice: 100 USD @ 15000 = 1,500,000 IDR
        // Payment: 100 USD @ 14800 = 1,480,000 IDR
        // Variance: -20,000 IDR
        let variance = calculate_forex_variance(dec!(100), dec!(15000), dec!(14800));
        assert_eq!(variance, dec!(-20000));
    }
}
