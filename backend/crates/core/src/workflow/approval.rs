//! Approval rules engine for transaction authorization.
//!
//! This module implements the approval rules matching and
//! user authorization checks for transaction approvals.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::workflow::error::WorkflowError;

/// User role in the organization hierarchy.
///
/// Roles are ordered from lowest to highest privilege.
/// Higher roles can perform all actions of lower roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Can only view transactions.
    Viewer = 0,
    /// Can create and submit transactions.
    Submitter = 1,
    /// Can approve transactions within their limit.
    Approver = 2,
    /// Can approve and post transactions.
    Accountant = 3,
    /// Full access except ownership transfer.
    Admin = 4,
    /// Full access including ownership transfer.
    Owner = 5,
}

impl UserRole {
    /// Parse a role from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "viewer" => Some(Self::Viewer),
            "submitter" => Some(Self::Submitter),
            "approver" => Some(Self::Approver),
            "accountant" => Some(Self::Accountant),
            "admin" => Some(Self::Admin),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    /// Returns the string representation of the role.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Submitter => "submitter",
            Self::Approver => "approver",
            Self::Accountant => "accountant",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }
}

/// An approval rule that determines who can approve transactions.
///
/// Rules are matched by transaction type and amount range.
/// When multiple rules match, the one with lowest priority value wins.
#[derive(Debug, Clone)]
pub struct ApprovalRule {
    /// Unique identifier for the rule.
    pub id: Uuid,
    /// Human-readable name for the rule.
    pub name: String,
    /// Minimum amount for this rule to apply (inclusive, None = no minimum).
    pub min_amount: Option<Decimal>,
    /// Maximum amount for this rule to apply (inclusive, None = no maximum).
    pub max_amount: Option<Decimal>,
    /// Transaction types this rule applies to.
    pub transaction_types: Vec<String>,
    /// The role required to approve matching transactions.
    pub required_role: String,
    /// Priority for rule selection (lower = higher priority).
    pub priority: i16,
}

/// Stateless engine for evaluating approval rules.
pub struct ApprovalEngine;

impl ApprovalEngine {
    /// Determine the required approver role for a transaction.
    ///
    /// # Arguments
    /// * `rules` - The approval rules to evaluate
    /// * `transaction_type` - The type of transaction
    /// * `total_amount` - The total amount of the transaction
    ///
    /// # Returns
    /// The required role string if a matching rule is found, None otherwise.
    #[must_use]
    pub fn get_required_approval(
        rules: &[ApprovalRule],
        transaction_type: &str,
        total_amount: Decimal,
    ) -> Option<String> {
        let mut applicable: Vec<_> = rules
            .iter()
            .filter(|r| {
                r.transaction_types.is_empty()
                    || r.transaction_types.contains(&transaction_type.to_string())
            })
            .filter(|r| {
                let above_min = r.min_amount.is_none_or(|min| total_amount >= min);
                let below_max = r.max_amount.is_none_or(|max| total_amount <= max);
                above_min && below_max
            })
            .collect();

        // Sort by priority (lower = higher priority)
        applicable.sort_by_key(|r| r.priority);
        applicable.first().map(|r| r.required_role.clone())
    }

    /// Check if a user can approve a transaction.
    ///
    /// # Arguments
    /// * `user_role` - The user's role as a string
    /// * `user_approval_limit` - The user's approval limit (for Approver role)
    /// * `required_role` - The required role for approval
    /// * `transaction_amount` - The transaction amount
    ///
    /// # Returns
    /// * `Ok(())` if the user can approve
    /// * `Err(WorkflowError::InsufficientRole)` if role is too low
    /// * `Err(WorkflowError::ExceedsApprovalLimit)` if amount exceeds limit
    pub fn can_approve(
        user_role: &str,
        user_approval_limit: Option<Decimal>,
        required_role: &str,
        transaction_amount: Decimal,
    ) -> Result<(), WorkflowError> {
        let user_role_enum =
            UserRole::parse(user_role).ok_or_else(|| WorkflowError::InsufficientRole {
                user_role: user_role.to_string(),
                required_role: required_role.to_string(),
            })?;

        let required_role_enum =
            UserRole::parse(required_role).ok_or_else(|| WorkflowError::InsufficientRole {
                user_role: user_role.to_string(),
                required_role: required_role.to_string(),
            })?;

        // Check role hierarchy
        if user_role_enum < required_role_enum {
            return Err(WorkflowError::InsufficientRole {
                user_role: user_role.to_string(),
                required_role: required_role.to_string(),
            });
        }

        // Check approval limit (only for Approver role, higher roles have unlimited)
        if user_role_enum == UserRole::Approver
            && let Some(limit) = user_approval_limit
            && transaction_amount > limit
        {
            return Err(WorkflowError::ExceedsApprovalLimit {
                amount: transaction_amount,
                limit,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_from_str() {
        assert_eq!(UserRole::parse("viewer"), Some(UserRole::Viewer));
        assert_eq!(UserRole::parse("SUBMITTER"), Some(UserRole::Submitter));
        assert_eq!(UserRole::parse("Approver"), Some(UserRole::Approver));
        assert_eq!(UserRole::parse("accountant"), Some(UserRole::Accountant));
        assert_eq!(UserRole::parse("admin"), Some(UserRole::Admin));
        assert_eq!(UserRole::parse("owner"), Some(UserRole::Owner));
        assert_eq!(UserRole::parse("invalid"), None);
    }

    #[test]
    fn test_user_role_as_str() {
        assert_eq!(UserRole::Viewer.as_str(), "viewer");
        assert_eq!(UserRole::Submitter.as_str(), "submitter");
        assert_eq!(UserRole::Approver.as_str(), "approver");
        assert_eq!(UserRole::Accountant.as_str(), "accountant");
        assert_eq!(UserRole::Admin.as_str(), "admin");
        assert_eq!(UserRole::Owner.as_str(), "owner");
    }

    #[test]
    fn test_user_role_ordering() {
        assert!(UserRole::Viewer < UserRole::Submitter);
        assert!(UserRole::Submitter < UserRole::Approver);
        assert!(UserRole::Approver < UserRole::Accountant);
        assert!(UserRole::Accountant < UserRole::Admin);
        assert!(UserRole::Admin < UserRole::Owner);
    }

    #[test]
    fn test_get_required_approval_single_rule() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Default".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(100, 0));
        assert_eq!(result, Some("approver".to_string()));
    }

    #[test]
    fn test_get_required_approval_no_match() {
        let rules = vec![ApprovalRule {
            id: Uuid::new_v4(),
            name: "Default".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec!["expense".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        }];

        let result = ApprovalEngine::get_required_approval(&rules, "invoice", Decimal::new(100, 0));
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_required_approval_amount_range() {
        let rules = vec![
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Small".to_string(),
                min_amount: None,
                max_amount: Some(Decimal::new(1000, 0)),
                transaction_types: vec!["expense".to_string()],
                required_role: "approver".to_string(),
                priority: 1,
            },
            ApprovalRule {
                id: Uuid::new_v4(),
                name: "Large".to_string(),
                min_amount: Some(Decimal::new(1001, 0)),
                max_amount: None,
                transaction_types: vec!["expense".to_string()],
                required_role: "admin".to_string(),
                priority: 2,
            },
        ];

        // Small amount
        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(500, 0));
        assert_eq!(result, Some("approver".to_string()));

        // Large amount
        let result =
            ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(5000, 0));
        assert_eq!(result, Some("admin".to_string()));
    }

    #[test]
    fn test_get_required_approval_priority() {
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
        ];

        let result = ApprovalEngine::get_required_approval(&rules, "expense", Decimal::new(100, 0));
        assert_eq!(result, Some("approver".to_string()));
    }

    #[test]
    fn test_can_approve_sufficient_role() {
        let result = ApprovalEngine::can_approve("admin", None, "approver", Decimal::new(1000, 0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_approve_insufficient_role() {
        let result =
            ApprovalEngine::can_approve("submitter", None, "approver", Decimal::new(1000, 0));
        assert!(matches!(
            result,
            Err(WorkflowError::InsufficientRole { .. })
        ));
    }

    #[test]
    fn test_can_approve_within_limit() {
        let result = ApprovalEngine::can_approve(
            "approver",
            Some(Decimal::new(5000, 0)),
            "approver",
            Decimal::new(1000, 0),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_approve_exceeds_limit() {
        let result = ApprovalEngine::can_approve(
            "approver",
            Some(Decimal::new(500, 0)),
            "approver",
            Decimal::new(1000, 0),
        );
        assert!(matches!(
            result,
            Err(WorkflowError::ExceedsApprovalLimit { .. })
        ));
    }

    #[test]
    fn test_can_approve_admin_bypasses_limit() {
        // Admin should not be subject to approval limits
        let result = ApprovalEngine::can_approve(
            "admin",
            Some(Decimal::new(500, 0)), // Even with a limit set
            "approver",
            Decimal::new(10000, 0), // Large amount
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_can_approve_owner_bypasses_limit() {
        let result = ApprovalEngine::can_approve(
            "owner",
            Some(Decimal::new(500, 0)),
            "approver",
            Decimal::new(10000, 0),
        );
        assert!(result.is_ok());
    }
}
