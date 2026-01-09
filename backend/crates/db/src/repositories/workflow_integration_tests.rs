//! Integration tests for transaction workflow.
//!
//! Tests the full workflow cycle: draft → pending → approved → posted → voided.
//! Validates Requirements 1.1-1.4, 2.1-2.7, 4.1-4.5, 5.1-5.4.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use zeltra_core::workflow::{
        ApprovalEngine, ApprovalRule, OriginalEntry, ReversalInput, ReversalService,
        TransactionStatus, WorkflowError, WorkflowService,
    };

    // ========================================================================
    // Integration Test: Full Workflow Cycle
    // **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 2.1, 2.7**
    // ========================================================================

    /// Strategy for generating user IDs
    fn user_id_strategy() -> impl Strategy<Value = Uuid> {
        any::<[u8; 16]>().prop_map(Uuid::from_bytes)
    }

    /// Strategy for generating non-empty strings (must have at least one non-whitespace char)
    fn non_empty_string_strategy() -> impl Strategy<Value = String> {
        // Ensure at least one alphanumeric character followed by optional chars
        "[a-zA-Z0-9][a-zA-Z0-9 ]{0,49}".prop_map(String::from)
    }

    /// Strategy for generating positive decimal amounts
    fn amount_strategy() -> impl Strategy<Value = Decimal> {
        (1i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Integration Test 1: Happy path workflow cycle**
        ///
        /// *For any* user and transaction, the workflow SHALL progress:
        /// draft → pending → approved → posted
        ///
        /// **Validates: Requirements 1.1, 1.2, 1.4**
        #[test]
        fn prop_happy_path_workflow(
            user_id in user_id_strategy(),
            approval_notes in proptest::option::of(non_empty_string_strategy()),
        ) {
            // Start with Draft
            let status = TransactionStatus::Draft;

            // Step 1: Submit (draft → pending)
            let submit_result = WorkflowService::submit(status, user_id);
            prop_assert!(submit_result.is_ok(), "Submit should succeed from draft");
            let action = submit_result.unwrap();
            prop_assert_eq!(action.new_status(), TransactionStatus::Pending);

            // Step 2: Approve (pending → approved)
            let approve_result = WorkflowService::approve(
                TransactionStatus::Pending,
                user_id,
                approval_notes.clone(),
            );
            prop_assert!(approve_result.is_ok(), "Approve should succeed from pending");
            let action = approve_result.unwrap();
            prop_assert_eq!(action.new_status(), TransactionStatus::Approved);

            // Step 3: Post (approved → posted)
            let post_result = WorkflowService::post(TransactionStatus::Approved, user_id);
            prop_assert!(post_result.is_ok(), "Post should succeed from approved");
            let action = post_result.unwrap();
            prop_assert_eq!(action.new_status(), TransactionStatus::Posted);
        }

        /// **Integration Test 2: Rejection workflow cycle**
        ///
        /// *For any* user and transaction, rejection SHALL return to draft:
        /// draft → pending → draft (rejected) → pending → approved → posted
        ///
        /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4**
        #[test]
        fn prop_rejection_workflow(
            user_id in user_id_strategy(),
            rejection_reason in non_empty_string_strategy(),
        ) {
            // Start with Draft
            let status = TransactionStatus::Draft;

            // Step 1: Submit (draft → pending)
            let submit_result = WorkflowService::submit(status, user_id);
            prop_assert!(submit_result.is_ok());

            // Step 2: Reject (pending → draft)
            let reject_result = WorkflowService::reject(
                TransactionStatus::Pending,
                rejection_reason.clone(),
            );
            prop_assert!(reject_result.is_ok(), "Reject should succeed from pending");
            let action = reject_result.unwrap();
            prop_assert_eq!(action.new_status(), TransactionStatus::Draft);

            // Step 3: Re-submit (draft → pending)
            let resubmit_result = WorkflowService::submit(TransactionStatus::Draft, user_id);
            prop_assert!(resubmit_result.is_ok(), "Re-submit should succeed from draft");

            // Step 4: Approve (pending → approved)
            let approve_result = WorkflowService::approve(
                TransactionStatus::Pending,
                user_id,
                None,
            );
            prop_assert!(approve_result.is_ok());

            // Step 5: Post (approved → posted)
            let post_result = WorkflowService::post(TransactionStatus::Approved, user_id);
            prop_assert!(post_result.is_ok());
            let action = post_result.unwrap();
            prop_assert_eq!(action.new_status(), TransactionStatus::Posted);
        }

        /// **Integration Test 3: Void workflow**
        ///
        /// *For any* posted transaction, void SHALL create reversing entries:
        /// posted → voided (with reversing transaction)
        ///
        /// **Validates: Requirements 2.1, 2.5, 2.7**
        #[test]
        fn prop_void_workflow(
            user_id in user_id_strategy(),
            void_reason in non_empty_string_strategy(),
        ) {
            // Start with Posted
            let status = TransactionStatus::Posted;

            // Void (posted → voided)
            let void_result = WorkflowService::void(status, user_id, void_reason.clone());
            prop_assert!(void_result.is_ok(), "Void should succeed from posted");
            let action = void_result.unwrap();
            prop_assert_eq!(action.new_status(), TransactionStatus::Voided);
        }
    }

    // ========================================================================
    // Integration Test: Reversing Entry Balance
    // **Validates: Requirements 2.1, 2.7**
    // ========================================================================

    /// Strategy for generating balanced entry sets
    fn balanced_entries_strategy() -> impl Strategy<Value = Vec<OriginalEntry>> {
        // Generate a debit amount, then create matching debit and credit entries
        (1i64..100_000i64, user_id_strategy(), user_id_strategy()).prop_map(
            |(amount_cents, debit_account, credit_account)| {
                let amount = Decimal::new(amount_cents, 2);
                vec![
                    OriginalEntry {
                        account_id: debit_account,
                        source_currency: "USD".to_string(),
                        source_amount: amount,
                        exchange_rate: Decimal::ONE,
                        functional_amount: amount,
                        debit: amount,
                        credit: Decimal::ZERO,
                        memo: Some("Test debit".to_string()),
                        dimensions: vec![],
                    },
                    OriginalEntry {
                        account_id: credit_account,
                        source_currency: "USD".to_string(),
                        source_amount: amount,
                        exchange_rate: Decimal::ONE,
                        functional_amount: amount,
                        debit: Decimal::ZERO,
                        credit: amount,
                        memo: Some("Test credit".to_string()),
                        dimensions: vec![],
                    },
                ]
            },
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 3: Void Creates Balanced Reversing Entry**
        ///
        /// *For any* balanced original entries, reversing entries SHALL also be balanced.
        ///
        /// **Validates: Requirements 2.1, 2.7**
        #[test]
        fn prop_reversing_entries_balanced(
            entries in balanced_entries_strategy(),
            user_id in user_id_strategy(),
            void_reason in non_empty_string_strategy(),
        ) {
            let transaction_id = Uuid::new_v4();
            let fiscal_period_id = Uuid::new_v4();

            // Verify original entries are balanced
            let original_debit: Decimal = entries.iter().map(|e| e.debit).sum();
            let original_credit: Decimal = entries.iter().map(|e| e.credit).sum();
            prop_assert_eq!(original_debit, original_credit, "Original entries should be balanced");

            // Create reversal input
            let input = ReversalInput {
                original_transaction_id: transaction_id,
                original_entries: entries.clone(),
                fiscal_period_id,
                voided_by: user_id,
                void_reason,
            };

            // Generate reversing entries
            let output = ReversalService::create_reversing_entries(&input);

            // Verify reversing entries are balanced
            let reversing_debit: Decimal = output.reversing_entries.iter()
                .filter(|e| matches!(e.entry_type, zeltra_core::ledger::types::EntryType::Debit))
                .map(|e| e.source_amount)
                .sum();
            let reversing_credit: Decimal = output.reversing_entries.iter()
                .filter(|e| matches!(e.entry_type, zeltra_core::ledger::types::EntryType::Credit))
                .map(|e| e.source_amount)
                .sum();

            prop_assert_eq!(reversing_debit, reversing_credit, "Reversing entries should be balanced");

            // Verify debit/credit swap
            prop_assert_eq!(reversing_debit, original_credit, "Reversing debits should equal original credits");
            prop_assert_eq!(reversing_credit, original_debit, "Reversing credits should equal original debits");
        }

        /// **Property 4: Reversing entries preserve amounts**
        ///
        /// *For any* original entry, the reversing entry SHALL have the same amount.
        ///
        /// **Validates: Requirements 2.1**
        #[test]
        fn prop_reversing_entries_preserve_amounts(
            entries in balanced_entries_strategy(),
            user_id in user_id_strategy(),
            void_reason in non_empty_string_strategy(),
        ) {
            let input = ReversalInput {
                original_transaction_id: Uuid::new_v4(),
                original_entries: entries.clone(),
                fiscal_period_id: Uuid::new_v4(),
                voided_by: user_id,
                void_reason,
            };

            let output = ReversalService::create_reversing_entries(&input);

            // Each reversing entry should have same amount as original
            prop_assert_eq!(
                output.reversing_entries.len(),
                entries.len(),
                "Should have same number of reversing entries"
            );

            for (original, reversing) in entries.iter().zip(output.reversing_entries.iter()) {
                prop_assert_eq!(
                    original.source_amount,
                    reversing.source_amount,
                    "Reversing entry should preserve source amount"
                );
                prop_assert_eq!(
                    original.account_id,
                    reversing.account_id,
                    "Reversing entry should preserve account"
                );
            }
        }
    }

    // ========================================================================
    // Integration Test: Approval Queue
    // **Validates: Requirements 5.1, 5.2, 5.3, 5.4**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 8: Approval Rule Priority Ordering**
        ///
        /// *For any* set of rules with different priorities, the lowest priority SHALL be selected.
        ///
        /// **Validates: Requirements 3.2, 3.3**
        #[test]
        fn prop_approval_rule_priority(
            amount in amount_strategy(),
        ) {
            // Create rules with different priorities
            let rules = vec![
                ApprovalRule {
                    id: Uuid::new_v4(),
                    name: "High priority".to_string(),
                    min_amount: None,
                    max_amount: None,
                    transaction_types: vec!["journal".to_string()],
                    required_role: "admin".to_string(),
                    priority: 10,
                },
                ApprovalRule {
                    id: Uuid::new_v4(),
                    name: "Low priority".to_string(),
                    min_amount: None,
                    max_amount: None,
                    transaction_types: vec!["journal".to_string()],
                    required_role: "approver".to_string(),
                    priority: 1,
                },
                ApprovalRule {
                    id: Uuid::new_v4(),
                    name: "Medium priority".to_string(),
                    min_amount: None,
                    max_amount: None,
                    transaction_types: vec!["journal".to_string()],
                    required_role: "accountant".to_string(),
                    priority: 5,
                },
            ];

            let result = ApprovalEngine::get_required_approval(&rules, "journal", amount);
            prop_assert!(result.is_some(), "Should find a matching rule");
            prop_assert_eq!(result.unwrap(), "approver", "Should select lowest priority rule");
        }

        /// **Property 9: Role Hierarchy Enforcement**
        ///
        /// *For any* user role and required role, approval SHALL be allowed iff user_role >= required_role.
        ///
        /// **Validates: Requirements 3.4, 3.6**
        #[test]
        fn prop_role_hierarchy(
            amount in amount_strategy(),
        ) {
            // Owner can approve anything
            let owner_result = ApprovalEngine::can_approve("owner", None, "admin", amount);
            prop_assert!(owner_result.is_ok(), "Owner should approve admin-required");

            // Admin can approve approver-required
            let admin_result = ApprovalEngine::can_approve("admin", None, "approver", amount);
            prop_assert!(admin_result.is_ok(), "Admin should approve approver-required");

            // Approver cannot approve admin-required
            let approver_result = ApprovalEngine::can_approve("approver", Some(dec!(1000000)), "admin", amount);
            prop_assert!(approver_result.is_err(), "Approver should not approve admin-required");

            // Viewer cannot approve anything
            let viewer_result = ApprovalEngine::can_approve("viewer", None, "approver", amount);
            prop_assert!(viewer_result.is_err(), "Viewer should not approve anything");
        }

        /// **Property 10: Approval Limit Enforcement**
        ///
        /// *For any* Approver role, approval SHALL be rejected when amount > limit.
        ///
        /// **Validates: Requirements 3.5**
        #[test]
        fn prop_approval_limit(
            limit_cents in 1000i64..100_000i64,
        ) {
            let limit = Decimal::new(limit_cents, 2);
            let under_limit = limit - dec!(1);
            let over_limit = limit + dec!(1);

            // Under limit should succeed
            let under_result = ApprovalEngine::can_approve("approver", Some(limit), "approver", under_limit);
            prop_assert!(under_result.is_ok(), "Under limit should succeed");

            // At limit should succeed
            let at_result = ApprovalEngine::can_approve("approver", Some(limit), "approver", limit);
            prop_assert!(at_result.is_ok(), "At limit should succeed");

            // Over limit should fail
            let over_result = ApprovalEngine::can_approve("approver", Some(limit), "approver", over_limit);
            prop_assert!(over_result.is_err(), "Over limit should fail");

            // Admin bypasses limit
            let admin_result = ApprovalEngine::can_approve("admin", None, "approver", over_limit);
            prop_assert!(admin_result.is_ok(), "Admin should bypass limit");
        }
    }

    // ========================================================================
    // Integration Test: Immutability via Workflow
    // **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**
    // ========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 6: Posted Transactions Are Immutable**
        ///
        /// *For any* posted transaction, only void is allowed.
        ///
        /// **Validates: Requirements 4.1, 4.2**
        #[test]
        fn prop_posted_immutable(
            user_id in user_id_strategy(),
        ) {
            let status = TransactionStatus::Posted;

            // Submit should fail
            let submit_result = WorkflowService::submit(status, user_id);
            prop_assert!(submit_result.is_err(), "Submit should fail for posted");

            // Approve should fail
            let approve_result = WorkflowService::approve(status, user_id, None);
            prop_assert!(approve_result.is_err(), "Approve should fail for posted");

            // Reject should fail
            let reject_result = WorkflowService::reject(status, "reason".to_string());
            prop_assert!(reject_result.is_err(), "Reject should fail for posted");

            // Post should fail
            let post_result = WorkflowService::post(status, user_id);
            prop_assert!(post_result.is_err(), "Post should fail for posted");

            // Void should succeed
            let void_result = WorkflowService::void(status, user_id, "reason".to_string());
            prop_assert!(void_result.is_ok(), "Void should succeed for posted");
        }

        /// **Property 7: Voided Transactions Are Immutable**
        ///
        /// *For any* voided transaction, no operations are allowed.
        ///
        /// **Validates: Requirements 4.3, 4.4**
        #[test]
        fn prop_voided_immutable(
            user_id in user_id_strategy(),
        ) {
            let status = TransactionStatus::Voided;

            // All operations should fail
            let submit_result = WorkflowService::submit(status, user_id);
            prop_assert!(submit_result.is_err(), "Submit should fail for voided");

            let approve_result = WorkflowService::approve(status, user_id, None);
            prop_assert!(approve_result.is_err(), "Approve should fail for voided");

            let reject_result = WorkflowService::reject(status, "reason".to_string());
            prop_assert!(reject_result.is_err(), "Reject should fail for voided");

            let post_result = WorkflowService::post(status, user_id);
            prop_assert!(post_result.is_err(), "Post should fail for voided");

            let void_result = WorkflowService::void(status, user_id, "reason".to_string());
            prop_assert!(void_result.is_err(), "Void should fail for voided");
        }

        /// **Property 7b: Draft/Pending Transactions Are Mutable**
        ///
        /// *For any* draft or pending transaction, appropriate operations are allowed.
        ///
        /// **Validates: Requirements 4.5**
        #[test]
        fn prop_draft_pending_mutable(
            user_id in user_id_strategy(),
        ) {
            // Draft can be submitted
            let submit_result = WorkflowService::submit(TransactionStatus::Draft, user_id);
            prop_assert!(submit_result.is_ok(), "Submit should succeed for draft");

            // Pending can be approved
            let approve_result = WorkflowService::approve(TransactionStatus::Pending, user_id, None);
            prop_assert!(approve_result.is_ok(), "Approve should succeed for pending");

            // Pending can be rejected
            let reject_result = WorkflowService::reject(TransactionStatus::Pending, "reason".to_string());
            prop_assert!(reject_result.is_ok(), "Reject should succeed for pending");
        }
    }

    // ========================================================================
    // Unit Tests: Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_rejection_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, String::new());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_whitespace_rejection_reason_fails() {
        let result = WorkflowService::reject(TransactionStatus::Pending, "   ".to_string());
        assert!(matches!(
            result,
            Err(WorkflowError::RejectionReasonRequired)
        ));
    }

    #[test]
    fn test_empty_void_reason_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(TransactionStatus::Posted, user_id, String::new());
        assert!(matches!(result, Err(WorkflowError::VoidReasonRequired)));
    }

    #[test]
    fn test_whitespace_void_reason_fails() {
        let user_id = Uuid::new_v4();
        let result = WorkflowService::void(TransactionStatus::Posted, user_id, "   ".to_string());
        assert!(matches!(result, Err(WorkflowError::VoidReasonRequired)));
    }

    #[test]
    fn test_full_workflow_cycle() {
        let user_id = Uuid::new_v4();

        // Draft → Pending
        let submit = WorkflowService::submit(TransactionStatus::Draft, user_id).unwrap();
        assert_eq!(submit.new_status(), TransactionStatus::Pending);

        // Pending → Approved
        let approve = WorkflowService::approve(TransactionStatus::Pending, user_id, None).unwrap();
        assert_eq!(approve.new_status(), TransactionStatus::Approved);

        // Approved → Posted
        let post = WorkflowService::post(TransactionStatus::Approved, user_id).unwrap();
        assert_eq!(post.new_status(), TransactionStatus::Posted);

        // Posted → Voided
        let void =
            WorkflowService::void(TransactionStatus::Posted, user_id, "Test void".to_string())
                .unwrap();
        assert_eq!(void.new_status(), TransactionStatus::Voided);
    }

    #[test]
    fn test_rejection_cycle() {
        let user_id = Uuid::new_v4();

        // Draft → Pending
        let submit = WorkflowService::submit(TransactionStatus::Draft, user_id).unwrap();
        assert_eq!(submit.new_status(), TransactionStatus::Pending);

        // Pending → Draft (rejected)
        let reject =
            WorkflowService::reject(TransactionStatus::Pending, "Needs revision".to_string())
                .unwrap();
        assert_eq!(reject.new_status(), TransactionStatus::Draft);

        // Draft → Pending (re-submit)
        let resubmit = WorkflowService::submit(TransactionStatus::Draft, user_id).unwrap();
        assert_eq!(resubmit.new_status(), TransactionStatus::Pending);
    }

    #[test]
    fn test_reversing_entry_memo_prefix() {
        let entries = vec![OriginalEntry {
            account_id: Uuid::new_v4(),
            source_currency: "USD".to_string(),
            source_amount: dec!(100),
            exchange_rate: Decimal::ONE,
            functional_amount: dec!(100),
            debit: dec!(100),
            credit: Decimal::ZERO,
            memo: Some("Original memo".to_string()),
            dimensions: vec![],
        }];

        let input = ReversalInput {
            original_transaction_id: Uuid::new_v4(),
            original_entries: entries,
            fiscal_period_id: Uuid::new_v4(),
            voided_by: Uuid::new_v4(),
            void_reason: "Test".to_string(),
        };

        let output = ReversalService::create_reversing_entries(&input);
        let reversing_memo = output.reversing_entries[0].memo.as_ref().unwrap();
        assert!(
            reversing_memo.starts_with("Reversal: "),
            "Memo should have Reversal prefix"
        );
    }

    #[test]
    fn test_approval_rule_no_match() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Journal only".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec!["journal".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Expense type should not match
        let result = ApprovalEngine::get_required_approval(&rules, "expense", dec!(100));
        assert!(result.is_none(), "Should not match expense type");
    }

    #[test]
    fn test_approval_rule_amount_boundary() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Small amounts".to_string(),
            min_amount: Some(dec!(0)),
            max_amount: Some(dec!(1000)),
            transaction_types: vec!["journal".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // At max boundary should match
        let at_max = ApprovalEngine::get_required_approval(&rules, "journal", dec!(1000));
        assert!(at_max.is_some(), "Should match at max boundary");

        // Over max should not match
        let over_max = ApprovalEngine::get_required_approval(&rules, "journal", dec!(1001));
        assert!(over_max.is_none(), "Should not match over max");

        // At min boundary should match
        let at_min = ApprovalEngine::get_required_approval(&rules, "journal", dec!(0));
        assert!(at_min.is_some(), "Should match at min boundary");
    }
}
