//! Subscription status middleware.
//!
//! Blocks requests from organizations with expired/cancelled subscriptions.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::EntityTrait;
use serde_json::json;
use tracing::warn;
use zeltra_db::entities::{organizations, sea_orm_active_enums::SubscriptionStatus};

use crate::AppState;

/// Middleware to check subscription status before processing requests.
///
/// Allows requests if:
/// - Subscription status is 'active' or 'trialing'
/// - Request is to auth endpoints (login, register, etc.)
/// - Request is to billing/subscription endpoints
///
/// Blocks requests if:
/// - Subscription status is 'expired', 'cancelled', or 'past_due'
pub async fn check_subscription_status(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();

    // Allow auth endpoints (login, register, verify, etc.)
    if path.starts_with("/auth/") {
        return next.run(request).await;
    }

    // Allow health check
    if path == "/health" {
        return next.run(request).await;
    }

    // Allow billing/subscription endpoints (so users can upgrade)
    if path.starts_with("/billing") || path.starts_with("/subscriptions") {
        return next.run(request).await;
    }

    // Extract org_id from request extensions (set by auth middleware)
    let org_id = request.extensions().get::<uuid::Uuid>().copied();

    let Some(org_id) = org_id else {
        // No org_id means not authenticated or auth middleware hasn't run yet
        // Let it pass and let auth middleware handle it
        return next.run(request).await;
    };

    // Check subscription status
    let org = match organizations::Entity::find_by_id(org_id)
        .one(&*state.db)
        .await
    {
        Ok(Some(org)) => org,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "organization_not_found",
                    "message": "Organization not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            warn!(error = %e, "Failed to check subscription status");
            // On error, allow request to proceed (fail open)
            return next.run(request).await;
        }
    };

    // Check if subscription is active or trialing
    match org.subscription_status {
        SubscriptionStatus::Active | SubscriptionStatus::Trialing => {
            // All good, proceed
            next.run(request).await
        }
        SubscriptionStatus::Expired => {
            warn!(
                org_id = %org_id,
                org_name = %org.name,
                "🚫 Blocked request from organization with expired subscription"
            );

            (
                StatusCode::PAYMENT_REQUIRED,
                Json(json!({
                    "error": "subscription_expired",
                    "message": "Your trial has expired. Please upgrade to continue using Zeltra.",
                    "subscription_status": "expired",
                    "trial_ends_at": org.trial_ends_at
                })),
            )
                .into_response()
        }
        SubscriptionStatus::Cancelled => {
            warn!(
                org_id = %org_id,
                org_name = %org.name,
                "🚫 Blocked request from cancelled organization"
            );

            (
                StatusCode::PAYMENT_REQUIRED,
                Json(json!({
                    "error": "subscription_cancelled",
                    "message": "Your subscription has been cancelled. Please reactivate to continue.",
                    "subscription_status": "cancelled"
                })),
            )
                .into_response()
        }
        SubscriptionStatus::PastDue => {
            warn!(
                org_id = %org_id,
                org_name = %org.name,
                "🚫 Blocked request from organization with past due payment"
            );

            (
                StatusCode::PAYMENT_REQUIRED,
                Json(json!({
                    "error": "payment_past_due",
                    "message": "Your payment is past due. Please update your payment method to continue.",
                    "subscription_status": "past_due"
                })),
            )
                .into_response()
        }
    }
}
