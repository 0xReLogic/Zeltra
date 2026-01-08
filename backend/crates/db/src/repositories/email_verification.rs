//! Email verification repository for database operations.

use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entities::{email_verification_tokens, users};

/// Email verification repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct EmailVerificationRepository {
    db: DatabaseConnection,
}

impl EmailVerificationRepository {
    /// Creates a new email verification repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Hashes a verification token for storage.
    #[must_use]
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generates a random verification token.
    #[must_use]
    pub fn generate_token() -> String {
        // Generate a URL-safe random token
        let bytes: [u8; 32] = rand::random();
        base64_url::encode(&bytes)
    }

    /// Creates a new verification token for a user.
    /// Returns the raw token (not hashed) to be sent via email.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create_token(&self, user_id: Uuid) -> Result<String, DbErr> {
        // Invalidate any existing tokens for this user
        self.invalidate_user_tokens(user_id).await?;

        let raw_token = Self::generate_token();
        let token_hash = Self::hash_token(&raw_token);
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // Token valid for 24 hours

        let token = email_verification_tokens::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at.into()),
            used_at: Set(None),
            created_at: Set(now.into()),
        };

        token.insert(&self.db).await?;

        Ok(raw_token)
    }

    /// Verifies a token and marks the user's email as verified.
    /// Returns the user if verification succeeds.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid, expired, or already used.
    pub async fn verify_token(&self, raw_token: &str) -> Result<users::Model, DbErr> {
        let token_hash = Self::hash_token(raw_token);
        let now = Utc::now();

        // Find the token
        let token = email_verification_tokens::Entity::find()
            .filter(email_verification_tokens::Column::TokenHash.eq(&token_hash))
            .filter(email_verification_tokens::Column::UsedAt.is_null())
            .filter(email_verification_tokens::Column::ExpiresAt.gt(now))
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Invalid or expired verification token".to_string()))?;

        // Mark token as used
        let mut token_active: email_verification_tokens::ActiveModel = token.clone().into();
        token_active.used_at = Set(Some(now.into()));
        token_active.update(&self.db).await?;

        // Update user's email_verified_at
        let user = users::Entity::find_by_id(token.user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("User not found".to_string()))?;

        let mut user_active: users::ActiveModel = user.into();
        user_active.email_verified_at = Set(Some(now.into()));
        user_active.updated_at = Set(now.into());
        let updated_user = user_active.update(&self.db).await?;

        Ok(updated_user)
    }

    /// Invalidates all existing tokens for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn invalidate_user_tokens(&self, user_id: Uuid) -> Result<u64, DbErr> {
        let now = Utc::now();

        let result = email_verification_tokens::Entity::update_many()
            .col_expr(
                email_verification_tokens::Column::UsedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(email_verification_tokens::Column::UserId.eq(user_id))
            .filter(email_verification_tokens::Column::UsedAt.is_null())
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Checks if a user's email is verified.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn is_email_verified(&self, user_id: Uuid) -> Result<bool, DbErr> {
        let user = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("User not found".to_string()))?;

        Ok(user.email_verified_at.is_some())
    }

    /// Cleans up expired tokens (for maintenance).
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete fails.
    pub async fn cleanup_expired(&self) -> Result<u64, DbErr> {
        let result = email_verification_tokens::Entity::delete_many()
            .filter(email_verification_tokens::Column::ExpiresAt.lt(Utc::now()))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}
