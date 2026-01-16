# Implementation Plan: Audit Fix Accounts Ledger

## Overview

This implementation plan fixes synchronization issues between backend API, OpenAPI specification, and frontend types for the Accounts and Ledger domain. Tasks are ordered: backend fixes → OpenAPI regeneration → frontend migration → E2E verification.

## Tasks

- [x] 1. Fix Backend OpenAPI Annotations
  - [x] 1.1 Add Query parameter annotation to ListAccountsQuery
    - Modify `backend/crates/api/src/routes/accounts.rs`
    - Add `#[into_params(parameter_in = Query)]` attribute to ListAccountsQuery struct
    - _Requirements: 1.1, 1.2_

  - [x] 1.2 Add GetAccountsResponse wrapper schema
    - Add new struct `GetAccountsResponse` with `accounts: Vec<AccountResponse>` field
    - Add utoipa::ToSchema derive
    - _Requirements: 1.3, 1.4_

  - [x] 1.3 Update list_accounts endpoint response annotation
    - Change `body = [AccountResponse]` to `body = GetAccountsResponse`
    - _Requirements: 2.2_

  - [x] 1.4 Run backend build to verify changes compile
    - Execute `cargo build` in backend directory
    - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Regenerate OpenAPI and Frontend Types
  - [x] 2.1 Regenerate OpenAPI from backend
    - Run `cargo run --bin generate-openapi`
    - Verify GetAccountsResponse schema exists in openapi.yaml
    - Verify query params are `in: query` and `required: false`
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 2.2 Run split-openapi.py to update split files
    - Execute `python split-openapi.py` in contracts directory
    - Verify 03-accounts-ledger-schemas.yaml has GetAccountsResponse
    - _Requirements: 2.1_

  - [x] 2.3 Regenerate frontend types
    - Run `pnpm openapi-typescript ../contracts/openapi.yaml -o src/types/api.generated.ts`
    - _Requirements: 3.1, 3.2, 3.3_

- [x] 3. Checkpoint - Backend and OpenAPI Sync Complete
  - Verify openapi.yaml has correct query param annotations
  - Verify GetAccountsResponse schema exists
  - Ask user if questions arise

- [x] 4. Update Frontend Types
  - [x] 4.1 Update api-helpers.ts with account type exports
    - Add GetAccountsResponse export if not present
    - Verify AccountResponse, CreateAccountRequest, UpdateAccountRequest exports
    - _Requirements: 3.1, 3.2, 3.3_

  - [x] 4.2 Refactor types/accounts.ts to use generated types
    - Replace manual Account interface with AccountResponse alias
    - Replace manual CreateAccountRequest with generated type
    - Update GetAccountsResponse to match generated wrapper
    - _Requirements: 3.1, 3.2, 3.6_

  - [x] 4.3 Remove duplicate ledger types from accounts.ts
    - Remove LedgerEntry interface (use LedgerEntryResponse)
    - Remove GetLedgerResponse interface (use AccountLedgerResponse)
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5. Update Frontend Queries
  - [x] 5.1 Update useAccounts hook
    - Ensure return type matches GetAccountsResponse wrapper
    - _Requirements: 6.1_

  - [x] 5.2 Update useAccount hook
    - Ensure return type is AccountResponse
    - _Requirements: 6.2_

  - [x] 5.3 Update useAccountLedger hook
    - Change return type to AccountLedgerResponse
    - Remove local GetLedgerResponse usage
    - _Requirements: 6.3_

  - [x] 5.4 Update useCreateAccount hook
    - Ensure CreateAccountRequest includes is_active and allow_direct_posting
    - _Requirements: 4.1, 4.2, 4.3, 6.4_

  - [x] 5.5 Update useUpdateAccount hook
    - Use UpdateAccountRequest from generated types
    - _Requirements: 6.5_

  - [x] 5.6 Verify frontend build passes
    - Run `pnpm build` in frontend directory
    - Fix any type errors
    - _Requirements: 3.1, 3.2, 3.3_

- [x] 6. Checkpoint - Frontend Types Sync Complete
  - Ensure frontend build passes
  - Ask user if questions arise

- [x] 7. E2E Testing - Accounts Flows
  - [x] 7.1 E2E: Login and navigate to accounts
    - Use MCP Playwright to login with corp@zeltra.io
    - Navigate to accounts/chart-of-accounts page
    - Verify page loads correctly
    - _Requirements: 7.1_

  - [x] 7.2 E2E: Test accounts list with filters
    - Verify accounts list displays with wrapper response
    - Test type filter (asset, liability, etc.)
    - Verify response structure matches GetAccountsResponse
    - **Property 1: Response Wrapper Consistency**
    - _Requirements: 7.1_

  - [x] 7.3 E2E: Test create account with optional fields
    - Click create account button
    - Fill form including is_active and allow_direct_posting
    - Submit and verify account created
    - **Property 2: Optional Fields Acceptance**
    - _Requirements: 4.3, 7.2_

  - [x] 7.4 E2E: Test account ledger view
    - Navigate to an account's ledger
    - Verify ledger entries display with all fields
    - Check dimensions, exchange rates, etc. are present
    - _Requirements: 7.3_

  - [x] 7.5 E2E: Test toggle account status
    - Find an account
    - Toggle is_active status
    - Verify status changed
    - **Property 4: Toggle Status Round-trip**
    - _Requirements: 7.4_

- [x] 8. UI/UX Verification
  - [x] 8.1 Verify loading states
    - Check skeleton loaders appear during accounts data fetch
    - Check loading indicators on form submission
    - _Requirements: 7.1_

  - [x] 8.2 Verify error handling UI
    - Test with invalid data to trigger errors
    - Verify user-friendly error messages display
    - Test duplicate account code error
    - _Requirements: 7.2_

  - [x] 8.3 Verify data refresh behavior
    - After creating/updating account, verify list refreshes
    - No full page reload should occur
    - _Requirements: 7.1_

  - [x] 8.4 Verify responsive design
    - Test accounts table on mobile viewport
    - Test create account form on mobile
    - _Requirements: 7.1_

- [x] 9. Final Checkpoint
  - Ensure all tests pass
  - Verify backend tests: `cargo test`
  - Verify frontend build: `pnpm build`
  - All E2E flows working correctly

## Notes

- E2E tests use MCP Playwright for browser automation
- Login credentials: corp@zeltra.io / qwertyui
- Backend must be running for E2E tests
- Frontend dev server must be running for E2E tests
- Use Cognio memory (project: zeltra-bug) to store any unusual bugs found
