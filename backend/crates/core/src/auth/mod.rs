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
}
