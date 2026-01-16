# Implementation Plan: Audit Fix Transactions

## Overview

This implementation plan addresses synchronization issues between backend API, OpenAPI specification, and frontend types for the Transactions domain. Tasks are ordered to fix backend first, then OpenAPI, then frontend, followed by E2E testing.

## Tasks

- [x] 1. Fix Backend VoidResponse Implementation
  - [x] 1.1 Update void_transaction handler to return proper VoidResponse struct
    - Modify `backend/crates/api/src/routes/transactions.rs`
    - Replace inline JSON with VoidResponse struct using map_transaction_to_response
    - Fetch full transaction data for both original and reversing transactions
    - _Requirements: 1.1, 1.2, 1.3_

  - [x] 1.2 Run backend tests to verify changes
    - Execute `cargo test` in backend directory
    - Ensure all existing tests pass
    - _Requirements: 1.1, 1.2_

- [x] 2. Update OpenAPI Split Script and Regenerate
  - [x] 2.1 Update split-openapi.py to correctly categorize schemas
    - Add PaginatedTransactionsResponse, PaginationMeta, PendingTransactionResponse to transaction schemas
    - Move LedgerEntryResponse to report schemas
    - Remove PendingApprovalsResponse from transaction schemas (keep in dashboard only)
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2_

  - [x] 2.2 Regenerate OpenAPI from backend
    - Run `cargo build` to regenerate openapi.yaml
    - _Requirements: 6.3_

  - [x] 2.3 Run split-openapi.py to regenerate split files
    - Execute `python split-openapi.py` in contracts directory
    - Verify schemas are in correct files
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2_

  - [x] 2.4 Verify OpenAPI reference integrity
    - Check all $ref references are valid across split files
    - **Property 3: OpenAPI Reference Integrity**
    - **Validates: Requirements 3.3**

- [x] 3. Fix Frontend Types
  - [x] 3.1 Update api-helpers.ts with missing exports
    - Add PayInvoiceRequest export
    - Add PendingTransactionResponse export
    - Add PaginatedTransactionsResponse export
    - Add PaginationMeta export
    - _Requirements: 4.2, 4.3_

  - [x] 3.2 Update transactions.ts to use generated types
    - Remove manual GetPendingTransactionsResponse interface
    - Use generated PendingTransactionResponse type
    - _Requirements: 4.1_

  - [x] 3.3 Regenerate frontend types from OpenAPI
    - Run `pnpm openapi-typescript ../contracts/openapi.yaml -o src/types/api.generated.ts`
    - _Requirements: 6.1_

  - [x] 3.4 Verify frontend build passes
    - Run `pnpm build` in frontend directory
    - Fix any type errors that arise
    - _Requirements: 6.2_

- [x] 4. Checkpoint - Backend and Types Sync Complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. E2E Testing - Transaction Flows ✅ ALL COMPLETED
  - [x] 5.1 E2E: Login and navigate to transactions
    - Use MCP Playwright to login with corp@zeltra.io
    - Navigate to transactions page
    - Verify page loads correctly
    - _Requirements: 7.1_

  - [x] 5.2 E2E: Test transaction list with pagination
    - Verify transaction list displays
    - Test pagination controls
    - Verify response matches PaginatedTransactionsResponse schema
    - **Property 5: Pagination Response Structure**
    - _Requirements: 5.2, 7.2_

  - [x] 5.3 E2E: Test create transaction flow ✅ COMPLETED
    - Click create transaction button
    - Fill form with valid data
    - Verify balance validation works (Property 6)
    - Submit and verify response matches TransactionResponse
    - **FIXED: Session race condition - refresh_token was being set to undefined**
    - **FIXED: crypto.randomUUID fallback for Playwright environment**
    - **FIXED: Mock dimension data replaced with real API calls in dimensions.ts**
    - **VERIFIED: UI form submission works (201 Created)**
    - _Requirements: 5.1, 7.1, 8.1, 8.2, 8.3, 8.4_

  - [x] 5.4 E2E: Test void transaction flow ✅ COMPLETED
    - Find a posted transaction
    - Click void button
    - Enter void reason
    - Verify response contains full VoidResponse with both transactions
    - **FIXED: Empty JSON body for workflow mutations (submit, approve, post)**
    - **VERIFIED: Full workflow tested: draft → pending → approved → posted → voided**
    - **Property 1: VoidResponse Schema Compliance**
    - _Requirements: 5.3, 7.3_

  - [x] 5.5 E2E: Test pending approvals view ✅ COMPLETED
    - Navigate to pending approvals
    - Verify can_approve status displays correctly
    - Verify response matches PendingTransactionResponse[] structure
    - **VERIFIED: Pending transactions display with checkboxes and action buttons**
    - _Requirements: 5.4, 7.4_

  - [x] 5.6 E2E: Test bulk approve flow ✅ COMPLETED
    - Select multiple pending transactions
    - Click bulk approve
    - Verify individual success/failure status for each
    - Verify response matches BulkApproveResponse schema
    - **VERIFIED: Bulk approve works - "Processed 2 transactions"**
    - _Requirements: 5.5, 7.5_

- [x] 6. UI/UX Verification ✅ ALL COMPLETED
  - [x] 6.1 Verify loading states
    - Check skeleton loaders appear during data fetch
    - Check loading indicators on form submission
    - **VERIFIED: Loading spinner shown during page load**
    - _Requirements: 7.7_

  - [x] 6.2 Verify error handling UI
    - Test with invalid data to trigger errors
    - Verify user-friendly error messages display
    - **VERIFIED: Toast notification shows "API Error: 422 Unprocessable Entity"**
    - _Requirements: 7.6_

  - [x] 6.3 Verify data refresh behavior
    - After creating/voiding transaction, verify list refreshes
    - No full page reload should occur
    - **VERIFIED: List updates automatically after operations via React Query invalidation**
    - _Requirements: 7.8_

- [x] 7. Final Checkpoint ✅ ALL VERIFIED
  - Ensure all tests pass, ask the user if questions arise.
  - [x] Verify backend tests: `cargo test`
  - [x] Verify frontend build: `pnpm build`
  - [x] All E2E flows working correctly

## Notes

- Tasks marked with `*` are optional property-based tests
- E2E tests use MCP Playwright for manual execution
- Login credentials: corp@zeltra.io / qwertyui
- Backend must be running for E2E tests
- Frontend dev server must be running for E2E tests
