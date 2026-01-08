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
