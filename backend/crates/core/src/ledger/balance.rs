//! Account balance calculations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use zeltra_shared::types::AccountId;

/// Account balance at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// The account ID.
    pub account_id: AccountId,
    /// Total debit amount.
    pub debit_total: Decimal,
    /// Total credit amount.
    pub credit_total: Decimal,
    /// Net balance (debit - credit for debit-normal accounts).
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
