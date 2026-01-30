//! Property-based tests for Report Repository.
//!
//! Feature: entities-model-implementation
//!
//! Tests universal correctness properties:
//! - Property 17: Report entity filtering
//! - Property 18: Consolidated report generation

use chrono::NaiveDate;
use proptest::prelude::*;
use rust_decimal::Decimal;
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set, TryIntoModel};
use std::env;
use tokio::runtime::Runtime;
use uuid::Uuid;
use zeltra_db::{
    entities::{
        chart_of_accounts, entities, ledger_entries, organization_users, organizations,
        transactions,
        sea_orm_active_enums::{
            AccountType, SubscriptionTier, TransactionStatus, TransactionType, UserRole,
        },
        users,
    },
    repositories::report::ReportRepository,
};

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

/// Helper to create a test user with organization
async fn setup_user_and_org(db: &DatabaseConnection) -> (Uuid, Uuid) {
    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();

    // Create user
    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        subscription_tier: Set(SubscriptionTier::Enterprise),
        subscription_status: Set(
            zeltra_db::entities::sea_orm_active_enums::SubscriptionStatus::Active,
        ),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    users::Entity::insert(user)
        .exec(db)
        .await
        .expect("Failed to insert user");

    // Create organization
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        base_currency: Set("USD".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    organizations::Entity::insert(org)
        .exec(db)
        .await
        .expect("Failed to insert org");

    // Link user as owner
    let org_user = organization_users::ActiveModel {
        user_id: Set(user_id),
        organization_id: Set(org_id),
        role: Set(UserRole::Owner),
        approval_limit: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    organization_users::Entity::insert(org_user)
        .exec(db)
        .await
        .expect("Failed to insert org_user");

    (user_id, org_id)
}

/// Helper to create an entity
async fn create_entity(
    db: &DatabaseConnection,
    org_id: Uuid,
    name: &str,
) -> entities::Model {
    let entity = entities::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        name: Set(name.to_string()),
        legal_name: Set(Some(name.to_string())),
        tax_id: Set(None),
        entity_type: Set("subsidiary".to_string()),
        base_currency: Set("USD".to_string()),
        is_active: Set(true),
        settings: Set(serde_json::json!({})),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    entities::Entity::insert(entity.clone())
        .exec(db)
        .await
        .expect("Failed to insert entity");
    
    entity.try_into_model().unwrap()
}

/// Helper to create an account
async fn create_account(
    db: &DatabaseConnection,
    org_id: Uuid,
    entity_id: Uuid,
    code: &str,
    account_type: AccountType,
) -> chart_of_accounts::Model {
    let account = chart_of_accounts::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        entity_id: Set(entity_id),
        code: Set(code.to_string()),
        name: Set(format!("Account {}", code)),
        description: Set(None),
        account_type: Set(account_type),
        account_subtype: Set(None),
        parent_id: Set(None),
        currency: Set("USD".to_string()),
        is_active: Set(true),
        is_system_account: Set(false),
        allow_direct_posting: Set(true),
        is_bank_account: Set(false),
        bank_account_number: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    chart_of_accounts::Entity::insert(account.clone())
        .exec(db)
        .await
        .expect("Failed to insert account");
    
    account.try_into_model().unwrap()
}

/// Helper to create a transaction with entries
async fn create_transaction_with_entries(
    db: &DatabaseConnection,
    org_id: Uuid,
    entity_id: Uuid,
    debit_account_id: Uuid,
    credit_account_id: Uuid,
    amount: Decimal,
    date: NaiveDate,
) -> transactions::Model {
    let tx_id = Uuid::new_v4();
    
    // Create transaction
    let transaction = transactions::ActiveModel {
        id: Set(tx_id),
        organization_id: Set(org_id),
        entity_id: Set(Some(entity_id)),
        transaction_type: Set(TransactionType::Journal),
        transaction_date: Set(date),
        description: Set("Test transaction".to_string()),
        reference_number: Set(Some(format!("REF-{}", tx_id))),
        status: Set(TransactionStatus::Posted),
        memo: Set(None),
        created_by: Set(Uuid::nil()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    transactions::Entity::insert(transaction.clone())
        .exec(db)
        .await
        .expect("Failed to insert transaction");
    
    // Create debit entry
    let debit_entry = ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx_id),
        account_id: Set(debit_account_id),
        entity_id: Set(entity_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(amount),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(amount),
        debit: Set(amount),
        credit: Set(Decimal::ZERO),
        account_current_balance: Set(amount),
        memo: Set(None),
        compliance_metadata: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    ledger_entries::Entity::insert(debit_entry)
        .exec(db)
        .await
        .expect("Failed to insert debit entry");
    
    // Create credit entry
    let credit_entry = ledger_entries::ActiveModel {
        id: Set(Uuid::new_v4()),
        transaction_id: Set(tx_id),
        account_id: Set(credit_account_id),
        entity_id: Set(entity_id),
        source_currency: Set("USD".to_string()),
        source_amount: Set(amount),
        exchange_rate: Set(Decimal::ONE),
        functional_currency: Set("USD".to_string()),
        functional_amount: Set(amount),
        debit: Set(Decimal::ZERO),
        credit: Set(amount),
        account_current_balance: Set(-amount),
        memo: Set(None),
        compliance_metadata: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    ledger_entries::Entity::insert(credit_entry)
        .exec(db)
        .await
        .expect("Failed to insert credit entry");
    
    transaction.try_into_model().unwrap()
}

/// Helper: Run async code in a temporary runtime
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(future)
}

proptest! {
    // Limit cases for DB integration tests to avoid timeouts
    #![proptest_config(proptest::test_runner::Config::with_cases(10))]

    /// Property 17: Report entity filtering
    /// Feature: entities-model-implementation, Property 17: Report entity filtering
    ///
    /// When generating a financial report for a specific entity, the report should include
    /// only data where entity_id equals the selected entity.
    #[test]
    fn prop_report_entity_filtering(
        entity_count in 2usize..5usize,
        tx_per_entity in 1usize..3usize
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_and_org(&db).await;
            
            // Create multiple entities
            let mut entities_list = Vec::new();
            let mut accounts_per_entity = Vec::new();
            
            for i in 0..entity_count {
                let entity = create_entity(&db, org_id, &format!("Entity {}", i)).await;
                
                // Create accounts for this entity
                let debit_account = create_account(
                    &db,
                    org_id,
                    entity.id,
                    &format!("{}100", i + 1),
                    AccountType::Asset,
                ).await;
                
                let credit_account = create_account(
                    &db,
                    org_id,
                    entity.id,
                    &format!("{}200", i + 1),
                    AccountType::Liability,
                ).await;
                
                entities_list.push(entity);
                accounts_per_entity.push((debit_account, credit_account));
            }
            
            // Create transactions for each entity
            let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
            for i in 0..entity_count {
                for j in 0..tx_per_entity {
                    let amount = Decimal::from((i + 1) * 100 + j * 10);
                    create_transaction_with_entries(
                        &db,
                        org_id,
                        entities_list[i].id,
                        accounts_per_entity[i].0.id,
                        accounts_per_entity[i].1.id,
                        amount,
                        date,
                    ).await;
                }
            }
            
            // Generate balance sheet for each entity
            let repo = ReportRepository::new(db.clone());
            for i in 0..entity_count {
                let balances = repo.query_balance_sheet(
                    org_id,
                    date,
                    Some(entities_list[i].id),
                ).await.unwrap();
                
                // Verify all accounts belong to the selected entity
                for balance in &balances {
                    let account = chart_of_accounts::Entity::find_by_id(balance.account_id)
                        .one(&db)
                        .await
                        .unwrap()
                        .unwrap();
                    
                    assert_eq!(
                        account.entity_id,
                        entities_list[i].id,
                        "Account should belong to entity {}",
                        i
                    );
                }
                
                // Verify we have exactly 2 accounts (debit and credit) for this entity
                assert_eq!(
                    balances.len(),
                    2,
                    "Should have exactly 2 accounts for entity {}",
                    i
                );
            }
        });
    }

    /// Property 18: Consolidated report generation
    /// Feature: entities-model-implementation, Property 18: Consolidated report generation
    ///
    /// When generating a consolidated report, the report should combine data from all entities
    /// in the organization and eliminate intercompany transactions based on intercompany mappings.
    ///
    /// Note: This property tests the basic consolidation (combining data from all entities).
    /// Intercompany elimination is tested separately in integration tests.
    #[test]
    fn prop_consolidated_report_generation(
        entity_count in 2usize..4usize,
        amount_per_entity in 100u32..500u32
    ) {
        block_on(async {
            let db = Database::connect(&get_database_url()).await.unwrap();
            let (_user_id, org_id) = setup_user_and_org(&db).await;
            
            // Create multiple entities
            let mut entities_list = Vec::new();
            let mut total_expected_debit = Decimal::ZERO;
            let mut total_expected_credit = Decimal::ZERO;
            
            for i in 0..entity_count {
                let entity = create_entity(&db, org_id, &format!("Entity {}", i)).await;
                
                // Create accounts for this entity
                let debit_account = create_account(
                    &db,
                    org_id,
                    entity.id,
                    &format!("{}100", i + 1),
                    AccountType::Asset,
                ).await;
                
                let credit_account = create_account(
                    &db,
                    org_id,
                    entity.id,
                    &format!("{}200", i + 1),
                    AccountType::Liability,
                ).await;
                
                // Create transaction for this entity
                let amount = Decimal::from(amount_per_entity + (i as u32 * 50));
                let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
                
                create_transaction_with_entries(
                    &db,
                    org_id,
                    entity.id,
                    debit_account.id,
                    credit_account.id,
                    amount,
                    date,
                ).await;
                
                total_expected_debit += amount;
                total_expected_credit += amount;
                
                entities_list.push(entity);
            }
            
            // Generate consolidated balance sheet (no entity filter)
            let repo = ReportRepository::new(db.clone());
            let balances = repo.query_balance_sheet(
                org_id,
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                None, // No entity filter = consolidated
            ).await.unwrap();
            
            // Verify we have accounts from all entities
            assert_eq!(
                balances.len(),
                entity_count * 2,
                "Consolidated report should include accounts from all {} entities",
                entity_count
            );
            
            // Calculate total debits and credits
            let total_debit: Decimal = balances.iter()
                .filter(|b| b.account_type == AccountType::Asset)
                .map(|b| b.total_debit)
                .sum();
            
            let total_credit: Decimal = balances.iter()
                .filter(|b| b.account_type == AccountType::Liability)
                .map(|b| b.total_credit)
                .sum();
            
            // Verify totals match expected
            assert_eq!(
                total_debit,
                total_expected_debit,
                "Total debits should match sum of all entities"
            );
            
            assert_eq!(
                total_credit,
                total_expected_credit,
                "Total credits should match sum of all entities"
            );
            
            // Verify balance sheet balances (Assets = Liabilities)
            let total_assets: Decimal = balances.iter()
                .filter(|b| b.account_type == AccountType::Asset)
                .map(|b| b.balance)
                .sum();
            
            let total_liabilities: Decimal = balances.iter()
                .filter(|b| b.account_type == AccountType::Liability)
                .map(|b| b.balance)
                .sum();
            
            assert_eq!(
                total_assets,
                total_liabilities,
                "Consolidated balance sheet should balance: Assets = Liabilities"
            );
        });
    }
}
