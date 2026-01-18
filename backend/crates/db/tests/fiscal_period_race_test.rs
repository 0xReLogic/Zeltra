//! Fiscal period race tests.

use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;
use zeltra_db::entities::{
    chart_of_accounts, fiscal_periods, fiscal_years, organizations,
    sea_orm_active_enums::{
        AccountType, FiscalPeriodStatus, FiscalYearStatus, TransactionStatus, TransactionType,
    },
    users,
};
use zeltra_db::repositories::{
    organization::OrganizationRepository,
    transaction::{CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository},
    workflow::WorkflowRepository,
};

// ============================================================================
// Helper Functions
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
        email: Set(format!("test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Test User".to_string()),
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
        "Test Org",
        &format!("test-org-{}", Uuid::new_v4()),
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
            "CODE-{}",
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

async fn create_fiscal_year(
    db: &DatabaseConnection,
    org_id: Uuid,
    year: i32,
) -> fiscal_years::Model {
    let fy = fiscal_years::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        name: Set(format!("FY {year}")),
        start_date: Set(NaiveDate::from_ymd_opt(year, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(year, 12, 31).unwrap()),
        status: Set(FiscalYearStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    fy.insert(db).await.expect("Failed to create fiscal year")
}

// ============================================================================
// Test Case
// ============================================================================

#[tokio::test]
async fn test_post_transaction_to_closed_period_race() {
    let db = setup_db().await;

    let workflow_repo = WorkflowRepository::new(db.clone());

    let (org, user, tx_final, period_id) = setup_race_scenario(&db).await;
    assert_eq!(tx_final.status, TransactionStatus::Approved);

    // 5. ATTACK SIMULATION: Admin CLOSES the period
    let mut period_active: fiscal_periods::ActiveModel =
        fiscal_periods::Entity::find_by_id(period_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .into();
    period_active.status = Set(FiscalPeriodStatus::Closed);
    period_active.update(&db).await.unwrap();

    // 6. User attempts to POST (Race Condition)
    // This SHOULD FAIL because the period is now closed.
    let result = workflow_repo
        .post_transaction(org.id, tx_final.id, user.id)
        .await;

    match result {
        Ok(_) => panic!(
            "CRITICAL: Transaction posted to CLOSED fiscal period! Security invariant violated."
        ),
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Cannot post to closed fiscal period"),
                "Expected 'Cannot post to closed fiscal period' error, got: {error_msg}"
            );
        }
    }
}

async fn setup_race_scenario(
    db: &DatabaseConnection,
) -> (
    organizations::Model,
    users::Model,
    zeltra_db::entities::transactions::Model,
    Uuid,
) {
    let tx_repo = TransactionRepository::new(db.clone());
    let workflow_repo = WorkflowRepository::new(db.clone());

    // 1. Setup Data
    let user = create_user(db).await;
    let org = create_org(db, user.id).await;
    let asset_acct = create_account(db, org.id, "Cash", AccountType::Asset).await;
    let expense_acct = create_account(db, org.id, "Expense", AccountType::Expense).await;

    // 2. Create FY and OPEN Fiscal Period (Jan 2026)
    let fy = create_fiscal_year(db, org.id, 2026).await;
    let period_id = Uuid::new_v4();
    let period = fiscal_periods::ActiveModel {
        id: Set(period_id),
        organization_id: Set(org.id),
        fiscal_year_id: Set(fy.id),
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
        .expect("Failed to create fiscal period");

    // 3. Create Draft Transaction
    let tx_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let input = CreateTransactionInput {
        organization_id: org.id,
        transaction_type: TransactionType::Journal,
        transaction_date: tx_date,
        description: "Test Race".to_string(),
        reference_number: None,
        memo: None,
        entries: vec![
            CreateLedgerEntryInput {
                account_id: asset_acct.id,
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
                account_id: expense_acct.id,
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
        idempotency_key: None,
        iso_metadata: None,
    };

    let tx_draft = tx_repo
        .create_transaction(input)
        .await
        .expect("Failed to create draft")
        .transaction;
    assert_eq!(tx_draft.status, TransactionStatus::Draft);

    // 4. Move to Approved
    let _ = workflow_repo
        .submit_transaction(org.id, tx_draft.id, user.id)
        .await
        .expect("Failed to submit");
    let tx_final = workflow_repo
        .approve_transaction(org.id, tx_draft.id, user.id, None)
        .await
        .expect("Failed to approve");

    (org, user, tx_final, period_id)
}
