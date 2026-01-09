//! Ledger domain types for transaction creation and validation.
//!
//! This module defines the core types used for creating and validating
//! financial transactions in the double-entry bookkeeping system.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entry type: either Debit or Credit.
///
/// In double-entry bookkeeping:
/// - Debits increase asset/expense accounts, decrease liability/equity/revenue accounts
/// - Credits decrease asset/expense accounts, increase liability/equity/revenue accounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// Debit entry.
    Debit,
    /// Credit entry.
    Credit,
}

/// Transaction type classification.
///
/// Categorizes transactions for reporting and workflow purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// General journal entry.
    Journal,
    /// Expense transaction.
    Expense,
    /// Sales invoice.
    Invoice,
    /// Vendor bill.
    Bill,
    /// Payment (incoming or outgoing).
    Payment,
    /// Transfer between accounts.
    Transfer,
    /// Adjustment entry.
    Adjustment,
    /// Opening balance entry.
    OpeningBalance,
    /// Reversal of a previous transaction.
    Reversal,
}

/// Transaction status in the approval workflow.
///
/// Transactions progress through these states from creation to posting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// Transaction is being drafted and can be modified.
    Draft,
    /// Transaction has been submitted for approval.
    Pending,
    /// Transaction has been approved and is ready for posting.
    Approved,
    /// Transaction has been posted to the ledger (immutable).
    Posted,
    /// Transaction has been voided (immutable).
    Voided,
}

impl TransactionStatus {
    /// Returns true if the transaction can be modified.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        matches!(self, Self::Draft)
    }

    /// Returns true if the transaction is immutable.
    #[must_use]
    pub fn is_immutable(&self) -> bool {
        matches!(self, Self::Posted | Self::Voided)
    }
}

/// Fiscal period status controlling posting permissions.
///
/// Determines who can post transactions to a fiscal period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FiscalPeriodStatus {
    /// Period is open - all authorized users can post.
    Open,
    /// Period is soft-closed - only accountant/admin/owner can post.
    SoftClose,
    /// Period is closed - no posting allowed.
    Closed,
}

impl FiscalPeriodStatus {
    /// Returns true if the period allows any posting.
    #[must_use]
    pub fn allows_posting(&self) -> bool {
        !matches!(self, Self::Closed)
    }

    /// Returns true if posting requires elevated privileges.
    #[must_use]
    pub fn requires_elevated_privileges(&self) -> bool {
        matches!(self, Self::SoftClose)
    }
}

/// Input for a single ledger entry in a transaction.
///
/// This is the input format for creating new transactions.
/// The system will resolve exchange rates and calculate functional amounts.
#[derive(Debug, Clone)]
pub struct LedgerEntryInput {
    /// The account to post to.
    pub account_id: Uuid,
    /// The source currency code (ISO 4217).
    pub source_currency: String,
    /// The amount in source currency (must be positive).
    pub source_amount: Decimal,
    /// Whether this is a debit or credit entry.
    pub entry_type: EntryType,
    /// Optional memo/description for this entry.
    pub memo: Option<String>,
    /// Dimension value IDs to tag this entry with.
    pub dimensions: Vec<Uuid>,
}

/// Input for creating a new transaction.
///
/// Contains all the information needed to create a transaction
/// with multiple ledger entries.
#[derive(Debug, Clone)]
pub struct CreateTransactionInput {
    /// The organization this transaction belongs to.
    pub organization_id: Uuid,
    /// The type of transaction.
    pub transaction_type: TransactionType,
    /// The date of the transaction.
    pub transaction_date: NaiveDate,
    /// A description of the transaction.
    pub description: String,
    /// Optional reference number (e.g., invoice number).
    pub reference_number: Option<String>,
    /// Optional memo/notes.
    pub memo: Option<String>,
    /// The ledger entries (must have at least 2).
    pub entries: Vec<LedgerEntryInput>,
    /// The user creating the transaction.
    pub created_by: Uuid,
}

/// A resolved ledger entry with exchange rate applied.
///
/// After validation and exchange rate lookup, entries are resolved
/// to include the functional currency amounts.
#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    /// The account to post to.
    pub account_id: Uuid,
    /// The source currency code.
    pub source_currency: String,
    /// The amount in source currency.
    pub source_amount: Decimal,
    /// The exchange rate applied (source to functional).
    pub exchange_rate: Decimal,
    /// The functional (base) currency code.
    pub functional_currency: String,
    /// The amount in functional currency.
    pub functional_amount: Decimal,
    /// The debit amount in functional currency (0 if credit).
    pub debit: Decimal,
    /// The credit amount in functional currency (0 if debit).
    pub credit: Decimal,
    /// Optional memo/description.
    pub memo: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Result of transaction creation.
///
/// Contains the created transaction with resolved entries and totals.
#[derive(Debug)]
pub struct TransactionResult {
    /// The transaction ID.
    pub id: Uuid,
    /// The reference number (if any).
    pub reference_number: Option<String>,
    /// The transaction status.
    pub status: TransactionStatus,
    /// The resolved entries with exchange rates applied.
    pub entries: Vec<ResolvedEntry>,
    /// The transaction totals.
    pub totals: TransactionTotals,
}

/// Transaction totals for validation and display.
///
/// Contains the sum of debits and credits in functional currency.
#[derive(Debug, Clone)]
pub struct TransactionTotals {
    /// Total debit amount in functional currency.
    pub functional_debit: Decimal,
    /// Total credit amount in functional currency.
    pub functional_credit: Decimal,
    /// Whether the transaction is balanced (debits == credits).
    pub is_balanced: bool,
}

impl TransactionTotals {
    /// Creates new transaction totals from debit and credit sums.
    #[must_use]
    pub fn new(functional_debit: Decimal, functional_credit: Decimal) -> Self {
        Self {
            functional_debit,
            functional_credit,
            is_balanced: functional_debit == functional_credit,
        }
    }

    /// Returns the difference between debits and credits.
    #[must_use]
    pub fn difference(&self) -> Decimal {
        self.functional_debit - self.functional_credit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_status_editable() {
        assert!(TransactionStatus::Draft.is_editable());
        assert!(!TransactionStatus::Pending.is_editable());
        assert!(!TransactionStatus::Approved.is_editable());
        assert!(!TransactionStatus::Posted.is_editable());
        assert!(!TransactionStatus::Voided.is_editable());
    }

    #[test]
    fn test_transaction_status_immutable() {
        assert!(!TransactionStatus::Draft.is_immutable());
        assert!(!TransactionStatus::Pending.is_immutable());
        assert!(!TransactionStatus::Approved.is_immutable());
        assert!(TransactionStatus::Posted.is_immutable());
        assert!(TransactionStatus::Voided.is_immutable());
    }

    #[test]
    fn test_fiscal_period_status_posting() {
        assert!(FiscalPeriodStatus::Open.allows_posting());
        assert!(FiscalPeriodStatus::SoftClose.allows_posting());
        assert!(!FiscalPeriodStatus::Closed.allows_posting());
    }

    #[test]
    fn test_fiscal_period_status_privileges() {
        assert!(!FiscalPeriodStatus::Open.requires_elevated_privileges());
        assert!(FiscalPeriodStatus::SoftClose.requires_elevated_privileges());
        assert!(!FiscalPeriodStatus::Closed.requires_elevated_privileges());
    }

    #[test]
    fn test_transaction_totals_balanced() {
        let totals = TransactionTotals::new(Decimal::new(10000, 2), Decimal::new(10000, 2));
        assert!(totals.is_balanced);
        assert_eq!(totals.difference(), Decimal::ZERO);
    }

    #[test]
    fn test_transaction_totals_unbalanced() {
        let totals = TransactionTotals::new(Decimal::new(10000, 2), Decimal::new(5000, 2));
        assert!(!totals.is_balanced);
        assert_eq!(totals.difference(), Decimal::new(5000, 2));
    }
}
