//! Attachment types and data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Attachment type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentType {
    /// Receipt for expense.
    Receipt,
    /// Invoice document.
    Invoice,
    /// Contract document.
    Contract,
    /// Supporting document.
    SupportingDocument,
    /// Other document type.
    #[default]
    Other,
}

impl AttachmentType {
    /// Convert to database string value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Receipt => "receipt",
            Self::Invoice => "invoice",
            Self::Contract => "contract",
            Self::SupportingDocument => "supporting_document",
            Self::Other => "other",
        }
    }

    /// Parse from database string value.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "receipt" => Some(Self::Receipt),
            "invoice" => Some(Self::Invoice),
            "contract" => Some(Self::Contract),
            "supporting_document" => Some(Self::SupportingDocument),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

/// Input for requesting an upload URL.
#[derive(Debug, Clone)]
pub struct RequestUploadInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Transaction ID to attach to.
    pub transaction_id: Uuid,
    /// Original filename.
    pub filename: String,
    /// MIME type of the file.
    pub content_type: String,
    /// File size in bytes.
    pub file_size: u64,
    /// Attachment type classification.
    pub attachment_type: AttachmentType,
    /// User requesting the upload.
    pub user_id: Uuid,
}

/// Result of requesting an upload URL.
#[derive(Debug, Clone)]
pub struct RequestUploadResult {
    /// Generated attachment ID.
    pub attachment_id: Uuid,
    /// Presigned upload URL.
    pub upload_url: String,
    /// HTTP method to use (PUT).
    pub upload_method: String,
    /// Required headers for the upload.
    pub upload_headers: std::collections::HashMap<String, String>,
    /// When the URL expires.
    pub expires_at: DateTime<Utc>,
    /// Storage key for the file.
    pub storage_key: String,
}

/// Input for confirming an upload.
#[derive(Debug, Clone)]
pub struct ConfirmUploadInput {
    /// Attachment ID from request_upload.
    pub attachment_id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Original filename.
    pub filename: String,
    /// MIME type.
    pub content_type: String,
    /// File size in bytes.
    pub file_size: i64,
    /// Storage key.
    pub storage_key: String,
    /// Attachment type.
    pub attachment_type: AttachmentType,
    /// User who uploaded.
    pub uploaded_by: Uuid,
}

/// Input for creating an attachment record.
#[derive(Debug, Clone)]
pub struct CreateAttachmentInput {
    /// Attachment ID.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Transaction ID.
    pub transaction_id: Option<Uuid>,
    /// Attachment type.
    pub attachment_type: AttachmentType,
    /// Original filename.
    pub filename: String,
    /// File size in bytes.
    pub file_size: i64,
    /// MIME type.
    pub mime_type: String,
    /// SHA256 checksum (optional).
    pub checksum_sha256: Option<String>,
    /// Storage provider name.
    pub storage_provider: String,
    /// Storage bucket/container.
    pub storage_bucket: String,
    /// Storage key/path.
    pub storage_key: String,
    /// Storage region (optional).
    pub storage_region: Option<String>,
    /// User who uploaded.
    pub uploaded_by: Uuid,
}

/// Attachment domain model.
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Unique identifier.
    pub id: Uuid,
    /// Organization ID.
    pub organization_id: Uuid,
    /// Transaction ID (optional).
    pub transaction_id: Option<Uuid>,
    /// Attachment type.
    pub attachment_type: AttachmentType,
    /// Original filename.
    pub filename: String,
    /// File size in bytes.
    pub file_size: i64,
    /// MIME type.
    pub mime_type: String,
    /// SHA256 checksum.
    pub checksum_sha256: Option<String>,
    /// Storage provider.
    pub storage_provider: String,
    /// Storage bucket.
    pub storage_bucket: String,
    /// Storage key.
    pub storage_key: String,
    /// Storage region.
    pub storage_region: Option<String>,
    /// User who uploaded.
    pub uploaded_by: Uuid,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_type_roundtrip() {
        let types = [
            AttachmentType::Receipt,
            AttachmentType::Invoice,
            AttachmentType::Contract,
            AttachmentType::SupportingDocument,
            AttachmentType::Other,
        ];

        for t in types {
            let s = t.as_str();
            let parsed = AttachmentType::parse(s);
            assert_eq!(parsed, Some(t));
        }
    }

    #[test]
    fn test_attachment_type_unknown() {
        assert_eq!(AttachmentType::parse("unknown"), None);
    }
}
