//! Attachment service for file management.
//!
//! This module provides business logic for file attachments including:
//! - Upload request validation
//! - Presigned URL generation
//! - Upload confirmation
//! - Download URL generation
//! - Attachment deletion

mod error;
mod service;
mod types;

pub use error::AttachmentError;
pub use service::{AttachmentRepository, AttachmentService};
pub use types::{
    Attachment, AttachmentType, ConfirmUploadInput, CreateAttachmentInput, RequestUploadInput,
    RequestUploadResult,
};
