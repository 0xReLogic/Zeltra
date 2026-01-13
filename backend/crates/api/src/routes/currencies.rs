//! Currency listing routes.

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use sea_orm::EntityTrait;
use serde::Serialize;
use serde_json::json;
use tracing::error;

use crate::{AppState, middleware::AuthUser};
use zeltra_db::entities::currencies;

/// Creates the currency routes (requires auth middleware to be applied externally).
pub fn routes() -> Router<AppState> {
    Router::new().route("/currencies", get(list_currencies))
}

/// Response for a currency.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CurrencyResponse {
    /// Currency code (ISO 4217).
    #[schema(example = "USD")]
    pub code: String,
    /// Currency name.
    #[schema(example = "US Dollar")]
    pub name: String,
    /// Currency symbol.
    #[schema(example = "$")]
    pub symbol: String,
    /// Number of decimal places.
    #[schema(example = 2)]
    pub decimal_places: i16,
}

/// GET `/currencies` - List all currencies.
#[utoipa::path(
    get,
    path = "/currencies",
    responses(
        (status = 200, description = "List of currencies", body = [CurrencyResponse]),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Currencies",
    security(("bearerAuth" = []))
)]
async fn list_currencies(State(state): State<AppState>, _auth: AuthUser) -> impl IntoResponse {
    match currencies::Entity::find().all(&*state.db).await {
        Ok(currencies) => {
            let response: Vec<CurrencyResponse> = currencies
                .into_iter()
                .map(|c| CurrencyResponse {
                    code: c.code,
                    name: c.name,
                    symbol: c.symbol,
                    decimal_places: c.decimal_places,
                })
                .collect();

            (StatusCode::OK, Json(json!({ "currencies": response }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list currencies");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response()
        }
    }
}
