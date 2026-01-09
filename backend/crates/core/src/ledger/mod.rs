//! Double-entry bookkeeping logic.
//!
//! This module implements the core ledger functionality:
//! - Ledger entries (debits and credits)
//! - Transaction aggregates
//! - Balance calculations
//! - Business rule validation
//! - Domain types for transaction creation
//! - Error types for ledger operations
//! - Ledger service for transaction validation
//! - Fiscal period validation

pub mod balance;
pub mod entry;
pub mod error;
pub mod fiscal;
pub mod service;
pub mod transaction;
pub mod types;
pub mod validation;

#[cfg(test)]
mod service_props;
#[cfg(test)]
mod validation_props;

pub use balance::AccountBalance;
pub use entry::{EntryType, LedgerEntry};
pub use error::LedgerError;
pub use fiscal::{period_allows_posting, period_requires_elevated_privileges, validate_posting_permission};
pub use service::{AccountInfo, LedgerService};
pub use transaction::{Transaction, TransactionStatus};
pub use types::{
    CreateTransactionInput, EntryType as InputEntryType, FiscalPeriodStatus, LedgerEntryInput,
    ResolvedEntry, TransactionResult, TransactionTotals, TransactionType,
};
