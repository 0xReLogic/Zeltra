//! Authentication and password hashing.
//!
//! This module provides:
//! - Password hashing with Argon2id
//! - Password verification
//! - User role definitions

mod password;

pub use password::{PasswordError, hash_password, verify_password};

use serde::{Deserialize, Serialize};

/// User roles within an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Full access, can transfer ownership.
    Owner,
    /// Full access except ownership transfer.
    Admin,
    /// Can create/edit transactions, close periods.
    Accountant,
    /// Can approve transactions up to their limit.
    Approver,
    /// Read-only access.
    Viewer,
    /// Can submit transactions for approval.
    Submitter,
}

impl UserRole {
    /// Returns true if this role can approve transactions.
    #[must_use]
    pub const fn can_approve(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Approver)
    }

    /// Returns true if this role can post to soft-closed periods.
    #[must_use]
    pub const fn can_post_soft_close(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Accountant)
    }

    /// Returns true if this role can manage users.
    #[must_use]
    pub const fn can_manage_users(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Returns true if this role can modify organization settings.
    #[must_use]
    pub const fn can_modify_settings(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Owner => write!(f, "owner"),
            Self::Admin => write!(f, "admin"),
            Self::Accountant => write!(f, "accountant"),
            Self::Approver => write!(f, "approver"),
            Self::Viewer => write!(f, "viewer"),
            Self::Submitter => write!(f, "submitter"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[test]
    fn test_role_permissions() {
        assert!(UserRole::Owner.can_approve());
        assert!(UserRole::Admin.can_approve());
        assert!(UserRole::Approver.can_approve());
        assert!(!UserRole::Accountant.can_approve());
        assert!(!UserRole::Viewer.can_approve());

        assert!(UserRole::Owner.can_post_soft_close());
        assert!(UserRole::Admin.can_post_soft_close());
        assert!(UserRole::Accountant.can_post_soft_close());
        assert!(!UserRole::Approver.can_post_soft_close());
    }

    #[rstest]
    #[case(UserRole::Owner, true)]
    #[case(UserRole::Admin, true)]
    #[case(UserRole::Approver, true)]
    #[case(UserRole::Accountant, false)]
    #[case(UserRole::Viewer, false)]
    #[case(UserRole::Submitter, false)]
    fn role_can_approve_matrix(#[case] role: UserRole, #[case] expected: bool) {
        assert_eq!(role.can_approve(), expected);
    }

    #[rstest]
    #[case(UserRole::Owner, true)]
    #[case(UserRole::Admin, true)]
    #[case(UserRole::Accountant, true)]
    #[case(UserRole::Approver, false)]
    #[case(UserRole::Viewer, false)]
    #[case(UserRole::Submitter, false)]
    fn role_can_post_soft_close_matrix(#[case] role: UserRole, #[case] expected: bool) {
        assert_eq!(role.can_post_soft_close(), expected);
    }

    #[rstest]
    #[case(UserRole::Owner, true)]
    #[case(UserRole::Admin, true)]
    #[case(UserRole::Accountant, false)]
    #[case(UserRole::Approver, false)]
    #[case(UserRole::Viewer, false)]
    #[case(UserRole::Submitter, false)]
    fn role_can_manage_users_matrix(#[case] role: UserRole, #[case] expected: bool) {
        assert_eq!(role.can_manage_users(), expected);
    }

    #[rstest]
    #[case(UserRole::Owner, true)]
    #[case(UserRole::Admin, true)]
    #[case(UserRole::Accountant, false)]
    #[case(UserRole::Approver, false)]
    #[case(UserRole::Viewer, false)]
    #[case(UserRole::Submitter, false)]
    fn role_can_modify_settings_matrix(#[case] role: UserRole, #[case] expected: bool) {
        assert_eq!(role.can_modify_settings(), expected);
    }

    #[rstest]
    #[case(UserRole::Owner, "owner")]
    #[case(UserRole::Admin, "admin")]
    #[case(UserRole::Accountant, "accountant")]
    #[case(UserRole::Approver, "approver")]
    #[case(UserRole::Viewer, "viewer")]
    #[case(UserRole::Submitter, "submitter")]
    fn role_display_formats_to_snake_case(#[case] role: UserRole, #[case] expected: &str) {
        assert_eq!(role.to_string(), expected);
    }

    #[rstest]
    #[case(UserRole::Owner, "\"owner\"")]
    #[case(UserRole::Admin, "\"admin\"")]
    #[case(UserRole::Accountant, "\"accountant\"")]
    #[case(UserRole::Approver, "\"approver\"")]
    #[case(UserRole::Viewer, "\"viewer\"")]
    #[case(UserRole::Submitter, "\"submitter\"")]
    fn role_serializes_to_snake_case(#[case] role: UserRole, #[case] expected: &str) {
        let serialized = serde_json::to_string(&role).expect("serialize role");
        assert_eq!(serialized, expected);
    }

    #[rstest]
    #[case("owner", UserRole::Owner)]
    #[case("admin", UserRole::Admin)]
    #[case("accountant", UserRole::Accountant)]
    #[case("approver", UserRole::Approver)]
    #[case("viewer", UserRole::Viewer)]
    #[case("submitter", UserRole::Submitter)]
    fn role_deserializes_from_snake_case(#[case] raw: &str, #[case] expected: UserRole) {
        let value = json!(raw);
        let parsed: UserRole = serde_json::from_value(value).expect("deserialize role");
        assert_eq!(parsed, expected);
    }
}
