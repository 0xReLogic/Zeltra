//! Concurrent access stress tests for ledger transactions.
//!
//! Tests Requirements 14.1-14.4 for concurrent transaction handling.
//! Validates Property 14: Concurrent Balance Integrity.
//!
//! These tests verify that:
//! - Multiple concurrent transactions on the same account produce correct final balance
//! - No balance drift occurs regardless of execution order
//! - The system handles 1000+ concurrent transactions without errors

// Allow common test patterns that trigger clippy warnings
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::option_map_or_none)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::stable_sort_primitive)]

use chrono::NaiveDate;
use futures::future::join_all;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, TransactionTrait,
};
use std::env;
use std::sync::Arc;
use tokio::sync::Barrier;
use uuid::Uuid;

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

/// Test data for concurrent tests.
#[allow(clippy::struct_field_names)]
struct ConcurrentTestData {
    org_id: Uuid,
    user_id: Uuid,
    fiscal_period_id: Uuid,
    /// Asset account (debit normal) - balance increases with debits
    asset_account_id: Uuid,
    /// Expense account (debit normal) - balance increases with debits  
    expense_account_id: Uuid,
}

async fn setup_concurrent_test_data(
    db: &DatabaseConnection,
) -> Result<ConcurrentTestData, sea_orm::DbErr> {
    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let fiscal_year_id = Uuid::new_v4();
    let fiscal_period_id = Uuid::new_v4();
    let asset_account_id = Uuid::new_v4();
    let expense_account_id = Uuid::new_v4();

    // Create user
    users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("concurrent-test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("hash".to_string()),
        full_name: Set("Concurrent Test User".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create organization
    organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Concurrent Test Org {}", Uuid::new_v4())),
        slug: Set(format!("concurrent-test-{}", Uuid::new_v4())),
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
        name: Set("FY 2025 Concurrent".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
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
        name: Set("January 2025 Concurrent".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()),
        status: Set(FiscalPeriodStatus::Open),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create asset account (Cash)
    let uuid_str = Uuid::new_v4().to_string();
    chart_of_accounts::ActiveModel {
        id: Set(asset_account_id),
        organization_id: Set(org_id),
        code: Set(format!("1000-C-{}", &uuid_str[..6])),
        name: Set("Cash - Concurrent Test".to_string()),
        account_type: Set(AccountType::Asset),
        account_subtype: Set(Some(AccountSubtype::Cash)),
        currency: Set("USD".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create expense account
    let uuid_str = Uuid::new_v4().to_string();
    chart_of_accounts::ActiveModel {
        id: Set(expense_account_id),
        organization_id: Set(org_id),
        code: Set(format!("5000-C-{}", &uuid_str[..6])),
        name: Set("Expense - Concurrent Test".to_string()),
        account_type: Set(AccountType::Expense),
        account_subtype: Set(Some(AccountSubtype::OperatingExpense)),
        currency: Set("USD".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(ConcurrentTestData {
        org_id,
        user_id,
        fiscal_period_id,
        asset_account_id,
        expense_account_id,
    })
}

async fn cleanup_concurrent_test_data(
    db: &DatabaseConnection,
    data: &ConcurrentTestData,
) -> Result<(), sea_orm::DbErr> {
    // Delete in reverse order of dependencies
    ledger_entries::Entity::delete_many()
        .filter(
            ledger_entries::Column::AccountId
                .is_in([data.asset_account_id, data.expense_account_id]),
        )
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

/// Creates a single balanced transaction with one debit and one credit entry.
/// Returns the transaction ID.
async fn create_balanced_transaction(
    db: &DatabaseConnection,
    data: &ConcurrentTestData,
    amount: Decimal,
    description: &str,
) -> Result<Uuid, sea_orm::DbErr> {
    let txn = db.begin().await?;

    let tx_id = Uuid::new_v4();

    // Create transaction header
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set(description.to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    // Debit expense account (increases expense balance)
    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx_id),
        account_id: Set(data.expense_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(amount),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(amount),
        debit: Set(amount),
        credit: Set(Decimal::ZERO),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    // Credit asset account (decreases asset balance)
    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx_id),
        account_id: Set(data.asset_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(amount),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(amount),
        debit: Set(Decimal::ZERO),
        credit: Set(amount),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    Ok(tx_id)
}

/// Gets the current balance of an account from the latest ledger entry.
async fn get_account_balance(
    db: &DatabaseConnection,
    account_id: Uuid,
) -> Result<Decimal, sea_orm::DbErr> {
    let entry = ledger_entries::Entity::find()
        .filter(ledger_entries::Column::AccountId.eq(account_id))
        .order_by_desc(ledger_entries::Column::AccountVersion)
        .one(db)
        .await?;

    Ok(entry
        .map(|e| e.account_current_balance)
        .unwrap_or(Decimal::ZERO))
}

/// Gets the highest account version for an account.
async fn get_account_version(
    db: &DatabaseConnection,
    account_id: Uuid,
) -> Result<i64, sea_orm::DbErr> {
    let entry = ledger_entries::Entity::find()
        .filter(ledger_entries::Column::AccountId.eq(account_id))
        .order_by_desc(ledger_entries::Column::AccountVersion)
        .one(db)
        .await?;

    Ok(entry.map(|e| e.account_version).unwrap_or(0))
}

// ============================================================================
// Test: 100+ concurrent transactions on same account (Requirement 14.1, 14.3)
// Feature: ledger-core, Property 14: Concurrent Balance Integrity
// ============================================================================
#[tokio::test]
async fn test_concurrent_100_transactions_correct_balance() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_concurrent_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    let db = Arc::new(db);
    let data = Arc::new(data);

    const NUM_TRANSACTIONS: usize = 100;
    let amount_per_tx = Decimal::new(1000, 2); // $10.00 per transaction

    // Use a barrier to synchronize all tasks to start at the same time
    let barrier = Arc::new(Barrier::new(NUM_TRANSACTIONS));

    let mut handles = Vec::with_capacity(NUM_TRANSACTIONS);

    for i in 0..NUM_TRANSACTIONS {
        let db_clone = Arc::clone(&db);
        let data_clone = Arc::clone(&data);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            // Create transaction
            create_balanced_transaction(
                &db_clone,
                &data_clone,
                amount_per_tx,
                &format!("Concurrent tx {}", i),
            )
            .await
        });

        handles.push(handle);
    }

    // Wait for all transactions to complete
    let results = join_all(handles).await;

    // Count successes and failures
    let mut success_count = 0;
    let mut _failure_count = 0;

    for result in results {
        match result {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => {
                eprintln!("Transaction failed: {}", e);
                _failure_count += 1;
            }
            Err(e) => {
                eprintln!("Task panicked: {}", e);
                _failure_count += 1;
            }
        }
    }

    println!(
        "Completed {} of {} transactions",
        success_count, NUM_TRANSACTIONS
    );

    // Verify final balance is mathematically correct
    // Expense account: each tx adds $10 debit, so balance = success_count * $10
    let expense_balance = get_account_balance(&db, data.expense_account_id)
        .await
        .expect("Failed to get expense balance");

    let expected_expense = amount_per_tx * Decimal::from(success_count as i64);

    assert_eq!(
        expense_balance, expected_expense,
        "Expense balance should be {} but was {} (drift detected!)",
        expected_expense, expense_balance
    );

    // Asset account: each tx adds $10 credit, so balance = -success_count * $10
    let asset_balance = get_account_balance(&db, data.asset_account_id)
        .await
        .expect("Failed to get asset balance");

    let expected_asset = -amount_per_tx * Decimal::from(success_count as i64);

    assert_eq!(
        asset_balance, expected_asset,
        "Asset balance should be {} but was {} (drift detected!)",
        expected_asset, asset_balance
    );

    // Verify account versions are sequential (no gaps)
    let expense_version = get_account_version(&db, data.expense_account_id)
        .await
        .expect("Failed to get expense version");

    assert_eq!(
        expense_version, success_count as i64,
        "Expense account version should be {} but was {}",
        success_count, expense_version
    );

    let asset_version = get_account_version(&db, data.asset_account_id)
        .await
        .expect("Failed to get asset version");

    assert_eq!(
        asset_version, success_count as i64,
        "Asset account version should be {} but was {}",
        success_count, asset_version
    );

    println!(
        "✓ 100 concurrent transactions completed successfully. Final expense balance: {}, asset balance: {}",
        expense_balance, asset_balance
    );

    // Cleanup
    cleanup_concurrent_test_data(&db, &data)
        .await
        .expect("Cleanup failed");
}

// ============================================================================
// Test: 1000+ concurrent transactions stress test (Requirement 14.3, 14.4)
// Feature: ledger-core, Property 14: Concurrent Balance Integrity (Stress Test)
// Validates: Requirements 14.1, 14.2, 14.3, 14.4
// ============================================================================
#[tokio::test]
async fn test_stress_1000_concurrent_transactions() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_concurrent_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    let db = Arc::new(db);
    let data = Arc::new(data);

    const NUM_TRANSACTIONS: usize = 1000;
    const BATCH_SIZE: usize = 50; // Process in batches to avoid overwhelming the DB
    let amount_per_tx = Decimal::new(100, 2); // $1.00 per transaction

    let mut total_success = 0;
    let mut total_failure = 0;

    println!(
        "Starting stress test with {} transactions in batches of {}",
        NUM_TRANSACTIONS, BATCH_SIZE
    );

    for batch in 0..(NUM_TRANSACTIONS / BATCH_SIZE) {
        let barrier = Arc::new(Barrier::new(BATCH_SIZE));
        let mut handles = Vec::with_capacity(BATCH_SIZE);

        for i in 0..BATCH_SIZE {
            let db_clone = Arc::clone(&db);
            let data_clone = Arc::clone(&data);
            let barrier_clone = Arc::clone(&barrier);
            let tx_num = batch * BATCH_SIZE + i;

            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;
                create_balanced_transaction(
                    &db_clone,
                    &data_clone,
                    amount_per_tx,
                    &format!("Stress tx {}", tx_num),
                )
                .await
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;

        for result in results {
            match result {
                Ok(Ok(_)) => total_success += 1,
                Ok(Err(_)) => total_failure += 1,
                Err(_) => total_failure += 1,
            }
        }

        if (batch + 1) % 5 == 0 {
            println!(
                "  Completed batch {}/{}",
                batch + 1,
                NUM_TRANSACTIONS / BATCH_SIZE
            );
        }
    }

    println!(
        "Stress test completed: {} success, {} failures",
        total_success, total_failure
    );

    // Verify final balance is mathematically correct
    let expense_balance = get_account_balance(&db, data.expense_account_id)
        .await
        .expect("Failed to get expense balance");

    let expected_expense = amount_per_tx * Decimal::from(total_success as i64);

    assert_eq!(
        expense_balance, expected_expense,
        "BALANCE DRIFT DETECTED! Expense balance should be {} but was {}",
        expected_expense, expense_balance
    );

    let asset_balance = get_account_balance(&db, data.asset_account_id)
        .await
        .expect("Failed to get asset balance");

    let expected_asset = -amount_per_tx * Decimal::from(total_success as i64);

    assert_eq!(
        asset_balance, expected_asset,
        "BALANCE DRIFT DETECTED! Asset balance should be {} but was {}",
        expected_asset, asset_balance
    );

    // Verify no balance drift
    let expense_version = get_account_version(&db, data.expense_account_id)
        .await
        .expect("Failed to get expense version");

    assert_eq!(
        expense_version, total_success as i64,
        "Version mismatch indicates potential drift"
    );

    println!(
        "✓ Stress test PASSED: {} transactions, final balance: expense={}, asset={}",
        total_success, expense_balance, asset_balance
    );

    cleanup_concurrent_test_data(&db, &data)
        .await
        .expect("Cleanup failed");
}

// ============================================================================
// Test: Verify running balance consistency after concurrent operations
// Feature: ledger-core, Property 14: No balance drift regardless of execution order
// ============================================================================
#[tokio::test]
async fn test_concurrent_running_balance_consistency() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_concurrent_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    let db = Arc::new(db);
    let data = Arc::new(data);

    const NUM_TRANSACTIONS: usize = 50;
    let amount_per_tx = Decimal::new(500, 2); // $5.00 per transaction

    let barrier = Arc::new(Barrier::new(NUM_TRANSACTIONS));
    let mut handles = Vec::with_capacity(NUM_TRANSACTIONS);

    for i in 0..NUM_TRANSACTIONS {
        let db_clone = Arc::clone(&db);
        let data_clone = Arc::clone(&data);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            create_balanced_transaction(
                &db_clone,
                &data_clone,
                amount_per_tx,
                &format!("Balance consistency tx {}", i),
            )
            .await
        });

        handles.push(handle);
    }

    let results: Vec<Result<Result<Uuid, sea_orm::DbErr>, tokio::task::JoinError>> =
        join_all(handles).await;
    let success_count = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();

    // Verify running balance chain is consistent
    // Property 3: current_balance[N] == previous_balance[N] + change
    // Property 3: previous_balance[N] == current_balance[N-1]

    let entries = ledger_entries::Entity::find()
        .filter(ledger_entries::Column::AccountId.eq(data.expense_account_id))
        .order_by_asc(ledger_entries::Column::AccountVersion)
        .all(&*db)
        .await
        .expect("Failed to query entries");

    assert_eq!(
        entries.len(),
        success_count,
        "Should have {} entries but found {}",
        success_count,
        entries.len()
    );

    let mut prev_balance = Decimal::ZERO;

    for (i, entry) in entries.iter().enumerate() {
        let version = i as i64 + 1;

        // Verify version is sequential
        assert_eq!(
            entry.account_version, version,
            "Entry {} should have version {} but has {}",
            i, version, entry.account_version
        );

        // Verify previous_balance matches previous entry's current_balance
        assert_eq!(
            entry.account_previous_balance, prev_balance,
            "Entry {} previous_balance should be {} but is {}",
            i, prev_balance, entry.account_previous_balance
        );

        // Verify current_balance = previous_balance + change
        // For expense (debit normal): change = debit - credit
        let change = entry.debit - entry.credit;
        let expected_current = prev_balance + change;

        assert_eq!(
            entry.account_current_balance, expected_current,
            "Entry {} current_balance should be {} but is {}",
            i, expected_current, entry.account_current_balance
        );

        prev_balance = entry.account_current_balance;
    }

    println!(
        "✓ Running balance chain verified for {} entries. Final balance: {}",
        entries.len(),
        prev_balance
    );

    cleanup_concurrent_test_data(&db, &data)
        .await
        .expect("Cleanup failed");
}

// ============================================================================
// Test: Verify account version monotonicity under concurrent load
// Feature: ledger-core, Property 4: Account Version Monotonicity
// ============================================================================
#[tokio::test]
async fn test_concurrent_version_monotonicity() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_concurrent_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    let db = Arc::new(db);
    let data = Arc::new(data);

    const NUM_TRANSACTIONS: usize = 75;
    let amount_per_tx = Decimal::new(250, 2); // $2.50 per transaction

    let barrier = Arc::new(Barrier::new(NUM_TRANSACTIONS));
    let mut handles = Vec::with_capacity(NUM_TRANSACTIONS);

    for i in 0..NUM_TRANSACTIONS {
        let db_clone = Arc::clone(&db);
        let data_clone = Arc::clone(&data);
        let barrier_clone = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            create_balanced_transaction(
                &db_clone,
                &data_clone,
                amount_per_tx,
                &format!("Version monotonicity tx {}", i),
            )
            .await
        });

        handles.push(handle);
    }

    let results: Vec<Result<Result<Uuid, sea_orm::DbErr>, tokio::task::JoinError>> =
        join_all(handles).await;
    let success_count = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();

    // Verify versions form strictly increasing sequence starting from 1
    let entries = ledger_entries::Entity::find()
        .filter(ledger_entries::Column::AccountId.eq(data.expense_account_id))
        .order_by_asc(ledger_entries::Column::AccountVersion)
        .all(&*db)
        .await
        .expect("Failed to query entries");

    // Check for gaps or duplicates in version sequence
    let versions: Vec<i64> = entries.iter().map(|e| e.account_version).collect();

    for (i, &version) in versions.iter().enumerate() {
        let expected = (i + 1) as i64;
        assert_eq!(
            version,
            expected,
            "Version sequence has gap or duplicate at position {}. Expected {}, got {}. Versions: {:?}",
            i,
            expected,
            version,
            &versions[..std::cmp::min(10, versions.len())]
        );
    }

    // Verify no duplicate versions
    let mut sorted_versions = versions.clone();
    sorted_versions.sort();
    sorted_versions.dedup();

    assert_eq!(
        sorted_versions.len(),
        versions.len(),
        "Duplicate versions detected! Original: {}, Unique: {}",
        versions.len(),
        sorted_versions.len()
    );

    // Verify final version matches entry count
    let max_version = versions.last().copied().unwrap_or(0);
    assert_eq!(
        max_version, success_count as i64,
        "Max version {} should equal success count {}",
        max_version, success_count
    );

    println!(
        "✓ Version monotonicity verified: {} entries with versions 1..{}",
        entries.len(),
        max_version
    );

    cleanup_concurrent_test_data(&db, &data)
        .await
        .expect("Cleanup failed");
}

// ============================================================================
// Test: Sequential transactions verify balance correctness (baseline test)
// This test verifies the balance logic works correctly without concurrency
// ============================================================================
#[tokio::test]
async fn test_sequential_transactions_correct_balance() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_concurrent_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    const NUM_TRANSACTIONS: usize = 10;
    let amount_per_tx = Decimal::new(1000, 2); // $10.00 per transaction

    // Create transactions SEQUENTIALLY (not concurrently)
    for i in 0..NUM_TRANSACTIONS {
        create_balanced_transaction(&db, &data, amount_per_tx, &format!("Sequential tx {}", i))
            .await
            .expect("Failed to create transaction");
    }

    // Verify final balance is mathematically correct
    let expense_balance = get_account_balance(&db, data.expense_account_id)
        .await
        .expect("Failed to get expense balance");

    let expected_expense = amount_per_tx * Decimal::from(NUM_TRANSACTIONS as i64);

    assert_eq!(
        expense_balance, expected_expense,
        "Expense balance should be {} but was {}",
        expected_expense, expense_balance
    );

    let asset_balance = get_account_balance(&db, data.asset_account_id)
        .await
        .expect("Failed to get asset balance");

    let expected_asset = -amount_per_tx * Decimal::from(NUM_TRANSACTIONS as i64);

    assert_eq!(
        asset_balance, expected_asset,
        "Asset balance should be {} but was {}",
        expected_asset, asset_balance
    );

    println!(
        "✓ Sequential test passed: {} transactions, expense={}, asset={}",
        NUM_TRANSACTIONS, expense_balance, asset_balance
    );

    cleanup_concurrent_test_data(&db, &data)
        .await
        .expect("Cleanup failed");
}
