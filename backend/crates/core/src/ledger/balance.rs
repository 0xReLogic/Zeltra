//! Account balance calculations.
//!
//! Implements Requirements 8.1-8.7 for account balance tracking.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use zeltra_shared::types::AccountId;

/// Account types for balance calculation rules.
/// 
/// Requirements 8.4, 8.5:
/// - Asset/Expense: balance += debit - credit (debit-normal)
/// - Liability/Equity/Revenue: balance += credit - debit (credit-normal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountTypeForBalance {
    /// Debit-normal accounts (Asset, Expense)
    DebitNormal,
    /// Credit-normal accounts (Liability, Equity, Revenue)
    CreditNormal,
}

impl AccountTypeForBalance {
    /// Determines the account type category from a string.
    #[must_use]
    pub fn from_account_type(account_type: &str) -> Self {
        match account_type.to_lowercase().as_str() {
            "asset" | "expense" => Self::DebitNormal,
            "liability" | "equity" | "revenue" => Self::CreditNormal,
            _ => Self::DebitNormal, // Default to debit-normal
        }
    }

    /// Calculates the balance change for an entry.
    /// 
    /// Requirement 8.4: Asset/Expense → balance += debit - credit
    /// Requirement 8.5: Liability/Equity/Revenue → balance += credit - debit
    #[must_use]
    pub fn calculate_balance_change(self, debit: Decimal, credit: Decimal) -> Decimal {
        match self {
            Self::DebitNormal => debit - credit,
            Self::CreditNormal => credit - debit,
        }
    }
}

/// Account balance at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// The account ID.
    pub account_id: AccountId,
    /// Total debit amount.
    pub debit_total: Decimal,
    /// Total credit amount.
    pub credit_total: Decimal,
    /// Net balance (calculated based on account type).
    pub balance: Decimal,
    /// Currency code.
    pub currency: String,
}

impl AccountBalance {
    /// Creates a new account balance.
    #[must_use]
    pub fn new(account_id: AccountId, currency: String) -> Self {
        Self {
            account_id,
            debit_total: Decimal::ZERO,
            credit_total: Decimal::ZERO,
            balance: Decimal::ZERO,
            currency,
        }
    }

    /// Adds a debit amount.
    pub fn add_debit(&mut self, amount: Decimal) {
        self.debit_total += amount;
        self.balance = self.debit_total - self.credit_total;
    }

    /// Adds a credit amount.
    pub fn add_credit(&mut self, amount: Decimal) {
        self.credit_total += amount;
        self.balance = self.debit_total - self.credit_total;
    }
}

/// Running balance information for a ledger entry.
/// 
/// Requirements 8.1-8.3:
/// - account_version: monotonically increasing counter
/// - previous_balance: balance before this entry
/// - current_balance: balance after this entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningBalance {
    /// Account version (monotonically increasing).
    pub account_version: i64,
    /// Balance before this entry.
    pub previous_balance: Decimal,
    /// Balance after this entry.
    pub current_balance: Decimal,
}

impl RunningBalance {
    /// Creates a new running balance for the first entry on an account.
    #[must_use]
    pub fn first_entry(balance_change: Decimal) -> Self {
        Self {
            account_version: 1,
            previous_balance: Decimal::ZERO,
            current_balance: balance_change,
        }
    }

    /// Creates a new running balance based on the previous entry.
    /// 
    /// Property 3: Running Balance Consistency
    /// - current_balance[N] = previous_balance[N] + balance_change
    /// - previous_balance[N] = current_balance[N-1]
    #[must_use]
    pub fn next_entry(previous: &Self, balance_change: Decimal) -> Self {
        Self {
            account_version: previous.account_version + 1,
            previous_balance: previous.current_balance,
            current_balance: previous.current_balance + balance_change,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rust_decimal_macros::dec;

    // ========================================================================
    // Property 3: Running Balance Consistency
    // **Validates: Requirements 8.2, 8.3, 8.7**
    // ========================================================================

    /// Strategy for generating balance changes (can be positive or negative)
    fn balance_change_strategy() -> impl Strategy<Value = Decimal> {
        (-100_000i64..100_000i64).prop_map(|n| Decimal::new(n, 2))
    }

    /// Strategy for generating a sequence of balance changes
    fn balance_changes_strategy(max_len: usize) -> impl Strategy<Value = Vec<Decimal>> {
        prop::collection::vec(balance_change_strategy(), 1..=max_len)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 3.1: Current balance equals previous plus change**
        ///
        /// *For any* ledger entry N, current_balance[N] SHALL equal
        /// previous_balance[N] + balance_change.
        ///
        /// **Validates: Requirements 8.3**
        #[test]
        fn prop_current_equals_previous_plus_change(
            balance_change in balance_change_strategy(),
        ) {
            let rb = RunningBalance::first_entry(balance_change);
            
            // For first entry: current = 0 + change
            prop_assert_eq!(
                rb.current_balance,
                rb.previous_balance + balance_change,
                "current_balance should equal previous_balance + change"
            );
        }

        /// **Property 3.2: Previous balance equals prior current balance**
        ///
        /// *For any* ledger entry N (where N > 1), previous_balance[N] SHALL equal
        /// current_balance[N-1].
        ///
        /// **Validates: Requirements 8.2**
        #[test]
        fn prop_previous_equals_prior_current(
            change1 in balance_change_strategy(),
            change2 in balance_change_strategy(),
        ) {
            let rb1 = RunningBalance::first_entry(change1);
            let rb2 = RunningBalance::next_entry(&rb1, change2);
            
            prop_assert_eq!(
                rb2.previous_balance,
                rb1.current_balance,
                "previous_balance[N] should equal current_balance[N-1]"
            );
        }

        /// **Property 3.3: Running balance chain consistency**
        ///
        /// *For any* sequence of N entries, the final balance SHALL equal
        /// the sum of all balance changes.
        ///
        /// **Validates: Requirements 8.7**
        #[test]
        fn prop_final_balance_equals_sum_of_changes(
            changes in balance_changes_strategy(20),
        ) {
            prop_assume!(!changes.is_empty());

            // Build the chain
            let mut current = RunningBalance::first_entry(changes[0]);
            for change in changes.iter().skip(1) {
                current = RunningBalance::next_entry(&current, *change);
            }

            // Final balance should equal sum of all changes
            let expected_balance: Decimal = changes.iter().copied().sum();
            prop_assert_eq!(
                current.current_balance,
                expected_balance,
                "Final balance should equal sum of all changes"
            );
        }

        /// **Property 3.4: Balance chain is deterministic**
        ///
        /// *For any* sequence of balance changes, applying them in the same order
        /// SHALL always produce the same final balance.
        ///
        /// **Validates: Requirements 8.7**
        #[test]
        fn prop_balance_chain_deterministic(
            changes in balance_changes_strategy(10),
        ) {
            prop_assume!(!changes.is_empty());

            // Build chain twice
            let build_chain = |changes: &[Decimal]| -> RunningBalance {
                let mut current = RunningBalance::first_entry(changes[0]);
                for change in changes.iter().skip(1) {
                    current = RunningBalance::next_entry(&current, *change);
                }
                current
            };

            let result1 = build_chain(&changes);
            let result2 = build_chain(&changes);

            prop_assert_eq!(result1.current_balance, result2.current_balance);
            prop_assert_eq!(result1.account_version, result2.account_version);
        }

        /// **Property 3.5: Version count equals entry count**
        ///
        /// *For any* sequence of N entries, the final account_version SHALL equal N.
        ///
        /// **Validates: Requirements 8.1**
        #[test]
        fn prop_version_equals_entry_count(
            changes in balance_changes_strategy(20),
        ) {
            prop_assume!(!changes.is_empty());

            let mut current = RunningBalance::first_entry(changes[0]);
            for change in changes.iter().skip(1) {
                current = RunningBalance::next_entry(&current, *change);
            }

            prop_assert_eq!(
                current.account_version as usize,
                changes.len(),
                "account_version should equal number of entries"
            );
        }

        /// **Property 3.6: Zero changes preserve balance**
        ///
        /// *For any* starting balance, adding a zero change SHALL preserve the balance.
        ///
        /// **Validates: Requirements 8.3**
        #[test]
        fn prop_zero_change_preserves_balance(
            initial_change in balance_change_strategy(),
        ) {
            let rb1 = RunningBalance::first_entry(initial_change);
            let rb2 = RunningBalance::next_entry(&rb1, Decimal::ZERO);

            prop_assert_eq!(
                rb2.current_balance,
                rb1.current_balance,
                "Zero change should preserve balance"
            );
        }
    }

    // ========================================================================
    // Property 4: Account Version Monotonicity
    // **Validates: Requirements 8.1**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 4.1: Version starts at 1**
        ///
        /// *For any* first entry on an account, account_version SHALL be 1.
        ///
        /// **Validates: Requirements 8.1**
        #[test]
        fn prop_version_starts_at_one(
            balance_change in balance_change_strategy(),
        ) {
            let rb = RunningBalance::first_entry(balance_change);
            prop_assert_eq!(
                rb.account_version,
                1,
                "First entry should have account_version = 1"
            );
        }

        /// **Property 4.2: Version strictly increases by 1**
        ///
        /// *For any* consecutive entries, account_version[N] SHALL equal
        /// account_version[N-1] + 1.
        ///
        /// **Validates: Requirements 8.1**
        #[test]
        fn prop_version_strictly_increases(
            changes in balance_changes_strategy(20),
        ) {
            prop_assume!(!changes.is_empty());

            let mut versions = Vec::with_capacity(changes.len());
            let mut current = RunningBalance::first_entry(changes[0]);
            versions.push(current.account_version);

            for change in changes.iter().skip(1) {
                current = RunningBalance::next_entry(&current, *change);
                versions.push(current.account_version);
            }

            // Check strictly increasing by 1
            for i in 1..versions.len() {
                prop_assert_eq!(
                    versions[i],
                    versions[i - 1] + 1,
                    "Version should increase by exactly 1"
                );
            }
        }

        /// **Property 4.3: Version sequence is contiguous**
        ///
        /// *For any* N entries, the versions SHALL form sequence [1, 2, 3, ..., N].
        ///
        /// **Validates: Requirements 8.1**
        #[test]
        fn prop_version_sequence_contiguous(
            changes in balance_changes_strategy(20),
        ) {
            prop_assume!(!changes.is_empty());

            let mut versions = Vec::with_capacity(changes.len());
            let mut current = RunningBalance::first_entry(changes[0]);
            versions.push(current.account_version);

            for change in changes.iter().skip(1) {
                current = RunningBalance::next_entry(&current, *change);
                versions.push(current.account_version);
            }

            // Expected sequence: [1, 2, 3, ..., N]
            let expected: Vec<i64> = (1..=changes.len() as i64).collect();
            prop_assert_eq!(
                versions,
                expected,
                "Versions should form contiguous sequence [1, 2, ..., N]"
            );
        }

        /// **Property 4.4: Version is always positive**
        ///
        /// *For any* entry, account_version SHALL be positive (> 0).
        ///
        /// **Validates: Requirements 8.1**
        #[test]
        fn prop_version_always_positive(
            changes in balance_changes_strategy(20),
        ) {
            prop_assume!(!changes.is_empty());

            let mut current = RunningBalance::first_entry(changes[0]);
            prop_assert!(current.account_version > 0, "Version must be positive");

            for change in changes.iter().skip(1) {
                current = RunningBalance::next_entry(&current, *change);
                prop_assert!(current.account_version > 0, "Version must be positive");
            }
        }
    }

    // ========================================================================
    // Unit tests for specific examples
    // ========================================================================

    #[test]
    fn test_debit_normal_balance_change() {
        let account_type = AccountTypeForBalance::DebitNormal;
        
        // Debit increases balance
        assert_eq!(account_type.calculate_balance_change(dec!(100), dec!(0)), dec!(100));
        
        // Credit decreases balance
        assert_eq!(account_type.calculate_balance_change(dec!(0), dec!(50)), dec!(-50));
        
        // Net effect
        assert_eq!(account_type.calculate_balance_change(dec!(100), dec!(30)), dec!(70));
    }

    #[test]
    fn test_credit_normal_balance_change() {
        let account_type = AccountTypeForBalance::CreditNormal;
        
        // Credit increases balance
        assert_eq!(account_type.calculate_balance_change(dec!(0), dec!(100)), dec!(100));
        
        // Debit decreases balance
        assert_eq!(account_type.calculate_balance_change(dec!(50), dec!(0)), dec!(-50));
        
        // Net effect
        assert_eq!(account_type.calculate_balance_change(dec!(30), dec!(100)), dec!(70));
    }

    #[test]
    fn test_account_type_from_string() {
        assert_eq!(AccountTypeForBalance::from_account_type("asset"), AccountTypeForBalance::DebitNormal);
        assert_eq!(AccountTypeForBalance::from_account_type("expense"), AccountTypeForBalance::DebitNormal);
        assert_eq!(AccountTypeForBalance::from_account_type("liability"), AccountTypeForBalance::CreditNormal);
        assert_eq!(AccountTypeForBalance::from_account_type("equity"), AccountTypeForBalance::CreditNormal);
        assert_eq!(AccountTypeForBalance::from_account_type("revenue"), AccountTypeForBalance::CreditNormal);
        
        // Case insensitive
        assert_eq!(AccountTypeForBalance::from_account_type("ASSET"), AccountTypeForBalance::DebitNormal);
        assert_eq!(AccountTypeForBalance::from_account_type("Revenue"), AccountTypeForBalance::CreditNormal);
    }

    #[test]
    fn test_running_balance_first_entry() {
        let rb = RunningBalance::first_entry(dec!(100));
        
        assert_eq!(rb.account_version, 1);
        assert_eq!(rb.previous_balance, dec!(0));
        assert_eq!(rb.current_balance, dec!(100));
    }

    #[test]
    fn test_running_balance_chain() {
        // First entry: +100
        let rb1 = RunningBalance::first_entry(dec!(100));
        assert_eq!(rb1.current_balance, dec!(100));
        
        // Second entry: +50
        let rb2 = RunningBalance::next_entry(&rb1, dec!(50));
        assert_eq!(rb2.account_version, 2);
        assert_eq!(rb2.previous_balance, dec!(100)); // = rb1.current_balance
        assert_eq!(rb2.current_balance, dec!(150));
        
        // Third entry: -30
        let rb3 = RunningBalance::next_entry(&rb2, dec!(-30));
        assert_eq!(rb3.account_version, 3);
        assert_eq!(rb3.previous_balance, dec!(150)); // = rb2.current_balance
        assert_eq!(rb3.current_balance, dec!(120));
    }
}
