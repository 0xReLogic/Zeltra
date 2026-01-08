//! Integration tests for Row-Level Security (RLS) tenant isolation.
//!
//! These tests verify that RLS policies correctly isolate data between tenants.
//! Requires a running `PostgreSQL` database with migrations applied.

#![allow(clippy::similar_names)]

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;
use zeltra_db::{
    entities::{fiscal_years, organizations, users},
    rls::RlsConnection,
};

/// Get database URL for superuser (used for setup/cleanup).
fn get_admin_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

/// Get database URL for app user (non-superuser, subject to RLS).
fn get_app_database_url() -> String {
    std::env::var("APP_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://zeltra_app:zeltra_app_password@localhost:5432/zeltra_dev".to_string()
    })
}

/// Setup test data: create 2 organizations with fiscal years.
async fn setup_test_data(db: &DatabaseConnection) -> (Uuid, Uuid, Uuid, Uuid) {
    // Create test user first
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

    // Create Organization A
    let org_a_id = Uuid::new_v4();
    let org_a = organizations::ActiveModel {
        id: Set(org_a_id),
        name: Set("Organization A".to_string()),
        slug: Set(format!("org-a-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    org_a
        .insert(db)
        .await
        .expect("Failed to create Organization A");

    // Create Organization B
    let org_b_id = Uuid::new_v4();
    let org_b = organizations::ActiveModel {
        id: Set(org_b_id),
        name: Set("Organization B".to_string()),
        slug: Set(format!("org-b-{}", Uuid::new_v4())),
        base_currency: Set("EUR".to_string()),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    org_b
        .insert(db)
        .await
        .expect("Failed to create Organization B");

    // Create Fiscal Year for Org A
    let fy_a_id = Uuid::new_v4();
    let fy_a = fiscal_years::ActiveModel {
        id: Set(fy_a_id),
        organization_id: Set(org_a_id),
        name: Set("FY2026-A".to_string()),
        start_date: Set(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        ..Default::default()
    };
    fy_a.insert(db)
        .await
        .expect("Failed to create Fiscal Year A");

    // Create Fiscal Year for Org B
    let fy_b_id = Uuid::new_v4();
    let fy_b = fiscal_years::ActiveModel {
        id: Set(fy_b_id),
        organization_id: Set(org_b_id),
        name: Set("FY2026-B".to_string()),
        start_date: Set(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        ..Default::default()
    };
    fy_b.insert(db)
        .await
        .expect("Failed to create Fiscal Year B");

    (org_a_id, org_b_id, fy_a_id, fy_b_id)
}

/// Cleanup test data after tests.
async fn cleanup_test_data(db: &DatabaseConnection, org_a_id: Uuid, org_b_id: Uuid) {
    // Delete fiscal years (cascade from org delete)
    // Delete organizations
    organizations::Entity::delete_by_id(org_a_id)
        .exec(db)
        .await
        .ok();
    organizations::Entity::delete_by_id(org_b_id)
        .exec(db)
        .await
        .ok();
}

#[tokio::test]
async fn test_rls_isolates_fiscal_years_between_tenants() {
    // Use admin connection for setup
    let admin_db = Database::connect(&get_admin_database_url())
        .await
        .expect("Failed to connect to database as admin");

    let (org_a_id, org_b_id, fy_a_id, fy_b_id) = setup_test_data(&admin_db).await;

    // Use app connection (non-superuser) for RLS tests
    let db = Database::connect(&get_app_database_url())
        .await
        .expect("Failed to connect to database as app user");

    // Test 1: With Org A context, should only see Org A's fiscal year
    {
        let rls = RlsConnection::new(&db, org_a_id)
            .await
            .expect("Failed to create RLS connection for Org A");

        let fiscal_years = fiscal_years::Entity::find()
            .all(rls.transaction())
            .await
            .expect("Failed to query fiscal years");

        // Should only see Org A's fiscal year
        assert_eq!(
            fiscal_years.len(),
            1,
            "Org A should see exactly 1 fiscal year"
        );
        assert_eq!(
            fiscal_years[0].id, fy_a_id,
            "Org A should see its own fiscal year"
        );
        assert_eq!(
            fiscal_years[0].organization_id, org_a_id,
            "Fiscal year should belong to Org A"
        );

        rls.rollback().await.expect("Failed to rollback");
    }

    // Test 2: With Org B context, should only see Org B's fiscal year
    {
        let rls = RlsConnection::new(&db, org_b_id)
            .await
            .expect("Failed to create RLS connection for Org B");

        let fiscal_years = fiscal_years::Entity::find()
            .all(rls.transaction())
            .await
            .expect("Failed to query fiscal years");

        // Should only see Org B's fiscal year
        assert_eq!(
            fiscal_years.len(),
            1,
            "Org B should see exactly 1 fiscal year"
        );
        assert_eq!(
            fiscal_years[0].id, fy_b_id,
            "Org B should see its own fiscal year"
        );
        assert_eq!(
            fiscal_years[0].organization_id, org_b_id,
            "Fiscal year should belong to Org B"
        );

        rls.rollback().await.expect("Failed to rollback");
    }

    // Test 3: Org A cannot access Org B's fiscal year by ID
    {
        let rls = RlsConnection::new(&db, org_a_id)
            .await
            .expect("Failed to create RLS connection for Org A");

        let fy_b = fiscal_years::Entity::find_by_id(fy_b_id)
            .one(rls.transaction())
            .await
            .expect("Query should succeed");

        // Should NOT find Org B's fiscal year
        assert!(
            fy_b.is_none(),
            "Org A should NOT be able to access Org B's fiscal year by ID"
        );

        rls.rollback().await.expect("Failed to rollback");
    }

    // Cleanup using admin connection
    cleanup_test_data(&admin_db, org_a_id, org_b_id).await;
}

#[tokio::test]
async fn test_rls_with_empty_context_returns_nothing() {
    // Use admin connection for setup
    let admin_db = Database::connect(&get_admin_database_url())
        .await
        .expect("Failed to connect to database as admin");

    let (org_a_id, org_b_id, _, _) = setup_test_data(&admin_db).await;

    // Use app connection (non-superuser) for RLS tests
    let db = Database::connect(&get_app_database_url())
        .await
        .expect("Failed to connect to database as app user");

    // Test with a non-existent organization ID (simulates empty/invalid context)
    let fake_org_id = Uuid::new_v4();
    {
        let rls = RlsConnection::new(&db, fake_org_id)
            .await
            .expect("Failed to create RLS connection");

        let fiscal_years = fiscal_years::Entity::find()
            .all(rls.transaction())
            .await
            .expect("Failed to query fiscal years");

        // Should see nothing with invalid org context
        assert!(
            fiscal_years.is_empty(),
            "Invalid org context should return no fiscal years"
        );

        rls.rollback().await.expect("Failed to rollback");
    }

    // Cleanup using admin connection
    cleanup_test_data(&admin_db, org_a_id, org_b_id).await;
}

#[tokio::test]
async fn test_rls_insert_respects_context() {
    // Use admin connection for setup
    let admin_db = Database::connect(&get_admin_database_url())
        .await
        .expect("Failed to connect to database as admin");

    let (org_a_id, org_b_id, _, _) = setup_test_data(&admin_db).await;

    // Use app connection (non-superuser) for RLS tests
    let db = Database::connect(&get_app_database_url())
        .await
        .expect("Failed to connect to database as app user");

    // Create a new fiscal year within Org A's context
    let new_fy_id = Uuid::new_v4();
    {
        let rls = RlsConnection::new(&db, org_a_id)
            .await
            .expect("Failed to create RLS connection for Org A");

        let new_fy = fiscal_years::ActiveModel {
            id: Set(new_fy_id),
            organization_id: Set(org_a_id),
            name: Set("FY2027-A".to_string()),
            start_date: Set(chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()),
            end_date: Set(chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap()),
            ..Default::default()
        };
        new_fy
            .insert(rls.transaction())
            .await
            .expect("Failed to insert fiscal year");

        rls.commit().await.expect("Failed to commit");
    }

    // Verify Org A can see the new fiscal year
    {
        let rls = RlsConnection::new(&db, org_a_id)
            .await
            .expect("Failed to create RLS connection for Org A");

        let fiscal_years = fiscal_years::Entity::find()
            .all(rls.transaction())
            .await
            .expect("Failed to query fiscal years");

        // Should see 2 fiscal years now (original + new)
        assert_eq!(
            fiscal_years.len(),
            2,
            "Org A should see 2 fiscal years after insert"
        );

        rls.rollback().await.expect("Failed to rollback");
    }

    // Verify Org B still cannot see Org A's fiscal years
    {
        let rls = RlsConnection::new(&db, org_b_id)
            .await
            .expect("Failed to create RLS connection for Org B");

        let fiscal_years = fiscal_years::Entity::find()
            .all(rls.transaction())
            .await
            .expect("Failed to query fiscal years");

        // Should still only see 1 fiscal year (Org B's own)
        assert_eq!(
            fiscal_years.len(),
            1,
            "Org B should still see only 1 fiscal year"
        );

        rls.rollback().await.expect("Failed to rollback");
    }

    // Cleanup the new fiscal year using admin connection
    fiscal_years::Entity::delete_by_id(new_fy_id)
        .exec(&admin_db)
        .await
        .ok();

    // Cleanup using admin connection
    cleanup_test_data(&admin_db, org_a_id, org_b_id).await;
}
