//! Double-entry bookkeeping logic.
//!
//! This module implements the core ledger functionality:
//! - Ledger entries (debits and credits)
//! - Transaction aggregates
//! - Balance calculations
//! - Business rule validation

pub mod balance;
pub mod entry;
pub mod transaction;
pub mod validation;

pub use balance::AccountBalance;
pub use entry::{EntryType, LedgerEntry};
pub use transaction::{Transaction, TransactionStatus};
