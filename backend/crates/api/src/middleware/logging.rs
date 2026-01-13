//! Request logging middleware.
//!
//! Implements Requirements 5.3 for request logging.

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response},
};
use tower_http::trace::{MakeSpan, OnRequest, OnResponse};
use tracing::{Level, Span, info, warn};
use uuid::Uuid;

/// Request ID header name.
pub const REQUEST_ID_HEADER: &str = "X-Request-Id";

/// Custom span maker that includes request_id.
#[derive(Clone, Debug)]
pub struct RequestSpan;

impl<B> MakeSpan<B> for RequestSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map_or_else(|| Uuid::new_v4().to_string(), ToString::to_string);

        let path = request
            .extensions()
            .get::<MatchedPath>()
            .map_or(request.uri().path(), MatchedPath::as_str);

        tracing::span!(
            Level::INFO,
            "request",
            request_id = %request_id,
            method = %request.method(),
            path = %path,
            version = ?request.version(),
        )
    }
}

/// Custom request logger.
#[derive(Clone, Debug)]
pub struct RequestLogger;

impl<B> OnRequest<B> for RequestLogger {
    fn on_request(&mut self, request: &Request<B>, _span: &Span) {
        let method = request.method();
        let uri = request.uri();

        info!(
            method = %method,
            uri = %uri,
            "Started processing request"
        );
    }
}

/// Custom response logger.
#[derive(Clone, Debug)]
pub struct ResponseLogger;

impl<B> OnResponse<B> for ResponseLogger {
    fn on_response(self, response: &Response<B>, latency: std::time::Duration, _span: &Span) {
        let status = response.status();
        let latency_ms = latency.as_millis();

        if status.is_success() {
            info!(
                status = %status.as_u16(),
                latency_ms = %latency_ms,
                "Request completed"
            );
        } else if status.is_client_error() {
            warn!(
                status = %status.as_u16(),
                latency_ms = %latency_ms,
                "Client error"
            );
        } else {
            warn!(
                status = %status.as_u16(),
                latency_ms = %latency_ms,
                "Server error"
            );
        }
    }
}

/// Middleware to inject request ID if not present.
pub async fn inject_request_id(
    mut request: Request<Body>,
    next: axum::middleware::Next,
) -> Response<Body> {
    // Generate request ID if not present
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| Uuid::new_v4().to_string(), ToString::to_string);

    // Insert request ID header if not present
    if !request.headers().contains_key(REQUEST_ID_HEADER)
        && let Ok(value) = request_id.parse()
    {
        request.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    // Store request ID in extensions for later use
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Process request
    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(value) = request_id.parse() {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    response
}

/// Request ID extension.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    /// Gets the request ID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_header() {
        assert_eq!(REQUEST_ID_HEADER, "X-Request-Id");
    }

    #[test]
    fn test_request_id_struct() {
        let id = RequestId("test-123".to_string());
        assert_eq!(id.as_str(), "test-123");
    }
}
