//! Idempotency tests.

use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use uuid::Uuid;
use zeltra_db::entities::{
    chart_of_accounts, fiscal_periods, fiscal_years, organizations,
    sea_orm_active_enums::{AccountType, FiscalPeriodStatus, FiscalYearStatus, TransactionType},
    users,
};
use zeltra_db::repositories::{
    organization::OrganizationRepository,
    transaction::{CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository},
};

// ============================================================================
// Helper Functions (Inlined)
// ============================================================================

fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

async fn setup_db() -> DatabaseConnection {
    Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database")
}

async fn create_user(db: &DatabaseConnection) -> users::Model {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("idem-test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Idempotency Test User".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    user.insert(db).await.expect("Failed to create test user")
}

async fn create_org(db: &DatabaseConnection, owner_id: Uuid) -> organizations::Model {
    let repo = OrganizationRepository::new(db.clone());
    repo.create_with_owner(
        "Idempotency Test Org",
        &format!("idem-org-{}", Uuid::new_v4()),
        "USD",
        "UTC",
        owner_id,
    )
    .await
    .expect("Failed to create organization")
}

async fn create_account(
    db: &DatabaseConnection,
    org_id: Uuid,
    name: &str,
    account_type: AccountType,
) -> chart_of_accounts::Model {
    let account = chart_of_accounts::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        code: Set(format!(
            "IDEM-{}",
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        )),
        name: Set(name.to_string()),
        account_type: Set(account_type),
        currency: Set("USD".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    account.insert(db).await.expect("Failed to create account")
}

async fn create_fiscal_data(db: &DatabaseConnection, org_id: Uuid) -> fiscal_periods::Model {
    let fy = fiscal_years::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        name: Set("FY 2026 Idem".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        status: Set(FiscalYearStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    let fy_model = fy.insert(db).await.expect("Failed to create fiscal year");

    let period = fiscal_periods::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fy_model.id),
        name: Set("Jan 2026".to_string()),
        period_number: Set(1),
        is_adjustment_period: Set(false),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
        status: Set(FiscalPeriodStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    period
        .insert(db)
        .await
        .expect("Failed to create fiscal period")
}

// ============================================================================
// Test Case
// ============================================================================

#[tokio::test]
async fn test_create_transaction_idempotency() {
    let db = setup_db().await;
    let tx_repo = TransactionRepository::new(db.clone());

    let user = create_user(&db).await;
    let org = create_org(&db, user.id).await;
    let _period = create_fiscal_data(&db, org.id).await;
    let acct1 = create_account(&db, org.id, "Cash", AccountType::Asset).await;
    let acct2 = create_account(&db, org.id, "Revenue", AccountType::Revenue).await;

    let idempotency_key = Uuid::new_v4();
    let tx_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    let input = CreateTransactionInput {
        organization_id: org.id,
        entity_id: Uuid::new_v4(),
        transaction_type: TransactionType::Journal,
        transaction_date: tx_date,
        description: "Idempotent Tx".to_string(),
        reference_number: Some("REF-123".to_string()),
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: acct1.id,
                source_currency: "USD".to_string(),
                source_amount: dec!(100),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(100),
                debit: dec!(100),
                credit: dec!(0),
                memo: None,
                compliance_metadata: None,
                dimensions: vec![],
            },
            CreateLedgerEntryInput {
                account_id: acct2.id,
                source_currency: "USD".to_string(),
                source_amount: dec!(100),
                exchange_rate: dec!(1),
                functional_currency: "USD".to_string(),
                functional_amount: dec!(100),
                debit: dec!(0),
                credit: dec!(100),
                memo: None,
                compliance_metadata: None,
                dimensions: vec![],
            },
        ],
        created_by: user.id,
        timezone: "UTC".to_string(),
        idempotency_key: Some(idempotency_key),
        iso_metadata: None,
    };

    // 1. First Call: Should create new transaction
    let result1 = tx_repo
        .create_transaction(input.clone())
        .await
        .expect("First call failed");
    println!("Result 1 ID: {}", result1.transaction.id);

    // 2. Second Call: Should return EXISTING transaction (Same ID)
    let result2 = tx_repo
        .create_transaction(input)
        .await
        .expect("Second call failed");
    println!("Result 2 ID: {}", result2.transaction.id);

    assert_eq!(
        result1.transaction.id, result2.transaction.id,
        "Transaction IDs should be identical for idempotent request"
    );
    assert_eq!(
        result1.transaction.created_at, result2.transaction.created_at,
        "Timestamps should match"
    );
}
