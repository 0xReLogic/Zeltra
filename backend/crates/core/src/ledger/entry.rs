//! Ledger entry domain types.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use zeltra_shared::types::{AccountId, LedgerEntryId, TransactionId};

/// Type of ledger entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// Debit entry (increases assets/expenses, decreases liabilities/equity/revenue).
    Debit,
    /// Credit entry (decreases assets/expenses, increases liabilities/equity/revenue).
    Credit,
}

/// A single ledger entry in a transaction.
///
/// Each transaction consists of multiple entries that must balance (debits = credits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Unique identifier for this entry.
    pub id: LedgerEntryId,
    /// The transaction this entry belongs to.
    pub transaction_id: TransactionId,
    /// The account affected by this entry.
    pub account_id: AccountId,
    /// Whether this is a debit or credit.
    pub entry_type: EntryType,
    /// Amount in the transaction currency.
    pub amount: Decimal,
    /// Amount in the organization's base currency.
    pub base_amount: Decimal,
    /// Optional description for this line item.
    pub description: Option<String>,
}

impl LedgerEntry {
    /// Returns the signed amount (positive for debit, negative for credit).
    #[must_use]
    pub fn signed_amount(&self) -> Decimal {
        match self.entry_type {
            EntryType::Debit => self.amount,
            EntryType::Credit => -self.amount,
        }
    }

    /// Returns the signed base amount.
    #[must_use]
    pub fn signed_base_amount(&self) -> Decimal {
        match self.entry_type {
            EntryType::Debit => self.base_amount,
            EntryType::Credit => -self.base_amount,
        }
    }
}
