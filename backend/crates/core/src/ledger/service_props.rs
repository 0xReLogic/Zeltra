//! Property-based tests for LedgerService.
//!
//! Feature: ledger-core
//! - Property 1: Transaction Balance Integrity
//! - Property 5: Currency Conversion Correctness
//! - Property 16: Multi-Currency Entry Completeness

use chrono::NaiveDate;
use proptest::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::error::LedgerError;
use super::service::{AccountInfo, LedgerService};
use super::types::{CreateTransactionInput, EntryType, LedgerEntryInput, TransactionType};
use crate::currency::CurrencyService;

/// Strategy to generate positive decimal amounts (0.01 to 10,000.00).
fn positive_amount() -> impl Strategy<Value = Decimal> {
    (1i64..1_000_000i64).prop_map(|cents| Decimal::new(cents, 2))
}

/// Strategy to generate positive exchange rates (0.01 to 100.00).
fn positive_rate() -> impl Strategy<Value = Decimal> {
    (1i64..10_000i64).prop_map(|v| Decimal::new(v, 2))
}

/// Strategy to generate entry type.
fn entry_type_strategy() -> impl Strategy<Value = EntryType> {
    prop_oneof![Just(EntryType::Debit), Just(EntryType::Credit)]
}

/// Strategy to generate currency codes.
fn currency_code() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("USD".to_string()),
        Just("EUR".to_string()),
        Just("GBP".to_string()),
        Just("JPY".to_string()),
    ]
}

/// Helper to create a ledger entry input.
fn make_entry(entry_type: EntryType, amount: Decimal, currency: &str) -> LedgerEntryInput {
    LedgerEntryInput {
        account_id: Uuid::new_v4(),
        source_currency: currency.to_string(),
        source_amount: amount,
        entry_type,
        memo: None,
        dimensions: vec![],
    }
}

/// Helper to create transaction input.
fn make_input(entries: Vec<LedgerEntryInput>) -> CreateTransactionInput {
    CreateTransactionInput {
        organization_id: Uuid::new_v4(),
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        description: "Test transaction".to_string(),
        reference_number: None,
        memo: None,
        entries,
        created_by: Uuid::new_v4(),
    }
}

/// Mock account validator that always succeeds.
fn ok_account_validator(id: Uuid) -> Result<AccountInfo, LedgerError> {
    Ok(AccountInfo {
        id,
        is_active: true,
        allow_direct_posting: true,
        currency: "USD".to_string(),
    })
}

/// Mock dimension validator that always succeeds.
fn ok_dimension_validator(_dims: &[Uuid]) -> Result<(), LedgerError> {
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 1: Transaction Balance Integrity
    // Validates: Requirements 5.2, 6.6
    // =========================================================================

    /// Property 1.1: Valid balanced transactions are accepted.
    ///
    /// *For any* transaction with equal debit and credit amounts in functional
    /// currency, validation SHALL succeed and totals SHALL be balanced.
    #[test]
    fn prop_balanced_transaction_accepted(
        amount in positive_amount(),
    ) {
        let entries = vec![
            make_entry(EntryType::Debit, amount, "USD"),
            make_entry(EntryType::Credit, amount, "USD"),
        ];
        let input = make_input(entries);

        let rate_lookup = |_: &str, _: &str, _: NaiveDate| Some(Decimal::ONE);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok(), "Balanced transaction should be accepted");
        let (_, totals) = result.unwrap();
        prop_assert!(totals.is_balanced, "Totals should be balanced");
        prop_assert_eq!(totals.functional_debit, totals.functional_credit);
    }

    /// Property 1.2: Unbalanced transactions are rejected.
    ///
    /// *For any* transaction where debit != credit in functional currency,
    /// validation SHALL fail with UnbalancedTransaction error.
    #[test]
    fn prop_unbalanced_transaction_rejected(
        debit_amount in positive_amount(),
        credit_amount in positive_amount(),
    ) {
        // Skip if amounts happen to be equal
        prop_assume!(debit_amount != credit_amount);

        let entries = vec![
            make_entry(EntryType::Debit, debit_amount, "USD"),
            make_entry(EntryType::Credit, credit_amount, "USD"),
        ];
        let input = make_input(entries);

        let rate_lookup = |_: &str, _: &str, _: NaiveDate| Some(Decimal::ONE);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(
            matches!(result, Err(LedgerError::UnbalancedTransaction { .. })),
            "Unbalanced transaction should be rejected"
        );
    }

    /// Property 1.3: Multi-entry balanced transactions are accepted.
    ///
    /// *For any* transaction with multiple debits and credits that sum to equal
    /// amounts, validation SHALL succeed.
    #[test]
    fn prop_multi_entry_balanced_accepted(
        amount1 in positive_amount(),
        amount2 in positive_amount(),
    ) {
        let total = amount1 + amount2;
        let entries = vec![
            make_entry(EntryType::Debit, amount1, "USD"),
            make_entry(EntryType::Debit, amount2, "USD"),
            make_entry(EntryType::Credit, total, "USD"),
        ];
        let input = make_input(entries);

        let rate_lookup = |_: &str, _: &str, _: NaiveDate| Some(Decimal::ONE);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok(), "Multi-entry balanced transaction should be accepted");
        let (_, totals) = result.unwrap();
        prop_assert!(totals.is_balanced);
    }

    // =========================================================================
    // Property 5: Currency Conversion Correctness
    // Validates: Requirements 6.2, 6.3, 6.4
    // =========================================================================

    /// Property 5.1: Functional amount equals source * rate (rounded).
    ///
    /// *For any* entry with source_currency != functional_currency,
    /// functional_amount SHALL equal source_amount * exchange_rate rounded
    /// to 4 decimal places using Banker's Rounding.
    #[test]
    fn prop_currency_conversion_correct(
        source_amount in positive_amount(),
        rate in positive_rate(),
    ) {
        let entries = vec![
            make_entry(EntryType::Debit, source_amount, "EUR"),
            make_entry(EntryType::Credit, source_amount, "EUR"),
        ];
        let input = make_input(entries);

        let rate_lookup = move |_: &str, _: &str, _: NaiveDate| Some(rate);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok());
        let (resolved, _) = result.unwrap();

        for entry in &resolved {
            let expected = CurrencyService::convert(source_amount, rate);
            prop_assert_eq!(
                entry.functional_amount, expected,
                "functional_amount should equal source * rate (rounded)"
            );
            prop_assert_eq!(entry.exchange_rate, rate);
        }
    }

    /// Property 5.2: Same currency has rate = 1 and functional = source.
    ///
    /// *For any* entry where source_currency equals functional_currency,
    /// exchange_rate SHALL be 1 and functional_amount SHALL equal source_amount.
    #[test]
    fn prop_same_currency_rate_is_one(
        amount in positive_amount(),
    ) {
        let entries = vec![
            make_entry(EntryType::Debit, amount, "USD"),
            make_entry(EntryType::Credit, amount, "USD"),
        ];
        let input = make_input(entries);

        let rate_lookup = |_: &str, _: &str, _: NaiveDate| Some(Decimal::ONE);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok());
        let (resolved, _) = result.unwrap();

        for entry in &resolved {
            prop_assert_eq!(entry.exchange_rate, Decimal::ONE);
            // functional_amount should equal source_amount (rounded to 4 decimals)
            let expected = CurrencyService::round(amount, 4);
            prop_assert_eq!(entry.functional_amount, expected);
        }
    }

    /// Property 5.3: Exchange rate is always positive.
    ///
    /// *For any* resolved entry, exchange_rate SHALL be positive.
    #[test]
    fn prop_exchange_rate_positive(
        amount in positive_amount(),
        rate in positive_rate(),
    ) {
        let entries = vec![
            make_entry(EntryType::Debit, amount, "EUR"),
            make_entry(EntryType::Credit, amount, "EUR"),
        ];
        let input = make_input(entries);

        let rate_lookup = move |_: &str, _: &str, _: NaiveDate| Some(rate);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok());
        let (resolved, _) = result.unwrap();

        for entry in &resolved {
            prop_assert!(entry.exchange_rate > Decimal::ZERO);
        }
    }

    // =========================================================================
    // Property 16: Multi-Currency Entry Completeness
    // Validates: Requirements 6.5
    // =========================================================================

    /// Property 16.1: All three currency fields are populated.
    ///
    /// *For any* resolved entry, source_amount, exchange_rate, and
    /// functional_amount SHALL all be populated (non-zero for valid entries).
    #[test]
    fn prop_all_currency_fields_populated(
        amount in positive_amount(),
        rate in positive_rate(),
        currency in currency_code(),
    ) {
        let entries = vec![
            make_entry(EntryType::Debit, amount, &currency),
            make_entry(EntryType::Credit, amount, &currency),
        ];
        let input = make_input(entries);

        let rate_lookup = move |_: &str, _: &str, _: NaiveDate| Some(rate);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok());
        let (resolved, _) = result.unwrap();

        for entry in &resolved {
            // source_amount should be populated (positive)
            prop_assert!(entry.source_amount > Decimal::ZERO, "source_amount should be positive");

            // exchange_rate should be populated (positive)
            prop_assert!(entry.exchange_rate > Decimal::ZERO, "exchange_rate should be positive");

            // functional_amount should be populated (positive)
            prop_assert!(entry.functional_amount > Decimal::ZERO, "functional_amount should be positive");

            // functional_currency should be set
            prop_assert!(!entry.functional_currency.is_empty(), "functional_currency should be set");

            // source_currency should be set
            prop_assert!(!entry.source_currency.is_empty(), "source_currency should be set");
        }
    }

    /// Property 16.2: Debit/credit amounts match functional amount.
    ///
    /// *For any* resolved entry, either debit or credit SHALL equal
    /// functional_amount (the other being zero).
    #[test]
    fn prop_debit_credit_matches_functional(
        amount in positive_amount(),
        entry_type in entry_type_strategy(),
    ) {
        let entries = vec![
            make_entry(entry_type, amount, "USD"),
            make_entry(
                if entry_type == EntryType::Debit { EntryType::Credit } else { EntryType::Debit },
                amount,
                "USD"
            ),
        ];
        let input = make_input(entries);

        let rate_lookup = |_: &str, _: &str, _: NaiveDate| Some(Decimal::ONE);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        prop_assert!(result.is_ok());
        let (resolved, _) = result.unwrap();

        for entry in &resolved {
            // Either debit or credit should equal functional_amount
            let has_debit = entry.debit == entry.functional_amount && entry.credit == Decimal::ZERO;
            let has_credit = entry.credit == entry.functional_amount && entry.debit == Decimal::ZERO;
            prop_assert!(
                has_debit || has_credit,
                "Either debit or credit should equal functional_amount"
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Specific example: balanced transaction with exact amounts.
    #[test]
    fn test_balanced_100_100() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100), "USD"),
            make_entry(EntryType::Credit, dec!(100), "USD"),
        ];
        let input = make_input(entries);

        let rate_lookup = |_: &str, _: &str, _: NaiveDate| Some(Decimal::ONE);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(result.is_ok());
        let (_, totals) = result.unwrap();
        assert!(totals.is_balanced);
        assert_eq!(totals.functional_debit, dec!(100));
        assert_eq!(totals.functional_credit, dec!(100));
    }

    /// Specific example: multi-currency balanced after conversion.
    #[test]
    fn test_multi_currency_balanced() {
        // EUR 100 * 1.5 = USD 150
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100), "EUR"),
            make_entry(EntryType::Credit, dec!(150), "USD"),
        ];
        let input = make_input(entries);

        let rate_lookup = |from: &str, _: &str, _: NaiveDate| {
            if from == "EUR" {
                Some(dec!(1.5))
            } else {
                Some(Decimal::ONE)
            }
        };

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(result.is_ok());
        let (resolved, totals) = result.unwrap();
        assert!(totals.is_balanced);
        assert_eq!(resolved[0].functional_amount, dec!(150));
        assert_eq!(resolved[0].exchange_rate, dec!(1.5));
    }
}
