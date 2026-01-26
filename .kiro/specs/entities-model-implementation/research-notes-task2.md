# Task 2 Research Notes: Axum and utoipa Patterns for 2025-2026

## Research Date: 2026-01-25

### Axum Middleware Best Practices (2025-2026)

**Key Findings:**

1. **Middleware Stack Ordering** (from GitHub discussion #3202):
   - Middleware should be applied in the correct order: logging/CORS → authentication → routing
   - Use `tower::ServiceBuilder` for composing middleware layers
   - Apply global middleware before routing for consistent behavior

2. **Modern Middleware Patterns**:
   - Use `axum::middleware::from_fn` for custom middleware
   - Leverage `tower-http` for common middleware (CORS, tracing, compression)
   - Use `State` extractor for sharing application state
   - Implement `IntoResponse` for custom error types

3. **Performance Considerations**:
   - Tokio `JoinSet` for batch async task management (better than manual `JoinHandle`)
   - Automatic task cancellation via Drop trait
   - Zero-cost abstractions with compile-time guarantees

### utoipa OpenAPI Generation Patterns (2025)

**Key Findings:**

1. **Current Version**: utoipa 5.4.0 (as of June 2025)
   - Stable and widely adopted (1M+ downloads/month)
   - Used in 1,354 crates
   - MIT/Apache-2.0 licensed

2. **Best Practices**:
   - Use `#[derive(OpenApi)]` for API documentation struct
   - Annotate handlers with `#[utoipa::path]` macro
   - Use `#[derive(ToSchema)]` for request/response types
   - Integrate with Axum via `utoipa-axum` crate
   - Generate OpenAPI spec at compile time (code-first approach)

3. **Common Patterns**:
   ```rust
   #[derive(OpenApi)]
   #[openapi(
       paths(list_entities, create_entity, get_entity),
       components(schemas(Entity, CreateEntityRequest, EntityResponse))
   )]
   struct ApiDoc;
   ```

4. **Integration Tips**:
   - Use `utoipa-swagger-ui` for interactive documentation
   - Manually register paths in OpenApi struct (can be tedious for large APIs)
   - Consider using helper macros for reducing boilerplate

### Rust API Error Handling Best Practices (2025)

**Key Findings:**

1. **IntoResponse Pattern** (Elegant Error Handling):
   - Implement `IntoResponse` trait for custom error types
   - Use enum for different error variants
   - Map errors to appropriate HTTP status codes
   - Return structured JSON error responses

2. **Modern Error Handling**:
   ```rust
   pub enum AppError {
       NotFound,
       Unauthorized,
       BadRequest(String),
       Internal(String),
   }
   
   impl IntoResponse for AppError {
       fn into_response(self) -> Response {
           let (status, message) = match self {
               AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
               AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
               // ...
           };
           (status, Json(json!({ "error": message }))).into_response()
       }
   }
   ```

3. **Best Practices**:
   - Use `?` operator for error propagation
   - Wrap `anyhow::Error` in custom newtype for web handlers
   - Implement `From<E>` for automatic error conversion
   - Return `Result<Json<T>, AppError>` from handlers
   - Avoid exposing internal error details to clients

4. **Error Response Structure**:
   - Include error code (machine-readable)
   - Include error message (human-readable)
   - Optional: Include request_id for tracing
   - Optional: Include validation errors for 400 responses

### Implementation Recommendations for Task 2

1. **Entity API Routes**:
   - Use `#[utoipa::path]` on all entity endpoints
   - Implement custom `AppError` type with `IntoResponse`
   - Use `State(AppState)` for database access
   - Use `Path` and `Json` extractors for parameters
   - Return `Result<Json<T>, AppError>` from handlers

2. **Subscription Middleware**:
   - Use `axum::middleware::from_fn` for subscription check
   - Extract user from request extensions (set by auth middleware)
   - Query user's subscription status from database
   - Return HTTP 402 for expired/cancelled subscriptions
   - Use `next.run(request).await` to continue chain

3. **OpenAPI Documentation**:
   - Create `ApiDoc` struct with `#[derive(OpenApi)]`
   - Register all entity routes in `paths(...)`
   - Register all schemas in `components(schemas(...))`
   - Add examples to request/response types
   - Generate spec with `cargo run --bin generate-openapi`

4. **Error Handling**:
   - Define error codes: `ENTITY_LIMIT_EXCEEDED`, `DUPLICATE_ENTITY_NAME`, etc.
   - Map database errors to appropriate HTTP status codes
   - Include clear error messages for client consumption
   - Log internal errors for debugging (don't expose to client)

### References

- Axum Documentation: https://docs.rs/axum/latest/axum/
- utoipa Documentation: https://docs.rs/utoipa/latest/utoipa/
- Axum Middleware Guide: https://docs.rs/axum/latest/axum/middleware/
- Error Handling in Axum: https://docs.rs/axum/latest/axum/error_handling/
- utoipa GitHub: https://github.com/juhaku/utoipa

### Notes

- Axum 0.8.8 is the latest stable version (as of research date)
- utoipa 5.4.0 is the latest stable version
- Both crates are actively maintained and widely used
- No breaking changes expected in near future
- Focus on compile-time safety and zero-cost abstractions
