//! Authentication types for JWT and tokens.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims for access tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: Uuid,
    /// Organization ID (current context).
    pub org: Uuid,
    /// User's role in the organization.
    pub role: String,
    /// Issued at timestamp.
    pub iat: i64,
    /// Expiration timestamp.
    pub exp: i64,
}

impl Claims {
    /// Creates new claims for a user.
    #[must_use]
    pub fn new(user_id: Uuid, org_id: Uuid, role: &str, expires_at: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            org: org_id,
            role: role.to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        }
    }

    /// Returns the user ID from claims.
    #[must_use]
    pub const fn user_id(&self) -> Uuid {
        self.sub
    }

    /// Returns the organization ID from claims.
    #[must_use]
    pub const fn organization_id(&self) -> Uuid {
        self.org
    }
}

/// Token pair returned after successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (short-lived).
    pub access_token: String,
    /// Refresh token (long-lived).
    pub refresh_token: String,
    /// Access token expiration in seconds.
    pub expires_in: i64,
}

impl TokenPair {
    /// Creates a new token pair.
    #[must_use]
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
        }
    }
}

/// Login request payload.
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    /// User email.
    pub email: String,
    /// User password.
    pub password: String,
}

/// Registration request payload.
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    /// User email.
    pub email: String,
    /// User password.
    pub password: String,
    /// User full name.
    pub full_name: String,
}

/// Login response payload.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    /// Authenticated user info.
    pub user: UserInfo,
    /// Access token.
    pub access_token: String,
    /// Refresh token.
    pub refresh_token: String,
    /// Token expiration in seconds.
    pub expires_in: i64,
}

/// User info returned in auth responses.
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    /// User ID.
    pub id: Uuid,
    /// User email.
    pub email: String,
    /// User full name.
    pub full_name: String,
    /// Organizations the user belongs to.
    pub organizations: Vec<UserOrganization>,
}

/// Organization info for a user.
#[derive(Debug, Clone, Serialize)]
pub struct UserOrganization {
    /// Organization ID.
    pub id: Uuid,
    /// Organization name.
    pub name: String,
    /// Organization slug.
    pub slug: String,
    /// User's role in this organization.
    pub role: String,
}

/// Refresh token request.
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshRequest {
    /// The refresh token.
    pub refresh_token: String,
}

/// Logout request.
#[derive(Debug, Clone, Deserialize)]
pub struct LogoutRequest {
    /// The refresh token to invalidate.
    pub refresh_token: String,
}

/// Create organization request.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrganizationRequest {
    /// Organization name.
    pub name: String,
    /// Organization slug (URL-friendly).
    pub slug: String,
    /// Base currency (ISO 4217 code).
    pub base_currency: String,
    /// Timezone (IANA format).
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// Add user to organization request.
#[derive(Debug, Clone, Deserialize)]
pub struct AddUserRequest {
    /// User email to add.
    pub email: String,
    /// Role to assign.
    pub role: String,
    /// Approval limit (for approver role).
    pub approval_limit: Option<String>,
}

/// Update organization request.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateOrganizationRequest {
    /// Organization name (optional).
    pub name: Option<String>,
    /// Base currency (optional, ISO 4217 code).
    pub base_currency: Option<String>,
    /// Timezone (optional, IANA format).
    pub timezone: Option<String>,
}

/// Email verification request.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyEmailRequest {
    /// The verification token from the email link.
    pub token: String,
}

/// Resend verification email request.
#[derive(Debug, Clone, Deserialize)]
pub struct ResendVerificationRequest {
    /// User email to resend verification to.
    pub email: String,
}

/// Email verification response.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyEmailResponse {
    /// Success message.
    pub message: String,
    /// Whether email is now verified.
    pub verified: bool,
}

/// Update organization member request.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMemberRequest {
    /// New role (optional).
    pub role: Option<String>,
    /// New approval limit (optional, null to clear).
    pub approval_limit: Option<Option<String>>,
}
