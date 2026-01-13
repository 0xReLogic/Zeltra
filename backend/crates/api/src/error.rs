//! Standardized API error responses.
//!
//! Implements Requirements 5.1 for consistent error handling.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

/// Standard API error response.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiError {
    /// Error code (e.g., "validation_error", "not_found").
    #[schema(example = "not_found")]
    pub error: String,
    /// Human-readable error message.
    #[schema(example = "Resource not found")]
    pub message: String,
    /// Request ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// Creates a new API error.
    #[must_use]
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            request_id: None,
            details: None,
        }
    }

    /// Adds a request ID to the error.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Adds details to the error.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    // ========================================================================
    // Common Error Constructors
    // ========================================================================

    /// 400 Bad Request - Validation error.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> (StatusCode, Self) {
        (
            StatusCode::BAD_REQUEST,
            Self::new("validation_error", message),
        )
    }

    /// 400 Bad Request - Invalid input.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::BAD_REQUEST, Self::new("bad_request", message))
    }

    /// 401 Unauthorized - Invalid or missing authentication.
    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::UNAUTHORIZED, Self::new("unauthorized", message))
    }

    /// 401 Unauthorized - Invalid JWT token.
    #[must_use]
    pub fn invalid_token() -> (StatusCode, Self) {
        (
            StatusCode::UNAUTHORIZED,
            Self::new("invalid_token", "Invalid or expired authentication token"),
        )
    }

    /// 403 Forbidden - Insufficient permissions.
    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::FORBIDDEN, Self::new("forbidden", message))
    }

    /// 403 Forbidden - Insufficient role.
    #[must_use]
    pub fn insufficient_role(required_role: &str) -> (StatusCode, Self) {
        (
            StatusCode::FORBIDDEN,
            Self::new(
                "insufficient_permissions",
                format!("This action requires the '{required_role}' role"),
            ),
        )
    }

    /// 404 Not Found - Resource not found.
    #[must_use]
    pub fn not_found(resource: &str) -> (StatusCode, Self) {
        (
            StatusCode::NOT_FOUND,
            Self::new("not_found", format!("{resource} not found")),
        )
    }

    /// 404 Not Found - Entity not found by ID.
    #[must_use]
    pub fn entity_not_found(entity: &str, id: Uuid) -> (StatusCode, Self) {
        (
            StatusCode::NOT_FOUND,
            Self::new("not_found", format!("{entity} with ID {id} not found")),
        )
    }

    /// 409 Conflict - Resource already exists.
    #[must_use]
    pub fn conflict(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::CONFLICT, Self::new("conflict", message))
    }

    /// 422 Unprocessable Entity - Business logic error.
    #[must_use]
    pub fn unprocessable(message: impl Into<String>) -> (StatusCode, Self) {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Self::new("unprocessable_entity", message),
        )
    }

    /// 429 Too Many Requests - Rate limit exceeded.
    #[must_use]
    pub fn rate_limited(retry_after_secs: u64) -> (StatusCode, Self) {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Self::new(
                "rate_limited",
                format!("Too many requests. Please retry after {retry_after_secs} seconds"),
            )
            .with_details(serde_json::json!({ "retry_after": retry_after_secs })),
        )
    }

    /// 500 Internal Server Error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> (StatusCode, Self) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Self::new("internal_error", message),
        )
    }

    /// 500 Internal Server Error - Generic.
    #[must_use]
    pub fn internal_error() -> (StatusCode, Self) {
        Self::internal("An unexpected error occurred")
    }

    /// 503 Service Unavailable.
    #[must_use]
    pub fn service_unavailable(message: impl Into<String>) -> (StatusCode, Self) {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Self::new("service_unavailable", message),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

/// API error with status code.
pub struct ApiErrorResponse {
    /// HTTP status code.
    pub status: StatusCode,
    /// Error body.
    pub error: ApiError,
}

impl ApiErrorResponse {
    /// Creates a new API error response.
    #[must_use]
    pub const fn new(status: StatusCode, error: ApiError) -> Self {
        Self { status, error }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.error)).into_response()
    }
}

impl From<(StatusCode, ApiError)> for ApiErrorResponse {
    fn from((status, error): (StatusCode, ApiError)) -> Self {
        Self { status, error }
    }
}

/// Extension trait for adding request_id to errors.
pub trait WithRequestId {
    /// Adds request ID to the error response.
    #[must_use]
    fn with_request_id(self, request_id: impl Into<String>) -> Self;
}

impl WithRequestId for (StatusCode, ApiError) {
    fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.1.request_id = Some(request_id.into());
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let (status, error) = ApiError::validation("Invalid email format");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error.error, "validation_error");
        assert_eq!(error.message, "Invalid email format");
    }

    #[test]
    fn test_not_found_error() {
        let (status, error) = ApiError::not_found("User");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(error.error, "not_found");
        assert_eq!(error.message, "User not found");
    }

    #[test]
    fn test_entity_not_found() {
        let id = Uuid::new_v4();
        let (status, error) = ApiError::entity_not_found("Account", id);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(error.message.contains(&id.to_string()));
    }

    #[test]
    fn test_insufficient_role() {
        let (status, error) = ApiError::insufficient_role("admin");
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(error.error, "insufficient_permissions");
        assert!(error.message.contains("admin"));
    }

    #[test]
    fn test_rate_limited() {
        let (status, error) = ApiError::rate_limited(60);
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(error.error, "rate_limited");
        assert!(error.details.is_some());
    }

    #[test]
    fn test_with_request_id() {
        let (status, error) = ApiError::internal_error().with_request_id("req-123");
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_with_details() {
        let error = ApiError::new("test", "test message")
            .with_details(serde_json::json!({"field": "email"}));
        assert!(error.details.is_some());
    }
}
