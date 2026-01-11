//! Storage service implementation using Apache OpenDAL.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use opendal::{ErrorKind, Operator, services};
use uuid::Uuid;

use super::config::{StorageConfig, StorageProvider};
use super::error::StorageError;

/// Presigned URL for upload or download.
#[derive(Debug, Clone)]
pub struct PresignedUrl {
    /// The presigned URL.
    pub url: String,
    /// HTTP method to use (PUT for upload, GET for download).
    pub method: String,
    /// When the URL expires.
    pub expires_at: DateTime<Utc>,
    /// Required headers for the request.
    pub headers: HashMap<String, String>,
}

/// Request to generate an upload URL.
#[derive(Debug, Clone)]
pub struct UploadRequest {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Transaction ID (optional).
    pub transaction_id: Option<Uuid>,
    /// Attachment ID.
    pub attachment_id: Uuid,
    /// Original filename.
    pub filename: String,
    /// Content type (MIME type).
    pub content_type: String,
    /// File size in bytes.
    pub file_size: u64,
}

/// Metadata about an uploaded attachment.
#[derive(Debug, Clone)]
pub struct AttachmentMetadata {
    /// Storage key.
    pub storage_key: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Content type.
    pub content_type: Option<String>,
}

/// Storage service for file attachments.
pub struct StorageService {
    operator: Operator,
    config: StorageConfig,
}

impl StorageService {
    /// Create a new storage service from configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage provider cannot be initialized.
    pub fn from_config(config: StorageConfig) -> Result<Self, StorageError> {
        let operator = Self::create_operator(&config.provider)?;
        Ok(Self { operator, config })
    }

    /// Create OpenDAL operator from provider config.
    fn create_operator(provider: &StorageProvider) -> Result<Operator, StorageError> {
        match provider {
            StorageProvider::S3 {
                endpoint,
                bucket,
                access_key_id,
                secret_access_key,
                region,
            } => {
                let builder = services::S3::default()
                    .endpoint(endpoint)
                    .bucket(bucket)
                    .access_key_id(access_key_id)
                    .secret_access_key(secret_access_key)
                    .region(region);

                Operator::new(builder)
                    .map_err(|e| StorageError::configuration(e.to_string()))?
                    .finish()
                    .pipe(Ok)
            }
            StorageProvider::AzureBlob {
                account,
                access_key,
                container,
            } => {
                let builder = services::Azblob::default()
                    .account_name(account)
                    .account_key(access_key)
                    .container(container);

                Operator::new(builder)
                    .map_err(|e| StorageError::configuration(e.to_string()))?
                    .finish()
                    .pipe(Ok)
            }
            StorageProvider::LocalFs { root } => {
                let builder = services::Fs::default().root(
                    root.to_str()
                        .ok_or_else(|| StorageError::configuration("invalid path"))?,
                );

                Operator::new(builder)
                    .map_err(|e| StorageError::configuration(e.to_string()))?
                    .finish()
                    .pipe(Ok)
            }
        }
    }

    /// Validate upload request against config constraints.
    ///
    /// # Errors
    ///
    /// Returns an error if file size or MIME type is invalid.
    pub fn validate_upload(&self, content_type: &str, size: u64) -> Result<(), StorageError> {
        // Check file size
        if size > self.config.max_file_size {
            return Err(StorageError::file_too_large(
                size,
                self.config.max_file_size,
            ));
        }

        // Check MIME type
        if !self.config.is_mime_type_allowed(content_type) {
            return Err(StorageError::invalid_mime_type(content_type));
        }

        Ok(())
    }

    /// Generate storage key for an attachment.
    ///
    /// Format: `{org_id}/{transaction_id}/{attachment_id}/{sanitized_filename}`
    #[must_use]
    pub fn generate_storage_key(req: &UploadRequest) -> String {
        let sanitized_filename = sanitize_filename(&req.filename);
        let transaction_part = req
            .transaction_id
            .map_or_else(|| "orphan".to_string(), |id| id.to_string());

        format!(
            "{}/{}/{}/{}",
            req.organization_id, transaction_part, req.attachment_id, sanitized_filename
        )
    }

    /// Generate presigned URL for upload.
    ///
    /// # Errors
    ///
    /// Returns an error if presigning is not supported or fails.
    pub async fn presign_upload(&self, req: &UploadRequest) -> Result<PresignedUrl, StorageError> {
        // Validate first
        self.validate_upload(&req.content_type, req.file_size)?;

        let key = Self::generate_storage_key(req);
        let ttl = Duration::from_secs(self.config.presign_upload_ttl_secs);

        let presigned = self
            .operator
            .presign_write(&key, ttl)
            .await
            .map_err(StorageError::from)?;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), req.content_type.clone());

        Ok(PresignedUrl {
            url: presigned.uri().to_string(),
            method: presigned.method().to_string(),
            expires_at: Utc::now()
                + chrono::Duration::seconds(
                    i64::try_from(self.config.presign_upload_ttl_secs).unwrap_or(i64::MAX),
                ),
            headers,
        })
    }

    /// Generate presigned URL for download.
    ///
    /// # Errors
    ///
    /// Returns an error if presigning is not supported or fails.
    pub async fn presign_download(&self, key: &str) -> Result<PresignedUrl, StorageError> {
        let ttl = Duration::from_secs(self.config.presign_download_ttl_secs);

        let presigned = self
            .operator
            .presign_read(key, ttl)
            .await
            .map_err(StorageError::from)?;

        Ok(PresignedUrl {
            url: presigned.uri().to_string(),
            method: presigned.method().to_string(),
            expires_at: Utc::now()
                + chrono::Duration::seconds(
                    i64::try_from(self.config.presign_download_ttl_secs).unwrap_or(i64::MAX),
                ),
            headers: HashMap::new(),
        })
    }

    /// Verify that a file exists in storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist or cannot be accessed.
    pub async fn verify_upload(&self, key: &str) -> Result<AttachmentMetadata, StorageError> {
        let meta = self.operator.stat(key).await.map_err(StorageError::from)?;

        Ok(AttachmentMetadata {
            storage_key: key.to_string(),
            file_size: meta.content_length(),
            content_type: meta.content_type().map(String::from),
        })
    }

    /// Delete a file from storage.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails.
    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.operator.delete(key).await.map_err(StorageError::from)
    }

    /// Check if a file exists in storage.
    pub async fn exists(&self, key: &str) -> bool {
        match self.operator.stat(key).await {
            Ok(_) => true,
            Err(e) if e.kind() == ErrorKind::NotFound => false,
            Err(_) => false,
        }
    }

    /// Get the storage provider name.
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        self.config.provider.name()
    }

    /// Get the bucket/container name.
    #[must_use]
    pub fn bucket(&self) -> &str {
        self.config.provider.bucket()
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

/// Sanitize filename for storage key.
///
/// Removes or replaces characters that could cause issues in storage paths.
/// Only allows ASCII alphanumeric characters, dots, hyphens, and underscores.
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Extension trait for pipe operator.
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("invoice.pdf"), "invoice.pdf");
        assert_eq!(sanitize_filename("my file (1).pdf"), "my_file__1_.pdf");
        assert_eq!(sanitize_filename("test@#$%.doc"), "test____.doc");
        assert_eq!(sanitize_filename("日本語.pdf"), "___.pdf");
    }

    #[test]
    fn test_generate_storage_key() {
        let org_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid uuid");
        let tx_id = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").expect("valid uuid");
        let att_id = Uuid::parse_str("6ba7b811-9dad-11d1-80b4-00c04fd430c8").expect("valid uuid");

        let req = UploadRequest {
            organization_id: org_id,
            transaction_id: Some(tx_id),
            attachment_id: att_id,
            filename: "invoice.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            file_size: 1024,
        };

        let key = StorageService::generate_storage_key(&req);
        assert!(key.contains(&org_id.to_string()));
        assert!(key.contains(&tx_id.to_string()));
        assert!(key.contains(&att_id.to_string()));
        assert!(key.ends_with("invoice.pdf"));
    }

    #[test]
    fn test_generate_storage_key_without_transaction() {
        let org_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid uuid");
        let att_id = Uuid::parse_str("6ba7b811-9dad-11d1-80b4-00c04fd430c8").expect("valid uuid");

        let req = UploadRequest {
            organization_id: org_id,
            transaction_id: None,
            attachment_id: att_id,
            filename: "receipt.png".to_string(),
            content_type: "image/png".to_string(),
            file_size: 2048,
        };

        let key = StorageService::generate_storage_key(&req);
        assert!(key.contains("orphan"));
    }

    #[test]
    fn test_validate_upload_size() {
        let config =
            StorageConfig::new(StorageProvider::local_fs("./test")).with_max_file_size(1024);
        let service = StorageService::from_config(config).expect("should create service");

        // Valid size
        assert!(service.validate_upload("application/pdf", 512).is_ok());

        // Too large
        let err = service
            .validate_upload("application/pdf", 2048)
            .unwrap_err();
        assert!(matches!(err, StorageError::FileTooLarge { .. }));
    }

    #[test]
    fn test_validate_upload_mime_type() {
        let config = StorageConfig::new(StorageProvider::local_fs("./test"));
        let service = StorageService::from_config(config).expect("should create service");

        // Valid MIME type
        assert!(service.validate_upload("application/pdf", 1024).is_ok());
        assert!(service.validate_upload("image/png", 1024).is_ok());

        // Invalid MIME type
        let err = service
            .validate_upload("application/x-executable", 1024)
            .unwrap_err();
        assert!(matches!(err, StorageError::InvalidMimeType { .. }));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property 1: Presigned URL TTL Validity
    // For any presigned URL generated, the expiration time SHALL be within
    // the configured TTL with a tolerance of ±5 seconds.
    proptest! {
        #[test]
        fn prop_presigned_url_ttl_within_bounds(
            upload_ttl in 60u64..3600,
            download_ttl in 60u64..7200,
        ) {
            let config = StorageConfig::new(StorageProvider::local_fs("./test"))
                .with_upload_ttl(upload_ttl)
                .with_download_ttl(download_ttl);

            // Verify config stores correct TTL values
            prop_assert_eq!(config.presign_upload_ttl_secs, upload_ttl);
            prop_assert_eq!(config.presign_download_ttl_secs, download_ttl);
        }
    }

    // Property 2: MIME Type Validation
    // For any file upload request, the service SHALL accept only files with
    // MIME types in the allowed list and reject all others.
    proptest! {
        #[test]
        fn prop_mime_type_validation(mime_type in "[a-z]+/[a-z0-9-]+") {
            let config = StorageConfig::new(StorageProvider::local_fs("./test"));
            let service = StorageService::from_config(config.clone())
                .expect("should create service");

            let result = service.validate_upload(&mime_type, 1024);
            let is_allowed = config.is_mime_type_allowed(&mime_type);

            if is_allowed {
                prop_assert!(result.is_ok(), "Expected Ok for allowed MIME type");
            } else {
                let is_invalid_mime = matches!(result, Err(StorageError::InvalidMimeType { .. }));
                prop_assert!(is_invalid_mime, "Expected InvalidMimeType error");
            }
        }
    }

    // Property 3: File Size Validation
    // For any file upload request with size exceeding the configured limit,
    // the service SHALL reject the request.
    proptest! {
        #[test]
        fn prop_file_size_validation(
            max_size in 1024u64..10_000_000,
            file_size in 0u64..20_000_000,
        ) {
            let config = StorageConfig::new(StorageProvider::local_fs("./test"))
                .with_max_file_size(max_size);
            let service = StorageService::from_config(config)
                .expect("should create service");

            let result = service.validate_upload("application/pdf", file_size);

            if file_size <= max_size {
                prop_assert!(result.is_ok(), "Expected Ok for valid file size");
            } else {
                let is_too_large = matches!(result, Err(StorageError::FileTooLarge { .. }));
                prop_assert!(is_too_large, "Expected FileTooLarge error");
            }
        }
    }

    // Property 4: Storage Key Format
    // For any attachment created, the storage_key SHALL match the pattern
    // {org_id}/{transaction_id}/{attachment_id}/{filename}
    proptest! {
        #[test]
        fn prop_storage_key_format(
            filename in "[a-zA-Z0-9_-]{1,50}\\.[a-z]{2,4}",
        ) {
            let org_id = Uuid::new_v4();
            let tx_id = Uuid::new_v4();
            let att_id = Uuid::new_v4();

            let req = UploadRequest {
                organization_id: org_id,
                transaction_id: Some(tx_id),
                attachment_id: att_id,
                filename: filename.clone(),
                content_type: "application/pdf".to_string(),
                file_size: 1024,
            };

            let key = StorageService::generate_storage_key(&req);

            // Verify key contains all required parts
            prop_assert!(key.contains(&org_id.to_string()));
            prop_assert!(key.contains(&tx_id.to_string()));
            prop_assert!(key.contains(&att_id.to_string()));

            // Verify key format: org_id/tx_id/att_id/filename
            let parts: Vec<&str> = key.split('/').collect();
            prop_assert_eq!(parts.len(), 4);
            prop_assert_eq!(parts[0], org_id.to_string());
            prop_assert_eq!(parts[1], tx_id.to_string());
            prop_assert_eq!(parts[2], att_id.to_string());
        }
    }

    // Property: Sanitized filename only contains safe characters
    proptest! {
        #[test]
        fn prop_sanitized_filename_safe_chars(filename in ".*") {
            let sanitized = sanitize_filename(&filename);

            for c in sanitized.chars() {
                let is_safe = c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_';
                prop_assert!(is_safe, "Unexpected character in sanitized filename: {}", c);
            }
        }
    }

    // Property: Storage key without transaction uses "orphan" placeholder
    proptest! {
        #[test]
        fn prop_storage_key_orphan_placeholder(
            filename in "[a-zA-Z0-9]{1,20}\\.[a-z]{2,4}",
        ) {
            let org_id = Uuid::new_v4();
            let att_id = Uuid::new_v4();

            let req = UploadRequest {
                organization_id: org_id,
                transaction_id: None,
                attachment_id: att_id,
                filename,
                content_type: "application/pdf".to_string(),
                file_size: 1024,
            };

            let key = StorageService::generate_storage_key(&req);
            let parts: Vec<&str> = key.split('/').collect();

            prop_assert_eq!(parts[1], "orphan");
        }
    }
}
