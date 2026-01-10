//! Report error types.

use chrono::NaiveDate;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during report generation.
#[derive(Debug, Error)]
pub enum ReportError {
    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Fiscal period not found.
    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    /// Invalid date range.
    #[error("Invalid date range: start {start} is after end {end}")]
    InvalidDateRange {
        /// Start date.
        start: NaiveDate,
        /// End date.
        end: NaiveDate,
    },

    /// Invalid dimension type.
    #[error("Invalid dimension type: {0}")]
    InvalidDimensionType(String),

    /// No data found.
    #[error("No data found for the specified criteria")]
    NoDataFound,
}
