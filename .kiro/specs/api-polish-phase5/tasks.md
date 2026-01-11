# Implementation Plan: API Polish & Attachments (Phase 5)

## Overview

This implementation plan covers Phase 5 of Zeltra Backend: file attachments with OpenDAL, live exchange rates via Frankfurter, missing master data APIs, dashboard analytics, and API polish.

## Context Recovery Instructions

**IMPORTANT:** If you lose context during implementation:
1. Re-read `.kiro/specs/api-polish-phase5/requirements.md` for acceptance criteria
2. Re-read `.kiro/specs/api-polish-phase5/design.md` for architecture and interfaces
3. Check `PROGRESS.md` for current status
4. Check `contracts/openapi.yaml` for API specifications

## Code Quality Rules

- **ALWAYS** run `cargo fmt` before committing
- **ALWAYS** run `cargo clippy -- -D warnings` and fix ALL warnings
- **NEVER** use `#[allow(...)]` to suppress fatal warnings - fix the root cause
- **NEVER** use `.unwrap()` in production code - use proper error handling
- **NEVER** use `.clone()` unnecessarily - prefer references

## Tasks

- [x] 1. Setup OpenDAL Storage Infrastructure
  - [x] 1.1 Add OpenDAL dependencies to Cargo.toml
    - Add `opendal` with features: `services-azblob`, `services-s3`, `services-fs`
    - Add `opendal` feature `services-vercel-blob` for Vercel support
    - _Requirements: 1.7_
  - [x] 1.2 Create StorageConfig and StorageProvider enum
    - Implement config parsing from environment variables
    - Support: S3 (R2/Supabase), AzureBlob, VercelBlob, LocalFs
    - _Requirements: 1.7_
  - [x] 1.3 Implement StorageService wrapper
    - Create `from_config()` to initialize OpenDAL Operator
    - Implement `presign_upload()` with configurable TTL
    - Implement `presign_download()` with configurable TTL
    - Implement `verify_upload()` to check file exists
    - Implement `delete()` to remove file
    - _Requirements: 1.1, 1.3_
  - [x] 1.4 Write property tests for StorageService
    - **Property 1: Presigned URL TTL Validity**
    - **Validates: Requirements 1.1, 1.3**

- [x] 2. Implement Attachment Service
  - [x] 2.1 Create AttachmentService in core crate
    - Implement `validate_upload()` for MIME type and size checks
    - Implement `request_upload()` to generate presigned URL
    - Implement `confirm_upload()` to verify and create DB record
    - Implement `get_download_url()` to generate download URL
    - Implement `delete()` to remove attachment and storage object
    - Implement `list_by_transaction()` to list attachments
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  - [x] 2.2 Create AttachmentRepository in db crate
    - Implement CRUD operations for attachments table
    - Use existing SeaORM entity (already in schema)
    - _Requirements: 1.2, 1.4_
  - [x] 2.3 Implement storage key generation
    - Format: `{org_id}/{transaction_id}/{attachment_id}/{sanitized_filename}`
    - Sanitize filename to remove special characters
    - _Requirements: 1.8_
  - [x] 2.4 Write property tests for Attachment validation
    - **Property 2: MIME Type Validation**
    - **Property 3: File Size Validation**
    - **Property 4: Storage Key Format**
    - **Validates: Requirements 1.5, 1.6, 1.8**

- [x] 3. Implement Attachment API Routes
  - [x] 3.1 Create attachment routes in api crate
    - `POST /transactions/:id/attachments/upload` - Request upload URL
    - `POST /transactions/:id/attachments` - Confirm upload
    - `GET /transactions/:id/attachments` - List attachments
    - `GET /attachments/:id` - Get attachment with download URL
    - `DELETE /attachments/:id` - Delete attachment
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - [x] 3.2 Write integration tests for attachment endpoints
    - Test upload flow with mock storage
    - Test download URL generation
    - Test deletion
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 4. Checkpoint - Attachment System
  - Run `cargo fmt` to format code
  - Run `cargo clippy -- -D warnings` and fix all warnings (no `#[allow]` for fatal warnings)
  - Ensure all attachment tests pass
  - Test with local filesystem storage
  - If context is lost, re-read `.kiro/specs/api-polish-phase5/requirements.md` and `design.md`
  - Ask user if questions arise

- [ ] 5. Implement Exchange Rate Fetcher
  - [ ] 5.1 Create ExchangeRateFetcher service
    - Implement Frankfurter API client using reqwest
    - Parse JSON response to FetchedRate structs
    - Handle API errors gracefully
    - _Requirements: 2.1, 2.3_
  - [ ] 5.2 Implement rate storage logic
    - Store fetched rates with source='frankfurter'
    - Implement upsert logic for existing rates
    - _Requirements: 2.2, 2.5_
  - [ ] 5.3 Implement bulk rate import
    - Validate all rates before inserting (atomic)
    - Return detailed error report for invalid rates
    - _Requirements: 2.4_
  - [ ] 5.4 Write property tests for exchange rate operations
    - **Property 5: Bulk Rate Import Atomicity**
    - **Property 6: Rate Upsert Behavior**
    - **Property 7: External Service Failure Isolation**
    - **Validates: Requirements 2.3, 2.4, 2.5**

- [ ] 6. Implement Exchange Rate API Routes
  - [ ] 6.1 Create exchange rate routes
    - `POST /exchange-rates/fetch` - Trigger Frankfurter fetch
    - `POST /exchange-rates/bulk` - Bulk import rates
    - _Requirements: 2.1, 2.4_
  - [ ] 6.2 Write integration tests for exchange rate endpoints
    - Test fetch with mock Frankfurter API
    - Test bulk import validation
    - _Requirements: 2.1, 2.4_

- [ ] 7. Checkpoint - Exchange Rate System
  - Run `cargo fmt` to format code
  - Run `cargo clippy -- -D warnings` and fix all warnings (no `#[allow]` for fatal warnings)
  - Ensure all exchange rate tests pass
  - Test with mock Frankfurter API
  - If context is lost, re-read `.kiro/specs/api-polish-phase5/requirements.md` and `design.md`
  - Ask user if questions arise

- [ ] 8. Implement Missing Master Data APIs
  - [ ] 8.1 Add account status toggle endpoint
    - `PATCH /accounts/:id/status` - Toggle is_active
    - Validate account exists and belongs to org
    - _Requirements: 3.1_
  - [ ] 8.2 Add dimension value update endpoints
    - `PATCH /dimension-values/:id` - Update name/code
    - `PATCH /dimension-values/:id/status` - Toggle is_active
    - _Requirements: 3.2, 3.3_
  - [ ] 8.3 Implement deactivation validation
    - Allow deactivation of accounts with posted transactions
    - Prevent new postings to deactivated accounts
    - Prevent new assignments to deactivated dimension values
    - _Requirements: 3.4, 3.5_
  - [ ] 8.4 Write property tests for master data operations
    - **Property 8: Toggle Status Idempotence**
    - **Property 9: Deactivated Entity Usage Prevention**
    - **Validates: Requirements 3.1, 3.3, 3.4, 3.5**

- [ ] 9. Checkpoint - Master Data APIs
  - Run `cargo fmt` to format code
  - Run `cargo clippy -- -D warnings` and fix all warnings (no `#[allow]` for fatal warnings)
  - Ensure all master data tests pass
  - Update OpenAPI spec with new endpoints
  - If context is lost, re-read `.kiro/specs/api-polish-phase5/requirements.md` and `design.md`
  - Ask user if questions arise

- [ ] 10. Implement Dashboard Service
  - [ ] 10.1 Create DashboardService in core crate
    - Implement `get_metrics()` for cash position, burn rate, runway
    - Implement `get_cash_flow()` for monthly inflow/outflow
    - Implement `get_recent_activity()` with cursor pagination
    - Implement `get_budget_vs_actual()` for variance summary
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_
  - [ ] 10.2 Create DashboardRepository in db crate
    - Implement cash position query (sum of cash/bank accounts)
    - Implement expense aggregation for burn rate
    - Implement activity log query with cursor
    - Implement budget variance aggregation
    - _Requirements: 4.2, 4.3, 4.5, 4.6, 4.7_
  - [ ] 10.3 Write property tests for dashboard calculations
    - **Property 10: Cash Position Calculation**
    - **Property 11: Burn Rate Calculation**
    - **Property 12: Runway Days Calculation**
    - **Property 13: Cash Flow Aggregation**
    - **Property 14: Cursor-Based Pagination Consistency**
    - **Validates: Requirements 4.2, 4.3, 4.4, 4.5, 4.6**

- [ ] 11. Implement Dashboard API Routes
  - [ ] 11.1 Create dashboard routes
    - `GET /dashboard/metrics` - Dashboard metrics
    - `GET /dashboard/cash-flow` - Cash flow chart data
    - `GET /dashboard/recent-activity` - Activity log
    - `GET /dashboard/budget-vs-actual` - Budget summary
    - _Requirements: 4.1, 4.5, 4.6, 4.7_
  - [ ] 11.2 Write integration tests for dashboard endpoints
    - Test with seeded ledger data
    - Test pagination behavior
    - _Requirements: 4.1, 4.5, 4.6, 4.7_

- [ ] 12. Checkpoint - Dashboard System
  - Run `cargo fmt` to format code
  - Run `cargo clippy -- -D warnings` and fix all warnings (no `#[allow]` for fatal warnings)
  - Ensure all dashboard tests pass
  - Verify calculations match expected values
  - If context is lost, re-read `.kiro/specs/api-polish-phase5/requirements.md` and `design.md`
  - Ask user if questions arise

- [ ] 13. Implement API Polish
  - [ ] 13.1 Standardize error responses
    - Create consistent ApiError struct
    - Implement IntoResponse for all error types
    - Add request_id to all error responses
    - _Requirements: 5.1_
  - [ ] 13.2 Add rate limiting middleware
    - Add tower-governor dependency
    - Configure rate limits (requests per second, burst size)
    - Return 429 with Retry-After header
    - _Requirements: 5.2_
  - [ ] 13.3 Add request logging middleware
    - Log request_id, method, path, duration, status
    - Add user_id to log context when authenticated
    - _Requirements: 5.3_
  - [ ] 13.4 Improve auth error responses
    - Return 401 for invalid JWT with clear message
    - Return 403 for insufficient permissions with required role
    - _Requirements: 5.4, 5.5_
  - [ ] 13.5 Write property tests for error handling
    - **Property 15: Error Response Consistency**
    - **Property 16: Auth Error Response**
    - **Validates: Requirements 5.1, 5.4, 5.5**

- [ ] 14. Checkpoint - API Polish
  - Run `cargo fmt` to format code
  - Run `cargo clippy -- -D warnings` and fix all warnings (no `#[allow]` for fatal warnings)
  - Ensure all error handling tests pass
  - Verify rate limiting works correctly
  - If context is lost, re-read `.kiro/specs/api-polish-phase5/requirements.md` and `design.md`
  - Ask user if questions arise

- [ ] 15. Implement Security Tests
  - [ ] 15.1 Write SQL injection prevention tests
    - Test query parameters with injection payloads
    - Test request bodies with injection payloads
    - Verify all inputs are sanitized or rejected
    - _Requirements: 6.2_
  - [ ] 15.2 Write cross-tenant isolation tests
    - Test RLS enforcement across all endpoints
    - Verify users cannot access other org's data
    - _Requirements: 6.3_
  - [ ] 15.3 Write input validation tests
    - Test malicious inputs are sanitized
    - Test boundary conditions
    - _Requirements: 6.4_
  - [ ] 15.4 Write property tests for security
    - **Property 17: SQL Injection Prevention**
    - **Property 18: Cross-Tenant Isolation**
    - **Validates: Requirements 6.2, 6.3**

- [ ] 16. Implement Concurrency Tests
  - [ ] 16.1 Write concurrent transaction stress test
    - Run 100+ concurrent transactions on same account
    - Verify final balance equals expected value
    - _Requirements: 6.5_
  - [ ] 16.2 Write property test for balance integrity
    - **Property 19: Concurrent Transaction Balance Integrity**
    - **Validates: Requirements 6.5**

- [ ] 17. Update OpenAPI Specification
  - [ ] 17.1 Add attachment endpoints to openapi.yaml
    - Document request/response schemas
    - Add error responses
    - _Requirements: 5.1_
  - [ ] 17.2 Add exchange rate endpoints to openapi.yaml
    - Document bulk import schema
    - Document fetch endpoint
    - _Requirements: 2.1, 2.4_
  - [ ] 17.3 Add master data endpoints to openapi.yaml
    - Document status toggle endpoints
    - Document dimension value update
    - _Requirements: 3.1, 3.2, 3.3_
  - [ ] 17.4 Add dashboard endpoints to openapi.yaml
    - Document metrics response schema
    - Document pagination parameters
    - _Requirements: 4.1, 4.5, 4.6, 4.7_

- [ ] 18. Final Checkpoint - Phase 5 Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (no `#[allow]` for fatal warnings)
  - Run `cargo test` - ensure all tests pass (target: 50+ new tests)
  - Run full test suite
  - Update PROGRESS.md with completed endpoints
  - Update contracts/openapi.yaml with all new endpoints
  - If context is lost, re-read `.kiro/specs/api-polish-phase5/requirements.md` and `design.md`
  - Ask user if questions arise

## Notes

- All tasks are required for comprehensive implementation
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation with `cargo fmt` and `cargo clippy`
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- OpenDAL allows switching storage providers via config only - no code changes needed
- Frankfurter API can be self-hosted for production reliability

