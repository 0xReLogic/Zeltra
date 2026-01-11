//! Storage error types.

use thiserror::Error;

/// Storage operation errors.
#[derive(Debug, Error)]
pub enum StorageError {
    /// File size exceeds maximum allowed.
    #[error("file size {size} bytes exceeds maximum allowed {max} bytes")]
    FileTooLarge {
        /// Actual file size.
        size: u64,
        /// Maximum allowed size.
        max: u64,
    },

    /// MIME type not allowed.
    #[error("MIME type '{mime_type}' is not allowed")]
    InvalidMimeType {
        /// The invalid MIME type.
        mime_type: String,
    },

    /// File not found in storage.
    #[error("file not found: {key}")]
    NotFound {
        /// Storage key that was not found.
        key: String,
    },

    /// Presign operation not supported by provider.
    #[error("presign operation not supported by storage provider")]
    PresignNotSupported,

    /// Storage provider configuration error.
    #[error("storage configuration error: {0}")]
    Configuration(String),

    /// OpenDAL operation error.
    #[error("storage operation failed: {0}")]
    Operation(String),

    /// Invalid storage key format.
    #[error("invalid storage key: {0}")]
    InvalidKey(String),
}

impl StorageError {
    /// Create a file too large error.
    #[must_use]
    pub fn file_too_large(size: u64, max: u64) -> Self {
        Self::FileTooLarge { size, max }
    }

    /// Create an invalid MIME type error.
    #[must_use]
    pub fn invalid_mime_type(mime_type: impl Into<String>) -> Self {
        Self::InvalidMimeType {
            mime_type: mime_type.into(),
        }
    }

    /// Create a not found error.
    #[must_use]
    pub fn not_found(key: impl Into<String>) -> Self {
        Self::NotFound { key: key.into() }
    }

    /// Create a configuration error.
    #[must_use]
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create an operation error.
    #[must_use]
    pub fn operation(msg: impl Into<String>) -> Self {
        Self::Operation(msg.into())
    }
}

impl From<opendal::Error> for StorageError {
    fn from(err: opendal::Error) -> Self {
        match err.kind() {
            opendal::ErrorKind::NotFound => Self::NotFound {
                key: err.to_string(),
            },
            opendal::ErrorKind::Unsupported => Self::PresignNotSupported,
            _ => Self::Operation(err.to_string()),
        }
    }
}
