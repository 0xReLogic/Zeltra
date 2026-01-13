#![allow(missing_docs)]
use chrono::NaiveDate;
use rust_decimal_macros::dec;
use sea_orm::{Database, EntityTrait, Set};
use std::env;
use uuid::Uuid;

use zeltra_db::{
    entities::{
        approval_rules,
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
    use zeltra_core::workflow::{ApprovalEngine, ApprovalRule};
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
async fn test_bulk_approval_atomicity() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");
    let repo = WorkflowRepository::new(db.clone());
    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let fiscal_period_id = Uuid::parse_str("a46ede63-994d-4c5d-9c67-3af65116a05c").unwrap();

    // Create 3 pending transactions
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4(); // This one will be already "Posted" to cause failure in approve_transaction

    let txn1 = transactions::ActiveModel {
        id: Set(id1),
        organization_id: Set(org_id),
        fiscal_period_id: Set(fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Txn 1".to_string()),
        status: Set(TransactionStatus::Pending),
        created_by: Set(user_id),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    let txn2 = transactions::ActiveModel {
        id: Set(id2),
        organization_id: Set(org_id),
        fiscal_period_id: Set(fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Txn 2".to_string()),
        status: Set(TransactionStatus::Pending),
        created_by: Set(user_id),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    let txn3 = transactions::ActiveModel {
        id: Set(id3),
        organization_id: Set(org_id),
        fiscal_period_id: Set(fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Txn 3 (Posted)".to_string()),
        status: Set(TransactionStatus::Posted), // WRONG STATUS for approval
        created_by: Set(user_id),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };

    transactions::Entity::insert(txn1).exec(&db).await.unwrap();
    transactions::Entity::insert(txn2).exec(&db).await.unwrap();
    transactions::Entity::insert(txn3).exec(&db).await.unwrap();

    let res = repo
        .bulk_approve(
            org_id,
            vec![id1, id2, id3],
            user_id,
            Some("Bulk test".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(
        res.success_count, 0,
        "No transactions should be approved because one failed"
    );
    assert_eq!(res.failure_count, 1);

    // Verify txn1 and txn2 are still Pending (weren't committed)
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

    assert_eq!(
        t1.status,
        TransactionStatus::Pending,
        "Txn 1 should remain Pending after rollback"
    );
    assert_eq!(
        t2.status,
        TransactionStatus::Pending,
        "Txn 2 should remain Pending after rollback"
    );

    // Cleanup
    let _ = transactions::Entity::delete_by_id(id1).exec(&db).await;
    let _ = transactions::Entity::delete_by_id(id2).exec(&db).await;
    let _ = transactions::Entity::delete_by_id(id3).exec(&db).await;
}
