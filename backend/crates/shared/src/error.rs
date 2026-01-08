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
