//! Shared types, errors, and configuration for Zeltra.
//!
//! This crate provides common types used across all other crates:
//! - Money types with decimal precision
//! - Typed IDs for type-safe entity references
//! - Pagination types for list endpoints
//! - Application-wide error types
//! - Configuration management

pub mod config;
pub mod error;
pub mod types;

pub use config::AppConfig;
pub use error::{AppError, AppResult};
