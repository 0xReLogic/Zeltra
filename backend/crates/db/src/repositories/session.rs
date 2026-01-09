//! Session repository for database operations.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entities::sessions;

/// Session repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct SessionRepository {
    db: DatabaseConnection,
}

impl SessionRepository {
    /// Creates a new session repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Hashes a refresh token for storage.
    #[must_use]
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Creates a new session.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        refresh_token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<sessions::Model, DbErr> {
        let now = chrono::Utc::now().into();
        let token_hash = Self::hash_token(refresh_token);

        let session = sessions::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            organization_id: Set(organization_id),
            refresh_token_hash: Set(token_hash),
            user_agent: Set(user_agent.map(String::from)),
            ip_address: Set(ip_address.map(String::from)),
            expires_at: Set(expires_at.into()),
            revoked_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        session.insert(&self.db).await
    }

    /// Finds a session by refresh token.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_token(
        &self,
        refresh_token: &str,
    ) -> Result<Option<sessions::Model>, DbErr> {
        let token_hash = Self::hash_token(refresh_token);

        sessions::Entity::find()
            .filter(sessions::Column::RefreshTokenHash.eq(token_hash))
            .filter(sessions::Column::RevokedAt.is_null())
            .one(&self.db)
            .await
    }

    /// Finds a session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<sessions::Model>, DbErr> {
        sessions::Entity::find_by_id(id).one(&self.db).await
    }

    /// Gets all active sessions for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<sessions::Model>, DbErr> {
        sessions::Entity::find()
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::RevokedAt.is_null())
            .filter(sessions::Column::ExpiresAt.gt(chrono::Utc::now()))
            .order_by_desc(sessions::Column::CreatedAt)
            .all(&self.db)
            .await
    }

    /// Revokes a session by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn revoke(&self, id: Uuid) -> Result<(), DbErr> {
        let now = chrono::Utc::now().into();

        sessions::ActiveModel {
            id: Set(id),
            revoked_at: Set(Some(now)),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&self.db)
        .await?;

        Ok(())
    }

    /// Revokes a session by refresh token.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn revoke_by_token(&self, refresh_token: &str) -> Result<bool, DbErr> {
        let session = self.find_by_token(refresh_token).await?;

        if let Some(s) = session {
            self.revoke(s.id).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Revokes all sessions for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn revoke_all_user_sessions(&self, user_id: Uuid) -> Result<u64, DbErr> {
        let now = chrono::Utc::now();

        let result = sessions::Entity::update_many()
            .col_expr(
                sessions::Column::RevokedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                sessions::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::RevokedAt.is_null())
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Revokes all sessions for a user except the current one.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn revoke_other_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<u64, DbErr> {
        let now = chrono::Utc::now();

        let result = sessions::Entity::update_many()
            .col_expr(
                sessions::Column::RevokedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                sessions::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::Id.ne(current_session_id))
            .filter(sessions::Column::RevokedAt.is_null())
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Counts active sessions for a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn count_active_sessions(&self, user_id: Uuid) -> Result<u64, DbErr> {
        sessions::Entity::find()
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::RevokedAt.is_null())
            .filter(sessions::Column::ExpiresAt.gt(chrono::Utc::now()))
            .count(&self.db)
            .await
    }

    /// Cleans up expired sessions (for maintenance).
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete fails.
    pub async fn cleanup_expired(&self) -> Result<u64, DbErr> {
        let result = sessions::Entity::delete_many()
            .filter(sessions::Column::ExpiresAt.lt(chrono::Utc::now()))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }

    /// Revokes all sessions for a user in a specific organization.
    ///
    /// This is used when removing a user from an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database update fails.
    pub async fn revoke_user_org_sessions(
        &self,
        user_id: Uuid,
        org_id: Uuid,
    ) -> Result<u64, DbErr> {
        let now = chrono::Utc::now();

        let result = sessions::Entity::update_many()
            .col_expr(
                sessions::Column::RevokedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                sessions::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(sessions::Column::UserId.eq(user_id))
            .filter(sessions::Column::OrganizationId.eq(org_id))
            .filter(sessions::Column::RevokedAt.is_null())
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}
