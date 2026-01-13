//! Exchange rate fetcher for external APIs.
//!
//! This module provides the `ExchangeRateFetcher` service that retrieves
//! exchange rates from external sources like the Frankfurter API.

use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during exchange rate fetching.
#[derive(Debug, Error)]
pub enum FetcherError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse response.
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Invalid currency code.
    #[error("Invalid currency code: {0}")]
    InvalidCurrency(String),

    /// API returned an error.
    #[error("API error: {0}")]
    ApiError(String),

    /// Rate source is not configured for fetching.
    #[error("Rate source does not support fetching: {0}")]
    UnsupportedSource(String),
}

/// Source of exchange rates.
#[derive(Debug, Clone)]
pub enum RateSource {
    /// Frankfurter API (ECB data).
    Frankfurter {
        /// Base URL for the API.
        base_url: String,
    },
    /// Mock source for testing.
    Mock,
    /// Manual entry only (no fetching).
    Manual,
}

impl Default for RateSource {
    fn default() -> Self {
        Self::Frankfurter {
            base_url: "https://api.frankfurter.app".to_string(),
        }
    }
}

impl RateSource {
    /// Create a Frankfurter source with default URL.
    #[must_use]
    pub fn frankfurter() -> Self {
        Self::default()
    }

    /// Create a Frankfurter source with custom URL.
    #[must_use]
    pub fn frankfurter_with_url(base_url: impl Into<String>) -> Self {
        Self::Frankfurter {
            base_url: base_url.into(),
        }
    }

    /// Create from environment variable.
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var("EXCHANGE_RATE_SOURCE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "mock" => Self::Mock,
            "manual" => Self::Manual,
            "frankfurter" | "" => {
                let base_url = std::env::var("FRANKFURTER_API_URL")
                    .unwrap_or_else(|_| "https://api.frankfurter.app".to_string());
                Self::Frankfurter { base_url }
            }
            other => {
                tracing::warn!("Unknown rate source '{}', defaulting to Frankfurter", other);
                Self::default()
            }
        }
    }
}

/// A fetched exchange rate.
#[derive(Debug, Clone)]
pub struct FetchedRate {
    /// Source currency code (e.g., "EUR").
    pub from_currency: String,
    /// Target currency code (e.g., "USD").
    pub to_currency: String,
    /// Exchange rate (1 from_currency = rate to_currency).
    pub rate: Decimal,
    /// Date this rate is effective.
    pub effective_date: NaiveDate,
    /// Source of the rate.
    pub source: String,
}

/// Frankfurter API response structure.
#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    /// Amount (always 1).
    #[allow(dead_code)]
    amount: f64,
    /// Base currency.
    base: String,
    /// Date of the rates.
    date: String,
    /// Map of currency code to rate.
    rates: HashMap<String, f64>,
}

/// Exchange rate fetcher service.
///
/// Retrieves exchange rates from external APIs like Frankfurter.
pub struct ExchangeRateFetcher {
    client: Client,
    source: RateSource,
}

impl ExchangeRateFetcher {
    /// Create a new fetcher with the given source.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(source: RateSource) -> Result<Self, FetcherError> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self { client, source })
    }

    /// Create a new fetcher from environment configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn from_env() -> Result<Self, FetcherError> {
        Self::new(RateSource::from_env())
    }

    /// Get the current rate source.
    #[must_use]
    pub const fn source(&self) -> &RateSource {
        &self.source
    }

    /// Fetch the latest exchange rates.
    ///
    /// # Arguments
    ///
    /// * `base_currency` - The base currency code (e.g., "EUR").
    /// * `target_currencies` - List of target currency codes to fetch rates for.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The rate source doesn't support fetching (Manual).
    /// - The HTTP request fails.
    /// - The response cannot be parsed.
    pub async fn fetch_latest(
        &self,
        base_currency: &str,
        target_currencies: &[String],
    ) -> Result<Vec<FetchedRate>, FetcherError> {
        match &self.source {
            RateSource::Frankfurter { base_url } => {
                self.fetch_from_frankfurter(base_url, base_currency, target_currencies, None)
                    .await
            }
            RateSource::Mock => Ok(Self::generate_mock_rates(base_currency, target_currencies)),
            RateSource::Manual => Err(FetcherError::UnsupportedSource("manual".to_string())),
        }
    }

    /// Fetch historical exchange rates for a specific date.
    ///
    /// # Arguments
    ///
    /// * `base_currency` - The base currency code (e.g., "EUR").
    /// * `target_currencies` - List of target currency codes to fetch rates for.
    /// * `date` - The date to fetch rates for.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The rate source doesn't support fetching (Manual).
    /// - The HTTP request fails.
    /// - The response cannot be parsed.
    pub async fn fetch_historical(
        &self,
        base_currency: &str,
        target_currencies: &[String],
        date: NaiveDate,
    ) -> Result<Vec<FetchedRate>, FetcherError> {
        match &self.source {
            RateSource::Frankfurter { base_url } => {
                self.fetch_from_frankfurter(base_url, base_currency, target_currencies, Some(date))
                    .await
            }
            RateSource::Mock => Ok(Self::generate_mock_rates_for_date(
                base_currency,
                target_currencies,
                date,
            )),
            RateSource::Manual => Err(FetcherError::UnsupportedSource("manual".to_string())),
        }
    }

    /// Fetch rates from Frankfurter API.
    async fn fetch_from_frankfurter(
        &self,
        base_url: &str,
        base_currency: &str,
        target_currencies: &[String],
        rate_date: Option<NaiveDate>,
    ) -> Result<Vec<FetchedRate>, FetcherError> {
        // Build URL
        let date_path = rate_date.map_or_else(
            || "latest".to_string(),
            |d| d.format("%Y-%m-%d").to_string(),
        );
        let symbols = target_currencies.join(",");

        let url = format!(
            "{}/{}?from={}&to={}",
            base_url.trim_end_matches('/'),
            date_path,
            base_currency.to_uppercase(),
            symbols.to_uppercase()
        );

        tracing::debug!("Fetching exchange rates from: {url}");

        // Make request
        let response = self.client.get(&url).send().await?;

        // Check status
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(FetcherError::ApiError(format!("HTTP {status}: {body}")));
        }

        // Parse response
        let resp_data: FrankfurterResponse = response.json().await?;

        // Parse date
        let effective_date =
            NaiveDate::parse_from_str(&resp_data.date, "%Y-%m-%d").map_err(|e| {
                FetcherError::ParseError(format!("Invalid date '{}': {e}", resp_data.date))
            })?;

        // Convert to FetchedRate
        let rates = resp_data
            .rates
            .into_iter()
            .map(|(currency, rate)| {
                let decimal_rate = Decimal::from_str(&rate.to_string())
                    .unwrap_or_else(|_| Decimal::from_f64_retain(rate).unwrap_or(Decimal::ONE));

                FetchedRate {
                    from_currency: resp_data.base.clone(),
                    to_currency: currency,
                    rate: decimal_rate,
                    effective_date,
                    source: "frankfurter".to_string(),
                }
            })
            .collect();

        Ok(rates)
    }

    /// Generate mock rates for testing.
    fn generate_mock_rates(base_currency: &str, target_currencies: &[String]) -> Vec<FetchedRate> {
        let today = chrono::Utc::now().date_naive();
        Self::create_mock_rates(base_currency, target_currencies, today)
    }

    /// Generate mock rates for a specific date.
    fn generate_mock_rates_for_date(
        base_currency: &str,
        target_currencies: &[String],
        rate_date: NaiveDate,
    ) -> Vec<FetchedRate> {
        Self::create_mock_rates(base_currency, target_currencies, rate_date)
    }

    /// Create mock rates (associated function).
    fn create_mock_rates(
        base_currency: &str,
        target_currencies: &[String],
        rate_date: NaiveDate,
    ) -> Vec<FetchedRate> {
        // Fixed mock rates for common currencies
        let mock_rates: HashMap<&str, Decimal> = [
            ("USD", Decimal::new(108, 2)),       // 1.08
            ("GBP", Decimal::new(86, 2)),        // 0.86
            ("JPY", Decimal::new(16200, 2)),     // 162.00
            ("CHF", Decimal::new(96, 2)),        // 0.96
            ("CAD", Decimal::new(147, 2)),       // 1.47
            ("AUD", Decimal::new(165, 2)),       // 1.65
            ("CNY", Decimal::new(785, 2)),       // 7.85
            ("IDR", Decimal::new(1_720_000, 2)), // 17200.00
        ]
        .into_iter()
        .collect();

        target_currencies
            .iter()
            .filter_map(|currency| {
                let rate = mock_rates.get(currency.to_uppercase().as_str())?;
                Some(FetchedRate {
                    from_currency: base_currency.to_uppercase(),
                    to_currency: currency.to_uppercase(),
                    rate: *rate,
                    effective_date: rate_date,
                    source: "mock".to_string(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_source_default() {
        let source = RateSource::default();
        assert!(matches!(source, RateSource::Frankfurter { .. }));

        if let RateSource::Frankfurter { base_url } = source {
            assert_eq!(base_url, "https://api.frankfurter.app");
        }
    }

    #[test]
    fn test_rate_source_frankfurter_custom_url() {
        let source = RateSource::frankfurter_with_url("https://custom.api.com");
        if let RateSource::Frankfurter { base_url } = source {
            assert_eq!(base_url, "https://custom.api.com");
        } else {
            panic!("Expected Frankfurter source");
        }
    }

    #[test]
    fn test_mock_rates_generation() {
        let rates = ExchangeRateFetcher::generate_mock_rates(
            "EUR",
            &["USD".to_string(), "GBP".to_string(), "JPY".to_string()],
        );

        assert_eq!(rates.len(), 3);

        let usd_rate = rates.iter().find(|r| r.to_currency == "USD").unwrap();
        assert_eq!(usd_rate.from_currency, "EUR");
        assert_eq!(usd_rate.rate, Decimal::new(108, 2));
        assert_eq!(usd_rate.source, "mock");
    }

    #[test]
    fn test_mock_rates_for_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let rates = ExchangeRateFetcher::generate_mock_rates_for_date(
            "EUR",
            &["USD".to_string(), "GBP".to_string()],
            date,
        );

        assert_eq!(rates.len(), 2);
        assert!(rates.iter().all(|r| r.effective_date == date));
    }

    #[test]
    fn test_mock_rates_unknown_currency() {
        let rates = ExchangeRateFetcher::generate_mock_rates(
            "EUR",
            &["XYZ".to_string()], // Unknown currency
        );

        assert!(rates.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_latest_mock() {
        let fetcher = ExchangeRateFetcher::new(RateSource::Mock).unwrap();
        let rates = fetcher
            .fetch_latest("EUR", &["USD".to_string(), "GBP".to_string()])
            .await
            .unwrap();

        assert_eq!(rates.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_manual_returns_error() {
        let fetcher = ExchangeRateFetcher::new(RateSource::Manual).unwrap();
        let result = fetcher.fetch_latest("EUR", &["USD".to_string()]).await;

        assert!(matches!(result, Err(FetcherError::UnsupportedSource(_))));
    }

    #[tokio::test]
    async fn test_fetch_historical_mock() {
        let fetcher = ExchangeRateFetcher::new(RateSource::Mock).unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let rates = fetcher
            .fetch_historical("EUR", &["USD".to_string()], date)
            .await
            .unwrap();

        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].effective_date, date);
    }

    #[test]
    fn test_fetched_rate_fields() {
        let rate = FetchedRate {
            from_currency: "EUR".to_string(),
            to_currency: "USD".to_string(),
            rate: Decimal::new(108, 2),
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            source: "test".to_string(),
        };

        assert_eq!(rate.from_currency, "EUR");
        assert_eq!(rate.to_currency, "USD");
        assert_eq!(rate.rate, Decimal::new(108, 2));
        assert_eq!(rate.source, "test");
    }
}
