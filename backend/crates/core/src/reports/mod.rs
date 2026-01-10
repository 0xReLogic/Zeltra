//! Financial report generation.
//!
//! This module provides pure business logic for generating financial reports:
//! - Trial Balance
//! - Balance Sheet
//! - Income Statement
//! - Account Ledger
//! - Dimensional Reports

pub mod error;
pub mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use error::ReportError;
pub use service::ReportService;
pub use types::*;
