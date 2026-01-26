//! Automated Accruals Engine logic.
//!
//! This module handles the calculation and scheduling of automated
//! revenue and expense accruals.

use super::types::{
    AccrualFrequency, CreateTransactionInput, EntryType, LedgerEntryInput, TransactionType,
};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use uuid::Uuid;

impl std::str::FromStr for AccrualFrequency {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "monthly" => Ok(AccrualFrequency::Monthly),
            "quarterly" => Ok(AccrualFrequency::Quarterly),
            "yearly" => Ok(AccrualFrequency::Yearly),
            _ => Err(()),
        }
    }
}

/// Accrual engine for processing automated schedules.
#[derive(Debug, Clone)]
pub struct AccrualEngine;

impl AccrualEngine {
    /// Calculate the next run date based on frequency.
    ///
    /// # Panics
    ///
    /// Panics if the date math enters an invalid state (unreachable with valid NaiveDate).
    pub fn calculate_next_run_date(last_run: NaiveDate, frequency: AccrualFrequency) -> NaiveDate {
        match frequency {
            AccrualFrequency::Monthly => {
                let mut year = last_run.year();
                let mut month = last_run.month() + 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
                NaiveDate::from_ymd_opt(year, month, last_run.day()).unwrap_or_else(|| {
                    // Handle end-of-month cases (e.g., Jan 31 -> Feb 28)
                    let mut m = month + 1;
                    let mut y = year;
                    if m > 12 {
                        m = 1;
                        y += 1;
                    }
                    NaiveDate::from_ymd_opt(y, m, 1)
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                })
            }
            AccrualFrequency::Quarterly => {
                let mut year = last_run.year();
                let mut month = last_run.month() + 3;
                if month > 12 {
                    month -= 12;
                    year += 1;
                }
                NaiveDate::from_ymd_opt(year, month, last_run.day()).unwrap_or_else(|| {
                    let mut m = month + 1;
                    let mut y = year;
                    if m > 12 {
                        m = 1;
                        y += 1;
                    }
                    NaiveDate::from_ymd_opt(y, m, 1)
                        .unwrap()
                        .pred_opt()
                        .unwrap()
                })
            }
            AccrualFrequency::Yearly => {
                NaiveDate::from_ymd_opt(last_run.year() + 1, last_run.month(), last_run.day())
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(last_run.year() + 1, 2, 28).unwrap())
            }
        }
    }

    /// Calculate the amount for a specific period.
    pub fn calculate_period_amount(
        total_amount: Decimal,
        total_amount_recognized: Decimal,
        total_periods: i32,
        periods_processed: i32,
    ) -> Decimal {
        if total_periods <= periods_processed {
            return Decimal::ZERO;
        }

        let remaining_amount = total_amount - total_amount_recognized;
        let remaining_periods = total_periods - periods_processed;

        if remaining_periods <= 1 {
            // Last period or one-off: use residual to prevent rounding errors
            return remaining_amount;
        }

        // Proportional calculation for remaining periods
        (remaining_amount / Decimal::from(remaining_periods)).round_dp(4)
    }
}

/// Input for creating an accrual transaction via the engine.
#[derive(Debug, Clone)]
pub struct AccrualTransactionInput<'a> {
    /// Organization ID.
    pub org_id: Uuid,
    /// Entity ID (optional for backward compatibility).
    pub entity_id: Option<Uuid>,
    /// Schedule name.
    pub schedule_name: &'a str,
    /// Amount to accrue.
    pub amount: Decimal,
    /// Currency ID.
    pub currency: &'a str,
    /// Debit account ID.
    pub debit_account_id: Uuid,
    /// Credit account ID.
    pub credit_account_id: Uuid,
    /// Date of the run.
    pub run_date: NaiveDate,
    /// User ID who triggered the run.
    pub created_by: Uuid,
}

impl AccrualEngine {
    /// Generate transaction input for an accrual run.
    pub fn create_accrual_transaction(input: &AccrualTransactionInput) -> CreateTransactionInput {
        let memo = format!("Accrual for {name}", name = input.schedule_name);

        let entries = vec![
            LedgerEntryInput {
                account_id: input.debit_account_id,
                source_currency: input.currency.to_string(),
                source_amount: input.amount,
                entry_type: EntryType::Debit,
                memo: Some(memo.clone()),
                functional_amount: None,
                compliance_metadata: None,
                dimensions: vec![],
            },
            LedgerEntryInput {
                account_id: input.credit_account_id,
                source_currency: input.currency.to_string(),
                source_amount: input.amount,
                entry_type: EntryType::Credit,
                memo: Some(memo),
                functional_amount: None,
                compliance_metadata: None,
                dimensions: vec![],
            },
        ];

        CreateTransactionInput {
            organization_id: input.org_id,
            entity_id: input.entity_id,
            transaction_type: TransactionType::Accrual,
            transaction_date: input.run_date,
            description: format!("Automated Accrual: {name}", name = input.schedule_name),
            reference_number: Some(format!(
                "ACCR-{name}-{date}",
                name = input.schedule_name,
                date = input.run_date
            )),
            memo: None,
            entries,
            created_by: input.created_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_period_amount() {
        let total = dec!(1000);
        let periods = 3;

        // Period 1: 1000 / 3 = 333.3333
        let p1 = AccrualEngine::calculate_period_amount(total, dec!(0), periods, 0);
        assert_eq!(p1, dec!(333.3333));

        // Period 2: (1000 - 333.3333) / 2 = 666.6667 / 2 = 333.3334
        let p2 = AccrualEngine::calculate_period_amount(total, dec!(333.3333), periods, 1);
        assert_eq!(p2, dec!(333.3334));

        // Period 3 (Final): 1000 - (333.3333 + 333.3334) = 333.3333
        let p3 = AccrualEngine::calculate_period_amount(total, dec!(666.6667), periods, 2);
        assert_eq!(p3, dec!(333.3333));
    }

    #[test]
    fn test_calculate_period_amount_amendment() {
        let total = dec!(1000);
        // Original: 3 periods. After 1 period ($333.3333 recognized), amend to 5 total periods.
        let recognized = dec!(333.3333);
        let new_total_periods = 5;
        let p2_new =
            AccrualEngine::calculate_period_amount(total, recognized, new_total_periods, 1);
        // Remaining: 1000 - 333.3333 = 666.6667
        // Remaining periods: 5 - 1 = 4
        // Amount: 666.6667 / 4 = 166.6667
        assert_eq!(p2_new, dec!(166.6667));
    }

    #[test]
    fn test_calculate_next_run_date_monthly() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        // Jan 31 -> Feb 28
        let next = AccrualEngine::calculate_next_run_date(start, AccrualFrequency::Monthly);
        assert_eq!(next, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());

        let start2 = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();
        // Feb 28 -> Mar 28 (or 31? logic currently does Mar 28 if day exists)
        let next2 = AccrualEngine::calculate_next_run_date(start2, AccrualFrequency::Monthly);
        assert_eq!(next2, NaiveDate::from_ymd_opt(2026, 3, 28).unwrap());
    }
}
