//! Workflow service for transaction state transitions.
//!
//! This module implements the core state machine logic for
//! transitioning transactions through the approval workflow.

use chrono::Utc;
use uuid::Uuid;

use crate::workflow::error::WorkflowError;
use crate::workflow::types::{TransactionStatus, WorkflowAction};

/// Stateless service for managing transaction workflow transitions.
///
/// All methods are associated functions that validate and execute
/// state transitions, returning the appropriate `WorkflowAction`
/// with audit trail information.
pub struct WorkflowService;

impl WorkflowService {
    /// Submit a draft transaction for approval.
    ///
    /// # Arguments
    /// * `current_status` - The current status of the transaction
    /// * `submitted_by` - The user submitting the transaction
    ///
    /// # Returns
    /// * `Ok(WorkflowAction::Submit)` if the transition is valid
    /// * `Err(WorkflowError::InvalidTransition)` if not in Draft status
    pub fn submit(
        current_status: TransactionStatus,
        submitted_by: Uuid,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Draft => Ok(WorkflowAction::Submit {
                new_status: TransactionStatus::Pending,
                submitted_by,
                submitted_at: Utc::now(),
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Pending,
            }),
        }
    }

    /// Approve a pending transaction.
    ///
    /// # Arguments
    /// * `current_status` - The current status of the transaction
    /// * `approved_by` - The user approving the transaction
    /// * `approval_notes` - Optional notes from the approver
    ///
    /// # Returns
    /// * `Ok(WorkflowAction::Approve)` if the transition is valid
    /// * `Err(WorkflowError::InvalidTransition)` if not in Pending status
    pub fn approve(
        current_status: TransactionStatus,
        approved_by: Uuid,
        approval_notes: Option<String>,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Pending => Ok(WorkflowAction::Approve {
                new_status: TransactionStatus::Approved,
                approved_by,
                approved_at: Utc::now(),
                approval_notes,
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Approved,
            }),
        }
    }

    /// Reject a pending transaction back to draft.
    ///
    /// # Arguments
    /// * `current_status` - The current status of the transaction
    /// * `rejection_reason` - The reason for rejection (required)
    ///
    /// # Returns
    /// * `Ok(WorkflowAction::Reject)` if the transition is valid
    /// * `Err(WorkflowError::InvalidTransition)` if not in Pending status
    /// * `Err(WorkflowError::RejectionReasonRequired)` if reason is empty
    pub fn reject(
        current_status: TransactionStatus,
        rejection_reason: String,
    ) -> Result<WorkflowAction, WorkflowError> {
        if rejection_reason.trim().is_empty() {
            return Err(WorkflowError::RejectionReasonRequired);
        }

        match current_status {
            TransactionStatus::Pending => Ok(WorkflowAction::Reject {
                new_status: TransactionStatus::Draft,
                rejection_reason,
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Draft,
            }),
        }
    }

    /// Post an approved transaction to the ledger.
    ///
    /// # Arguments
    /// * `current_status` - The current status of the transaction
    /// * `posted_by` - The user posting the transaction
    ///
    /// # Returns
    /// * `Ok(WorkflowAction::Post)` if the transition is valid
    /// * `Err(WorkflowError::InvalidTransition)` if not in Approved status
    pub fn post(
        current_status: TransactionStatus,
        posted_by: Uuid,
    ) -> Result<WorkflowAction, WorkflowError> {
        match current_status {
            TransactionStatus::Approved => Ok(WorkflowAction::Post {
                new_status: TransactionStatus::Posted,
                posted_by,
                posted_at: Utc::now(),
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Posted,
            }),
        }
    }

    /// Void a posted transaction.
    ///
    /// # Arguments
    /// * `current_status` - The current status of the transaction
    /// * `voided_by` - The user voiding the transaction
    /// * `void_reason` - The reason for voiding (required)
    ///
    /// # Returns
    /// * `Ok(WorkflowAction::Void)` if the transition is valid
    /// * `Err(WorkflowError::InvalidTransition)` if not in Posted status
    /// * `Err(WorkflowError::VoidReasonRequired)` if reason is empty
    pub fn void(
        current_status: TransactionStatus,
        voided_by: Uuid,
        void_reason: String,
    ) -> Result<WorkflowAction, WorkflowError> {
        if void_reason.trim().is_empty() {
            return Err(WorkflowError::VoidReasonRequired);
        }

        match current_status {
            TransactionStatus::Posted => Ok(WorkflowAction::Void {
                new_status: TransactionStatus::Voided,
                voided_by,
                voided_at: Utc::now(),
                void_reason,
            }),
            _ => Err(WorkflowError::InvalidTransition {
                from: current_status,
                to: TransactionStatus::Voided,
            }),
        }
    }

    /// Check if a status transition is valid.
    ///
    /// Valid transitions:
    /// - Draft → Pending (submit)
    /// - Pending → Approved (approve)
    /// - Pending → Draft (reject)
    /// - Approved → Posted (post)
    /// - Posted → Voided (void)
    ///
    /// # Arguments
    /// * `from` - The current status
    /// * `to` - The target status
    ///
    /// # Returns
    /// `true` if the transition is valid, `false` otherwise
    #[must_use]
    pub fn is_valid_transition(from: TransactionStatus, to: TransactionStatus) -> bool {
        matches!(
            (from, to),
            (TransactionStatus::Draft, TransactionStatus::Pending)
                | (
                    TransactionStatus::Pending,
                    TransactionStatus::Approved | TransactionStatus::Draft
                )
                | (TransactionStatus::Approved, TransactionStatus::Posted)
                | (TransactionStatus::Posted, TransactionStatus::Voided)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_from_draft() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::submit(TransactionStatus::Draft, user_id);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action.new_status(), TransactionStatus::Pending);
    }

    #[test]
    fn test_submit_from_non_draft_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::submit(TransactionStatus::Pending, user_id);
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_approve_from_pending() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::approve(TransactionStatus::Pending, user_id, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().new_status(), TransactionStatus::Approved);
    }

    #[test]
    fn test_approve_from_non_pending_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::approve(TransactionStatus::Draft, user_id, None);
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_reject_from_pending() {
        let result =
            WorkflowService::reject(TransactionStatus::Pending, "Invalid data".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().new_status(), TransactionStatus::Draft);
    }

    #[test]
    fn test_reject_empty_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, String::new());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_reject_whitespace_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, "   ".to_string());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_post_from_approved() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::post(TransactionStatus::Approved, user_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().new_status(), TransactionStatus::Posted);
    }

    #[test]
    fn test_post_from_non_approved_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::post(TransactionStatus::Pending, user_id);
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_void_from_posted() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(
            TransactionStatus::Posted,
            user_id,
            "Error found".to_string(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().new_status(), TransactionStatus::Voided);
    }

    #[test]
    fn test_void_empty_reason_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(TransactionStatus::Posted, user_id, String::new());
        assert!(matches!(result, Err(WorkflowError::VoidReasonRequired)));
    }

    #[test]
    fn test_void_from_non_posted_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(
            TransactionStatus::Approved,
            user_id,
            "Error found".to_string(),
        );
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn test_is_valid_transition() {
        // Valid transitions
        assert!(WorkflowService::is_valid_transition(
            TransactionStatus::Draft,
            TransactionStatus::Pending
        ));
        assert!(WorkflowService::is_valid_transition(
            TransactionStatus::Pending,
            TransactionStatus::Approved
        ));
        assert!(WorkflowService::is_valid_transition(
            TransactionStatus::Pending,
            TransactionStatus::Draft
        ));
        assert!(WorkflowService::is_valid_transition(
            TransactionStatus::Approved,
            TransactionStatus::Posted
        ));
        assert!(WorkflowService::is_valid_transition(
            TransactionStatus::Posted,
            TransactionStatus::Voided
        ));

        // Invalid transitions
        assert!(!WorkflowService::is_valid_transition(
            TransactionStatus::Draft,
            TransactionStatus::Approved
        ));
        assert!(!WorkflowService::is_valid_transition(
            TransactionStatus::Draft,
            TransactionStatus::Posted
        ));
        assert!(!WorkflowService::is_valid_transition(
            TransactionStatus::Voided,
            TransactionStatus::Draft
        ));
    }
}
