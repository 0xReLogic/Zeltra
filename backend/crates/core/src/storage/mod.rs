//! Storage service for file attachments using Apache OpenDAL.
//!
//! This module provides vendor-agnostic object storage with support for:
//! - S3-compatible: Cloudflare R2, Supabase Storage, AWS S3, DigitalOcean Spaces
//! - Azure Blob Storage
//! - Local filesystem (development only)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Apache OpenDAL                              │
//! │                   (Unified Storage API)                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ op.write("key", data)      │ op.presign_read("key", duration)   │
//! │ op.read("key")             │ op.presign_write("key", duration)  │
//! │ op.delete("key")           │ op.stat("key")                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

mod config;
mod error;
mod service;

pub use config::{StorageConfig, StorageProvider};
pub use error::StorageError;
pub use service::{AttachmentMetadata, PresignedUrl, StorageService, UploadRequest};
