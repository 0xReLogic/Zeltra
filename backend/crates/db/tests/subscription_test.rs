//! Integration tests for subscription and tier management.
//!
//! Tests verify tier limits, feature checks, and usage tracking.

#![allow(clippy::similar_names)]

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;
use zeltra_db::{
    entities::organizations,
    entities::sea_orm_active_enums::SubscriptionTier,
    repositories::{Feature, ResourceLimit, SubscriptionRepository},
};

use organizations::Entity as OrgEntity;

/// Get database URL for admin operations.
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

/// Create a test organization with default starter tier.
async fn create_test_org(db: &DatabaseConnection) -> Uuid {
    let org_id = Uuid::new_v4();
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {org_id}")),
        slug: Set(format!("test-org-{org_id}")),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    org.insert(db).await.expect("Failed to create test org");
    org_id
}

/// Create a test organization with specific tier.
async fn create_test_org_with_tier(db: &DatabaseConnection, tier: SubscriptionTier) -> Uuid {
    let org_id = Uuid::new_v4();
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {org_id}")),
        slug: Set(format!("test-org-{org_id}")),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        subscription_tier: Set(tier),
        ..Default::default()
    };
    org.insert(db).await.expect("Failed to create test org");
    org_id
}

/// Cleanup test organization.
async fn cleanup_org(db: &DatabaseConnection, org_id: Uuid) {
    OrgEntity::delete_by_id(org_id).exec(db).await.ok();
}

#[tokio::test]
async fn test_get_tier_limits() {
    use zeltra_db::entities::sea_orm_active_enums::SubscriptionTier;

    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    // Test starter tier
    let starter = SubscriptionRepository::get_tier_limits(&db, SubscriptionTier::Starter)
        .await
        .expect("Failed to get starter limits");

    assert!(starter.is_some());
    let starter = starter.unwrap();
    assert_eq!(starter.max_users, Some(50));
    assert!(!starter.has_multi_currency);
    assert!(!starter.has_simulation);

    // Test growth tier
    let growth = SubscriptionRepository::get_tier_limits(&db, SubscriptionTier::Growth)
        .await
        .expect("Failed to get growth limits");

    assert!(growth.is_some());
    let growth = growth.unwrap();
    assert_eq!(growth.max_users, Some(200));
    assert!(growth.has_multi_currency);
    assert!(!growth.has_simulation);
    assert!(growth.has_api_access);

    // Test enterprise tier
    let enterprise = SubscriptionRepository::get_tier_limits(&db, SubscriptionTier::Enterprise)
        .await
        .expect("Failed to get enterprise limits");

    assert!(enterprise.is_some());
    let enterprise = enterprise.unwrap();
    assert!(enterprise.max_users.is_none()); // Unlimited
    assert!(enterprise.has_multi_currency);
    assert!(enterprise.has_simulation);
}

#[tokio::test]
async fn test_has_feature_starter_tier() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    let org_id = create_test_org(&db).await;

    // Starter tier should NOT have these features
    let has_multi_currency =
        SubscriptionRepository::has_feature(&db, org_id, Feature::MultiCurrency)
            .await
            .expect("Failed to check feature");
    assert!(
        !has_multi_currency,
        "Starter should not have multi-currency"
    );

    let has_simulation = SubscriptionRepository::has_feature(&db, org_id, Feature::Simulation)
        .await
        .expect("Failed to check feature");
    assert!(!has_simulation, "Starter should not have simulation");

    let has_sso = SubscriptionRepository::has_feature(&db, org_id, Feature::Sso)
        .await
        .expect("Failed to check feature");
    assert!(!has_sso, "Starter should not have SSO");

    cleanup_org(&db, org_id).await;
}

#[tokio::test]
async fn test_has_feature_growth_tier() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    // Create org with Growth tier
    let org_id = create_test_org_with_tier(&db, SubscriptionTier::Growth).await;

    // Growth tier should have these features
    let has_multi_currency =
        SubscriptionRepository::has_feature(&db, org_id, Feature::MultiCurrency)
            .await
            .expect("Failed to check feature");
    assert!(has_multi_currency, "Growth should have multi-currency");

    let has_simulation = SubscriptionRepository::has_feature(&db, org_id, Feature::Simulation)
        .await
        .expect("Failed to check feature");
    assert!(!has_simulation, "Growth should not have simulation");

    let has_sso = SubscriptionRepository::has_feature(&db, org_id, Feature::Sso)
        .await
        .expect("Failed to check feature");
    assert!(!has_sso, "Growth should not have SSO");

    let has_api = SubscriptionRepository::has_feature(&db, org_id, Feature::ApiAccess)
        .await
        .expect("Failed to check feature");
    assert!(has_api, "Growth should have API access");

    cleanup_org(&db, org_id).await;
}

#[tokio::test]
async fn test_check_user_limit() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    let org_id = create_test_org(&db).await;

    // Check user limit - should be allowed (0 users, limit 50)
    let result = SubscriptionRepository::check_limit(&db, org_id, ResourceLimit::Users)
        .await
        .expect("Failed to check limit");

    assert!(result.allowed, "Should be allowed with 0 users");
    assert_eq!(result.current, 0);
    assert_eq!(result.limit, Some(50));

    cleanup_org(&db, org_id).await;
}

#[tokio::test]
async fn test_usage_tracking() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    let org_id = create_test_org(&db).await;

    // Get or create usage - should create new record
    let usage = SubscriptionRepository::get_or_create_current_usage(&db, org_id)
        .await
        .expect("Failed to get usage");

    assert_eq!(usage.organization_id, org_id);
    assert_eq!(usage.transaction_count, 0);
    assert_eq!(usage.api_call_count, 0);

    // Increment transaction count
    SubscriptionRepository::increment_transaction_count(&db, org_id)
        .await
        .expect("Failed to increment");

    let usage = SubscriptionRepository::get_or_create_current_usage(&db, org_id)
        .await
        .expect("Failed to get usage");
    assert_eq!(usage.transaction_count, 1);

    // Increment API call count
    SubscriptionRepository::increment_api_call_count(&db, org_id)
        .await
        .expect("Failed to increment");

    let usage = SubscriptionRepository::get_or_create_current_usage(&db, org_id)
        .await
        .expect("Failed to get usage");
    assert_eq!(usage.api_call_count, 1);

    cleanup_org(&db, org_id).await;
}

#[tokio::test]
async fn test_trial_expiry_check() {
    use chrono::{Duration, Utc};
    use zeltra_db::entities::sea_orm_active_enums::SubscriptionStatus;

    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    let org_id = create_test_org(&db).await;

    // Default trial_ends_at is NULL, which means expired per our logic
    let is_expired = SubscriptionRepository::is_trial_expired(&db, org_id)
        .await
        .expect("Failed to check trial");
    assert!(is_expired, "Org with NULL trial_ends_at should be expired");

    // Set trial_ends_at to future date
    let org = organizations::Entity::find_by_id(org_id)
        .one(&db)
        .await
        .expect("Failed to find org")
        .expect("Org not found");

    let mut active: organizations::ActiveModel = org.into();
    active.trial_ends_at = Set(Some((Utc::now() + Duration::days(14)).into()));
    active.update(&db).await.expect("Failed to update org");

    // Now should NOT be expired
    let is_expired = SubscriptionRepository::is_trial_expired(&db, org_id)
        .await
        .expect("Failed to check trial");
    assert!(
        !is_expired,
        "Org with future trial_ends_at should not be expired"
    );

    // Set trial_ends_at to past date
    let org = organizations::Entity::find_by_id(org_id)
        .one(&db)
        .await
        .expect("Failed to find org")
        .expect("Org not found");

    let mut active: organizations::ActiveModel = org.into();
    active.trial_ends_at = Set(Some((Utc::now() - Duration::days(1)).into()));
    active.update(&db).await.expect("Failed to update org");

    // Now SHOULD be expired
    let is_expired = SubscriptionRepository::is_trial_expired(&db, org_id)
        .await
        .expect("Failed to check trial");
    assert!(is_expired, "Org with past trial_ends_at should be expired");

    // Change status to active - should not be expired regardless of trial_ends_at
    let org = organizations::Entity::find_by_id(org_id)
        .one(&db)
        .await
        .expect("Failed to find org")
        .expect("Org not found");

    let mut active: organizations::ActiveModel = org.into();
    active.subscription_status = Set(SubscriptionStatus::Active);
    active.update(&db).await.expect("Failed to update org");

    let is_expired = SubscriptionRepository::is_trial_expired(&db, org_id)
        .await
        .expect("Failed to check trial");
    assert!(
        !is_expired,
        "Active org should not be considered trial expired"
    );

    cleanup_org(&db, org_id).await;
}

#[tokio::test]
async fn test_upgrade_tier() {
    use zeltra_db::entities::sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier};

    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect");

    let org_id = create_test_org(&db).await;

    // Upgrade to growth tier
    SubscriptionRepository::upgrade_tier(&db, org_id, SubscriptionTier::Growth)
        .await
        .expect("Failed to upgrade");

    // Verify upgrade
    let org = OrgEntity::find_by_id(org_id)
        .one(&db)
        .await
        .expect("Failed to find org")
        .expect("Org not found");

    assert_eq!(org.subscription_tier, SubscriptionTier::Growth);
    assert_eq!(org.subscription_status, SubscriptionStatus::Active);

    // Now should have multi-currency feature
    let has_multi_currency =
        SubscriptionRepository::has_feature(&db, org_id, Feature::MultiCurrency)
            .await
            .expect("Failed to check feature");
    assert!(has_multi_currency, "Growth tier should have multi-currency");

    cleanup_org(&db, org_id).await;
}
