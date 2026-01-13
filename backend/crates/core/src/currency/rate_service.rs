//! Exchange rate service for managing rates from various sources.
//!
//! This module provides the `ExchangeRateService` that coordinates between
//! the fetcher (external APIs) and storage (database).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

use super::fetcher::{ExchangeRateFetcher, FetchedRate, FetcherError, RateSource};

/// Errors that can occur during exchange rate service operations.
#[derive(Debug, Error)]
pub enum RateServiceError {
    /// Fetcher error.
    #[error("Failed to fetch rates: {0}")]
    FetchError(#[from] FetcherError),

    /// Validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Storage error.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Rate not found.
    #[error("Rate not found for {from}/{to} on {date}")]
    RateNotFound {
        /// Source currency.
        from: String,
        /// Target currency.
        to: String,
        /// Date.
        date: NaiveDate,
    },
}

/// Input for bulk rate import.
#[derive(Debug, Clone)]
pub struct BulkRateImportInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Rates to import.
    pub rates: Vec<RateImportItem>,
    /// User performing the import.
    pub imported_by: Option<Uuid>,
}

/// A single rate to import.
#[derive(Debug, Clone)]
pub struct RateImportItem {
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate.
    pub rate: Decimal,
    /// Effective date.
    pub effective_date: NaiveDate,
}

/// Result of a bulk import operation.
#[derive(Debug, Clone)]
pub struct BulkImportResult {
    /// Number of rates imported (new).
    pub imported_count: usize,
    /// Number of rates updated (existing).
    pub updated_count: usize,
    /// Validation errors for individual rates.
    pub errors: Vec<RateImportError>,
}

/// Error for a single rate in bulk import.
#[derive(Debug, Clone)]
pub struct RateImportError {
    /// Index of the rate in the input.
    pub index: usize,
    /// Currency pair.
    pub currency_pair: String,
    /// Error message.
    pub message: String,
}

/// Result of fetching rates from external API.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Rates that were fetched.
    pub fetched_rates: Vec<FetchedRate>,
    /// Number of rates stored.
    pub stored_count: usize,
}

/// Exchange rate service for coordinating rate operations.
pub struct ExchangeRateService {
    fetcher: ExchangeRateFetcher,
}

impl ExchangeRateService {
    /// Create a new exchange rate service.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetcher cannot be created.
    pub fn new(source: RateSource) -> Result<Self, RateServiceError> {
        let fetcher = ExchangeRateFetcher::new(source)?;
        Ok(Self { fetcher })
    }

    /// Create a new exchange rate service from environment configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the fetcher cannot be created.
    pub fn from_env() -> Result<Self, RateServiceError> {
        let fetcher = ExchangeRateFetcher::from_env()?;
        Ok(Self { fetcher })
    }

    /// Fetch latest rates from external API.
    ///
    /// This only fetches the rates - storage is handled by the caller
    /// (typically the API layer with database access).
    ///
    /// # Arguments
    ///
    /// * `base_currency` - The base currency code (e.g., "EUR").
    /// * `target_currencies` - List of target currency codes.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching fails.
    pub async fn fetch_latest(
        &self,
        base_currency: &str,
        target_currencies: &[String],
    ) -> Result<Vec<FetchedRate>, RateServiceError> {
        let rates = self
            .fetcher
            .fetch_latest(base_currency, target_currencies)
            .await?;
        Ok(rates)
    }

    /// Fetch historical rates from external API.
    ///
    /// # Arguments
    ///
    /// * `base_currency` - The base currency code.
    /// * `target_currencies` - List of target currency codes.
    /// * `date` - The date to fetch rates for.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching fails.
    pub async fn fetch_historical(
        &self,
        base_currency: &str,
        target_currencies: &[String],
        date: NaiveDate,
    ) -> Result<Vec<FetchedRate>, RateServiceError> {
        let rates = self
            .fetcher
            .fetch_historical(base_currency, target_currencies, date)
            .await?;
        Ok(rates)
    }

    /// Validate a bulk import request.
    ///
    /// Returns validation errors for any invalid rates.
    /// If the returned vector is empty, all rates are valid.
    #[must_use]
    pub fn validate_bulk_import(input: &BulkRateImportInput) -> Vec<RateImportError> {
        let mut errors = Vec::new();

        for (index, rate) in input.rates.iter().enumerate() {
            // Validate rate is positive
            if rate.rate <= Decimal::ZERO {
                errors.push(RateImportError {
                    index,
                    currency_pair: format!("{}/{}", rate.from_currency, rate.to_currency),
                    message: "Rate must be positive".to_string(),
                });
                continue;
            }

            // Validate currencies are different
            if rate.from_currency == rate.to_currency {
                errors.push(RateImportError {
                    index,
                    currency_pair: format!("{}/{}", rate.from_currency, rate.to_currency),
                    message: "From and to currencies must be different".to_string(),
                });
                continue;
            }

            // Validate currency codes are valid (3 uppercase letters)
            if !is_valid_currency_code(&rate.from_currency) {
                errors.push(RateImportError {
                    index,
                    currency_pair: format!("{}/{}", rate.from_currency, rate.to_currency),
                    message: format!("Invalid from currency code: {}", rate.from_currency),
                });
                continue;
            }

            if !is_valid_currency_code(&rate.to_currency) {
                errors.push(RateImportError {
                    index,
                    currency_pair: format!("{}/{}", rate.from_currency, rate.to_currency),
                    message: format!("Invalid to currency code: {}", rate.to_currency),
                });
            }
        }

        errors
    }

    /// Get the current rate source.
    #[must_use]
    pub const fn source(&self) -> &RateSource {
        self.fetcher.source()
    }
}

/// Validate a currency code (3 uppercase letters).
fn is_valid_currency_code(code: &str) -> bool {
    code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase())
}

/// Parse a rate string to Decimal.
///
/// # Errors
///
/// Returns an error if the string cannot be parsed.
pub fn parse_rate(rate_str: &str) -> Result<Decimal, RateServiceError> {
    Decimal::from_str(rate_str)
        .map_err(|e| RateServiceError::ValidationError(format!("Invalid rate '{rate_str}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bulk_import_valid() {
        let input = BulkRateImportInput {
            organization_id: Uuid::new_v4(),
            rates: vec![
                RateImportItem {
                    from_currency: "EUR".to_string(),
                    to_currency: "USD".to_string(),
                    rate: Decimal::new(108, 2),
                    effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                },
                RateImportItem {
                    from_currency: "EUR".to_string(),
                    to_currency: "GBP".to_string(),
                    rate: Decimal::new(86, 2),
                    effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                },
            ],
            imported_by: None,
        };

        let errors = ExchangeRateService::validate_bulk_import(&input);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_bulk_import_negative_rate() {
        let input = BulkRateImportInput {
            organization_id: Uuid::new_v4(),
            rates: vec![RateImportItem {
                from_currency: "EUR".to_string(),
                to_currency: "USD".to_string(),
                rate: Decimal::new(-108, 2),
                effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            }],
            imported_by: None,
        };

        let errors = ExchangeRateService::validate_bulk_import(&input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("positive"));
    }

    #[test]
    fn test_validate_bulk_import_same_currency() {
        let input = BulkRateImportInput {
            organization_id: Uuid::new_v4(),
            rates: vec![RateImportItem {
                from_currency: "EUR".to_string(),
                to_currency: "EUR".to_string(),
                rate: Decimal::ONE,
                effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            }],
            imported_by: None,
        };

        let errors = ExchangeRateService::validate_bulk_import(&input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("different"));
    }

    #[test]
    fn test_validate_bulk_import_invalid_currency_code() {
        let input = BulkRateImportInput {
            organization_id: Uuid::new_v4(),
            rates: vec![RateImportItem {
                from_currency: "eu".to_string(), // lowercase
                to_currency: "USD".to_string(),
                rate: Decimal::new(108, 2),
                effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            }],
            imported_by: None,
        };

        let errors = ExchangeRateService::validate_bulk_import(&input);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Invalid from currency"));
    }

    #[test]
    fn test_is_valid_currency_code() {
        assert!(is_valid_currency_code("USD"));
        assert!(is_valid_currency_code("EUR"));
        assert!(is_valid_currency_code("IDR"));

        assert!(!is_valid_currency_code("usd")); // lowercase
        assert!(!is_valid_currency_code("US")); // too short
        assert!(!is_valid_currency_code("USDD")); // too long
        assert!(!is_valid_currency_code("US1")); // contains digit
    }

    #[test]
    fn test_parse_rate() {
        assert_eq!(parse_rate("1.08").unwrap(), Decimal::new(108, 2));
        assert_eq!(parse_rate("0.86").unwrap(), Decimal::new(86, 2));
        assert_eq!(parse_rate("162.00").unwrap(), Decimal::new(16200, 2));

        assert!(parse_rate("invalid").is_err());
        assert!(parse_rate("").is_err());
    }

    #[tokio::test]
    async fn test_fetch_latest_mock() {
        let service = ExchangeRateService::new(RateSource::Mock).unwrap();
        let rates = service
            .fetch_latest("EUR", &["USD".to_string(), "GBP".to_string()])
            .await
            .unwrap();

        assert_eq!(rates.len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_historical_mock() {
        let service = ExchangeRateService::new(RateSource::Mock).unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let rates = service
            .fetch_historical("EUR", &["USD".to_string()], date)
            .await
            .unwrap();

        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].effective_date, date);
    }
}
