//! Property-based tests for Subscription Middleware.
//!
//! Feature: entities-model-implementation
//!
//! Tests universal correctness properties:
//! - Property 16: Subscription status middleware - test all status combinations

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
};
use proptest::prelude::*;
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tower::ServiceExt;
use uuid::Uuid;
use zeltra_db::entities::{
    sea_orm_active_enums::{SubscriptionStatus, SubscriptionTier},
    users,
};
use zeltra_shared::Claims;

// Simplified AppState for testing (only needs db)
#[derive(Clone)]
struct TestAppState {
    pub db: Arc<DatabaseConnection>,
}

// Simplified subscription middleware for testing
async fn test_subscription_middleware(
    State(state): State<TestAppState>,
    request: Request<Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract claims from request extensions
    let claims = request.extensions().get::<Claims>().cloned();

    let Some(claims) = claims else {
        return next.run(request).await;
    };

    let user_id = claims.user_id();

    // Check user's subscription status
    let user = match users::Entity::find_by_id(user_id).one(&*state.db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({
                    "error": "user_not_found",
                    "message": "User not found"
                })),
            )
                .into_response();
        }
        Err(_) => {
            return next.run(request).await;
        }
    };

    // Check if subscription is active or trialing
    match user.subscription_status {
        SubscriptionStatus::Active | SubscriptionStatus::Trialing => {
            next.run(request).await
        }
        SubscriptionStatus::Expired => {
            (
                StatusCode::PAYMENT_REQUIRED,
                axum::Json(serde_json::json!({
                    "error": "subscription_expired",
                    "message": "Your trial has expired. Please upgrade to continue using Zeltra.",
                    "subscription_status": "expired",
                    "trial_ends_at": user.trial_ends_at
                })),
            )
                .into_response()
        }
        SubscriptionStatus::Cancelled => {
            (
                StatusCode::PAYMENT_REQUIRED,
                axum::Json(serde_json::json!({
                    "error": "subscription_cancelled",
                    "message": "Your subscription has been cancelled. Please reactivate to continue.",
                    "subscription_status": "cancelled"
                })),
            )
                .into_response()
        }
        SubscriptionStatus::PastDue => {
            (
                StatusCode::PAYMENT_REQUIRED,
                axum::Json(serde_json::json!({
                    "error": "payment_past_due",
                    "message": "Your payment is past due. Please update your payment method to continue.",
                    "subscription_status": "past_due"
                })),
            )
                .into_response()
        }
    }
}

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
        })
    })
}

/// Helper to create a test user with a specific subscription status
async fn setup_user_with_status(db: &DatabaseConnection, status: SubscriptionStatus) -> Uuid {
    let user_id = Uuid::new_v4();

    let user = users::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("hash".to_string()),
        full_name: Set("Test User".to_string()),
        subscription_tier: Set(SubscriptionTier::Starter),
        subscription_status: Set(status),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    users::Entity::insert(user)
        .exec(db)
        .await
        .expect("Failed to insert user");

    user_id
}

/// Helper: Run async code in a temporary runtime
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(future)
}

/// Test handler that returns 200 OK
async fn test_handler() -> impl IntoResponse {
    (StatusCode::OK, "Success")
}

/// Create a test app with subscription middleware
fn create_test_app(state: TestAppState) -> Router {
    Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            test_subscription_middleware,
        ))
        .with_state(state)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 16: Subscription Status Middleware
    ///
    /// Feature: entities-model-implementation, Property 16: Subscription Status Middleware
    ///
    /// Tests that subscription middleware correctly allows/blocks requests based on status:
    /// - 'active' and 'trialing' statuses should allow requests (HTTP 200)
    /// - 'expired' and 'cancelled' statuses should reject with HTTP 402
    /// - 'past_due' status should reject with HTTP 402
    #[test]
    fn property_16_subscription_status_middleware(
        seed in 0u64..1000u64,
    ) {
        block_on(async {
            // Connect to database
            let db_url = get_database_url();
            let db = Database::connect(&db_url)
                .await
                .expect("Failed to connect to database");
            let db = Arc::new(db);

            // Test all subscription statuses
            let statuses = vec![
                (SubscriptionStatus::Active, true, "active"),
                (SubscriptionStatus::Trialing, true, "trialing"),
                (SubscriptionStatus::Expired, false, "expired"),
                (SubscriptionStatus::Cancelled, false, "cancelled"),
                (SubscriptionStatus::PastDue, false, "past_due"),
            ];

            for (status, should_allow, status_name) in statuses {
                // Create user with specific status
                let user_id = setup_user_with_status(&db, status).await;

                // Create app state
                let state = TestAppState {
                    db: db.clone(),
                };

                // Create test app
                let app = create_test_app(state);

                // Create claims for the user
                let org_id = Uuid::new_v4(); // Dummy org ID for testing
                let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
                let claims = Claims::new(
                    user_id,
                    org_id,
                    "owner",
                    expires_at,
                );

                // Create request with claims in extensions
                let mut request = Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap();
                request.extensions_mut().insert(claims);

                // Send request
                let response = app
                    .oneshot(request)
                    .await
                    .expect("Failed to send request");

                // Verify response
                if should_allow {
                    assert_eq!(
                        response.status(),
                        StatusCode::OK,
                        "Status '{}' should allow request (seed: {})",
                        status_name,
                        seed
                    );
                } else {
                    assert_eq!(
                        response.status(),
                        StatusCode::PAYMENT_REQUIRED,
                        "Status '{}' should reject with HTTP 402 (seed: {})",
                        status_name,
                        seed
                    );
                }

                // Cleanup: Delete test user
                users::Entity::delete_by_id(user_id)
                    .exec(&*db)
                    .await
                    .expect("Failed to delete test user");
            }
        });
    }
}
