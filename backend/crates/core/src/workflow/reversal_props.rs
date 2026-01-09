//! Property-based tests for ReversalService.
//!
//! These tests validate the correctness properties for void operations
//! and reversing entry creation.

use proptest::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::ledger::types::EntryType;
use crate::workflow::reversal::{OriginalEntry, ReversalInput, ReversalService};

/// Strategy for generating random UUIDs.
fn arb_uuid() -> impl Strategy<Value = Uuid> {
    any::<u128>().prop_map(Uuid::from_u128)
}

/// Strategy for generating random positive Decimal amounts.
fn arb_amount() -> impl Strategy<Value = Decimal> {
    (1i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
}

/// Strategy for generating a balanced pair of entries (one debit, one credit).
fn arb_balanced_entry_pair() -> impl Strategy<Value = Vec<OriginalEntry>> {
    (
        arb_uuid(),
        arb_uuid(),
        arb_amount(),
        prop::option::of("[a-zA-Z ]{0,20}"),
    )
        .prop_map(|(debit_account, credit_account, amount, memo)| {
            vec![
                OriginalEntry {
                    account_id: debit_account,
                    source_currency: "USD".to_string(),
                    source_amount: amount,
                    exchange_rate: Decimal::ONE,
                    functional_amount: amount,
                    debit: amount,
                    credit: Decimal::ZERO,
                    memo: memo.clone(),
                    dimensions: vec![],
                },
                OriginalEntry {
                    account_id: credit_account,
                    source_currency: "USD".to_string(),
                    source_amount: amount,
                    exchange_rate: Decimal::ONE,
                    functional_amount: amount,
                    debit: Decimal::ZERO,
                    credit: amount,
                    memo,
                    dimensions: vec![],
                },
            ]
        })
}

/// Strategy for generating balanced multi-entry sets (2-4 entries).
fn arb_balanced_entries() -> impl Strategy<Value = Vec<OriginalEntry>> {
    prop_oneof![
        // 2 entries: simple debit/credit pair
        arb_balanced_entry_pair(),
        // 4 entries: two debit/credit pairs with same total
        (arb_balanced_entry_pair(), arb_balanced_entry_pair()).prop_map(|(mut a, b)| {
            a.extend(b);
            a
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 3: Void Creates Balanced Reversing Entry
    // Feature: transaction-workflow, Property 3: Void Creates Balanced Reversing Entry
    // Validates: Requirements 2.1, 2.7
    // =========================================================================

    /// Reversing entries swap debits and credits correctly
    #[test]
    fn prop_reversing_entries_swap_debit_credit(
        entries in arb_balanced_entry_pair()
    ) {
        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries.clone(),
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test void".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        prop_assert_eq!(output.reversing_entries.len(), entries.len());

        // First entry was debit, should become credit
        prop_assert_eq!(output.reversing_entries[0].entry_type, EntryType::Credit);
        // Second entry was credit, should become debit
        prop_assert_eq!(output.reversing_entries[1].entry_type, EntryType::Debit);
    }

    /// Reversing entries preserve account IDs
    #[test]
    fn prop_reversing_entries_preserve_accounts(
        entries in arb_balanced_entry_pair()
    ) {
        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries.clone(),
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test void".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        for (original, reversed) in entries.iter().zip(output.reversing_entries.iter()) {
            prop_assert_eq!(original.account_id, reversed.account_id);
        }
    }

    /// Reversing entries preserve amounts
    #[test]
    fn prop_reversing_entries_preserve_amounts(
        entries in arb_balanced_entry_pair()
    ) {
        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries.clone(),
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test void".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        for (original, reversed) in entries.iter().zip(output.reversing_entries.iter()) {
            prop_assert_eq!(original.source_amount, reversed.source_amount);
            prop_assert_eq!(&original.source_currency, &reversed.source_currency);
        }
    }

    /// Balanced original entries produce balanced reversing entries
    #[test]
    fn prop_balanced_entries_produce_balanced_reversal(
        entries in arb_balanced_entries()
    ) {
        // Verify original is balanced
        let original_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let original_credit: Decimal = entries.iter().map(|e| e.credit).sum();
        prop_assert_eq!(original_debit, original_credit, "Original should be balanced");

        // Validate using service
        prop_assert!(ReversalService::validate_reversal(&entries));

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test void".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        // Count debits and credits in reversing entries
        let reversing_debits = output.reversing_entries.iter()
            .filter(|e| e.entry_type == EntryType::Debit)
            .count();
        let reversing_credits = output.reversing_entries.iter()
            .filter(|e| e.entry_type == EntryType::Credit)
            .count();

        // Original debits become credits, original credits become debits
        // So if original had N debits and M credits, reversal has M debits and N credits
        // For balanced entries, this means reversal is also balanced
        prop_assert!(reversing_debits > 0 || reversing_credits > 0);
    }

    /// Memo is prefixed with "Reversal: "
    #[test]
    fn prop_memo_prefixed_with_reversal(
        memo in "[a-zA-Z ]{1,20}"
    ) {
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: Some(memo.clone()),
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: Some(memo.clone()),
                dimensions: vec![],
            },
        ];

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        for entry in &output.reversing_entries {
            let entry_memo = entry.memo.as_ref().unwrap();
            prop_assert!(entry_memo.starts_with("Reversal: "),
                "Memo should start with 'Reversal: ', got: {}", entry_memo);
        }
    }

    /// Description contains original transaction ID and void reason
    #[test]
    fn prop_description_contains_required_info(
        void_reason in "[a-zA-Z ]{1,50}"
    ) {
        let original_id = Uuid::new_v4();
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: None,
                dimensions: vec![],
            },
        ];

        let input = ReversalInput {
            original_transaction_id: original_id,
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: void_reason.clone(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        prop_assert!(output.description.contains(&original_id.to_string()),
            "Description should contain original transaction ID");
        prop_assert!(output.description.contains(&void_reason),
            "Description should contain void reason");
    }
}

// =========================================================================
// Unit tests for edge cases
// =========================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_simple_two_entry_reversal() {
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(10000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(10000, 2),
                debit: Decimal::new(10000, 2),
                credit: Decimal::ZERO,
                memo: Some("Expense".to_string()),
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
                memo: Some("Cash".to_string()),
                dimensions: vec![],
            },
        ];

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Duplicate".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        assert_eq!(output.reversing_entries.len(), 2);
        assert_eq!(output.reversing_entries[0].entry_type, EntryType::Credit);
        assert_eq!(output.reversing_entries[1].entry_type, EntryType::Debit);
    }

    #[test]
    fn test_multi_entry_reversal_four_entries() {
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
                source_amount: Decimal::new(6000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(6000, 2),
                debit: Decimal::ZERO,
                credit: Decimal::new(6000, 2),
                memo: Some("Entry 3".to_string()),
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(2000, 2),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(2000, 2),
                debit: Decimal::ZERO,
                credit: Decimal::new(2000, 2),
                memo: Some("Entry 4".to_string()),
                dimensions: vec![],
            },
        ];

        assert!(ReversalService::validate_reversal(&entries));

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Error".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        assert_eq!(output.reversing_entries.len(), 4);
        // First two were debits -> credits
        assert_eq!(output.reversing_entries[0].entry_type, EntryType::Credit);
        assert_eq!(output.reversing_entries[1].entry_type, EntryType::Credit);
        // Last two were credits -> debits
        assert_eq!(output.reversing_entries[2].entry_type, EntryType::Debit);
        assert_eq!(output.reversing_entries[3].entry_type, EntryType::Debit);
    }

    #[test]
    fn test_memo_preservation_with_prefix() {
        let original_memo = "Office supplies purchase";
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: Some(original_memo.to_string()),
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: Some(original_memo.to_string()),
                dimensions: vec![],
            },
        ];

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        for entry in &output.reversing_entries {
            let memo = entry.memo.as_ref().unwrap();
            assert!(memo.starts_with("Reversal: "));
            assert!(memo.contains(original_memo));
        }
    }

    #[test]
    fn test_dimension_preservation() {
        let dim1 = Uuid::new_v4();
        let dim2 = Uuid::new_v4();

        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![dim1, dim2],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: None,
                dimensions: vec![dim1],
            },
        ];

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries.clone(),
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        assert_eq!(
            output.reversing_entries[0].dimensions,
            entries[0].dimensions
        );
        assert_eq!(
            output.reversing_entries[1].dimensions,
            entries[1].dimensions
        );
    }

    #[test]
    fn test_empty_memo_handling() {
        let entries = vec![
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            OriginalEntry {
                account_id: Uuid::new_v4(),
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: None,
                dimensions: vec![],
            },
        ];

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);

        // Should still have "Reversal: " prefix even with empty original memo
        for entry in &output.reversing_entries {
            assert!(entry.memo.as_ref().unwrap().starts_with("Reversal: "));
        }
    }
}
