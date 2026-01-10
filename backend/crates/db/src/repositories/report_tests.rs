//! Tests for report repository.
//!
//! Property tests for account ledger and dimensional reports.

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use super::{calculate_balance, is_debit_normal};
use crate::entities::sea_orm_active_enums::AccountType;

// ============================================================================
// Property 13: Account Ledger Running Balance
// **Validates: Requirements 8.3**
// ============================================================================

// Note: Property 13 tests that running_balance equals account_current_balance
// from the ledger entry. This is tested at the integration level since it
// requires database state. The repository correctly uses account_current_balance
// as the running_balance field.

// ============================================================================
// Property 14: Account Ledger Ordering
// **Validates: Requirements 8.6**
// ============================================================================

// Note: Property 14 tests that entries are ordered by transaction_date and
// creation order. This is enforced by the query's ORDER BY clause and tested
// at the integration level.

// ============================================================================
// Balance Calculation Property Tests
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property: Debit-normal accounts have positive balance when debits > credits**
    ///
    /// *For any* Asset or Expense account, when total_debit > total_credit,
    /// the balance SHALL be positive.
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_debit_normal_positive_balance(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        prop_assume!(debit > credit);

        let balance_asset = calculate_balance(&AccountType::Asset, debit, credit);
        prop_assert!(balance_asset > Decimal::ZERO, "Asset balance should be positive when debit > credit");

        let balance_expense = calculate_balance(&AccountType::Expense, debit, credit);
        prop_assert!(balance_expense > Decimal::ZERO, "Expense balance should be positive when debit > credit");
    }

    /// **Property: Credit-normal accounts have positive balance when credits > debits**
    ///
    /// *For any* Liability, Equity, or Revenue account, when total_credit > total_debit,
    /// the balance SHALL be positive.
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_credit_normal_positive_balance(
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        prop_assume!(credit > debit);

        let balance_liability = calculate_balance(&AccountType::Liability, debit, credit);
        prop_assert!(balance_liability > Decimal::ZERO, "Liability balance should be positive when credit > debit");

        let balance_equity = calculate_balance(&AccountType::Equity, debit, credit);
        prop_assert!(balance_equity > Decimal::ZERO, "Equity balance should be positive when credit > debit");

        let balance_revenue = calculate_balance(&AccountType::Revenue, debit, credit);
        prop_assert!(balance_revenue > Decimal::ZERO, "Revenue balance should be positive when credit > debit");
    }

    /// **Property: Balance calculation formula is consistent**
    ///
    /// *For any* account type and any debit/credit amounts:
    /// - Debit-normal: balance = debit - credit
    /// - Credit-normal: balance = credit - debit
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_balance_formula_consistent(
        account_type in account_type_strategy(),
        debit in amount_strategy(),
        credit in amount_strategy(),
    ) {
        let balance = calculate_balance(&account_type, debit, credit);

        let expected = if is_debit_normal(&account_type) {
            debit - credit
        } else {
            credit - debit
        };

        prop_assert_eq!(balance, expected, "Balance formula should match account type");
    }

    /// **Property: Zero entries produce zero balance**
    ///
    /// *For any* account type, when both debit and credit are zero,
    /// the balance SHALL be zero.
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_zero_entries_zero_balance(
        account_type in account_type_strategy(),
    ) {
        let balance = calculate_balance(&account_type, Decimal::ZERO, Decimal::ZERO);
        prop_assert_eq!(balance, Decimal::ZERO, "Zero entries should produce zero balance");
    }

    /// **Property: Equal debits and credits produce zero balance**
    ///
    /// *For any* account type, when debit equals credit,
    /// the balance SHALL be zero.
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_equal_entries_zero_balance(
        account_type in account_type_strategy(),
        amount in amount_strategy(),
    ) {
        let balance = calculate_balance(&account_type, amount, amount);
        prop_assert_eq!(balance, Decimal::ZERO, "Equal debit and credit should produce zero balance");
    }
}

// ============================================================================
// Unit Tests for Specific Examples
// ============================================================================

#[test]
fn test_asset_balance_calculation() {
    // Asset is debit-normal: balance = debit - credit
    assert_eq!(
        calculate_balance(&AccountType::Asset, dec!(1000), dec!(0)),
        dec!(1000)
    );
    assert_eq!(
        calculate_balance(&AccountType::Asset, dec!(0), dec!(500)),
        dec!(-500)
    );
    assert_eq!(
        calculate_balance(&AccountType::Asset, dec!(1000), dec!(300)),
        dec!(700)
    );
}

#[test]
fn test_expense_balance_calculation() {
    // Expense is debit-normal: balance = debit - credit
    assert_eq!(
        calculate_balance(&AccountType::Expense, dec!(500), dec!(0)),
        dec!(500)
    );
    assert_eq!(
        calculate_balance(&AccountType::Expense, dec!(0), dec!(200)),
        dec!(-200)
    );
}

#[test]
fn test_liability_balance_calculation() {
    // Liability is credit-normal: balance = credit - debit
    assert_eq!(
        calculate_balance(&AccountType::Liability, dec!(0), dec!(1000)),
        dec!(1000)
    );
    assert_eq!(
        calculate_balance(&AccountType::Liability, dec!(300), dec!(0)),
        dec!(-300)
    );
    assert_eq!(
        calculate_balance(&AccountType::Liability, dec!(200), dec!(1000)),
        dec!(800)
    );
}

#[test]
fn test_equity_balance_calculation() {
    // Equity is credit-normal: balance = credit - debit
    assert_eq!(
        calculate_balance(&AccountType::Equity, dec!(0), dec!(5000)),
        dec!(5000)
    );
    assert_eq!(
        calculate_balance(&AccountType::Equity, dec!(1000), dec!(0)),
        dec!(-1000)
    );
}

#[test]
fn test_revenue_balance_calculation() {
    // Revenue is credit-normal: balance = credit - debit
    assert_eq!(
        calculate_balance(&AccountType::Revenue, dec!(0), dec!(2000)),
        dec!(2000)
    );
    assert_eq!(
        calculate_balance(&AccountType::Revenue, dec!(500), dec!(0)),
        dec!(-500)
    );
}

#[test]
fn test_is_debit_normal() {
    assert!(is_debit_normal(&AccountType::Asset));
    assert!(is_debit_normal(&AccountType::Expense));
    assert!(!is_debit_normal(&AccountType::Liability));
    assert!(!is_debit_normal(&AccountType::Equity));
    assert!(!is_debit_normal(&AccountType::Revenue));
}
