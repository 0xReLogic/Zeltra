//! Property-based tests for WorkflowService.
//!
//! These tests validate the correctness properties defined in the design document
//! using proptest for randomized input generation.

use proptest::prelude::*;
use uuid::Uuid;

use crate::workflow::error::WorkflowError;
use crate::workflow::service::WorkflowService;
use crate::workflow::types::TransactionStatus;

/// Strategy for generating random TransactionStatus values.
fn arb_status() -> impl Strategy<Value = TransactionStatus> {
    prop_oneof![
        Just(TransactionStatus::Draft),
        Just(TransactionStatus::Pending),
        Just(TransactionStatus::Approved),
        Just(TransactionStatus::Posted),
        Just(TransactionStatus::Voided),
    ]
}

/// Strategy for generating random UUIDs.
fn arb_uuid() -> impl Strategy<Value = Uuid> {
    any::<u128>().prop_map(Uuid::from_u128)
}

/// Strategy for generating non-empty strings (for reasons).
fn arb_non_empty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,100}".prop_map(|s| s.trim().to_string())
}

/// Strategy for generating optional approval notes.
fn arb_approval_notes() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), arb_non_empty_string().prop_map(Some),]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 1: Valid State Transitions
    // Feature: transaction-workflow, Property 1: Valid State Transitions
    // Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.6
    // =========================================================================

    /// Draft + submit → Pending with correct audit fields
    #[test]
    fn prop_submit_from_draft_succeeds(user_id in arb_uuid()) {
        let result = WorkflowService::submit(TransactionStatus::Draft, user_id);
        prop_assert!(result.is_ok());
        let action = result.unwrap();
        prop_assert_eq!(action.new_status(), TransactionStatus::Pending);

        // Verify audit fields
        if let crate::workflow::types::WorkflowAction::Submit { submitted_by, .. } = action {
            prop_assert_eq!(submitted_by, user_id);
        } else {
            prop_assert!(false, "Expected Submit action");
        }
    }

    /// Pending + approve → Approved with correct audit fields
    #[test]
    fn prop_approve_from_pending_succeeds(
        user_id in arb_uuid(),
        notes in arb_approval_notes()
    ) {
        let result = WorkflowService::approve(TransactionStatus::Pending, user_id, notes.clone());
        prop_assert!(result.is_ok());
        let action = result.unwrap();
        prop_assert_eq!(action.new_status(), TransactionStatus::Approved);

        // Verify audit fields
        if let crate::workflow::types::WorkflowAction::Approve { approved_by, approval_notes, .. } = action {
            prop_assert_eq!(approved_by, user_id);
            prop_assert_eq!(approval_notes, notes);
        } else {
            prop_assert!(false, "Expected Approve action");
        }
    }

    /// Pending + reject → Draft with rejection reason
    #[test]
    fn prop_reject_from_pending_succeeds(reason in arb_non_empty_string()) {
        // Skip empty strings after trim
        prop_assume!(!reason.trim().is_empty());

        let result = WorkflowService::reject(TransactionStatus::Pending, reason.clone());
        prop_assert!(result.is_ok());
        let action = result.unwrap();
        prop_assert_eq!(action.new_status(), TransactionStatus::Draft);

        // Verify rejection reason
        if let crate::workflow::types::WorkflowAction::Reject { rejection_reason, .. } = action {
            prop_assert_eq!(rejection_reason, reason);
        } else {
            prop_assert!(false, "Expected Reject action");
        }
    }

    /// Approved + post → Posted with correct audit fields
    #[test]
    fn prop_post_from_approved_succeeds(user_id in arb_uuid()) {
        let result = WorkflowService::post(TransactionStatus::Approved, user_id);
        prop_assert!(result.is_ok());
        let action = result.unwrap();
        prop_assert_eq!(action.new_status(), TransactionStatus::Posted);

        // Verify audit fields
        if let crate::workflow::types::WorkflowAction::Post { posted_by, .. } = action {
            prop_assert_eq!(posted_by, user_id);
        } else {
            prop_assert!(false, "Expected Post action");
        }
    }

    /// Posted + void → Voided with correct audit fields
    #[test]
    fn prop_void_from_posted_succeeds(
        user_id in arb_uuid(),
        reason in arb_non_empty_string()
    ) {
        // Skip empty strings after trim
        prop_assume!(!reason.trim().is_empty());

        let result = WorkflowService::void(TransactionStatus::Posted, user_id, reason.clone());
        prop_assert!(result.is_ok());
        let action = result.unwrap();
        prop_assert_eq!(action.new_status(), TransactionStatus::Voided);

        // Verify audit fields
        if let crate::workflow::types::WorkflowAction::Void { voided_by, void_reason, .. } = action {
            prop_assert_eq!(voided_by, user_id);
            prop_assert_eq!(void_reason, reason);
        } else {
            prop_assert!(false, "Expected Void action");
        }
    }

    // =========================================================================
    // Property 2: Invalid State Transitions Rejected
    // Feature: transaction-workflow, Property 2: Invalid State Transitions Rejected
    // Validates: Requirements 1.5, 1.6
    // =========================================================================

    /// Submit from non-Draft status returns InvalidTransition
    #[test]
    fn prop_submit_from_non_draft_fails(
        status in arb_status(),
        user_id in arb_uuid()
    ) {
        prop_assume!(status != TransactionStatus::Draft);

        let result = WorkflowService::submit(status, user_id);
        match result {
            Err(WorkflowError::InvalidTransition { from, to }) => {
                prop_assert_eq!(from, status);
                prop_assert_eq!(to, TransactionStatus::Pending);
            }
            _ => prop_assert!(false, "Expected InvalidTransition error"),
        }
    }

    /// Approve from non-Pending status returns InvalidTransition
    #[test]
    fn prop_approve_from_non_pending_fails(
        status in arb_status(),
        user_id in arb_uuid()
    ) {
        prop_assume!(status != TransactionStatus::Pending);

        let result = WorkflowService::approve(status, user_id, None);
        match result {
            Err(WorkflowError::InvalidTransition { from, to }) => {
                prop_assert_eq!(from, status);
                prop_assert_eq!(to, TransactionStatus::Approved);
            }
            _ => prop_assert!(false, "Expected InvalidTransition error"),
        }
    }

    /// Reject from non-Pending status returns InvalidTransition
    #[test]
    fn prop_reject_from_non_pending_fails(
        status in arb_status(),
        reason in arb_non_empty_string()
    ) {
        prop_assume!(status != TransactionStatus::Pending);
        prop_assume!(!reason.trim().is_empty());

        let result = WorkflowService::reject(status, reason);
        match result {
            Err(WorkflowError::InvalidTransition { from, to }) => {
                prop_assert_eq!(from, status);
                prop_assert_eq!(to, TransactionStatus::Draft);
            }
            _ => prop_assert!(false, "Expected InvalidTransition error"),
        }
    }

    /// Post from non-Approved status returns InvalidTransition
    #[test]
    fn prop_post_from_non_approved_fails(
        status in arb_status(),
        user_id in arb_uuid()
    ) {
        prop_assume!(status != TransactionStatus::Approved);

        let result = WorkflowService::post(status, user_id);
        match result {
            Err(WorkflowError::InvalidTransition { from, to }) => {
                prop_assert_eq!(from, status);
                prop_assert_eq!(to, TransactionStatus::Posted);
            }
            _ => prop_assert!(false, "Expected InvalidTransition error"),
        }
    }

    /// Void from non-Posted status returns InvalidTransition
    #[test]
    fn prop_void_from_non_posted_fails(
        status in arb_status(),
        user_id in arb_uuid(),
        reason in arb_non_empty_string()
    ) {
        prop_assume!(status != TransactionStatus::Posted);
        prop_assume!(!reason.trim().is_empty());

        let result = WorkflowService::void(status, user_id, reason);
        match result {
            Err(WorkflowError::InvalidTransition { from, to }) => {
                prop_assert_eq!(from, status);
                prop_assert_eq!(to, TransactionStatus::Voided);
            }
            _ => prop_assert!(false, "Expected InvalidTransition error"),
        }
    }

    /// is_valid_transition returns true only for valid transitions
    #[test]
    fn prop_is_valid_transition_consistency(
        from in arb_status(),
        to in arb_status()
    ) {
        let is_valid = WorkflowService::is_valid_transition(from, to);

        // Define valid transitions
        let expected_valid = matches!(
            (from, to),
            (TransactionStatus::Draft, TransactionStatus::Pending)
                | (TransactionStatus::Pending, TransactionStatus::Approved)
                | (TransactionStatus::Pending, TransactionStatus::Draft)
                | (TransactionStatus::Approved, TransactionStatus::Posted)
                | (TransactionStatus::Posted, TransactionStatus::Voided)
        );

        prop_assert_eq!(is_valid, expected_valid,
            "is_valid_transition({:?}, {:?}) = {}, expected {}",
            from, to, is_valid, expected_valid);
    }
}

// =========================================================================
// Unit tests for edge cases
// =========================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_reject_empty_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, String::new());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_reject_whitespace_only_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, "   ".to_string());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_reject_tab_only_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, "\t\t".to_string());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_void_empty_reason_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(TransactionStatus::Posted, user_id, String::new());
        assert!(matches!(result, Err(WorkflowError::VoidReasonRequired)));
    }

    #[test]
    fn test_void_whitespace_only_reason_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(TransactionStatus::Posted, user_id, "   ".to_string());
        assert!(matches!(result, Err(WorkflowError::VoidReasonRequired)));
    }

    #[test]
    fn test_void_newline_only_reason_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(TransactionStatus::Posted, user_id, "\n\n".to_string());
        assert!(matches!(result, Err(WorkflowError::VoidReasonRequired)));
    }

    /// Test all 25 combinations of is_valid_transition (5x5 matrix)
    #[test]
    fn test_is_valid_transition_all_combinations() {
        let statuses = [
            TransactionStatus::Draft,
            TransactionStatus::Pending,
            TransactionStatus::Approved,
            TransactionStatus::Posted,
            TransactionStatus::Voided,
        ];

        // Valid transitions
        let valid_transitions = [
            (TransactionStatus::Draft, TransactionStatus::Pending),
            (TransactionStatus::Pending, TransactionStatus::Approved),
            (TransactionStatus::Pending, TransactionStatus::Draft),
            (TransactionStatus::Approved, TransactionStatus::Posted),
            (TransactionStatus::Posted, TransactionStatus::Voided),
        ];

        for from in &statuses {
            for to in &statuses {
                let is_valid = WorkflowService::is_valid_transition(*from, *to);
                let expected = valid_transitions.contains(&(*from, *to));
                assert_eq!(
                    is_valid, expected,
                    "is_valid_transition({:?}, {:?}) = {}, expected {}",
                    from, to, is_valid, expected
                );
            }
        }
    }

    /// Test that same status transitions are invalid
    #[test]
    fn test_same_status_transitions_invalid() {
        let statuses = [
            TransactionStatus::Draft,
            TransactionStatus::Pending,
            TransactionStatus::Approved,
            TransactionStatus::Posted,
            TransactionStatus::Voided,
        ];

        for status in &statuses {
            assert!(
                !WorkflowService::is_valid_transition(*status, *status),
                "Same status transition should be invalid: {:?} -> {:?}",
                status,
                status
            );
        }
    }

    /// Test that voided status cannot transition to anything
    #[test]
    fn test_voided_cannot_transition() {
        let statuses = [
            TransactionStatus::Draft,
            TransactionStatus::Pending,
            TransactionStatus::Approved,
            TransactionStatus::Posted,
            TransactionStatus::Voided,
        ];

        for to in &statuses {
            assert!(
                !WorkflowService::is_valid_transition(TransactionStatus::Voided, *to),
                "Voided should not transition to {:?}",
                to
            );
        }
    }
}
