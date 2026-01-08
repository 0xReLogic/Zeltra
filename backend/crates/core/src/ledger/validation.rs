//! Business rule validation for ledger operations.

use rust_decimal::Decimal;
use thiserror::Error;

use super::entry::{EntryType, LedgerEntry};

/// Validation errors for ledger operations.
#[derive(Debug, Error)]
pub enum LedgerValidationError {
    /// Transaction entries do not balance.
    #[error("Transaction is unbalanced: debits ({debits}) != credits ({credits})")]
    Unbalanced {
        /// Total debit amount.
        debits: Decimal,
        /// Total credit amount.
        credits: Decimal,
    },

    /// Transaction has no entries.
    #[error("Transaction must have at least one entry")]
    NoEntries,

    /// Transaction has only one side (all debits or all credits).
    #[error("Transaction must have both debit and credit entries")]
    SingleSided,

    /// Entry amount is zero or negative.
    #[error("Entry amount must be positive")]
    InvalidAmount,
}

/// Validates that a set of ledger entries is balanced.
///
/// # Errors
///
/// Returns an error if the entries are not balanced or violate business rules.
pub fn validate_entries(entries: &[LedgerEntry]) -> Result<(), LedgerValidationError> {
    if entries.is_empty() {
        return Err(LedgerValidationError::NoEntries);
    }

    let mut total_debits = Decimal::ZERO;
    let mut total_credits = Decimal::ZERO;
    let mut has_debit = false;
    let mut has_credit = false;

    for entry in entries {
        if entry.amount <= Decimal::ZERO {
            return Err(LedgerValidationError::InvalidAmount);
        }

        match entry.entry_type {
            EntryType::Debit => {
                total_debits += entry.amount;
                has_debit = true;
            }
            EntryType::Credit => {
                total_credits += entry.amount;
                has_credit = true;
            }
        }
    }

    if !has_debit || !has_credit {
        return Err(LedgerValidationError::SingleSided);
    }

    if total_debits != total_credits {
        return Err(LedgerValidationError::Unbalanced {
            debits: total_debits,
            credits: total_credits,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeltra_shared::types::{AccountId, LedgerEntryId, TransactionId};

    fn make_entry(entry_type: EntryType, amount: Decimal) -> LedgerEntry {
        LedgerEntry {
            id: LedgerEntryId::new(),
            transaction_id: TransactionId::new(),
            account_id: AccountId::new(),
            entry_type,
            amount,
            base_amount: amount,
            description: None,
        }
    }

    #[test]
    fn test_balanced_entries() {
        let entries = vec![
            make_entry(EntryType::Debit, Decimal::new(10000, 2)),
            make_entry(EntryType::Credit, Decimal::new(10000, 2)),
        ];
        assert!(validate_entries(&entries).is_ok());
    }

    #[test]
    fn test_unbalanced_entries() {
        let entries = vec![
            make_entry(EntryType::Debit, Decimal::new(10000, 2)),
            make_entry(EntryType::Credit, Decimal::new(5000, 2)),
        ];
        assert!(matches!(
            validate_entries(&entries),
            Err(LedgerValidationError::Unbalanced { .. })
        ));
    }

    #[test]
    fn test_no_entries() {
        let entries: Vec<LedgerEntry> = vec![];
        assert!(matches!(
            validate_entries(&entries),
            Err(LedgerValidationError::NoEntries)
        ));
    }

    #[test]
    fn test_single_sided() {
        let entries = vec![
            make_entry(EntryType::Debit, Decimal::new(10000, 2)),
            make_entry(EntryType::Debit, Decimal::new(5000, 2)),
        ];
        assert!(matches!(
            validate_entries(&entries),
            Err(LedgerValidationError::SingleSided)
        ));
    }
}
