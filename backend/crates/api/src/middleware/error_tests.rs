//! Property tests for error handling.
//!
//! **Property 15: Error Response Consistency**
//! **Property 16: Auth Error Response**
//! **Validates: Requirements 5.1, 5.4, 5.5**

use crate::error::ApiError;
use axum::http::StatusCode;
use proptest::prelude::*;

// ============================================================================
// Property 15: Error Response Consistency
// For any error type, the response must contain error code and message fields
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 15: All ApiError instances have non-empty error code and message
    #[test]
    fn prop_error_response_has_required_fields(
        error_code in "[a-z_]{3,20}",
        message in ".{5,100}"
    ) {
        let error = ApiError::new(&error_code, &message);

        // Error code must be non-empty
        prop_assert!(!error.error.is_empty(), "Error code must not be empty");

        // Message must be non-empty
        prop_assert!(!error.message.is_empty(), "Message must not be empty");

        // Error code should match what we passed
        prop_assert_eq!(error.error, error_code);

        // Message should match what we passed
        prop_assert_eq!(error.message, message);
    }

    /// Property 15: Error constructors return correct status codes
    #[test]
    fn prop_error_constructors_return_correct_status(
        message in ".{5,50}"
    ) {
        // Validation error -> 400
        let (status, _) = ApiError::validation(&message);
        prop_assert_eq!(status, StatusCode::BAD_REQUEST);

        // Unauthorized -> 401
        let (status, _) = ApiError::unauthorized(&message);
        prop_assert_eq!(status, StatusCode::UNAUTHORIZED);

        // Forbidden -> 403
        let (status, _) = ApiError::forbidden(&message);
        prop_assert_eq!(status, StatusCode::FORBIDDEN);

        // Internal -> 500
        let (status, _) = ApiError::internal(&message);
        prop_assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    /// Property 15: request_id is properly attached when provided
    #[test]
    fn prop_request_id_attached(
        request_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
    ) {
        let error = ApiError::new("test", "test message")
            .with_request_id(&request_id);

        prop_assert!(error.request_id.is_some());
        prop_assert_eq!(error.request_id.unwrap(), request_id);
    }
}

// ============================================================================
// Property 16: Auth Error Response
// For any auth error, response must be 401 for invalid token, 403 for insufficient role
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 16: Invalid token always returns 401
    #[test]
    fn prop_invalid_token_returns_401(_dummy in 0..100u32) {
        let (status, error) = ApiError::invalid_token();

        prop_assert_eq!(status, StatusCode::UNAUTHORIZED);
        prop_assert_eq!(error.error, "invalid_token");
    }

    /// Property 16: Insufficient role always returns 403 with role in message
    #[test]
    fn prop_insufficient_role_returns_403(
        role in "(admin|accountant|viewer|editor|manager)"
    ) {
        let (status, error) = ApiError::insufficient_role(&role);

        prop_assert_eq!(status, StatusCode::FORBIDDEN);
        prop_assert_eq!(error.error, "insufficient_permissions");
        prop_assert!(
            error.message.contains(&role),
            "Message should contain the required role"
        );
    }

    /// Property 16: Rate limited returns 429 with retry_after in details
    #[test]
    fn prop_rate_limited_returns_429_with_retry(
        retry_after in 1u64..3600
    ) {
        let (status, error) = ApiError::rate_limited(retry_after);

        prop_assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        prop_assert_eq!(error.error, "rate_limited");
        prop_assert!(error.details.is_some());

        let details = error.details.unwrap();
        prop_assert_eq!(
            details.get("retry_after").and_then(serde_json::Value::as_u64),
            Some(retry_after)
        );
    }
}

// ============================================================================
// Unit Tests for specific error scenarios
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_not_found_error_format() {
        let (status, error) = ApiError::not_found("Account");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(error.error, "not_found");
        assert_eq!(error.message, "Account not found");
    }

    #[test]
    fn test_entity_not_found_includes_id() {
        let id = Uuid::new_v4();
        let (status, error) = ApiError::entity_not_found("Transaction", id);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(error.message.contains(&id.to_string()));
    }

    #[test]
    fn test_conflict_error() {
        let (status, error) = ApiError::conflict("Email already exists");
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(error.error, "conflict");
    }

    #[test]
    fn test_service_unavailable() {
        let (status, error) = ApiError::service_unavailable("Database connection failed");
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(error.error, "service_unavailable");
    }

    #[test]
    fn test_error_with_details() {
        let error =
            ApiError::new("validation_error", "Invalid input").with_details(serde_json::json!({
                "field": "email",
                "reason": "invalid format"
            }));

        assert!(error.details.is_some());
        let details = error.details.unwrap();
        assert_eq!(details.get("field").unwrap(), "email");
    }
}
