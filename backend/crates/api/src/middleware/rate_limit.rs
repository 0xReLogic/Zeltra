//! Rate limiting middleware.
//!
//! Implements Requirements 5.2 for API rate limiting.

use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::{
    num::NonZeroU32,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::error::ApiError;

/// Rate limiter state.
pub type GlobalRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Rate limit configuration.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Requests per second.
    pub requests_per_second: u32,
    /// Burst size (max requests in a burst).
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

impl RateLimitConfig {
    /// Creates a new rate limit config.
    #[must_use]
    pub const fn new(requests_per_second: u32, burst_size: u32) -> Self {
        Self {
            requests_per_second,
            burst_size,
        }
    }

    /// Creates a rate limiter from this config.
    #[must_use]
    pub fn create_limiter(&self) -> Arc<GlobalRateLimiter> {
        let quota =
            Quota::per_second(NonZeroU32::new(self.requests_per_second).unwrap_or(NonZeroU32::MIN))
                .allow_burst(NonZeroU32::new(self.burst_size).unwrap_or(NonZeroU32::MIN));

        Arc::new(RateLimiter::direct(quota))
    }
}

/// Rate limiting layer.
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<GlobalRateLimiter>,
}

impl RateLimitLayer {
    /// Creates a new rate limit layer.
    #[must_use]
    pub fn new(limiter: Arc<GlobalRateLimiter>) -> Self {
        Self { limiter }
    }

    /// Creates a rate limit layer with default config.
    #[must_use]
    pub fn with_default_config() -> Self {
        Self::new(RateLimitConfig::default().create_limiter())
    }

    /// Creates a rate limit layer with custom config.
    #[must_use]
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self::new(config.create_limiter())
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Rate limiting service.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<GlobalRateLimiter>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let limiter = self.limiter.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Check rate limit
            match limiter.check() {
                Ok(()) => {
                    // Request allowed, proceed
                    inner.call(req).await
                }
                Err(not_until) => {
                    // Rate limited
                    let wait_time = not_until.wait_time_from(governor::clock::Clock::now(
                        &governor::clock::DefaultClock::default(),
                    ));
                    let retry_after = wait_time.as_secs().max(1);

                    let (status, error) = ApiError::rate_limited(retry_after);
                    let mut response = (status, axum::Json(error)).into_response();

                    // Add Retry-After header
                    if let Ok(header_value) = retry_after.to_string().parse() {
                        response.headers_mut().insert("Retry-After", header_value);
                    }

                    Ok(response)
                }
            }
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 200);
    }

    #[test]
    fn test_custom_config() {
        let config = RateLimitConfig::new(50, 100);
        assert_eq!(config.requests_per_second, 50);
        assert_eq!(config.burst_size, 100);
    }

    #[test]
    fn test_create_limiter() {
        let config = RateLimitConfig::default();
        let limiter = config.create_limiter();
        // Should allow first request
        assert!(limiter.check().is_ok());
    }
}
