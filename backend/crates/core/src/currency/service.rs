//! Currency service for conversion and exchange rate operations.
//!
//! This module provides the main service interface for currency operations,
//! including conversion with Banker's Rounding and exchange rate lookup.

use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Currency service for conversion operations.
///
/// Provides methods for converting amounts between currencies using
/// Banker's Rounding (MidpointNearestEven) strategy.
pub struct CurrencyService;

impl CurrencyService {
    /// Convert amount using exchange rate with Banker's Rounding.
    ///
    /// Uses `RoundingStrategy::MidpointNearestEven` (Banker's Rounding) which:
    /// - Rounds 2.5 → 2 (to nearest even)
    /// - Rounds 3.5 → 4 (to nearest even)
    /// - Rounds 2.25 → 2.2 (to nearest even at 1 decimal)
    /// - Rounds 2.35 → 2.4 (to nearest even at 1 decimal)
    ///
    /// # Arguments
    ///
    /// * `amount` - The source amount to convert
    /// * `rate` - The exchange rate (1 source = rate target)
    ///
    /// # Returns
    ///
    /// The converted amount rounded to 4 decimal places using Banker's Rounding.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal_macros::dec;
    /// use zeltra_core::currency::CurrencyService;
    ///
    /// let result = CurrencyService::convert(dec!(100), dec!(1.5));
    /// assert_eq!(result, dec!(150.0000));
    /// ```
    #[must_use]
    pub fn convert(amount: Decimal, rate: Decimal) -> Decimal {
        (amount * rate).round_dp_with_strategy(4, RoundingStrategy::MidpointNearestEven)
    }

    /// Convert amount with custom decimal places.
    ///
    /// # Arguments
    ///
    /// * `amount` - The source amount to convert
    /// * `rate` - The exchange rate
    /// * `decimal_places` - Number of decimal places to round to
    ///
    /// # Returns
    ///
    /// The converted amount rounded to specified decimal places using Banker's Rounding.
    #[must_use]
    pub fn convert_with_precision(amount: Decimal, rate: Decimal, decimal_places: u32) -> Decimal {
        (amount * rate).round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointNearestEven)
    }

    /// Round a decimal value using Banker's Rounding.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to round
    /// * `decimal_places` - Number of decimal places
    ///
    /// # Returns
    ///
    /// The rounded value using Banker's Rounding (MidpointNearestEven).
    #[must_use]
    pub fn round(value: Decimal, decimal_places: u32) -> Decimal {
        value.round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointNearestEven)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_convert_basic() {
        // 100 * 1.5 = 150
        let result = CurrencyService::convert(dec!(100), dec!(1.5));
        assert_eq!(result, dec!(150.0000));
    }

    #[test]
    fn test_convert_same_currency() {
        // Same currency: rate = 1, result = source
        let result = CurrencyService::convert(dec!(100.50), Decimal::ONE);
        assert_eq!(result, dec!(100.5000));
    }

    #[test]
    fn test_convert_rounds_to_4_decimals() {
        // 100 * 1.23456789 = 123.456789 → rounds to 123.4568
        let result = CurrencyService::convert(dec!(100), dec!(1.23456789));
        assert_eq!(result, dec!(123.4568));
    }

    #[test]
    fn test_bankers_rounding_midpoint_to_even() {
        // 2.5 → 2 (nearest even)
        let result = CurrencyService::round(dec!(2.5), 0);
        assert_eq!(result, dec!(2));

        // 3.5 → 4 (nearest even)
        let result = CurrencyService::round(dec!(3.5), 0);
        assert_eq!(result, dec!(4));

        // 2.25 → 2.2 (nearest even at 1 decimal)
        let result = CurrencyService::round(dec!(2.25), 1);
        assert_eq!(result, dec!(2.2));

        // 2.35 → 2.4 (nearest even at 1 decimal)
        let result = CurrencyService::round(dec!(2.35), 1);
        assert_eq!(result, dec!(2.4));
    }

    #[test]
    fn test_convert_with_precision() {
        // 100 * 1.5 = 150, rounded to 2 decimals
        let result = CurrencyService::convert_with_precision(dec!(100), dec!(1.5), 2);
        assert_eq!(result, dec!(150.00));

        // 100 * 1.23456 = 123.456, rounded to 0 decimals
        let result = CurrencyService::convert_with_precision(dec!(100), dec!(1.23456), 0);
        assert_eq!(result, dec!(123));
    }
}
