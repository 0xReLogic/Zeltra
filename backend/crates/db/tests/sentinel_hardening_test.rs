#![allow(missing_docs)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_lines)]
#![allow(unused_imports)]

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::env;
use uuid::Uuid;

use zeltra_core::ledger::types::AccrualFrequency;
use zeltra_db::entities::{
    accrual_schedules, chart_of_accounts, currencies, fiscal_periods, fiscal_years, organizations,
    revaluation_logs,
    sea_orm_active_enums::{
        AccountType, FiscalPeriodStatus, FiscalYearStatus, RateSource, SubscriptionStatus,
        SubscriptionTier, TransactionType,
    },
    users,
};
use zeltra_db::entities::{sea_orm_active_enums::TransactionStatus, transactions};
use zeltra_db::repositories::accrual::{AccrualRepository, CreateAccrualScheduleInput};
use zeltra_db::repositories::exchange_rate::{CreateExchangeRateInput, ExchangeRateRepository};
use zeltra_db::repositories::revaluation::RevaluationRepository;
use zeltra_db::repositories::transaction::{
    CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository,
};

/// Sentinel hardening tests.
fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

async fn setup_test_org(db: &DatabaseConnection) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let system_user_id = Uuid::nil();
    let fiscal_year_id = Uuid::new_v4();
    let fiscal_period_id = Uuid::new_v4();
    let random_id = &Uuid::new_v4().to_string()[..8];

    println!("DEBUG: setup_test_org starting for org {}", org_id);

    // 1. Ensure Currencies exist
    let idr = currencies::ActiveModel {
        code: Set("IDR".to_string()),
        name: Set("Indonesian Rupiah".to_string()),
        symbol: Set("Rp".to_string()),
        decimal_places: Set(0),
        is_active: Set(true),
    };
    let usd = currencies::ActiveModel {
        code: Set("USD".to_string()),
        name: Set("US Dollar".to_string()),
        symbol: Set("$".to_string()),
        decimal_places: Set(2),
        is_active: Set(true),
    };

    let _ = currencies::Entity::insert(idr)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(currencies::Column::Code)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await;
    let _ = currencies::Entity::insert(usd)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(currencies::Column::Code)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await;

    // 2. Create System User (ID = nil)
    let system_user = users::ActiveModel {
        id: Set(system_user_id),
        email: Set("system@zeltra.ai".to_string()),
        password_hash: Set("system_hash".to_string()),
        full_name: Set("SystemUser".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    let _ = users::Entity::insert(system_user)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(users::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await;

    // 3. Create Real User
    users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@zeltra.ai", random_id)),
        password_hash: Set("fixed_hash".to_string()),
        full_name: Set("Sentinel Hardening Test User".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert user");

    // 4. Create Org
    organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Hardening-{}", random_id)),
        slug: Set(format!("hardening-{}", random_id)),
        base_currency: Set("IDR".to_string()),
        timezone: Set("UTC".to_string()),
        settings: Set(serde_json::json!({})),
        is_active: Set(true),
        subscription_tier: Set(SubscriptionTier::Starter),
        subscription_status: Set(SubscriptionStatus::Trialing),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert organization");

    // 5. Create Fiscal Year
    fiscal_years::ActiveModel {
        id: Set(fiscal_year_id),
        organization_id: Set(org_id),
        name: Set(format!("FY2026-{}", random_id)),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        status: Set(FiscalYearStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert fiscal year");

    // 6. Create Fiscal Period
    fiscal_periods::ActiveModel {
        id: Set(fiscal_period_id),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fiscal_year_id),
        name: Set(format!("P01-2026-{}", random_id)),
        period_number: Set(1),
        is_adjustment_period: Set(false),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        status: Set(FiscalPeriodStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert fiscal period");

    // 7. Create Accounts
    let bank_id = Uuid::new_v4();
    let expense_id = Uuid::new_v4();
    let equity_id = Uuid::new_v4();

    chart_of_accounts::ActiveModel {
        id: Set(bank_id),
        organization_id: Set(org_id),
        code: Set(format!("1101-{}", random_id)),
        name: Set("Bank USD".to_string()),
        account_type: Set(AccountType::Asset),
        currency: Set("USD".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert bank account");

    chart_of_accounts::ActiveModel {
        id: Set(expense_id),
        organization_id: Set(org_id),
        code: Set(format!("6101-{}", random_id)),
        name: Set("General Expense".to_string()),
        account_type: Set(AccountType::Expense),
        currency: Set("IDR".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert expense account");

    chart_of_accounts::ActiveModel {
        id: Set(equity_id),
        organization_id: Set(org_id),
        code: Set(format!("3101-{}", random_id)),
        name: Set("Retained Earnings".to_string()),
        account_type: Set(AccountType::Equity),
        currency: Set("IDR".to_string()),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("Failed to insert equity account");

    (
        org_id,
        bank_id,
        expense_id,
        equity_id,
        user_id,
        fiscal_period_id,
    )
}

#[tokio::test]
async fn test_accrual_amendment_consistency() {
    println!("DEBUG: test_accrual_amendment_consistency started");
    let db: DatabaseConnection = Database::connect(get_database_url())
        .await
        .expect("Failed to connect to DB");
    let (org_id, bank_id, expense_id, _, _, _) = setup_test_org(&db).await;

    let accrual_repo = AccrualRepository::new(db.clone());
    let tx_repo = TransactionRepository::new(db.clone());

    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    let input = CreateAccrualScheduleInput {
        organization_id: org_id,
        name: format!("Accrual-{}", &Uuid::new_v4().to_string()[..8]),
        description: None,
        total_amount: dec!(1200),
        currency_id: "USD".to_string(),
        debit_account_id: expense_id,
        credit_account_id: bank_id,
        start_date,
        end_date: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
        frequency: AccrualFrequency::Monthly,
        total_periods: 3,
        next_run_date: Some(start_date),
    };

    println!("DEBUG: creating accrual schedule");
    let schedule = accrual_repo
        .create_schedule(input)
        .await
        .expect("Failed to create schedule");

    println!("DEBUG: processing first run");
    accrual_repo
        .process_due_accruals(&tx_repo, start_date)
        .await
        .expect("Failed to process first run");

    let schedule: accrual_schedules::Model = accrual_schedules::Entity::find_by_id(schedule.id)
        .one(&db)
        .await
        .unwrap()
        .expect("Schedule disappeared");
    assert_eq!(schedule.total_amount_recognized, dec!(400));

    println!("DEBUG: amending schedule");
    let mut active: accrual_schedules::ActiveModel = schedule.into();
    active.total_periods = Set(5);
    let schedule: accrual_schedules::Model =
        active.update(&db).await.expect("Failed to update schedule");

    let run_date_2 = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
    println!("DEBUG: processing second run");
    accrual_repo
        .process_due_accruals(&tx_repo, run_date_2)
        .await
        .expect("Failed to process second run");

    let schedule: accrual_schedules::Model = accrual_schedules::Entity::find_by_id(schedule.id)
        .one(&db)
        .await
        .unwrap()
        .expect("Schedule disappeared after 2nd run");
    assert_eq!(schedule.total_amount_recognized, dec!(600));

    println!("DEBUG: processing remaining runs");
    accrual_repo
        .process_due_accruals(&tx_repo, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        .await
        .expect("Failed 3rd run");
    accrual_repo
        .process_due_accruals(&tx_repo, NaiveDate::from_ymd_opt(2026, 4, 1).unwrap())
        .await
        .expect("Failed 4th run");
    accrual_repo
        .process_due_accruals(&tx_repo, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        .await
        .expect("Failed 5th run");

    let schedule: accrual_schedules::Model = accrual_schedules::Entity::find_by_id(schedule.id)
        .one(&db)
        .await
        .unwrap()
        .expect("Schedule disappeared at end");
    assert_eq!(schedule.total_amount_recognized, dec!(1200));
    println!("DEBUG: test_accrual_amendment_consistency PASSED");
}

#[tokio::test]
async fn test_revaluation_concurrency_protection() {
    println!("DEBUG: test_revaluation_concurrency_protection started");
    let db: DatabaseConnection = Database::connect(get_database_url())
        .await
        .expect("Failed to connect to DB");
    let (org_id, bank_id, expense_id, equity_id, user_id, _) = setup_test_org(&db).await;

    let reval_repo = RevaluationRepository::new(db.clone());
    let tx_repo = TransactionRepository::new(db.clone());
    let rate_repo = ExchangeRateRepository::new(db.clone());

    let as_of = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
    println!("DEBUG: setup exchange rate");
    rate_repo
        .create_or_update_rate(CreateExchangeRateInput {
            organization_id: org_id,
            from_currency: "USD".to_string(),
            to_currency: "IDR".to_string(),
            rate: dec!(15000),
            effective_date: as_of,
            source: RateSource::Manual,
            source_reference: None,
            created_by: Some(user_id),
        })
        .await
        .expect("Failed to set exchange rate");

    println!("DEBUG: create initial balance");
    let initial_tx = tx_repo
        .create_transaction(CreateTransactionInput {
            organization_id: org_id,
            transaction_type: TransactionType::Journal,
            transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            description: "Initial Balance".to_string(),
            reference_number: None,
            memo: None,
            entries: vec![
                CreateLedgerEntryInput {
                    account_id: bank_id,
                    source_currency: "USD".to_string(),
                    source_amount: dec!(100),
                    exchange_rate: dec!(14000),
                    functional_currency: "IDR".to_string(),
                    functional_amount: dec!(1400000),
                    debit: dec!(1400000),
                    credit: dec!(0),
                    memo: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
                CreateLedgerEntryInput {
                    account_id: equity_id,
                    source_currency: "IDR".to_string(),
                    source_amount: dec!(1400000),
                    exchange_rate: dec!(1),
                    functional_currency: "IDR".to_string(),
                    functional_amount: dec!(1400000),
                    debit: dec!(0),
                    credit: dec!(1400000),
                    memo: None,
                    compliance_metadata: None,
                    dimensions: vec![],
                },
            ],
            created_by: user_id,
            timezone: "UTC".to_string(),
            idempotency_key: Some(Uuid::new_v4()),
            iso_metadata: None,
        })
        .await
        .expect("Failed to create initial transaction");

    // Manually post the transaction so revaluation can see it
    let mut active_tx: transactions::ActiveModel = initial_tx.transaction.into();
    active_tx.status = Set(TransactionStatus::Posted);
    active_tx
        .update(&db)
        .await
        .expect("Failed to post initial transaction");

    println!("DEBUG: first revaluation run");
    let processed = reval_repo
        .process_revaluations(org_id, as_of, expense_id, &rate_repo, &tx_repo)
        .await
        .expect("Failed first revaluation");
    assert_eq!(processed, 1);

    let logs: Vec<revaluation_logs::Model> = revaluation_logs::Entity::find()
        .filter(revaluation_logs::Column::OrganizationId.eq(org_id))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(logs.len(), 1);

    println!("DEBUG: second revaluation run");
    let processed_again = reval_repo
        .process_revaluations(org_id, as_of, expense_id, &rate_repo, &tx_repo)
        .await
        .expect("Failed second revaluation");
    assert_eq!(processed_again, 0);

    let logs_final: Vec<revaluation_logs::Model> = revaluation_logs::Entity::find()
        .filter(revaluation_logs::Column::OrganizationId.eq(org_id))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(logs_final.len(), 1);
    println!("DEBUG: test_revaluation_concurrency_protection PASSED");
}
