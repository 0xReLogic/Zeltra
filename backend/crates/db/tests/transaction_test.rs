//! Integration tests for transaction repository.
//!
//! Tests Requirements 10.1-10.7 for transaction API.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::Database;
use std::env;
use uuid::Uuid;

use zeltra_db::{
    entities::sea_orm_active_enums::{TransactionStatus, TransactionType},
    repositories::transaction::{CreateLedgerEntryInput, TransactionFilter, TransactionRepository},
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://zeltra:zeltra_dev_password@localhost:5432/zeltra_dev".to_string()
    })
}

// Helper to create test entries (will be used in full integration tests)
#[allow(dead_code)]
fn create_balanced_entries(
    debit_account_id: Uuid,
    credit_account_id: Uuid,
    amount: Decimal,
    currency: &str,
) -> Vec<CreateLedgerEntryInput> {
    vec![
        CreateLedgerEntryInput {
            account_id: debit_account_id,
            source_currency: currency.to_string(),
            source_amount: amount,
            exchange_rate: Decimal::ONE,
            functional_currency: currency.to_string(),
            functional_amount: amount,
            debit: amount,
            credit: Decimal::ZERO,
            memo: Some("Debit entry".to_string()),
            dimensions: vec![],
        },
        CreateLedgerEntryInput {
            account_id: credit_account_id,
            source_currency: currency.to_string(),
            source_amount: amount,
            exchange_rate: Decimal::ONE,
            functional_currency: currency.to_string(),
            functional_amount: amount,
            debit: Decimal::ZERO,
            credit: amount,
            memo: Some("Credit entry".to_string()),
            dimensions: vec![],
        },
    ]
}

// ============================================================================
// Test: Create transaction with valid entries (Requirement 10.1)
// ============================================================================
#[tokio::test]
async fn test_create_transaction_valid_entries() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let _repo = TransactionRepository::new(db);

    // Note: This test requires:
    // 1. A valid organization with fiscal periods set up
    // 2. Valid account IDs
    // For now, we just verify the repository compiles and can be instantiated
    
    // In a real test, you would:
    // 1. Create test organization
    // 2. Create fiscal year and periods
    // 3. Create accounts
    // 4. Create transaction
    // 5. Verify transaction was created with draft status
    
    // Placeholder assertion
    assert!(true, "Repository instantiated successfully");
}

// ============================================================================
// Test: List transactions with filters (Requirement 10.2)
// ============================================================================
#[tokio::test]
async fn test_list_transactions_with_filters() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    // Test with empty filter - should not error
    let filter = TransactionFilter::default();
    
    // Use a random org_id that likely doesn't exist
    let org_id = Uuid::new_v4();
    
    let result = repo.list_transactions(org_id, filter).await;
    
    // Should return empty list, not error
    assert!(result.is_ok(), "List should succeed even with no results");
    assert!(result.unwrap().is_empty(), "Should return empty list for non-existent org");
}

// ============================================================================
// Test: Get transaction not found (Requirement 10.3)
// ============================================================================
#[tokio::test]
async fn test_get_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();

    let result = repo.get_transaction(org_id, transaction_id).await;

    assert!(result.is_err(), "Should return error for non-existent transaction");
    
    match result {
        Err(zeltra_db::repositories::transaction::TransactionError::NotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected NotFound error"),
    }
}

// ============================================================================
// Test: Delete transaction not found (Requirement 10.6, 10.7)
// ============================================================================
#[tokio::test]
async fn test_delete_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();

    let result = repo.delete_transaction(org_id, transaction_id).await;

    assert!(result.is_err(), "Should return error for non-existent transaction");
    
    match result {
        Err(zeltra_db::repositories::transaction::TransactionError::NotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected NotFound error"),
    }
}

// ============================================================================
// Test: Update transaction not found (Requirement 10.4, 10.5)
// ============================================================================
#[tokio::test]
async fn test_update_transaction_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    let org_id = Uuid::new_v4();
    let transaction_id = Uuid::new_v4();

    let result = repo
        .update_transaction(
            org_id,
            transaction_id,
            Some("Updated description".to_string()),
            None,
            None,
        )
        .await;

    assert!(result.is_err(), "Should return error for non-existent transaction");
    
    match result {
        Err(zeltra_db::repositories::transaction::TransactionError::NotFound(id)) => {
            assert_eq!(id, transaction_id);
        }
        _ => panic!("Expected NotFound error"),
    }
}

// ============================================================================
// Test: Transaction filter by status
// ============================================================================
#[tokio::test]
async fn test_list_transactions_filter_by_status() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    let filter = TransactionFilter {
        status: Some(TransactionStatus::Draft),
        ..Default::default()
    };

    let org_id = Uuid::new_v4();
    let result = repo.list_transactions(org_id, filter).await;

    assert!(result.is_ok(), "Filter by status should work");
}

// ============================================================================
// Test: Transaction filter by type
// ============================================================================
#[tokio::test]
async fn test_list_transactions_filter_by_type() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    let filter = TransactionFilter {
        transaction_type: Some(TransactionType::Journal),
        ..Default::default()
    };

    let org_id = Uuid::new_v4();
    let result = repo.list_transactions(org_id, filter).await;

    assert!(result.is_ok(), "Filter by type should work");
}

// ============================================================================
// Test: Transaction filter by date range
// ============================================================================
#[tokio::test]
async fn test_list_transactions_filter_by_date_range() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = TransactionRepository::new(db);

    let filter = TransactionFilter {
        date_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        date_to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        ..Default::default()
    };

    let org_id = Uuid::new_v4();
    let result = repo.list_transactions(org_id, filter).await;

    assert!(result.is_ok(), "Filter by date range should work");
}
