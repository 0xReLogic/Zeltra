#![allow(missing_docs)]
use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use std::env;
use uuid::Uuid;

use zeltra_core::workflow::{ApprovalEngine, ApprovalRule};
use zeltra_db::{
    entities::{
        approval_rules, organization_users,
        sea_orm_active_enums::{TransactionStatus, TransactionType, UserRole},
        transactions,
    },
    repositories::workflow::WorkflowRepository,
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

#[tokio::test]
#[allow(clippy::similar_names)]
async fn test_approval_determinism() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");
    let repo = WorkflowRepository::new(db.clone());
    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();

    // Create two rules with same priority but different roles
    // Rule A: priority 1, admin (ID is smaller)
    // Rule B: priority 1, owner (ID is larger)
    let rule_a_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let rule_b_id = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();

    // Clean up if exists
    let _ = approval_rules::Entity::delete_by_id(rule_a_id)
        .exec(&db)
        .await;
    let _ = approval_rules::Entity::delete_by_id(rule_b_id)
        .exec(&db)
        .await;

    let rule_a = approval_rules::ActiveModel {
        id: Set(rule_a_id),
        organization_id: Set(org_id),
        name: Set("Rule A".to_string()),
        priority: Set(1),
        required_role: Set(UserRole::Admin),
        transaction_types: Set(vec![]),
        is_active: Set(true),
        ..Default::default()
    };
    let rule_b = approval_rules::ActiveModel {
        id: Set(rule_b_id),
        organization_id: Set(org_id),
        name: Set("Rule B".to_string()),
        priority: Set(1),
        required_role: Set(UserRole::Owner),
        transaction_types: Set(vec![]),
        is_active: Set(true),
        ..Default::default()
    };

    approval_rules::Entity::insert(rule_a)
        .exec(&db)
        .await
        .unwrap();
    approval_rules::Entity::insert(rule_b)
        .exec(&db)
        .await
        .unwrap();

    // Fetch rules via repo helper
    let _rules = repo.get_pending_transactions(org_id, Uuid::new_v4()).await; // Triggers rule fetch

    // Actually, I can test ApprovalEngine directly since it's core logic
    let core_rules = vec![
        ApprovalRule {
            id: rule_b_id, // Owner, large ID
            name: "Rule B".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec![],
            required_role: "owner".to_string(),
            priority: 1,
        },
        ApprovalRule {
            id: rule_a_id, // Admin, small ID
            name: "Rule A".to_string(),
            min_amount: None,
            max_amount: None,
            transaction_types: vec![],
            required_role: "admin".to_string(),
            priority: 1,
        },
    ];

    let winner = ApprovalEngine::get_required_approval(&core_rules, "journal", dec!(100));
    assert_eq!(
        winner,
        Some("admin".to_string()),
        "Rule A should win because of smaller UUID for determinism"
    );

    // Cleanup
    let _ = approval_rules::Entity::delete_by_id(rule_a_id)
        .exec(&db)
        .await;
    let _ = approval_rules::Entity::delete_by_id(rule_b_id)
        .exec(&db)
        .await;
}

#[tokio::test]
async fn test_null_approval_limit_security() {
    use zeltra_core::workflow::ApprovalEngine;

    // Approver with NULL limit
    let res = ApprovalEngine::can_approve("approver", None, "approver", dec!(0.01));
    assert!(
        res.is_err(),
        "Approver with NULL limit should not be able to approve even 0.01"
    );

    // Approver with 100 limit
    let res = ApprovalEngine::can_approve("approver", Some(dec!(100)), "approver", dec!(50));
    assert!(res.is_ok(), "Approver with 100 limit should approve 50");

    // Accountant with NULL limit (Unlimited)
    let res = ApprovalEngine::can_approve("accountant", None, "approver", dec!(1000000));
    assert!(
        res.is_ok(),
        "Accountant should have unlimited even if limit is NULL"
    );
}

#[tokio::test]
async fn test_recursive_void_protection() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");
    let repo = WorkflowRepository::new(db.clone());
    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    // Create a reversal transaction
    let tx_id = Uuid::new_v4();
    let txn = transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(org_id),
        fiscal_period_id: Set(Uuid::parse_str("a46ede63-994d-4c5d-9c67-3af65116a05c").unwrap()),
        transaction_type: Set(TransactionType::Reversal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Reversal to void".to_string()),
        status: Set(TransactionStatus::Posted),
        created_by: Set(user_id),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    transactions::Entity::insert(txn).exec(&db).await.unwrap();

    let res = repo
        .void_transaction(
            org_id,
            tx_id,
            user_id,
            "Attempt to void a reversal".to_string(),
        )
        .await;

    assert!(res.is_err(), "Voiding a reversal should be forbidden");
    if let Err(e) = res {
        assert!(
            e.to_string().contains("Cannot void a reversal transaction"),
            "Error message should mention reversal protection"
        );
    }

    // Cleanup
    let _ = transactions::Entity::delete_by_id(tx_id).exec(&db).await;
}

#[tokio::test]
async fn test_bulk_approval_partial_success() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = WorkflowRepository::new(db.clone());

    let (org_id, user_id, fiscal_period_id) = setup_bulk_test_data(&db).await;

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4(); // This one will be already "Posted" to cause failure in approve_transaction

    create_test_transaction(
        &db,
        id1,
        org_id,
        fiscal_period_id,
        user_id,
        "Txn 1",
        TransactionStatus::Pending,
    )
    .await;
    create_test_transaction(
        &db,
        id2,
        org_id,
        fiscal_period_id,
        user_id,
        "Txn 2",
        TransactionStatus::Pending,
    )
    .await;
    create_test_transaction(
        &db,
        id3,
        org_id,
        fiscal_period_id,
        user_id,
        "Txn 3 (Posted)",
        TransactionStatus::Posted,
    )
    .await;

    let res = repo
        .bulk_approve(
            org_id,
            vec![id1, id2, id3],
            user_id,
            Some("Bulk test".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(res.success_count, 2, "Should approve 2 valid transactions");
    assert_eq!(res.failure_count, 1, "Should fail 1 invalid transaction");

    // Verify txn1 and txn2 are Approved
    let t1 = transactions::Entity::find_by_id(id1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let t2 = transactions::Entity::find_by_id(id2)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(t1.status, TransactionStatus::Approved);
    assert_eq!(t2.status, TransactionStatus::Approved);

    // Verify txn3 is still Posted
    let t3 = transactions::Entity::find_by_id(id3)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(t3.status, TransactionStatus::Posted);

    // Cleanup
    let _ = transactions::Entity::delete_by_id(id1).exec(&db).await;
    let _ = transactions::Entity::delete_by_id(id2).exec(&db).await;
    let _ = transactions::Entity::delete_by_id(id3).exec(&db).await;
}
async fn setup_bulk_test_data(db: &DatabaseConnection) -> (Uuid, Uuid, Uuid) {
    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let fiscal_period_id = Uuid::new_v4();

    let org = zeltra_db::entities::organizations::ActiveModel {
        id: Set(org_id),
        name: Set("Test Org".to_string()),
        slug: Set(format!("test-org-{org_id}")),
        base_currency: Set("USD".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    zeltra_db::entities::organizations::Entity::insert(org)
        .exec(db)
        .await
        .unwrap();

    let fiscal_year_id = Uuid::new_v4();
    let fiscal_year = zeltra_db::entities::fiscal_years::ActiveModel {
        id: Set(fiscal_year_id),
        organization_id: Set(org_id),
        name: Set("2026".to_string()),
        status: Set(zeltra_db::entities::sea_orm_active_enums::FiscalYearStatus::Open),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    zeltra_db::entities::fiscal_years::Entity::insert(fiscal_year)
        .exec(db)
        .await
        .unwrap();

    let fiscal_period = zeltra_db::entities::fiscal_periods::ActiveModel {
        id: Set(fiscal_period_id),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fiscal_year_id),
        name: Set("Jan 2026".to_string()),
        period_number: Set(1),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
        status: Set(zeltra_db::entities::sea_orm_active_enums::FiscalPeriodStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    zeltra_db::entities::fiscal_periods::Entity::insert(fiscal_period)
        .exec(db)
        .await
        .unwrap();

    let user = zeltra_db::entities::users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("user-{user_id}@example.com")),
        password_hash: Set("hashed_pass".to_string()),
        full_name: Set("Test User".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    zeltra_db::entities::users::Entity::insert(user)
        .exec(db)
        .await
        .unwrap();

    let org_user = organization_users::ActiveModel {
        organization_id: Set(org_id),
        user_id: Set(user_id),
        role: Set(zeltra_db::entities::sea_orm_active_enums::UserRole::Approver),
        approval_limit: Set(Some(rust_decimal::Decimal::new(1_000_000, 0))),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };
    organization_users::Entity::insert(org_user)
        .exec(db)
        .await
        .unwrap();

    (org_id, user_id, fiscal_period_id)
}

async fn create_test_transaction(
    db: &DatabaseConnection,
    id: Uuid,
    org_id: Uuid,
    fiscal_period_id: Uuid,
    user_id: Uuid,
    description: &str,
    status: TransactionStatus,
) {
    let txn = transactions::ActiveModel {
        id: Set(id),
        organization_id: Set(org_id),
        fiscal_period_id: Set(fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set(description.to_string()),
        status: Set(status),
        created_by: Set(user_id),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    transactions::Entity::insert(txn).exec(db).await.unwrap();
}
