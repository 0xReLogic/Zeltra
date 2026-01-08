//! Currency conversion logic.
//!
//! CRITICAL: Rounding strategy for multi-currency:
//! - Always round to currency's decimal places
//! - Use banker's rounding (round half to even)
//! - Store both original and converted amounts

use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

/// Converts an amount using the given exchange rate.
///
/// Uses banker's rounding (round half to even) to minimize cumulative errors.
#[must_use]
pub fn convert_amount(amount: Decimal, rate: Decimal, decimal_places: u32) -> Decimal {
    let converted = amount * rate;
    converted.round_dp_with_strategy(decimal_places, RoundingStrategy::MidpointNearestEven)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_convert_amount() {
        // 100 USD * 15000 = 1,500,000 IDR
        let amount = dec!(100);
        let rate = dec!(15000);
        let result = convert_amount(amount, rate, 0);
        assert_eq!(result, dec!(1500000));
    }

    #[test]
    fn test_convert_with_rounding() {
        // 100.50 USD * 15000.5 = 1,507,550.25 IDR -> rounds to 1,507,550
        let amount = dec!(100.50);
        let rate = dec!(15000.5);
        let result = convert_amount(amount, rate, 0);
        assert_eq!(result, dec!(1507550));
    }

    #[test]
    fn test_bankers_rounding() {
        // Test banker's rounding (round half to even)
        // 2.5 rounds to 2, 3.5 rounds to 4
        let result1 = convert_amount(dec!(1), dec!(2.5), 0);
        assert_eq!(result1, dec!(2)); // rounds to even

        let result2 = convert_amount(dec!(1), dec!(3.5), 0);
        assert_eq!(result2, dec!(4)); // rounds to even
    }
}
