//! Authentication types for JWT and tokens.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Helper enum for handling nested Option updates.
///
/// This solves the problem of distinguishing between:
/// - Field not provided in request (None)
/// - Field explicitly set to null (Some(None))
/// - Field set to a value (Some(Some(value)))
#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum OptionalUpdate<T> {
    /// Value is provided.
    Value(T),
    /// Value is explicitly set to null.
    #[serde(deserialize_with = "deserialize_null")]
    Null,
}

fn deserialize_null<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_null() {
        Ok(())
    } else {
        Err(Error::custom("expected null"))
    }
}

impl<T> OptionalUpdate<T> {
    /// Converts to Option<Option<T>> for database operations.
    pub fn into_option(self) -> Option<Option<T>> {
        match self {
            OptionalUpdate::Value(v) => Some(Some(v)),
            OptionalUpdate::Null => Some(None),
        }
    }
}

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
#[derive(Debug, Clone, Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    /// User email.
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User password.
    #[validate(length(min = 8, max = 128))]
    #[schema(example = "SecureP@ssw0rd!")]
    pub password: String,
}

/// Registration request payload.
#[derive(Debug, Clone, Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    /// User email.
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User password (minimum 8 characters).
    #[validate(length(min = 8, max = 128))]
    #[schema(example = "SecureP@ssw0rd!")]
    pub password: String,
    /// User full name.
    #[validate(length(min = 2, max = 255))]
    #[schema(example = "John Doe")]
    pub full_name: String,
}

/// Registration response payload.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RegisterResponse {
    /// Registered user info.
    pub user: UserInfo,
    /// Success message.
    pub message: String,
}

/// Login response payload.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
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
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct UserInfo {
    /// User ID.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// User email.
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User full name.
    #[schema(example = "John Doe")]
    pub full_name: String,
    /// Organizations the user belongs to.
    pub organizations: Vec<UserOrganization>,
}

/// Organization info for a user.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
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
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct RefreshRequest {
    /// The refresh token.
    pub refresh_token: String,
}

/// Refresh token response.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct RefreshResponse {
    /// New access token.
    pub access_token: String,
    /// Token expiration in seconds.
    pub expires_in: i64,
}

/// Logout request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct LogoutRequest {
    /// The refresh token to invalidate.
    pub refresh_token: String,
}

/// Create organization request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateOrganizationRequest {
    /// Organization name.
    #[schema(example = "Acme Corporation")]
    pub name: String,
    /// Organization slug (URL-friendly).
    #[schema(example = "acme-corp")]
    pub slug: String,
    /// Base currency (ISO 4217 code).
    #[schema(example = "USD")]
    pub base_currency: String,
    /// Timezone (IANA format).
    #[serde(default = "default_timezone")]
    #[schema(default = "UTC", example = "America/New_York")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// Add user to organization request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct AddUserRequest {
    /// User email to add.
    #[schema(example = "newuser@example.com")]
    pub email: String,
    /// Role to assign.
    #[schema(example = "viewer")]
    pub role: String,
    /// Approval limit (for approver role).
    #[schema(example = "10000.00")]
    pub approval_limit: Option<String>,
}

/// Update organization request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateOrganizationRequest {
    /// Organization name (optional).
    #[schema(nullable = true, example = "Acme Corporation")]
    pub name: Option<String>,
    /// Base currency (optional, ISO 4217 code).
    #[schema(nullable = true, example = "EUR")]
    pub base_currency: Option<String>,
    /// Timezone (optional, IANA format).
    #[schema(nullable = true, example = "Europe/London")]
    pub timezone: Option<String>,
}

/// Email verification request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct VerifyEmailRequest {
    /// The verification token from the email link.
    pub token: String,
}

/// Resend verification email request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct ResendVerificationRequest {
    /// User email to resend verification to.
    pub email: String,
}

/// Resend verification email response.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ResendVerificationResponse {
    /// Success message.
    pub message: String,
}

/// Email verification response.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct VerifyEmailResponse {
    /// Success message.
    pub message: String,
    /// Whether email is now verified.
    pub verified: bool,
}

/// Update organization member request.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberRequest {
    /// New role (optional).
    #[schema(nullable = true)]
    pub role: Option<String>,
    /// New approval limit (optional, use null to clear).
    #[schema(nullable = true)]
    pub approval_limit: Option<OptionalUpdate<String>>,
}
