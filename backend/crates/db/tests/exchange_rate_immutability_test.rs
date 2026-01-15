use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use sea_orm::{
    ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set,
};
use uuid::Uuid;
use zeltra_db::entities::{
    currencies, fiscal_periods, fiscal_years,
    sea_orm_active_enums::{FiscalPeriodStatus, FiscalYearStatus, RateSource},
    users, organizations,
};
use zeltra_db::repositories::{
    exchange_rate::{CreateExchangeRateInput, ExchangeRateRepository},
    organization::OrganizationRepository,
};

// ============================================================================
// Helper Functions (Inlined)
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
        email: Set(format!("rate-test-{}@example.com", Uuid::new_v4())),
        password_hash: Set("$argon2id$test".to_string()),
        full_name: Set("Rate Test User".to_string()),
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
        "Rate Test Org",
        &format!("rate-org-{}", Uuid::new_v4()),
        "USD",
        "UTC",
        owner_id,
    )
    .await
    .expect("Failed to create organization")
}

async fn ensure_currency(db: &DatabaseConnection, code: &str) {
    let currency = currencies::ActiveModel {
        code: Set(code.to_string()),
        name: Set(format!("Test {}", code)),
        symbol: Set("$".to_string()),
        decimal_places: Set(2),
        is_active: Set(true),
    };
    // Ignore conflict if exists
    let _ = currency.insert(db).await.ok();
}

async fn create_closed_fiscal_period(db: &DatabaseConnection, org_id: Uuid, year: i32, month: u32) {
    let fy_id = Uuid::new_v4();
    let fy = fiscal_years::ActiveModel {
        id: Set(fy_id),
        organization_id: Set(org_id),
        name: Set(format!("FY {}", year)),
        start_date: Set(NaiveDate::from_ymd_opt(year, 1, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(year, 12, 31).unwrap()),
        status: Set(FiscalYearStatus::Open),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    fy.insert(db).await.expect("Failed to create fiscal year");

    let period = fiscal_periods::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        fiscal_year_id: Set(fy_id),
        name: Set(format!("Period {}", month)),
        period_number: Set(month as i16),
        is_adjustment_period: Set(false),
        start_date: Set(NaiveDate::from_ymd_opt(year, month, 1).unwrap()),
        end_date: Set(NaiveDate::from_ymd_opt(year, month, 28).unwrap()), // Simplification
        status: Set(FiscalPeriodStatus::Closed), // <--- CLOSED
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    period.insert(db).await.expect("Failed to create closed fiscal period");
}

// ============================================================================
// Test Case
// ============================================================================

#[tokio::test]
async fn test_cannot_update_rate_in_closed_period() {
    let db = setup_db().await;
    let repo = ExchangeRateRepository::new(db.clone());

    let user = create_user(&db).await;
    let org = create_org(&db, user.id).await;
    ensure_currency(&db, "EUR").await;

    // 1. Create a Closed Fiscal Period for Jan 2025
    create_closed_fiscal_period(&db, org.id, 2025, 1).await;

    let effective_date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

    // 2. Attempt to Create/Update Rate in Closed Period
    let input = CreateExchangeRateInput {
        organization_id: org.id,
        from_currency: "USD".to_string(),
        to_currency: "EUR".to_string(),
        rate: dec!(0.85),
        effective_date,
        source: RateSource::Manual,
        source_reference: None,
        created_by: Some(user.id),
    };

    let result = repo.create_or_update_rate(input).await;

    // 3. Assert Success (Error caught correctly)
    match result {
        Ok(_) => panic!("Should NOT allow creating rate in closed fiscal period"),
        Err(e) => {
            let msg = e.to_string();
            // Match the actual error message format "Cannot modify exchange rate in closed fiscal period covering {0}"
            assert!(msg.contains("closed fiscal period"), "Unexpected error: {}", msg);
        }
    }
}
