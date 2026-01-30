//! Unit tests for Entity Repository.
//!
//! Feature: entities-model-implementation
//!
//! Tests specific examples and edge cases:
//! - Create entity with valid data
//! - Create entity with duplicate name (should fail)
//! - Create entity exceeding Starter limit (should fail)
//! - Create entity exceeding Growth limit (should fail)
//! - List entities filters by is_active
//! - Update entity with partial data
//! - Soft delete doesn't remove from database

use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;
use zeltra_db::{
    entities::{
        entities, organization_users, organizations,
        sea_orm_active_enums::{SubscriptionTier, UserRole},
        users,
    },
    repositories::entity::{EntityRepository, UpdateEntityParams},
};

/// Helper to create a test user with a specific subscription tier
async fn setup_user_with_tier(db: &DatabaseConnection, tier: SubscriptionTier) -> (Uuid, Uuid) {
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();

    // Create user with subscription tier
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        subscription_tier: Set(tier),
        subscription_status: Set(
            zeltra_db::entities::sea_orm_active_enums::SubscriptionStatus::Active,
        ),
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

fn get_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        std::env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

#[tokio::test]
async fn test_create_entity_with_valid_data() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;
    let repo = EntityRepository::new(db.clone());

    let result = repo
        .create(
            org_id,
            "Test Entity".to_string(),
            "USD".to_string(),
            "main".to_string(),
            Some("Test Legal Name".to_string()),
            Some("TAX123".to_string()),
        )
        .await;

    assert!(
        result.is_ok(),
        "Failed to create entity: {:?}",
        result.err()
    );

    let entity = result.unwrap();
    assert_eq!(entity.name, "Test Entity");
    assert_eq!(entity.legal_name, Some("Test Legal Name".to_string()));
    assert_eq!(entity.tax_id, Some("TAX123".to_string()));
    assert_eq!(entity.entity_type, "main");
    assert_eq!(entity.base_currency, "USD");
    assert!(entity.is_active);
    assert_eq!(entity.organization_id, org_id);
}

#[tokio::test]
async fn test_create_entity_with_duplicate_name() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Growth).await;
    let repo = EntityRepository::new(db.clone());

    // Create first entity
    let result1 = repo
        .create(
            org_id,
            "Duplicate Name".to_string(),
            "USD".to_string(),
            "main".to_string(),
            None,
            None,
        )
        .await;
    assert!(result1.is_ok(), "First entity creation should succeed");

    // Try to create second entity with same name
    let result2 = repo
        .create(
            org_id,
            "Duplicate Name".to_string(),
            "EUR".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await;

    assert!(
        result2.is_err(),
        "Should fail to create entity with duplicate name"
    );

    // Verify error message contains "unique" or "duplicate"
    let error_msg = format!("{:?}", result2.err().unwrap());
    assert!(
        error_msg.to_lowercase().contains("unique")
            || error_msg.to_lowercase().contains("duplicate"),
        "Error should mention unique constraint violation: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_entity_exceeding_starter_limit() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;
    let repo = EntityRepository::new(db.clone());

    // Starter tier: max_entities = 1
    // Create 1 entity (should succeed)
    let result = repo
        .create(
            org_id,
            "Entity 0".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await;
    assert!(
        result.is_ok(),
        "Should create entity 0 within limit: {:?}",
        result.err()
    );

    // Try to create 2nd entity (should fail)
    let result = repo
        .create(
            org_id,
            "Entity 1".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await;

    assert!(
        result.is_err(),
        "Should fail to create entity exceeding Starter limit"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("Entity limit reached"),
        "Error should mention entity limit: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_create_entity_exceeding_growth_limit() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Growth).await;
    let repo = EntityRepository::new(db.clone());

    // Growth tier: max_entities = 5
    // Create 5 entities (should succeed)
    for i in 0..5 {
        let result = repo
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
            "Should create entity {} within limit: {:?}",
            i,
            result.err()
        );
    }

    // Try to create 6th entity (should fail)
    let result = repo
        .create(
            org_id,
            "Entity 5".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await;

    assert!(
        result.is_err(),
        "Should fail to create entity exceeding Growth limit"
    );

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("Entity limit reached"),
        "Error should mention entity limit: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_list_entities_filters_by_is_active() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Growth).await;
    let repo = EntityRepository::new(db.clone());

    // Create 3 active entities
    let entity1 = repo
        .create(
            org_id,
            "Active Entity 1".to_string(),
            "USD".to_string(),
            "main".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

    let entity2 = repo
        .create(
            org_id,
            "Active Entity 2".to_string(),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

    let entity3 = repo
        .create(
            org_id,
            "Active Entity 3".to_string(),
            "USD".to_string(),
            "branch".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

    // Soft delete entity2
    repo.delete(entity2.id).await.unwrap();

    // List entities (should only return active ones)
    let entities = repo.list_by_organization(org_id).await.unwrap();

    assert_eq!(entities.len(), 2, "Should only return active entities");
    assert!(
        entities.iter().any(|e| e.id == entity1.id),
        "Should include entity1"
    );
    assert!(
        !entities.iter().any(|e| e.id == entity2.id),
        "Should not include soft-deleted entity2"
    );
    assert!(
        entities.iter().any(|e| e.id == entity3.id),
        "Should include entity3"
    );
}

#[tokio::test]
async fn test_update_entity_with_partial_data() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;
    let repo = EntityRepository::new(db.clone());

    // Create entity
    let entity = repo
        .create(
            org_id,
            "Original Name".to_string(),
            "USD".to_string(),
            "main".to_string(),
            Some("Original Legal Name".to_string()),
            Some("TAX123".to_string()),
        )
        .await
        .unwrap();

    // Update only name and tax_id (partial update)
    let params = UpdateEntityParams {
        name: Some("Updated Name".to_string()),
        legal_name: None, // Don't update
        tax_id: Some("TAX456".to_string()),
        entity_type: None,   // Don't update
        base_currency: None, // Don't update
        settings: None,      // Don't update
    };

    let updated = repo.update(entity.id, params).await.unwrap();

    // Verify updated fields
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.tax_id, Some("TAX456".to_string()));

    // Verify unchanged fields
    assert_eq!(updated.legal_name, Some("Original Legal Name".to_string()));
    assert_eq!(updated.entity_type, "main");
    assert_eq!(updated.base_currency, "USD");
    assert_eq!(updated.organization_id, org_id);
    assert!(updated.is_active);
}

#[tokio::test]
async fn test_soft_delete_doesnt_remove_from_database() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;
    let repo = EntityRepository::new(db.clone());

    // Create entity
    let entity = repo
        .create(
            org_id,
            "To Be Deleted".to_string(),
            "USD".to_string(),
            "main".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

    let entity_id = entity.id;

    // Soft delete
    repo.delete(entity_id).await.unwrap();

    // Verify entity still exists in database
    let deleted_entity = repo.find_by_id(entity_id).await.unwrap();
    assert!(
        deleted_entity.is_some(),
        "Entity should still exist in database after soft delete"
    );

    let deleted_entity = deleted_entity.unwrap();
    assert!(!deleted_entity.is_active, "Entity should be inactive");
    assert_eq!(deleted_entity.id, entity_id);
    assert_eq!(deleted_entity.name, "To Be Deleted");

    // Verify entity doesn't appear in list
    let entities = repo.list_by_organization(org_id).await.unwrap();
    assert!(
        !entities.iter().any(|e| e.id == entity_id),
        "Soft-deleted entity should not appear in list"
    );

    // Verify count doesn't include soft-deleted entity
    let count = repo.count_by_organization(org_id).await.unwrap();
    assert_eq!(count, 0, "Count should not include soft-deleted entities");

    // Verify we can query it directly from database
    let direct_query = entities::Entity::find_by_id(entity_id)
        .one(&db)
        .await
        .unwrap();
    assert!(
        direct_query.is_some(),
        "Should be able to query soft-deleted entity directly"
    );
    assert!(!direct_query.unwrap().is_active);
}

#[tokio::test]
async fn test_enterprise_tier_has_no_entity_limit() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Enterprise).await;
    let repo = EntityRepository::new(db.clone());

    // Create 10 entities (more than Growth limit of 5)
    for i in 0..10 {
        let result = repo
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
            "Enterprise tier should allow unlimited entities, failed at {}: {:?}",
            i,
            result.err()
        );
    }

    let count = repo.count_by_organization(org_id).await.unwrap();
    assert_eq!(count, 10, "Should have created all 10 entities");
}

#[tokio::test]
async fn test_update_nonexistent_entity() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let repo = EntityRepository::new(db.clone());

    let nonexistent_id = Uuid::new_v4();
    let params = UpdateEntityParams {
        name: Some("New Name".to_string()),
        ..Default::default()
    };

    let result = repo.update(nonexistent_id, params).await;

    assert!(result.is_err(), "Should fail to update nonexistent entity");

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("Entity not found"),
        "Error should mention entity not found: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_delete_nonexistent_entity() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let repo = EntityRepository::new(db.clone());

    let nonexistent_id = Uuid::new_v4();
    let result = repo.delete(nonexistent_id).await;

    assert!(result.is_err(), "Should fail to delete nonexistent entity");

    let error_msg = format!("{:?}", result.err().unwrap());
    assert!(
        error_msg.contains("Entity not found"),
        "Error should mention entity not found: {}",
        error_msg
    );
}
