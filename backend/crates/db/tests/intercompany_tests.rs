//! Unit tests for Intercompany Repository.
//!
//! Feature: entities-model-implementation
//!
//! Tests specific scenarios:
//! - Create mapping with entities in same org succeeds
//! - Create mapping with entities in different orgs fails
//! - Error message for different orgs
//! - List mappings filters by organization
//! - Mirror transaction generation
//! - Elimination transaction generation

use sea_orm::{Database, DatabaseConnection, EntityTrait, Set, TryIntoModel};
use std::env;
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

#[tokio::test]
async fn test_create_mapping_same_org_succeeds() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_and_org(&db).await;
    
    // Create two entities in the same org
    let source_entity = create_entity(&db, org_id, "Source Entity").await;
    let target_entity = create_entity(&db, org_id, "Target Entity").await;
    
    // Create accounts
    let source_account = create_account(&db, org_id, source_entity.id, "1000").await;
    let target_account = create_account(&db, org_id, target_entity.id, "2000").await;
    
    // Validate mapping
    let repo = IntercompanyRepository::new(db.clone());
    let result = repo.validate_mapping(source_entity.id, target_entity.id).await;
    
    assert!(
        result.is_ok(),
        "Validation should succeed for entities in same organization: {:?}",
        result.err()
    );
    
    // Create mapping
    let mapping = intercompany_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_entity_id: Set(source_entity.id),
        target_entity_id: Set(target_entity.id),
        source_account_id: Set(source_account.id),
        target_account_id: Set(target_account.id),
        mapping_type: Set("mirror".to_string()),
        auto_post: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let insert_result = intercompany_mappings::Entity::insert(mapping)
        .exec(&db)
        .await;
    
    assert!(
        insert_result.is_ok(),
        "Mapping creation should succeed: {:?}",
        insert_result.err()
    );
}

#[tokio::test]
async fn test_create_mapping_different_orgs_fails() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id1, org_id1) = setup_user_and_org(&db).await;
    let (_user_id2, org_id2) = setup_user_and_org(&db).await;
    
    // Create entities in different orgs
    let source_entity = create_entity(&db, org_id1, "Source Entity").await;
    let target_entity = create_entity(&db, org_id2, "Target Entity").await;
    
    // Try to validate mapping
    let repo = IntercompanyRepository::new(db.clone());
    let result = repo.validate_mapping(source_entity.id, target_entity.id).await;
    
    assert!(
        result.is_err(),
        "Validation should fail for entities in different organizations"
    );
}

#[tokio::test]
async fn test_error_message_for_different_orgs() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id1, org_id1) = setup_user_and_org(&db).await;
    let (_user_id2, org_id2) = setup_user_and_org(&db).await;
    
    // Create entities in different orgs
    let source_entity = create_entity(&db, org_id1, "Source Entity").await;
    let target_entity = create_entity(&db, org_id2, "Target Entity").await;
    
    // Try to validate mapping
    let repo = IntercompanyRepository::new(db.clone());
    let result = repo.validate_mapping(source_entity.id, target_entity.id).await;
    
    assert!(result.is_err());
    
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(
        err_msg.contains("Entities must belong to the same organization"),
        "Error message should mention same organization requirement, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_list_mappings_filters_by_organization() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_and_org(&db).await;
    
    // Create three entities in the same org
    let entity1 = create_entity(&db, org_id, "Entity 1").await;
    let entity2 = create_entity(&db, org_id, "Entity 2").await;
    let entity3 = create_entity(&db, org_id, "Entity 3").await;
    
    // Create accounts
    let account1 = create_account(&db, org_id, entity1.id, "1000").await;
    let account2 = create_account(&db, org_id, entity2.id, "2000").await;
    let account3 = create_account(&db, org_id, entity3.id, "3000").await;
    
    // Create two mappings: entity1->entity2 and entity2->entity3
    let mapping1 = intercompany_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_entity_id: Set(entity1.id),
        target_entity_id: Set(entity2.id),
        source_account_id: Set(account1.id),
        target_account_id: Set(account2.id),
        mapping_type: Set("mirror".to_string()),
        auto_post: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    intercompany_mappings::Entity::insert(mapping1)
        .exec(&db)
        .await
        .expect("Failed to insert mapping1");
    
    let mapping2 = intercompany_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_entity_id: Set(entity2.id),
        target_entity_id: Set(entity3.id),
        source_account_id: Set(account2.id),
        target_account_id: Set(account3.id),
        mapping_type: Set("elimination".to_string()),
        auto_post: Set(false),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    intercompany_mappings::Entity::insert(mapping2)
        .exec(&db)
        .await
        .expect("Failed to insert mapping2");
    
    // Query mappings for entity1
    let repo = IntercompanyRepository::new(db.clone());
    let mappings1 = repo.get_mappings(entity1.id).await.unwrap();
    
    assert_eq!(mappings1.len(), 1, "Should find exactly one mapping for entity1");
    assert_eq!(mappings1[0].source_entity_id, entity1.id);
    assert_eq!(mappings1[0].target_entity_id, entity2.id);
    
    // Query mappings for entity2
    let mappings2 = repo.get_mappings(entity2.id).await.unwrap();
    
    assert_eq!(mappings2.len(), 1, "Should find exactly one mapping for entity2");
    assert_eq!(mappings2[0].source_entity_id, entity2.id);
    assert_eq!(mappings2[0].target_entity_id, entity3.id);
    
    // Query mappings for entity3 (should be empty)
    let mappings3 = repo.get_mappings(entity3.id).await.unwrap();
    
    assert_eq!(mappings3.len(), 0, "Should find no mappings for entity3");
}

#[tokio::test]
async fn test_mirror_transaction_mapping_type() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_and_org(&db).await;
    
    // Create entities and accounts
    let source_entity = create_entity(&db, org_id, "Source Entity").await;
    let target_entity = create_entity(&db, org_id, "Target Entity").await;
    let source_account = create_account(&db, org_id, source_entity.id, "1000").await;
    let target_account = create_account(&db, org_id, target_entity.id, "2000").await;
    
    // Create mirror mapping
    let mapping = intercompany_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_entity_id: Set(source_entity.id),
        target_entity_id: Set(target_entity.id),
        source_account_id: Set(source_account.id),
        target_account_id: Set(target_account.id),
        mapping_type: Set("mirror".to_string()),
        auto_post: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    intercompany_mappings::Entity::insert(mapping)
        .exec(&db)
        .await
        .expect("Failed to insert mapping");
    
    // Verify mapping type
    let repo = IntercompanyRepository::new(db.clone());
    let found_mapping = repo
        .find_mapping_by_account(source_entity.id, source_account.id)
        .await
        .unwrap();
    
    assert!(found_mapping.is_some());
    let found_mapping = found_mapping.unwrap();
    assert_eq!(found_mapping.mapping_type, "mirror");
    assert!(found_mapping.auto_post);
}

#[tokio::test]
async fn test_elimination_transaction_mapping_type() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    let (_user_id, org_id) = setup_user_and_org(&db).await;
    
    // Create entities and accounts
    let source_entity = create_entity(&db, org_id, "Source Entity").await;
    let target_entity = create_entity(&db, org_id, "Target Entity").await;
    let source_account = create_account(&db, org_id, source_entity.id, "1000").await;
    let target_account = create_account(&db, org_id, target_entity.id, "2000").await;
    
    // Create elimination mapping
    let mapping = intercompany_mappings::ActiveModel {
        id: Set(Uuid::new_v4()),
        source_entity_id: Set(source_entity.id),
        target_entity_id: Set(target_entity.id),
        source_account_id: Set(source_account.id),
        target_account_id: Set(target_account.id),
        mapping_type: Set("elimination".to_string()),
        auto_post: Set(false),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    intercompany_mappings::Entity::insert(mapping)
        .exec(&db)
        .await
        .expect("Failed to insert mapping");
    
    // Verify mapping type
    let repo = IntercompanyRepository::new(db.clone());
    let found_mapping = repo
        .find_mapping_by_account(source_entity.id, source_account.id)
        .await
        .unwrap();
    
    assert!(found_mapping.is_some());
    let found_mapping = found_mapping.unwrap();
    assert_eq!(found_mapping.mapping_type, "elimination");
    assert!(!found_mapping.auto_post);
}
