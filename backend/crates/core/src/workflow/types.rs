//! Workflow domain types for transaction lifecycle management.
//!
//! This module defines the core types used for managing transaction
//! status transitions and workflow actions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Transaction status in the approval workflow.
///
/// Transactions progress through these states from creation to posting.
/// The valid transitions are:
/// - Draft → Pending (submit)
/// - Pending → Approved (approve)
/// - Pending → Draft (reject)
/// - Approved → Posted (post)
/// - Posted → Voided (void)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// Transaction is being drafted and can be modified.
    Draft,
    /// Transaction has been submitted for approval.
    Pending,
    /// Transaction has been approved and is ready for posting.
    Approved,
    /// Transaction has been posted to the ledger (immutable).
    Posted,
    /// Transaction has been voided (immutable).
    Voided,
}

impl TransactionStatus {
    /// Returns the string representation of the status.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Posted => "posted",
            Self::Voided => "voided",
        }
    }

    /// Parses a status from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(Self::Draft),
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "posted" => Some(Self::Posted),
            "voided" => Some(Self::Voided),
            _ => None,
        }
    }

    /// Returns true if the transaction can be modified.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        matches!(self, Self::Draft | Self::Pending)
    }

    /// Returns true if the transaction is immutable.
    #[must_use]
    pub fn is_immutable(&self) -> bool {
        matches!(self, Self::Posted | Self::Voided)
    }
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Workflow action representing a state transition with audit data.
///
/// Each variant captures the action performed, the resulting status,
/// and the audit trail information (who, when, why).
#[derive(Debug, Clone)]
pub enum WorkflowAction {
    /// Submit a draft transaction for approval.
    Submit {
        /// The new status after submission.
        new_status: TransactionStatus,
        /// The user who submitted the transaction.
        submitted_by: Uuid,
        /// When the transaction was submitted.
        submitted_at: DateTime<Utc>,
    },
    /// Approve a pending transaction.
    Approve {
        /// The new status after approval.
        new_status: TransactionStatus,
        /// The user who approved the transaction.
        approved_by: Uuid,
        /// When the transaction was approved.
        approved_at: DateTime<Utc>,
        /// Optional notes from the approver.
        approval_notes: Option<String>,
    },
    /// Reject a pending transaction back to draft.
    Reject {
        /// The new status after rejection (Draft).
        new_status: TransactionStatus,
        /// The reason for rejection.
        rejection_reason: String,
    },
    /// Post an approved transaction to the ledger.
    Post {
        /// The new status after posting.
        new_status: TransactionStatus,
        /// The user who posted the transaction.
        posted_by: Uuid,
        /// When the transaction was posted.
        posted_at: DateTime<Utc>,
    },
    /// Void a posted transaction.
    Void {
        /// The new status after voiding.
        new_status: TransactionStatus,
        /// The user who voided the transaction.
        voided_by: Uuid,
        /// When the transaction was voided.
        voided_at: DateTime<Utc>,
        /// The reason for voiding.
        void_reason: String,
    },
}

impl WorkflowAction {
    /// Returns the new status resulting from this action.
    #[must_use]
    pub fn new_status(&self) -> TransactionStatus {
        match self {
            Self::Submit { new_status, .. }
            | Self::Approve { new_status, .. }
            | Self::Reject { new_status, .. }
            | Self::Post { new_status, .. }
            | Self::Void { new_status, .. } => *new_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_as_str() {
        assert_eq!(TransactionStatus::Draft.as_str(), "draft");
        assert_eq!(TransactionStatus::Pending.as_str(), "pending");
        assert_eq!(TransactionStatus::Approved.as_str(), "approved");
        assert_eq!(TransactionStatus::Posted.as_str(), "posted");
        assert_eq!(TransactionStatus::Voided.as_str(), "voided");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(
            TransactionStatus::parse("draft"),
            Some(TransactionStatus::Draft)
        );
        assert_eq!(
            TransactionStatus::parse("PENDING"),
            Some(TransactionStatus::Pending)
        );
        assert_eq!(
            TransactionStatus::parse("Approved"),
            Some(TransactionStatus::Approved)
        );
        assert_eq!(
            TransactionStatus::parse("posted"),
            Some(TransactionStatus::Posted)
        );
        assert_eq!(
            TransactionStatus::parse("voided"),
            Some(TransactionStatus::Voided)
        );
        assert_eq!(TransactionStatus::parse("invalid"), None);
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TransactionStatus::Draft), "draft");
        assert_eq!(format!("{}", TransactionStatus::Posted), "posted");
    }

    #[test]
    fn test_status_editable() {
        assert!(TransactionStatus::Draft.is_editable());
        assert!(TransactionStatus::Pending.is_editable());
        assert!(!TransactionStatus::Approved.is_editable());
        assert!(!TransactionStatus::Posted.is_editable());
        assert!(!TransactionStatus::Voided.is_editable());
    }

    #[test]
    fn test_status_immutable() {
        assert!(!TransactionStatus::Draft.is_immutable());
        assert!(!TransactionStatus::Pending.is_immutable());
        assert!(!TransactionStatus::Approved.is_immutable());
        assert!(TransactionStatus::Posted.is_immutable());
        assert!(TransactionStatus::Voided.is_immutable());
    }
}
