//! Multi-currency handling and exchange rates.
//!
//! This module provides:
//! - Currency conversion with Banker's Rounding
//! - Exchange rate types and operations
//! - Amount allocation using Largest Remainder Method

pub mod allocation;
pub mod conversion;
pub mod exchange;
pub mod service;

#[cfg(test)]
mod props;

pub use allocation::AllocationUtil;
pub use conversion::convert_amount;
pub use exchange::ExchangeRate;
pub use service::CurrencyService;
