//! Attachment management routes.
//!
//! Implements Requirements 1.1-1.8 for attachment API endpoints.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};
use zeltra_core::attachment::{
    AttachmentRepository as AttachmentRepoTrait, AttachmentService, AttachmentType,
    ConfirmUploadInput, RequestUploadInput,
};
use zeltra_db::{OrganizationRepository, repositories::AttachmentRepository};

/// Creates the attachment routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        // Transaction-scoped attachment routes
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/attachments/upload",
            post(request_upload),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/attachments",
            post(confirm_upload),
        )
        .route(
            "/organizations/{org_id}/transactions/{transaction_id}/attachments",
            get(list_attachments),
        )
        // Direct attachment routes
        .route(
            "/organizations/{org_id}/attachments/{attachment_id}",
            get(get_attachment),
        )
        .route(
            "/organizations/{org_id}/attachments/{attachment_id}",
            delete(delete_attachment),
        )
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for requesting an upload URL.
#[derive(Debug, Deserialize)]
pub struct RequestUploadRequest {
    /// Original filename.
    pub filename: String,
    /// MIME type of the file.
    pub content_type: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Attachment type classification.
    #[serde(default)]
    pub attachment_type: Option<String>,
}

/// Response for upload URL request.
#[derive(Debug, Serialize)]
pub struct RequestUploadResponse {
    /// Generated attachment ID.
    pub attachment_id: Uuid,
    /// Presigned upload URL.
    pub upload_url: String,
    /// HTTP method to use (PUT).
    pub upload_method: String,
    /// Required headers for the upload.
    pub upload_headers: std::collections::HashMap<String, String>,
    /// When the URL expires (ISO 8601).
    pub expires_at: String,
    /// Storage key for confirmation.
    pub storage_key: String,
}

/// Request body for confirming an upload.
#[derive(Debug, Deserialize)]
pub struct ConfirmUploadRequest {
    /// Attachment ID from request_upload.
    pub attachment_id: Uuid,
    /// Original filename.
    pub filename: String,
    /// MIME type.
    pub content_type: String,
    /// File size in bytes.
    pub file_size: i64,
    /// Storage key from request_upload.
    pub storage_key: String,
    /// Attachment type classification.
    #[serde(default)]
    pub attachment_type: Option<String>,
}

/// Response for an attachment.
#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    /// Attachment ID.
    pub id: Uuid,
    /// Transaction ID.
    pub transaction_id: Option<Uuid>,
    /// Attachment type.
    pub attachment_type: String,
    /// Original filename.
    pub filename: String,
    /// File size in bytes.
    pub file_size: i64,
    /// MIME type.
    pub mime_type: String,
    /// Storage provider.
    pub storage_provider: String,
    /// Uploaded by user ID.
    pub uploaded_by: Uuid,
    /// Created at timestamp (ISO 8601).
    pub created_at: String,
    /// Download URL (presigned, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// Download URL expiration (ISO 8601, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url_expires_at: Option<String>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse attachment type from string.
fn parse_attachment_type(s: Option<&str>) -> AttachmentType {
    match s {
        Some("receipt") => AttachmentType::Receipt,
        Some("invoice") => AttachmentType::Invoice,
        Some("contract") => AttachmentType::Contract,
        Some("supporting_document") => AttachmentType::SupportingDocument,
        _ => AttachmentType::Other,
    }
}

/// Convert attachment type to string.
fn attachment_type_to_string(t: AttachmentType) -> &'static str {
    match t {
        AttachmentType::Receipt => "receipt",
        AttachmentType::Invoice => "invoice",
        AttachmentType::Contract => "contract",
        AttachmentType::SupportingDocument => "supporting_document",
        AttachmentType::Other => "other",
    }
}

/// Check if user is a member of the organization.
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
            error!(error = %e, "Failed to check membership");
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

// ============================================================================
// Route Handlers
// ============================================================================

/// POST `/organizations/{org_id}/transactions/{transaction_id}/attachments/upload`
/// Request a presigned upload URL.
///
/// Requirements: 1.1
async fn request_upload(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<RequestUploadRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Check if storage service is available
    let Some(storage) = &state.storage else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "storage_not_configured",
                "message": "File storage is not configured"
            })),
        )
            .into_response();
    };

    let attachment_repo = AttachmentRepository::new((*state.db).clone());
    let service = AttachmentService::new(storage.clone(), std::sync::Arc::new(attachment_repo));

    let input = RequestUploadInput {
        organization_id: org_id,
        transaction_id,
        filename: payload.filename,
        content_type: payload.content_type,
        file_size: payload.file_size,
        attachment_type: parse_attachment_type(payload.attachment_type.as_deref()),
        user_id: auth.user_id(),
    };

    match service.request_upload(input).await {
        Ok(result) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                attachment_id = %result.attachment_id,
                "Upload URL requested"
            );

            let response = RequestUploadResponse {
                attachment_id: result.attachment_id,
                upload_url: result.upload_url,
                upload_method: result.upload_method,
                upload_headers: result.upload_headers,
                expires_at: result.expires_at.to_rfc3339(),
                storage_key: result.storage_key,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to request upload URL");
            match e {
                zeltra_core::attachment::AttachmentError::TransactionNotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "transaction_not_found",
                        "message": "Transaction not found"
                    })),
                )
                    .into_response(),
                zeltra_core::attachment::AttachmentError::Storage(storage_err) => {
                    // Check for specific storage errors
                    let msg = storage_err.to_string();
                    if msg.contains("too large") {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "error": "file_too_large",
                                "message": msg
                            })),
                        )
                            .into_response()
                    } else if msg.contains("MIME type") {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "error": "invalid_mime_type",
                                "message": msg
                            })),
                        )
                            .into_response()
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({
                                "error": "storage_error",
                                "message": "Storage operation failed"
                            })),
                        )
                            .into_response()
                    }
                }
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

/// POST `/organizations/{org_id}/transactions/{transaction_id}/attachments`
/// Confirm an upload and create the attachment record.
///
/// Requirements: 1.2
async fn confirm_upload(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ConfirmUploadRequest>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Check if storage service is available
    let Some(storage) = &state.storage else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "storage_not_configured",
                "message": "File storage is not configured"
            })),
        )
            .into_response();
    };

    let attachment_repo = AttachmentRepository::new((*state.db).clone());
    let service = AttachmentService::new(storage.clone(), std::sync::Arc::new(attachment_repo));

    let input = ConfirmUploadInput {
        attachment_id: payload.attachment_id,
        organization_id: org_id,
        transaction_id,
        filename: payload.filename,
        content_type: payload.content_type,
        file_size: payload.file_size,
        storage_key: payload.storage_key,
        attachment_type: parse_attachment_type(payload.attachment_type.as_deref()),
        uploaded_by: auth.user_id(),
    };

    match service.confirm_upload(input).await {
        Ok(attachment) => {
            info!(
                org_id = %org_id,
                transaction_id = %transaction_id,
                attachment_id = %attachment.id,
                "Attachment confirmed"
            );

            let response = AttachmentResponse {
                id: attachment.id,
                transaction_id: attachment.transaction_id,
                attachment_type: attachment_type_to_string(attachment.attachment_type).to_string(),
                filename: attachment.filename,
                file_size: attachment.file_size,
                mime_type: attachment.mime_type,
                storage_provider: attachment.storage_provider,
                uploaded_by: attachment.uploaded_by,
                created_at: attachment.created_at.to_rfc3339(),
                download_url: None,
                download_url_expires_at: None,
            };

            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to confirm upload");
            match e {
                zeltra_core::attachment::AttachmentError::UploadNotVerified => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "upload_not_verified",
                        "message": "File not found in storage. Please upload the file first."
                    })),
                )
                    .into_response(),
                zeltra_core::attachment::AttachmentError::FileSizeMismatch { expected, actual } => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "file_size_mismatch",
                        "message": format!("File size mismatch. Expected: {}, Actual: {}", expected, actual)
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

/// GET `/organizations/{org_id}/transactions/{transaction_id}/attachments`
/// List attachments for a transaction.
///
/// Requirements: 1.4
async fn list_attachments(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    let attachment_repo = AttachmentRepository::new((*state.db).clone());

    match attachment_repo
        .list_by_transaction(transaction_id, org_id)
        .await
    {
        Ok(attachments) => {
            let items: Vec<AttachmentResponse> = attachments
                .into_iter()
                .map(|a| AttachmentResponse {
                    id: a.id,
                    transaction_id: a.transaction_id,
                    attachment_type: attachment_type_to_string(a.attachment_type).to_string(),
                    filename: a.filename,
                    file_size: a.file_size,
                    mime_type: a.mime_type,
                    storage_provider: a.storage_provider,
                    uploaded_by: a.uploaded_by,
                    created_at: a.created_at.to_rfc3339(),
                    download_url: None,
                    download_url_expires_at: None,
                })
                .collect();

            (StatusCode::OK, Json(json!({ "attachments": items }))).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list attachments");
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

/// GET `/organizations/{org_id}/attachments/{attachment_id}`
/// Get attachment with download URL.
///
/// Requirements: 1.3
async fn get_attachment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, attachment_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Check if storage service is available
    let Some(storage) = &state.storage else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "storage_not_configured",
                "message": "File storage is not configured"
            })),
        )
            .into_response();
    };

    let attachment_repo = AttachmentRepository::new((*state.db).clone());
    let service = AttachmentService::new(storage.clone(), std::sync::Arc::new(attachment_repo));

    // Get attachment
    let attachment = match service.get_by_id(attachment_id, org_id).await {
        Ok(a) => a,
        Err(zeltra_core::attachment::AttachmentError::NotFound(_)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Attachment not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to get attachment");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "An error occurred"
                })),
            )
                .into_response();
        }
    };

    // Get download URL
    let (download_url, download_url_expires_at) =
        match service.get_download_url(attachment_id, org_id).await {
            Ok(presigned) => (Some(presigned.url), Some(presigned.expires_at.to_rfc3339())),
            Err(e) => {
                error!(error = %e, "Failed to generate download URL");
                (None, None)
            }
        };

    let response = AttachmentResponse {
        id: attachment.id,
        transaction_id: attachment.transaction_id,
        attachment_type: attachment_type_to_string(attachment.attachment_type).to_string(),
        filename: attachment.filename,
        file_size: attachment.file_size,
        mime_type: attachment.mime_type,
        storage_provider: attachment.storage_provider,
        uploaded_by: attachment.uploaded_by,
        created_at: attachment.created_at.to_rfc3339(),
        download_url,
        download_url_expires_at,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// DELETE `/organizations/{org_id}/attachments/{attachment_id}`
/// Delete an attachment.
///
/// Requirements: 1.4
async fn delete_attachment(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((org_id, attachment_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let org_repo = OrganizationRepository::new((*state.db).clone());

    // Check membership
    if let Err(response) = check_membership(&org_repo, org_id, auth.user_id()).await {
        return response;
    }

    // Check if storage service is available
    let Some(storage) = &state.storage else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "storage_not_configured",
                "message": "File storage is not configured"
            })),
        )
            .into_response();
    };

    let attachment_repo = AttachmentRepository::new((*state.db).clone());
    let service = AttachmentService::new(storage.clone(), std::sync::Arc::new(attachment_repo));

    match service.delete(attachment_id, org_id).await {
        Ok(()) => {
            info!(
                org_id = %org_id,
                attachment_id = %attachment_id,
                "Attachment deleted"
            );

            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(zeltra_core::attachment::AttachmentError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "Attachment not found"
            })),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to delete attachment");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_attachment_type() {
        assert_eq!(
            parse_attachment_type(Some("receipt")),
            AttachmentType::Receipt
        );
        assert_eq!(
            parse_attachment_type(Some("invoice")),
            AttachmentType::Invoice
        );
        assert_eq!(
            parse_attachment_type(Some("contract")),
            AttachmentType::Contract
        );
        assert_eq!(
            parse_attachment_type(Some("supporting_document")),
            AttachmentType::SupportingDocument
        );
        assert_eq!(parse_attachment_type(Some("other")), AttachmentType::Other);
        assert_eq!(
            parse_attachment_type(Some("unknown")),
            AttachmentType::Other
        );
        assert_eq!(parse_attachment_type(None), AttachmentType::Other);
    }

    #[test]
    fn test_attachment_type_to_string() {
        assert_eq!(
            attachment_type_to_string(AttachmentType::Receipt),
            "receipt"
        );
        assert_eq!(
            attachment_type_to_string(AttachmentType::Invoice),
            "invoice"
        );
        assert_eq!(
            attachment_type_to_string(AttachmentType::Contract),
            "contract"
        );
        assert_eq!(
            attachment_type_to_string(AttachmentType::SupportingDocument),
            "supporting_document"
        );
        assert_eq!(attachment_type_to_string(AttachmentType::Other), "other");
    }
}

/// Integration tests that require a real database connection.
/// Run with: cargo test -p zeltra-api --test '*' -- --ignored
/// Or set DATABASE_URL env var and run: cargo test -p zeltra-api attachments::integration_tests
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, header::AUTHORIZATION},
        middleware::from_fn_with_state,
    };
    use http_body_util::BodyExt;
    use sea_orm::{Database, DatabaseConnection};
    use std::sync::Arc;
    use tower::ServiceExt;
    use zeltra_core::storage::{StorageConfig, StorageProvider, StorageService};
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

    /// Helper to create a test AppState with real DB but no storage.
    async fn create_test_state_with_db() -> AppState {
        let db_url = get_database_url();
        let db = Database::connect(&db_url)
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

    /// Helper to create a test AppState with real DB and local storage.
    async fn create_test_state_with_db_and_storage() -> AppState {
        let db_url = get_database_url();
        let db = Database::connect(&db_url)
            .await
            .expect("Failed to connect to database");
        let jwt_service = JwtService::new(JwtConfig::default());
        let email_service = EmailService::new(EmailConfig::default());

        let storage_config = StorageConfig::new(StorageProvider::local_fs("./test_uploads"));
        let storage = StorageService::from_config(storage_config)
            .ok()
            .map(Arc::new);

        AppState {
            db: Arc::new(db),
            jwt_service: Arc::new(jwt_service),
            email_service: Arc::new(email_service),
            storage,
        }
    }

    /// Helper to create a valid auth token for a specific user/org.
    fn create_auth_token(state: &AppState, user_id: Uuid, org_id: Uuid) -> String {
        state
            .jwt_service
            .generate_access_token(user_id, org_id, "admin")
            .expect("should generate token")
    }

    #[tokio::test]
    async fn test_list_attachments_no_auth() {
        let state = create_test_state_with_db().await;

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let org_id = Uuid::new_v4();
        let tx_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/organizations/{}/transactions/{}/attachments",
                        org_id, tx_id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_request_upload_no_storage_returns_503() {
        let state = create_test_state_with_db().await;

        // Get a real org and user from DB
        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let tx_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/organizations/{}/transactions/{}/attachments/upload",
                        org_user.org_id, tx_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"filename":"test.pdf","content_type":"application/pdf","file_size":1024}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_attachment_no_storage_returns_503() {
        let state = create_test_state_with_db().await;

        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let attachment_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/organizations/{}/attachments/{}",
                        org_user.org_id, attachment_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_delete_attachment_no_storage_returns_503() {
        let state = create_test_state_with_db().await;

        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let attachment_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/organizations/{}/attachments/{}",
                        org_user.org_id, attachment_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_list_attachments_non_member_returns_403() {
        let state = create_test_state_with_db().await;

        // Get a real org from DB
        let org_user = get_test_org_and_user(&state.db).await;

        // Create token for a random user that is NOT a member of the org
        let random_user_id = Uuid::new_v4();
        let token = create_auth_token(&state, random_user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let tx_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/organizations/{}/transactions/{}/attachments",
                        org_user.org_id, tx_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_list_attachments_empty_list() {
        let state = create_test_state_with_db().await;

        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        // Use a random transaction ID that doesn't exist
        let tx_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/organizations/{}/transactions/{}/attachments",
                        org_user.org_id, tx_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["attachments"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_request_upload_with_storage_transaction_not_found() {
        let state = create_test_state_with_db_and_storage().await;

        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        // Use a random transaction ID that doesn't exist
        let tx_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/organizations/{}/transactions/{}/attachments/upload",
                        org_user.org_id, tx_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"filename":"test.pdf","content_type":"application/pdf","file_size":1024}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "transaction_not_found");
    }

    #[tokio::test]
    async fn test_get_attachment_not_found() {
        let state = create_test_state_with_db_and_storage().await;

        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let attachment_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/organizations/{}/attachments/{}",
                        org_user.org_id, attachment_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_attachment_not_found() {
        let state = create_test_state_with_db_and_storage().await;

        let org_user = get_test_org_and_user(&state.db).await;
        let token = create_auth_token(&state, org_user.user_id, org_user.org_id);

        let app = Router::new()
            .merge(routes())
            .layer(from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state);

        let attachment_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/organizations/{}/attachments/{}",
                        org_user.org_id, attachment_id
                    ))
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Test data helper struct.
    struct TestOrgUser {
        org_id: Uuid,
        user_id: Uuid,
    }

    /// Get or create a test organization and user from the database.
    async fn get_test_org_and_user(db: &DatabaseConnection) -> TestOrgUser {
        use sea_orm::{EntityTrait, QueryOrder};
        use zeltra_db::entities::{organization_users, organizations, users};

        // Try to get existing org_user relationship
        let org_user = organization_users::Entity::find()
            .order_by_asc(organization_users::Column::CreatedAt)
            .one(db)
            .await
            .expect("Failed to query organization_users");

        if let Some(ou) = org_user {
            return TestOrgUser {
                org_id: ou.organization_id,
                user_id: ou.user_id,
            };
        }

        // If no org_user exists, get first org and first user
        let org = organizations::Entity::find()
            .one(db)
            .await
            .expect("Failed to query organizations")
            .expect("No organizations in database - please seed test data");

        let user = users::Entity::find()
            .one(db)
            .await
            .expect("Failed to query users")
            .expect("No users in database - please seed test data");

        TestOrgUser {
            org_id: org.id,
            user_id: user.id,
        }
    }
}
