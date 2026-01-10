//! Property-based tests for budget repository.
//!
//! Tests for Property 5 (Actual Amount Calculation) and Property 8 (Budget Line Uniqueness).

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::entities::ledger_entries;
use crate::entities::sea_orm_active_enums::AccountType;
use crate::repositories::budget::{calculate_actual_by_account_type, is_debit_normal_account};

// ============================================================================
// Property 5: Actual Amount Calculation by Account Type
// **Validates: Requirements 4.2, 4.3**
// ============================================================================

/// Strategy for generating positive decimal amounts
fn amount_strategy() -> impl Strategy<Value = Decimal> {
    (0i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
}

/// Strategy for generating account types
fn account_type_strategy() -> impl Strategy<Value = AccountType> {
    prop_oneof![
        Just(AccountType::Asset),
        Just(AccountType::Expense),
        Just(AccountType::Liability),
        Just(AccountType::Equity),
        Just(AccountType::Revenue),
    ]
}

/// Creates a mock ledger entry with specified debit and credit amounts.
fn mock_entry(debit: Decimal, credit: Decimal) -> ledger_entries::Model {
    use chrono::Utc;
    use uuid::Uuid;

    ledger_entries::Model {
        id: Uuid::new_v4(),
        transaction_id: Uuid::new_v4(),
        account_id: Uuid::new_v4(),
        source_currency: "USD".to_string(),
        source_amount: debit + credit,
        exchange_rate: dec!(1),
        functional_currency: "USD".to_string(),
        functional_amount: debit + credit,
        debit,
        credit,
        memo: None,
        event_at: Utc::now().into(),
        created_at: Utc::now().into(),
        account_version: 1,
        account_previous_balance: dec!(0),
        account_current_balance: debit - credit,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 5.1: Expense accounts use debit-normal calculation**
    ///
    /// *For any* expense account, actual = sum(debit) - sum(credit).
    ///
    /// **Validates: Requirements 4.2**
    #[test]
    fn prop_expense_actual_debit_minus_credit(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        let entries = vec![mock_entry(debit, credit)];
        let actual = calculate_actual_by_account_type(&AccountType::Expense, &entries);

        prop_assert_eq!(actual, debit - credit, "Expense actual should be debit - credit");
    }

    /// **Property 5.2: Asset accounts use debit-normal calculation**
    ///
    /// *For any* asset account, actual = sum(debit) - sum(credit).
    ///
    /// **Validates: Requirements 4.2**
    #[test]
    fn prop_asset_actual_debit_minus_credit(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        let entries = vec![mock_entry(debit, credit)];
        let actual = calculate_actual_by_account_type(&AccountType::Asset, &entries);

        prop_assert_eq!(actual, debit - credit, "Asset actual should be debit - credit");
    }

    /// **Property 5.3: Revenue accounts use credit-normal calculation**
    ///
    /// *For any* revenue account, actual = sum(credit) - sum(debit).
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn prop_revenue_actual_credit_minus_debit(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        let entries = vec![mock_entry(debit, credit)];
        let actual = calculate_actual_by_account_type(&AccountType::Revenue, &entries);

        prop_assert_eq!(actual, credit - debit, "Revenue actual should be credit - debit");
    }

    /// **Property 5.4: Liability accounts use credit-normal calculation**
    ///
    /// *For any* liability account, actual = sum(credit) - sum(debit).
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn prop_liability_actual_credit_minus_debit(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        let entries = vec![mock_entry(debit, credit)];
        let actual = calculate_actual_by_account_type(&AccountType::Liability, &entries);

        prop_assert_eq!(actual, credit - debit, "Liability actual should be credit - debit");
    }

    /// **Property 5.5: Equity accounts use credit-normal calculation**
    ///
    /// *For any* equity account, actual = sum(credit) - sum(debit).
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn prop_equity_actual_credit_minus_debit(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        let entries = vec![mock_entry(debit, credit)];
        let actual = calculate_actual_by_account_type(&AccountType::Equity, &entries);

        prop_assert_eq!(actual, credit - debit, "Equity actual should be credit - debit");
    }

    /// **Property 5.6: Actual calculation is additive across entries**
    ///
    /// *For any* account type and multiple entries, the actual amount equals
    /// the sum of individual entry calculations.
    ///
    /// **Validates: Requirements 4.2, 4.3**
    #[test]
    fn prop_actual_additive_across_entries(
        account_type in account_type_strategy(),
        debit1 in amount_strategy(),
        credit1 in amount_strategy(),
        debit2 in amount_strategy(),
        credit2 in amount_strategy(),
    ) {
        let entries = vec![
            mock_entry(debit1, credit1),
            mock_entry(debit2, credit2),
        ];

        let actual = calculate_actual_by_account_type(&account_type, &entries);

        let total_debit = debit1 + debit2;
        let total_credit = credit1 + credit2;

        let expected = if is_debit_normal_account(&account_type) {
            total_debit - total_credit
        } else {
            total_credit - total_debit
        };

        prop_assert_eq!(actual, expected, "Actual should be additive across entries");
    }

    /// **Property 5.7: Empty entries produce zero actual**
    ///
    /// *For any* account type, an empty list of entries produces zero actual.
    ///
    /// **Validates: Requirements 4.2, 4.3**
    #[test]
    fn prop_empty_entries_zero_actual(
        account_type in account_type_strategy(),
    ) {
        let entries: Vec<ledger_entries::Model> = vec![];
        let actual = calculate_actual_by_account_type(&account_type, &entries);

        prop_assert_eq!(actual, Decimal::ZERO, "Empty entries should produce zero actual");
    }

    /// **Property 5.8: Debit-normal classification is correct**
    ///
    /// *For any* account type, is_debit_normal returns true only for Asset and Expense.
    ///
    /// **Validates: Requirements 4.2, 4.3**
    #[test]
    fn prop_debit_normal_classification(
        account_type in account_type_strategy(),
    ) {
        let is_debit = is_debit_normal_account(&account_type);

        let expected = matches!(account_type, AccountType::Asset | AccountType::Expense);

        prop_assert_eq!(is_debit, expected, "Debit-normal classification should be correct");
    }
}

// ============================================================================
// Unit Tests for Specific Examples
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_expense_positive_actual() {
        // Expense account with more debits than credits = positive actual (spending)
        let entries = vec![
            mock_entry(dec!(1000), dec!(0)),
            mock_entry(dec!(500), dec!(200)),
        ];

        let actual = calculate_actual_by_account_type(&AccountType::Expense, &entries);
        assert_eq!(actual, dec!(1300)); // (1000 + 500) - (0 + 200)
    }

    #[test]
    fn test_expense_negative_actual() {
        // Expense account with more credits than debits = negative actual (refund)
        let entries = vec![mock_entry(dec!(100), dec!(500))];

        let actual = calculate_actual_by_account_type(&AccountType::Expense, &entries);
        assert_eq!(actual, dec!(-400)); // 100 - 500
    }

    #[test]
    fn test_revenue_positive_actual() {
        // Revenue account with more credits than debits = positive actual (income)
        let entries = vec![
            mock_entry(dec!(0), dec!(5000)),
            mock_entry(dec!(100), dec!(1000)),
        ];

        let actual = calculate_actual_by_account_type(&AccountType::Revenue, &entries);
        assert_eq!(actual, dec!(5900)); // (5000 + 1000) - (0 + 100)
    }

    #[test]
    fn test_revenue_negative_actual() {
        // Revenue account with more debits than credits = negative actual (reversal)
        let entries = vec![mock_entry(dec!(1000), dec!(200))];

        let actual = calculate_actual_by_account_type(&AccountType::Revenue, &entries);
        assert_eq!(actual, dec!(-800)); // 200 - 1000
    }

    #[test]
    fn test_asset_balance() {
        // Asset account: debit increases, credit decreases
        let entries = vec![
            mock_entry(dec!(10000), dec!(0)), // Cash received
            mock_entry(dec!(0), dec!(3000)),  // Cash paid out
        ];

        let actual = calculate_actual_by_account_type(&AccountType::Asset, &entries);
        assert_eq!(actual, dec!(7000)); // 10000 - 3000
    }

    #[test]
    fn test_liability_balance() {
        // Liability account: credit increases, debit decreases
        let entries = vec![
            mock_entry(dec!(0), dec!(5000)), // Loan received
            mock_entry(dec!(1000), dec!(0)), // Loan payment
        ];

        let actual = calculate_actual_by_account_type(&AccountType::Liability, &entries);
        assert_eq!(actual, dec!(4000)); // 5000 - 1000
    }

    #[test]
    fn test_equity_balance() {
        // Equity account: credit increases, debit decreases
        let entries = vec![
            mock_entry(dec!(0), dec!(100000)), // Capital contribution
            mock_entry(dec!(5000), dec!(0)),   // Distribution
        ];

        let actual = calculate_actual_by_account_type(&AccountType::Equity, &entries);
        assert_eq!(actual, dec!(95000)); // 100000 - 5000
    }

    #[test]
    fn test_is_debit_normal() {
        assert!(is_debit_normal_account(&AccountType::Asset));
        assert!(is_debit_normal_account(&AccountType::Expense));
        assert!(!is_debit_normal_account(&AccountType::Liability));
        assert!(!is_debit_normal_account(&AccountType::Equity));
        assert!(!is_debit_normal_account(&AccountType::Revenue));
    }
}
