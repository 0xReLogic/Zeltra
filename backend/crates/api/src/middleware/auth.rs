//! Authentication middleware for protected routes.

use axum::{
    Json,
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::AppState;
use zeltra_shared::Claims;

/// Extracts the bearer token from the Authorization header.
fn extract_bearer_token(header: &str) -> Option<&str> {
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
}

/// Authentication middleware that validates JWT tokens.
///
/// This middleware:
/// 1. Extracts the Bearer token from the Authorization header
/// 2. Validates the token using the JWT service
/// 3. Stores the claims in request extensions for handlers to access
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let Some(token) = auth_header.and_then(extract_bearer_token) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "missing_token",
                "message": "Authorization header with Bearer token is required"
            })),
        )
            .into_response();
    };

    // Validate token
    match state.jwt_service.validate_token(token) {
        Ok(claims) => {
            // Store claims in request extensions
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(e) => {
            let (status, error, message) = match e {
                zeltra_shared::JwtError::Expired => (
                    StatusCode::UNAUTHORIZED,
                    "token_expired",
                    "Token has expired",
                ),
                _ => (
                    StatusCode::UNAUTHORIZED,
                    "invalid_token",
                    "Invalid or malformed token",
                ),
            };

            (status, Json(json!({ "error": error, "message": message }))).into_response()
        }
    }
}

/// Extractor for authenticated user claims.
///
/// Use this in handlers to get the authenticated user's claims:
///
/// ```ignore
/// async fn handler(claims: AuthUser) -> impl IntoResponse {
///     let user_id = claims.user_id();
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

impl AuthUser {
    /// Returns the user ID from the claims.
    #[must_use]
    pub fn user_id(&self) -> uuid::Uuid {
        self.0.user_id()
    }

    /// Returns the organization ID from the claims.
    #[must_use]
    pub fn organization_id(&self) -> uuid::Uuid {
        self.0.organization_id()
    }

    /// Returns the user's role.
    #[must_use]
    pub fn role(&self) -> &str {
        &self.0.role
    }

    /// Returns the inner claims.
    #[must_use]
    pub fn claims(&self) -> &Claims {
        &self.0
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthUser)
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "unauthorized",
                        "message": "Authentication required"
                    })),
                )
            })
    }
}
