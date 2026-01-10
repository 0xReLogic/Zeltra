use super::*;

#[test]
fn test_app_error_status_codes() {
    assert_eq!(AppError::Unauthorized("test".into()).status_code(), 401);
    assert_eq!(AppError::Forbidden("test".into()).status_code(), 403);
    assert_eq!(AppError::NotFound("test".into()).status_code(), 404);
    assert_eq!(AppError::Validation("test".into()).status_code(), 400);
    assert_eq!(AppError::BusinessRule("test".into()).status_code(), 422);
    assert_eq!(AppError::Conflict("test".into()).status_code(), 409);
    assert_eq!(AppError::Database("test".into()).status_code(), 500);
    assert_eq!(AppError::ExternalService("test".into()).status_code(), 500);
    assert_eq!(AppError::Internal("test".into()).status_code(), 500);
}

#[test]
fn test_app_error_error_codes() {
    assert_eq!(
        AppError::Unauthorized("test".into()).error_code(),
        "UNAUTHORIZED"
    );
    assert_eq!(AppError::Forbidden("test".into()).error_code(), "FORBIDDEN");
    assert_eq!(AppError::NotFound("test".into()).error_code(), "NOT_FOUND");
    assert_eq!(
        AppError::Validation("test".into()).error_code(),
        "VALIDATION_ERROR"
    );
    assert_eq!(
        AppError::BusinessRule("test".into()).error_code(),
        "BUSINESS_RULE_VIOLATION"
    );
    assert_eq!(AppError::Conflict("test".into()).error_code(), "CONFLICT");
    assert_eq!(
        AppError::Database("test".into()).error_code(),
        "DATABASE_ERROR"
    );
    assert_eq!(
        AppError::ExternalService("test".into()).error_code(),
        "EXTERNAL_SERVICE_ERROR"
    );
    assert_eq!(
        AppError::Internal("test".into()).error_code(),
        "INTERNAL_ERROR"
    );
}

#[test]
fn test_app_error_display() {
    assert_eq!(
        format!("{}", AppError::Unauthorized("msg".into())),
        "Authentication failed: msg"
    );
    assert_eq!(
        format!("{}", AppError::Forbidden("msg".into())),
        "Access denied: msg"
    );
    assert_eq!(
        format!("{}", AppError::NotFound("msg".into())),
        "Not found: msg"
    );
    assert_eq!(
        format!("{}", AppError::Validation("msg".into())),
        "Validation error: msg"
    );
    assert_eq!(
        format!("{}", AppError::BusinessRule("msg".into())),
        "Business rule violation: msg"
    );
    assert_eq!(
        format!("{}", AppError::Conflict("msg".into())),
        "Conflict: msg"
    );
    assert_eq!(
        format!("{}", AppError::Database("msg".into())),
        "Database error: msg"
    );
    assert_eq!(
        format!("{}", AppError::ExternalService("msg".into())),
        "External service error: msg"
    );
    assert_eq!(
        format!("{}", AppError::Internal("msg".into())),
        "Internal error: msg"
    );
}
