#![allow(missing_docs)]
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use std::env;
use uuid::Uuid;

use zeltra_db::{
    entities::{
        fiscal_periods, fiscal_years, ledger_entries,
        sea_orm_active_enums::{FiscalPeriodStatus, TransactionStatus, TransactionType},
        transactions,
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

/// Helper to create a fiscal year and period for testing
async fn setup_fiscal_period(
    db: &DatabaseConnection,
    org_id: Uuid,
    entity_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Uuid {
    use zeltra_db::entities::{
        entities, organizations,
        sea_orm_active_enums::{FiscalYearStatus, SubscriptionStatus, SubscriptionTier},
    };

    // Ensure organization exists
    let org_exists = organizations::Entity::find_by_id(org_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some();

    if !org_exists {
        let org = organizations::ActiveModel {
            id: Set(org_id),
            name: Set("Test Organization".to_string()),
            slug: Set(format!("test-org-{}", org_id)),
            base_currency: Set("USD".to_string()),
            timezone: Set("UTC".to_string()),
            settings: Set(serde_json::json!({})),
            subscription_tier: Set(SubscriptionTier::Enterprise),
            subscription_status: Set(SubscriptionStatus::Active),
            is_active: Set(true),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        organizations::Entity::insert(org)
            .exec(db)
            .await
            .expect("Failed to create test organization");
    }

    // Ensure entity exists
    let entity_exists = entities::Entity::find_by_id(entity_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some();

    if !entity_exists {
        let entity = entities::ActiveModel {
            id: Set(entity_id),
            organization_id: Set(org_id),
            name: Set("Test Entity".to_string()),
            legal_name: Set(Some("Test Entity Legal Name".to_string())),
            tax_id: Set(None),
            entity_type: Set("main".to_string()),
            base_currency: Set("USD".to_string()),
            is_active: Set(true),
            settings: Set(serde_json::json!({})),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };
        entities::Entity::insert(entity)
            .exec(db)
            .await
            .expect("Failed to create test entity");
    }

    let fiscal_year_id = Uuid::new_v4();
    let fiscal_period_id = Uuid::new_v4();

    // Create fiscal year
    let fiscal_year = fiscal_years::ActiveModel {
        id: Set(fiscal_year_id),
        organization_id: Set(org_id),
        entity_id: Set(entity_id),
        name: Set("Test FY 2026".to_string()),
        start_date: Set(start_date),
        end_date: Set(end_date),
        status: Set(FiscalYearStatus::Open),
        closed_by: Set(None),
        closed_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    fiscal_years::Entity::insert(fiscal_year)
        .exec(db)
        .await
        .expect("Failed to create fiscal year");

    // Create fiscal period
    let fiscal_period = fiscal_periods::ActiveModel {
        id: Set(fiscal_period_id),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fiscal_year_id),
        name: Set("Test Period".to_string()),
        period_number: Set(1),
        start_date: Set(start_date),
        end_date: Set(end_date),
        status: Set(FiscalPeriodStatus::Open),
        is_adjustment_period: Set(false),
        closed_by: Set(None),
        closed_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    fiscal_periods::Entity::insert(fiscal_period)
        .exec(db)
        .await
        .expect("Failed to create fiscal period");

    fiscal_period_id
}

#[tokio::test]
async fn test_unique_account_version_constraint() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let account_id = Uuid::parse_str("6525770f-8bff-44ce-b88a-938778c109f3").unwrap();
    let transaction_id = Uuid::new_v4();
    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let fiscal_period_id = Uuid::parse_str("a46ede63-994d-4c5d-9c67-3af65116a05c").unwrap();
    let entity_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    // Ensure organization and entity exist
    use zeltra_db::entities::{
        entities, organizations,
        sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier},
    };

    let org_exists = organizations::Entity::find_by_id(org_id)
        .one(&db)
        .await
        .ok()
        .flatten()
        .is_some();

    if !org_exists {
        let org = organizations::ActiveModel {
            id: Set(org_id),
            name: Set("Test Organization".to_string()),
            slug: Set(format!("test-org-{}", org_id)),
            base_currency: Set("USD".to_string()),
            timezone: Set("UTC".to_string()),
            settings: Set(serde_json::json!({})),
            subscription_tier: Set(SubscriptionTier::Enterprise),
            subscription_status: Set(SubscriptionStatus::Active),
            is_active: Set(true),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };
        organizations::Entity::insert(org)
            .exec(&db)
            .await
            .expect("Failed to create test organization");
    }

    let entity_exists = entities::Entity::find_by_id(entity_id)
        .one(&db)
        .await
        .ok()
        .flatten()
        .is_some();

    if !entity_exists {
        let entity = entities::ActiveModel {
            id: Set(entity_id),
            organization_id: Set(org_id),
            name: Set("Test Entity".to_string()),
            legal_name: Set(Some("Test Entity Legal Name".to_string())),
            tax_id: Set(None),
            entity_type: Set("main".to_string()),
            base_currency: Set("USD".to_string()),
            is_active: Set(true),
            settings: Set(serde_json::json!({})),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };
        entities::Entity::insert(entity)
            .exec(&db)
            .await
            .expect("Failed to create test entity");
    }

    // Create a dummy transaction first to satisfy FK
    let txn_model = transactions::ActiveModel {
        id: Set(transaction_id),
        organization_id: Set(org_id),
        entity_id: Set(entity_id),
        fiscal_period_id: Set(fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Unique Constraint Test".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(user_id),
        timezone: Set("UTC".to_string()),
        idempotency_key: Set(None),
        iso_metadata: Set(None),
        ..Default::default()
    };

    let entry1 = ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(transaction_id),
        account_id: Set(account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(dec!(100)),
        exchange_rate: Set(dec!(1)),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(dec!(100)),
        debit: Set(dec!(100)),
        credit: Set(dec!(0)),
        account_version: Set(777_222),
        account_previous_balance: Set(dec!(0)),
        account_current_balance: Set(dec!(100)),
        entry_hash: Set(None),
        previous_entry_hash: Set(None),
        ..Default::default()
    };

    let entry2 = ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(transaction_id),
        account_id: Set(account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(dec!(100)),
        exchange_rate: Set(dec!(1)),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(dec!(100)),
        debit: Set(dec!(100)),
        credit: Set(dec!(0)),
        account_version: Set(777_222), // Same version!
        account_previous_balance: Set(dec!(100)),
        account_current_balance: Set(dec!(200)),
        entry_hash: Set(None),
        previous_entry_hash: Set(None),
        ..Default::default()
    };

    let txn = db.begin().await.unwrap();

    // Disable trigger BEFORE any activity in the transaction to avoid "pending trigger events" error
    txn.execute_unprepared("ALTER TABLE ledger_entries DISABLE TRIGGER trg_update_account_balance")
        .await
        .unwrap();

    // Setup transaction and first entry
    transactions::Entity::insert(txn_model)
        .exec(&txn)
        .await
        .unwrap();
    ledger_entries::Entity::insert(entry1)
        .exec(&txn)
        .await
        .unwrap();

    // Try to insert duplicate
    let res = ledger_entries::Entity::insert(entry2).exec(&txn).await;

    // RE-ENABLE TRIGGER BEFORE ASSERTING
    let _ = txn
        .execute_unprepared("ALTER TABLE ledger_entries ENABLE TRIGGER trg_update_account_balance")
        .await;

    assert!(
        res.is_err(),
        "Duplicate account version should fail with UNIQUE constraint when trigger is disabled"
    );

    txn.rollback().await.unwrap();
}

#[tokio::test]
async fn test_residual_adjustment_insertion() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");
    let repo = TransactionRepository::new(db.clone());

    let org_id = Uuid::parse_str("d2b40c00-d207-4104-b8b6-b4e925abb507").unwrap();
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
    let entity_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let acc1 = Uuid::parse_str("6525770f-8bff-44ce-b88a-938778c109f3").unwrap();
    let acc2 = Uuid::parse_str("8d0d3b59-6886-4892-b6ab-4781f171e40d").unwrap();

    // Setup fiscal period for the test
    let _fiscal_period_id = setup_fiscal_period(
        &db,
        org_id,
        entity_id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;

    let input = CreateTransactionInput {
        organization_id: org_id,
        entity_id,
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        description: "Rounding Test".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: acc1,
                source_currency: "USD".to_string(),
                source_amount: dec!(100.00),
                exchange_rate: dec!(0.33333333),
                functional_currency: "EUR".to_string(),
                functional_amount: dec!(33.33),
                debit: dec!(33.33),
                credit: dec!(0),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
            CreateLedgerEntryInput {
                account_id: acc2,
                source_currency: "USD".to_string(),
                source_amount: dec!(200.00),
                exchange_rate: dec!(0.33333333),
                functional_currency: "EUR".to_string(),
                functional_amount: dec!(66.67), // 200 * 0.333... = 66.666...
                debit: dec!(0),
                credit: dec!(66.67),
                memo: None,
                dimensions: vec![],
                compliance_metadata: None,
            },
            CreateLedgerEntryInput {
                account_id: acc1,
                source_currency: "USD".to_string(),
                source_amount: dec!(100.00),
                exchange_rate: dec!(0.33333333),
                functional_currency: "EUR".to_string(),
                functional_amount: dec!(33.33),
                debit: dec!(33.33),
                credit: dec!(0),
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

    // Total functional debit: 33.33 + 33.33 = 66.66
    // Total functional credit: 66.67
    // Difference (residual): -0.01 (need to add 0.01 to debit or remove from credit)
    // Our logic adjusts the last entry (acc1, debit 33.33)
    // So Entry 3 (debit 33.33) should become debit 33.34

    let result = repo
        .create_transaction(input)
        .await
        .expect("Should balance with residual adjustment");

    // Total functional debits/credits should be equal
    let total_debit: Decimal = result.entries.iter().map(|e| e.entry.debit).sum();
    let total_credit: Decimal = result.entries.iter().map(|e| e.entry.credit).sum();
    assert_eq!(
        total_debit, total_credit,
        "Debits and Credits must be exactly equal after adjustment"
    );
    assert_eq!(total_debit, dec!(66.67));

    // Cleanup
    repo.delete_transaction(org_id, result.transaction.id)
        .await
        .unwrap();
}
