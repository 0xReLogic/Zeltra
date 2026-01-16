# Requirements Document

## Introduction

This feature addresses the synchronization issues between backend API responses, OpenAPI specification, and frontend types for the Transactions domain. The audit identified several mismatches that cause frontend errors and inconsistent behavior. This fix will ensure type safety across the stack and validate correctness through E2E testing.

## Glossary

- **Backend**: Rust API server using Axum framework with utoipa for OpenAPI generation
- **OpenAPI**: The OpenAPI 3.0 specification file that serves as the contract between backend and frontend
- **Frontend**: Next.js application using TypeScript with types generated from OpenAPI
- **E2E Testing**: End-to-end testing using Playwright to validate full user flows
- **VoidResponse**: Response structure returned when voiding a transaction
- **PendingTransactionResponse**: Response structure for transactions awaiting approval
- **Split Files**: Domain-specific YAML files extracted from the main openapi.yaml

## Requirements

### Requirement 1: Fix VoidResponse Backend Implementation

**User Story:** As a frontend developer, I want the void transaction endpoint to return the complete VoidResponse structure, so that I can display full transaction details after voiding.

#### Acceptance Criteria

1. WHEN a transaction is voided, THE Backend SHALL return a VoidResponse struct containing full TransactionResponse for both original_transaction and reversing_transaction
2. WHEN the void endpoint returns, THE VoidResponse SHALL include all fields defined in the OpenAPI VoidResponse schema
3. IF the void operation fails, THEN THE Backend SHALL return appropriate error responses as defined in OpenAPI

### Requirement 2: Add Missing Schemas to Transaction Split File

**User Story:** As a developer maintaining the OpenAPI split files, I want all transaction-related schemas in the correct split file, so that the documentation is complete and organized.

#### Acceptance Criteria

1. THE Split_File (02-transactions-schemas.yaml) SHALL contain PaginatedTransactionsResponse schema
2. THE Split_File (02-transactions-schemas.yaml) SHALL contain PaginationMeta schema
3. THE Split_File (02-transactions-schemas.yaml) SHALL contain PendingTransactionResponse schema
4. WHEN the split-openapi.py script runs, THE Script SHALL correctly categorize these schemas into the transactions file

### Requirement 3: Relocate Misplaced Schemas

**User Story:** As a developer, I want schemas organized by their domain, so that the codebase is maintainable and logical.

#### Acceptance Criteria

1. THE LedgerEntryResponse schema SHALL be located in 05-reports-schemas.yaml (not transactions)
2. THE PendingApprovalsResponse schema SHALL be located only in 09-dashboard-schemas.yaml (removed from transactions)
3. WHEN schemas are relocated, THE References ($ref) SHALL remain valid across all split files

### Requirement 4: Fix Frontend Transaction Types

**User Story:** As a frontend developer, I want to use generated types for all transaction responses, so that I have type safety and don't need manual type definitions.

#### Acceptance Criteria

1. THE Frontend SHALL use generated PendingTransactionResponse type instead of manual GetPendingTransactionsResponse interface
2. THE api-helpers.ts SHALL export PayInvoiceRequest type
3. THE api-helpers.ts SHALL export PendingTransactionResponse type
4. WHEN OpenAPI types are regenerated, THE Frontend types SHALL match the backend response structures

### Requirement 5: E2E Testing for Transaction Flows

**User Story:** As a QA engineer, I want E2E tests covering critical transaction flows, so that I can verify the frontend-backend integration works correctly.

#### Acceptance Criteria

1. WHEN a user creates a transaction, THE E2E_Test SHALL verify the response matches TransactionResponse schema
2. WHEN a user lists transactions, THE E2E_Test SHALL verify pagination works correctly with PaginatedTransactionsResponse
3. WHEN a user voids a transaction, THE E2E_Test SHALL verify the response contains full VoidResponse with both transactions
4. WHEN a user views pending transactions, THE E2E_Test SHALL verify the response matches PendingTransactionResponse[] structure
5. WHEN a user bulk approves transactions, THE E2E_Test SHALL verify the response matches BulkApproveResponse schema

### Requirement 6: Regenerate and Validate Types

**User Story:** As a developer, I want automated type generation that produces correct types, so that frontend and backend stay in sync.

#### Acceptance Criteria

1. WHEN the OpenAPI spec is updated, THE Generator SHALL produce updated api.generated.ts
2. WHEN types are regenerated, THE Frontend build SHALL pass without type errors
3. WHEN the backend OpenAPI is regenerated, THE Output SHALL match the expected schema structure

### Requirement 7: Transaction UI/UX Improvements

**User Story:** As a user, I want a smooth and error-free transaction management experience, so that I can efficiently manage my financial transactions.

#### Acceptance Criteria

1. WHEN a user creates a transaction, THE UI SHALL display proper loading states and success/error feedback
2. WHEN a user views the transaction list, THE UI SHALL display pagination controls that work correctly with the API response
3. WHEN a user voids a transaction, THE UI SHALL display both the voided transaction and the reversing transaction details
4. WHEN a user views pending approvals, THE UI SHALL correctly show the can_approve status for each transaction
5. WHEN a user performs bulk approval, THE UI SHALL display individual success/failure status for each transaction
6. WHEN API errors occur, THE UI SHALL display user-friendly error messages based on the error response
7. WHEN transaction data is loading, THE UI SHALL display appropriate skeleton loaders or loading indicators
8. WHEN a transaction action completes, THE UI SHALL automatically refresh the relevant data without full page reload

### Requirement 8: Transaction Form Validation

**User Story:** As a user, I want clear validation feedback when creating transactions, so that I can correct errors before submission.

#### Acceptance Criteria

1. WHEN a user enters invalid data in transaction form, THE UI SHALL display inline validation errors
2. WHEN transaction entries don't balance (debits != credits), THE UI SHALL prevent submission and show balance error
3. WHEN required fields are missing, THE UI SHALL highlight the missing fields with clear error messages
4. WHEN amount format is invalid, THE UI SHALL show format guidance to the user
5. WHEN a user selects an account, THE UI SHALL validate the account exists and is active
