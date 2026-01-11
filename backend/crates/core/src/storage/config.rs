//! Storage configuration types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Storage provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StorageProvider {
    /// S3-compatible storage: Cloudflare R2, Supabase, AWS S3, DigitalOcean Spaces
    S3 {
        /// S3 endpoint URL.
        endpoint: String,
        /// S3 bucket name.
        bucket: String,
        /// AWS access key ID.
        access_key_id: String,
        /// AWS secret access key.
        secret_access_key: String,
        /// AWS region.
        region: String,
    },
    /// Azure Blob Storage
    AzureBlob {
        /// Azure storage account name.
        account: String,
        /// Azure storage access key.
        access_key: String,
        /// Azure container name.
        container: String,
    },
    /// Local filesystem (development only)
    LocalFs {
        /// Root directory path.
        root: PathBuf,
    },
}

impl StorageProvider {
    /// Create S3-compatible provider (Cloudflare R2, Supabase, AWS S3).
    #[must_use]
    pub fn s3(
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self::S3 {
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            region: region.into(),
        }
    }

    /// Create Azure Blob Storage provider.
    #[must_use]
    pub fn azure_blob(
        account: impl Into<String>,
        access_key: impl Into<String>,
        container: impl Into<String>,
    ) -> Self {
        Self::AzureBlob {
            account: account.into(),
            access_key: access_key.into(),
            container: container.into(),
        }
    }

    /// Create local filesystem provider (development only).
    #[must_use]
    pub fn local_fs(root: impl Into<PathBuf>) -> Self {
        Self::LocalFs { root: root.into() }
    }

    /// Get the provider name for database storage.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::S3 { .. } => "s3",
            Self::AzureBlob { .. } => "azure_blob",
            Self::LocalFs { .. } => "local",
        }
    }

    /// Get the bucket/container name.
    #[must_use]
    pub fn bucket(&self) -> &str {
        match self {
            Self::S3 { bucket, .. } => bucket,
            Self::AzureBlob { container, .. } => container,
            Self::LocalFs { root } => root.to_str().unwrap_or("local"),
        }
    }
}

/// Storage service configuration.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Storage provider configuration.
    pub provider: StorageProvider,
    /// Maximum file size in bytes.
    pub max_file_size: u64,
    /// Presigned upload URL TTL in seconds (default: 900 = 15 minutes).
    pub presign_upload_ttl_secs: u64,
    /// Presigned download URL TTL in seconds (default: 3600 = 1 hour).
    pub presign_download_ttl_secs: u64,
    /// Allowed MIME types for upload.
    pub allowed_mime_types: Vec<String>,
}

impl StorageConfig {
    /// Default max file size: 10MB.
    pub const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
    /// Default upload TTL: 15 minutes.
    pub const DEFAULT_UPLOAD_TTL: u64 = 900;
    /// Default download TTL: 1 hour.
    pub const DEFAULT_DOWNLOAD_TTL: u64 = 3600;

    /// Create a new storage config with default settings.
    #[must_use]
    pub fn new(provider: StorageProvider) -> Self {
        Self {
            provider,
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
            presign_upload_ttl_secs: Self::DEFAULT_UPLOAD_TTL,
            presign_download_ttl_secs: Self::DEFAULT_DOWNLOAD_TTL,
            allowed_mime_types: Self::default_mime_types(),
        }
    }

    /// Set maximum file size.
    #[must_use]
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set presigned upload URL TTL.
    #[must_use]
    pub fn with_upload_ttl(mut self, secs: u64) -> Self {
        self.presign_upload_ttl_secs = secs;
        self
    }

    /// Set presigned download URL TTL.
    #[must_use]
    pub fn with_download_ttl(mut self, secs: u64) -> Self {
        self.presign_download_ttl_secs = secs;
        self
    }

    /// Set allowed MIME types.
    #[must_use]
    pub fn with_allowed_mime_types(mut self, types: Vec<String>) -> Self {
        self.allowed_mime_types = types;
        self
    }

    /// Default allowed MIME types for attachments.
    #[must_use]
    pub fn default_mime_types() -> Vec<String> {
        vec![
            // Documents
            "application/pdf".to_string(),
            "application/msword".to_string(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            "application/vnd.ms-excel".to_string(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            // Images
            "image/png".to_string(),
            "image/jpeg".to_string(),
            "image/gif".to_string(),
            "image/webp".to_string(),
        ]
    }

    /// Check if a MIME type is allowed.
    #[must_use]
    pub fn is_mime_type_allowed(&self, mime_type: &str) -> bool {
        self.allowed_mime_types.iter().any(|t| t == mime_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_provider_s3() {
        let provider = StorageProvider::s3(
            "https://account.r2.cloudflarestorage.com",
            "attachments",
            "access_key",
            "secret_key",
            "auto",
        );
        assert_eq!(provider.name(), "s3");
        assert_eq!(provider.bucket(), "attachments");
    }

    #[test]
    fn test_storage_provider_azure() {
        let provider = StorageProvider::azure_blob("zeltradev", "access_key", "attachments");
        assert_eq!(provider.name(), "azure_blob");
        assert_eq!(provider.bucket(), "attachments");
    }

    #[test]
    fn test_storage_provider_local() {
        let provider = StorageProvider::local_fs("./storage");
        assert_eq!(provider.name(), "local");
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::new(StorageProvider::local_fs("./storage"));
        assert_eq!(config.max_file_size, StorageConfig::DEFAULT_MAX_FILE_SIZE);
        assert_eq!(
            config.presign_upload_ttl_secs,
            StorageConfig::DEFAULT_UPLOAD_TTL
        );
        assert_eq!(
            config.presign_download_ttl_secs,
            StorageConfig::DEFAULT_DOWNLOAD_TTL
        );
        assert!(!config.allowed_mime_types.is_empty());
    }

    #[test]
    fn test_mime_type_validation() {
        let config = StorageConfig::new(StorageProvider::local_fs("./storage"));
        assert!(config.is_mime_type_allowed("application/pdf"));
        assert!(config.is_mime_type_allowed("image/png"));
        assert!(!config.is_mime_type_allowed("application/x-executable"));
        assert!(!config.is_mime_type_allowed("text/html"));
    }
}
