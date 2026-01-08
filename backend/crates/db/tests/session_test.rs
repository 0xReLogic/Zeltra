//! Integration tests for Session repository.

use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use uuid::Uuid;
use zeltra_db::{
    SessionRepository,
    entities::{organizations, users},
};

/// Get database URL from environment or use default.
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

/// Create a test user for session tests.
async fn create_test_user(db: &DatabaseConnection) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("session-test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Session Test User".to_string()),
        is_active: Set(true),
        ..Default::default()
    };
    user.insert(db).await.expect("Failed to create test user");
    user_id
}

/// Create a test organization for session tests.
async fn create_test_org(db: &DatabaseConnection) -> Uuid {
    let org_id = Uuid::new_v4();
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set("Session Test Org".to_string()),
        slug: Set(format!("session-test-org-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    org.insert(db).await.expect("Failed to create test org");
    org_id
}

#[tokio::test]
async fn test_session_create() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let org_id = create_test_org(&db).await;
    let repo = SessionRepository::new(db.clone());
    let expires_at = Utc::now() + Duration::days(7);

    let session = repo
        .create(
            user_id,
            org_id,
            "test_refresh_token",
            expires_at,
            Some("Test Agent"),
            Some("127.0.0.1"),
        )
        .await
        .expect("Failed to create session");

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.organization_id, org_id);
    assert_eq!(session.user_agent.as_deref(), Some("Test Agent"));
    assert_eq!(session.ip_address.as_deref(), Some("127.0.0.1"));
    assert!(session.revoked_at.is_none());
}

#[tokio::test]
async fn test_session_find_by_token() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let org_id = create_test_org(&db).await;
    let repo = SessionRepository::new(db.clone());
    let token = format!("find_token_{}", Uuid::new_v4());
    let expires_at = Utc::now() + Duration::days(7);

    // Create session
    let session = repo
        .create(user_id, org_id, &token, expires_at, None, None)
        .await
        .expect("Failed to create session");

    // Find by token
    let found = repo
        .find_by_token(&token)
        .await
        .expect("Query should succeed")
        .expect("Session should exist");

    assert_eq!(found.id, session.id);
}

#[tokio::test]
async fn test_session_find_by_token_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = SessionRepository::new(db.clone());

    let result = repo
        .find_by_token("nonexistent_token")
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_session_revoke() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let org_id = create_test_org(&db).await;
    let repo = SessionRepository::new(db.clone());
    let token = format!("revoke_test_{}", Uuid::new_v4());
    let expires_at = Utc::now() + Duration::days(7);

    // Create session
    let session = repo
        .create(user_id, org_id, &token, expires_at, None, None)
        .await
        .expect("Failed to create session");

    assert!(session.revoked_at.is_none());

    // Revoke session
    repo.revoke(session.id)
        .await
        .expect("Failed to revoke session");

    // Verify revoked - find_by_token should return None for revoked sessions
    let found = repo
        .find_by_token(&token)
        .await
        .expect("Query should succeed");

    assert!(
        found.is_none(),
        "Revoked session should not be found by token"
    );
}

#[tokio::test]
async fn test_session_revoke_all_user_sessions() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let org_id = create_test_org(&db).await;
    let repo = SessionRepository::new(db.clone());
    let expires_at = Utc::now() + Duration::days(7);

    // Create multiple sessions
    let token1 = format!("multi_session_1_{}", Uuid::new_v4());
    let token2 = format!("multi_session_2_{}", Uuid::new_v4());

    repo.create(user_id, org_id, &token1, expires_at, Some("Agent 1"), None)
        .await
        .expect("Failed to create session 1");
    repo.create(user_id, org_id, &token2, expires_at, Some("Agent 2"), None)
        .await
        .expect("Failed to create session 2");

    // Revoke all
    let count = repo
        .revoke_all_user_sessions(user_id)
        .await
        .expect("Failed to revoke all sessions");

    assert!(count >= 2);

    // Verify both revoked
    let session1 = repo.find_by_token(&token1).await.unwrap();
    let session2 = repo.find_by_token(&token2).await.unwrap();

    assert!(session1.is_none(), "Session 1 should be revoked");
    assert!(session2.is_none(), "Session 2 should be revoked");
}

#[tokio::test]
async fn test_session_hash_token() {
    // Same token should produce same hash
    let hash1 = SessionRepository::hash_token("test_token");
    let hash2 = SessionRepository::hash_token("test_token");
    assert_eq!(hash1, hash2);

    // Different tokens should produce different hashes
    let hash3 = SessionRepository::hash_token("different_token");
    assert_ne!(hash1, hash3);
}
