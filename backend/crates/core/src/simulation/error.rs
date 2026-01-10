//! Simulation error types.

use chrono::NaiveDate;
use thiserror::Error;

/// Simulation-related errors.
#[derive(Debug, Error)]
pub enum SimulationError {
    /// Invalid base period (start after end).
    #[error("Invalid base period: start {start} is after end {end}")]
    InvalidBasePeriod {
        /// Start date.
        start: NaiveDate,
        /// End date.
        end: NaiveDate,
    },

    /// Invalid projection months (must be 1-60).
    #[error("Projection months must be between 1 and 60")]
    InvalidProjectionMonths,

    /// Invalid growth rate (must be -1.0 to 10.0).
    #[error("Growth rate must be between -1.0 and 10.0")]
    InvalidGrowthRate,

    /// No historical data found.
    #[error("No historical data found for the base period")]
    NoHistoricalData,
}
