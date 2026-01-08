//! User repository for database operations.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::{organization_users, organizations, users};

/// User repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    /// Creates a new user repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Finds a user by email.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<users::Model>, DbErr> {
        users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db)
            .await
    }

    /// Finds a user by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<users::Model>, DbErr> {
        users::Entity::find_by_id(id).one(&self.db).await
    }

    /// Creates a new user.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create(
        &self,
        email: &str,
        password_hash: &str,
        full_name: &str,
    ) -> Result<users::Model, DbErr> {
        let now = chrono::Utc::now().into();
        let user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash.to_string()),
            full_name: Set(full_name.to_string()),
            is_active: Set(true),
            email_verified_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        user.insert(&self.db).await
    }

    /// Gets all organizations for a user with their roles.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_user_organizations(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(organizations::Model, organization_users::Model)>, DbErr> {
        organization_users::Entity::find()
            .filter(organization_users::Column::UserId.eq(user_id))
            .find_also_related(organizations::Entity)
            .all(&self.db)
            .await
            .map(|results| {
                results
                    .into_iter()
                    .filter_map(|(ou, org)| org.map(|o| (o, ou)))
                    .collect()
            })
    }

    /// Gets the first organization for a user (for default login context).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_default_organization(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(organizations::Model, organization_users::Model)>, DbErr> {
        let result = organization_users::Entity::find()
            .filter(organization_users::Column::UserId.eq(user_id))
            .find_also_related(organizations::Entity)
            .one(&self.db)
            .await?;

        Ok(result.and_then(|(ou, org)| org.map(|o| (o, ou))))
    }

    /// Checks if an email is already registered.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn email_exists(&self, email: &str) -> Result<bool, DbErr> {
        let count = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }
}
