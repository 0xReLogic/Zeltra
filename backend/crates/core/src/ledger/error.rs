//! Ledger error types for validation and state errors.
//!
//! This module defines all errors that can occur during ledger operations,
//! including validation errors, account errors, fiscal period errors,
//! currency errors, dimension errors, and transaction state errors.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during ledger operations.
#[derive(Debug, Error)]
pub enum LedgerError {
    // ========== Validation Errors ==========
    /// Transaction must have at least 2 entries.
    #[error("Transaction must have at least 2 entries")]
    InsufficientEntries,

    /// Transaction is not balanced (debits != credits).
    #[error("Transaction is not balanced. Debit: {debit}, Credit: {credit}")]
    UnbalancedTransaction {
        /// Total debit amount in functional currency.
        debit: Decimal,
        /// Total credit amount in functional currency.
        credit: Decimal,
    },

    /// Entry amount cannot be zero.
    #[error("Entry amount cannot be zero")]
    ZeroAmount,

    /// Entry amount cannot be negative.
    #[error("Entry amount cannot be negative")]
    NegativeAmount,

    /// Entry must specify either debit or credit, not both.
    #[error("Entry must specify either debit or credit, not both")]
    InvalidEntryType,

    // ========== Account Errors ==========
    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Account is inactive and cannot be used.
    #[error("Account {0} is inactive")]
    AccountInactive(Uuid),

    /// Account does not allow direct posting.
    #[error("Account {0} does not allow direct posting")]
    AccountNoDirectPosting(Uuid),

    /// Account type cannot be changed because it has ledger entries.
    #[error("Cannot change account type for account {0} because it has ledger entries")]
    AccountTypeChangeNotAllowed(Uuid),

    // ========== Fiscal Period Errors ==========
    /// No fiscal period found for the transaction date.
    #[error("No fiscal period found for date {0}")]
    NoFiscalPeriod(NaiveDate),

    /// Fiscal period is closed, no posting allowed.
    #[error("Fiscal period is closed, no posting allowed")]
    PeriodClosed,

    /// Fiscal period is soft-closed, only accountants can post.
    #[error("Fiscal period is soft-closed, only accountants can post")]
    PeriodSoftClosed,

    /// Cannot close fiscal period because earlier periods are still open.
    #[error("Cannot close fiscal period: earlier periods must be closed first")]
    EarlierPeriodsNotClosed,

    // ========== Currency Errors ==========
    /// No exchange rate found for the currency pair on the given date.
    #[error("No exchange rate found for {from} to {to} on {date}")]
    NoExchangeRate {
        /// Source currency code.
        from: String,
        /// Target currency code.
        to: String,
        /// Date for which the rate was requested.
        date: NaiveDate,
    },

    /// Exchange rate must be positive.
    #[error("Exchange rate must be positive")]
    InvalidExchangeRate,

    /// Source and target currencies must be different.
    #[error("Source and target currencies must be different")]
    SameCurrencyExchange,

    // ========== Dimension Errors ==========
    /// Invalid dimension value.
    #[error("Invalid dimension value: {0}")]
    InvalidDimension(Uuid),

    /// Dimension value is inactive.
    #[error("Dimension value {0} is inactive")]
    DimensionInactive(Uuid),

    /// Required dimension type is missing.
    #[error("Required dimension type missing: {0}")]
    RequiredDimensionMissing(String),

    /// Dimension value does not belong to the organization.
    #[error("Dimension value {0} does not belong to the organization")]
    DimensionOrganizationMismatch(Uuid),

    // ========== Transaction State Errors ==========
    /// Cannot modify a posted transaction.
    #[error("Cannot modify posted transaction")]
    CannotModifyPosted,

    /// Cannot modify a voided transaction.
    #[error("Cannot modify voided transaction")]
    CannotModifyVoided,

    /// Can only delete draft transactions.
    #[error("Can only delete draft transactions")]
    CanOnlyDeleteDraft,

    /// Transaction not found.
    #[error("Transaction not found: {0}")]
    TransactionNotFound(Uuid),

    // ========== Concurrency Errors ==========
    /// Concurrent modification detected.
    #[error("Concurrent modification detected, please retry")]
    ConcurrentModification,

    /// Account version mismatch.
    #[error("Account version mismatch for account {account_id}: expected {expected}, got {actual}")]
    AccountVersionMismatch {
        /// The account ID.
        account_id: Uuid,
        /// The expected version.
        expected: i64,
        /// The actual version found.
        actual: i64,
    },

    // ========== Database Errors ==========
    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LedgerError {
    /// Returns the error code for API responses.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InsufficientEntries => "INSUFFICIENT_ENTRIES",
            Self::UnbalancedTransaction { .. } => "UNBALANCED_TRANSACTION",
            Self::ZeroAmount => "ZERO_AMOUNT",
            Self::NegativeAmount => "NEGATIVE_AMOUNT",
            Self::InvalidEntryType => "INVALID_ENTRY_TYPE",
            Self::AccountNotFound(_) => "ACCOUNT_NOT_FOUND",
            Self::AccountInactive(_) => "ACCOUNT_INACTIVE",
            Self::AccountNoDirectPosting(_) => "ACCOUNT_NO_DIRECT_POSTING",
            Self::AccountTypeChangeNotAllowed(_) => "ACCOUNT_TYPE_CHANGE_NOT_ALLOWED",
            Self::NoFiscalPeriod(_) => "NO_FISCAL_PERIOD",
            Self::PeriodClosed => "PERIOD_CLOSED",
            Self::PeriodSoftClosed => "PERIOD_SOFT_CLOSED",
            Self::EarlierPeriodsNotClosed => "EARLIER_PERIODS_NOT_CLOSED",
            Self::NoExchangeRate { .. } => "NO_EXCHANGE_RATE",
            Self::InvalidExchangeRate => "INVALID_EXCHANGE_RATE",
            Self::SameCurrencyExchange => "SAME_CURRENCY_EXCHANGE",
            Self::InvalidDimension(_) => "INVALID_DIMENSION",
            Self::DimensionInactive(_) => "DIMENSION_INACTIVE",
            Self::RequiredDimensionMissing(_) => "REQUIRED_DIMENSION_MISSING",
            Self::DimensionOrganizationMismatch(_) => "DIMENSION_ORGANIZATION_MISMATCH",
            Self::CannotModifyPosted => "CANNOT_MODIFY_POSTED",
            Self::CannotModifyVoided => "CANNOT_MODIFY_VOIDED",
            Self::CanOnlyDeleteDraft => "CAN_ONLY_DELETE_DRAFT",
            Self::TransactionNotFound(_) => "TRANSACTION_NOT_FOUND",
            Self::ConcurrentModification => "CONCURRENT_MODIFICATION",
            Self::AccountVersionMismatch { .. } => "ACCOUNT_VERSION_MISMATCH",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Returns the HTTP status code for this error.
    #[must_use]
    pub fn http_status_code(&self) -> u16 {
        match self {
            // 400 Bad Request - validation errors
            Self::InsufficientEntries
            | Self::UnbalancedTransaction { .. }
            | Self::ZeroAmount
            | Self::NegativeAmount
            | Self::InvalidEntryType
            | Self::AccountInactive(_)
            | Self::AccountNoDirectPosting(_)
            | Self::AccountTypeChangeNotAllowed(_)
            | Self::NoFiscalPeriod(_)
            | Self::PeriodClosed
            | Self::EarlierPeriodsNotClosed
            | Self::NoExchangeRate { .. }
            | Self::InvalidExchangeRate
            | Self::SameCurrencyExchange
            | Self::InvalidDimension(_)
            | Self::DimensionInactive(_)
            | Self::RequiredDimensionMissing(_)
            | Self::DimensionOrganizationMismatch(_)
            | Self::CannotModifyPosted
            | Self::CannotModifyVoided
            | Self::CanOnlyDeleteDraft => 400,

            // 403 Forbidden - permission errors
            Self::PeriodSoftClosed => 403,

            // 404 Not Found
            Self::AccountNotFound(_) | Self::TransactionNotFound(_) => 404,

            // 409 Conflict - concurrency errors
            Self::ConcurrentModification | Self::AccountVersionMismatch { .. } => 409,

            // 500 Internal Server Error
            Self::Database(_) | Self::Internal(_) => 500,
        }
    }

    /// Returns true if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConcurrentModification | Self::AccountVersionMismatch { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(LedgerError::InsufficientEntries.error_code(), "INSUFFICIENT_ENTRIES");
        assert_eq!(
            LedgerError::UnbalancedTransaction {
                debit: Decimal::new(100, 2),
                credit: Decimal::new(50, 2),
            }
            .error_code(),
            "UNBALANCED_TRANSACTION"
        );
        assert_eq!(LedgerError::ZeroAmount.error_code(), "ZERO_AMOUNT");
        assert_eq!(LedgerError::NegativeAmount.error_code(), "NEGATIVE_AMOUNT");
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(LedgerError::InsufficientEntries.http_status_code(), 400);
        assert_eq!(LedgerError::PeriodSoftClosed.http_status_code(), 403);
        assert_eq!(
            LedgerError::AccountNotFound(Uuid::nil()).http_status_code(),
            404
        );
        assert_eq!(LedgerError::ConcurrentModification.http_status_code(), 409);
        assert_eq!(
            LedgerError::Database("test".to_string()).http_status_code(),
            500
        );
    }

    #[test]
    fn test_retryable_errors() {
        assert!(LedgerError::ConcurrentModification.is_retryable());
        assert!(LedgerError::AccountVersionMismatch {
            account_id: Uuid::nil(),
            expected: 1,
            actual: 2,
        }
        .is_retryable());
        assert!(!LedgerError::InsufficientEntries.is_retryable());
        assert!(!LedgerError::ZeroAmount.is_retryable());
    }

    #[test]
    fn test_error_display() {
        let err = LedgerError::UnbalancedTransaction {
            debit: Decimal::new(10000, 2),
            credit: Decimal::new(5000, 2),
        };
        assert_eq!(
            err.to_string(),
            "Transaction is not balanced. Debit: 100.00, Credit: 50.00"
        );

        let err = LedgerError::NoExchangeRate {
            from: "EUR".to_string(),
            to: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };
        assert_eq!(
            err.to_string(),
            "No exchange rate found for EUR to USD on 2024-01-15"
        );
    }
}
