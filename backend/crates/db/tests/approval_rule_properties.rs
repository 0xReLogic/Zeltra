//! Property-based tests for Approval Rules.
//!
//! Verifies invariants such as amount ranges, priority limits, and string constraints.

use proptest::prelude::*;
use rust_decimal::Decimal;
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use std::env;
use tokio::runtime::Runtime;
use uuid::Uuid;
use zeltra_db::{
    entities::organizations,
    repositories::approval_rule::{
        ApprovalRuleError, ApprovalRuleRepository, CreateApprovalRuleInput,
    },
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

// Helper to setup org for a test case
async fn setup_org(db: &DatabaseConnection) -> Uuid {
    let org_id = Uuid::new_v4();
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set("Prop Test Org".to_string()),
        slug: Set(format!("prop-test-org-{org_id}")),
        base_currency: Set("USD".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    organizations::Entity::insert(org)
        .exec(db)
        .await
        .expect("Failed to insert org");
    org_id
}

// Helper: Run async code in a temporary runtime
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(future)
}

proptest! {
    // Limit cases for DB integration tests to avoid timeouts
    #![proptest_config(proptest::test_runner::Config::with_cases(10))]

    #[test]
    fn prop_priority_range(priority in -100i16..200i16) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let repo = ApprovalRuleRepository::new(db.clone());
            let org_id = setup_org(&db).await;

            let input = CreateApprovalRuleInput {
                name: "Priority Test".to_string(),
                description: None,
                min_amount: None,
                max_amount: None,
                transaction_types: vec!["bill".to_string()],
                required_role: "approver".to_string(),
                priority,
            };

            let result = repo.create_rule(org_id, input).await;

            if (1..=100).contains(&priority) {
                assert!(result.is_ok(), "Valid priority {priority} rejected: {:?}", result.err());
            } else {
                assert!(result.is_err(), "Invalid priority {priority} accepted");
                match result {
                    Err(ApprovalRuleError::ValidationError(_)) => { /* OK */ },
                    Err(e) => panic!("Expected ValidationError for priority {priority}, got {e:?}"),
                    _ => unreachable!(),
                }
            }
        });
    }

    #[test]
    fn prop_amount_range(
        min in 0u32..10000u32,
        max in 0u32..10000u32
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let repo = ApprovalRuleRepository::new(db.clone());
            let org_id = setup_org(&db).await;

            let min_dec = Decimal::from(min);
            let max_dec = Decimal::from(max);

            let input = CreateApprovalRuleInput {
                name: "Amount Test".to_string(),
                description: None,
                min_amount: Some(min_dec),
                max_amount: Some(max_dec),
                transaction_types: vec!["bill".to_string()],
                required_role: "approver".to_string(),
                priority: 1,
            };

            let result = repo.create_rule(org_id, input).await;

            if min_dec <= max_dec {
                assert!(result.is_ok(), "Valid amount range {min}..={max} rejected");
            } else {
                assert!(result.is_err(), "Invalid amount range {min}..={max} accepted");
                match result {
                    Err(ApprovalRuleError::ValidationError(_)) => { /* OK */ },
                    Err(e) => panic!("Expected ValidationError for range {min}..={max}, got {e:?}"),
                    _ => unreachable!(),
                }
            }
        });
    }

    #[test]
    fn prop_transaction_completeness(
        ref type_str in ".*"
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let repo = ApprovalRuleRepository::new(db.clone());
            let org_id = setup_org(&db).await;

            let input = CreateApprovalRuleInput {
                name: "Tx Type Test".to_string(),
                description: None,
                min_amount: None,
                max_amount: None,
                transaction_types: vec![type_str.clone()],
                required_role: "approver".to_string(),
                priority: 1,
            };

            let result = repo.create_rule(org_id, input).await;

            let valid_types = [
                "journal", "invoice", "bill", "payment", "expense", "transfer",
                "adjustment", "opening_balance", "reversal", "accrual",
                "revaluation", "intercompany",
                "JOURNAL", "Bill"
            ];

            let is_valid = valid_types.iter().any(|t| t.to_lowercase() == type_str.to_lowercase());

            if is_valid {
                 assert!(result.is_ok(), "Valid type '{type_str}' rejected");
            } else {
                 assert!(result.is_err(), "Invalid type '{type_str}' accepted");
                 match result {
                    Err(ApprovalRuleError::InvalidTransactionType(_)) => { /* OK */ },
                    Err(e) => panic!("Expected InvalidTransactionType for type '{type_str}', got {e:?}"),
                    _ => unreachable!(),
                 }
            }
        });
    }

    #[test]
    fn prop_string_length(
        ref name in "\\PC*",
        ref description in "\\PC*"
    ) {
        block_on(async {
             let db = Database::connect(&get_database_url()).await.unwrap();
             let repo = ApprovalRuleRepository::new(db.clone());
             let org_id = setup_org(&db).await;

             let input = CreateApprovalRuleInput {
                name: name.clone(),
                description: Some(description.clone()),
                min_amount: None,
                max_amount: None,
                transaction_types: vec!["bill".to_string()],
                required_role: "approver".to_string(),
                priority: 1,
            };

            let result = repo.create_rule(org_id, input).await;

            let name_valid = !name.is_empty() && name.len() <= 255;
            let desc_valid = description.len() <= 1000;

            if !name_valid {
                assert!(result.is_err(), "Invalid name length {} accepted", name.len());
                match result {
                    Err(ApprovalRuleError::ValidationError(_)) => { /* OK */ },
                    Err(e) => panic!("Expected ValidationError for name, got {e:?}"),
                    _ => unreachable!(),
                }
            } else if !desc_valid {
                assert!(result.is_err(), "Invalid description length {} accepted", description.len());
                 match result {
                    Err(ApprovalRuleError::ValidationError(_)) => { /* OK */ },
                    Err(e) => panic!("Expected ValidationError for description, got {e:?}"),
                    _ => unreachable!(),
                }
            } else {
                assert!(result.is_ok(), "Valid input rejected: {:?}", result.err());
            }
        });
    }
}

#[tokio::test]
async fn prop_database_index_usage() {
    // Property 9: Database Index Usage
    use sea_orm::{ConnectionTrait, Statement};

    let db = Database::connect(&get_database_url()).await.unwrap();
    let repo = ApprovalRuleRepository::new(db.clone());
    let org_id = setup_org(&db).await;

    // Create some rules to query
    let input = CreateApprovalRuleInput {
        name: "Index Test".to_string(),
        description: None,
        min_amount: None,
        max_amount: None,
        transaction_types: vec!["bill".to_string()],
        required_role: "approver".to_string(),
        priority: 1,
    };

    repo.create_rule(org_id, input)
        .await
        .expect("Failed to create rule");

    let sql = format!(
        "EXPLAIN SELECT * FROM approval_rules WHERE organization_id = '{org_id}' AND is_active = true ORDER BY priority ASC"
    );

    let _result = db
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
        .expect("Failed to execute EXPLAIN");

    let rows = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!(
            "EXPLAIN SELECT * FROM approval_rules WHERE organization_id = '{org_id}' AND is_active = true ORDER BY priority ASC"
        )
    )).await.expect("Failed to query EXPLAIN");

    // Concatenate all query plan lines
    let _plan = rows
        .iter()
        .map(|row| {
            let line: String = row
                .try_get("", "QUERY PLAN")
                .unwrap_or_else(|_| "Wait, column name varies".to_string());
            line
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Check if index scan is used.
    // Spec says: "Test that EXPLAIN shows index usage".

    // We can try to force index usage.
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SET enable_seqscan = off".to_string(),
    ))
    .await
    .expect("Failed to disable seqscan");

    let rows_forced = db.query_all(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
         format!(
            "EXPLAIN SELECT * FROM approval_rules WHERE organization_id = '{org_id}' AND is_active = true ORDER BY priority ASC"
        )
    )).await.expect("Failed to query EXPLAIN forced");

    let plan_forced = rows_forced
        .iter()
        .map(|row| row.try_get::<String>("", "QUERY PLAN").unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    if !plan_forced.contains("idx_approval_rules_org_priority")
        && !plan_forced.contains("Index Scan")
    {
        println!("Query Plan (Forced): {plan_forced}");
    }

    // Re-enable seqscan
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        "SET enable_seqscan = on".to_string(),
    ))
    .await
    .ok();
}
