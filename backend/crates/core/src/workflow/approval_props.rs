//! Property-based tests for ApprovalEngine.
//!
//! These tests validate the correctness properties defined in the design document
//! for approval rules matching and user authorization.

use proptest::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::workflow::approval::{ApprovalEngine, ApprovalRule, UserRole};
use crate::workflow::error::WorkflowError;

/// Strategy for generating random positive Decimal amounts.
fn arb_amount() -> impl Strategy<Value = Decimal> {
    (1i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
}

/// Strategy for generating random UserRole values.
fn arb_user_role() -> impl Strategy<Value = UserRole> {
    prop_oneof![
        Just(UserRole::Viewer),
        Just(UserRole::Submitter),
        Just(UserRole::Approver),
        Just(UserRole::Accountant),
        Just(UserRole::Admin),
        Just(UserRole::Owner),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // =========================================================================
    // Property 8: Approval Rule Priority Ordering
    // Feature: transaction-workflow, Property 8: Approval Rule Priority Ordering
    // Validates: Requirements 3.2, 3.3
    // =========================================================================

    /// When multiple rules match, the one with lowest priority value is selected
    #[test]
    fn prop_rule_priority_ordering(
        amount in arb_amount()
    ) {
        // Create rules with different priorities that all match
        let rules = vec![
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Low Priority".to_string(),
                min_amount: None,
                max_amount: None,
                transaction_types: vec!["expense".to_string()],
                required_role: "admin".to_string(),
                priority: 10,
            },
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "High Priority".to_string(),
                min_amount: None,
                max_amount: None,
                transaction_types: vec!["expense".to_string()],
                required_role: "approver".to_string(),
                priority: 1,
            },
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Medium Priority".to_string(),
                min_amount: None,
                max_amount: None,
                transaction_types: vec!["expense".to_string()],
                required_role: "accountant".to_string(),
                priority: 5,
            },
        ];

        let result = ApprovalEngine::get_required_approval(&rules, "expense", amount);

        // Should always select the rule with priority 1 (approver)
        prop_assert_eq!(result, Some("approver".to_string()));
    }

    /// Rules are filtered by amount range correctly
    #[test]
    fn prop_rule_amount_range_filtering(
        amount in 100i64..500i64
    ) {
        let amount = Decimal::new(amount, 0);

        let rules = vec![
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Small".to_string(),
                min_amount: None,
                max_amount: Some(Decimal::new(99, 0)),
                transaction_types: vec!["expense".to_string()],
                required_role: "submitter".to_string(),
                priority: 1,
            },
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Medium".to_string(),
                min_amount: Some(Decimal::new(100, 0)),
                max_amount: Some(Decimal::new(500, 0)),
                transaction_types: vec!["expense".to_string()],
                required_role: "approver".to_string(),
                priority: 1,
            },
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Large".to_string(),
                min_amount: Some(Decimal::new(501, 0)),
                max_amount: None,
                transaction_types: vec!["expense".to_string()],
                required_role: "admin".to_string(),
                priority: 1,
            },
        ];

        let result = ApprovalEngine::get_required_approval(&rules, "expense", amount);

        // Amount 100-500 should match "Medium" rule
        prop_assert_eq!(result, Some("approver".to_string()));
    }

    // =========================================================================
    // Property 9: Role Hierarchy Enforcement
    // Feature: transaction-workflow, Property 9: Role Hierarchy Enforcement
    // Validates: Requirements 3.4, 3.6
    // =========================================================================

    /// Approval allowed iff user_role >= required_role in hierarchy
    #[test]
    fn prop_role_hierarchy_enforcement(
        user_role in arb_user_role(),
        required_role in arb_user_role(),
        amount in arb_amount()
    ) {
        // Skip approval limit check by using high limit
        let limit = Some(Decimal::new(999_999_999, 0));

        let result = ApprovalEngine::can_approve(
            user_role.as_str(),
            limit,
            required_role.as_str(),
            amount,
        );

        if user_role >= required_role {
            // Should succeed (unless approver with limit issue, but we set high limit)
            prop_assert!(result.is_ok(), "Expected Ok for {:?} >= {:?}", user_role, required_role);
        } else {
            // Should fail with InsufficientRole
            match result {
                Err(WorkflowError::InsufficientRole { .. }) => {}
                other => prop_assert!(false, "Expected InsufficientRole for {:?} < {:?}, got {:?}",
                    user_role, required_role, other),
            }
        }
    }

    // =========================================================================
    // Property 10: Approval Limit Enforcement
    // Feature: transaction-workflow, Property 10: Approval Limit Enforcement
    // Validates: Requirements 3.5
    // =========================================================================

    /// Approver role is subject to approval limit
    #[test]
    fn prop_approver_limit_enforcement(
        limit in 100i64..1000i64,
        amount in 100i64..2000i64
    ) {
        let limit_decimal = Decimal::new(limit, 0);
        let amount_decimal = Decimal::new(amount, 0);

        let result = ApprovalEngine::can_approve(
            "approver",
            Some(limit_decimal),
            "approver",
            amount_decimal,
        );

        if amount_decimal <= limit_decimal {
            prop_assert!(result.is_ok(), "Expected Ok when amount {} <= limit {}", amount, limit);
        } else {
            match result {
                Err(WorkflowError::ExceedsApprovalLimit { .. }) => {}
                other => prop_assert!(false, "Expected ExceedsApprovalLimit when amount {} > limit {}, got {:?}",
                    amount, limit, other),
            }
        }
    }

    /// Admin bypasses approval limit
    #[test]
    fn prop_admin_bypasses_limit(
        limit in 1i64..100i64,
        amount in 1000i64..10000i64
    ) {
        let limit_decimal = Decimal::new(limit, 0);
        let amount_decimal = Decimal::new(amount, 0);

        // Amount is always > limit in this test
        let result = ApprovalEngine::can_approve(
            "admin",
            Some(limit_decimal),
            "approver",
            amount_decimal,
        );

        prop_assert!(result.is_ok(), "Admin should bypass limit");
    }

    /// Owner bypasses approval limit
    #[test]
    fn prop_owner_bypasses_limit(
        limit in 1i64..100i64,
        amount in 1000i64..10000i64
    ) {
        let limit_decimal = Decimal::new(limit, 0);
        let amount_decimal = Decimal::new(amount, 0);

        let result = ApprovalEngine::can_approve(
            "owner",
            Some(limit_decimal),
            "approver",
            amount_decimal,
        );

        prop_assert!(result.is_ok(), "Owner should bypass limit");
    }

    /// Accountant bypasses approval limit
    #[test]
    fn prop_accountant_bypasses_limit(
        limit in 1i64..100i64,
        amount in 1000i64..10000i64
    ) {
        let limit_decimal = Decimal::new(limit, 0);
        let amount_decimal = Decimal::new(amount, 0);

        let result = ApprovalEngine::can_approve(
            "accountant",
            Some(limit_decimal),
            "approver",
            amount_decimal,
        );

        prop_assert!(result.is_ok(), "Accountant should bypass limit");
    }
}

// =========================================================================
// Unit tests for edge cases
// =========================================================================

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_no_matching_rules_returns_none() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Expense Only".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Invoice type doesn't match
        let result = ApprovalEngine::get_required_approval(&rules, "invoice", Decimal::new(100, 0));
        assert_eq!(result, None);
    }

    #[test]
    fn test_exact_min_amount_boundary() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Min 100".to_string(),
            min_amount: Some(Decimal::new(100, 0)),
            max_amount: None,
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Exactly at min_amount should match
        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(100, 0));
        assert_eq!(result, Some("approver".to_string()));

        // Below min_amount should not match
        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(99, 0));
        assert_eq!(result, None);
    }

    #[test]
    fn test_exact_max_amount_boundary() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Max 1000".to_string(),
            min_amount: None,
            max_amount: Some(Decimal::new(1000, 0)),
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Exactly at max_amount should match
        let result =
            ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(1000, 0));
        assert_eq!(result, Some("approver".to_string()));

        // Above max_amount should not match
        let result =
            ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(1001, 0));
        assert_eq!(result, None);
    }

    #[test]
    fn test_null_min_amount_no_lower_bound() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "No Min".to_string(),
            min_amount: None,
            max_amount: Some(Decimal::new(1000, 0)),
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Very small amount should match (no lower bound)
        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(1, 2));
        assert_eq!(result, Some("approver".to_string()));
    }

    #[test]
    fn test_null_max_amount_no_upper_bound() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "No Max".to_string(),
            min_amount: Some(Decimal::new(100, 0)),
            max_amount: None,
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Very large amount should match (no upper bound)
        let result =
            ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(999_999_999, 0));
        assert_eq!(result, Some("approver".to_string()));
    }

    #[test]
    fn test_empty_transaction_types_matches_all() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "All Types".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec![], // Empty = matches all
            required_role: "approver".to_string(),
            priority: 1,
        }];

        // Should match any transaction type
        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(100, 0));
        assert_eq!(result, Some("approver".to_string()));

        let result = ApprovalEngine::get_required_approval(&rules, "invoice", Decimal::new(100, 0));
        assert_eq!(result, Some("approver".to_string()));

        let result = ApprovalEngine::get_required_approval(&rules, "payment", Decimal::new(100, 0));
        assert_eq!(result, Some("approver".to_string()));
    }

    #[test]
    fn test_empty_rules_returns_none() {
        let rules: Vec<ApprovalRule> = vec![];
        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(100, 0));
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_user_role_returns_error() {
        let result =
            ApprovalEngine::can_approve("invalid_role", None, "approver", Decimal::new(100, 0));
        assert!(matches!(
            result,
            Err(WorkflowError::InsufficientRole { .. })
        ));
    }

    #[test]
    fn test_invalid_required_role_returns_error() {
        let result =
            ApprovalEngine::can_approve("admin", None, "invalid_role", Decimal::new(100, 0));
        assert!(matches!(
            result,
            Err(WorkflowError::InsufficientRole { .. })
        ));
    }

    #[test]
    fn test_approver_without_limit_can_approve_any_amount() {
        let result = ApprovalEngine::can_approve(
            "approver",
            None, // No limit set
            "approver",
            Decimal::new(999_999_999, 0),
        );
        assert!(result.is_ok());
    }
}
