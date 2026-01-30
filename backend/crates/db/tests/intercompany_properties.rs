//! Property-based tests for Intercompany Repository.
//!
//! Feature: entities-model-implementation
//!
//! Tests universal correctness properties:
//! - Property 13: Intercompany same-organization validation
//! - Property 14: Intercompany mapping filtering
//! - Property 15: Intercompany transaction processing

use proptest::prelude::*;
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set, TryIntoModel};
use std::env;
use tokio::runtime::Runtime;
use uuid::Uuid;
use zeltra_db::{
    entities::{
        chart_of_accounts, entities, intercompany_mappings, organization_users, organizations,
        sea_orm_active_enums::{AccountType, SubscriptionTier, UserRole},
        users,
    },
    repositories::intercompany::IntercompanyRepository,
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

/// Helper to create a test user with organization
async fn setup_user_and_org(db: &DatabaseConnection) -> (Uuid, Uuid) {
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();

    // Create user
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        subscription_tier: Set(SubscriptionTier::Enterprise),
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

/// Helper to create an entity
async fn create_entity(
    db: &DatabaseConnection,
    org_id: Uuid,
    name: &str,
) -> entities::Model {
    let entity = entities::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        name: Set(name.to_string()),
        legal_name: Set(Some(name.to_string())),
        tax_id: Set(None),
        entity_type: Set("subsidiary".to_string()),
        base_currency: Set("USD".to_string()),
        is_active: Set(true),
        settings: Set(serde_json::json!({})),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    entities::Entity::insert(entity.clone())
        .exec(db)
        .await
        .expect("Failed to insert entity");
    
    entity.try_into_model().unwrap()
}

/// Helper to create an account
async fn create_account(
    db: &DatabaseConnection,
    org_id: Uuid,
    entity_id: Uuid,
    code: &str,
) -> chart_of_accounts::Model {
    let account = chart_of_accounts::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        entity_id: Set(entity_id),
        code: Set(code.to_string()),
        name: Set(format!("Account {}", code)),
        description: Set(None),
        account_type: Set(AccountType::Asset),
        account_subtype: Set(None),
        parent_id: Set(None),
        currency: Set("USD".to_string()),
        is_active: Set(true),
        is_system_account: Set(false),
        allow_direct_posting: Set(true),
        is_bank_account: Set(false),
        bank_account_number: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    chart_of_accounts::Entity::insert(account.clone())
        .exec(db)
        .await
        .expect("Failed to insert account");
    
    account.try_into_model().unwrap()
}

/// Helper: Run async code in a temporary runtime
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(future)
}

proptest! {
    // Limit cases for DB integration tests to avoid timeouts
    #![proptest_config(proptest::test_runner::Config::with_cases(10))]

    /// Property 13: Intercompany same-organization validation
    /// Feature: entities-model-implementation, Property 13: Intercompany same-organization validation
    ///
    /// When creating an intercompany mapping between two entities, the creation should succeed
    /// if both entities belong to the same organization, and should fail with error
    /// "Entities must belong to the same organization" if they belong to different organizations.
    #[test]
    fn prop_intercompany_same_org_validation(
        same_org in proptest::bool::ANY
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id1) = setup_user_and_org(&db).await;
            
            // Create source entity in org1
            let source_entity = create_entity(&db, org_id1, "Source Entity").await;
            let _source_account = create_account(&db, org_id1, source_entity.id, "1000").await;
            
            // Create target entity in same or different org
            let target_org_id = if same_org {
                org_id1
            } else {
                let (_user_id2, org_id2) = setup_user_and_org(&db).await;
                org_id2
            };
            
            let target_entity = create_entity(&db, target_org_id, "Target Entity").await;
            let _target_account = create_account(&db, target_org_id, target_entity.id, "2000").await;
            
            // Try to validate mapping
            let repo = IntercompanyRepository::new(db.clone());
            let result = repo.validate_mapping(source_entity.id, target_entity.id).await;
            
            if same_org {
                assert!(
                    result.is_ok(),
                    "Validation should succeed for entities in same organization: {:?}",
                    result.err()
                );
            } else {
                assert!(
                    result.is_err(),
                    "Validation should fail for entities in different organizations"
                );
                
                let err_msg = format!("{:?}", result.err().unwrap());
                assert!(
                    err_msg.contains("Entities must belong to the same organization"),
                    "Error message should mention same organization requirement, got: {}",
                    err_msg
                );
            }
        });
    }

    /// Property 14: Intercompany mapping filtering
    /// Feature: entities-model-implementation, Property 14: Intercompany mapping filtering
    ///
    /// When listing intercompany mappings for an organization, the results should contain
    /// only mappings where both source_entity_id and target_entity_id belong to entities
    /// in that organization.
    #[test]
    fn prop_intercompany_mapping_filtering(
        mapping_count in 1usize..5usize
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_and_org(&db).await;
            
            // Create multiple entities in the same org
            let mut entities_list = Vec::new();
            let mut accounts_list = Vec::new();
            
            for i in 0..mapping_count + 1 {
                let entity = create_entity(&db, org_id, &format!("Entity {}", i)).await;
                let account = create_account(&db, org_id, entity.id, &format!("{}000", i + 1)).await;
                entities_list.push(entity);
                accounts_list.push(account);
            }
            
            // Create intercompany mappings
            let mut created_mappings = Vec::new();
            for i in 0..mapping_count {
                let mapping = intercompany_mappings::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    source_entity_id: Set(entities_list[i].id),
                    target_entity_id: Set(entities_list[i + 1].id),
                    source_account_id: Set(accounts_list[i].id),
                    target_account_id: Set(accounts_list[i + 1].id),
                    mapping_type: Set("mirror".to_string()),
                    auto_post: Set(true),
                    created_at: Set(chrono::Utc::now().into()),
                    updated_at: Set(chrono::Utc::now().into()),
                };
                intercompany_mappings::Entity::insert(mapping.clone())
                    .exec(&db)
                    .await
                    .expect("Failed to insert mapping");
                created_mappings.push(mapping.try_into_model().unwrap());
            }
            
            // Query mappings for each source entity
            let repo = IntercompanyRepository::new(db.clone());
            for i in 0..mapping_count {
                let mappings = repo.get_mappings(entities_list[i].id).await.unwrap();
                
                // Should find exactly one mapping for this source entity
                assert_eq!(
                    mappings.len(),
                    1,
                    "Should find exactly one mapping for entity {}",
                    i
                );
                
                // Verify the mapping is correct
                assert_eq!(mappings[0].source_entity_id, entities_list[i].id);
                assert_eq!(mappings[0].target_entity_id, entities_list[i + 1].id);
                
                // Verify both entities belong to the same org
                let source_entity = entities::Entity::find_by_id(mappings[0].source_entity_id)
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap();
                let target_entity = entities::Entity::find_by_id(mappings[0].target_entity_id)
                    .one(&db)
                    .await
                    .unwrap()
                    .unwrap();
                
                assert_eq!(
                    source_entity.organization_id,
                    target_entity.organization_id,
                    "Both entities should belong to the same organization"
                );
                assert_eq!(
                    source_entity.organization_id,
                    org_id,
                    "Entities should belong to the test organization"
                );
            }
        });
    }

    /// Property 15: Intercompany transaction processing
    /// Feature: entities-model-implementation, Property 15: Intercompany transaction processing
    ///
    /// When a transaction is posted to an account with an intercompany mapping,
    /// a corresponding mirror or elimination entry should be automatically created
    /// in the target entity according to the mapping_type.
    ///
    /// Note: This property tests the validation and setup for intercompany processing.
    /// The actual transaction processing is tested in integration tests.
    #[test]
    fn prop_intercompany_transaction_processing_setup(
        mapping_type_idx in 0usize..2usize
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_and_org(&db).await;
            
            // Map index to mapping type
            let mapping_type = match mapping_type_idx {
                0 => "mirror",
                1 => "elimination",
                _ => unreachable!(),
            };
            
            // Create source and target entities
            let source_entity = create_entity(&db, org_id, "Source Entity").await;
            let target_entity = create_entity(&db, org_id, "Target Entity").await;
            
            // Create accounts
            let source_account = create_account(&db, org_id, source_entity.id, "1000").await;
            let target_account = create_account(&db, org_id, target_entity.id, "2000").await;
            
            // Create intercompany mapping
            let mapping = intercompany_mappings::ActiveModel {
                id: Set(Uuid::new_v4()),
                source_entity_id: Set(source_entity.id),
                target_entity_id: Set(target_entity.id),
                source_account_id: Set(source_account.id),
                target_account_id: Set(target_account.id),
                mapping_type: Set(mapping_type.to_string()),
                auto_post: Set(true),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
            };
            intercompany_mappings::Entity::insert(mapping.clone())
                .exec(&db)
                .await
                .expect("Failed to insert mapping");
            
            // Verify mapping was created correctly
            let repo = IntercompanyRepository::new(db.clone());
            let found_mapping = repo
                .find_mapping_by_account(source_entity.id, source_account.id)
                .await
                .unwrap();
            
            assert!(
                found_mapping.is_some(),
                "Mapping should be found by source account"
            );
            
            let found_mapping = found_mapping.unwrap();
            assert_eq!(found_mapping.source_entity_id, source_entity.id);
            assert_eq!(found_mapping.target_entity_id, target_entity.id);
            assert_eq!(found_mapping.source_account_id, source_account.id);
            assert_eq!(found_mapping.target_account_id, target_account.id);
            assert_eq!(found_mapping.mapping_type, mapping_type);
            assert!(found_mapping.auto_post);
            
            // Verify entities are in same organization
            let validation = repo.validate_mapping(source_entity.id, target_entity.id).await;
            assert!(
                validation.is_ok(),
                "Validation should succeed for entities in same organization: {:?}",
                validation.err()
            );
        });
    }
}
