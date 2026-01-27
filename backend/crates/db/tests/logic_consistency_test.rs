#![allow(missing_docs)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use sea_orm::{ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::env;
use uuid::Uuid;

use serial_test::serial;
use zeltra_db::{
    entities::{
        budget_line_dimensions, budget_lines, sea_orm_active_enums::TransactionType, transactions,
    },
    repositories::transaction::{
        CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository,
    },
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

#[tokio::test]
#[serial]
async fn test_budget_dimension_validation() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");
    let repo = TransactionRepository::new(db.clone());

    // Persistent verified UUIDs from the database
    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let fiscal_period_id = Uuid::parse_str("a46ede63-994d-4c5d-9c67-3af65116a05c").unwrap();
    let account_id = Uuid::parse_str("8d0d3b59-6886-4892-b6ab-4781f171e40d").unwrap(); // Office Supplies
    let dimension_value_id = Uuid::parse_str("1823eb97-e83c-4997-bd42-5f65a1212d7c").unwrap(); // Dept: Sales
    let budget_id = Uuid::parse_str("e1628d01-9252-475a-9388-37206b039fd8").unwrap();
    let bank_account_id = Uuid::parse_str("6525770f-8bff-44ce-b88a-938778c109f3").unwrap(); // Cash

    // 1. Setup a budget line for this account/period
    let line_id = Uuid::new_v4();

    // Cleanup previous attempts to ensure clean state
    let _ = budget_line_dimensions::Entity::delete_many()
        .filter(budget_line_dimensions::Column::DimensionValueId.eq(dimension_value_id))
        .exec(&db)
        .await;
    let _ = budget_lines::Entity::delete_many()
        .filter(budget_lines::Column::AccountId.eq(account_id))
        .filter(budget_lines::Column::FiscalPeriodId.eq(fiscal_period_id))
        .exec(&db)
        .await;

    let bl = budget_lines::ActiveModel {
        id: Set(line_id),
        budget_id: Set(budget_id),
        account_id: Set(account_id),
        fiscal_period_id: Set(fiscal_period_id),
        amount: Set(dec!(1000)),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    budget_lines::Entity::insert(bl).exec(&db).await.unwrap();

    let bld = budget_line_dimensions::ActiveModel {
        id: Set(Uuid::new_v4()),
        budget_line_id: Set(line_id),
        dimension_value_id: Set(dimension_value_id),
        created_at: Set(Utc::now().into()),
    };
    budget_line_dimensions::Entity::insert(bld)
        .exec(&db)
        .await
        .unwrap();

    // 2. Attempt to create a transaction to this account WITHOUT the dimension
    let input = CreateTransactionInput {
        organization_id: org_id,
        entity_id: Uuid::new_v4(),
        transaction_type: TransactionType::Expense,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        description: "Test without dimension".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(100),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(100),
                debit: dec!(100),
                credit: dec!(0),
                memo: None,
                dimensions: vec![], // Missing required dimension!
                compliance_metadata: None,
            },
            CreateLedgerEntryInput {
                account_id: bank_account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(100),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(100),
                debit: dec!(0),
                credit: dec!(100),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
        ],
        created_by: user_id,
        timezone: "UTC".to_string(),
        idempotency_key: None,
        iso_metadata: None,
    };

    let res = repo.create_transaction(input).await;
    assert!(
        res.is_err(),
        "Transaction should fail due to missing budget dimension"
    );
    if let Err(e) = res {
        assert!(
            e.to_string().contains("Budget constraint violation"),
            "Error should be BudgetConstraintViolation, got: {}",
            e
        );
    }

    // 3. Attempt to create a transaction WITH the dimension
    let input_with_dim = CreateTransactionInput {
        organization_id: org_id,
        entity_id: Uuid::new_v4(),
        transaction_type: TransactionType::Expense,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        description: "Test with dimension".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(200),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(200),
                debit: dec!(200),
                credit: dec!(0),
                memo: None,
                dimensions: vec![dimension_value_id], // Dimension provided!
                compliance_metadata: None,
            },
            CreateLedgerEntryInput {
                account_id: bank_account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(200),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(200),
                debit: dec!(0),
                credit: dec!(200),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
        ],
        created_by: user_id,
        timezone: "UTC".to_string(),
        idempotency_key: None,
        iso_metadata: None,
    };

    let res_ok = repo.create_transaction(input_with_dim).await;
    assert!(
        res_ok.is_ok(),
        "Transaction should succeed with required dimension, got error: {:?}",
        res_ok.err()
    );

    // Cleanup
    let _ = budget_lines::Entity::delete_by_id(line_id).exec(&db).await;
    if let Ok(tx_res) = res_ok {
        let _ = transactions::Entity::delete_by_id(tx_res.transaction.id)
            .exec(&db)
            .await;
    }
}

#[tokio::test]
#[serial]
async fn test_sequential_account_versions() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");
    let repo = TransactionRepository::new(db.clone());

    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let account_id = Uuid::parse_str("6525770f-8bff-44ce-b88a-938778c109f3").unwrap(); // Cash
    let other_account_id = Uuid::parse_str("8d0d3b59-6886-4892-b6ab-4781f171e40d").unwrap(); // Office Supplies (has budget constraint)
    let dimension_value_id = Uuid::parse_str("1823eb97-e83c-4997-bd42-5f65a1212d7c").unwrap();

    // 1. Get current version
    let (v_initial, _, _) = repo
        .get_latest_account_state::<DatabaseConnection>(&db, account_id)
        .await
        .unwrap();

    // 2. Create a transaction (v_initial + 1)
    let input1 = CreateTransactionInput {
        organization_id: org_id,
        entity_id: Uuid::new_v4(),
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        description: "Seq test 1".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(10),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(10),
                debit: dec!(10),
                credit: dec!(0),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
            CreateLedgerEntryInput {
                account_id: other_account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(10),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(10),
                debit: dec!(0),
                credit: dec!(10),
                memo: None,
                compliance_metadata: None,
                dimensions: vec![dimension_value_id], // Added to satisfy budget constraint
            },
        ],
        created_by: user_id,
        timezone: "UTC".to_string(),
        idempotency_key: None,
        iso_metadata: None,
    };
    let res1 = repo.create_transaction(input1).await.unwrap();
    let entry1 = res1
        .entries
        .iter()
        .find(|e| e.entry.account_id == account_id)
        .unwrap();
    assert_eq!(entry1.entry.account_version, v_initial + 1);

    // 3. Attempt a failing transaction (would be v_initial + 2)
    let input_fail = CreateTransactionInput {
        organization_id: org_id,
        entity_id: Uuid::new_v4(),
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        description: "Seq test fail".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(10),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(10),
                debit: dec!(10),
                credit: dec!(0),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
            // UNBALANCED!
        ],
        created_by: user_id,
        timezone: "UTC".to_string(),
        idempotency_key: None,
        iso_metadata: None,
    };
    let _ = repo.create_transaction(input_fail).await;

    // 4. Create another successful transaction (MUST be v_initial + 2, no gap!)
    let input2 = CreateTransactionInput {
        organization_id: org_id,
        entity_id: Uuid::new_v4(),
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        description: "Seq test 2".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(20),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(20),
                debit: dec!(20),
                credit: dec!(0),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
            CreateLedgerEntryInput {
                account_id: other_account_id,
                source_currency: "USD".to_string(),
                source_amount: dec!(20),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(20),
                debit: dec!(0),
                credit: dec!(20),
                memo: None,
                compliance_metadata: None,
                dimensions: vec![dimension_value_id], // Added to satisfy budget constraint
            },
        ],
        created_by: user_id,
        timezone: "UTC".to_string(),
        idempotency_key: None,
        iso_metadata: None,
    };
    let res2 = repo.create_transaction(input2).await.unwrap();
    let entry2 = res2
        .entries
        .iter()
        .find(|e| e.entry.account_id == account_id)
        .unwrap();
    assert_eq!(
        entry2.entry.account_version,
        v_initial + 2,
        "Should NOT have a gap even after a failed transaction attempted to take a version"
    );

    // Cleanup
    let _ = transactions::Entity::delete_by_id(res1.transaction.id)
        .exec(&db)
        .await;
    let _ = transactions::Entity::delete_by_id(res2.transaction.id)
        .exec(&db)
        .await;
}
