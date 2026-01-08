//! Exchange rate types and logic.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Exchange rate between two currencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate (1 from_currency = rate to_currency).
    pub rate: Decimal,
    /// Date this rate is effective.
    pub effective_date: NaiveDate,
}

impl ExchangeRate {
    /// Creates a new exchange rate.
    #[must_use]
    pub const fn new(
        from_currency: String,
        to_currency: String,
        rate: Decimal,
        effective_date: NaiveDate,
    ) -> Self {
        Self {
            from_currency,
            to_currency,
            rate,
            effective_date,
        }
    }

    /// Returns the inverse rate.
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self {
            from_currency: self.to_currency.clone(),
            to_currency: self.from_currency.clone(),
            rate: Decimal::ONE / self.rate,
            effective_date: self.effective_date,
        }
    }
}
