//! Integration tests for Organization repository.
//!
//! Tests CRUD operations including the update (PATCH) functionality.

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;
use zeltra_db::{
    OrganizationRepository,
    entities::{organizations, users},
};

/// Get database URL from environment or use default.
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

/// Create a test user for organization tests.
async fn create_test_user(db: &DatabaseConnection) -> Uuid {
    let user_id = Uuid::new_v4();
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Test User".to_string()),
        is_active: Set(true),
        ..Default::default()
    };
    user.insert(db).await.expect("Failed to create test user");
    user_id
}

/// Cleanup test organization.
async fn cleanup_org(db: &DatabaseConnection, org_id: Uuid) {
    organizations::Entity::delete_by_id(org_id)
        .exec(db)
        .await
        .ok();
}

#[tokio::test]
async fn test_organization_update_name() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Original Name",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update only name
    let updated = repo
        .update(org.id, Some("Updated Name"), None, None)
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.base_currency, "USD"); // unchanged
    assert_eq!(updated.timezone, "UTC"); // unchanged

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_currency() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update only currency
    let updated = repo
        .update(org.id, None, Some("IDR"), None)
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "Test Org"); // unchanged
    assert_eq!(updated.base_currency, "IDR");
    assert_eq!(updated.timezone, "UTC"); // unchanged

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_timezone() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update only timezone
    let updated = repo
        .update(org.id, None, None, Some("Asia/Jakarta"))
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "Test Org"); // unchanged
    assert_eq!(updated.base_currency, "USD"); // unchanged
    assert_eq!(updated.timezone, "Asia/Jakarta");

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_multiple_fields() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Original Name",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Update all fields
    let updated = repo
        .update(
            org.id,
            Some("New Company Name"),
            Some("EUR"),
            Some("Europe/London"),
        )
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    assert_eq!(updated.name, "New Company Name");
    assert_eq!(updated.base_currency, "EUR");
    assert_eq!(updated.timezone, "Europe/London");

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_organization_update_nonexistent() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = OrganizationRepository::new(db.clone());

    // Try to update non-existent organization
    let result = repo
        .update(Uuid::new_v4(), Some("New Name"), None, None)
        .await
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for non-existent org");
}

#[tokio::test]
async fn test_organization_update_preserves_other_fields() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    let original_slug = org.slug.clone();
    let original_created_at = org.created_at;

    // Update name
    let updated = repo
        .update(org.id, Some("Updated Name"), None, None)
        .await
        .expect("Failed to update organization")
        .expect("Organization should exist");

    // Verify other fields are preserved
    assert_eq!(updated.slug, original_slug, "Slug should not change");
    assert_eq!(
        updated.created_at, original_created_at,
        "created_at should not change"
    );
    assert!(
        updated.updated_at > original_created_at,
        "updated_at should be newer"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

// ============================================================================
// Integration Tests for update_organization() with validation
// ============================================================================

use zeltra_db::entities::sea_orm_active_enums::UserRole;
use zeltra_db::repositories::organization::OrganizationError;

#[tokio::test]
async fn test_update_organization_happy_path() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Original Name",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    let original_updated_at = org.updated_at;

    // Update using new method with validation
    let updated = repo
        .update_organization(org.id, Some("New Name"), None, Some("Asia/Jakarta"))
        .await
        .expect("Failed to update organization");

    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.base_currency, "USD"); // unchanged
    assert_eq!(updated.timezone, "Asia/Jakarta");
    assert!(
        updated.updated_at > original_updated_at,
        "updated_at should be newer"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_update_organization_empty_update_rejected() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Try to update with no fields
    let result = repo.update_organization(org.id, None, None, None).await;

    assert!(
        matches!(result, Err(OrganizationError::EmptyUpdate)),
        "Should reject empty update"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_update_organization_invalid_name_rejected() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Try empty name
    let result = repo.update_organization(org.id, Some(""), None, None).await;
    assert!(
        matches!(result, Err(OrganizationError::InvalidName)),
        "Should reject empty name"
    );

    // Try name > 255 chars
    let long_name = "x".repeat(256);
    let result = repo
        .update_organization(org.id, Some(&long_name), None, None)
        .await;
    assert!(
        matches!(result, Err(OrganizationError::InvalidName)),
        "Should reject name > 255 chars"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_update_organization_invalid_currency_rejected() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Try invalid currency
    let result = repo
        .update_organization(org.id, None, Some("XXX"), None)
        .await;
    assert!(
        matches!(result, Err(OrganizationError::InvalidCurrency(_))),
        "Should reject invalid currency"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_update_organization_invalid_timezone_rejected() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    // Try invalid timezone
    let result = repo
        .update_organization(org.id, None, None, Some("Invalid/Timezone"))
        .await;
    assert!(
        matches!(result, Err(OrganizationError::InvalidTimezone(_))),
        "Should reject invalid timezone"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_update_organization_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let repo = OrganizationRepository::new(db.clone());

    // Try to update non-existent organization
    let result = repo
        .update_organization(Uuid::new_v4(), Some("New Name"), None, None)
        .await;

    assert!(
        matches!(result, Err(OrganizationError::NotFound)),
        "Should return NotFound for non-existent org"
    );
}

// ============================================================================
// Integration Tests for remove_member()
// ============================================================================

#[tokio::test]
async fn test_remove_member_happy_path() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let member_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization with owner
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Add a member
    repo.add_user(org.id, member_id, UserRole::Viewer, None)
        .await
        .expect("Failed to add member");

    // Verify member exists
    assert!(repo.is_member(org.id, member_id).await.unwrap());

    // Remove member as owner
    repo.remove_member(org.id, member_id, &UserRole::Owner)
        .await
        .expect("Failed to remove member");

    // Verify member is removed
    assert!(!repo.is_member(org.id, member_id).await.unwrap());

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_remove_member_not_member() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let non_member_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Try to remove non-member
    let result = repo
        .remove_member(org.id, non_member_id, &UserRole::Owner)
        .await;

    assert!(
        matches!(result, Err(OrganizationError::NotMember)),
        "Should return NotMember error"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_remove_member_admin_cannot_remove_owner() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let second_owner_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization with owner
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Add second owner
    repo.add_user(org.id, second_owner_id, UserRole::Owner, None)
        .await
        .expect("Failed to add second owner");

    // Try to remove owner as admin (should fail)
    let result = repo.remove_member(org.id, owner_id, &UserRole::Admin).await;

    assert!(
        matches!(result, Err(OrganizationError::Forbidden)),
        "Admin should not be able to remove owner"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_remove_member_last_owner_protected() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization with single owner
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Try to remove the only owner (should fail)
    let result = repo.remove_member(org.id, owner_id, &UserRole::Owner).await;

    assert!(
        matches!(result, Err(OrganizationError::LastOwner)),
        "Should not be able to remove last owner"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_remove_member_owner_can_remove_owner() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let second_owner_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization with owner
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Add second owner
    repo.add_user(org.id, second_owner_id, UserRole::Owner, None)
        .await
        .expect("Failed to add second owner");

    // Owner can remove another owner (not last)
    repo.remove_member(org.id, second_owner_id, &UserRole::Owner)
        .await
        .expect("Owner should be able to remove another owner");

    // Verify second owner is removed
    assert!(!repo.is_member(org.id, second_owner_id).await.unwrap());

    // Cleanup
    cleanup_org(&db, org.id).await;
}

#[tokio::test]
async fn test_remove_member_org_not_found() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Try to remove from non-existent org
    let result = repo
        .remove_member(Uuid::new_v4(), user_id, &UserRole::Owner)
        .await;

    assert!(
        matches!(result, Err(OrganizationError::NotFound)),
        "Should return NotFound for non-existent org"
    );
}

// ============================================================================
// Integration Tests for timestamp update (Property 8)
// ============================================================================

#[tokio::test]
async fn test_update_organization_timestamp_changes() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let user_id = create_test_user(&db).await;
    let repo = OrganizationRepository::new(db.clone());

    // Create organization
    let org = repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            user_id,
        )
        .await
        .expect("Failed to create organization");

    let original_updated_at = org.updated_at;

    // Small delay to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Update organization
    let updated = repo
        .update_organization(org.id, Some("New Name"), None, None)
        .await
        .expect("Failed to update organization");

    // Property 8: Update Timestamp Consistency
    // updated_at should be greater than its previous value
    assert!(
        updated.updated_at > original_updated_at,
        "updated_at should increase after update"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

// ============================================================================
// Integration Tests for session revocation (Property 5)
// ============================================================================

use zeltra_db::SessionRepository;

#[tokio::test]
async fn test_session_revocation_on_user_removal() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let member_id = create_test_user(&db).await;
    let org_repo = OrganizationRepository::new(db.clone());
    let session_repo = SessionRepository::new(db.clone());

    // Create organization
    let org = org_repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Add member
    org_repo
        .add_user(org.id, member_id, UserRole::Viewer, None)
        .await
        .expect("Failed to add member");

    // Create session for member
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
    let _session = session_repo
        .create(
            member_id,
            org.id,
            "test-refresh-token",
            expires_at,
            None,
            None,
        )
        .await
        .expect("Failed to create session");

    // Verify session is active
    let active_count = session_repo
        .count_active_sessions(member_id)
        .await
        .expect("Failed to count sessions");
    assert!(active_count > 0, "Should have active session");

    // Remove member
    org_repo
        .remove_member(org.id, member_id, &UserRole::Owner)
        .await
        .expect("Failed to remove member");

    // Revoke sessions (this is what the API route does)
    let revoked = session_repo
        .revoke_user_org_sessions(member_id, org.id)
        .await
        .expect("Failed to revoke sessions");

    // Property 5: Session Revocation on Removal
    // All active sessions for that user in that organization SHALL be revoked
    assert!(revoked > 0, "Should have revoked at least one session");

    // Verify session is revoked by checking it can't be found by token
    let found = session_repo
        .find_by_token("test-refresh-token")
        .await
        .expect("Failed to find session");
    assert!(
        found.is_none(),
        "Session should be revoked and not findable"
    );

    // Cleanup
    cleanup_org(&db, org.id).await;
}

// ============================================================================
// Integration Tests for data preservation (Property 6)
// ============================================================================

use chrono::NaiveDate;
use rust_decimal_macros::dec;
use zeltra_db::entities::sea_orm_active_enums::{AccountSubtype, AccountType, TransactionType};
use zeltra_db::repositories::account::{AccountRepository, CreateAccountInput};
use zeltra_db::repositories::fiscal::{CreateFiscalYearInput, FiscalRepository};
use zeltra_db::repositories::transaction::{
    CreateLedgerEntryInput, CreateTransactionInput, TransactionRepository,
};

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_data_preservation_on_user_removal() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner_id = create_test_user(&db).await;
    let member_id = create_test_user(&db).await;
    let org_repo = OrganizationRepository::new(db.clone());
    let fiscal_repo = FiscalRepository::new(db.clone());
    let account_repo = AccountRepository::new(db.clone());
    let tx_repo = TransactionRepository::new(db.clone());

    // Create organization
    let org = org_repo
        .create_with_owner(
            "Test Org",
            &format!("test-org-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner_id,
        )
        .await
        .expect("Failed to create organization");

    // Add member
    org_repo
        .add_user(org.id, member_id, UserRole::Accountant, None)
        .await
        .expect("Failed to add member");

    // Create fiscal year (needed for transaction to have a valid fiscal period)
    let _fiscal_year = fiscal_repo
        .create_fiscal_year(CreateFiscalYearInput {
            organization_id: org.id,
            name: "FY 2026".to_string(),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        })
        .await
        .expect("Failed to create fiscal year");

    // Create accounts
    let cash_account = account_repo
        .create_account(CreateAccountInput {
            organization_id: org.id,
            code: format!(
                "1000-{}",
                Uuid::new_v4().to_string().split('-').next().unwrap()
            ),
            name: "Cash".to_string(),
            account_type: AccountType::Asset,
            account_subtype: Some(AccountSubtype::Cash),
            currency: "USD".to_string(),
            parent_id: None,
            description: None,
            is_active: true,
            allow_direct_posting: true,
        })
        .await
        .expect("Failed to create cash account");

    let expense_account = account_repo
        .create_account(CreateAccountInput {
            organization_id: org.id,
            code: format!(
                "5000-{}",
                Uuid::new_v4().to_string().split('-').next().unwrap()
            ),
            name: "Office Supplies".to_string(),
            account_type: AccountType::Expense,
            account_subtype: Some(AccountSubtype::OperatingExpense),
            currency: "USD".to_string(),
            parent_id: None,
            description: None,
            is_active: true,
            allow_direct_posting: true,
        })
        .await
        .expect("Failed to create expense account");

    // Create transaction by member
    let tx = tx_repo
        .create_transaction(CreateTransactionInput {
            organization_id: org.id,
            transaction_type: TransactionType::Expense,
            transaction_date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            description: "Office supplies purchase".to_string(),
            reference_number: Some("EXP-001".to_string()),
            memo: None,
            created_by: member_id, // Created by member
            entries: vec![
                CreateLedgerEntryInput {
                    account_id: expense_account.id,
                    source_currency: "USD".to_string(),
                    source_amount: dec!(100.00),
                    exchange_rate: dec!(1.0),
                    functional_currency: "USD".to_string(),
                    functional_amount: dec!(100.00),
                    debit: dec!(100.00),
                    credit: dec!(0),
                    memo: None,
                    dimensions: vec![],
                },
                CreateLedgerEntryInput {
                    account_id: cash_account.id,
                    source_currency: "USD".to_string(),
                    source_amount: dec!(100.00),
                    exchange_rate: dec!(1.0),
                    functional_currency: "USD".to_string(),
                    functional_amount: dec!(100.00),
                    debit: dec!(0),
                    credit: dec!(100.00),
                    memo: None,
                    dimensions: vec![],
                },
            ],
        })
        .await
        .expect("Failed to create transaction");

    let tx_id = tx.transaction.id;

    // Remove member
    org_repo
        .remove_member(org.id, member_id, &UserRole::Owner)
        .await
        .expect("Failed to remove member");

    // Property 6: Data Preservation on Removal
    // All transactions and ledger entries created by that user SHALL remain unchanged
    let found_tx = tx_repo
        .get_transaction(org.id, tx_id)
        .await
        .expect("Failed to get transaction");

    assert_eq!(
        found_tx.transaction.id, tx_id,
        "Transaction should still exist"
    );
    assert_eq!(
        found_tx.transaction.created_by, member_id,
        "created_by should still reference removed user"
    );
    assert_eq!(found_tx.entries.len(), 2, "Entries should still exist");

    // Cleanup
    cleanup_org(&db, org.id).await;
}

// ============================================================================
// Integration Tests for cross-tenant isolation (Property 7)
// ============================================================================

#[tokio::test]
async fn test_cross_tenant_isolation() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let owner1_id = create_test_user(&db).await;
    let owner2_id = create_test_user(&db).await;
    let org_repo = OrganizationRepository::new(db.clone());

    // Create two organizations
    let org1 = org_repo
        .create_with_owner(
            "Org 1",
            &format!("test-org1-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner1_id,
        )
        .await
        .expect("Failed to create org 1");

    let org2 = org_repo
        .create_with_owner(
            "Org 2",
            &format!("test-org2-{}", Uuid::new_v4()),
            "USD",
            "UTC",
            owner2_id,
        )
        .await
        .expect("Failed to create org 2");

    // Property 7: Cross-Tenant Isolation
    // User from org1 should not be member of org2
    let is_member = org_repo
        .is_member(org2.id, owner1_id)
        .await
        .expect("Failed to check membership");
    assert!(!is_member, "Owner1 should not be member of Org2");

    // User from org2 should not be member of org1
    let is_member = org_repo
        .is_member(org1.id, owner2_id)
        .await
        .expect("Failed to check membership");
    assert!(!is_member, "Owner2 should not be member of Org1");

    // Try to remove user from org they don't belong to
    let result = org_repo
        .remove_member(org2.id, owner1_id, &UserRole::Owner)
        .await;
    assert!(
        matches!(result, Err(OrganizationError::NotMember)),
        "Should not be able to remove user from org they don't belong to"
    );

    // Cleanup
    cleanup_org(&db, org1.id).await;
    cleanup_org(&db, org2.id).await;
}
