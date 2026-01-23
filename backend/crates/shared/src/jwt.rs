//! JWT token generation and validation.
//!
//! Provides secure JWT handling with access and refresh tokens.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use thiserror::Error;
use uuid::Uuid;

use crate::auth::Claims;

/// JWT configuration.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens.
    pub secret: String,
    /// Access token expiration in minutes.
    pub access_token_expires_minutes: i64,
    /// Refresh token expiration in days.
    pub refresh_token_expires_days: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-me-in-production".to_string(),
            access_token_expires_minutes: 15,
            refresh_token_expires_days: 7,
        }
    }
}

/// Errors that can occur during JWT operations.
#[derive(Debug, Error)]
pub enum JwtError {
    /// Token encoding failed.
    #[error("failed to encode token: {0}")]
    EncodingError(String),

    /// Token decoding failed.
    #[error("failed to decode token: {0}")]
    DecodingError(String),

    /// Token has expired.
    #[error("token has expired")]
    Expired,

    /// Token is invalid.
    #[error("invalid token")]
    Invalid,
}

/// JWT service for token operations.
#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("config", &self.config)
            .field("encoding_key", &"[hidden]")
            .field("decoding_key", &"[hidden]")
            .finish()
    }
}

impl JwtService {
    /// Creates a new JWT service with the given configuration.
    ///
    /// # Panics
    ///
    /// Panics if the JWT secret is set to the default value "change-me-in-production".
    /// This is a critical security check to prevent production deployments with insecure secrets.
    #[must_use]
    pub fn new(config: JwtConfig) -> Self {
        // Critical security check: Prevent using default JWT secret in production
        assert!(
            config.secret != "change-me-in-production",
            "\n\n\
            ╔═══════════════════════════════════════════════════════════════════════════╗\n\
            ║                      CRITICAL SECURITY ERROR                              ║\n\
            ╠═══════════════════════════════════════════════════════════════════════════╣\n\
            ║                                                                           ║\n\
            ║  JWT secret is set to the default value 'change-me-in-production'        ║\n\
            ║                                                                           ║\n\
            ║  This is a CRITICAL SECURITY VULNERABILITY that would allow attackers    ║\n\
            ║  to forge authentication tokens and gain unauthorized access to your     ║\n\
            ║  application.                                                            ║\n\
            ║                                                                           ║\n\
            ║  HOW TO FIX:                                                             ║\n\
            ║  1. Generate a strong random secret (at least 32 characters)             ║\n\
            ║     Example: openssl rand -base64 32                                     ║\n\
            ║                                                                           ║\n\
            ║  2. Set the JWT_SECRET environment variable:                             ║\n\
            ║     export JWT_SECRET=\"your-secure-random-secret-here\"                  ║\n\
            ║                                                                           ║\n\
            ║  3. Or update your configuration file (config/default.toml):             ║\n\
            ║     [jwt]                                                                ║\n\
            ║     secret = \"your-secure-random-secret-here\"                           ║\n\
            ║                                                                           ║\n\
            ║  NEVER commit secrets to version control!                                ║\n\
            ║  Use environment variables or secure secret management.                  ║\n\
            ║                                                                           ║\n\
            ╚═══════════════════════════════════════════════════════════════════════════╝\n\n"
        );

        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    /// Generates an access token for a user.
    ///
    /// # Errors
    ///
    /// Returns `JwtError::EncodingError` if token generation fails.
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        role: &str,
    ) -> Result<String, JwtError> {
        let expires_at = Utc::now() + Duration::minutes(self.config.access_token_expires_minutes);
        let claims = Claims::new(user_id, org_id, role, expires_at);

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::EncodingError(e.to_string()))
    }

    /// Generates a refresh token for a user.
    ///
    /// # Errors
    ///
    /// Returns `JwtError::EncodingError` if token generation fails.
    pub fn generate_refresh_token(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        role: &str,
    ) -> Result<String, JwtError> {
        let expires_at = Utc::now() + Duration::days(self.config.refresh_token_expires_days);
        let claims = Claims::new(user_id, org_id, role, expires_at);

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::EncodingError(e.to_string()))
    }

    /// Validates and decodes a token.
    ///
    /// # Errors
    ///
    /// Returns `JwtError::Expired` if the token has expired.
    /// Returns `JwtError::Invalid` if the token is malformed.
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::default();

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
                _ => JwtError::DecodingError(e.to_string()),
            })
    }

    /// Returns the access token expiration in seconds.
    #[must_use]
    pub const fn access_token_expires_in(&self) -> i64 {
        self.config.access_token_expires_minutes * 60
    }

    /// Returns the refresh token expiration in days.
    #[must_use]
    pub const fn refresh_token_expires_days(&self) -> i64 {
        self.config.refresh_token_expires_days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> JwtService {
        JwtService::new(JwtConfig {
            secret: "test-secret-key-for-testing".to_string(),
            access_token_expires_minutes: 15,
            refresh_token_expires_days: 7,
        })
    }

    #[test]
    fn test_generate_access_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let token = service
            .generate_access_token(user_id, org_id, "admin")
            .unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_validate_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let token = service
            .generate_access_token(user_id, org_id, "admin")
            .unwrap();
        let claims = service.validate_token(&token).unwrap();

        assert_eq!(claims.user_id(), user_id);
        assert_eq!(claims.organization_id(), org_id);
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_invalid_token() {
        let service = create_test_service();
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "CRITICAL SECURITY ERROR")]
    fn test_panic_on_default_secret() {
        // This should panic because we're using the default secret
        let _service = JwtService::new(JwtConfig::default());
    }

    #[test]
    fn test_custom_secret_works() {
        // This should NOT panic because we're using a custom secret
        let service = JwtService::new(JwtConfig {
            secret: "my-custom-secure-secret-key".to_string(),
            access_token_expires_minutes: 15,
            refresh_token_expires_days: 7,
        });

        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let token = service
            .generate_access_token(user_id, org_id, "admin")
            .unwrap();
        assert!(!token.is_empty());
    }
}
