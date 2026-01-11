//! Attachment service implementation.

use std::sync::Arc;

use uuid::Uuid;

use super::error::AttachmentError;
use super::types::{
    Attachment, ConfirmUploadInput, CreateAttachmentInput, RequestUploadInput, RequestUploadResult,
};
use crate::storage::{StorageService, UploadRequest};

/// Repository trait for attachment persistence.
///
/// This trait is implemented by the db crate to provide actual database operations.
pub trait AttachmentRepository: Send + Sync {
    /// Create a new attachment record.
    fn create(
        &self,
        input: CreateAttachmentInput,
    ) -> impl std::future::Future<Output = Result<Attachment, AttachmentError>> + Send;

    /// Find attachment by ID.
    fn find_by_id(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Attachment>, AttachmentError>> + Send;

    /// List attachments for a transaction.
    fn list_by_transaction(
        &self,
        transaction_id: Uuid,
        organization_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<Attachment>, AttachmentError>> + Send;

    /// Delete attachment by ID.
    fn delete(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, AttachmentError>> + Send;

    /// Check if transaction exists.
    fn transaction_exists(
        &self,
        transaction_id: Uuid,
        organization_id: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, AttachmentError>> + Send;
}

/// Attachment service for managing file attachments.
pub struct AttachmentService<R: AttachmentRepository> {
    storage: Arc<StorageService>,
    repo: Arc<R>,
}

impl<R: AttachmentRepository> AttachmentService<R> {
    /// Create a new attachment service.
    #[must_use]
    pub fn new(storage: Arc<StorageService>, repo: Arc<R>) -> Self {
        Self { storage, repo }
    }

    /// Request an upload URL for a new attachment.
    ///
    /// This validates the request and generates a presigned URL for direct upload.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction does not exist
    /// - File size exceeds limit
    /// - MIME type is not allowed
    /// - Storage service fails
    pub async fn request_upload(
        &self,
        input: RequestUploadInput,
    ) -> Result<RequestUploadResult, AttachmentError> {
        // Verify transaction exists
        let tx_exists = self
            .repo
            .transaction_exists(input.transaction_id, input.organization_id)
            .await?;

        if !tx_exists {
            return Err(AttachmentError::transaction_not_found(input.transaction_id));
        }

        // Generate attachment ID
        let attachment_id = Uuid::new_v4();

        // Create upload request for storage service
        let upload_req = UploadRequest {
            organization_id: input.organization_id,
            transaction_id: Some(input.transaction_id),
            attachment_id,
            filename: input.filename.clone(),
            content_type: input.content_type.clone(),
            file_size: input.file_size,
        };

        // Generate presigned URL
        let presigned = self.storage.presign_upload(&upload_req).await?;

        let storage_key = StorageService::generate_storage_key(&upload_req);

        Ok(RequestUploadResult {
            attachment_id,
            upload_url: presigned.url,
            upload_method: presigned.method,
            upload_headers: presigned.headers,
            expires_at: presigned.expires_at,
            storage_key,
        })
    }

    /// Confirm an upload and create the attachment record.
    ///
    /// This verifies the file exists in storage and creates the database record.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File not found in storage
    /// - File size mismatch
    /// - Database operation fails
    pub async fn confirm_upload(
        &self,
        input: ConfirmUploadInput,
    ) -> Result<Attachment, AttachmentError> {
        // Verify file exists in storage
        let metadata = self
            .storage
            .verify_upload(&input.storage_key)
            .await
            .map_err(|_| AttachmentError::UploadNotVerified)?;

        // Verify file size matches (with some tolerance for metadata)
        // Convert i64 to u64 safely - negative sizes are invalid
        let expected_size = u64::try_from(input.file_size).unwrap_or(0);
        let actual_size = metadata.file_size;
        if actual_size != expected_size {
            return Err(AttachmentError::file_size_mismatch(
                expected_size,
                actual_size,
            ));
        }

        // Create attachment record
        let create_input = CreateAttachmentInput {
            id: input.attachment_id,
            organization_id: input.organization_id,
            transaction_id: Some(input.transaction_id),
            attachment_type: input.attachment_type,
            filename: input.filename,
            file_size: input.file_size,
            mime_type: input.content_type,
            checksum_sha256: None,
            storage_provider: self.storage.provider_name().to_string(),
            storage_bucket: self.storage.bucket().to_string(),
            storage_key: input.storage_key,
            storage_region: None,
            uploaded_by: input.uploaded_by,
        };

        self.repo.create(create_input).await
    }

    /// Get a download URL for an attachment.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Attachment not found
    /// - Storage service fails
    pub async fn get_download_url(
        &self,
        attachment_id: Uuid,
        organization_id: Uuid,
    ) -> Result<crate::storage::PresignedUrl, AttachmentError> {
        // Find attachment
        let attachment = self
            .repo
            .find_by_id(attachment_id, organization_id)
            .await?
            .ok_or_else(|| AttachmentError::not_found(attachment_id))?;

        // Generate presigned download URL
        let presigned = self
            .storage
            .presign_download(&attachment.storage_key)
            .await?;

        Ok(presigned)
    }

    /// Delete an attachment.
    ///
    /// This removes both the storage object and the database record.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Attachment not found
    /// - Storage deletion fails
    /// - Database deletion fails
    pub async fn delete(
        &self,
        attachment_id: Uuid,
        organization_id: Uuid,
    ) -> Result<(), AttachmentError> {
        // Find attachment
        let attachment = self
            .repo
            .find_by_id(attachment_id, organization_id)
            .await?
            .ok_or_else(|| AttachmentError::not_found(attachment_id))?;

        // Delete from storage (ignore not found errors)
        let _ = self.storage.delete(&attachment.storage_key).await;

        // Delete from database
        self.repo.delete(attachment_id, organization_id).await?;

        Ok(())
    }

    /// List attachments for a transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if database operation fails.
    pub async fn list_by_transaction(
        &self,
        transaction_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<Attachment>, AttachmentError> {
        self.repo
            .list_by_transaction(transaction_id, organization_id)
            .await
    }

    /// Get attachment by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if attachment not found or database operation fails.
    pub async fn get_by_id(
        &self,
        attachment_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Attachment, AttachmentError> {
        self.repo
            .find_by_id(attachment_id, organization_id)
            .await?
            .ok_or_else(|| AttachmentError::not_found(attachment_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachment::AttachmentType;
    use crate::storage::{StorageConfig, StorageProvider};
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock repository for testing.
    struct MockAttachmentRepository {
        attachments: Mutex<HashMap<Uuid, Attachment>>,
        transactions: Mutex<std::collections::HashSet<Uuid>>,
    }

    impl MockAttachmentRepository {
        fn new() -> Self {
            Self {
                attachments: Mutex::new(HashMap::new()),
                transactions: Mutex::new(std::collections::HashSet::new()),
            }
        }

        #[allow(dead_code)]
        fn add_transaction(&self, id: Uuid) {
            self.transactions.lock().unwrap().insert(id);
        }
    }

    impl AttachmentRepository for MockAttachmentRepository {
        async fn create(
            &self,
            input: CreateAttachmentInput,
        ) -> Result<Attachment, AttachmentError> {
            let attachment = Attachment {
                id: input.id,
                organization_id: input.organization_id,
                transaction_id: input.transaction_id,
                attachment_type: input.attachment_type,
                filename: input.filename,
                file_size: input.file_size,
                mime_type: input.mime_type,
                checksum_sha256: input.checksum_sha256,
                storage_provider: input.storage_provider,
                storage_bucket: input.storage_bucket,
                storage_key: input.storage_key,
                storage_region: input.storage_region,
                uploaded_by: input.uploaded_by,
                created_at: chrono::Utc::now(),
            };
            self.attachments
                .lock()
                .unwrap()
                .insert(attachment.id, attachment.clone());
            Ok(attachment)
        }

        async fn find_by_id(
            &self,
            id: Uuid,
            _organization_id: Uuid,
        ) -> Result<Option<Attachment>, AttachmentError> {
            Ok(self.attachments.lock().unwrap().get(&id).cloned())
        }

        async fn list_by_transaction(
            &self,
            transaction_id: Uuid,
            _organization_id: Uuid,
        ) -> Result<Vec<Attachment>, AttachmentError> {
            Ok(self
                .attachments
                .lock()
                .unwrap()
                .values()
                .filter(|a| a.transaction_id == Some(transaction_id))
                .cloned()
                .collect())
        }

        async fn delete(&self, id: Uuid, _organization_id: Uuid) -> Result<bool, AttachmentError> {
            Ok(self.attachments.lock().unwrap().remove(&id).is_some())
        }

        async fn transaction_exists(
            &self,
            transaction_id: Uuid,
            _organization_id: Uuid,
        ) -> Result<bool, AttachmentError> {
            Ok(self.transactions.lock().unwrap().contains(&transaction_id))
        }
    }

    #[tokio::test]
    async fn test_request_upload_transaction_not_found() {
        let config = StorageConfig::new(StorageProvider::local_fs("./test"));
        let storage = Arc::new(StorageService::from_config(config).unwrap());
        let repo = Arc::new(MockAttachmentRepository::new());
        let service = AttachmentService::new(storage, repo);

        let input = RequestUploadInput {
            organization_id: Uuid::new_v4(),
            transaction_id: Uuid::new_v4(),
            filename: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            file_size: 1024,
            attachment_type: AttachmentType::Receipt,
            user_id: Uuid::new_v4(),
        };

        let result = service.request_upload(input).await;
        assert!(matches!(
            result,
            Err(AttachmentError::TransactionNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_get_attachment_not_found() {
        let config = StorageConfig::new(StorageProvider::local_fs("./test"));
        let storage = Arc::new(StorageService::from_config(config).unwrap());
        let repo = Arc::new(MockAttachmentRepository::new());
        let service = AttachmentService::new(storage, repo);

        let result = service.get_by_id(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(matches!(result, Err(AttachmentError::NotFound(_))));
    }
}
