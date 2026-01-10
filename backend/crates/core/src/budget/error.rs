//! Budget error types.

use thiserror::Error;
use uuid::Uuid;

/// Budget-related errors.
#[derive(Debug, Error)]
pub enum BudgetError {
    /// Budget not found.
    #[error("Budget not found: {0}")]
    NotFound(Uuid),

    /// Budget is locked and cannot be modified.
    #[error("Budget is locked and cannot be modified")]
    BudgetLocked,

    /// Budget name already exists for this fiscal year.
    #[error("Budget name already exists for this fiscal year")]
    DuplicateName,

    /// Fiscal year not found.
    #[error("Fiscal year not found: {0}")]
    FiscalYearNotFound(Uuid),

    /// Fiscal period not found.
    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    /// Fiscal period does not belong to budget's fiscal year.
    #[error("Fiscal period does not belong to budget's fiscal year")]
    PeriodNotInFiscalYear,

    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Budget line already exists for this account and period.
    #[error("Budget line already exists for this account and period")]
    DuplicateBudgetLine,

    /// Amount cannot be negative.
    #[error("Amount cannot be negative")]
    NegativeAmount,

    /// Currency mismatch.
    #[error("Currency mismatch: expected {expected}, got {got}")]
    CurrencyMismatch {
        /// Expected currency.
        expected: String,
        /// Actual currency.
        got: String,
    },

    /// Invalid dimension value.
    #[error("Invalid dimension value: {0}")]
    InvalidDimension(Uuid),
}
