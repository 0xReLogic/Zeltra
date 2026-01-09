//! Workflow error types for transaction lifecycle management.
//!
//! This module defines all error types that can occur during
//! workflow operations such as status transitions, approvals, and voids.

use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

use crate::workflow::types::TransactionStatus;

/// Errors that can occur during workflow operations.
#[derive(Debug, Error)]
pub enum WorkflowError {
    /// Attempted an invalid status transition.
    #[error("Invalid status transition from {from} to {to}")]
    InvalidTransition {
        /// The current status.
        from: TransactionStatus,
        /// The attempted target status.
        to: TransactionStatus,
    },

    /// Attempted to modify a posted transaction.
    #[error("Cannot modify posted transaction")]
    CannotModifyPosted,

    /// Attempted to modify a voided transaction.
    #[error("Cannot modify voided transaction")]
    CannotModifyVoided,

    /// User is not authorized to approve the transaction.
    #[error("User is not authorized to approve this transaction")]
    NotAuthorizedToApprove,

    /// User is not authorized to approve the transaction (with user_id).
    #[error("User {user_id} is not authorized to approve this transaction")]
    NotAuthorizedToApproveUser {
        /// The user who attempted to approve.
        user_id: Uuid,
    },

    /// Transaction amount exceeds user's approval limit.
    #[error("Transaction amount {amount} exceeds user approval limit {limit}")]
    ExceedsApprovalLimit {
        /// The transaction amount.
        amount: Decimal,
        /// The user's approval limit.
        limit: Decimal,
    },

    /// No approval rule found for the transaction.
    #[error("No approval rule found for transaction type {transaction_type} with amount {amount}")]
    NoApprovalRuleFound {
        /// The transaction type.
        transaction_type: String,
        /// The transaction amount.
        amount: Decimal,
    },

    /// User's role does not meet the required role.
    #[error("User role {user_role} does not meet required role {required_role}")]
    InsufficientRole {
        /// The user's role.
        user_role: String,
        /// The required role for the operation.
        required_role: String,
    },

    /// Transaction not found.
    #[error("Transaction {0} not found")]
    TransactionNotFound(Uuid),

    /// Void reason is required but not provided.
    #[error("Void reason is required")]
    VoidReasonRequired,

    /// Rejection reason is required but not provided.
    #[error("Rejection reason is required")]
    RejectionReasonRequired,

    /// Database error.
    #[error("Database error: {0}")]
    Database(String),
}

impl WorkflowError {
    /// Returns the HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::InvalidTransition { .. }
            | Self::CannotModifyPosted
            | Self::CannotModifyVoided
            | Self::VoidReasonRequired
            | Self::RejectionReasonRequired => 400,

            Self::NotAuthorizedToApprove
            | Self::NotAuthorizedToApproveUser { .. }
            | Self::ExceedsApprovalLimit { .. }
            | Self::InsufficientRole { .. } => 403,

            Self::TransactionNotFound(_) | Self::NoApprovalRuleFound { .. } => 404,

            Self::Database(_) => 500,
        }
    }

    /// Returns the error code for API responses.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidTransition { .. } => "INVALID_TRANSITION",
            Self::CannotModifyPosted => "CANNOT_MODIFY_POSTED",
            Self::CannotModifyVoided => "CANNOT_MODIFY_VOIDED",
            Self::NotAuthorizedToApprove | Self::NotAuthorizedToApproveUser { .. } => {
                "NOT_AUTHORIZED_TO_APPROVE"
            }
            Self::ExceedsApprovalLimit { .. } => "EXCEEDS_APPROVAL_LIMIT",
            Self::NoApprovalRuleFound { .. } => "NO_APPROVAL_RULE_FOUND",
            Self::InsufficientRole { .. } => "INSUFFICIENT_ROLE",
            Self::TransactionNotFound(_) => "TRANSACTION_NOT_FOUND",
            Self::VoidReasonRequired => "VOID_REASON_REQUIRED",
            Self::RejectionReasonRequired => "REJECTION_REASON_REQUIRED",
            Self::Database(_) => "DATABASE_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_transition_error() {
        let err = WorkflowError::InvalidTransition {
            from: TransactionStatus::Draft,
            to: TransactionStatus::Posted,
        };
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.error_code(), "INVALID_TRANSITION");
        assert!(err.to_string().contains("draft"));
        assert!(err.to_string().contains("posted"));
    }

    #[test]
    fn test_cannot_modify_posted_error() {
        let err = WorkflowError::CannotModifyPosted;
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.error_code(), "CANNOT_MODIFY_POSTED");
    }

    #[test]
    fn test_cannot_modify_voided_error() {
        let err = WorkflowError::CannotModifyVoided;
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.error_code(), "CANNOT_MODIFY_VOIDED");
    }

    #[test]
    fn test_not_authorized_error() {
        let err = WorkflowError::NotAuthorizedToApprove;
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.error_code(), "NOT_AUTHORIZED_TO_APPROVE");
    }

    #[test]
    fn test_not_authorized_user_error() {
        let err = WorkflowError::NotAuthorizedToApproveUser {
            user_id: Uuid::nil(),
        };
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.error_code(), "NOT_AUTHORIZED_TO_APPROVE");
    }

    #[test]
    fn test_exceeds_limit_error() {
        let err = WorkflowError::ExceedsApprovalLimit {
            amount: Decimal::new(10000, 2),
            limit: Decimal::new(5000, 2),
        };
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.error_code(), "EXCEEDS_APPROVAL_LIMIT");
    }

    #[test]
    fn test_insufficient_role_error() {
        let err = WorkflowError::InsufficientRole {
            user_role: "submitter".to_string(),
            required_role: "approver".to_string(),
        };
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.error_code(), "INSUFFICIENT_ROLE");
    }

    #[test]
    fn test_transaction_not_found_error() {
        let err = WorkflowError::TransactionNotFound(Uuid::nil());
        assert_eq!(err.status_code(), 404);
        assert_eq!(err.error_code(), "TRANSACTION_NOT_FOUND");
    }

    #[test]
    fn test_void_reason_required_error() {
        let err = WorkflowError::VoidReasonRequired;
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.error_code(), "VOID_REASON_REQUIRED");
    }

    #[test]
    fn test_rejection_reason_required_error() {
        let err = WorkflowError::RejectionReasonRequired;
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.error_code(), "REJECTION_REASON_REQUIRED");
    }
}
