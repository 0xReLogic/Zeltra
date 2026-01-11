# Requirements Document

## Introduction

Phase 5 of Zeltra Backend focuses on completing the API layer with file attachments, live exchange rate integration, missing master data endpoints, dashboard analytics, and API polish including documentation and load testing. This phase transforms the backend from feature-complete to production-ready.

## Glossary

- **Storage_Service**: Abstraction layer over Apache OpenDAL for vendor-agnostic file storage operations
- **Attachment**: File uploaded and linked to a transaction (receipts, invoices, documents)
- **Presigned_URL**: Time-limited URL for direct upload/download to storage without proxying through backend
- **Exchange_Rate_Fetcher**: Service that retrieves live exchange rates from Frankfurter API
- **Frankfurter_API**: Open-source exchange rate API using ECB data (self-hosted or public)
- **Dashboard_Service**: Service that aggregates metrics for dashboard display
- **Activity_Log**: Chronological record of system events (transactions, budgets, users)
- **Rate_Limiter**: Middleware that restricts request frequency per client
- **OpenDAL**: Apache project providing unified storage API for 40+ backends (Azure Blob, S3, R2, local filesystem)

## Requirements

### Requirement 1: File Attachment Storage

**User Story:** As an accountant, I want to attach receipts and invoices to transactions, so that I have supporting documentation for audit purposes.

#### Acceptance Criteria

1. WHEN a user requests an upload URL, THE Storage_Service SHALL generate a presigned URL valid for 15 minutes
2. WHEN a user confirms an upload, THE Attachment_Service SHALL validate the file exists in storage and create a database record
3. WHEN a user requests to download an attachment, THE Storage_Service SHALL generate a presigned download URL valid for 1 hour
4. WHEN a user deletes an attachment, THE Attachment_Service SHALL remove both the database record and the storage object
5. WHEN an attachment is uploaded, THE Attachment_Service SHALL validate file type against allowed MIME types (PDF, PNG, JPG, JPEG, GIF, WEBP, DOC, DOCX, XLS, XLSX)
6. WHEN an attachment exceeds the size limit (10MB default), THE Attachment_Service SHALL reject the upload with a clear error message
7. THE Storage_Service SHALL support multiple backends (Azure Blob, S3/R2, local filesystem) via configuration only
8. FOR ALL attachments, THE storage_key SHALL follow the pattern: `{org_id}/{transaction_id}/{attachment_id}/{filename}`

### Requirement 2: Live Exchange Rate Integration

**User Story:** As a finance manager, I want exchange rates to be automatically fetched from a reliable source, so that I don't have to manually enter rates daily.

#### Acceptance Criteria

1. WHEN an admin triggers a rate fetch, THE Exchange_Rate_Fetcher SHALL retrieve rates from Frankfurter API for the organization's base currency
2. WHEN rates are fetched, THE Exchange_Rate_Fetcher SHALL store them in the exchange_rates table with source='frankfurter'
3. WHEN the Frankfurter API is unavailable, THE Exchange_Rate_Fetcher SHALL return an error without affecting existing rates
4. WHEN bulk importing rates, THE Exchange_Rate_Service SHALL validate all rates before inserting any (atomic operation)
5. WHEN a rate already exists for the same currency pair and date, THE Exchange_Rate_Service SHALL update the existing rate
6. THE Exchange_Rate_Fetcher SHALL support configurable rate source (mock, frankfurter, manual) via environment variable

### Requirement 3: Missing Master Data APIs

**User Story:** As an admin, I want to toggle account and dimension statuses, so that I can manage master data without deleting records.

#### Acceptance Criteria

1. WHEN an admin toggles an account status, THE Account_Service SHALL update is_active and return the updated account
2. WHEN an admin updates a dimension value, THE Dimension_Service SHALL update the name and/or code fields
3. WHEN an admin toggles a dimension value status, THE Dimension_Service SHALL update is_active and return the updated value
4. WHEN deactivating an account with posted transactions, THE Account_Service SHALL allow deactivation but prevent new postings
5. WHEN deactivating a dimension value linked to entries, THE Dimension_Service SHALL allow deactivation but prevent new assignments

### Requirement 4: Dashboard Analytics

**User Story:** As a finance manager, I want to see key metrics on my dashboard, so that I can quickly understand the financial health of my organization.

#### Acceptance Criteria

1. WHEN a user requests dashboard metrics, THE Dashboard_Service SHALL return cash position, burn rate, runway days, and pending approvals
2. WHEN calculating cash position, THE Dashboard_Service SHALL sum balances of all accounts with subtype 'cash' or 'bank'
3. WHEN calculating burn rate, THE Dashboard_Service SHALL compute average daily expenses over the last 30 days
4. WHEN calculating runway days, THE Dashboard_Service SHALL divide cash position by daily burn rate
5. WHEN a user requests cash flow data, THE Dashboard_Service SHALL return monthly inflow/outflow for the specified period
6. WHEN a user requests recent activity, THE Dashboard_Service SHALL return paginated activity items with cursor-based pagination
7. WHEN a user requests budget vs actual summary, THE Dashboard_Service SHALL return aggregated variance data by department

### Requirement 5: API Documentation and Polish

**User Story:** As a developer integrating with Zeltra, I want comprehensive API documentation, so that I can understand and use the API correctly.

#### Acceptance Criteria

1. THE API SHALL return consistent error responses with code, message, and optional details across all endpoints
2. WHEN a client exceeds rate limits, THE Rate_Limiter SHALL return HTTP 429 with Retry-After header
3. THE API SHALL log all requests with request_id, duration, status_code, and user_id
4. WHEN an invalid JWT is provided, THE Auth_Middleware SHALL return HTTP 401 with clear error message
5. WHEN a user lacks permission, THE Auth_Middleware SHALL return HTTP 403 with required role information

### Requirement 6: Load and Security Testing

**User Story:** As a system administrator, I want the API to handle concurrent load securely, so that the system remains stable under production traffic.

#### Acceptance Criteria

1. THE API SHALL handle 100 concurrent users with p95 latency under 500ms for standard operations
2. THE API SHALL reject SQL injection attempts in all query parameters and request bodies
3. THE API SHALL prevent cross-tenant data access through RLS enforcement
4. THE API SHALL validate and sanitize all user inputs before processing
5. WHEN stress testing concurrent transactions, THE Ledger_Service SHALL maintain balance integrity

