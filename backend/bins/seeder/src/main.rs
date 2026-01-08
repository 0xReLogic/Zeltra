//! Database seeder for Zeltra development and testing.
//!
//! Seeds test organization, exchange rates, dimension types, and dimension values
//! for local development and testing purposes.
//!
//! Usage: cargo run --bin seeder

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::str::FromStr;
use uuid::Uuid;
use zeltra_db::entities::{
    dimension_types, dimension_values, exchange_rates, organizations,
    sea_orm_active_enums::{RateSource, SubscriptionStatus, SubscriptionTier},
    users,
};

/// Test organization ID (consistent for all seeds)
const TEST_ORG_ID: &str = "00000000-0000-0000-0000-000000000001";
/// Test user ID (consistent for all seeds)
const TEST_USER_ID: &str = "00000000-0000-0000-0000-000000000002";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    println!("Connecting to database...");
    let db = zeltra_db::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Seeding test user...");
    seed_test_user(&db).await;

    println!("Seeding test organization...");
    seed_test_organization(&db).await;

    println!("Seeding exchange rates...");
    seed_exchange_rates(&db).await;

    println!("Seeding dimension types...");
    seed_dimension_types(&db).await;

    println!("Seeding dimension values...");
    seed_dimension_values(&db).await;

    println!("Seeding complete!");
}

fn test_org_id() -> Uuid {
    Uuid::parse_str(TEST_ORG_ID).unwrap()
}

fn test_user_id() -> Uuid {
    Uuid::parse_str(TEST_USER_ID).unwrap()
}

/// Seeds a test user for development.
async fn seed_test_user(db: &DatabaseConnection) {
    // Check if user already exists
    if users::Entity::find_by_id(test_user_id())
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        println!("  Test user already exists, skipping...");
        return;
    }

    let user = users::ActiveModel {
        id: Set(test_user_id()),
        email: Set("test@zeltra.dev".to_string()),
        password_hash: Set("$argon2id$v=19$m=65536,t=3,p=4$test_hash".to_string()),
        full_name: Set("Test User".to_string()),
        is_active: Set(true),
        email_verified_at: Set(Some(Utc::now().into())),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    if let Err(e) = user.insert(db).await {
        eprintln!("Failed to insert test user: {e}");
    } else {
        println!("  Created test user: test@zeltra.dev");
    }
}

/// Seeds a test organization for development.
async fn seed_test_organization(db: &DatabaseConnection) {
    // Check if org already exists
    if organizations::Entity::find_by_id(test_org_id())
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        println!("  Test organization already exists, skipping...");
        return;
    }

    let org = organizations::ActiveModel {
        id: Set(test_org_id()),
        name: Set("Test Organization".to_string()),
        slug: Set("test-org".to_string()),
        base_currency: Set("USD".to_string()),
        timezone: Set("UTC".to_string()),
        settings: Set(serde_json::json!({})),
        subscription_tier: Set(SubscriptionTier::Enterprise),
        subscription_status: Set(SubscriptionStatus::Active),
        trial_ends_at: Set(None),
        subscription_ends_at: Set(None),
        payment_provider: Set(None),
        payment_customer_id: Set(None),
        payment_subscription_id: Set(None),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    if let Err(e) = org.insert(db).await {
        eprintln!("Failed to insert test organization: {e}");
    } else {
        println!("  Created test organization: Test Organization");
    }
}

/// Seeds 30 days of exchange rates with USD as base currency.
async fn seed_exchange_rates(db: &DatabaseConnection) {
    let org_id = test_org_id();
    let user_id = test_user_id();

    // Exchange rates relative to USD (approximate values for testing)
    let rates = [
        ("EUR", "0.92"),
        ("GBP", "0.79"),
        ("JPY", "149.50"),
        ("IDR", "15750.00"),
        ("SGD", "1.34"),
        ("AUD", "1.53"),
        ("CAD", "1.36"),
        ("CHF", "0.88"),
        ("CNY", "7.24"),
        ("HKD", "7.82"),
        ("INR", "83.12"),
        ("KRW", "1320.00"),
        ("MXN", "17.15"),
        ("MYR", "4.72"),
        ("NZD", "1.64"),
        ("PHP", "56.20"),
        ("THB", "35.80"),
        ("TWD", "31.50"),
        ("VND", "24500.00"),
        ("ZAR", "18.90"),
    ];

    let today = Utc::now().date_naive();
    let mut inserted = 0;

    for day_offset in 0..30 {
        let effective_date = today - Duration::days(day_offset);

        for (to_currency, base_rate) in &rates {
            // Add small daily variation (0.1% to simulate market movement)
            // Using Decimal for all calculations to avoid float arithmetic
            let variation_pct = if day_offset % 2 == 0 {
                Decimal::from(day_offset) * Decimal::from_str("0.001").unwrap()
            } else {
                Decimal::from(day_offset) * Decimal::from_str("-0.001").unwrap()
            };
            let variation = Decimal::ONE + variation_pct;
            let rate_value = Decimal::from_str(base_rate).unwrap() * variation;

            let exchange_rate = exchange_rates::ActiveModel {
                id: Set(Uuid::new_v4()),
                organization_id: Set(org_id),
                from_currency: Set("USD".to_string()),
                to_currency: Set(to_currency.to_string()),
                rate: Set(rate_value.round_dp(6)),
                effective_date: Set(effective_date),
                source: Set(RateSource::Manual),
                source_reference: Set(Some("seeder".to_string())),
                created_by: Set(Some(user_id)),
                created_at: Set(Utc::now().into()),
            };

            if let Err(e) = exchange_rate.insert(db).await {
                // Ignore duplicate key errors (rate already exists)
                if !e.to_string().contains("duplicate key") {
                    eprintln!("Failed to insert exchange rate: {e}");
                }
            } else {
                inserted += 1;
            }
        }
    }

    println!("  Inserted {inserted} exchange rates (30 days x 20 currency pairs)");
}

/// Seeds dimension types for organizational structure.
async fn seed_dimension_types(db: &DatabaseConnection) {
    let org_id = test_org_id();

    let dimension_types_data = [
        (
            "DEPARTMENT",
            "Department",
            "Organizational departments for cost allocation",
            1,
        ),
        (
            "PROJECT",
            "Project",
            "Projects for tracking project-specific expenses",
            2,
        ),
        (
            "COST_CENTER",
            "Cost Center",
            "Cost centers for budget management",
            3,
        ),
        ("LOCATION", "Location", "Geographic locations or offices", 4),
        (
            "PRODUCT",
            "Product",
            "Product lines for revenue/expense tracking",
            5,
        ),
    ];

    let mut inserted = 0;
    for (code, name, description, sort_order) in dimension_types_data {
        let dimension_type = dimension_types::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(org_id),
            code: Set(code.to_string()),
            name: Set(name.to_string()),
            description: Set(Some(description.to_string())),
            is_required: Set(false),
            is_active: Set(true),
            sort_order: Set(sort_order),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        if let Err(e) = dimension_type.insert(db).await {
            if !e.to_string().contains("duplicate key") {
                eprintln!("Failed to insert dimension type {code}: {e}");
            }
        } else {
            inserted += 1;
        }
    }

    println!("  Inserted {inserted} dimension types");
}

/// Seeds sample dimension values for testing.
#[allow(clippy::too_many_lines)]
async fn seed_dimension_values(db: &DatabaseConnection) {
    use sea_orm::{ColumnTrait, QueryFilter};

    let org_id = test_org_id();

    // First, get the dimension type IDs
    let dept_type = dimension_types::Entity::find()
        .filter(dimension_types::Column::Code.eq("DEPARTMENT"))
        .filter(dimension_types::Column::OrganizationId.eq(org_id))
        .one(db)
        .await
        .ok()
        .flatten();

    let project_type = dimension_types::Entity::find()
        .filter(dimension_types::Column::Code.eq("PROJECT"))
        .filter(dimension_types::Column::OrganizationId.eq(org_id))
        .one(db)
        .await
        .ok()
        .flatten();

    let cost_center_type = dimension_types::Entity::find()
        .filter(dimension_types::Column::Code.eq("COST_CENTER"))
        .filter(dimension_types::Column::OrganizationId.eq(org_id))
        .one(db)
        .await
        .ok()
        .flatten();

    // Seed departments
    if let Some(dept) = dept_type {
        let departments = [
            ("DEPT-ENG", "Engineering"),
            ("DEPT-SALES", "Sales"),
            ("DEPT-MKT", "Marketing"),
            ("DEPT-FIN", "Finance"),
            ("DEPT-HR", "Human Resources"),
            ("DEPT-OPS", "Operations"),
        ];

        let mut inserted = 0;
        for (code, name) in departments {
            let value = dimension_values::ActiveModel {
                id: Set(Uuid::new_v4()),
                organization_id: Set(org_id),
                dimension_type_id: Set(dept.id),
                code: Set(code.to_string()),
                name: Set(name.to_string()),
                description: Set(None),
                parent_id: Set(None),
                is_active: Set(true),
                effective_from: Set(None),
                effective_to: Set(None),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };

            if let Err(e) = value.insert(db).await {
                if !e.to_string().contains("duplicate key") {
                    eprintln!("Failed to insert department {code}: {e}");
                }
            } else {
                inserted += 1;
            }
        }
        println!("  Inserted {inserted} departments");
    }

    // Seed projects
    if let Some(proj) = project_type {
        let projects = [
            ("PROJ-ALPHA", "Project Alpha"),
            ("PROJ-BETA", "Project Beta"),
            ("PROJ-GAMMA", "Project Gamma"),
            ("PROJ-INFRA", "Infrastructure Upgrade"),
            ("PROJ-MOBILE", "Mobile App Development"),
        ];

        let mut inserted = 0;
        for (code, name) in projects {
            let value = dimension_values::ActiveModel {
                id: Set(Uuid::new_v4()),
                organization_id: Set(org_id),
                dimension_type_id: Set(proj.id),
                code: Set(code.to_string()),
                name: Set(name.to_string()),
                description: Set(None),
                parent_id: Set(None),
                is_active: Set(true),
                effective_from: Set(None),
                effective_to: Set(None),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };

            if let Err(e) = value.insert(db).await {
                if !e.to_string().contains("duplicate key") {
                    eprintln!("Failed to insert project {code}: {e}");
                }
            } else {
                inserted += 1;
            }
        }
        println!("  Inserted {inserted} projects");
    }

    // Seed cost centers
    if let Some(cc) = cost_center_type {
        let cost_centers = [
            ("CC-100", "General Administration"),
            ("CC-200", "Research & Development"),
            ("CC-300", "Sales & Marketing"),
            ("CC-400", "Customer Support"),
            ("CC-500", "IT Infrastructure"),
        ];

        let mut inserted = 0;
        for (code, name) in cost_centers {
            let value = dimension_values::ActiveModel {
                id: Set(Uuid::new_v4()),
                organization_id: Set(org_id),
                dimension_type_id: Set(cc.id),
                code: Set(code.to_string()),
                name: Set(name.to_string()),
                description: Set(None),
                parent_id: Set(None),
                is_active: Set(true),
                effective_from: Set(None),
                effective_to: Set(None),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };

            if let Err(e) = value.insert(db).await {
                if !e.to_string().contains("duplicate key") {
                    eprintln!("Failed to insert cost center {code}: {e}");
                }
            } else {
                inserted += 1;
            }
        }
        println!("  Inserted {inserted} cost centers");
    }
}
