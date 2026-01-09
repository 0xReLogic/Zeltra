//! Integration tests for ExchangeRateRepository.
//!
//! Tests actual database operations for exchange rate management.
//! **Validates: Requirements 4.1-4.8**

use chrono::NaiveDate;
use rust_decimal_macros::dec;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use uuid::Uuid;
use zeltra_db::{
    entities::{organizations, sea_orm_active_enums::RateSource},
    repositories::{
        CreateExchangeRateInput, ExchangeRateError, ExchangeRateRepository, RateLookupMethod,
    },
};

/// Get database URL from environment or use default.
fn get_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string())
}

/// Create a test organization.
async fn create_test_org(db: &DatabaseConnection) -> Uuid {
    let org_id = Uuid::new_v4();
    let org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Exchange Rate Test Org {}", Uuid::new_v4())),
        slug: Set(format!("exrate-test-{}", Uuid::new_v4())),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        ..Default::default()
    };
    org.insert(db).await.expect("Failed to create test org");
    org_id
}

// ============================================================================
// Test 1: Create exchange rate (Requirement 4.1)
// ============================================================================
#[tokio::test]
async fn test_create_exchange_rate() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let org_id = create_test_org(&db).await;
    let repo = ExchangeRateRepository::new(db.clone());

    let input = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "EUR".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(1.10),
        effective_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        source: RateSource::Manual,
        source_reference: Some("Test".to_string()),
        created_by: None,
    };

    let result = repo.create_or_update_rate(input).await;
    assert!(result.is_ok(), "Should create exchange rate");

    let rate = result.unwrap();
    assert_eq!(rate.from_currency, "EUR");
    assert_eq!(rate.to_currency, "USD");
    assert_eq!(rate.rate, dec!(1.10));
}

// ============================================================================
// Test 2: Upsert behavior - update existing rate (Requirement 4.5)
// ============================================================================
#[tokio::test]
async fn test_upsert_exchange_rate() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let org_id = create_test_org(&db).await;
    let repo = ExchangeRateRepository::new(db.clone());
    let date = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

    // Create initial rate
    let input1 = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "GBP".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(1.25),
        effective_date: date,
        source: RateSource::Manual,
        source_reference: None,
        created_by: None,
    };
    let rate1 = repo.create_or_update_rate(input1).await.unwrap();

    // Update same currency pair and date with new rate
    let input2 = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "GBP".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(1.30),
        effective_date: date,
        source: RateSource::Api,
        source_reference: Some("Updated".to_string()),
        created_by: None,
    };
    let rate2 = repo.create_or_update_rate(input2).await.unwrap();

    // Should be same record (same ID), but updated rate
    assert_eq!(rate1.id, rate2.id, "Should update existing record");
    assert_eq!(rate2.rate, dec!(1.30), "Rate should be updated");
    assert_eq!(rate2.source, RateSource::Api, "Source should be updated");
}

// ============================================================================
// Test 3: Validation - rate must be positive (Requirement 4.2)
// ============================================================================
#[tokio::test]
async fn test_reject_non_positive_rate() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let org_id = create_test_org(&db).await;
    let repo = ExchangeRateRepository::new(db.clone());

    // Zero rate
    let input_zero = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "EUR".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(0),
        effective_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        source: RateSource::Manual,
        source_reference: None,
        created_by: None,
    };
    let result = repo.create_or_update_rate(input_zero).await;
    assert!(matches!(result, Err(ExchangeRateError::NonPositiveRate)));

    // Negative rate
    let input_neg = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "EUR".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(-1.5),
        effective_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        source: RateSource::Manual,
        source_reference: None,
        created_by: None,
    };
    let result = repo.create_or_update_rate(input_neg).await;
    assert!(matches!(result, Err(ExchangeRateError::NonPositiveRate)));
}

// ============================================================================
// Test 4: Validation - currencies must be different (Requirement 4.3)
// ============================================================================
#[tokio::test]
async fn test_reject_same_currency() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let org_id = create_test_org(&db).await;
    let repo = ExchangeRateRepository::new(db.clone());

    let input = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "USD".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(1.0),
        effective_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        source: RateSource::Manual,
        source_reference: None,
        created_by: None,
    };

    let result = repo.create_or_update_rate(input).await;
    assert!(matches!(result, Err(ExchangeRateError::SameCurrency)));
}

// ============================================================================
// Test 5: Find rate - direct lookup (Requirement 4.6)
// ============================================================================
#[tokio::test]
async fn test_find_direct_rate() {
    let db = Database::connect(&get_database_url())
        .await
        .expect("Failed to connect to database");

    let org_id = create_test_org(&db).await;
    let repo = ExchangeRateRepository::new(db.clone());

    // Create rate
    let input = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: "EUR".to_string(),
        to_currency: "USD".to_string(),
        rate: dec!(1.12),
        effective_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
        source: RateSource::Manual,
        source_reference: None,
        created_by: None,
    };
    repo.create_or_update_rate(input).await.unwrap();

    // Find rate on same date
    let lookup = repo
        .find_rate(org_id, "EUR", "USD", NaiveDate::from_ymd_opt(2025, 1, 10).unwrap())
        .await
        .unwrap();

    assert_eq!(lookup.rate, dec!(1.12));
    assert_eq!(lookup.lookup_method, RateLookupMethod::Direct);

    // Find rate on later date (should use most recent)
    let lookup2 = repo
        .find_rate(org_id, "EUR", "USD", NaiveDate::from_ymd_opt(2025, 1, 20).unwrap())
        .await
        .unwrap();

    assert_eq!(lookup2.rate, dec!(1.12));
    assert_eq!(lookup2.lookup_method, RateLookupMethod::Direct);
}

