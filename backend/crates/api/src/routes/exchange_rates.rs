//! Exchange rate management routes.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::{RateSource, UserRole},
    repositories::exchange_rate::{CreateExchangeRateInput, ExchangeRateRepository, RateLookupMethod},
};

/// Creates the exchange rate routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/organizations/{org_id}/exchange-rates", get(get_exchange_rate))
        .route("/organizations/{org_id}/exchange-rates", post(create_exchange_rate))
}

/// Query parameters for getting an exchange rate.
#[derive(Debug, Deserialize)]
pub struct GetExchangeRateQuery {
    /// Source currency code.
    pub from: String,
    /// Target currency code.
    pub to: String,
    /// Date for the rate lookup (defaults to today).
    pub date: Option<NaiveDate>,
}

/// Request body for creating/updating an exchange rate.
#[derive(Debug, Deserialize)]
pub struct CreateExchangeRateRequest {
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate (from_currency * rate = to_currency).
    pub rate: Decimal,
    /// Effective date for this rate.
    pub effective_date: NaiveDate,
    /// Source of the rate: "manual", "api", "bank_feed".
    pub source: Option<String>,
    /// Optional reference (e.g., API provider, bank name).
    pub source_reference: Option<String>,
}

/// Response for an exchange rate lookup.
#[derive(Debug, Serialize)]
pub struct ExchangeRateResponse {
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate.
    pub rate: String,
    /// Effective date of the rate.
    pub effective_date: NaiveDate,
    /// How the rate was obtained: "direct", "inverse", "triangulated".
    pub lookup_method: String,
}

/// GET `/organizations/{org_id}/exchange-rates` - Get exchange rate for currency pair.
async fn get_exchange_rate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Query(query): Query<GetExchangeRateQuery>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let rate_repo = ExchangeRateRepository::new((*state.db).clone());

    let date = query.date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    match rate_repo.find_rate(org_id, &query.from, &query.to, date).await {
        Ok(lookup) => {
            let response = ExchangeRateResponse {
                from_currency: query.from,
                to_currency: query.to,
                rate: lookup.rate.to_string(),
                effective_date: lookup.effective_date,
                lookup_method: lookup_method_to_string(&lookup.lookup_method),
            };

            (StatusCode::OK, Json(json!(response))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to get exchange rate");
            match e {
                zeltra_db::repositories::exchange_rate::ExchangeRateError::RateNotFound(from, to, date) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "rate_not_found",
                        "message": format!("No exchange rate found for {}/{} on or before {}", from, to, date)
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}

/// POST `/organizations/{org_id}/exchange-rates` - Create or update an exchange rate.
async fn create_exchange_rate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<CreateExchangeRateRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check admin/owner role
    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let rate_repo = ExchangeRateRepository::new((*state.db).clone());

    let source = payload
        .source
        .as_ref()
        .and_then(|s| string_to_rate_source(s))
        .unwrap_or(RateSource::Manual);

    let input = CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: payload.from_currency.clone(),
        to_currency: payload.to_currency.clone(),
        rate: payload.rate,
        effective_date: payload.effective_date,
        source,
        source_reference: payload.source_reference,
        created_by: Some(auth.user_id()),
    };

    match rate_repo.create_or_update_rate(input).await {
        Ok(rate) => {
            info!(
                org_id = %org_id,
                from = %rate.from_currency,
                to = %rate.to_currency,
                rate = %rate.rate,
                "Exchange rate created/updated"
            );

            (
                StatusCode::CREATED,
                Json(json!({
                    "id": rate.id,
                    "from_currency": rate.from_currency,
                    "to_currency": rate.to_currency,
                    "rate": rate.rate.to_string(),
                    "effective_date": rate.effective_date,
                    "source": rate_source_to_string(&rate.source),
                    "source_reference": rate.source_reference,
                    "created_at": rate.created_at
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to create exchange rate");
            match e {
                zeltra_db::repositories::exchange_rate::ExchangeRateError::NonPositiveRate => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "invalid_rate",
                        "message": "Exchange rate must be positive"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::exchange_rate::ExchangeRateError::SameCurrency => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "same_currency",
                        "message": "From and to currencies must be different"
                    })),
                )
                    .into_response(),
                zeltra_db::repositories::exchange_rate::ExchangeRateError::CurrencyNotFound(currency) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "currency_not_found",
                        "message": format!("Currency '{}' not found", currency)
                    })),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "internal_error",
                        "message": "An error occurred"
                    })),
                )
                    .into_response(),
            }
        }
    }
}

// Helper functions

async fn check_membership(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.is_member(org_id, user_id).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You are not a member of this organization"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking membership");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response())
        }
    }
}

async fn check_admin_role(
    org_repo: &OrganizationRepository,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<(), axum::response::Response> {
    match org_repo.has_role(org_id, user_id, UserRole::Admin).await {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "You need admin or owner role to perform this action"
            })),
        )
            .into_response()),
        Err(e) => {
            error!(error = %e, "Database error checking role");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response())
        }
    }
}

fn lookup_method_to_string(method: &RateLookupMethod) -> String {
    match method {
        RateLookupMethod::Direct => "direct".to_string(),
        RateLookupMethod::Inverse => "inverse".to_string(),
        RateLookupMethod::Triangulated => "triangulated".to_string(),
    }
}

fn rate_source_to_string(source: &RateSource) -> String {
    match source {
        RateSource::Manual => "manual".to_string(),
        RateSource::Api => "api".to_string(),
        RateSource::BankFeed => "bank_feed".to_string(),
    }
}

fn string_to_rate_source(s: &str) -> Option<RateSource> {
    match s.to_lowercase().as_str() {
        "manual" => Some(RateSource::Manual),
        "api" => Some(RateSource::Api),
        "bank_feed" => Some(RateSource::BankFeed),
        _ => None,
    }
}
