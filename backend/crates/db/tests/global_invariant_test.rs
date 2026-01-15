use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, Set, Statement,
};
use uuid::Uuid;
use zeltra_db::entities::{
    chart_of_accounts, fiscal_periods, fiscal_years, organizations,
    sea_orm_active_enums::{AccountType, FiscalPeriodStatus, FiscalYearStatus, TransactionType},
    users,
};
use zeltra_db::repositories::{
    organization::OrganizationRepository,
    transaction::{CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository},
    workflow::WorkflowRepository,
};

// ============================================================================
// Helper Functions (Inlined for self-contained reproduction)
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
        email: Set(format!("global-test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Global Test User".to_string()),
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
        "Global Invariant Test Org",
        &format!("global-org-{}", Uuid::new_v4()),
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

async fn create_fiscal_data(db: &DatabaseConnection, org_id: Uuid) -> fiscal_periods::Model {
    let fy = fiscal_years::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        name: Set("FY 2026 Global".to_string()),
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
// Main Global Invariant Test
// ============================================================================

#[tokio::test]
async fn test_global_invariant_sum_debits_eq_credits() {
    let db = setup_db().await;
    let tx_repo = TransactionRepository::new(db.clone());
    let workflow_repo = WorkflowRepository::new(db.clone());

    // 1. Setup Environment
    let user = create_user(&db).await;
    let org = create_org(&db, user.id).await;
    let _period = create_fiscal_data(&db, org.id).await;

    // 2. Create Chart of Accounts
    let cash_account = create_account(&db, org.id, "Cash USD", AccountType::Asset).await;
    let revenue_account = create_account(&db, org.id, "Sales Revenue", AccountType::Revenue).await;
    let expense_account = create_account(&db, org.id, "OpEx", AccountType::Expense).await;

    // 3. Generate Transactions
    // We will create a mix of simple USD transactions and Simulated Multi-Currency transactions
    // Note: The system assumes the client (or service) calculates conversions.
    // We will inject pre-calculated entries that respect the DB trigger `check_transaction_balance`.

    let transactions_to_create = vec![
        // Tx 1: Simple Revenue (USD 1000)
        (
            dec!(1000),
            cash_account.id,    // Debit
            revenue_account.id, // Credit
            "USD",
            dec!(1), // Rate
        ),
        // Tx 2: Expense (USD 50.55)
        (
            dec!(50.55),
            expense_account.id, // Debit
            cash_account.id,    // Credit
            "USD",
            dec!(1),
        ),
        // Tx 3: Large Amount (USD 1,000,000.00)
        (
            dec!(1000000),
            cash_account.id,
            revenue_account.id,
            "USD",
            dec!(1),
        ),
        // Tx 4: Complex Decimals (USD 123.4567)
        // Wait, DB limited to 4 decimals for amounts.
        (
            dec!(123.4567),
            expense_account.id,
            cash_account.id,
            "USD",
            dec!(1),
        ),
    ];

    for (amount, debit_acct, credit_acct, currency, rate) in transactions_to_create {
        let input = CreateTransactionInput {
            organization_id: org.id,
            transaction_type: TransactionType::Journal,
            transaction_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            description: format!("Global Invariant Test Tx {}", amount),
            reference_number: None,
            memo: None,
            entries: vec![
                CreateLedgerEntryInput {
                    account_id: debit_acct,
                    source_currency: currency.to_string(),
                    source_amount: amount,
                    exchange_rate: rate,
                    functional_currency: "USD".to_string(), // Org base is USD
                    functional_amount: amount * rate,       // Client calc
                    debit: amount * rate,                   // Debit functional
                    credit: dec!(0),
                    memo: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
                CreateLedgerEntryInput {
                    account_id: credit_acct,
                    source_currency: currency.to_string(),
                    source_amount: amount,
                    exchange_rate: rate,
                    functional_currency: "USD".to_string(),
                    functional_amount: amount * rate,
                    debit: dec!(0),
                    credit: amount * rate, // Credit functional
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

        let result = tx_repo
            .create_transaction(input)
            .await
            .expect("Failed to create tx");
        let tx_id = result.transaction.id;

        // Follow workflow: Draft -> Pending -> Approved -> Posted
        workflow_repo
            .submit_transaction(org.id, tx_id, user.id)
            .await
            .expect("Failed to submit");
        workflow_repo
            .approve_transaction(org.id, tx_id, user.id, None)
            .await
            .expect("Failed to approve");

        // Post checks existing balance triggers
        workflow_repo
            .post_transaction(org.id, tx_id, user.id)
            .await
            .expect("Failed to post transaction");
    }

    // 4. Verify Global Invariant via Raw SQL
    // We check specifically for THIS organization to filter out noise from other tests if concurrent
    let sql = r#"
        SELECT 
            COALESCE(SUM(debit), 0) as total_debit, 
            COALESCE(SUM(credit), 0) as total_credit
        FROM ledger_entries
        JOIN transactions ON transactions.id = ledger_entries.transaction_id
        WHERE transactions.organization_id = $1
          AND transactions.status = 'posted'
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        vec![org.id.into()],
    );

    let result = db.query_one(stmt).await.unwrap().unwrap();
    let total_debit: Decimal = result.try_get("", "total_debit").unwrap();
    let total_credit: Decimal = result.try_get("", "total_credit").unwrap();

    println!("Global Invariant Check for Org {}:", org.id);
    println!("  TOTAL DEBIT:  {}", total_debit);
    println!("  TOTAL CREDIT: {}", total_credit);
    println!("  DIFFERENCE:   {}", total_debit - total_credit);

    assert_eq!(
        total_debit, total_credit,
        "CRITICAL FAILURE: System is not balanced! Debits != Credits"
    );

    assert!(
        total_debit > dec!(0),
        "Sanity Check Failed: Total debit should be positive after transactions"
    );
}
