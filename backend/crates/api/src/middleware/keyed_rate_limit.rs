//! Keyed rate limiting middleware for per-user/per-IP rate limiting.
//!
//! This module provides rate limiting based on a key (user ID or IP address),
//! allowing different rate limits for different users or IP addresses.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{HeaderMap, Request},
    response::{IntoResponse, Response},
};
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState};
use std::{
    net::SocketAddr,
    num::NonZeroU32,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use uuid::Uuid;

use crate::error::ApiError;

/// Keyed rate limiter using DashMap for concurrent access.
///
/// This uses governor's built-in keyed state store with DashMap for
/// high-performance concurrent access across multiple threads.
pub type KeyedRateLimiter =
    RateLimiter<String, dashmap::DashMap<String, InMemoryState>, DefaultClock>;

/// Configuration for keyed rate limiting.
#[derive(Debug, Clone, Copy)]
pub struct KeyedRateLimitConfig {
    /// Requests per second per key.
    pub requests_per_second: u32,
    /// Burst size per key.
    pub burst_size: u32,
}

impl Default for KeyedRateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
        }
    }
}

impl KeyedRateLimitConfig {
    /// Creates a new keyed rate limit config.
    #[must_use]
    pub const fn new(requests_per_second: u32, burst_size: u32) -> Self {
        Self {
            requests_per_second,
            burst_size,
        }
    }

    /// Creates a keyed rate limiter from this config.
    #[must_use]
    pub fn create_limiter(&self) -> Arc<KeyedRateLimiter> {
        let quota =
            Quota::per_second(NonZeroU32::new(self.requests_per_second).unwrap_or(NonZeroU32::MIN))
                .allow_burst(NonZeroU32::new(self.burst_size).unwrap_or(NonZeroU32::MIN));

        Arc::new(RateLimiter::dashmap(quota))
    }
}

/// Extracts a rate limit key from the request.
///
/// Priority order:
/// 1. User ID from authenticated user (if available)
/// 2. IP address from X-Forwarded-For header
/// 3. IP address from connection info
/// 4. "anonymous" as fallback
pub fn extract_rate_limit_key(
    headers: &HeaderMap,
    user_id: Option<Uuid>,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> String {
    // Priority 1: User ID (most specific)
    if let Some(id) = user_id {
        return format!("user:{id}");
    }

    // Priority 2: X-Forwarded-For header (for proxied requests)
    if let Some(forwarded) = headers.get("x-forwarded-for")
        && let Ok(forwarded_str) = forwarded.to_str()
    {
        // Take the first IP in the chain (client IP)
        if let Some(client_ip) = forwarded_str.split(',').next() {
            return format!("ip:{}", client_ip.trim());
        }
    }

    // Priority 3: Direct connection IP
    if let Some(ConnectInfo(addr)) = connect_info {
        return format!("ip:{}", addr.ip());
    }

    // Fallback: anonymous (least specific)
    "anonymous".to_string()
}

/// Keyed rate limiting layer.
#[derive(Clone)]
pub struct KeyedRateLimitLayer {
    limiter: Arc<KeyedRateLimiter>,
}

impl KeyedRateLimitLayer {
    /// Creates a new keyed rate limit layer.
    #[must_use]
    pub fn new(limiter: Arc<KeyedRateLimiter>) -> Self {
        Self { limiter }
    }

    /// Creates a keyed rate limit layer with default config.
    #[must_use]
    pub fn with_default_config() -> Self {
        Self::new(KeyedRateLimitConfig::default().create_limiter())
    }

    /// Creates a keyed rate limit layer with custom config.
    #[must_use]
    pub fn with_config(config: KeyedRateLimitConfig) -> Self {
        Self::new(config.create_limiter())
    }
}

impl<S> Layer<S> for KeyedRateLimitLayer {
    type Service = KeyedRateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        KeyedRateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Keyed rate limiting service.
#[derive(Clone)]
pub struct KeyedRateLimitService<S> {
    inner: S,
    limiter: Arc<KeyedRateLimiter>,
}

impl<S> Service<Request<Body>> for KeyedRateLimitService<S>
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
            // Extract rate limit key from request
            let headers = req.headers();
            let connect_info = req.extensions().get::<ConnectInfo<SocketAddr>>().copied();

            // TODO: Extract user_id from auth extension when available
            let user_id = None;

            let key = extract_rate_limit_key(headers, user_id, connect_info.as_ref());

            // Check rate limit for this key
            match limiter.check_key(&key) {
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

                    // Add rate limit key to response headers for debugging
                    if let Ok(key_value) = key.parse() {
                        response.headers_mut().insert("X-RateLimit-Key", key_value);
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
        let config = KeyedRateLimitConfig::default();
        assert_eq!(config.requests_per_second, 10);
        assert_eq!(config.burst_size, 20);
    }

    #[test]
    fn test_custom_config() {
        let config = KeyedRateLimitConfig::new(5, 10);
        assert_eq!(config.requests_per_second, 5);
        assert_eq!(config.burst_size, 10);
    }

    #[test]
    fn test_create_limiter() {
        let config = KeyedRateLimitConfig::default();
        let limiter = config.create_limiter();

        // Should allow first request for a key
        assert!(limiter.check_key(&"test-key".to_string()).is_ok());
    }

    #[test]
    fn test_extract_key_with_user_id() {
        let headers = HeaderMap::new();
        let user_id = Some(Uuid::new_v4());
        let key = extract_rate_limit_key(&headers, user_id, None);

        assert!(key.starts_with("user:"));
    }

    #[test]
    fn test_extract_key_with_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1, 10.0.0.1".parse().unwrap());

        let key = extract_rate_limit_key(&headers, None, None);

        assert_eq!(key, "ip:192.168.1.1");
    }

    #[test]
    fn test_extract_key_fallback() {
        let headers = HeaderMap::new();
        let key = extract_rate_limit_key(&headers, None, None);

        assert_eq!(key, "anonymous");
    }

    #[test]
    fn test_per_key_rate_limiting() {
        let config = KeyedRateLimitConfig::new(2, 2);
        let limiter = config.create_limiter();

        // Key 1: Should allow 2 requests
        assert!(limiter.check_key(&"key1".to_string()).is_ok());
        assert!(limiter.check_key(&"key1".to_string()).is_ok());
        assert!(limiter.check_key(&"key1".to_string()).is_err()); // 3rd should fail

        // Key 2: Should still allow 2 requests (independent limit)
        assert!(limiter.check_key(&"key2".to_string()).is_ok());
        assert!(limiter.check_key(&"key2".to_string()).is_ok());
        assert!(limiter.check_key(&"key2".to_string()).is_err()); // 3rd should fail
    }
}
