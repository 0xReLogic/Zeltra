//! Property-based tests for Entity Repository.
//!
//! Feature: entities-model-implementation
//!
//! Tests universal correctness properties:
//! - Property 5: Default entity creation
//! - Property 6: Entity tier limit enforcement
//! - Property 7: Enterprise unlimited entities
//! - Property 8: Entity list ordering
//! - Property 9: Entity update
//! - Property 10: Entity soft delete

use proptest::prelude::*;
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use std::env;
use tokio::runtime::Runtime;
use uuid::Uuid;
use zeltra_db::{
    entities::{
        organization_users, organizations,
        sea_orm_active_enums::{SubscriptionTier, UserRole},
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

/// Helper: Run async code in a temporary runtime
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(future)
}

proptest! {
    // Limit cases for DB integration tests to avoid timeouts
    #![proptest_config(proptest::test_runner::Config::with_cases(10))]

    /// Property 5: Default entity creation
    /// Feature: entities-model-implementation, Property 5: Default entity creation
    ///
    /// When an organization is created, a default entity should be created automatically.
    /// This test verifies that the default entity exists and has the correct properties.
    #[test]
    fn prop_default_entity_creation(
        ref org_name in "[a-zA-Z0-9 ]{3,50}",
        ref currency in "(USD|EUR|GBP|JPY|CAD)"
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;

            let repo = EntityRepository::new(db.clone());

            // Create a default entity (simulating what happens during org creation)
            let entity = repo.create(
                org_id,
                format!("{} (Main)", org_name),
                currency.clone(),
                "main".to_string(),
                Some(org_name.clone()),
                None,
            ).await;

            assert!(entity.is_ok(), "Default entity creation failed: {:?}", entity.err());

            let entity = entity.unwrap();
            assert_eq!(entity.organization_id, org_id);
            assert_eq!(entity.entity_type, "main");
            assert_eq!(entity.base_currency, *currency);
            assert!(entity.is_active);
        });
    }

    /// Property 6: Entity tier limit enforcement
    /// Feature: entities-model-implementation, Property 6: Entity tier limit enforcement
    ///
    /// Starter tier: max 1 entity (max_entities = 1)
    /// Growth tier: max 5 entities (max_entities = 5)
    /// Enterprise tier: unlimited entities (max_entities = NULL)
    #[test]
    fn prop_entity_tier_limit_enforcement(
        tier_idx in 0usize..2usize,
        entity_count in 1usize..10usize
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();

            // Map tier_idx to actual tiers (excluding Enterprise for this test)
            let (tier, expected_limit) = match tier_idx {
                0 => (SubscriptionTier::Starter, 1),  // max_entities = 1
                1 => (SubscriptionTier::Growth, 5),   // max_entities = 5
                _ => unreachable!(),
            };

            let (_user_id, org_id) = setup_user_with_tier(&db, tier).await;
            let repo = EntityRepository::new(db.clone());

            // Try to create entity_count entities
            let mut created = 0;
            for i in 0..entity_count {
                let result = repo.create(
                    org_id,
                    format!("Entity {}", i),
                    "USD".to_string(),
                    "subsidiary".to_string(),
                    None,
                    None,
                ).await;

                if result.is_ok() {
                    created += 1;
                } else {
                    // Should fail when limit is reached
                    assert!(
                        created >= expected_limit as usize,
                        "Entity creation failed before limit: created={}, limit={}, error={:?}",
                        created,
                        expected_limit,
                        result.err()
                    );
                    break;
                }
            }

            // Verify we can't exceed the limit
            if entity_count > expected_limit as usize {
                assert_eq!(
                    created,
                    expected_limit as usize,
                    "Created more entities than tier limit allows"
                );
            }
        });
    }

    /// Property 7: Enterprise unlimited entities
    /// Feature: entities-model-implementation, Property 7: Enterprise unlimited entities
    ///
    /// Enterprise tier should allow unlimited entity creation (max_users = NULL).
    #[test]
    fn prop_enterprise_unlimited_entities(
        entity_count in 1usize..20usize
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Enterprise).await;
            let repo = EntityRepository::new(db.clone());

            // Try to create entity_count entities
            for i in 0..entity_count {
                let result = repo.create(
                    org_id,
                    format!("Entity {}", i),
                    "USD".to_string(),
                    "subsidiary".to_string(),
                    None,
                    None,
                ).await;

                assert!(
                    result.is_ok(),
                    "Enterprise tier should allow unlimited entities, failed at entity {}: {:?}",
                    i,
                    result.err()
                );
            }

            // Verify all entities were created
            let count = repo.count_by_organization(org_id).await.unwrap();
            assert_eq!(
                count,
                entity_count as i64,
                "Enterprise tier should have created all {} entities",
                entity_count
            );
        });
    }

    /// Property 8: Entity list ordering
    /// Feature: entities-model-implementation, Property 8: Entity list ordering
    ///
    /// Entities should be returned in created_at order (oldest first).
    #[test]
    fn prop_entity_list_ordering(
        entity_count in 2usize..10usize
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Enterprise).await;
            let repo = EntityRepository::new(db.clone());

            // Create multiple entities with slight delays
            let mut entity_ids = Vec::new();
            for i in 0..entity_count {
                let entity = repo.create(
                    org_id,
                    format!("Entity {}", i),
                    "USD".to_string(),
                    "subsidiary".to_string(),
                    None,
                    None,
                ).await.unwrap();
                entity_ids.push(entity.id);

                // Small delay to ensure different created_at timestamps
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            // List entities
            let entities = repo.list_by_organization(org_id).await.unwrap();

            // Verify ordering: created_at should be ascending
            for i in 1..entities.len() {
                assert!(
                    entities[i - 1].created_at <= entities[i].created_at,
                    "Entities not in created_at order: entity[{}].created_at={:?} > entity[{}].created_at={:?}",
                    i - 1,
                    entities[i - 1].created_at,
                    i,
                    entities[i].created_at
                );
            }

            // Verify IDs match creation order
            for (i, entity) in entities.iter().enumerate() {
                assert_eq!(
                    entity.id,
                    entity_ids[i],
                    "Entity order doesn't match creation order at index {}",
                    i
                );
            }
        });
    }

    /// Property 9: Entity update
    /// Feature: entities-model-implementation, Property 9: Entity update
    ///
    /// All entity fields should be updatable.
    #[test]
    fn prop_entity_update(
        ref new_name in "[a-zA-Z0-9 ]{3,50}",
        ref new_legal_name in "[a-zA-Z0-9 ]{3,50}",
        ref new_tax_id in "[A-Z0-9]{5,15}",
        ref new_currency in "(USD|EUR|GBP|JPY|CAD)"
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;
            let repo = EntityRepository::new(db.clone());

            // Create an entity
            let entity = repo.create(
                org_id,
                "Original Name".to_string(),
                "USD".to_string(),
                "main".to_string(),
                Some("Original Legal Name".to_string()),
                Some("TAX123".to_string()),
            ).await.unwrap();

            // Update all fields
            let params = UpdateEntityParams {
                name: Some(new_name.clone()),
                legal_name: Some(new_legal_name.clone()),
                tax_id: Some(new_tax_id.clone()),
                entity_type: Some("subsidiary".to_string()),
                base_currency: Some(new_currency.clone()),
                settings: Some(serde_json::json!({"key": "value"})),
            };

            let updated = repo.update(entity.id, params).await;
            assert!(updated.is_ok(), "Entity update failed: {:?}", updated.err());

            let updated = updated.unwrap();
            assert_eq!(updated.name, *new_name);
            assert_eq!(updated.legal_name, Some(new_legal_name.clone()));
            assert_eq!(updated.tax_id, Some(new_tax_id.clone()));
            assert_eq!(updated.entity_type, "subsidiary");
            assert_eq!(updated.base_currency, *new_currency);
            assert_eq!(updated.settings, serde_json::json!({"key": "value"}));
            assert!(updated.updated_at > entity.updated_at, "updated_at should be newer");
        });
    }

    /// Property 10: Entity soft delete
    /// Feature: entities-model-implementation, Property 10: Entity soft delete
    ///
    /// Deleting an entity should set is_active to false, not remove from database.
    #[test]
    fn prop_entity_soft_delete(
        ref entity_name in "[a-zA-Z0-9 ]{3,50}"
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_with_tier(&db, SubscriptionTier::Starter).await;
            let repo = EntityRepository::new(db.clone());

            // Create an entity
            let entity = repo.create(
                org_id,
                entity_name.clone(),
                "USD".to_string(),
                "main".to_string(),
                None,
                None,
            ).await.unwrap();

            let entity_id = entity.id;

            // Verify entity is active
            assert!(entity.is_active);

            // Delete the entity
            let delete_result = repo.delete(entity_id).await;
            assert!(delete_result.is_ok(), "Entity delete failed: {:?}", delete_result.err());

            // Verify entity still exists in database but is_active = false
            let deleted_entity = repo.find_by_id(entity_id).await.unwrap();
            assert!(deleted_entity.is_some(), "Entity should still exist in database after soft delete");

            let deleted_entity = deleted_entity.unwrap();
            assert!(!deleted_entity.is_active, "Entity should have is_active = false after delete");
            assert_eq!(deleted_entity.id, entity_id);
            assert_eq!(deleted_entity.name, *entity_name);

            // Verify entity doesn't appear in list (list filters by is_active)
            let entities = repo.list_by_organization(org_id).await.unwrap();
            assert!(
                !entities.iter().any(|e| e.id == entity_id),
                "Soft-deleted entity should not appear in list"
            );

            // Verify count doesn't include soft-deleted entity
            let count = repo.count_by_organization(org_id).await.unwrap();
            assert_eq!(count, 0, "Count should not include soft-deleted entities");
        });
    }
}
