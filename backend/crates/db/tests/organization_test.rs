//! Integration tests for Organization repository.
//!
//! Tests CRUD operations including the update (PATCH) functionality.

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;
use zeltra_db::{
    OrganizationRepository,
    entities::{organizations, users},
};

/// Get database URL from environment or use default.
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

/// Create a test user for organization tests.
async fn create_test_user(db: &DatabaseConnection) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Test User".to_string()),
        is_active: Set(true),
        ..Default::default()
    };
    user.insert(db).await.expect("Failed to create test user");
    user_id
}

/// Cleanup test organization.
async fn cleanup_org(db: &DatabaseConnection, org_id: Uuid) {
    organizations::Entity::delete_by_id(org_id)
        .exec(db)
        .await
        .ok();
}

#[tokio::test]
async fn test_organization_update_name() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Original Name",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update only name
    let updated = repo
        .update(org.id, Some("Updated Name"), None, None)
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.base_currency, "USD"); // unchanged
    assert_eq!(updated.timezone, "UTC"); // unchanged

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_currency() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update only currency
    let updated = repo
        .update(org.id, None, Some("IDR"), None)
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "Test Org"); // unchanged
    assert_eq!(updated.base_currency, "IDR");
    assert_eq!(updated.timezone, "UTC"); // unchanged

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_timezone() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update only timezone
    let updated = repo
        .update(org.id, None, None, Some("Asia/Jakarta"))
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "Test Org"); // unchanged
    assert_eq!(updated.base_currency, "USD"); // unchanged
    assert_eq!(updated.timezone, "Asia/Jakarta");

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_multiple_fields() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Original Name",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update all fields
    let updated = repo
        .update(
            org.id,
            Some("New Company Name"),
            Some("EUR"),
            Some("Europe/London"),
        )
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "New Company Name");
    assert_eq!(updated.base_currency, "EUR");
    assert_eq!(updated.timezone, "Europe/London");

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_nonexistent() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = OrganizationRepository::new(db.clone());

    // Try to update non-existent organization
    let result = repo
        .update(Uuid::new_v4(), Some("New Name"), None, None)
        .await
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for non-existent org");
}

#[tokio::test]
async fn test_organization_update_preserves_other_fields() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    let original_slug = org.slug.clone();
    let original_created_at = org.created_at;

    // Update name
    let updated = repo
        .update(org.id, Some("Updated Name"), None, None)
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    // Verify other fields are preserved
    assert_eq!(updated.slug, original_slug, "Slug should not change");
    assert_eq!(
        updated.created_at, original_created_at,
        "created_at should not change"
    );
    assert!(
        updated.updated_at > original_created_at,
        "updated_at should be newer"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}
