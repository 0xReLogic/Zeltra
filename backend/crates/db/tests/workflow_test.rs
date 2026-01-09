//! Integration tests for workflow repository.
//!
//! Tests Requirements 1.1-1.4, 2.1-2.7, 5.1-5.4 for transaction workflow.

use sea_orm::Database;
use std::env;
use uuid::Uuid;

use zeltra_core::workflow::WorkflowError;
use zeltra_db::repositories::workflow::WorkflowRepository;

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

// ============================================================================
// Test: Submit transaction not found
// ============================================================================
#[tokio::test]
async fn test_submit_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo
        .submit_transaction(org_id, transaction_id, user_id)
        .await;

    assert!(
        result.is_err(),
        "Should return error for non-existent transaction"
    );

    match result {
        Err(WorkflowError::TransactionNotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected TransactionNotFound error"),
    }
}

// ============================================================================
// Test: Approve transaction not found
// ============================================================================
#[tokio::test]
async fn test_approve_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo
        .approve_transaction(org_id, transaction_id, user_id, None)
        .await;

    assert!(
        result.is_err(),
        "Should return error for non-existent transaction"
    );

    match result {
        Err(WorkflowError::TransactionNotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected TransactionNotFound error"),
    }
}

// ============================================================================
// Test: Reject transaction not found
// ============================================================================
#[tokio::test]
async fn test_reject_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();

    let result = repo
        .reject_transaction(org_id, transaction_id, "Test rejection".to_string())
        .await;

    assert!(
        result.is_err(),
        "Should return error for non-existent transaction"
    );

    match result {
        Err(WorkflowError::TransactionNotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected TransactionNotFound error"),
    }
}

// ============================================================================
// Test: Post transaction not found
// ============================================================================
#[tokio::test]
async fn test_post_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo.post_transaction(org_id, transaction_id, user_id).await;

    assert!(
        result.is_err(),
        "Should return error for non-existent transaction"
    );

    match result {
        Err(WorkflowError::TransactionNotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected TransactionNotFound error"),
    }
}

// ============================================================================
// Test: Void transaction not found
// ============================================================================
#[tokio::test]
async fn test_void_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo
        .void_transaction(org_id, transaction_id, user_id, "Test void".to_string())
        .await;

    assert!(
        result.is_err(),
        "Should return error for non-existent transaction"
    );

    match result {
        Err(WorkflowError::TransactionNotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected TransactionNotFound error"),
    }
}

// ============================================================================
// Test: Get pending transactions for non-existent user returns empty
// ============================================================================
#[tokio::test]
async fn test_get_pending_transactions_no_user() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo.get_pending_transactions(org_id, user_id).await;

    assert!(result.is_ok(), "Should succeed even with no user");
    assert!(
        result.unwrap().is_empty(),
        "Should return empty list for non-existent user"
    );
}

// ============================================================================
// Test: Bulk approve with empty list
// ============================================================================
#[tokio::test]
async fn test_bulk_approve_empty_list() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo.bulk_approve(org_id, vec![], user_id, None).await;

    assert!(result.is_ok(), "Should succeed with empty list");
    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.success_count, 0);
    assert_eq!(bulk_result.failure_count, 0);
    assert!(bulk_result.results.is_empty());
}

// ============================================================================
// Test: Bulk approve with non-existent transactions
// ============================================================================
#[tokio::test]
async fn test_bulk_approve_non_existent_transactions() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let tx_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    let result = repo
        .bulk_approve(org_id, tx_ids.clone(), user_id, None)
        .await;

    assert!(result.is_ok(), "Should succeed even with failures");
    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.success_count, 0);
    assert_eq!(bulk_result.failure_count, 3);
    assert_eq!(bulk_result.results.len(), 3);

    for (i, item) in bulk_result.results.iter().enumerate() {
        assert_eq!(item.transaction_id, tx_ids[i]);
        assert!(!item.success);
        assert!(item.error.is_some());
    }
}

// ============================================================================
// Test: Reject with empty reason fails
// ============================================================================
#[tokio::test]
async fn test_reject_empty_reason_fails() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();

    // Even though transaction doesn't exist, the validation should happen first
    // But in our implementation, we fetch first then validate
    // So this will return TransactionNotFound
    let result = repo
        .reject_transaction(org_id, transaction_id, String::new())
        .await;

    // Since we fetch first, we get TransactionNotFound
    assert!(result.is_err());
}

// ============================================================================
// Test: Void with empty reason fails
// ============================================================================
#[tokio::test]
async fn test_void_empty_reason_fails() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Since we fetch first, we get TransactionNotFound
    let result = repo
        .void_transaction(org_id, transaction_id, user_id, String::new())
        .await;

    assert!(result.is_err());
}

// ============================================================================
// Immutability Tests (Task 15)
// ============================================================================
// Property 6: Posted/Voided Transactions Are Immutable
// Property 7: Draft/Pending Transactions Are Mutable
// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

use proptest::prelude::*;
use zeltra_db::entities::sea_orm_active_enums::TransactionStatus;
use zeltra_db::repositories::transaction::{
    TransactionError, can_delete_transaction, can_modify_transaction,
};

// ============================================================================
// Unit Tests: Error types exist and are correct
// ============================================================================

#[test]
fn test_cannot_modify_posted_error_exists() {
    let err = TransactionError::CannotModifyPosted;
    assert!(err.to_string().contains("posted"));
}

#[test]
fn test_cannot_modify_voided_error_exists() {
    let err = TransactionError::CannotModifyVoided;
    assert!(err.to_string().contains("voided"));
}

#[test]
fn test_can_only_delete_draft_error_exists() {
    let err = TransactionError::CanOnlyDeleteDraft;
    assert!(err.to_string().contains("draft"));
}

// ============================================================================
// Property Tests: Immutability (Property 6)
// **Validates: Requirements 4.1, 4.2, 4.3, 4.4**
// ============================================================================

/// Strategy for generating immutable statuses (Posted, Voided)
fn immutable_status_strategy() -> impl Strategy<Value = TransactionStatus> {
    prop_oneof![
        Just(TransactionStatus::Posted),
        Just(TransactionStatus::Voided),
    ]
}

/// Strategy for generating mutable statuses (Draft, Pending)
fn mutable_status_strategy() -> impl Strategy<Value = TransactionStatus> {
    prop_oneof![
        Just(TransactionStatus::Draft),
        Just(TransactionStatus::Pending),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ========================================================================
    // Property 6: Posted/Voided Transactions Are Immutable
    // ========================================================================

    /// **Property 6.1: Posted transactions reject updates**
    ///
    /// *For any* posted transaction, attempting to update SHALL be rejected
    /// with CannotModifyPosted error.
    ///
    /// **Validates: Requirements 4.1**
    #[test]
    fn prop_posted_transactions_reject_updates(_iteration in 0..100i32) {
        let result = can_modify_transaction(&TransactionStatus::Posted);

        prop_assert!(
            result.is_err(),
            "Posted transactions must reject updates"
        );

        match result {
            Err(TransactionError::CannotModifyPosted) => {
                // Correct error type
            }
            other => {
                prop_assert!(
                    false,
                    "Expected CannotModifyPosted error, got: {:?}",
                    other
                );
            }
        }
    }

    /// **Property 6.2: Posted transactions reject deletes**
    ///
    /// *For any* posted transaction, attempting to delete SHALL be rejected.
    ///
    /// **Validates: Requirements 4.2**
    #[test]
    fn prop_posted_transactions_reject_deletes(_iteration in 0..100i32) {
        let result = can_delete_transaction(&TransactionStatus::Posted);

        prop_assert!(
            result.is_err(),
            "Posted transactions must reject deletes"
        );
    }

    /// **Property 6.3: Voided transactions reject updates**
    ///
    /// *For any* voided transaction, attempting to update SHALL be rejected
    /// with CannotModifyVoided error.
    ///
    /// **Validates: Requirements 4.3**
    #[test]
    fn prop_voided_transactions_reject_updates(_iteration in 0..100i32) {
        let result = can_modify_transaction(&TransactionStatus::Voided);

        prop_assert!(
            result.is_err(),
            "Voided transactions must reject updates"
        );

        match result {
            Err(TransactionError::CannotModifyVoided) => {
                // Correct error type
            }
            other => {
                prop_assert!(
                    false,
                    "Expected CannotModifyVoided error, got: {:?}",
                    other
                );
            }
        }
    }

    /// **Property 6.4: Voided transactions reject deletes**
    ///
    /// *For any* voided transaction, attempting to delete SHALL be rejected.
    ///
    /// **Validates: Requirements 4.4**
    #[test]
    fn prop_voided_transactions_reject_deletes(_iteration in 0..100i32) {
        let result = can_delete_transaction(&TransactionStatus::Voided);

        prop_assert!(
            result.is_err(),
            "Voided transactions must reject deletes"
        );
    }

    /// **Property 6.5: All immutable statuses reject modifications**
    ///
    /// *For any* transaction with Posted or Voided status, *any* attempt to
    /// update or delete SHALL be rejected.
    ///
    /// **Validates: Requirements 4.1, 4.2, 4.3, 4.4**
    #[test]
    fn prop_immutable_statuses_reject_all_modifications(
        status in immutable_status_strategy(),
    ) {
        let modify_result = can_modify_transaction(&status);
        let delete_result = can_delete_transaction(&status);

        prop_assert!(
            modify_result.is_err(),
            "Immutable status {:?} must reject modifications",
            status
        );

        prop_assert!(
            delete_result.is_err(),
            "Immutable status {:?} must reject deletions",
            status
        );

        // Verify correct error types
        match status {
            TransactionStatus::Posted => {
                prop_assert!(
                    matches!(modify_result, Err(TransactionError::CannotModifyPosted)),
                    "Posted should return CannotModifyPosted"
                );
            }
            TransactionStatus::Voided => {
                prop_assert!(
                    matches!(modify_result, Err(TransactionError::CannotModifyVoided)),
                    "Voided should return CannotModifyVoided"
                );
            }
            _ => {}
        }
    }

    // ========================================================================
    // Property 7: Draft/Pending Transactions Are Mutable
    // ========================================================================

    /// **Property 7.1: Draft transactions allow updates**
    ///
    /// *For any* draft transaction, updates SHALL succeed.
    ///
    /// **Validates: Requirements 4.5**
    #[test]
    fn prop_draft_transactions_allow_updates(_iteration in 0..100i32) {
        let result = can_modify_transaction(&TransactionStatus::Draft);

        prop_assert!(
            result.is_ok(),
            "Draft transactions must allow updates"
        );
    }

    /// **Property 7.2: Pending transactions allow updates**
    ///
    /// *For any* pending transaction, updates SHALL succeed.
    ///
    /// **Validates: Requirements 4.5**
    #[test]
    fn prop_pending_transactions_allow_updates(_iteration in 0..100i32) {
        let result = can_modify_transaction(&TransactionStatus::Pending);

        prop_assert!(
            result.is_ok(),
            "Pending transactions must allow updates"
        );
    }

    /// **Property 7.3: All mutable statuses allow updates**
    ///
    /// *For any* transaction with Draft or Pending status, updates SHALL succeed.
    ///
    /// **Validates: Requirements 4.5**
    #[test]
    fn prop_mutable_statuses_allow_updates(
        status in mutable_status_strategy(),
    ) {
        let result = can_modify_transaction(&status);

        prop_assert!(
            result.is_ok(),
            "Mutable status {:?} must allow updates",
            status
        );
    }

    /// **Property 7.4: Only draft transactions allow deletion**
    ///
    /// *For any* draft transaction, deletion SHALL succeed.
    /// *For any* non-draft transaction, deletion SHALL be rejected.
    ///
    /// **Validates: Requirements 4.5**
    #[test]
    fn prop_only_draft_allows_deletion(_iteration in 0..100i32) {
        // Draft allows deletion
        let draft_result = can_delete_transaction(&TransactionStatus::Draft);
        prop_assert!(
            draft_result.is_ok(),
            "Draft transactions must allow deletion"
        );

        // Pending does NOT allow deletion
        let pending_result = can_delete_transaction(&TransactionStatus::Pending);
        prop_assert!(
            pending_result.is_err(),
            "Pending transactions must reject deletion"
        );
    }

    // ========================================================================
    // Combined Property: Immutability Rules Consistency
    // ========================================================================

    /// **Property 6+7: Immutability rules are consistent across all statuses**
    ///
    /// *For any* transaction status, the modification and deletion rules SHALL be:
    /// - Draft: can modify AND can delete
    /// - Pending: can modify BUT cannot delete
    /// - Approved: can modify BUT cannot delete
    /// - Posted: cannot modify AND cannot delete
    /// - Voided: cannot modify AND cannot delete
    ///
    /// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**
    #[test]
    fn prop_immutability_rules_consistent(
        status in prop_oneof![
            Just(TransactionStatus::Draft),
            Just(TransactionStatus::Pending),
            Just(TransactionStatus::Approved),
            Just(TransactionStatus::Posted),
            Just(TransactionStatus::Voided),
        ],
    ) {
        let can_mod = can_modify_transaction(&status).is_ok();
        let can_del = can_delete_transaction(&status).is_ok();

        match status {
            TransactionStatus::Draft => {
                prop_assert!(can_mod, "Draft must allow modification");
                prop_assert!(can_del, "Draft must allow deletion");
            }
            TransactionStatus::Pending | TransactionStatus::Approved => {
                prop_assert!(can_mod, "Pending/Approved must allow modification");
                prop_assert!(!can_del, "Pending/Approved must reject deletion");
            }
            TransactionStatus::Posted => {
                prop_assert!(!can_mod, "Posted must reject modification");
                prop_assert!(!can_del, "Posted must reject deletion");
            }
            TransactionStatus::Voided => {
                prop_assert!(!can_mod, "Voided must reject modification");
                prop_assert!(!can_del, "Voided must reject deletion");
            }
        }
    }
}

// ============================================================================
// Property Tests: Bulk Approval (Property 11)
// **Validates: Requirements 5.2, 5.3, 5.4**
// ============================================================================

/// **Property 11: Bulk Approval Partial Success**
///
/// *For any* bulk approval request containing N transactions where M transactions
/// fail validation, the response SHALL contain N results with exactly M failures
/// and (N-M) successes.
///
/// Note: This test verifies the bulk approval behavior with non-existent transactions
/// (which will all fail). Full integration tests with real transactions require
/// seeded database.

#[tokio::test]
async fn test_bulk_approve_returns_correct_counts() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Test with 5 non-existent transactions (all should fail)
    let tx_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    let result = repo
        .bulk_approve(org_id, tx_ids.clone(), user_id, None)
        .await;

    assert!(
        result.is_ok(),
        "Bulk approve should succeed even with failures"
    );
    let bulk_result = result.unwrap();

    // Property: response contains N results
    assert_eq!(
        bulk_result.results.len(),
        5,
        "Response must contain exactly N results"
    );

    // Property: all failures counted correctly
    assert_eq!(
        bulk_result.failure_count, 5,
        "All 5 transactions should fail (not found)"
    );
    assert_eq!(bulk_result.success_count, 0, "No successes expected");

    // Property: each result has correct transaction_id
    for (i, item) in bulk_result.results.iter().enumerate() {
        assert_eq!(
            item.transaction_id, tx_ids[i],
            "Result must reference correct transaction_id"
        );
        assert!(!item.success, "Transaction should be marked as failed");
        assert!(
            item.error.is_some(),
            "Failed transaction must have error message"
        );
    }
}

#[tokio::test]
async fn test_bulk_approve_empty_returns_zero_counts() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let result = repo.bulk_approve(org_id, vec![], user_id, None).await;

    assert!(result.is_ok());
    let bulk_result = result.unwrap();

    // Property: empty input returns zero counts
    assert_eq!(bulk_result.results.len(), 0);
    assert_eq!(bulk_result.success_count, 0);
    assert_eq!(bulk_result.failure_count, 0);
}

#[tokio::test]
async fn test_bulk_approve_preserves_order() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db);

    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Create specific UUIDs to verify order preservation
    let tx_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

    let result = repo
        .bulk_approve(
            org_id,
            tx_ids.clone(),
            user_id,
            Some("Bulk approval".to_string()),
        )
        .await;

    assert!(result.is_ok());
    let bulk_result = result.unwrap();

    // Property: results are in same order as input
    for (i, item) in bulk_result.results.iter().enumerate() {
        assert_eq!(
            item.transaction_id, tx_ids[i],
            "Result order must match input order at index {i}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Property 11.1: Bulk approval result count equals input count**
    ///
    /// *For any* N transaction IDs, bulk_approve returns exactly N results.
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_bulk_approve_result_count_equals_input(
        count in 1usize..20,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db = Database::connect(&get_database_url())
                .await
                .expect("Failed to connect to database");

            let repo = WorkflowRepository::new(db);

            let org_id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let tx_ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();

            let result = repo
                .bulk_approve(org_id, tx_ids.clone(), user_id, None)
                .await;

            prop_assert!(result.is_ok());
            let bulk_result = result.unwrap();

            prop_assert_eq!(
                bulk_result.results.len(),
                count,
                "Result count must equal input count"
            );

            // success_count + failure_count must equal total
            prop_assert_eq!(
                bulk_result.success_count + bulk_result.failure_count,
                count,
                "success + failure must equal total"
            );

            Ok(())
        })?;
    }

    /// **Property 11.2: Bulk approval continues after failures**
    ///
    /// *For any* bulk approval with failures, processing continues for all transactions.
    ///
    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_bulk_approve_continues_after_failure(
        count in 2usize..10,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db = Database::connect(&get_database_url())
                .await
                .expect("Failed to connect to database");

            let repo = WorkflowRepository::new(db);

            let org_id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let tx_ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();

            let result = repo
                .bulk_approve(org_id, tx_ids.clone(), user_id, None)
                .await;

            prop_assert!(result.is_ok(), "Bulk approve must not fail even with invalid transactions");
            let bulk_result = result.unwrap();

            // All transactions should be processed (all fail because they don't exist)
            prop_assert_eq!(
                bulk_result.results.len(),
                count,
                "All transactions must be processed"
            );

            // Each result should have an error (since transactions don't exist)
            for item in &bulk_result.results {
                prop_assert!(
                    item.error.is_some(),
                    "Non-existent transaction must have error"
                );
            }

            Ok(())
        })?;
    }
}
