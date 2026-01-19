//! Tier enforcement tests.
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::env;
use uuid::Uuid;
use zeltra_db::entities::organizations;
use zeltra_db::entities::sea_orm_active_enums::SubscriptionTier;

async fn setup_db() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    });
    zeltra_db::connect(&database_url).await.unwrap()
}

#[tokio::test]
async fn test_tier_limit_enforcement() {
    let db = setup_db().await;

    // 1. Create a Starter organization
    let org_id = Uuid::new_v4();
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set("Starter Org".to_string()),
        slug: Set(format!("starter-org-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        subscription_tier: Set(SubscriptionTier::Starter),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };
    org.insert(&db).await.unwrap();

    let org_repo = zeltra_db::OrganizationRepository::new(db.clone());
    let limits = org_repo
        .get_tier_limits(org_id)
        .await
        .unwrap()
        .expect("Limits should exist");

    assert_eq!(limits.tier, SubscriptionTier::Starter);
    assert!(!limits.has_auto_accruals);
    assert!(!limits.has_intercompany_hub);
    assert!(!limits.has_multi_currency);
    assert_eq!(limits.max_dimensions, 2);

    // 2. Create a Growth organization
    let growth_org_id = Uuid::new_v4();
    let growth_org = organizations::ActiveModel {
        id: Set(growth_org_id),
        name: Set("Growth Org".to_string()),
        slug: Set(format!("growth-org-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        subscription_tier: Set(SubscriptionTier::Growth),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };
    growth_org.insert(&db).await.unwrap();

    let growth_limits = org_repo
        .get_tier_limits(growth_org_id)
        .await
        .unwrap()
        .expect("Limits should exist");
    assert_eq!(growth_limits.tier, SubscriptionTier::Growth);
    assert!(!growth_limits.has_auto_accruals);
    assert!(!growth_limits.has_intercompany_hub);
    assert!(growth_limits.has_multi_currency);
    assert_eq!(growth_limits.max_dimensions, 999_999);

    // 3. Create an Enterprise organization
    let ent_org_id = Uuid::new_v4();
    let ent_org = organizations::ActiveModel {
        id: Set(ent_org_id),
        name: Set("Enterprise Org".to_string()),
        slug: Set(format!("ent-org-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        subscription_tier: Set(SubscriptionTier::Enterprise),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        updated_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };
    ent_org.insert(&db).await.unwrap();

    let ent_limits = org_repo
        .get_tier_limits(ent_org_id)
        .await
        .unwrap()
        .expect("Limits should exist");
    assert_eq!(ent_limits.tier, SubscriptionTier::Enterprise);
    assert!(ent_limits.has_auto_accruals);
    assert!(ent_limits.has_intercompany_hub);
    assert!(ent_limits.has_multi_currency);
    assert_eq!(ent_limits.max_dimensions, 999_999);
}
