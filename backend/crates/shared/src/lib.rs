//! Shared types, errors, and configuration for Zeltra.
//!
//! This crate provides common types used across all other crates:
//! - Money types with decimal precision
//! - Typed IDs for type-safe entity references
//! - Pagination types for list endpoints
//! - Application-wide error types
//! - Configuration management
//! - JWT claims and auth types
//! - Email service for transactional emails

pub mod auth;
pub mod config;
pub mod email;
pub mod error;
pub mod jwt;
pub mod types;

#[cfg(test)]
mod auth_phase1_tests;
#[cfg(test)]
mod jwt_tests;

pub use auth::{Claims, TokenPair};
pub use config::{AppConfig, EmailConfig};
pub use email::{EmailError, EmailService};
pub use error::{AppError, AppResult};
pub use jwt::{JwtConfig, JwtError, JwtService};
