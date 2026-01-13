//! Authentication middleware for protected routes.

use axum::{
    Json,
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::AppState;
use crate::error::ApiError;
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
        let (status, error) =
            ApiError::unauthorized("Authorization header with Bearer token is required");
        return (status, Json(error)).into_response();
    };

    // Validate token
    match state.jwt_service.validate_token(token) {
        Ok(claims) => {
            // Store claims in request extensions
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(e) => {
            let error = match e {
                zeltra_shared::JwtError::Expired => ApiError::new(
                    "token_expired",
                    "Token has expired. Please refresh your authentication.",
                ),
                zeltra_shared::JwtError::Invalid => {
                    ApiError::new("invalid_token", "Token is invalid or malformed.")
                }
                zeltra_shared::JwtError::DecodingError(_) => ApiError::new(
                    "invalid_token",
                    "Invalid or malformed authentication token.",
                ),
                zeltra_shared::JwtError::EncodingError(_) => {
                    ApiError::new("internal_error", "An unexpected error occurred.")
                }
            };

            (StatusCode::UNAUTHORIZED, Json(error)).into_response()
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

    /// Checks if the user has the admin role.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.0.role == "admin"
    }

    /// Checks if the user has at least the specified role level.
    /// Role hierarchy: admin > accountant > viewer
    #[must_use]
    pub fn has_role(&self, required_role: &str) -> bool {
        match required_role {
            "viewer" => true, // Everyone has at least viewer access
            "accountant" => self.0.role == "accountant" || self.0.role == "admin",
            "admin" => self.0.role == "admin",
            _ => false,
        }
    }

    /// Requires the user to have at least the specified role.
    /// Returns an error response if the user doesn't have sufficient permissions.
    pub fn require_role(&self, required_role: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
        if self.has_role(required_role) {
            Ok(())
        } else {
            let (status, error) = ApiError::insufficient_role(required_role);
            Err((status, Json(error)))
        }
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
    type Rejection = (StatusCode, Json<ApiError>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthUser)
            .ok_or_else(|| {
                let (status, error) = ApiError::unauthorized("Authentication required");
                (status, Json(error))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, middleware::from_fn_with_state};
    use sea_orm::DatabaseConnection;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;
    use zeltra_shared::{EmailConfig, EmailService, JwtConfig, JwtService};

    // Helper to create a test AppState
    fn create_test_state() -> AppState {
        // Use Disconnected variant since we don't need DB for auth middleware tests
        let db = DatabaseConnection::Disconnected;
        let jwt_service = JwtService::new(JwtConfig::default());
        let email_service = EmailService::new(EmailConfig::default());

        AppState {
            db: Arc::new(db),
            jwt_service: Arc::new(jwt_service),
            email_service: Arc::new(email_service),
            storage: None,
        }
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer token123"), Some("token123"));
        assert_eq!(extract_bearer_token("bearer token123"), Some("token123"));
        assert_eq!(extract_bearer_token("Basic token123"), None);
        assert_eq!(extract_bearer_token("Token token123"), None);
        assert_eq!(extract_bearer_token(""), None);
    }

    #[test]
    fn test_auth_user() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            org_id,
            "admin",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let auth_user = AuthUser(claims.clone());

        assert_eq!(auth_user.user_id(), user_id);
        assert_eq!(auth_user.organization_id(), org_id);
        assert_eq!(auth_user.role(), "admin");
        assert_eq!(auth_user.claims().user_id(), user_id);
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_token() {
        let state = create_test_state();
        let app = Router::new()
            .route("/", axum::routing::get(|| async { "OK" }))
            .layer(from_fn_with_state(state, auth_middleware));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_token() {
        let state = create_test_state();
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let token = state
            .jwt_service
            .generate_access_token(user_id, org_id, "user")
            .unwrap();

        let app = Router::new()
            .route(
                "/",
                axum::routing::get(|claims: AuthUser| async move {
                    assert_eq!(claims.role(), "user");
                    "OK"
                }),
            )
            .layer(from_fn_with_state(state, auth_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_token() {
        let state = create_test_state();
        let app = Router::new()
            .route("/", axum::routing::get(|| async { "OK" }))
            .layer(from_fn_with_state(state, auth_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(AUTHORIZATION, "Bearer invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_is_admin() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let admin_claims = Claims::new(
            user_id,
            org_id,
            "admin",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let admin_user = AuthUser(admin_claims);
        assert!(admin_user.is_admin());

        let viewer_claims = Claims::new(
            user_id,
            org_id,
            "viewer",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let viewer_user = AuthUser(viewer_claims);
        assert!(!viewer_user.is_admin());
    }

    #[test]
    fn test_has_role_hierarchy() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        // Admin has all roles
        let admin_claims = Claims::new(
            user_id,
            org_id,
            "admin",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let admin_user = AuthUser(admin_claims);
        assert!(admin_user.has_role("admin"));
        assert!(admin_user.has_role("accountant"));
        assert!(admin_user.has_role("viewer"));

        // Accountant has accountant and viewer
        let accountant_claims = Claims::new(
            user_id,
            org_id,
            "accountant",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let accountant_user = AuthUser(accountant_claims);
        assert!(!accountant_user.has_role("admin"));
        assert!(accountant_user.has_role("accountant"));
        assert!(accountant_user.has_role("viewer"));

        // Viewer only has viewer
        let viewer_claims = Claims::new(
            user_id,
            org_id,
            "viewer",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let viewer_user = AuthUser(viewer_claims);
        assert!(!viewer_user.has_role("admin"));
        assert!(!viewer_user.has_role("accountant"));
        assert!(viewer_user.has_role("viewer"));
    }

    #[test]
    fn test_require_role_success() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            org_id,
            "admin",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let auth_user = AuthUser(claims);

        assert!(auth_user.require_role("admin").is_ok());
        assert!(auth_user.require_role("accountant").is_ok());
        assert!(auth_user.require_role("viewer").is_ok());
    }

    #[test]
    fn test_require_role_failure() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            org_id,
            "viewer",
            chrono::Utc::now() + chrono::Duration::hours(1),
        );
        let auth_user = AuthUser(claims);

        let result = auth_user.require_role("admin");
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}
