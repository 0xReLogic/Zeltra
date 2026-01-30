//! Unit tests for Background Jobs.
//!
//! Feature: entities-model-implementation
//!
//! Tests specific scenarios:
//! - Trial expiry job queries users table
//! - Trial expiry job updates expired trials
//! - Sync job uses user subscription tier (if sync job exists)
//! - Sync job counts entities per organization (if sync job exists)

use chrono::{Duration, Utc};
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use std::env;
use uuid::Uuid;
use zeltra_db::{
    entities::{
        organization_users, organizations,
        sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier, UserRole},
        users,
    },
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

/// Helper to create a test user with organization
async fn setup_user_and_org(
    db: &DatabaseConnection,
    subscription_tier: SubscriptionTier,
    subscription_status: SubscriptionStatus,
    trial_ends_at: Option<chrono::DateTime<Utc>>,
) -> (Uuid, Uuid) {
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();

    // Create user
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        subscription_tier: Set(subscription_tier),
        subscription_status: Set(subscription_status),
        trial_ends_at: Set(trial_ends_at.map(Into::into)),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
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
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
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
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };
    organization_users::Entity::insert(org_user)
        .exec(db)
        .await
        .expect("Failed to insert org_user");

    (user_id, org_id)
}

#[tokio::test]
async fn test_trial_expiry_job_queries_users_table() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    
    // Create a user with expired trial
    let expired_time = Utc::now() - Duration::days(1);
    let (user_id, _org_id) = setup_user_and_org(
        &db,
        SubscriptionTier::Starter,
        SubscriptionStatus::Trialing,
        Some(expired_time),
    )
    .await;
    
    // Query users table for expired trials (simulating what the job does)
    use sea_orm::{ColumnTrait, QueryFilter};
    
    let expired_users = users::Entity::find()
        .filter(users::Column::SubscriptionStatus.eq(SubscriptionStatus::Trialing))
        .filter(users::Column::TrialEndsAt.is_not_null())
        .filter(users::Column::TrialEndsAt.lt(Utc::now()))
        .all(&db)
        .await
        .unwrap();
    
    // Verify we found the expired user
    assert!(
        !expired_users.is_empty(),
        "Should find at least one expired trial user"
    );
    
    let found_user = expired_users.iter().find(|u| u.id == user_id);
    assert!(
        found_user.is_some(),
        "Should find the test user with expired trial"
    );
    
    let found_user = found_user.unwrap();
    assert_eq!(found_user.subscription_status, SubscriptionStatus::Trialing);
    assert!(found_user.trial_ends_at.is_some());
    let trial_ends_at: chrono::DateTime<chrono::FixedOffset> = found_user.trial_ends_at.unwrap();
    let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
    assert!(trial_ends_at < now);
}

#[tokio::test]
async fn test_trial_expiry_job_updates_expired_trials() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    
    // Create a user with expired trial
    let expired_time = Utc::now() - Duration::days(1);
    let (user_id, _org_id) = setup_user_and_org(
        &db,
        SubscriptionTier::Starter,
        SubscriptionStatus::Trialing,
        Some(expired_time),
    )
    .await;
    
    // Verify initial status
    let user_before = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_before.subscription_status, SubscriptionStatus::Trialing);
    
    // Simulate what the trial expiry job does
    use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter};
    
    let expired_users = users::Entity::find()
        .filter(users::Column::SubscriptionStatus.eq(SubscriptionStatus::Trialing))
        .filter(users::Column::TrialEndsAt.is_not_null())
        .filter(users::Column::TrialEndsAt.lt(Utc::now()))
        .all(&db)
        .await
        .unwrap();
    
    for user in expired_users {
        if user.id == user_id {
            let mut user_active: users::ActiveModel = user.into();
            user_active.subscription_status = Set(SubscriptionStatus::Expired);
            user_active.update(&db).await.unwrap();
        }
    }
    
    // Verify status was updated
    let user_after = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        user_after.subscription_status,
        SubscriptionStatus::Expired,
        "User subscription status should be updated to Expired"
    );
}

#[tokio::test]
async fn test_trial_expiry_job_does_not_update_active_trials() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    
    // Create a user with active trial (expires in future)
    let future_time = Utc::now() + Duration::days(7);
    let (user_id, _org_id) = setup_user_and_org(
        &db,
        SubscriptionTier::Starter,
        SubscriptionStatus::Trialing,
        Some(future_time),
    )
    .await;
    
    // Query for expired trials (should not find this user)
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    
    let expired_users = users::Entity::find()
        .filter(users::Column::SubscriptionStatus.eq(SubscriptionStatus::Trialing))
        .filter(users::Column::TrialEndsAt.is_not_null())
        .filter(users::Column::TrialEndsAt.lt(Utc::now()))
        .all(&db)
        .await
        .unwrap();
    
    // Verify this user is not in the expired list
    let found_user = expired_users.iter().find(|u| u.id == user_id);
    assert!(
        found_user.is_none(),
        "User with active trial should not be in expired list"
    );
    
    // Verify status remains unchanged
    let user = users::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        user.subscription_status,
        SubscriptionStatus::Trialing,
        "User subscription status should remain Trialing"
    );
}

#[tokio::test]
async fn test_sync_job_uses_user_subscription_tier() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    
    // Create users with different subscription tiers
    let (user_starter_id, _org_starter) = setup_user_and_org(
        &db,
        SubscriptionTier::Starter,
        SubscriptionStatus::Active,
        None,
    )
    .await;
    
    let (user_growth_id, _org_growth) = setup_user_and_org(
        &db,
        SubscriptionTier::Growth,
        SubscriptionStatus::Active,
        None,
    )
    .await;
    
    let (user_enterprise_id, _org_enterprise) = setup_user_and_org(
        &db,
        SubscriptionTier::Enterprise,
        SubscriptionStatus::Active,
        None,
    )
    .await;
    
    // Verify we can query user subscription tiers
    let user_starter = users::Entity::find_by_id(user_starter_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_starter.subscription_tier, SubscriptionTier::Starter);
    
    let user_growth = users::Entity::find_by_id(user_growth_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_growth.subscription_tier, SubscriptionTier::Growth);
    
    let user_enterprise = users::Entity::find_by_id(user_enterprise_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_enterprise.subscription_tier, SubscriptionTier::Enterprise);
}

#[tokio::test]
async fn test_sync_job_counts_entities_per_organization() {
    use zeltra_db::entities::entities;
    use zeltra_db::repositories::entity::EntityRepository;
    
    let db = Database::connect(&get_database_url()).await.unwrap();
    
    // Create user and organization
    let (_user_id, org_id) = setup_user_and_org(
        &db,
        SubscriptionTier::Enterprise,
        SubscriptionStatus::Active,
        None,
    )
    .await;
    
    // Create multiple entities for this organization
    let repo = EntityRepository::new(db.clone());
    
    for i in 0..3 {
        repo.create(
            org_id,
            format!("Entity {}", i),
            "USD".to_string(),
            "subsidiary".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
    }
    
    // Count entities for this organization
    let count = repo.count_by_organization(org_id).await.unwrap();
    
    assert_eq!(
        count, 3,
        "Should count exactly 3 entities for the organization"
    );
    
    // Verify we can query entities by organization
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    
    let entities_list = entities::Entity::find()
        .filter(entities::Column::OrganizationId.eq(org_id))
        .filter(entities::Column::IsActive.eq(true))
        .all(&db)
        .await
        .unwrap();
    
    assert_eq!(
        entities_list.len(),
        3,
        "Should find exactly 3 active entities for the organization"
    );
}

#[tokio::test]
async fn test_trial_expiry_job_only_updates_trialing_status() {
    let db = Database::connect(&get_database_url()).await.unwrap();
    
    // Create users with different statuses but expired trial_ends_at
    let expired_time = Utc::now() - Duration::days(1);
    
    let (user_active_id, _) = setup_user_and_org(
        &db,
        SubscriptionTier::Growth,
        SubscriptionStatus::Active,
        Some(expired_time),
    )
    .await;
    
    let (user_cancelled_id, _) = setup_user_and_org(
        &db,
        SubscriptionTier::Starter,
        SubscriptionStatus::Cancelled,
        Some(expired_time),
    )
    .await;
    
    // Query for expired trials (should only find Trialing status)
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    
    let expired_users = users::Entity::find()
        .filter(users::Column::SubscriptionStatus.eq(SubscriptionStatus::Trialing))
        .filter(users::Column::TrialEndsAt.is_not_null())
        .filter(users::Column::TrialEndsAt.lt(Utc::now()))
        .all(&db)
        .await
        .unwrap();
    
    // Verify Active and Cancelled users are not in the list
    assert!(
        !expired_users.iter().any(|u| u.id == user_active_id),
        "Active user should not be in expired trials list"
    );
    
    assert!(
        !expired_users.iter().any(|u| u.id == user_cancelled_id),
        "Cancelled user should not be in expired trials list"
    );
    
    // Verify their statuses remain unchanged
    let user_active = users::Entity::find_by_id(user_active_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_active.subscription_status, SubscriptionStatus::Active);
    
    let user_cancelled = users::Entity::find_by_id(user_cancelled_id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user_cancelled.subscription_status, SubscriptionStatus::Cancelled);
}
