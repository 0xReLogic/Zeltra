# Requirements Document

## Introduction

This document specifies the requirements for verifying the remaining Phase 6-7 frontend features against the real backend API. These features have UI implementations but have not been tested with the actual API endpoints. The goal is to ensure all frontend components correctly integrate with backend services.

## Glossary

- **Frontend**: Next.js 16 application with React 19, TanStack Query, and Shadcn/UI
- **Backend**: Rust Axum API server (Zeltra)
- **Simulation**: Financial forecasting feature that projects future values based on historical data
- **Account_Ledger**: Report showing all entries for a specific account with running balance
- **Dimensional_Report**: Report that slices financial data by dimension (Department/Project/Cost Center)
- **Fiscal_Year**: Accounting year containing 12 monthly periods (optionally 13 with adjustment period)
- **Attachment**: File (PDF, image, document) linked to a transaction
- **OpenAPI_Types**: Auto-generated TypeScript types from `contracts/openapi.yaml`

## Requirements

### Requirement 1: Simulation API Verification

**User Story:** As a finance manager, I want to run financial simulations, so that I can forecast future cash flow and budget scenarios.

#### Acceptance Criteria

1. WHEN a user submits simulation parameters THEN the Frontend SHALL call `POST /organizations/{org_id}/simulation/run` with correct payload
2. WHEN the simulation API returns results THEN the Frontend SHALL display projection data in charts and tables
3. WHEN the simulation API returns an error THEN the Frontend SHALL display appropriate error message
4. THE Frontend SHALL use OpenAPI_Types for simulation request and response payloads

### Requirement 2: Attachments API Verification

**User Story:** As an accountant, I want to attach files to transactions, so that I can keep supporting documents with journal entries.

#### Acceptance Criteria

1. WHEN a user uploads a file THEN the Frontend SHALL call `POST /organizations/{org_id}/attachments/upload` to get presigned URL
2. WHEN presigned URL is received THEN the Frontend SHALL upload file directly to storage
3. WHEN upload completes THEN the Frontend SHALL call `POST /organizations/{org_id}/attachments` to confirm and link to transaction
4. WHEN viewing transaction detail THEN the Frontend SHALL call `GET /organizations/{org_id}/transactions/{id}/attachments` to list attachments
5. WHEN a user clicks download THEN the Frontend SHALL call `GET /organizations/{org_id}/attachments/{id}` to get presigned download URL
6. WHEN a user deletes attachment THEN the Frontend SHALL call `DELETE /organizations/{org_id}/attachments/{id}`
7. IF file type is not allowed THEN the Frontend SHALL display validation error before upload
8. IF file size exceeds limit THEN the Frontend SHALL display validation error before upload

### Requirement 3: Account Ledger View API Verification

**User Story:** As an accountant, I want to view the ledger for a specific account, so that I can see all entries and running balance.

#### Acceptance Criteria

1. WHEN a user selects an account THEN the Frontend SHALL call `GET /organizations/{org_id}/accounts/{id}/ledger` with optional date filters
2. WHEN the API returns ledger entries THEN the Frontend SHALL display entries with debit, credit, and running balance columns
3. WHEN the API returns empty results THEN the Frontend SHALL display "No entries found" message
4. THE Frontend SHALL support date range filtering via query parameters
5. THE Frontend SHALL use OpenAPI_Types for ledger response payload

### Requirement 4: Dimensional Reports API Verification

**User Story:** As a finance manager, I want to view reports sliced by dimension, so that I can analyze expenses by department, project, or cost center.

#### Acceptance Criteria

1. WHEN a user selects dimension type and value THEN the Frontend SHALL call `GET /organizations/{org_id}/reports/dimensional` with filters
2. WHEN the API returns dimensional data THEN the Frontend SHALL display grouped expenses in chart and table format
3. WHEN comparing across dimensions THEN the Frontend SHALL display comparison view
4. THE Frontend SHALL support filtering by dimension_type_id and dimension_value_id
5. THE Frontend SHALL use OpenAPI_Types for dimensional report response

### Requirement 5: Fiscal Year Creation API Verification

**User Story:** As an admin, I want to create a new fiscal year, so that I can set up accounting periods for the upcoming year.

#### Acceptance Criteria

1. WHEN a user submits fiscal year form THEN the Frontend SHALL call `POST /organizations/{org_id}/fiscal-years` with name, start_date, end_date
2. WHEN the API successfully creates fiscal year THEN the Frontend SHALL display success toast and refresh fiscal years list
3. WHEN the API returns validation error THEN the Frontend SHALL display specific error message
4. THE Frontend SHALL support optional adjustment period (period 13) flag
5. THE Frontend SHALL auto-generate 12 monthly periods based on start/end dates
6. THE Frontend SHALL use OpenAPI_Types for fiscal year request and response
