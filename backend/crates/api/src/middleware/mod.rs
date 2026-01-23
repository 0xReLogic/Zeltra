//! Middleware for request processing.

pub mod auth;
pub mod keyed_rate_limit;
pub mod logging;
pub mod rate_limit;

#[cfg(test)]
mod error_tests;

pub use auth::{AuthUser, auth_middleware};
pub use keyed_rate_limit::{KeyedRateLimitConfig, KeyedRateLimitLayer, extract_rate_limit_key};
pub use logging::{RequestId, RequestSpan, inject_request_id};
pub use rate_limit::{RateLimitConfig, RateLimitLayer};
