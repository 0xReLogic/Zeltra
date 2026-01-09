//! Integration tests for database triggers.
//!
//! Tests Requirements 13.1-13.6 for database trigger verification.
//!
//! These tests verify that PostgreSQL triggers enforce data integrity
//! at the database level, even if application logic fails.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter, TransactionTrait,
};
use std::env;
use uuid::Uuid;

use zeltra_db::entities::{
    chart_of_accounts, fiscal_periods, fiscal_years, ledger_entries, organizations,
    organization_users, transactions, users,
    sea_orm_active_enums::{
        AccountSubtype, AccountType, FiscalPeriodStatus, TransactionStatus, TransactionType,
        UserRole,
    },
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

/// Helper to create test data for trigger tests.
struct TestData {
    org_id: Uuid,
    user_id: Uuid,
    fiscal_period_id: Uuid,
    asset_account_id: Uuid,
    expense_account_id: Uuid,
}


async fn setup_test_data(db: &DatabaseConnection) -> Result<TestData, sea_orm::DbErr> {
    let org_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let fiscal_year_id = Uuid::new_v4();
    let fiscal_period_id = Uuid::new_v4();
    let asset_account_id = Uuid::new_v4();
    let expense_account_id = Uuid::new_v4();

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

    // Create organization user (composite primary key: user_id + organization_id)
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
        name: Set("FY 2025".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create fiscal period (OPEN)
    fiscal_periods::ActiveModel {
        id: Set(fiscal_period_id),
        fiscal_year_id: Set(fiscal_year_id),
        period_number: Set(1),
        name: Set("January 2025".to_string()),
        start_date: Set(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(2025, 1, 31).unwrap()),
        status: Set(FiscalPeriodStatus::Open),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create asset account
    chart_of_accounts::ActiveModel {
        id: Set(asset_account_id),
        organization_id: Set(org_id),
        code: Set(format!("1000-{}", Uuid::new_v4().to_string()[..8].to_string())),
        name: Set("Cash".to_string()),
        account_type: Set(AccountType::Asset),
        account_subtype: Set(Some(AccountSubtype::Cash)),
        currency: Set("USD".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Create expense account
    chart_of_accounts::ActiveModel {
        id: Set(expense_account_id),
        organization_id: Set(org_id),
        code: Set(format!("5000-{}", Uuid::new_v4().to_string()[..8].to_string())),
        name: Set("Office Supplies".to_string()),
        account_type: Set(AccountType::Expense),
        account_subtype: Set(Some(AccountSubtype::OperatingExpense)),
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
    })
}

async fn cleanup_test_data(db: &DatabaseConnection, data: &TestData) -> Result<(), sea_orm::DbErr> {
    // Delete in reverse order of dependencies
    ledger_entries::Entity::delete_many()
        .filter(ledger_entries::Column::AccountId.is_in([data.asset_account_id, data.expense_account_id]))
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
    
    organizations::Entity::delete_by_id(data.org_id).exec(db).await?;
    users::Entity::delete_by_id(data.user_id).exec(db).await?;
    
    Ok(())
}

// ============================================================================
// Test: trg_update_account_balance sets version and balances (Requirement 13.3)
// ============================================================================
#[tokio::test]
async fn test_trigger_update_account_balance_sets_version_and_balances() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Create a transaction
    let tx_id = Uuid::new_v4();
    let tx = transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Test transaction".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    };
    tx.insert(&db).await.expect("Failed to create transaction");

    // Insert debit entry - trigger should set account_version and balances
    let debit_entry_id = Uuid::new_v4();
    let debit_entry = ledger_entries::ActiveModel {
        id: Set(debit_entry_id),
        transaction_id: Set(tx_id),
        account_id: Set(data.expense_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(10000, 2)), // 100.00
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(10000, 2)),
        debit: Set(Decimal::new(10000, 2)),
        credit: Set(Decimal::ZERO),
        // Note: account_version, previous_balance, current_balance should be set by trigger
        ..Default::default()
    };
    debit_entry.insert(&db).await.expect("Failed to create debit entry");

    // Insert credit entry
    let credit_entry_id = Uuid::new_v4();
    let credit_entry = ledger_entries::ActiveModel {
        id: Set(credit_entry_id),
        transaction_id: Set(tx_id),
        account_id: Set(data.asset_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(10000, 2)),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(10000, 2)),
        debit: Set(Decimal::ZERO),
        credit: Set(Decimal::new(10000, 2)),
        ..Default::default()
    };
    credit_entry.insert(&db).await.expect("Failed to create credit entry");

    // Verify the trigger set account_version
    let debit_result = ledger_entries::Entity::find_by_id(debit_entry_id)
        .one(&db)
        .await
        .expect("Failed to query debit entry");
    
    let debit = debit_result.expect("Debit entry not found");
    assert_eq!(debit.account_version, 1, "First entry should have version 1");
    assert_eq!(debit.account_previous_balance, Decimal::ZERO, "Previous balance should be 0");
    // Expense account: balance = debit - credit = 100 - 0 = 100
    assert_eq!(debit.account_current_balance, Decimal::new(10000, 2), "Current balance should be 100.00");

    let credit_result = ledger_entries::Entity::find_by_id(credit_entry_id)
        .one(&db)
        .await
        .expect("Failed to query credit entry");
    
    let credit = credit_result.expect("Credit entry not found");
    assert_eq!(credit.account_version, 1, "First entry on asset account should have version 1");
    assert_eq!(credit.account_previous_balance, Decimal::ZERO, "Previous balance should be 0");
    // Asset account: balance = debit - credit = 0 - 100 = -100
    assert_eq!(credit.account_current_balance, Decimal::new(-10000, 2), "Current balance should be -100.00");

    // Cleanup
    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_update_account_balance increments version for multiple entries
// ============================================================================
#[tokio::test]
async fn test_trigger_account_version_increments() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Create first transaction
    let tx1_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx1_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Transaction 1".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create transaction 1");

    // First entry on expense account
    let entry1_id = Uuid::new_v4();
    ledger_entries::ActiveModel {
        id: Set(entry1_id),
        transaction_id: Set(tx1_id),
        account_id: Set(data.expense_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(5000, 2)), // 50.00
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(5000, 2)),
        debit: Set(Decimal::new(5000, 2)),
        credit: Set(Decimal::ZERO),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create entry 1");

    // Credit entry for balance
    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx1_id),
        account_id: Set(data.asset_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(5000, 2)),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(5000, 2)),
        debit: Set(Decimal::ZERO),
        credit: Set(Decimal::new(5000, 2)),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create credit entry 1");

    // Create second transaction with another entry on same expense account
    let tx2_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx2_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 16).unwrap()),
        description: Set("Transaction 2".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create transaction 2");

    // Second entry on same expense account - version should be 2
    let entry2_id = Uuid::new_v4();
    ledger_entries::ActiveModel {
        id: Set(entry2_id),
        transaction_id: Set(tx2_id),
        account_id: Set(data.expense_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(3000, 2)), // 30.00
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(3000, 2)),
        debit: Set(Decimal::new(3000, 2)),
        credit: Set(Decimal::ZERO),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create entry 2");

    // Credit entry for balance
    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx2_id),
        account_id: Set(data.asset_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(3000, 2)),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(3000, 2)),
        debit: Set(Decimal::ZERO),
        credit: Set(Decimal::new(3000, 2)),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create credit entry 2");

    // Verify versions
    let e1 = ledger_entries::Entity::find_by_id(entry1_id)
        .one(&db)
        .await
        .expect("Query failed")
        .expect("Entry 1 not found");
    
    let e2 = ledger_entries::Entity::find_by_id(entry2_id)
        .one(&db)
        .await
        .expect("Query failed")
        .expect("Entry 2 not found");

    assert_eq!(e1.account_version, 1, "First entry should have version 1");
    assert_eq!(e2.account_version, 2, "Second entry should have version 2");
    
    // Verify running balance
    assert_eq!(e1.account_current_balance, Decimal::new(5000, 2), "First balance should be 50.00");
    assert_eq!(e2.account_previous_balance, Decimal::new(5000, 2), "Previous should be 50.00");
    assert_eq!(e2.account_current_balance, Decimal::new(8000, 2), "Current should be 80.00 (50+30)");

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_prevent_posted_mod rejects updates to posted transactions (Req 13.4)
// ============================================================================
#[tokio::test]
async fn test_trigger_prevent_posted_modification() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Create a posted transaction
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Posted transaction".to_string()),
        status: Set(TransactionStatus::Posted), // Already posted
        created_by: Set(data.user_id),
        posted_by: Set(Some(data.user_id)),
        posted_at: Set(Some(chrono::Utc::now().into())),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create posted transaction");

    // Try to update the posted transaction (should fail due to trigger)
    let update_result = transactions::Entity::update_many()
        .col_expr(transactions::Column::Description, sea_orm::sea_query::Expr::value("Modified"))
        .filter(transactions::Column::Id.eq(tx_id))
        .exec(&db)
        .await;

    // The trigger should reject this update
    assert!(
        update_result.is_err(),
        "Trigger should reject modification of posted transaction"
    );

    if let Err(e) = update_result {
        let err_msg = e.to_string().to_lowercase();
        assert!(
            err_msg.contains("cannot modify posted") || err_msg.contains("reversing entry"),
            "Error should mention posted transaction: {}",
            e
        );
    }

    // Cleanup - delete the transaction directly (bypass trigger for cleanup)
    transactions::Entity::delete_by_id(tx_id)
        .exec(&db)
        .await
        .expect("Cleanup failed");
    
    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_prevent_posted_mod rejects updates to voided transactions (Req 13.5)
// ============================================================================
#[tokio::test]
async fn test_trigger_prevent_voided_modification() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Create a voided transaction
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Voided transaction".to_string()),
        status: Set(TransactionStatus::Voided),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create voided transaction");

    // Try to update the voided transaction (should fail due to trigger)
    let update_result = transactions::Entity::update_many()
        .col_expr(transactions::Column::Description, sea_orm::sea_query::Expr::value("Modified"))
        .filter(transactions::Column::Id.eq(tx_id))
        .exec(&db)
        .await;

    // The trigger should reject this update
    assert!(
        update_result.is_err(),
        "Trigger should reject modification of voided transaction"
    );

    if let Err(e) = update_result {
        let err_msg = e.to_string().to_lowercase();
        assert!(
            err_msg.contains("cannot modify voided") || err_msg.contains("voided transaction"),
            "Error should mention voided transaction: {}",
            e
        );
    }

    // Cleanup
    transactions::Entity::delete_by_id(tx_id)
        .exec(&db)
        .await
        .expect("Cleanup failed");
    
    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_prevent_posted_mod allows voiding a posted transaction
// ============================================================================
#[tokio::test]
async fn test_trigger_allows_voiding_posted_transaction() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Create a posted transaction
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Posted transaction".to_string()),
        status: Set(TransactionStatus::Posted),
        created_by: Set(data.user_id),
        posted_by: Set(Some(data.user_id)),
        posted_at: Set(Some(chrono::Utc::now().into())),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create posted transaction");

    // Void the posted transaction (should succeed)
    let void_result = transactions::Entity::update_many()
        .col_expr(transactions::Column::Status, sea_orm::sea_query::Expr::value(TransactionStatus::Voided))
        .filter(transactions::Column::Id.eq(tx_id))
        .exec(&db)
        .await;

    assert!(
        void_result.is_ok(),
        "Trigger should allow voiding a posted transaction: {:?}",
        void_result.err()
    );

    // Verify it was voided
    let tx = transactions::Entity::find_by_id(tx_id)
        .one(&db)
        .await
        .expect("Query failed")
        .expect("Transaction not found");
    
    assert_eq!(tx.status, TransactionStatus::Voided, "Transaction should be voided");

    // Cleanup
    transactions::Entity::delete_by_id(tx_id)
        .exec(&db)
        .await
        .expect("Cleanup failed");
    
    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_validate_fiscal_period rejects posting to closed period (Req 13.6)
// ============================================================================
#[tokio::test]
async fn test_trigger_validate_fiscal_period_closed() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Close the fiscal period
    fiscal_periods::Entity::update_many()
        .col_expr(fiscal_periods::Column::Status, sea_orm::sea_query::Expr::value(FiscalPeriodStatus::Closed))
        .filter(fiscal_periods::Column::Id.eq(data.fiscal_period_id))
        .exec(&db)
        .await
        .expect("Failed to close fiscal period");

    // Create a draft transaction
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Draft transaction".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("Failed to create draft transaction");

    // Try to post to closed period (should fail due to trigger)
    let post_result = transactions::Entity::update_many()
        .col_expr(transactions::Column::Status, sea_orm::sea_query::Expr::value(TransactionStatus::Posted))
        .col_expr(transactions::Column::PostedBy, sea_orm::sea_query::Expr::value(data.user_id))
        .col_expr(transactions::Column::PostedAt, sea_orm::sea_query::Expr::value(chrono::Utc::now()))
        .filter(transactions::Column::Id.eq(tx_id))
        .exec(&db)
        .await;

    // The trigger should reject posting to closed period
    assert!(
        post_result.is_err(),
        "Trigger should reject posting to closed fiscal period"
    );

    if let Err(e) = post_result {
        let err_msg = e.to_string().to_lowercase();
        assert!(
            err_msg.contains("closed") || err_msg.contains("fiscal period"),
            "Error should mention closed period: {}",
            e
        );
    }

    // Cleanup
    transactions::Entity::delete_by_id(tx_id)
        .exec(&db)
        .await
        .expect("Cleanup failed");
    
    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_check_balance rejects unbalanced transactions (Req 13.1, 13.2)
// ============================================================================
#[tokio::test]
async fn test_trigger_check_balance_rejects_unbalanced() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Use a database transaction to test the deferred constraint trigger
    let txn = db.begin().await.expect("Failed to begin transaction");

    // Create a transaction
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Unbalanced transaction".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to create transaction");

    // Insert only a debit entry (unbalanced - no credit)
    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx_id),
        account_id: Set(data.expense_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(10000, 2)), // 100.00
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(10000, 2)),
        debit: Set(Decimal::new(10000, 2)),
        credit: Set(Decimal::ZERO),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to create debit entry");

    // Try to commit - the deferred constraint trigger should reject
    let commit_result = txn.commit().await;

    // The trigger should reject the unbalanced transaction at commit time
    assert!(
        commit_result.is_err(),
        "Trigger should reject unbalanced transaction on commit"
    );

    if let Err(e) = commit_result {
        let err_msg = e.to_string().to_lowercase();
        assert!(
            err_msg.contains("not balanced") || err_msg.contains("debit") || err_msg.contains("credit"),
            "Error should mention balance issue: {}",
            e
        );
    }

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}


// ============================================================================
// Test: trg_check_balance accepts balanced transactions
// ============================================================================
#[tokio::test]
async fn test_trigger_check_balance_accepts_balanced() {
    let db = match Database::connect(&get_database_url()).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping test - database not available: {}", e);
            return;
        }
    };

    let data = match setup_test_data(&db).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test - setup failed: {}", e);
            return;
        }
    };

    // Use a database transaction
    let txn = db.begin().await.expect("Failed to begin transaction");

    // Create a transaction
    let tx_id = Uuid::new_v4();
    transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(data.org_id),
        fiscal_period_id: Set(data.fiscal_period_id),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
        description: Set("Balanced transaction".to_string()),
        status: Set(TransactionStatus::Draft),
        created_by: Set(data.user_id),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to create transaction");

    // Insert balanced entries (debit = credit)
    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
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
    .insert(&txn)
    .await
    .expect("Failed to create debit entry");

    ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx_id),
        account_id: Set(data.asset_account_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(Decimal::new(10000, 2)),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(Decimal::new(10000, 2)),
        debit: Set(Decimal::ZERO),
        credit: Set(Decimal::new(10000, 2)),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .expect("Failed to create credit entry");

    // Commit should succeed for balanced transaction
    let commit_result = txn.commit().await;
    assert!(
        commit_result.is_ok(),
        "Balanced transaction should commit successfully: {:?}",
        commit_result.err()
    );

    cleanup_test_data(&db, &data).await.expect("Cleanup failed");
}
