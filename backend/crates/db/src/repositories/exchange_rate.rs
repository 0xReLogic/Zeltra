//! Exchange rate repository for currency conversion database operations.
//!
//! Implements Requirements 4.1-4.8 for exchange rate management.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::{
    currencies, exchange_rates,
    sea_orm_active_enums::RateSource,
};

/// Error types for exchange rate operations.
#[derive(Debug, thiserror::Error)]
pub enum ExchangeRateError {
    /// Rate must be positive.
    #[error("Exchange rate must be positive")]
    NonPositiveRate,

    /// Currencies must be different.
    #[error("From and to currencies must be different")]
    SameCurrency,

    /// Currency not found.
    #[error("Currency '{0}' not found")]
    CurrencyNotFound(String),

    /// Exchange rate not found.
    #[error("No exchange rate found for {0}/{1} on or before {2}")]
    RateNotFound(String, String, NaiveDate),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Input for creating or updating an exchange rate.
#[derive(Debug, Clone)]
pub struct CreateExchangeRateInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate (from_currency * rate = to_currency).
    pub rate: Decimal,
    /// Effective date for this rate.
    pub effective_date: NaiveDate,
    /// Source of the rate (manual, api, bank_feed).
    pub source: RateSource,
    /// Optional reference (e.g., API provider, bank name).
    pub source_reference: Option<String>,
    /// User who created/updated the rate.
    pub created_by: Option<Uuid>,
}

/// Result of an exchange rate lookup.
#[derive(Debug, Clone)]
pub struct ExchangeRateLookup {
    /// The exchange rate.
    pub rate: Decimal,
    /// How the rate was obtained.
    pub lookup_method: RateLookupMethod,
    /// The effective date of the rate.
    pub effective_date: NaiveDate,
}

/// How an exchange rate was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLookupMethod {
    /// Direct rate found (from_currency -> to_currency).
    Direct,
    /// Inverse rate calculated (to_currency -> from_currency, then inverted).
    Inverse,
    /// Triangulated through USD.
    Triangulated,
}

/// Exchange rate repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct ExchangeRateRepository {
    db: DatabaseConnection,
}

impl ExchangeRateRepository {
    /// Creates a new exchange rate repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates or updates an exchange rate (upsert behavior).
    ///
    /// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Rate is not positive
    /// - From and to currencies are the same
    /// - Either currency does not exist
    pub async fn create_or_update_rate(
        &self,
        input: CreateExchangeRateInput,
    ) -> Result<exchange_rates::Model, ExchangeRateError> {
        // Validate rate is positive (Requirement 4.2)
        if input.rate <= Decimal::ZERO {
            return Err(ExchangeRateError::NonPositiveRate);
        }

        // Validate currencies are different (Requirement 4.3)
        if input.from_currency == input.to_currency {
            return Err(ExchangeRateError::SameCurrency);
        }

        // Validate currencies exist (Requirement 4.4)
        let from_currency = currencies::Entity::find_by_id(&input.from_currency)
            .one(&self.db)
            .await?;
        if from_currency.is_none() {
            return Err(ExchangeRateError::CurrencyNotFound(input.from_currency));
        }

        let to_currency = currencies::Entity::find_by_id(&input.to_currency)
            .one(&self.db)
            .await?;
        if to_currency.is_none() {
            return Err(ExchangeRateError::CurrencyNotFound(input.to_currency));
        }

        // Check if rate already exists for this currency pair and date (Requirement 4.5)
        let existing = exchange_rates::Entity::find()
            .filter(exchange_rates::Column::OrganizationId.eq(input.organization_id))
            .filter(exchange_rates::Column::FromCurrency.eq(&input.from_currency))
            .filter(exchange_rates::Column::ToCurrency.eq(&input.to_currency))
            .filter(exchange_rates::Column::EffectiveDate.eq(input.effective_date))
            .one(&self.db)
            .await?;

        let now = chrono::Utc::now().into();

        if let Some(existing_rate) = existing {
            // Update existing rate
            let mut active: exchange_rates::ActiveModel = existing_rate.into();
            active.rate = Set(input.rate);
            active.source = Set(input.source);
            active.source_reference = Set(input.source_reference);
            if input.created_by.is_some() {
                active.created_by = Set(input.created_by);
            }
            // Note: created_at is not updated on upsert

            let updated = active.update(&self.db).await?;
            Ok(updated)
        } else {
            // Create new rate
            let rate = exchange_rates::ActiveModel {
                id: Set(Uuid::new_v4()),
                organization_id: Set(input.organization_id),
                from_currency: Set(input.from_currency),
                to_currency: Set(input.to_currency),
                rate: Set(input.rate),
                effective_date: Set(input.effective_date),
                source: Set(input.source),
                source_reference: Set(input.source_reference),
                created_by: Set(input.created_by),
                created_at: Set(now),
            };

            let result = rate.insert(&self.db).await?;
            Ok(result)
        }
    }

    /// Finds an exchange rate for a currency pair on or before a date.
    ///
    /// Requirements: 4.6, 4.7, 4.8
    ///
    /// Lookup priority:
    /// 1. Direct rate (from_currency -> to_currency)
    /// 2. Inverse rate (to_currency -> from_currency, then invert)
    /// 3. Triangulation through USD
    /// 4. Error if no rate found
    ///
    /// # Errors
    ///
    /// Returns an error if no rate can be found (direct, inverse, or triangulated).
    pub async fn find_rate(
        &self,
        organization_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Result<ExchangeRateLookup, ExchangeRateError> {
        // Same currency = rate of 1
        if from_currency == to_currency {
            return Ok(ExchangeRateLookup {
                rate: Decimal::ONE,
                lookup_method: RateLookupMethod::Direct,
                effective_date: date,
            });
        }

        // Try direct rate first (Requirement 4.6)
        if let Some(direct) = self
            .find_direct_rate(organization_id, from_currency, to_currency, date)
            .await?
        {
            return Ok(ExchangeRateLookup {
                rate: direct.rate,
                lookup_method: RateLookupMethod::Direct,
                effective_date: direct.effective_date,
            });
        }

        // Try inverse rate
        if let Some(inverse) = self
            .find_direct_rate(organization_id, to_currency, from_currency, date)
            .await?
        {
            // Invert the rate: if USD/EUR = 0.85, then EUR/USD = 1/0.85
            let inverted_rate = Decimal::ONE / inverse.rate;
            return Ok(ExchangeRateLookup {
                rate: inverted_rate,
                lookup_method: RateLookupMethod::Inverse,
                effective_date: inverse.effective_date,
            });
        }

        // Try triangulation through USD (Requirement 4.7)
        if from_currency != "USD" && to_currency != "USD" {
            if let Some(triangulated) = self
                .find_triangulated_rate(organization_id, from_currency, to_currency, date)
                .await?
            {
                return Ok(triangulated);
            }
        }

        // No rate found (Requirement 4.8)
        Err(ExchangeRateError::RateNotFound(
            from_currency.to_string(),
            to_currency.to_string(),
            date,
        ))
    }

    /// Finds a direct exchange rate (most recent on or before date).
    async fn find_direct_rate(
        &self,
        organization_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Result<Option<exchange_rates::Model>, ExchangeRateError> {
        let rate = exchange_rates::Entity::find()
            .filter(exchange_rates::Column::OrganizationId.eq(organization_id))
            .filter(exchange_rates::Column::FromCurrency.eq(from_currency))
            .filter(exchange_rates::Column::ToCurrency.eq(to_currency))
            .filter(exchange_rates::Column::EffectiveDate.lte(date))
            .order_by_desc(exchange_rates::Column::EffectiveDate)
            .one(&self.db)
            .await?;

        Ok(rate)
    }

    /// Attempts to find a rate via triangulation through USD.
    ///
    /// from_currency -> USD -> to_currency
    async fn find_triangulated_rate(
        &self,
        organization_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Result<Option<ExchangeRateLookup>, ExchangeRateError> {
        // Get from_currency -> USD rate
        let from_to_usd = self
            .find_rate_with_inverse(organization_id, from_currency, "USD", date)
            .await?;

        // Get USD -> to_currency rate
        let usd_to_target = self
            .find_rate_with_inverse(organization_id, "USD", to_currency, date)
            .await?;

        match (from_to_usd, usd_to_target) {
            (Some((rate1, date1)), Some((rate2, date2))) => {
                // Triangulated rate = from->USD * USD->to
                let triangulated_rate = rate1 * rate2;
                // Use the older of the two dates as the effective date
                let effective_date = date1.min(date2);

                Ok(Some(ExchangeRateLookup {
                    rate: triangulated_rate,
                    lookup_method: RateLookupMethod::Triangulated,
                    effective_date,
                }))
            }
            _ => Ok(None),
        }
    }

    /// Helper to find rate with inverse fallback (returns rate and effective date).
    async fn find_rate_with_inverse(
        &self,
        organization_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Result<Option<(Decimal, NaiveDate)>, ExchangeRateError> {
        // Try direct
        if let Some(direct) = self
            .find_direct_rate(organization_id, from_currency, to_currency, date)
            .await?
        {
            return Ok(Some((direct.rate, direct.effective_date)));
        }

        // Try inverse
        if let Some(inverse) = self
            .find_direct_rate(organization_id, to_currency, from_currency, date)
            .await?
        {
            let inverted_rate = Decimal::ONE / inverse.rate;
            return Ok(Some((inverted_rate, inverse.effective_date)));
        }

        Ok(None)
    }

    /// Lists all exchange rates for an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_rates(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<exchange_rates::Model>, ExchangeRateError> {
        let rates = exchange_rates::Entity::find()
            .filter(exchange_rates::Column::OrganizationId.eq(organization_id))
            .order_by_desc(exchange_rates::Column::EffectiveDate)
            .order_by_asc(exchange_rates::Column::FromCurrency)
            .order_by_asc(exchange_rates::Column::ToCurrency)
            .all(&self.db)
            .await?;

        Ok(rates)
    }
}


// ============================================================================
// Pure validation functions for property testing
// ============================================================================

/// Validates that an exchange rate is positive.
#[must_use]
pub fn validate_rate_positive(rate: Decimal) -> bool {
    rate > Decimal::ZERO
}

/// Validates that currencies are different.
#[must_use]
pub fn validate_currencies_different(from: &str, to: &str) -> bool {
    from != to
}

/// Represents a stored exchange rate for testing.
#[derive(Debug, Clone)]
pub struct StoredRate {
    /// Source currency.
    pub from_currency: String,
    /// Target currency.
    pub to_currency: String,
    /// Exchange rate.
    pub rate: Decimal,
    /// Effective date.
    pub effective_date: NaiveDate,
}

/// Simulates exchange rate lookup logic (pure function for testing).
///
/// Returns the rate and lookup method, or None if not found.
#[must_use]
pub fn simulate_rate_lookup(
    stored_rates: &[StoredRate],
    from_currency: &str,
    to_currency: &str,
    date: NaiveDate,
) -> Option<(Decimal, RateLookupMethod)> {
    // Same currency = rate of 1
    if from_currency == to_currency {
        return Some((Decimal::ONE, RateLookupMethod::Direct));
    }

    // Try direct rate
    if let Some(rate) = find_best_rate(stored_rates, from_currency, to_currency, date) {
        return Some((rate, RateLookupMethod::Direct));
    }

    // Try inverse rate
    if let Some(rate) = find_best_rate(stored_rates, to_currency, from_currency, date) {
        let inverted = Decimal::ONE / rate;
        return Some((inverted, RateLookupMethod::Inverse));
    }

    // Try triangulation through USD
    if from_currency != "USD" && to_currency != "USD" {
        let from_to_usd = find_best_rate(stored_rates, from_currency, "USD", date)
            .or_else(|| find_best_rate(stored_rates, "USD", from_currency, date).map(|r| Decimal::ONE / r));

        let usd_to_target = find_best_rate(stored_rates, "USD", to_currency, date)
            .or_else(|| find_best_rate(stored_rates, to_currency, "USD", date).map(|r| Decimal::ONE / r));

        if let (Some(r1), Some(r2)) = (from_to_usd, usd_to_target) {
            return Some((r1 * r2, RateLookupMethod::Triangulated));
        }
    }

    None
}

/// Finds the best (most recent) rate on or before the given date.
fn find_best_rate(
    stored_rates: &[StoredRate],
    from_currency: &str,
    to_currency: &str,
    date: NaiveDate,
) -> Option<Decimal> {
    stored_rates
        .iter()
        .filter(|r| {
            r.from_currency == from_currency
                && r.to_currency == to_currency
                && r.effective_date <= date
        })
        .max_by_key(|r| r.effective_date)
        .map(|r| r.rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rust_decimal_macros::dec;

    // ========================================================================
    // Property 8: Exchange Rate Lookup Priority
    // **Validates: Requirements 4.6, 4.7**
    // ========================================================================

    /// Strategy for generating valid currency codes
    fn currency_code_strategy() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "USD".to_string(),
            "EUR".to_string(),
            "GBP".to_string(),
            "JPY".to_string(),
            "IDR".to_string(),
            "SGD".to_string(),
            "AUD".to_string(),
        ])
    }

    /// Strategy for generating positive exchange rates
    fn rate_strategy() -> impl Strategy<Value = Decimal> {
        (1i64..10000i64).prop_map(|n| Decimal::new(n, 4))
    }

    /// Strategy for generating dates
    fn date_strategy() -> impl Strategy<Value = NaiveDate> {
        (2020i32..2026i32, 1u32..13u32, 1u32..29u32).prop_map(|(y, m, d)| {
            NaiveDate::from_ymd_opt(y, m, d).unwrap_or(NaiveDate::from_ymd_opt(y, m, 1).unwrap())
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 8.1: Direct rate takes priority over inverse**
        ///
        /// *For any* currency pair where both direct and inverse rates exist,
        /// the lookup SHALL return the direct rate.
        ///
        /// **Validates: Requirements 4.6**
        #[test]
        fn prop_direct_rate_priority(
            from in currency_code_strategy(),
            to in currency_code_strategy(),
            direct_rate in rate_strategy(),
            inverse_rate in rate_strategy(),
            date in date_strategy(),
        ) {
            prop_assume!(from != to);

            let stored_rates = vec![
                StoredRate {
                    from_currency: from.clone(),
                    to_currency: to.clone(),
                    rate: direct_rate,
                    effective_date: date,
                },
                StoredRate {
                    from_currency: to.clone(),
                    to_currency: from.clone(),
                    rate: inverse_rate,
                    effective_date: date,
                },
            ];

            let result = simulate_rate_lookup(&stored_rates, &from, &to, date);
            prop_assert!(result.is_some());

            let (rate, method) = result.unwrap();
            prop_assert_eq!(method, RateLookupMethod::Direct);
            prop_assert_eq!(rate, direct_rate);
        }

        /// **Property 8.2: Inverse rate used when no direct rate**
        ///
        /// *For any* currency pair where only inverse rate exists,
        /// the lookup SHALL return the inverted rate.
        ///
        /// **Validates: Requirements 4.6**
        #[test]
        fn prop_inverse_rate_fallback(
            from in currency_code_strategy(),
            to in currency_code_strategy(),
            inverse_rate in rate_strategy(),
            date in date_strategy(),
        ) {
            prop_assume!(from != to);

            // Only store the inverse rate (to -> from)
            let stored_rates = vec![StoredRate {
                from_currency: to.clone(),
                to_currency: from.clone(),
                rate: inverse_rate,
                effective_date: date,
            }];

            let result = simulate_rate_lookup(&stored_rates, &from, &to, date);
            prop_assert!(result.is_some());

            let (rate, method) = result.unwrap();
            prop_assert_eq!(method, RateLookupMethod::Inverse);

            // Rate should be inverted
            let expected = Decimal::ONE / inverse_rate;
            prop_assert_eq!(rate, expected);
        }

        /// **Property 8.3: Same currency returns rate of 1**
        ///
        /// *For any* currency, looking up the rate from itself to itself
        /// SHALL return a rate of 1.
        ///
        /// **Validates: Requirements 4.6**
        #[test]
        fn prop_same_currency_rate_one(
            currency in currency_code_strategy(),
            date in date_strategy(),
        ) {
            let stored_rates: Vec<StoredRate> = vec![];

            let result = simulate_rate_lookup(&stored_rates, &currency, &currency, date);
            prop_assert!(result.is_some());

            let (rate, method) = result.unwrap();
            prop_assert_eq!(rate, Decimal::ONE);
            prop_assert_eq!(method, RateLookupMethod::Direct);
        }

        /// **Property 8.4: Rate validation - positive rates only**
        ///
        /// *For any* positive rate, validation SHALL pass.
        /// *For any* zero or negative rate, validation SHALL fail.
        ///
        /// **Validates: Requirements 4.2**
        #[test]
        fn prop_rate_must_be_positive(
            rate_cents in -1000i64..1000i64,
        ) {
            let rate = Decimal::new(rate_cents, 2);
            let is_valid = validate_rate_positive(rate);

            if rate > Decimal::ZERO {
                prop_assert!(is_valid, "Positive rate should be valid");
            } else {
                prop_assert!(!is_valid, "Zero or negative rate should be invalid");
            }
        }

        /// **Property 8.5: Currency validation - must be different**
        ///
        /// *For any* two currencies, validation SHALL pass only if they are different.
        ///
        /// **Validates: Requirements 4.3**
        #[test]
        fn prop_currencies_must_differ(
            from in currency_code_strategy(),
            to in currency_code_strategy(),
        ) {
            let is_valid = validate_currencies_different(&from, &to);

            if from == to {
                prop_assert!(!is_valid, "Same currencies should be invalid");
            } else {
                prop_assert!(is_valid, "Different currencies should be valid");
            }
        }
    }

    // ========================================================================
    // Unit tests for triangulation and edge cases
    // ========================================================================

    #[test]
    fn test_triangulation_through_usd() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

        // EUR -> USD = 1.10, USD -> GBP = 0.80
        // Expected: EUR -> GBP = 1.10 * 0.80 = 0.88
        let stored_rates = vec![
            StoredRate {
                from_currency: "EUR".to_string(),
                to_currency: "USD".to_string(),
                rate: dec!(1.10),
                effective_date: date,
            },
            StoredRate {
                from_currency: "USD".to_string(),
                to_currency: "GBP".to_string(),
                rate: dec!(0.80),
                effective_date: date,
            },
        ];

        let result = simulate_rate_lookup(&stored_rates, "EUR", "GBP", date);
        assert!(result.is_some());

        let (rate, method) = result.unwrap();
        assert_eq!(method, RateLookupMethod::Triangulated);
        assert_eq!(rate, dec!(0.88));
    }

    #[test]
    fn test_triangulation_with_inverse_rates() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

        // Only have USD -> EUR and GBP -> USD (both need inversion)
        // EUR -> USD = 1/0.91 ≈ 1.0989, USD -> GBP = 1/1.25 = 0.80
        let stored_rates = vec![
            StoredRate {
                from_currency: "USD".to_string(),
                to_currency: "EUR".to_string(),
                rate: dec!(0.91),
                effective_date: date,
            },
            StoredRate {
                from_currency: "GBP".to_string(),
                to_currency: "USD".to_string(),
                rate: dec!(1.25),
                effective_date: date,
            },
        ];

        let result = simulate_rate_lookup(&stored_rates, "EUR", "GBP", date);
        assert!(result.is_some());

        let (rate, method) = result.unwrap();
        assert_eq!(method, RateLookupMethod::Triangulated);
        // EUR -> USD = 1/0.91, USD -> GBP = 1/1.25
        // EUR -> GBP = (1/0.91) * (1/1.25) = 1/(0.91 * 1.25) = 1/1.1375
        let expected = (Decimal::ONE / dec!(0.91)) * (Decimal::ONE / dec!(1.25));
        assert_eq!(rate, expected);
    }

    #[test]
    fn test_no_rate_found() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let stored_rates: Vec<StoredRate> = vec![];

        let result = simulate_rate_lookup(&stored_rates, "EUR", "GBP", date);
        assert!(result.is_none());
    }

    #[test]
    fn test_most_recent_rate_used() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let stored_rates = vec![
            StoredRate {
                from_currency: "EUR".to_string(),
                to_currency: "USD".to_string(),
                rate: dec!(1.05),
                effective_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            },
            StoredRate {
                from_currency: "EUR".to_string(),
                to_currency: "USD".to_string(),
                rate: dec!(1.10),
                effective_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            },
            StoredRate {
                from_currency: "EUR".to_string(),
                to_currency: "USD".to_string(),
                rate: dec!(1.15),
                effective_date: NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(), // Future rate
            },
        ];

        let result = simulate_rate_lookup(&stored_rates, "EUR", "USD", date);
        assert!(result.is_some());

        let (rate, method) = result.unwrap();
        assert_eq!(method, RateLookupMethod::Direct);
        // Should use Jan 10 rate (most recent on or before Jan 15)
        assert_eq!(rate, dec!(1.10));
    }

    #[test]
    fn test_usd_to_usd_no_triangulation() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let stored_rates: Vec<StoredRate> = vec![];

        // USD to USD should return 1, not attempt triangulation
        let result = simulate_rate_lookup(&stored_rates, "USD", "USD", date);
        assert!(result.is_some());

        let (rate, method) = result.unwrap();
        assert_eq!(rate, Decimal::ONE);
        assert_eq!(method, RateLookupMethod::Direct);
    }
}
