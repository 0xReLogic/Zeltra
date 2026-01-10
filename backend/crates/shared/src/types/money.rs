//! Money type with decimal precision and currency.
//!
//! CRITICAL: Never use floating-point for money calculations.
//! This type wraps `rust_decimal::Decimal` for arbitrary precision.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a monetary amount with currency.
///
/// Uses `Decimal` internally to avoid floating-point precision errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// The amount in the smallest currency unit (e.g., cents).
    pub amount: Decimal,
    /// ISO 4217 currency code (e.g., "USD", "IDR").
    pub currency: Currency,
}

/// ISO 4217 currency codes supported by the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    /// US Dollar
    Usd,
    /// Indonesian Rupiah
    Idr,
    /// Euro
    Eur,
    /// Singapore Dollar
    Sgd,
    /// Japanese Yen
    Jpy,
}

impl Money {
    /// Creates a new Money instance.
    #[must_use]
    pub const fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Creates a zero amount in the specified currency.
    #[must_use]
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Returns true if the amount is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// Returns true if the amount is negative.
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usd => write!(f, "USD"),
            Self::Idr => write!(f, "IDR"),
            Self::Eur => write!(f, "EUR"),
            Self::Sgd => write!(f, "SGD"),
            Self::Jpy => write!(f, "JPY"),
        }
    }
}

impl std::str::FromStr for Currency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Self::Usd),
            "IDR" => Ok(Self::Idr),
            "EUR" => Ok(Self::Eur),
            "SGD" => Ok(Self::Sgd),
            "JPY" => Ok(Self::Jpy),
            _ => Err(format!("Unknown currency: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    #[test]
    fn test_money_new() {
        let amount = dec!(100.00);
        let money = Money::new(amount, Currency::Usd);
        assert_eq!(money.amount, amount);
        assert_eq!(money.currency, Currency::Usd);
    }

    #[test]
    fn test_money_zero() {
        let money = Money::zero(Currency::Idr);
        assert!(money.is_zero());
        assert_eq!(money.amount, Decimal::ZERO);
        assert_eq!(money.currency, Currency::Idr);
    }

    #[test]
    fn test_money_is_zero() {
        let zero_money = Money::new(dec!(0), Currency::Usd);
        assert!(zero_money.is_zero());

        let non_zero_money = Money::new(dec!(10), Currency::Usd);
        assert!(!non_zero_money.is_zero());
    }

    #[test]
    fn test_money_is_negative() {
        let positive = Money::new(dec!(10), Currency::Usd);
        assert!(!positive.is_negative());

        let negative = Money::new(dec!(-10), Currency::Usd);
        assert!(negative.is_negative());

        let zero = Money::new(dec!(0), Currency::Usd);
        assert!(!zero.is_negative());
    }

    #[test]
    fn test_currency_display() {
        assert_eq!(Currency::Usd.to_string(), "USD");
        assert_eq!(Currency::Idr.to_string(), "IDR");
        assert_eq!(Currency::Eur.to_string(), "EUR");
        assert_eq!(Currency::Sgd.to_string(), "SGD");
        assert_eq!(Currency::Jpy.to_string(), "JPY");
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(Currency::from_str("USD").unwrap(), Currency::Usd);
        assert_eq!(Currency::from_str("usd").unwrap(), Currency::Usd);
        assert_eq!(Currency::from_str("IDR").unwrap(), Currency::Idr);
        assert_eq!(Currency::from_str("EUR").unwrap(), Currency::Eur);
        assert_eq!(Currency::from_str("SGD").unwrap(), Currency::Sgd);
        assert_eq!(Currency::from_str("JPY").unwrap(), Currency::Jpy);

        assert!(Currency::from_str("XXX").is_err());
        assert!(Currency::from_str("").is_err());
    }
}
