//! Attachment error types.

use thiserror::Error;
use uuid::Uuid;

use crate::storage::StorageError;

/// Attachment operation errors.
#[derive(Debug, Error)]
pub enum AttachmentError {
    /// Attachment not found.
    #[error("attachment not found: {0}")]
    NotFound(Uuid),

    /// Transaction not found.
    #[error("transaction not found: {0}")]
    TransactionNotFound(Uuid),

    /// Upload not verified - file not found in storage.
    #[error("upload not verified: file not found in storage")]
    UploadNotVerified,

    /// File size mismatch between request and actual upload.
    #[error("file size mismatch: expected {expected}, got {actual}")]
    FileSizeMismatch {
        /// Expected file size.
        expected: u64,
        /// Actual file size.
        actual: u64,
    },

    /// Invalid MIME type.
    #[error("invalid MIME type: {0}")]
    InvalidMimeType(String),

    /// File too large.
    #[error("file too large: {size} bytes exceeds maximum {max} bytes")]
    FileTooLarge {
        /// Actual file size.
        size: u64,
        /// Maximum allowed size.
        max: u64,
    },

    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    /// Repository operation failed.
    #[error("repository error: {0}")]
    Repository(String),

    /// Unauthorized access.
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}

impl AttachmentError {
    /// Create a not found error.
    #[must_use]
    pub fn not_found(id: Uuid) -> Self {
        Self::NotFound(id)
    }

    /// Create a transaction not found error.
    #[must_use]
    pub fn transaction_not_found(id: Uuid) -> Self {
        Self::TransactionNotFound(id)
    }

    /// Create a file size mismatch error.
    #[must_use]
    pub fn file_size_mismatch(expected: u64, actual: u64) -> Self {
        Self::FileSizeMismatch { expected, actual }
    }

    /// Create a repository error.
    #[must_use]
    pub fn repository(msg: impl Into<String>) -> Self {
        Self::Repository(msg.into())
    }
}
