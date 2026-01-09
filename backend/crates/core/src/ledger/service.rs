//! Ledger service for transaction validation and resolution.
//!
//! This module provides the core business logic for validating and resolving
//! financial transactions before they are persisted to the database.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::error::LedgerError;
use super::types::{
    CreateTransactionInput, EntryType, LedgerEntryInput, ResolvedEntry, TransactionTotals,
};
use crate::currency::CurrencyService;

/// Information about an account needed for validation.
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// The account ID.
    pub id: Uuid,
    /// Whether the account is active.
    pub is_active: bool,
    /// Whether the account allows direct posting.
    pub allow_direct_posting: bool,
    /// The account's currency code.
    pub currency: String,
}

/// Ledger service for transaction validation and resolution.
///
/// This service contains pure business logic with no database dependencies.
/// It validates transactions and resolves exchange rates before persistence.
pub struct LedgerService;

impl LedgerService {
    /// Validate and resolve a transaction before persisting.
    ///
    /// This function performs all validation and resolution steps:
    /// 1. Validates minimum entries (at least 2)
    /// 2. Validates each entry's amount (positive, non-zero)
    /// 3. Validates accounts (exist, active, allow direct posting)
    /// 4. Validates dimensions (exist, active)
    /// 5. Resolves exchange rates for multi-currency entries
    /// 6. Calculates functional amounts using Banker's Rounding
    /// 7. Validates transaction balance (debits = credits in functional currency)
    ///
    /// # Arguments
    ///
    /// * `input` - The transaction input to validate
    /// * `org_base_currency` - The organization's functional currency
    /// * `exchange_rate_lookup` - Function to look up exchange rates
    /// * `account_validator` - Function to validate and get account info
    /// * `dimension_validator` - Function to validate dimension values
    ///
    /// # Returns
    ///
    /// A tuple of (resolved entries, transaction totals) on success.
    ///
    /// # Errors
    ///
    /// Returns `LedgerError` if validation fails.
    pub fn validate_and_resolve<F, A, D>(
        input: &CreateTransactionInput,
        org_base_currency: &str,
        exchange_rate_lookup: F,
        account_validator: A,
        dimension_validator: D,
    ) -> Result<(Vec<ResolvedEntry>, TransactionTotals), LedgerError>
    where
        F: Fn(&str, &str, NaiveDate) -> Option<Decimal>,
        A: Fn(Uuid) -> Result<AccountInfo, LedgerError>,
        D: Fn(&[Uuid]) -> Result<(), LedgerError>,
    {
        // 1. Validate minimum entries
        if input.entries.len() < 2 {
            return Err(LedgerError::InsufficientEntries);
        }

        // 2. Resolve each entry
        let mut resolved = Vec::with_capacity(input.entries.len());

        for entry in &input.entries {
            let resolved_entry = Self::resolve_entry(
                entry,
                input.transaction_date,
                org_base_currency,
                &exchange_rate_lookup,
                &account_validator,
                &dimension_validator,
            )?;
            resolved.push(resolved_entry);
        }

        // 3. Calculate totals and validate balance
        let totals = Self::calculate_totals(&resolved);

        if !totals.is_balanced {
            return Err(LedgerError::UnbalancedTransaction {
                debit: totals.functional_debit,
                credit: totals.functional_credit,
            });
        }

        Ok((resolved, totals))
    }

    /// Resolve a single entry with exchange rate lookup.
    fn resolve_entry<F, A, D>(
        entry: &LedgerEntryInput,
        transaction_date: NaiveDate,
        org_base_currency: &str,
        exchange_rate_lookup: &F,
        account_validator: &A,
        dimension_validator: &D,
    ) -> Result<ResolvedEntry, LedgerError>
    where
        F: Fn(&str, &str, NaiveDate) -> Option<Decimal>,
        A: Fn(Uuid) -> Result<AccountInfo, LedgerError>,
        D: Fn(&[Uuid]) -> Result<(), LedgerError>,
    {
        // Validate amount
        if entry.source_amount == Decimal::ZERO {
            return Err(LedgerError::ZeroAmount);
        }
        if entry.source_amount < Decimal::ZERO {
            return Err(LedgerError::NegativeAmount);
        }

        // Validate account
        let account_info = account_validator(entry.account_id)?;
        if !account_info.is_active {
            return Err(LedgerError::AccountInactive(entry.account_id));
        }
        if !account_info.allow_direct_posting {
            return Err(LedgerError::AccountNoDirectPosting(entry.account_id));
        }

        // Validate dimensions
        if !entry.dimensions.is_empty() {
            dimension_validator(&entry.dimensions)?;
        }

        // Get exchange rate
        let exchange_rate = if entry.source_currency == org_base_currency {
            Decimal::ONE
        } else {
            exchange_rate_lookup(&entry.source_currency, org_base_currency, transaction_date)
                .ok_or_else(|| LedgerError::NoExchangeRate {
                    from: entry.source_currency.clone(),
                    to: org_base_currency.to_string(),
                    date: transaction_date,
                })?
        };

        // Calculate functional amount with Banker's Rounding (4 decimal places)
        let functional_amount = CurrencyService::convert(entry.source_amount, exchange_rate);

        // Determine debit/credit amounts
        let (debit, credit) = match entry.entry_type {
            EntryType::Debit => (functional_amount, Decimal::ZERO),
            EntryType::Credit => (Decimal::ZERO, functional_amount),
        };

        Ok(ResolvedEntry {
            account_id: entry.account_id,
            source_currency: entry.source_currency.clone(),
            source_amount: entry.source_amount,
            exchange_rate,
            functional_currency: org_base_currency.to_string(),
            functional_amount,
            debit,
            credit,
            memo: entry.memo.clone(),
            dimensions: entry.dimensions.clone(),
        })
    }

    /// Calculate transaction totals from resolved entries.
    #[must_use]
    pub fn calculate_totals(entries: &[ResolvedEntry]) -> TransactionTotals {
        let functional_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let functional_credit: Decimal = entries.iter().map(|e| e.credit).sum();

        TransactionTotals::new(functional_debit, functional_credit)
    }

    /// Validate that a transaction can be modified.
    ///
    /// # Errors
    ///
    /// Returns error if transaction is posted or voided.
    pub fn validate_can_modify(
        status: super::types::TransactionStatus,
    ) -> Result<(), LedgerError> {
        use super::types::TransactionStatus;

        match status {
            TransactionStatus::Posted => Err(LedgerError::CannotModifyPosted),
            TransactionStatus::Voided => Err(LedgerError::CannotModifyVoided),
            _ => Ok(()),
        }
    }

    /// Validate that a transaction can be deleted.
    ///
    /// Only draft transactions can be deleted.
    ///
    /// # Errors
    ///
    /// Returns error if transaction is not in draft status.
    pub fn validate_can_delete(
        status: super::types::TransactionStatus,
    ) -> Result<(), LedgerError> {
        use super::types::TransactionStatus;

        if status != TransactionStatus::Draft {
            return Err(LedgerError::CanOnlyDeleteDraft);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_account_info(id: Uuid) -> AccountInfo {
        AccountInfo {
            id,
            is_active: true,
            allow_direct_posting: true,
            currency: "USD".to_string(),
        }
    }

    fn make_entry(entry_type: EntryType, amount: Decimal) -> LedgerEntryInput {
        LedgerEntryInput {
            account_id: Uuid::new_v4(),
            source_currency: "USD".to_string(),
            source_amount: amount,
            entry_type,
            memo: None,
            dimensions: vec![],
        }
    }

    fn make_input(entries: Vec<LedgerEntryInput>) -> CreateTransactionInput {
        CreateTransactionInput {
            organization_id: Uuid::new_v4(),
            transaction_type: super::super::types::TransactionType::Journal,
            transaction_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            description: "Test transaction".to_string(),
            reference_number: None,
            memo: None,
            entries,
            created_by: Uuid::new_v4(),
        }
    }

    // Mock validators
    fn ok_account_validator(id: Uuid) -> Result<AccountInfo, LedgerError> {
        Ok(make_account_info(id))
    }

    fn ok_dimension_validator(_dims: &[Uuid]) -> Result<(), LedgerError> {
        Ok(())
    }

    fn same_currency_rate_lookup(_from: &str, _to: &str, _date: NaiveDate) -> Option<Decimal> {
        Some(Decimal::ONE)
    }

    #[test]
    fn test_validate_balanced_transaction() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        let input = make_input(entries);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(result.is_ok());
        let (resolved, totals) = result.unwrap();
        assert_eq!(resolved.len(), 2);
        assert!(totals.is_balanced);
        assert_eq!(totals.functional_debit, dec!(100));
        assert_eq!(totals.functional_credit, dec!(100));
    }

    #[test]
    fn test_validate_unbalanced_transaction() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(50)),
        ];
        let input = make_input(entries);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(matches!(
            result,
            Err(LedgerError::UnbalancedTransaction { .. })
        ));
    }

    #[test]
    fn test_validate_insufficient_entries() {
        let entries = vec![make_entry(EntryType::Debit, dec!(100))];
        let input = make_input(entries);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(matches!(result, Err(LedgerError::InsufficientEntries)));
    }

    #[test]
    fn test_validate_zero_amount() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(0)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        let input = make_input(entries);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(matches!(result, Err(LedgerError::ZeroAmount)));
    }

    #[test]
    fn test_validate_negative_amount() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(-100)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        let input = make_input(entries);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(matches!(result, Err(LedgerError::NegativeAmount)));
    }

    #[test]
    fn test_validate_inactive_account() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        let input = make_input(entries);

        let inactive_validator = |id: Uuid| -> Result<AccountInfo, LedgerError> {
            Ok(AccountInfo {
                id,
                is_active: false,
                allow_direct_posting: true,
                currency: "USD".to_string(),
            })
        };

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            inactive_validator,
            ok_dimension_validator,
        );

        assert!(matches!(result, Err(LedgerError::AccountInactive(_))));
    }

    #[test]
    fn test_validate_no_direct_posting() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        let input = make_input(entries);

        let no_posting_validator = |id: Uuid| -> Result<AccountInfo, LedgerError> {
            Ok(AccountInfo {
                id,
                is_active: true,
                allow_direct_posting: false,
                currency: "USD".to_string(),
            })
        };

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            no_posting_validator,
            ok_dimension_validator,
        );

        assert!(matches!(result, Err(LedgerError::AccountNoDirectPosting(_))));
    }

    #[test]
    fn test_validate_missing_exchange_rate() {
        let mut entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        entries[0].source_currency = "EUR".to_string();
        let input = make_input(entries);

        let no_rate_lookup = |_from: &str, _to: &str, _date: NaiveDate| -> Option<Decimal> { None };

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            no_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(matches!(result, Err(LedgerError::NoExchangeRate { .. })));
    }

    #[test]
    fn test_multi_currency_conversion() {
        let mut entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(150)),
        ];
        entries[0].source_currency = "EUR".to_string();
        let input = make_input(entries);

        // EUR to USD rate = 1.5
        let rate_lookup = |from: &str, _to: &str, _date: NaiveDate| -> Option<Decimal> {
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

        // EUR 100 * 1.5 = USD 150
        assert_eq!(resolved[0].functional_amount, dec!(150));
        assert_eq!(resolved[0].exchange_rate, dec!(1.5));
        assert!(totals.is_balanced);
    }

    #[test]
    fn test_same_currency_rate_is_one() {
        let entries = vec![
            make_entry(EntryType::Debit, dec!(100)),
            make_entry(EntryType::Credit, dec!(100)),
        ];
        let input = make_input(entries);

        let result = LedgerService::validate_and_resolve(
            &input,
            "USD",
            same_currency_rate_lookup,
            ok_account_validator,
            ok_dimension_validator,
        );

        assert!(result.is_ok());
        let (resolved, _) = result.unwrap();

        // Same currency: rate = 1, functional = source
        assert_eq!(resolved[0].exchange_rate, Decimal::ONE);
        assert_eq!(resolved[0].functional_amount, dec!(100));
    }

    #[test]
    fn test_validate_can_modify_draft() {
        use super::super::types::TransactionStatus;
        assert!(LedgerService::validate_can_modify(TransactionStatus::Draft).is_ok());
    }

    #[test]
    fn test_validate_can_modify_posted() {
        use super::super::types::TransactionStatus;
        assert!(matches!(
            LedgerService::validate_can_modify(TransactionStatus::Posted),
            Err(LedgerError::CannotModifyPosted)
        ));
    }

    #[test]
    fn test_validate_can_modify_voided() {
        use super::super::types::TransactionStatus;
        assert!(matches!(
            LedgerService::validate_can_modify(TransactionStatus::Voided),
            Err(LedgerError::CannotModifyVoided)
        ));
    }

    #[test]
    fn test_validate_can_delete_draft() {
        use super::super::types::TransactionStatus;
        assert!(LedgerService::validate_can_delete(TransactionStatus::Draft).is_ok());
    }

    #[test]
    fn test_validate_can_delete_posted() {
        use super::super::types::TransactionStatus;
        assert!(matches!(
            LedgerService::validate_can_delete(TransactionStatus::Posted),
            Err(LedgerError::CanOnlyDeleteDraft)
        ));
    }
}
