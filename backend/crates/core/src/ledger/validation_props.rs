//! Property-based tests for ledger entry validation rules.
//!
//! Feature: ledger-core, Property 13: Entry Validation Rules
//! Validates: Requirements 5.1, 5.3, 5.4

use proptest::prelude::*;
use rust_decimal::Decimal;
use zeltra_shared::types::{AccountId, LedgerEntryId, TransactionId};

use super::entry::{EntryType, LedgerEntry};
use super::validation::{validate_entries, LedgerValidationError};

/// Strategy to generate a valid positive amount (> 0).
fn positive_amount() -> impl Strategy<Value = Decimal> {
    // Generate amounts from 0.01 to 1,000,000.00
    (1i64..100_000_000i64).prop_map(|cents| Decimal::new(cents, 2))
}

/// Strategy to generate a negative amount.
fn negative_amount() -> impl Strategy<Value = Decimal> {
    // Generate negative amounts from -0.01 to -1,000,000.00
    (1i64..100_000_000i64).prop_map(|cents| Decimal::new(-cents, 2))
}

/// Strategy to generate an entry type.
fn entry_type_strategy() -> impl Strategy<Value = EntryType> {
    prop_oneof![Just(EntryType::Debit), Just(EntryType::Credit)]
}

/// Helper to create a ledger entry for testing.
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 13: Entry Validation Rules
    // Validates: Requirements 5.1, 5.3, 5.4
    // =========================================================================

    /// Property 13.1: Zero amount entries are rejected.
    ///
    /// *For any* ledger entry with amount = 0, validation SHALL reject it.
    /// Validates: Requirement 5.3
    #[test]
    fn prop_zero_amount_rejected(
        entry_type in entry_type_strategy(),
        other_amount in positive_amount(),
    ) {
        // Create a balanced transaction with one zero-amount entry
        let entries = vec![
            make_entry(entry_type, Decimal::ZERO),
            make_entry(
                if entry_type == EntryType::Debit { EntryType::Credit } else { EntryType::Debit },
                other_amount,
            ),
        ];

        let result = validate_entries(&entries);
        prop_assert!(
            matches!(result, Err(LedgerValidationError::InvalidAmount)),
            "Zero amount should be rejected, got: {:?}",
            result
        );
    }

    /// Property 13.2: Negative amount entries are rejected.
    ///
    /// *For any* ledger entry with amount < 0, validation SHALL reject it.
    /// Validates: Requirement 5.4
    #[test]
    fn prop_negative_amount_rejected(
        entry_type in entry_type_strategy(),
        neg_amount in negative_amount(),
        other_amount in positive_amount(),
    ) {
        // Create entries with one negative amount
        let entries = vec![
            make_entry(entry_type, neg_amount),
            make_entry(
                if entry_type == EntryType::Debit { EntryType::Credit } else { EntryType::Debit },
                other_amount,
            ),
        ];

        let result = validate_entries(&entries);
        prop_assert!(
            matches!(result, Err(LedgerValidationError::InvalidAmount)),
            "Negative amount should be rejected, got: {:?}",
            result
        );
    }

    /// Property 13.3: Single entry transactions are rejected.
    ///
    /// *For any* transaction with only 1 entry, validation SHALL reject it
    /// because double-entry bookkeeping requires at least 2 entries.
    /// Validates: Requirement 5.1
    #[test]
    fn prop_single_entry_rejected(
        entry_type in entry_type_strategy(),
        amount in positive_amount(),
    ) {
        let entries = vec![make_entry(entry_type, amount)];

        let result = validate_entries(&entries);
        // Single entry will fail with SingleSided error (only debits or only credits)
        prop_assert!(
            matches!(result, Err(LedgerValidationError::SingleSided)),
            "Single entry should be rejected as single-sided, got: {:?}",
            result
        );
    }

    /// Property 13.4: Empty transactions are rejected.
    ///
    /// *For any* transaction with 0 entries, validation SHALL reject it.
    /// Validates: Requirement 5.1
    #[test]
    fn prop_empty_entries_rejected(_dummy in 0..1i32) {
        let entries: Vec<LedgerEntry> = vec![];

        let result = validate_entries(&entries);
        prop_assert!(
            matches!(result, Err(LedgerValidationError::NoEntries)),
            "Empty entries should be rejected, got: {:?}",
            result
        );
    }

    /// Property 13.5: Balanced transactions with valid amounts are accepted.
    ///
    /// *For any* transaction with at least 2 entries where:
    /// - All amounts are positive
    /// - Total debits equal total credits
    /// Validation SHALL accept it.
    /// Validates: Requirements 5.1, 5.3, 5.4 (positive case)
    #[test]
    fn prop_valid_balanced_transaction_accepted(
        amount in positive_amount(),
    ) {
        // Simple balanced transaction: one debit, one credit, same amount
        let entries = vec![
            make_entry(EntryType::Debit, amount),
            make_entry(EntryType::Credit, amount),
        ];

        let result = validate_entries(&entries);
        prop_assert!(
            result.is_ok(),
            "Valid balanced transaction should be accepted, got: {:?}",
            result
        );
    }

    /// Property 13.6: Multi-entry balanced transactions are accepted.
    ///
    /// *For any* transaction with multiple debit and credit entries where
    /// total debits equal total credits, validation SHALL accept it.
    /// Validates: Requirements 5.1, 5.3, 5.4 (positive case with multiple entries)
    #[test]
    fn prop_multi_entry_balanced_accepted(
        amount1 in positive_amount(),
        amount2 in positive_amount(),
    ) {
        // Transaction with 2 debits and 2 credits that balance
        let total = amount1 + amount2;
        let entries = vec![
            make_entry(EntryType::Debit, amount1),
            make_entry(EntryType::Debit, amount2),
            make_entry(EntryType::Credit, total),
        ];

        let result = validate_entries(&entries);
        prop_assert!(
            result.is_ok(),
            "Multi-entry balanced transaction should be accepted, got: {:?}",
            result
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Specific example: exactly zero amount.
    #[test]
    fn test_zero_amount_example() {
        let entries = vec![
            make_entry(EntryType::Debit, Decimal::ZERO),
            make_entry(EntryType::Credit, Decimal::new(100, 2)),
        ];
        assert!(matches!(
            validate_entries(&entries),
            Err(LedgerValidationError::InvalidAmount)
        ));
    }

    /// Specific example: negative amount.
    #[test]
    fn test_negative_amount_example() {
        let entries = vec![
            make_entry(EntryType::Debit, Decimal::new(-100, 2)),
            make_entry(EntryType::Credit, Decimal::new(100, 2)),
        ];
        assert!(matches!(
            validate_entries(&entries),
            Err(LedgerValidationError::InvalidAmount)
        ));
    }

    /// Specific example: minimum valid transaction (2 entries).
    #[test]
    fn test_minimum_valid_transaction() {
        let entries = vec![
            make_entry(EntryType::Debit, Decimal::new(100, 2)),
            make_entry(EntryType::Credit, Decimal::new(100, 2)),
        ];
        assert!(validate_entries(&entries).is_ok());
    }
}
