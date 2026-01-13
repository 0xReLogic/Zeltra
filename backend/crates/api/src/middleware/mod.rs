//! Middleware for request processing.

pub mod auth;
pub mod logging;
pub mod rate_limit;

#[cfg(test)]
mod error_tests;

pub use auth::{AuthUser, auth_middleware};
pub use logging::{RequestId, RequestSpan, inject_request_id};
pub use rate_limit::{RateLimitConfig, RateLimitLayer};
