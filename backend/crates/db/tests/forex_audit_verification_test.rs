//! Integration tests for Round 2 Verification: Audit Immutability & Forex Logic.
//!
//! Verifies:
//! 1. Ledger Entry Immutability (Audit Trigger).
//! 2. Forex Gain/Loss Calculation & Posting Flow (Simulation).

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::too_many_lines)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, QueryFilter,
};
use std::env;
use uuid::Uuid;

use zeltra_core::currency::calculate_forex_variance;
use zeltra_db::entities::{
    chart_of_accounts, fiscal_periods, fiscal_years, ledger_entries, organization_users,
    organizations,
    sea_orm_active_enums::{
        AccountSubtype, AccountType, FiscalPeriodStatus, TransactionStatus, TransactionType,
        UserRole,
    },
    transactions, users,
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

struct TestData {
    org_id: Uuid,
    user_id: Uuid,
    fiscal_period_id: Uuid,
    asset_account_id: Uuid,
    expense_account_id: Uuid,
    gain_loss_account_id: Uuid,
}

async fn setup_test_data(db: &DatabaseConnection) -> Result<TestData, sea_orm::DbErr> {
    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let fiscal_year_id = Uuid::new_v4();
    let fiscal_period_id = Uuid::new_v4();
    let asset_account_id = Uuid::new_v4();
    let expense_account_id = Uuid::new_v4();
    let gain_loss_account_id = Uuid::new_v4();

    // Create user
    users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create organization
    organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", Uuid::new_v4())),
        slug: Set(format!("test-org-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create organization user
    organization_users::ActiveModel {
        organization_id: Set(org_id),
        user_id: Set(user_id),
        role: Set(UserRole::Owner),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create fiscal year
    fiscal_years::ActiveModel {
        id: Set(fiscal_year_id),
        organization_id: Set(org_id),
        name: Set("FY 2026".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create fiscal period (OPEN)
    fiscal_periods::ActiveModel {
        id: Set(fiscal_period_id),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fiscal_year_id),
        period_number: Set(1),
        name: Set("January 2026".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
        status: Set(FiscalPeriodStatus::Open),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create accounts
    let uuid_str = Uuid::new_v4().to_string();
    chart_of_accounts::ActiveModel {
        id: Set(asset_account_id),
        organization_id: Set(org_id),
        code: Set(format!("1000-{}", &uuid_str[..8])),
        name: Set("Bank EUR".to_string()),
        account_type: Set(AccountType::Asset),
        account_subtype: Set(Some(AccountSubtype::Cash)),
        currency: Set("EUR".to_string()), // Foreign currency account
        ..Default::default()
    }
    .insert(db)
    .await?;

    let uuid_str2 = Uuid::new_v4().to_string();
    chart_of_accounts::ActiveModel {
        id: Set(expense_account_id),
        organization_id: Set(org_id),
        code: Set(format!("5000-{}", &uuid_str2[..8])),
        name: Set("Consulting Expense".to_string()),
        account_type: Set(AccountType::Expense),
        account_subtype: Set(Some(AccountSubtype::OperatingExpense)),
        currency: Set("USD".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let uuid_str3 = Uuid::new_v4().to_string();
    chart_of_accounts::ActiveModel {
        id: Set(gain_loss_account_id),
        organization_id: Set(org_id),
        code: Set(format!("8000-{}", &uuid_str3[..8])),
        name: Set("Forex Gain/Loss".to_string()),
        account_type: Set(AccountType::Expense), // Usually Expense or Income
        account_subtype: Set(Some(AccountSubtype::OtherExpense)),
        currency: Set("USD".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(TestData {
        org_id,
        user_id,
        fiscal_period_id,
        asset_account_id,
        expense_account_id,
        gain_loss_account_id,
    })
}

async fn cleanup_test_data(db: &DatabaseConnection, data: &TestData) -> Result<(), sea_orm::DbErr> {
    ledger_entries::Entity::delete_many()
        .filter(ledger_entries::Column::AccountId.is_in([
            data.asset_account_id,
            data.expense_account_id,
            data.gain_loss_account_id,
        ]))
        .exec(db)
        .await?;

    transactions::Entity::delete_many()
        .filter(transactions::Column::OrganizationId.eq(data.org_id))
        .exec(db)
        .await?;

    chart_of_accounts::Entity::delete_many()
        .filter(chart_of_accounts::Column::OrganizationId.eq(data.org_id))
        .exec(db)
        .await?;

    fiscal_periods::Entity::delete_many()
        .filter(fiscal_periods::Column::Id.eq(data.fiscal_period_id))
        .exec(db)
        .await?;

    fiscal_years::Entity::delete_many()
        .filter(fiscal_years::Column::OrganizationId.eq(data.org_id))
        .exec(db)
        .await?;

    organization_users::Entity::delete_many()
        .filter(organization_users::Column::OrganizationId.eq(data.org_id))
        .exec(db)
        .await?;

    organizations::Entity::delete_by_id(data.org_id)
        .exec(db)
        .await?;
    users::Entity::delete_by_id(data.user_id).exec(db).await?;

    Ok(())
}

#[tokio::test]
async fn test_audit_immutability() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - db not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping setup: {}", e);
            return;
        }
    };

    // 1. Create a transaction with an entry
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Immutability Test".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        timezone: Set("UTC".to_string()),
        idempotency_key: Set(None),
        iso_metadata: Set(None),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create tx");

    let entry_id = Uuid::new_v4();
    let _entry = ledger_entries::ActiveModel {
        id: Set(entry_id),
        transaction_id: Set(tx_id),
        account_id: Set(data.expense_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(10000, 2)),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(10000, 2)),
        debit: Set(Decimal::new(10000, 2)),
        credit: Set(Decimal::ZERO),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to insert entry");

    // 2. Attempt to UPDATE the entry directly
    let update_result = ledger_entries::Entity::update_many()
        .col_expr(
            ledger_entries::Column::SourceAmount,
            sea_orm::sea_query::Expr::value(Decimal::new(9999, 2)),
        )
        .filter(ledger_entries::Column::Id.eq(entry_id))
        .exec(&db)
        .await;

    // 3. Assert Failure
    assert!(
        update_result.is_err(),
        "Update should fail due to trigger protection"
    );

    if let Err(e) = update_result {
        let err_str = e.to_string().to_lowercase();
        assert!(
            err_str.contains("ledger entries are immutable")
                || err_str.contains("cannot update ledger"),
            "Error unexpected: {}",
            e
        );
    }

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_audit_truncate_protection() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - db not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping setup: {}", e);
            return;
        }
    };

    // 1. Create Data
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        description: Set("Truncate Test".to_string()),
        status: Set(TransactionStatus::Posted),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create tx");

    // Insert BALANCED entries
    ledger_entries::Entity::insert_many(vec![
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx_id),
            account_id: Set(data.expense_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(Decimal::new(100, 2)),
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(Decimal::new(100, 2)),
            debit: Set(Decimal::new(100, 2)),
            credit: Set(Decimal::ZERO),
            ..Default::default()
        },
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx_id),
            account_id: Set(data.asset_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(Decimal::new(100, 2)),
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(Decimal::new(100, 2)),
            debit: Set(Decimal::ZERO),
            credit: Set(Decimal::new(100, 2)),
            ..Default::default()
        },
    ])
    .exec(&db)
    .await
    .expect("Failed to insert balanced entries");

    // 2. Attempt TRUNCATE (Should FAIL if protected, but likely PASS currently)
    let truncate_result = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "TRUNCATE TABLE ledger_entries".to_string(),
        ))
        .await;

    // Report vulnerability if successful
    if truncate_result.is_ok() {
        eprintln!(
            "CRITICAL VULNERABILITY: TRUNCATE TABLE ledger_entries succeeded! Audit trail bypassed."
        );
    }

    match truncate_result {
        Ok(_) => {
            // We intentionally fail the test if TRUNCATE succeeds to vividly demonstrate the flaw.
            panic!("VULNERABILITY CONFIRMED: TRUNCATE TABLE bypassed the audit trigger.");
        }
        Err(e) => {
            println!("Protected: TRUNCATE failed as expected. Reason: {}", e);
        }
    }

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_concurrent_aggregation_integrity() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - db not available: {}", e);
            return;
        }
    };
    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(_) => return,
    };

    // Spawn 10 concurrent tasks inserting entries
    let mut handles = vec![];
    for i in 0..10 {
        let db = db.clone();
        let org_id = data.org_id;
        let period_id = data.fiscal_period_id;
        let user_id = data.user_id;
        let expense_id = data.expense_account_id;
        let asset_id = data.asset_account_id;

        handles.push(tokio::spawn(async move {
            let tx_id = Uuid::new_v4();
            transactions::ActiveModel {
                id: Set(tx_id),
                organization_id: Set(org_id),
                fiscal_period_id: Set(period_id),
                transaction_type: Set(TransactionType::Journal),
                transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
                description: Set(format!("Concurrent {}", i)),
                status: Set(TransactionStatus::Posted),
                created_by: Set(user_id),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();

            // Insert BALANCED entries
            ledger_entries::Entity::insert_many(vec![
                ledger_entries::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    transaction_id: Set(tx_id),
                    account_id: Set(expense_id),
                    source_currency: Set("USD".to_string()),
                    source_amount: Set(Decimal::new(100, 2)), // 1.00
                    exchange_rate: Set(Decimal::ONE),
                    functional_currency: Set("USD".to_string()),
                    functional_amount: Set(Decimal::new(100, 2)),
                    debit: Set(Decimal::new(100, 2)),
                    credit: Set(Decimal::ZERO),
                    ..Default::default()
                },
                ledger_entries::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    transaction_id: Set(tx_id),
                    account_id: Set(asset_id),
                    source_currency: Set("USD".to_string()),
                    source_amount: Set(Decimal::new(100, 2)), // 1.00
                    exchange_rate: Set(Decimal::ONE),
                    functional_currency: Set("USD".to_string()),
                    functional_amount: Set(Decimal::new(100, 2)),
                    debit: Set(Decimal::ZERO),
                    credit: Set(Decimal::new(100, 2)),
                    ..Default::default()
                },
            ])
            .exec(&db)
            .await
            .unwrap();
        }));
    }

    // Await all inserts
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify Sum
    // We expect 10 entries of 1.00 = 10.00

    use sea_orm::{EntityTrait, QuerySelect};
    let sum_debit: Option<Decimal> = ledger_entries::Entity::find()
        .filter(ledger_entries::Column::AccountId.eq(data.expense_account_id))
        .select_only()
        .column_as(ledger_entries::Column::Debit.sum(), "total_debit")
        .into_tuple()
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    let total = sum_debit.unwrap_or(Decimal::ZERO);
    assert_eq!(total, Decimal::new(1000, 2)); // 10.00

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_forex_gain_loss_logic_simulation() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - db not available: {}", e);
            return;
        }
    };
    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(_) => return,
    };

    // Scenario:
    // Invoice: 100 EUR @ 1.1 USD/EUR (Functional = 110 USD)
    // Pay: 100 EUR @ 1.2 USD/EUR (Functional = 120 USD)
    // Result: Paid 120 USD for a 110 USD liability. Loss of 10 USD.

    let invoice_amount_eur = Decimal::new(10000, 2); // 100.00
    let original_rate = Decimal::new(11000, 4); // 1.1000
    let payment_rate = Decimal::new(12000, 4); // 1.2000

    // 1. Verify Variance Calculation (Unit Test-ish)
    let variance = calculate_forex_variance(invoice_amount_eur, original_rate, payment_rate);
    // (1.2 - 1.1) * 100 = 0.1 * 100 = 10.00
    let expected_variance = Decimal::new(1000, 2); // 10.00
    assert_eq!(variance, expected_variance, "Variance calculation mismatch");

    // 2. Simulate Inserting Payment Transaction with Gain/Loss Entries
    // We manually construct balanced entries as the handler would.
    let bank_functional = invoice_amount_eur * payment_rate; // 100 * 1.2 = 120
    let clearing_functional = invoice_amount_eur * original_rate; // 100 * 1.1 = 110

    // Variance is +10 (Loss)
    // Entries:
    // 1. Credit Bank (Payment): 120
    // 2. Debit AP (Clearing): 110
    // 3. Debit Loss (Variance): 10
    // Total Debit (110+10=120) == Total Credit (120). Balanced!

    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Payment),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 20).unwrap()),
        description: Set("Forex Payment Sim".to_string()),
        status: Set(TransactionStatus::Posted),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create payment tx");

    ledger_entries::Entity::insert_many(vec![
        // Credit Bank (Asset)
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx_id),
            account_id: Set(data.asset_account_id),
            source_currency: Set("EUR".to_string()),
            source_amount: Set(invoice_amount_eur),
            exchange_rate: Set(payment_rate),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(bank_functional),
            debit: Set(Decimal::ZERO),
            credit: Set(bank_functional), // 120
            memo: Set(Some("Payment".to_string())),
            ..Default::default()
        },
        // Debit AP (Expense/Liability - mocking as expense for simplicity)
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx_id),
            account_id: Set(data.expense_account_id), // Mocking AP account
            source_currency: Set("EUR".to_string()),
            source_amount: Set(invoice_amount_eur),
            exchange_rate: Set(original_rate),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(clearing_functional),
            debit: Set(clearing_functional), // 110
            credit: Set(Decimal::ZERO),
            memo: Set(Some("Clearing".to_string())),
            ..Default::default()
        },
        // Debit Variance (Loss)
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx_id),
            account_id: Set(data.gain_loss_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(variance),
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(variance),
            debit: Set(variance), // 10
            credit: Set(Decimal::ZERO),
            memo: Set(Some("Realized Loss".to_string())),
            ..Default::default()
        },
    ])
    .exec(&db)
    .await
    .expect("Failed to insert balanced batch");

    // Pass implies DB accepted the transaction (balanced).

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_backdated_balance_reporting() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - db not available: {}", e);
            return;
        }
    };
    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(_) => return,
    };

    let repo = zeltra_db::repositories::account::AccountRepository::new(db.clone());

    // 1. Transaction 1: Today (2026-01-13) - $100
    let tx1_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx1_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 13).unwrap()),
        description: Set("Today's Tx".to_string()),
        status: Set(TransactionStatus::Posted),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    ledger_entries::Entity::insert_many(vec![
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx1_id),
            account_id: Set(data.expense_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(Decimal::new(10000, 2)), // 100.00
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(Decimal::new(10000, 2)),
            debit: Set(Decimal::new(10000, 2)),
            credit: Set(Decimal::ZERO),
            account_version: Set(1),
            account_previous_balance: Set(Decimal::ZERO),
            account_current_balance: Set(Decimal::new(10000, 2)),
            ..Default::default()
        },
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx1_id),
            account_id: Set(data.asset_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(Decimal::new(10000, 2)),
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(Decimal::new(10000, 2)),
            debit: Set(Decimal::ZERO),
            credit: Set(Decimal::new(10000, 2)),
            account_version: Set(1),
            account_previous_balance: Set(Decimal::ZERO),
            account_current_balance: Set(Decimal::new(-10000, 2)),
            ..Default::default()
        },
    ])
    .exec(&db)
    .await
    .unwrap();

    // 2. Transaction 2: Backdated (Yesterday 2026-01-12) - $50
    let tx2_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx2_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2026, 1, 12).unwrap()),
        description: Set("Backdated Tx".to_string()),
        status: Set(TransactionStatus::Posted),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Simulating backdated entry that gets HIGH VERSION but PAST DATE
    ledger_entries::Entity::insert_many(vec![
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx2_id),
            account_id: Set(data.expense_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(Decimal::new(5000, 2)), // 50.00
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(Decimal::new(5000, 2)),
            debit: Set(Decimal::new(5000, 2)),
            credit: Set(Decimal::ZERO),
            account_version: Set(2),
            account_previous_balance: Set(Decimal::new(10000, 2)),
            account_current_balance: Set(Decimal::new(15000, 2)), // 150.00
            ..Default::default()
        },
        ledger_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            transaction_id: Set(tx2_id),
            account_id: Set(data.asset_account_id),
            source_currency: Set("USD".to_string()),
            source_amount: Set(Decimal::new(5000, 2)),
            exchange_rate: Set(Decimal::ONE),
            functional_currency: Set("USD".to_string()),
            functional_amount: Set(Decimal::new(5000, 2)),
            debit: Set(Decimal::ZERO),
            credit: Set(Decimal::new(5000, 2)),
            account_version: Set(2),
            account_previous_balance: Set(Decimal::new(-10000, 2)),
            account_current_balance: Set(Decimal::new(-15000, 2)),
            ..Default::default()
        },
    ])
    .exec(&db)
    .await
    .unwrap();

    // 3. Verify Balance as of "Yesterday" (2026-01-12)
    let balance_yesterday = repo
        .get_balance_at_date(
            data.expense_account_id,
            NaiveDate::from_ymd_opt(2026, 1, 12).unwrap(),
        )
        .await
        .unwrap();

    println!(
        "Balance Yesterday (Found: {}, Expected: 50.00)",
        balance_yesterday
    );

    // Proving the flaw:
    assert_eq!(
        balance_yesterday,
        Decimal::new(5000, 2),
        "Mata Dewa Result: Point-in-Time reporting is WRONG. It found {} instead of 50.00",
        balance_yesterday
    );

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}
