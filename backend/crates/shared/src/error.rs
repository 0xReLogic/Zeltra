//! Application-wide error types.

use thiserror::Error;

/// Result type alias using `AppError`.
pub type AppResult<T> = Result<T, AppError>;

/// Application error types.
#[derive(Debug, Error)]
pub enum AppError {
    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    /// Access denied.
    #[error("Access denied: {0}")]
    Forbidden(String),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Business rule violation.
    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    /// Conflict (e.g., duplicate entry).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// External service error.
    #[error("External service error: {0}")]
    ExternalService(String),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Returns the HTTP status code for this error.
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Validation(_) => 400,
            Self::BusinessRule(_) => 422,
            Self::Conflict(_) => 409,
            Self::Database(_) | Self::ExternalService(_) | Self::Internal(_) => 500,
        }
    }

    /// Returns the error code for API responses.
    #[must_use]
    pub const fn error_code(&self) -> &'static str {
        match self {
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::BusinessRule(_) => "BUSINESS_RULE_VIOLATION",
            Self::Conflict(_) => "CONFLICT",
            Self::Database(_) => "DATABASE_ERROR",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(AppError::Unauthorized(String::new()).status_code(), 401);
        assert_eq!(AppError::Forbidden(String::new()).status_code(), 403);
        assert_eq!(AppError::NotFound(String::new()).status_code(), 404);
        assert_eq!(AppError::Validation(String::new()).status_code(), 400);
        assert_eq!(AppError::BusinessRule(String::new()).status_code(), 422);
        assert_eq!(AppError::Conflict(String::new()).status_code(), 409);
        assert_eq!(AppError::Database(String::new()).status_code(), 500);
        assert_eq!(AppError::ExternalService(String::new()).status_code(), 500);
        assert_eq!(AppError::Internal(String::new()).status_code(), 500);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            AppError::Unauthorized(String::new()).error_code(),
            "UNAUTHORIZED"
        );
        assert_eq!(AppError::Forbidden(String::new()).error_code(), "FORBIDDEN");
        assert_eq!(AppError::NotFound(String::new()).error_code(), "NOT_FOUND");
        assert_eq!(
            AppError::Validation(String::new()).error_code(),
            "VALIDATION_ERROR"
        );
        assert_eq!(
            AppError::BusinessRule(String::new()).error_code(),
            "BUSINESS_RULE_VIOLATION"
        );
        assert_eq!(AppError::Conflict(String::new()).error_code(), "CONFLICT");
        assert_eq!(
            AppError::Database(String::new()).error_code(),
            "DATABASE_ERROR"
        );
        assert_eq!(
            AppError::ExternalService(String::new()).error_code(),
            "EXTERNAL_SERVICE_ERROR"
        );
        assert_eq!(
            AppError::Internal(String::new()).error_code(),
            "INTERNAL_ERROR"
        );
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            AppError::Unauthorized("msg".into()).to_string(),
            "Authentication failed: msg"
        );
        assert_eq!(
            AppError::Forbidden("msg".into()).to_string(),
            "Access denied: msg"
        );
        assert_eq!(
            AppError::NotFound("msg".into()).to_string(),
            "Not found: msg"
        );
        assert_eq!(
            AppError::Validation("msg".into()).to_string(),
            "Validation error: msg"
        );
        assert_eq!(
            AppError::BusinessRule("msg".into()).to_string(),
            "Business rule violation: msg"
        );
        assert_eq!(
            AppError::Conflict("msg".into()).to_string(),
            "Conflict: msg"
        );
        assert_eq!(
            AppError::Database("msg".into()).to_string(),
            "Database error: msg"
        );
        assert_eq!(
            AppError::ExternalService("msg".into()).to_string(),
            "External service error: msg"
        );
        assert_eq!(
            AppError::Internal("msg".into()).to_string(),
            "Internal error: msg"
        );
    }
}
