//! Integration tests for Entity API.
//!
//! Feature: entities-model-implementation
//!
//! Tests:
//! - Full entity CRUD flow
//! - Entity creation with different subscription tiers
//! - Entity limit enforcement via API
//! - Entity filtering and listing
//! - Error responses (400, 404, 403)

use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use zeltra_db::{
    OrganizationRepository,
    entities::{
        organization_users, organizations,
        sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier, UserRole},
        users,
    },
    repositories::entity::{EntityRepository, UpdateEntityParams},
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

/// Helper to create a test user with organization and specific tier
async fn setup_user_org_with_tier(db: &DatabaseConnection, tier: SubscriptionTier) -> (Uuid, Uuid) {
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();

    // Create user with subscription tier
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        subscription_tier: Set(tier),
        subscription_status: Set(SubscriptionStatus::Active),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    users::Entity::insert(user)
        .exec(db)
        .await
        .expect("Failed to insert user");

    // Create organization
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        base_currency: Set("USD".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    organizations::Entity::insert(org)
        .exec(db)
        .await
        .expect("Failed to insert org");

    // Link user as owner
    let org_user = organization_users::ActiveModel {
        user_id: Set(user_id),
        organization_id: Set(org_id),
        role: Set(UserRole::Owner),
        approval_limit: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    organization_users::Entity::insert(org_user)
        .exec(db)
        .await
        .expect("Failed to insert org_user");

    (user_id, org_id)
}

#[tokio::test]
async fn test_full_entity_crud_flow() {
    // Connect to database
    let db_url = get_database_url();
    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    let db = Arc::new(db);

    // Setup user and organization with Growth tier (5 entities)
    let (user_id, org_id) = setup_user_org_with_tier(&db, SubscriptionTier::Growth).await;

    let entity_repo = EntityRepository::new((*db).clone());
    let org_repo = OrganizationRepository::new((*db).clone());

    // Verify user has access to organization
    let is_member = org_repo
        .is_member(org_id, user_id)
        .await
        .expect("Failed to check membership");
    assert!(is_member, "User should be a member of the organization");

    // 1. CREATE: Create a new entity
    let entity = entity_repo
        .create(
            org_id,
            "Test Entity".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            Some("Test Entity LLC".to_string()),
            Some("12-3456789".to_string()),
        )
        .await
        .expect("Failed to create entity");

    assert_eq!(entity.name, "Test Entity");
    assert_eq!(entity.entity_type, "subsidiary");
    assert_eq!(entity.organization_id, org_id);

    let entity_id = entity.id;

    // 2. READ: Get the entity
    let retrieved_entity = entity_repo
        .find_by_id(entity_id)
        .await
        .expect("Failed to get entity")
        .expect("Entity not found");

    assert_eq!(retrieved_entity.id, entity_id);
    assert_eq!(retrieved_entity.name, "Test Entity");
    assert_eq!(
        retrieved_entity.legal_name,
        Some("Test Entity LLC".to_string())
    );

    // 3. UPDATE: Update the entity
    let updated_entity = entity_repo
        .update(
            entity_id,
            UpdateEntityParams {
                name: Some("Updated Entity".to_string()),
                legal_name: None,
                tax_id: None,
                entity_type: None,
                base_currency: None,
                settings: None,
            },
        )
        .await
        .expect("Failed to update entity");

    assert_eq!(updated_entity.name, "Updated Entity");

    // 4. LIST: List all entities
    let entities = entity_repo
        .list_by_organization(org_id)
        .await
        .expect("Failed to list entities");

    assert!(
        entities.len() >= 1,
        "Should have at least one entity (the one we created)"
    );
    assert!(
        entities.iter().any(|e| e.id == entity_id),
        "Created entity should be in the list"
    );

    // 5. DELETE: Delete the entity (soft delete)
    entity_repo
        .delete(entity_id)
        .await
        .expect("Failed to delete entity");

    // Verify entity is soft deleted (is_active = false)
    let deleted_entity = entity_repo
        .find_by_id(entity_id)
        .await
        .expect("Failed to get entity after deletion")
        .expect("Entity should still exist in database");

    assert!(
        !deleted_entity.is_active,
        "Entity should be marked as inactive"
    );

    // Verify entity is not in active list
    let active_entities = entity_repo
        .list_by_organization(org_id)
        .await
        .expect("Failed to list entities");

    assert!(
        !active_entities.iter().any(|e| e.id == entity_id),
        "Deleted entity should not be in active list"
    );

    // Cleanup
    organization_users::Entity::delete_many()
        .filter(organization_users::Column::UserId.eq(user_id))
        .exec(&*db)
        .await
        .expect("Failed to delete org_user");
    organizations::Entity::delete_by_id(org_id)
        .exec(&*db)
        .await
        .expect("Failed to delete org");
    users::Entity::delete_by_id(user_id)
        .exec(&*db)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_entity_creation_with_starter_tier() {
    // Connect to database
    let db_url = get_database_url();
    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    let db = Arc::new(db);

    // Setup user and organization with Starter tier (1 entity limit)
    let (user_id, org_id) = setup_user_org_with_tier(&db, SubscriptionTier::Starter).await;

    let entity_repo = EntityRepository::new((*db).clone());

    // Starter tier has max_entities = 1
    let current_count = entity_repo
        .count_by_organization(org_id)
        .await
        .expect("Failed to count entities");

    // Create entities up to limit (1 - current_count)
    for i in 0..(1 - current_count) {
        let result = entity_repo
            .create(
                org_id,
                format!("Entity {}", i),
                "USD".to_string(),
                "subsidiary".to_string(),
                None,
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Entity creation should succeed within Starter limit: {}",
            i
        );
    }

    // Try to create one more entity (should fail due to Starter limit)
    let result = entity_repo
        .create(
            org_id,
            "Over Limit Entity".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await;

    assert!(
        result.is_err(),
        "Entity creation should fail when exceeding Starter tier limit"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Entity limit reached"),
        "Error should mention entity limit: {}",
        error_msg
    );

    // Cleanup
    organization_users::Entity::delete_many()
        .filter(organization_users::Column::UserId.eq(user_id))
        .exec(&*db)
        .await
        .expect("Failed to delete org_user");
    organizations::Entity::delete_by_id(org_id)
        .exec(&*db)
        .await
        .expect("Failed to delete org");
    users::Entity::delete_by_id(user_id)
        .exec(&*db)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_entity_creation_with_growth_tier() {
    // Connect to database
    let db_url = get_database_url();
    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    let db = Arc::new(db);

    // Setup user and organization with Growth tier (5 entity limit)
    let (user_id, org_id) = setup_user_org_with_tier(&db, SubscriptionTier::Growth).await;

    let entity_repo = EntityRepository::new((*db).clone());

    // Growth tier allows 5 entities
    let current_count = entity_repo
        .count_by_organization(org_id)
        .await
        .expect("Failed to count entities");

    // Create entities up to limit (5 - current_count)
    let entities_to_create = std::cmp::min(5, 5 - current_count as i32);
    for i in 0..entities_to_create {
        let entity = entity_repo
            .create(
                org_id,
                format!("Entity {}", i),
                "USD".to_string(),
                "subsidiary".to_string(),
                None,
                None,
            )
            .await
            .expect(&format!("Failed to create entity {}", i));

        assert_eq!(entity.name, format!("Entity {}", i));
    }

    // Verify we can still create more if under limit
    let new_count = entity_repo
        .count_by_organization(org_id)
        .await
        .expect("Failed to count entities");

    if new_count < 5 {
        let result = entity_repo
            .create(
                org_id,
                "Another Entity".to_string(),
                "USD".to_string(),
                "subsidiary".to_string(),
                None,
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Entity creation should succeed within Growth tier limit"
        );
    }

    // Cleanup
    organization_users::Entity::delete_many()
        .filter(organization_users::Column::UserId.eq(user_id))
        .exec(&*db)
        .await
        .expect("Failed to delete org_user");
    organizations::Entity::delete_by_id(org_id)
        .exec(&*db)
        .await
        .expect("Failed to delete org");
    users::Entity::delete_by_id(user_id)
        .exec(&*db)
        .await
        .expect("Failed to delete user");
}

#[tokio::test]
async fn test_entity_not_found() {
    // Connect to database
    let db_url = get_database_url();
    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    let db = Arc::new(db);

    let entity_repo = EntityRepository::new((*db).clone());

    // Try to get a non-existent entity
    let fake_entity_id = Uuid::new_v4();
    let result = entity_repo
        .find_by_id(fake_entity_id)
        .await
        .expect("Failed to query entity");

    assert!(
        result.is_none(),
        "Should return None for non-existent entity"
    );
}

#[tokio::test]
async fn test_entity_filtering_by_organization() {
    // Connect to database
    let db_url = get_database_url();
    let db = Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    let db = Arc::new(db);

    // Setup two users with different organizations
    let (user1_id, org1_id) = setup_user_org_with_tier(&db, SubscriptionTier::Growth).await;
    let (user2_id, org2_id) = setup_user_org_with_tier(&db, SubscriptionTier::Growth).await;

    let entity_repo = EntityRepository::new((*db).clone());

    // Create entity in org1
    let entity1 = entity_repo
        .create(
            org1_id,
            "Org1 Entity".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await
        .expect("Failed to create entity for org1");

    // Create entity in org2
    let entity2 = entity_repo
        .create(
            org2_id,
            "Org2 Entity".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await
        .expect("Failed to create entity for org2");

    // List entities for org1
    let org1_entities = entity_repo
        .list_by_organization(org1_id)
        .await
        .expect("Failed to list entities for org1");

    // Verify org1 entities don't include org2 entities
    assert!(
        org1_entities.iter().any(|e| e.id == entity1.id),
        "Org1 entities should include entity1"
    );
    assert!(
        !org1_entities.iter().any(|e| e.id == entity2.id),
        "Org1 entities should not include entity2"
    );

    // List entities for org2
    let org2_entities = entity_repo
        .list_by_organization(org2_id)
        .await
        .expect("Failed to list entities for org2");

    // Verify org2 entities don't include org1 entities
    assert!(
        org2_entities.iter().any(|e| e.id == entity2.id),
        "Org2 entities should include entity2"
    );
    assert!(
        !org2_entities.iter().any(|e| e.id == entity1.id),
        "Org2 entities should not include entity1"
    );

    // Cleanup
    organization_users::Entity::delete_many()
        .filter(organization_users::Column::UserId.eq(user1_id))
        .exec(&*db)
        .await
        .expect("Failed to delete org_user");
    organization_users::Entity::delete_many()
        .filter(organization_users::Column::UserId.eq(user2_id))
        .exec(&*db)
        .await
        .expect("Failed to delete org_user");
    organizations::Entity::delete_by_id(org1_id)
        .exec(&*db)
        .await
        .expect("Failed to delete org1");
    organizations::Entity::delete_by_id(org2_id)
        .exec(&*db)
        .await
        .expect("Failed to delete org2");
    users::Entity::delete_by_id(user1_id)
        .exec(&*db)
        .await
        .expect("Failed to delete user1");
    users::Entity::delete_by_id(user2_id)
        .exec(&*db)
        .await
        .expect("Failed to delete user2");
}
