//! Reversal service for voiding posted transactions.
//!
//! This module implements the creation of reversing entries
//! when voiding posted transactions, following accounting best practices.

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::ledger::types::{EntryType, LedgerEntryInput};

/// Input for creating a reversing transaction.
#[derive(Debug, Clone)]
pub struct ReversalInput {
    /// The ID of the original transaction being voided.
    pub original_transaction_id: Uuid,
    /// The original ledger entries to reverse.
    pub original_entries: Vec<OriginalEntry>,
    /// The fiscal period for the reversing transaction.
    pub fiscal_period_id: Uuid,
    /// The user voiding the transaction.
    pub voided_by: Uuid,
    /// The reason for voiding.
    pub void_reason: String,
}

/// An original ledger entry to be reversed.
#[derive(Debug, Clone)]
pub struct OriginalEntry {
    /// The account ID.
    pub account_id: Uuid,
    /// The source currency code.
    pub source_currency: String,
    /// The amount in source currency.
    pub source_amount: Decimal,
    /// The exchange rate applied.
    pub exchange_rate: Decimal,
    /// The amount in functional currency.
    pub functional_amount: Decimal,
    /// The debit amount (0 if credit).
    pub debit: Decimal,
    /// The credit amount (0 if debit).
    pub credit: Decimal,
    /// Optional memo.
    pub memo: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Output from creating a reversing transaction.
#[derive(Debug)]
pub struct ReversalOutput {
    /// The ID for the new reversing transaction.
    pub reversing_transaction_id: Uuid,
    /// The reversing ledger entries.
    pub reversing_entries: Vec<LedgerEntryInput>,
    /// Description for the reversing transaction.
    pub description: String,
}

/// Stateless service for creating reversing entries.
pub struct ReversalService;

impl ReversalService {
    /// Create reversing entries by swapping debits and credits.
    ///
    /// For each original entry:
    /// - Debits become credits
    /// - Credits become debits
    /// - All other fields are preserved
    /// - Memo is prefixed with "Reversal: "
    ///
    /// # Arguments
    /// * `input` - The reversal input containing original entries
    ///
    /// # Returns
    /// A `ReversalOutput` with the reversing transaction ID and entries.
    #[must_use]
    pub fn create_reversing_entries(input: &ReversalInput) -> ReversalOutput {
        let reversing_entries: Vec<LedgerEntryInput> = input
            .original_entries
            .iter()
            .map(|entry| {
                // Swap debit and credit
                let entry_type = if entry.debit > Decimal::ZERO {
                    EntryType::Credit
                } else {
                    EntryType::Debit
                };

                LedgerEntryInput {
                    account_id: entry.account_id,
                    source_currency: entry.source_currency.clone(),
                    source_amount: entry.source_amount,
                    entry_type,
                    memo: Some(format!(
                        "Reversal: {}",
                        entry.memo.clone().unwrap_or_default()
                    )),
                    dimensions: entry.dimensions.clone(),
                }
            })
            .collect();

        ReversalOutput {
            reversing_transaction_id: Uuid::new_v4(),
            reversing_entries,
            description: format!(
                "Reversal of transaction {}. Reason: {}",
                input.original_transaction_id, input.void_reason
            ),
        }
    }

    /// Validate that original entries are balanced.
    ///
    /// A valid transaction must have total debits equal to total credits.
    /// This should always be true for posted transactions.
    ///
    /// # Arguments
    /// * `original_entries` - The entries to validate
    ///
    /// # Returns
    /// `true` if balanced, `false` otherwise.
    #[must_use]
    pub fn validate_reversal(original_entries: &[OriginalEntry]) -> bool {
        let total_debit: Decimal = original_entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = original_entries.iter().map(|e| e.credit).sum();

        total_debit == total_credit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_balanced_entries() -> Vec<OriginalEntry> {
        vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(10000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(10000, 2),
                debit: Decimal::new(10000, 2),
                credit: Decimal::ZERO,
                memo: Some("Office supplies".to_string()),
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(10000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(10000, 2),
                debit: Decimal::ZERO,
                credit: Decimal::new(10000, 2),
                memo: Some("Cash payment".to_string()),
                dimensions: vec![],
            },
        ]
    }

    #[test]
    fn test_create_reversing_entries() {
        let entries = create_balanced_entries();
        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Duplicate entry".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        assert_eq!(output.reversing_entries.len(), 2);

        // First entry was debit, should become credit
        assert_eq!(output.reversing_entries[0].entry_type, EntryType::Credit);
        assert!(
            output.reversing_entries[0]
                .memo
                .as_ref()
                .unwrap()
                .starts_with("Reversal: ")
        );

        // Second entry was credit, should become debit
        assert_eq!(output.reversing_entries[1].entry_type, EntryType::Debit);
    }

    #[test]
    fn test_create_reversing_entries_preserves_amounts() {
        let entries = create_balanced_entries();
        let original_amount = entries[0].source_amount;
        let original_account = entries[0].account_id;

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Error".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        assert_eq!(output.reversing_entries[0].source_amount, original_amount);
        assert_eq!(output.reversing_entries[0].account_id, original_account);
    }

    #[test]
    fn test_create_reversing_entries_description() {
        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: create_balanced_entries(),
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Duplicate entry".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        assert!(output.description.contains("Reversal of transaction"));
        assert!(output.description.contains("Duplicate entry"));
    }

    #[test]
    fn test_validate_reversal_balanced() {
        let entries = create_balanced_entries();
        assert!(ReversalService::validate_reversal(&entries));
    }

    #[test]
    fn test_validate_reversal_unbalanced() {
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(10000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(10000, 2),
                debit: Decimal::new(10000, 2),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(5000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(5000, 2),
                debit: Decimal::ZERO,
                credit: Decimal::new(5000, 2),
                memo: None,
                dimensions: vec![],
            },
        ];

        assert!(!ReversalService::validate_reversal(&entries));
    }

    #[test]
    fn test_validate_reversal_empty() {
        let entries: Vec<OriginalEntry> = vec![];
        // Empty entries are technically balanced (0 = 0)
        assert!(ReversalService::validate_reversal(&entries));
    }

    #[test]
    fn test_multi_entry_reversal() {
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(5000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(5000, 2),
                debit: Decimal::new(5000, 2),
                credit: Decimal::ZERO,
                memo: Some("Entry 1".to_string()),
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(3000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(3000, 2),
                debit: Decimal::new(3000, 2),
                credit: Decimal::ZERO,
                memo: Some("Entry 2".to_string()),
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(8000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(8000, 2),
                debit: Decimal::ZERO,
                credit: Decimal::new(8000, 2),
                memo: Some("Entry 3".to_string()),
                dimensions: vec![],
            },
        ];

        assert!(ReversalService::validate_reversal(&entries));

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);
        assert_eq!(output.reversing_entries.len(), 3);

        // First two were debits, should become credits
        assert_eq!(output.reversing_entries[0].entry_type, EntryType::Credit);
        assert_eq!(output.reversing_entries[1].entry_type, EntryType::Credit);
        // Third was credit, should become debit
        assert_eq!(output.reversing_entries[2].entry_type, EntryType::Debit);
    }
}
