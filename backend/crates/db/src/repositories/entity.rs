//! Entity repository for database operations.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::sea_orm_active_enums::SubscriptionTier;
use crate::entities::{entities, organization_users, tier_limits, users};

/// Parameters for updating an entity.
#[derive(Debug, Default)]
pub struct UpdateEntityParams {
    pub name: Option<String>,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub entity_type: Option<String>,
    pub base_currency: Option<String>,
    pub settings: Option<serde_json::Value>,
}

/// Entity repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct EntityRepository {
    db: DatabaseConnection,
}

impl EntityRepository {
    /// Creates a new entity repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new entity with tier limit validation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database insert fails
    /// - The entity limit for the user's tier is exceeded
    /// - An entity with the same name already exists in the organization
    pub async fn create(
        &self,
        organization_id: Uuid,
        name: String,
        base_currency: String,
        entity_type: String,
        legal_name: Option<String>,
        tax_id: Option<String>,
    ) -> Result<entities::Model, DbErr> {
        // Get the organization owner's subscription tier
        let tier = self.get_org_owner_tier(organization_id).await?;

        // Check entity limit for the tier
        let current_count = self.count_by_organization(organization_id).await?;

        // Check tier limits (using max_entities field)
        let tier_limit = tier_limits::Entity::find_by_id(tier)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Tier limits not found".to_string()))?;

        // Enterprise tier has unlimited entities (NULL max_entities)
        if let Some(max_entities) = tier_limit.max_entities
            && current_count >= max_entities as i64
        {
            return Err(DbErr::Custom(
                "Entity limit reached for your tier".to_string(),
            ));
        }

        // Create the entity
        let now = chrono::Utc::now().into();
        let entity = entities::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(organization_id),
            name: Set(name),
            legal_name: Set(legal_name),
            tax_id: Set(tax_id),
            entity_type: Set(entity_type),
            base_currency: Set(base_currency),
            is_active: Set(true),
            settings: Set(serde_json::json!({})),
            created_at: Set(now),
            updated_at: Set(now),
        };

        entity.insert(&self.db).await
    }

    /// Lists all active entities for an organization, ordered by created_at.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<entities::Model>, DbErr> {
        entities::Entity::find()
            .filter(entities::Column::OrganizationId.eq(organization_id))
            .filter(entities::Column::IsActive.eq(true))
            .order_by_asc(entities::Column::CreatedAt)
            .all(&self.db)
            .await
    }

    /// Finds an entity by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_by_id(&self, entity_id: Uuid) -> Result<Option<entities::Model>, DbErr> {
        entities::Entity::find_by_id(entity_id).one(&self.db).await
    }

    /// Updates an entity.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database update fails
    /// - The entity is not found
    pub async fn update(
        &self,
        entity_id: Uuid,
        params: UpdateEntityParams,
    ) -> Result<entities::Model, DbErr> {
        let entity = entities::Entity::find_by_id(entity_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Entity not found".to_string()))?;

        let mut active_entity: entities::ActiveModel = entity.into();

        if let Some(n) = params.name {
            active_entity.name = Set(n);
        }
        if let Some(ln) = params.legal_name {
            active_entity.legal_name = Set(Some(ln));
        }
        if let Some(ti) = params.tax_id {
            active_entity.tax_id = Set(Some(ti));
        }
        if let Some(et) = params.entity_type {
            active_entity.entity_type = Set(et);
        }
        if let Some(bc) = params.base_currency {
            active_entity.base_currency = Set(bc);
        }
        if let Some(s) = params.settings {
            active_entity.settings = Set(s);
        }

        active_entity.updated_at = Set(chrono::Utc::now().into());

        active_entity.update(&self.db).await
    }

    /// Soft deletes an entity by setting is_active to false.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database update fails
    /// - The entity is not found
    pub async fn delete(&self, entity_id: Uuid) -> Result<(), DbErr> {
        let entity = entities::Entity::find_by_id(entity_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Entity not found".to_string()))?;

        let mut active_entity: entities::ActiveModel = entity.into();
        active_entity.is_active = Set(false);
        active_entity.updated_at = Set(chrono::Utc::now().into());

        active_entity.update(&self.db).await?;
        Ok(())
    }

    /// Counts active entities for an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn count_by_organization(&self, organization_id: Uuid) -> Result<i64, DbErr> {
        entities::Entity::find()
            .filter(entities::Column::OrganizationId.eq(organization_id))
            .filter(entities::Column::IsActive.eq(true))
            .count(&self.db)
            .await
            .map(|count| count as i64)
    }

    /// Gets the subscription tier of the organization's owner.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database query fails
    /// - The organization or owner is not found
    async fn get_org_owner_tier(&self, organization_id: Uuid) -> Result<SubscriptionTier, DbErr> {
        // Find the owner of the organization
        let owner = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(organization_id))
            .filter(organization_users::Column::Role.eq("owner"))
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Organization owner not found".to_string()))?;

        // Get the user's subscription tier
        let user = users::Entity::find_by_id(owner.user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("User not found".to_string()))?;

        Ok(user.subscription_tier)
    }
}
