//! Multi-currency handling and exchange rates.
//!
//! This module provides:
//! - Currency conversion with Banker's Rounding
//! - Exchange rate types and operations
//! - Amount allocation using Largest Remainder Method
//! - Exchange rate fetching from external APIs (Frankfurter)
//! - Exchange rate service for coordinating fetch and storage

pub mod allocation;
pub mod conversion;
pub mod exchange;
pub mod fetcher;
pub mod forex;
pub mod rate_service;
pub mod service;

#[cfg(test)]
mod props;

#[cfg(test)]
mod rate_props;

pub use allocation::AllocationUtil;
pub use conversion::convert_amount;
pub use exchange::ExchangeRate;
pub use fetcher::{ExchangeRateFetcher, FetchedRate, FetcherError, RateSource};
pub use forex::calculate_forex_variance;
pub use rate_service::{
    BulkImportResult, BulkRateImportInput, ExchangeRateService, RateImportError, RateImportItem,
    RateServiceError,
};
pub use service::CurrencyService;
