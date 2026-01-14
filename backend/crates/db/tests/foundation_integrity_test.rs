//! Foundation integrity tests for Zeltra 2026.
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{Database, DatabaseConnection, Set, ActiveModelTrait};
use uuid::Uuid;
use zeltra_db::entities::sea_orm_active_enums::{TransactionType};
use zeltra_db::repositories::transaction::{
    CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository,
};

// Helper to get DB URL
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

async fn setup_test_data(db: &DatabaseConnection) -> Result<TestData, Box<dyn std::error::Error>> {
    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Create User
    use zeltra_db::entities::users;
    users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("found-{}@example.com", user_id)),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Foundation User".to_string()),
        is_active: Set(true),
        ..Default::default()
    }.insert(db).await?;

    // Create Org
    use zeltra_db::entities::organizations;
    organizations::ActiveModel {
        id: Set(org_id),
        name: Set("Foundation Org".to_string()),
        slug: Set(format!("found-{}", org_id)),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }.insert(db).await?;

    // Create Fiscal Year
    use zeltra_db::entities::fiscal_years;
    let fiscal_year_id = Uuid::new_v4();
    fiscal_years::ActiveModel {
        id: Set(fiscal_year_id),
        organization_id: Set(org_id),
        name: Set("FY 2026".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }.insert(db).await?;

    // Create Fiscal Period
    use zeltra_db::entities::fiscal_periods;
    let fiscal_period_id = Uuid::new_v4();
    fiscal_periods::ActiveModel {
        id: Set(fiscal_period_id),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fiscal_year_id),
        period_number: Set(1),
        name: Set("2026-01".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        status: Set(zeltra_db::entities::sea_orm_active_enums::FiscalPeriodStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }.insert(db).await?;

    // Create Accounts
    use zeltra_db::entities::chart_of_accounts;
    use zeltra_db::entities::sea_orm_active_enums::AccountType;
    let asset_id = Uuid::new_v4();
    chart_of_accounts::ActiveModel {
        id: Set(asset_id),
        organization_id: Set(org_id),
        code: Set("1000-FOUND".to_string()),
        name: Set("Secure Asset".to_string()),
        account_type: Set(AccountType::Asset),
        currency: Set("USD".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }.insert(db).await?;

    let expense_id = Uuid::new_v4();
    chart_of_accounts::ActiveModel {
        id: Set(expense_id),
        organization_id: Set(org_id),
        code: Set("5000-FOUND".to_string()),
        name: Set("Secure Expense".to_string()),
        account_type: Set(AccountType::Expense),
        currency: Set("USD".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }.insert(db).await?;

    Ok(TestData {
        org_id,
        user_id,
        fiscal_period_id,
        asset_id,
        expense_id,
    })
}

struct TestData {
    org_id: Uuid,
    user_id: Uuid,
    _fiscal_period_id: Uuid,
    asset_id: Uuid,
    expense_id: Uuid,
}

#[tokio::test]
async fn test_foundation_idempotency() {
    let db = Database::connect(get_database_url()).await.unwrap();
    let data = setup_test_data(&db).await.unwrap();
    let repo = TransactionRepository::new(db.clone());

    let idempotency_key = Uuid::new_v4();
    let input = CreateTransactionInput {
        organization_id: data.org_id,
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
        description: "Idempotency Test".to_string(),
        reference_number: None,
        memo: None,
        created_by: data.user_id,
        timezone: "UTC".to_string(),
        idempotency_key: Some(idempotency_key),
        iso_metadata: Some(serde_json::json!({"purpose": "TEST"})),
        entries: vec![
            CreateLedgerEntryInput {
                account_id: data.expense_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            CreateLedgerEntryInput {
                account_id: data.asset_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: None,
                dimensions: vec![],
            },
        ],
    };

    // 1. First call
    let tx1 = repo.create_transaction(input.clone()).await.unwrap();
    
    // 2. Second call with same key
    let tx2 = repo.create_transaction(input).await.unwrap();

    assert_eq!(tx1.transaction.id, tx2.transaction.id, "Mata Dewa Result: Idempotency FAILED. Created two different transactions for same key.");
    println!("Idempotency OK: Returned same transaction ID {}", tx1.transaction.id);
}

#[tokio::test]
async fn test_foundation_hash_chaining() {
    let db = Database::connect(get_database_url()).await.unwrap();
    let data = setup_test_data(&db).await.unwrap();
    let repo = TransactionRepository::new(db.clone());

    // Transaction 1
    let input1 = CreateTransactionInput {
        organization_id: data.org_id,
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
        description: "Tx 1".to_string(),
        reference_number: None,
        memo: None,
        created_by: data.user_id,
        timezone: "UTC".to_string(),
        idempotency_key: Some(Uuid::new_v4()),
        iso_metadata: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: data.expense_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            CreateLedgerEntryInput {
                account_id: data.asset_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: None,
                dimensions: vec![],
            },
        ],
    };

    let tx1 = repo.create_transaction(input1).await.unwrap();
    let entry1 = &tx1.entries[0].entry;
    
    assert!(entry1.entry_hash.is_some(), "Entry 1 hash should exist");
    assert!(entry1.previous_entry_hash.is_none(), "Entry 1 previous hash should be None");

    // Transaction 2 (same account)
    let input2 = CreateTransactionInput {
        organization_id: data.org_id,
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
        description: "Tx 2".to_string(),
        reference_number: None,
        memo: None,
        created_by: data.user_id,
        timezone: "UTC".to_string(),
        idempotency_key: Some(Uuid::new_v4()),
        iso_metadata: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: data.expense_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(50, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(50, 0),
                debit: Decimal::new(50, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            CreateLedgerEntryInput {
                account_id: data.asset_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(50, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(50, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(50, 0),
                memo: None,
                dimensions: vec![],
            },
        ],
    };

    let tx2 = repo.create_transaction(input2).await.unwrap();
    let entry2_expense = tx2.entries.iter().find(|e| e.entry.account_id == data.expense_id).unwrap();
    
    assert_eq!(entry2_expense.entry.previous_entry_hash, entry1.entry_hash, "Mata Dewa Result: Hash Chaining FAILED. Entry 2 didn't point to Entry 1.");
    println!("Hash Chaining OK: Entry {} points to Entry {}", tx2.transaction.id, tx1.transaction.id);
}

#[tokio::test]
async fn test_foundation_tamper_detection() {
    let db = Database::connect(get_database_url()).await.unwrap();
    let data = setup_test_data(&db).await.unwrap();
    let repo = TransactionRepository::new(db.clone());

    // 1. Create a valid transaction
    let input = CreateTransactionInput {
        organization_id: data.org_id,
        transaction_type: TransactionType::Journal,
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
        description: "Tamper Test".to_string(),
        reference_number: None,
        memo: None,
        created_by: data.user_id,
        timezone: "UTC".to_string(),
        idempotency_key: Some(Uuid::new_v4()),
        iso_metadata: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: data.expense_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                memo: None,
                dimensions: vec![],
            },
            CreateLedgerEntryInput {
                account_id: data.asset_id,
                source_currency: "USD".to_string(),
                source_amount: Decimal::new(100, 0),
                exchange_rate: Decimal::ONE,
                functional_currency: "USD".to_string(),
                functional_amount: Decimal::new(100, 0),
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                memo: None,
                dimensions: vec![],
            },
        ],
    };

    let tx = repo.create_transaction(input).await.unwrap();
    let entry_id = tx.entries[0].entry.id;

    // 2. Initial verification should pass
    let initial_corrupted = repo.verify_ledger_integrity(data.org_id).await.unwrap();
    assert!(initial_corrupted.is_empty(), "Ledger should be healthy initially");

    // 3. TAMPER! (Simulate out-of-band DB update)
    // We update the debit amount without updating the hash.
    // NOTE: We must disable the trigger first because our DB is ALREADY SECURE!
    use sea_orm::{ConnectionTrait, Statement};
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE ledger_entries DISABLE TRIGGER trg_prevent_ledger_update",
    ))
    .await
    .unwrap();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("UPDATE ledger_entries SET debit = 999.99, functional_amount = 999.99 WHERE id = '{}'", entry_id),
    ))
    .await
    .unwrap();

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "ALTER TABLE ledger_entries ENABLE TRIGGER trg_prevent_ledger_update",
    ))
    .await
    .unwrap();

    // 4. Verification should now FAIL
    let corrupted = repo.verify_ledger_integrity(data.org_id).await.unwrap();
    assert!(!corrupted.is_empty(), "Mata Dewa Result: Tamper Detection FAILED. Ledger verified as healthy despite modified amount.");
    assert_eq!(corrupted[0], entry_id, "Corrupted entry ID should match the tampered one");
    
    println!("Tamper Detection OK: Successfully detected corruption in entry {}", entry_id);
}
