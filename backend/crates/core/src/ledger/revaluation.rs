//! Real-time Revaluation Engine logic for Zeltra Sentinel.
//!
//! Handles currency revaluation for non-functional currency accounts.

use super::types::{CreateTransactionInput, EntryType, LedgerEntryInput, TransactionType};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Engine for calculating currency revaluation.
pub struct RevaluationEngine;

impl RevaluationEngine {
    /// Calculates the unrealized gain or loss for an account.
    ///
    /// # Arguments
    /// * `carrying_functional_balance` - The current balance in functional currency (base currency).
    /// * `source_balance` - The current balance in the account's source currency.
    /// * `current_rate` - The current exchange rate (source -> functional).
    ///
    /// # Returns
    /// The amount of the unrealized gain (positive) or loss (negative) to be adjusted.
    pub fn calculate_adjustment(
        carrying_functional_balance: Decimal,
        source_balance: Decimal,
        current_rate: Decimal,
    ) -> Decimal {
        let current_value = source_balance * current_rate;
        current_value - carrying_functional_balance
    }
}

/// Input for creating a revaluation transaction.
#[derive(Debug, Clone)]
pub struct RevaluationTransactionInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Entity ID (optional for backward compatibility).
    pub entity_id: Option<Uuid>,
    /// Account being revalued.
    pub account_id: Uuid,
    /// Unrealized Gain/Loss account.
    pub gain_loss_account_id: Uuid,
    /// Account currency code.
    pub account_currency: String,
    /// Organization functional currency code.
    pub base_currency: String,
    /// Calculated adjustment amount (functional).
    pub adjustment_amount: Decimal,
    /// Exchange rate used.
    pub current_rate: Decimal,
    /// Transaction date.
    pub transaction_date: chrono::NaiveDate,
    /// User ID performing the run.
    pub created_by: Uuid,
}

impl RevaluationEngine {
    /// Creates a revaluation transaction input.
    pub fn create_revaluation_transaction(
        input: &RevaluationTransactionInput,
    ) -> CreateTransactionInput {
        let (reval_debit, _reval_credit, gl_debit, gl_credit) =
            if input.adjustment_amount > Decimal::ZERO {
                (
                    input.adjustment_amount,
                    Decimal::ZERO,
                    Decimal::ZERO,
                    input.adjustment_amount,
                )
            } else {
                let abs_amount = input.adjustment_amount.abs();
                (Decimal::ZERO, abs_amount, abs_amount, Decimal::ZERO)
            };

        let memo_text = format!(
            "FX Revaluation: {acc}/{base} at {rate}",
            acc = input.account_currency,
            base = input.base_currency,
            rate = input.current_rate
        );

        CreateTransactionInput {
            organization_id: input.organization_id,
            entity_id: input.entity_id,
            transaction_type: TransactionType::Revaluation,
            description: memo_text.clone(),
            transaction_date: input.transaction_date,
            reference_number: Some(format!(
                "REVAL-{}",
                Uuid::new_v4().to_string()[..8].to_uppercase()
            )),
            memo: Some(memo_text.clone()),
            entries: vec![
                LedgerEntryInput {
                    account_id: input.account_id,
                    source_currency: input.account_currency.clone(),
                    source_amount: Decimal::ZERO,
                    entry_type: if reval_debit > Decimal::ZERO {
                        EntryType::Debit
                    } else {
                        EntryType::Credit
                    },
                    memo: Some(memo_text.clone()),
                    functional_amount: Some(input.adjustment_amount.abs()),
                    compliance_metadata: None,
                    dimensions: vec![],
                },
                LedgerEntryInput {
                    account_id: input.gain_loss_account_id,
                    source_currency: input.base_currency.clone(),
                    source_amount: if gl_debit > Decimal::ZERO {
                        gl_debit
                    } else {
                        gl_credit
                    },
                    entry_type: if gl_debit > Decimal::ZERO {
                        EntryType::Debit
                    } else {
                        EntryType::Credit
                    },
                    memo: Some(memo_text),
                    functional_amount: Some(if gl_debit > Decimal::ZERO {
                        gl_debit
                    } else {
                        gl_credit
                    }),
                    compliance_metadata: None,
                    dimensions: vec![],
                },
            ],
            created_by: input.created_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_adjustment_gain() {
        // Carrying: 100 USD (at rate 1.0)
        // Source: 100 EUR
        // Current Rate: 1.1 (EUR/USD)
        // Current Value: 110 USD
        // Adjustment: +10 USD (Gain)
        let adjustment = RevaluationEngine::calculate_adjustment(dec!(100), dec!(100), dec!(1.1));
        assert_eq!(adjustment, dec!(10));
    }

    #[test]
    fn test_calculate_adjustment_loss() {
        // Carrying: 100 USD
        // Source: 100 EUR
        // Current Rate: 0.9
        // Current Value: 90 USD
        // Adjustment: -10 USD (Loss)
        let adjustment = RevaluationEngine::calculate_adjustment(dec!(100), dec!(100), dec!(0.9));
        assert_eq!(adjustment, dec!(-10));
    }
}
