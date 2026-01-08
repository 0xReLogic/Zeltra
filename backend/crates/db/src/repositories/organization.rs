//! Organization repository for database operations.

use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set, TransactionTrait,
};
use serde_json::json;
use uuid::Uuid;

use crate::entities::{
    organization_users, organizations,
    sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier, UserRole},
    users,
};

/// Organization repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct OrganizationRepository {
    db: DatabaseConnection,
}

impl OrganizationRepository {
    /// Creates a new organization repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Finds an organization by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<organizations::Model>, DbErr> {
        organizations::Entity::find_by_id(id).one(&self.db).await
    }

    /// Finds an organization by slug.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<organizations::Model>, DbErr> {
        organizations::Entity::find()
            .filter(organizations::Column::Slug.eq(slug))
            .one(&self.db)
            .await
    }

    /// Checks if a slug is already taken.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn slug_exists(&self, slug: &str) -> Result<bool, DbErr> {
        let count = organizations::Entity::find()
            .filter(organizations::Column::Slug.eq(slug))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Creates a new organization with the creator as owner.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn create_with_owner(
        &self,
        name: &str,
        slug: &str,
        base_currency: &str,
        timezone: &str,
        owner_id: Uuid,
    ) -> Result<organizations::Model, DbErr> {
        let txn = self.db.begin().await?;

        let now = chrono::Utc::now().into();
        let org_id = Uuid::new_v4();

        // Create organization
        let org = organizations::ActiveModel {
            id: Set(org_id),
            name: Set(name.to_string()),
            slug: Set(slug.to_string()),
            base_currency: Set(base_currency.to_string()),
            timezone: Set(timezone.to_string()),
            settings: Set(json!({})),
            is_active: Set(true),
            subscription_tier: Set(SubscriptionTier::Starter),
            subscription_status: Set(SubscriptionStatus::Trialing),
            trial_ends_at: Set(Some((chrono::Utc::now() + chrono::Duration::days(14)).into())),
            subscription_ends_at: Set(None),
            payment_provider: Set(None),
            payment_customer_id: Set(None),
            payment_subscription_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let org = org.insert(&txn).await?;

        // Add owner to organization
        let org_user = organization_users::ActiveModel {
            user_id: Set(owner_id),
            organization_id: Set(org_id),
            role: Set(UserRole::Owner),
            approval_limit: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        org_user.insert(&txn).await?;

        txn.commit().await?;

        Ok(org)
    }

    /// Adds a user to an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database insert fails.
    pub async fn add_user(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        role: UserRole,
        approval_limit: Option<Decimal>,
    ) -> Result<organization_users::Model, DbErr> {
        let now = chrono::Utc::now().into();

        let org_user = organization_users::ActiveModel {
            user_id: Set(user_id),
            organization_id: Set(org_id),
            role: Set(role),
            approval_limit: Set(approval_limit),
            created_at: Set(now),
            updated_at: Set(now),
        };

        org_user.insert(&self.db).await
    }

    /// Gets all users in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_users(
        &self,
        org_id: Uuid,
    ) -> Result<Vec<(users::Model, organization_users::Model)>, DbErr> {
        organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .find_also_related(users::Entity)
            .all(&self.db)
            .await
            .map(|results| {
                results
                    .into_iter()
                    .filter_map(|(ou, user)| user.map(|u| (u, ou)))
                    .collect()
            })
    }

    /// Gets a user's membership in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_user_membership(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<organization_users::Model>, DbErr> {
        organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
    }

    /// Checks if a user is a member of an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn is_member(&self, org_id: Uuid, user_id: Uuid) -> Result<bool, DbErr> {
        let count = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(org_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Checks if a user has a specific role or higher in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn has_role(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        required_role: UserRole,
    ) -> Result<bool, DbErr> {
        let membership = self.get_user_membership(org_id, user_id).await?;

        Ok(membership.map_or(false, |m| role_level(&m.role) >= role_level(&required_role)))
    }
}

/// Returns the privilege level of a role (higher = more privileges).
const fn role_level(role: &UserRole) -> u8 {
    match role {
        UserRole::Owner => 100,
        UserRole::Admin => 80,
        UserRole::Approver => 60,
        UserRole::Accountant => 40,
        UserRole::Submitter => 30,
        UserRole::Viewer => 20,
    }
}
