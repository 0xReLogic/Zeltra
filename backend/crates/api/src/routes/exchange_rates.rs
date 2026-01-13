//! Exchange rate management routes.
//!
//! Implements Requirements 2.1, 2.4 for exchange rate API endpoints.

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
use std::str::FromStr;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_core::currency::ExchangeRateService;
use zeltra_db::{
    OrganizationRepository,
    entities::sea_orm_active_enums::{RateSource, UserRole},
    repositories::exchange_rate::{
        CreateExchangeRateInput, ExchangeRateRepository, RateLookupMethod,
    },
};

/// Creates the exchange rate routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/organizations/{org_id}/exchange-rates",
            get(get_exchange_rate),
        )
        .route(
            "/organizations/{org_id}/exchange-rates",
            post(create_exchange_rate),
        )
        .route(
            "/organizations/{org_id}/exchange-rates/fetch",
            post(fetch_exchange_rates),
        )
        .route(
            "/organizations/{org_id}/exchange-rates/bulk",
            post(bulk_import_rates),
        )
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

// ============================================================================
// Fetch Rates Types (Task 6.1)
// ============================================================================

/// Request body for fetching rates from external API.
#[derive(Debug, Deserialize)]
pub struct FetchRatesRequest {
    /// Base currency code (e.g., "EUR").
    pub base_currency: String,
    /// Target currency codes to fetch rates for.
    pub target_currencies: Vec<String>,
    /// Optional date for historical rates (defaults to today).
    pub date: Option<NaiveDate>,
}

/// Response for fetch rates operation.
#[derive(Debug, Serialize)]
pub struct FetchRatesResponse {
    /// Number of rates fetched.
    pub fetched_count: usize,
    /// Number of rates stored (new).
    pub imported_count: usize,
    /// Number of rates updated (existing).
    pub updated_count: usize,
    /// The fetched rates.
    pub rates: Vec<FetchedRateItem>,
}

/// A single fetched rate item.
#[derive(Debug, Serialize)]
pub struct FetchedRateItem {
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate.
    pub rate: String,
    /// Effective date.
    pub effective_date: NaiveDate,
    /// Source of the rate.
    pub source: String,
}

// ============================================================================
// Bulk Import Types (Task 6.1)
// ============================================================================

/// Request body for bulk rate import.
#[derive(Debug, Deserialize)]
pub struct BulkImportRequest {
    /// Rates to import.
    pub rates: Vec<BulkRateItem>,
}

/// A single rate item for bulk import.
#[derive(Debug, Deserialize)]
pub struct BulkRateItem {
    /// Source currency code.
    pub from_currency: String,
    /// Target currency code.
    pub to_currency: String,
    /// Exchange rate (as string for precision).
    pub rate: String,
    /// Effective date.
    pub effective_date: NaiveDate,
}

/// Response for bulk import operation.
#[derive(Debug, Serialize)]
pub struct BulkImportResponse {
    /// Number of rates imported (new).
    pub imported_count: usize,
    /// Number of rates updated (existing).
    pub updated_count: usize,
    /// Validation errors for individual rates.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BulkImportError>,
}

/// Error for a single rate in bulk import.
#[derive(Debug, Serialize)]
pub struct BulkImportError {
    /// Index of the rate in the input.
    pub index: usize,
    /// Currency pair.
    pub currency_pair: String,
    /// Error message.
    pub message: String,
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

    let date = query
        .date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match rate_repo
        .find_rate(org_id, &query.from, &query.to, date)
        .await
    {
        Ok(lookup) => {
            let response = ExchangeRateResponse {
                from_currency: query.from,
                to_currency: query.to,
                rate: lookup.rate.to_string(),
                effective_date: lookup.effective_date,
                lookup_method: lookup_method_to_string(lookup.lookup_method),
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
                zeltra_db::repositories::exchange_rate::ExchangeRateError::CurrencyNotFound(
                    currency,
                ) => (
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

// ============================================================================
// Fetch Rates Endpoint (Task 6.1)
// ============================================================================

/// Validate fetch rates request.
fn validate_fetch_request(payload: &FetchRatesRequest) -> Option<axum::response::Response> {
    if payload.target_currencies.is_empty() {
        return Some(
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "validation_error",
                    "message": "At least one target currency is required"
                })),
            )
                .into_response(),
        );
    }

    if payload.target_currencies.len() > 50 {
        return Some(
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "validation_error",
                    "message": "Maximum 50 target currencies allowed per request"
                })),
            )
                .into_response(),
        );
    }

    None
}

/// Store fetched rates and build response.
async fn store_and_respond(
    rate_repo: &ExchangeRateRepository,
    fetched_rates: Vec<zeltra_core::currency::FetchedRate>,
    org_id: Uuid,
    user_id: Uuid,
    base_currency: &str,
) -> axum::response::Response {
    let fetched_count = fetched_rates.len();

    if fetched_rates.is_empty() {
        return (
            StatusCode::OK,
            Json(FetchRatesResponse {
                fetched_count: 0,
                imported_count: 0,
                updated_count: 0,
                rates: vec![],
            }),
        )
            .into_response();
    }

    // Convert to repository input format
    let inputs: Vec<CreateExchangeRateInput> = fetched_rates
        .iter()
        .map(|r| CreateExchangeRateInput {
            organization_id: org_id,
            from_currency: r.from_currency.clone(),
            to_currency: r.to_currency.clone(),
            rate: r.rate,
            effective_date: r.effective_date,
            source: RateSource::Api,
            source_reference: Some(r.source.clone()),
            created_by: Some(user_id),
        })
        .collect();

    // Store rates using bulk import
    let (imported_count, updated_count) = match rate_repo.bulk_import(inputs).await {
        Ok(counts) => counts,
        Err(e) => {
            error!(error = %e, "Failed to store fetched rates");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "storage_error",
                    "message": format!("Failed to store rates: {e}")
                })),
            )
                .into_response();
        }
    };

    info!(
        org_id = %org_id,
        base_currency = %base_currency,
        fetched_count = fetched_count,
        imported_count = imported_count,
        updated_count = updated_count,
        "Exchange rates fetched and stored"
    );

    // Build response
    let rates: Vec<FetchedRateItem> = fetched_rates
        .into_iter()
        .map(|r| FetchedRateItem {
            from_currency: r.from_currency,
            to_currency: r.to_currency,
            rate: r.rate.to_string(),
            effective_date: r.effective_date,
            source: r.source,
        })
        .collect();

    (
        StatusCode::OK,
        Json(FetchRatesResponse {
            fetched_count,
            imported_count,
            updated_count,
            rates,
        }),
    )
        .into_response()
}

/// POST `/organizations/{org_id}/exchange-rates/fetch` - Fetch rates from external API.
///
/// Requirements: 2.1
async fn fetch_exchange_rates(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<FetchRatesRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if let Some(response) = validate_fetch_request(&payload) {
        return response;
    }

    let rate_service = match ExchangeRateService::from_env() {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to create exchange rate service");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "service_error",
                    "message": "Failed to initialize exchange rate service"
                })),
            )
                .into_response();
        }
    };

    let fetched_rates = match fetch_rates_from_api(&rate_service, &payload).await {
        Ok(rates) => rates,
        Err(response) => return response,
    };

    let rate_repo = ExchangeRateRepository::new((*state.db).clone());
    store_and_respond(
        &rate_repo,
        fetched_rates,
        org_id,
        auth.user_id(),
        &payload.base_currency,
    )
    .await
}

/// Fetch rates from external API.
async fn fetch_rates_from_api(
    rate_service: &ExchangeRateService,
    payload: &FetchRatesRequest,
) -> Result<Vec<zeltra_core::currency::FetchedRate>, axum::response::Response> {
    if let Some(date) = payload.date {
        rate_service
            .fetch_historical(&payload.base_currency, &payload.target_currencies, date)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to fetch historical rates");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({
                        "error": "external_service_error",
                        "message": format!("Failed to fetch rates from external API: {e}")
                    })),
                )
                    .into_response()
            })
    } else {
        rate_service
            .fetch_latest(&payload.base_currency, &payload.target_currencies)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to fetch latest rates");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({
                        "error": "external_service_error",
                        "message": format!("Failed to fetch rates from external API: {e}")
                    })),
                )
                    .into_response()
            })
    }
}

// ============================================================================
// Bulk Import Endpoint (Task 6.1)
// ============================================================================

/// Validate bulk import request.
fn validate_bulk_request(payload: &BulkImportRequest) -> Option<axum::response::Response> {
    if payload.rates.is_empty() {
        return Some(
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "validation_error",
                    "message": "At least one rate is required"
                })),
            )
                .into_response(),
        );
    }

    if payload.rates.len() > 1000 {
        return Some(
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "validation_error",
                    "message": "Maximum 1000 rates allowed per request"
                })),
            )
                .into_response(),
        );
    }

    None
}

/// Validate a single rate item and convert to input.
fn validate_rate_item(
    item: &BulkRateItem,
    index: usize,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<CreateExchangeRateInput, BulkImportError> {
    let currency_pair = format!("{}/{}", item.from_currency, item.to_currency);

    // Parse rate string to Decimal
    let rate = Decimal::from_str(&item.rate).map_err(|e| BulkImportError {
        index,
        currency_pair: currency_pair.clone(),
        message: format!("Invalid rate format: {e}"),
    })?;

    // Validate rate is positive
    if rate <= Decimal::ZERO {
        return Err(BulkImportError {
            index,
            currency_pair,
            message: "Rate must be positive".to_string(),
        });
    }

    // Validate currencies are different
    if item.from_currency == item.to_currency {
        return Err(BulkImportError {
            index,
            currency_pair,
            message: "From and to currencies must be different".to_string(),
        });
    }

    // Validate currency codes
    if !is_valid_currency_code(&item.from_currency) {
        return Err(BulkImportError {
            index,
            currency_pair,
            message: format!("Invalid from currency code: {}", item.from_currency),
        });
    }

    if !is_valid_currency_code(&item.to_currency) {
        return Err(BulkImportError {
            index,
            currency_pair,
            message: format!("Invalid to currency code: {}", item.to_currency),
        });
    }

    Ok(CreateExchangeRateInput {
        organization_id: org_id,
        from_currency: item.from_currency.clone(),
        to_currency: item.to_currency.clone(),
        rate,
        effective_date: item.effective_date,
        source: RateSource::Manual,
        source_reference: Some("bulk_import".to_string()),
        created_by: Some(user_id),
    })
}

/// POST `/organizations/{org_id}/exchange-rates/bulk` - Bulk import exchange rates.
///
/// Requirements: 2.4
async fn bulk_import_rates(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(org_id): Path<Uuid>,
    Json(payload): Json<BulkImportRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    if let Err(response) = check_admin_role(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    if let Some(response) = validate_bulk_request(&payload) {
        return response;
    }

    // Pre-validate and parse rates
    let mut errors: Vec<BulkImportError> = Vec::new();
    let mut inputs: Vec<CreateExchangeRateInput> = Vec::new();

    for (index, item) in payload.rates.iter().enumerate() {
        match validate_rate_item(item, index, org_id, auth.user_id()) {
            Ok(input) => inputs.push(input),
            Err(e) => errors.push(e),
        }
    }

    // If there are validation errors, return them without importing
    if !errors.is_empty() {
        warn!(org_id = %org_id, error_count = errors.len(), "Bulk import validation failed");
        return (
            StatusCode::BAD_REQUEST,
            Json(BulkImportResponse {
                imported_count: 0,
                updated_count: 0,
                errors,
            }),
        )
            .into_response();
    }

    // Perform atomic bulk import
    let rate_repo = ExchangeRateRepository::new((*state.db).clone());
    match rate_repo.bulk_import(inputs).await {
        Ok((imported_count, updated_count)) => {
            info!(org_id = %org_id, imported_count, updated_count, "Bulk import completed");
            (
                StatusCode::OK,
                Json(BulkImportResponse {
                    imported_count,
                    updated_count,
                    errors: vec![],
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to bulk import rates");
            let error_msg = match e {
                zeltra_db::repositories::exchange_rate::ExchangeRateError::CurrencyNotFound(c) => {
                    format!("Currency '{c}' not found in database")
                }
                zeltra_db::repositories::exchange_rate::ExchangeRateError::NonPositiveRate => {
                    "One or more rates are not positive".to_string()
                }
                zeltra_db::repositories::exchange_rate::ExchangeRateError::SameCurrency => {
                    "One or more rates have same from/to currency".to_string()
                }
                _ => "Database error during import".to_string(),
            };
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "import_error", "message": error_msg })),
            )
                .into_response()
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

fn lookup_method_to_string(method: RateLookupMethod) -> String {
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

/// Validate a currency code (3 uppercase letters).
fn is_valid_currency_code(code: &str) -> bool {
    code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_currency_code() {
        assert!(is_valid_currency_code("USD"));
        assert!(is_valid_currency_code("EUR"));
        assert!(is_valid_currency_code("IDR"));

        assert!(!is_valid_currency_code("usd")); // lowercase
        assert!(!is_valid_currency_code("US")); // too short
        assert!(!is_valid_currency_code("USDD")); // too long
        assert!(!is_valid_currency_code("US1")); // contains digit
        assert!(!is_valid_currency_code("")); // empty
    }

    #[test]
    fn test_string_to_rate_source() {
        assert_eq!(string_to_rate_source("manual"), Some(RateSource::Manual));
        assert_eq!(string_to_rate_source("api"), Some(RateSource::Api));
        assert_eq!(
            string_to_rate_source("bank_feed"),
            Some(RateSource::BankFeed)
        );
        assert_eq!(string_to_rate_source("MANUAL"), Some(RateSource::Manual));
        assert_eq!(string_to_rate_source("unknown"), None);
    }

    #[test]
    fn test_rate_source_to_string() {
        assert_eq!(rate_source_to_string(&RateSource::Manual), "manual");
        assert_eq!(rate_source_to_string(&RateSource::Api), "api");
        assert_eq!(rate_source_to_string(&RateSource::BankFeed), "bank_feed");
    }

    #[test]
    fn test_lookup_method_to_string() {
        assert_eq!(lookup_method_to_string(RateLookupMethod::Direct), "direct");
        assert_eq!(
            lookup_method_to_string(RateLookupMethod::Inverse),
            "inverse"
        );
        assert_eq!(
            lookup_method_to_string(RateLookupMethod::Triangulated),
            "triangulated"
        );
    }

    #[test]
    fn test_validate_fetch_request_empty_currencies() {
        let payload = FetchRatesRequest {
            base_currency: "EUR".to_string(),
            target_currencies: vec![],
            date: None,
        };
        assert!(validate_fetch_request(&payload).is_some());
    }

    #[test]
    fn test_validate_fetch_request_too_many_currencies() {
        let payload = FetchRatesRequest {
            base_currency: "EUR".to_string(),
            target_currencies: (0..51).map(|i| format!("C{i:02}")).collect(),
            date: None,
        };
        assert!(validate_fetch_request(&payload).is_some());
    }

    #[test]
    fn test_validate_fetch_request_valid() {
        let payload = FetchRatesRequest {
            base_currency: "EUR".to_string(),
            target_currencies: vec!["USD".to_string(), "GBP".to_string()],
            date: None,
        };
        assert!(validate_fetch_request(&payload).is_none());
    }

    #[test]
    fn test_validate_bulk_request_empty() {
        let payload = BulkImportRequest { rates: vec![] };
        assert!(validate_bulk_request(&payload).is_some());
    }

    #[test]
    fn test_validate_bulk_request_too_many() {
        let payload = BulkImportRequest {
            rates: (0..1001)
                .map(|i| BulkRateItem {
                    from_currency: "EUR".to_string(),
                    to_currency: format!("C{i:02}"),
                    rate: "1.0".to_string(),
                    effective_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                })
                .collect(),
        };
        assert!(validate_bulk_request(&payload).is_some());
    }

    #[test]
    fn test_validate_rate_item_valid() {
        let item = BulkRateItem {
            from_currency: "EUR".to_string(),
            to_currency: "USD".to_string(),
            rate: "1.08".to_string(),
            effective_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        let result = validate_rate_item(&item, 0, Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rate_item_invalid_rate_format() {
        let item = BulkRateItem {
            from_currency: "EUR".to_string(),
            to_currency: "USD".to_string(),
            rate: "invalid".to_string(),
            effective_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        let result = validate_rate_item(&item, 0, Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Invalid rate format"));
    }

    #[test]
    fn test_validate_rate_item_negative_rate() {
        let item = BulkRateItem {
            from_currency: "EUR".to_string(),
            to_currency: "USD".to_string(),
            rate: "-1.08".to_string(),
            effective_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        let result = validate_rate_item(&item, 0, Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("positive"));
    }

    #[test]
    fn test_validate_rate_item_same_currency() {
        let item = BulkRateItem {
            from_currency: "EUR".to_string(),
            to_currency: "EUR".to_string(),
            rate: "1.0".to_string(),
            effective_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        let result = validate_rate_item(&item, 0, Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("different"));
    }

    #[test]
    fn test_validate_rate_item_invalid_currency_code() {
        let item = BulkRateItem {
            from_currency: "eu".to_string(), // lowercase
            to_currency: "USD".to_string(),
            rate: "1.08".to_string(),
            effective_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        let result = validate_rate_item(&item, 0, Uuid::new_v4(), Uuid::new_v4());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Invalid from currency")
        );
    }
}

// ============================================================================
// Integration Tests (Task 6.2)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, middleware::from_fn_with_state};
    use std::sync::Arc;
    use tower::ServiceExt;
    use zeltra_shared::{EmailConfig, EmailService, JwtConfig, JwtService};

    use crate::middleware::auth::auth_middleware;

    /// Get database URL from environment.
    fn get_database_url() -> String {
        std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("ZELTRA__DATABASE__URL"))
            .unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
            })
    }

    /// Helper to create a test AppState with real DB.
    async fn create_test_state_with_db() -> AppState {
        let db_url = get_database_url();
        let db = sea_orm::Database::connect(&db_url)
            .await
            .expect("Failed to connect to database");
        let jwt_service = JwtService::new(JwtConfig::default());
        let email_service = EmailService::new(EmailConfig::default());

        AppState {
            db: Arc::new(db),
            jwt_service: Arc::new(jwt_service),
            email_service: Arc::new(email_service),
            storage: None,
        }
    }

    #[tokio::test]
    async fn test_fetch_rates_no_auth() {
        let state = create_test_state_with_db().await;

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let org_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/organizations/{org_id}/exchange-rates/fetch"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"base_currency":"EUR","target_currencies":["USD"]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_bulk_import_no_auth() {
        let state = create_test_state_with_db().await;

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let org_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/organizations/{org_id}/exchange-rates/bulk"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"rates":[{"from_currency":"EUR","to_currency":"USD","rate":"1.08","effective_date":"2024-01-01"}]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
