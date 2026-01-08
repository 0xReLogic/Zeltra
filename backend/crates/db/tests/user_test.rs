//! Integration tests for User repository.

use sea_orm::Database;
use uuid::Uuid;
use zeltra_db::UserRepository;

/// Get database URL from environment or use default.
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

#[tokio::test]
async fn test_user_create_and_find_by_id() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = UserRepository::new(db.clone());
    let email = format!("test-{}@example.com", Uuid::new_v4());

    // Create user
    let user = repo
        .create(&email, "$argon2id$test_hash", "Test User")
        .await
        .expect("Failed to create user");

    assert_eq!(user.email, email);
    assert_eq!(user.full_name, "Test User");
    assert!(user.is_active);

    // Find by ID
    let found = repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User should exist");

    assert_eq!(found.id, user.id);
    assert_eq!(found.email, email);
}

#[tokio::test]
async fn test_user_find_by_email() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = UserRepository::new(db.clone());
    let email = format!("test-{}@example.com", Uuid::new_v4());

    // Create user
    let user = repo
        .create(&email, "$argon2id$test_hash", "Test User")
        .await
        .expect("Failed to create user");

    // Find by email
    let found = repo
        .find_by_email(&email)
        .await
        .expect("Failed to find user")
        .expect("User should exist");

    assert_eq!(found.id, user.id);
    assert_eq!(found.email, email);
}

#[tokio::test]
async fn test_user_find_by_email_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = UserRepository::new(db.clone());

    // Find non-existent email
    let result = repo
        .find_by_email("nonexistent@example.com")
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_user_find_by_id_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = UserRepository::new(db.clone());

    // Find non-existent ID
    let result = repo
        .find_by_id(Uuid::new_v4())
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_user_email_exists() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = UserRepository::new(db.clone());
    let email = format!("test-{}@example.com", Uuid::new_v4());

    // Check before creation
    let exists_before = repo
        .email_exists(&email)
        .await
        .expect("Query should succeed");
    assert!(!exists_before);

    // Create user
    repo.create(&email, "$argon2id$test_hash", "Test User")
        .await
        .expect("Failed to create user");

    // Check after creation
    let exists_after = repo
        .email_exists(&email)
        .await
        .expect("Query should succeed");
    assert!(exists_after);
}
