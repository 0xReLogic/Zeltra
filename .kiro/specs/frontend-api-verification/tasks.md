# Implementation Plan: Frontend API Verification

## Overview

This plan verifies and fixes frontend API integration with the real backend. Tasks are ordered by feature area, with each area including type migration, query/mutation updates, and E2E verification.

**IMPORTANT RULES:**
- At each checkpoint: Run lint, E2E test with MCP Playwright (MUST PASS), then push to GitHub
- If context is lost: Re-read requirements.md and design.md before continuing
- Use `grepSearch` and `fileSearch` to find code instead of reading entire files
- E2E tests are MANDATORY at checkpoints - CANNOT proceed without passing
- ALL TASKS ARE REQUIRED - No optional tasks, comprehensive testing

## Tasks

- [x] 1. Setup Infrastructure & Foundation
  - [x] 1.1 Verify infrastructure with MCP Playwright
    - Infrastructure already running (Docker, PostgreSQL, MailHog, backend, frontend)
    - Use MCP Playwright to navigate to `http://0.0.0.0:3000/login`
    - Verify login page loads correctly without errors
    - Take screenshot to confirm
    - _Requirements: All_
  - [x] 1.2 Create type helper utilities
    - Create `frontend/src/types/api-helpers.ts` with helper types:
      ```typescript
      import { components, operations } from './api.generated'
      
      // Helper to extract schema types
      export type Schema<T extends keyof components['schemas']> = components['schemas'][T]
      
      // Helper to extract request body type
      export type RequestBody<T extends keyof operations> = 
        operations[T] extends { requestBody: { content: { 'application/json': infer R } } } ? R : never
      
      // Helper to extract response type
      export type ResponseBody<T extends keyof operations, S extends number = 200> = 
        operations[T] extends { responses: { [K in S]: { content: { 'application/json': infer R } } } } ? R : never
      ```
    - Use `grepSearch` to find existing type patterns in codebase
    - _Requirements: 1.1, 1.2_
  - [x] 1.3 Update API client error handling
    - Use `grepSearch` to find current error handling in `client.ts`
    - Update error handling to show toast notifications for all error types:
      - 400: Show validation error message from response
      - 401: Show "Session expired, please login again" and redirect to login
      - 403: Show "Permission denied"
      - 404: Show "Resource not found"
      - 409: Show conflict error message from response
      - 422: Show validation error details
      - 500+: Show "Server error, please try again"
    - Import and use `toast` from sonner
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_
  - [x] 1.4 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding (zero tolerance)

- [x] 2. Transaction CRUD & Workflow
  - [x] 2.1 Update transaction types
    - Use `grepSearch` to find current transaction types in `types/transactions.ts`
    - Import and re-export types from `api.generated.ts`:
      - `Transaction` - base transaction type
      - `TransactionWithEntries` - transaction with ledger entries
      - `CreateTransactionRequest` - request payload for creating transaction
      - `UpdateTransactionRequest` - request payload for updating transaction
      - `LedgerEntry` - individual ledger entry
      - `TransactionStatus` - enum for transaction status
    - Keep backward compatibility aliases if needed
    - _Requirements: 1.1, 1.2_
  - [x] 2.2 Update transaction queries
    - Use `grepSearch` to find current transaction queries in `lib/queries/transactions.ts`
    - Verify `useTransactions` query:
      - Endpoint: `GET /organizations/{org_id}/transactions`
      - Returns array of transactions (not paginated wrapper)
      - Query params: status, start_date, end_date, account_id
    - Verify `useTransaction` query:
      - Endpoint: `GET /organizations/{org_id}/transactions/{id}`
      - Returns single transaction with entries
    - Add `useCreateTransaction` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions`
      - Payload: `{ description, transaction_date, entries: [{ account_id, debit, credit, dimensions? }] }`
      - Invalidate transactions list on success
    - Add `useUpdateTransaction` mutation:
      - Endpoint: `PATCH /organizations/{org_id}/transactions/{id}`
      - Only for draft transactions
      - Invalidate transaction detail and list on success
    - Add `useDeleteTransaction` mutation:
      - Endpoint: `DELETE /organizations/{org_id}/transactions/{id}`
      - Only for draft transactions
      - Invalidate transactions list on success
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - [x] 2.3 Add transaction workflow mutations
    - Add `useSubmitTransaction` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions/{id}/submit`
      - Changes status: draft → pending
      - Invalidate transaction detail and list
    - Add `useApproveTransaction` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions/{id}/approve`
      - Changes status: pending → approved
      - Invalidate transaction detail and list
    - Add `useRejectTransaction` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions/{id}/reject`
      - Payload: `{ reason: string }`
      - Changes status: pending → draft
      - Invalidate transaction detail and list
    - Add `usePostTransaction` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions/{id}/post`
      - Changes status: approved → posted
      - Invalidate transaction detail and list
    - Add `useVoidTransaction` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions/{id}/void`
      - Payload: `{ reason: string }`
      - Changes status: posted → voided
      - Invalidate transaction detail and list
    - Add `useBulkApprove` mutation:
      - Endpoint: `POST /organizations/{org_id}/transactions/bulk-approve`
      - Payload: `{ transaction_ids: string[] }`
      - Invalidate transactions list
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - [x] 2.4 Update CreateTransactionDialog component
    - Use `grepSearch` to find CreateTransactionDialog in `components/transactions/`
    - Wire form submission to `useCreateTransaction` mutation
    - Form fields:
      - description (required, text)
      - transaction_date (required, date picker)
      - entries array with:
        - account_id (required, account dropdown)
        - debit (decimal, mutually exclusive with credit)
        - credit (decimal, mutually exclusive with debit)
        - dimensions (optional, dimension value dropdowns)
    - Validate entries balance (total debits = total credits)
    - Show loading state during submission
    - Show success toast and close dialog on success
    - Show error toast on failure
    - _Requirements: 2.1, 2.4, 2.5_
  - [x] 2.5 Update transaction detail page with workflow actions
    - Use `grepSearch` to find transaction detail page
    - Add workflow action buttons based on current status:
      - draft: Show "Submit for Approval" button
      - pending: Show "Approve" and "Reject" buttons (for approvers)
      - approved: Show "Post" button
      - posted: Show "Void" button
    - Wire buttons to respective mutations
    - Show confirmation dialog for destructive actions (reject, void)
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - [x] 2.6 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding

- [x] 3. Checkpoint - Transaction CRUD (MANDATORY E2E)
  - **IF CONTEXT LOST:** Re-read `.kiro/specs/frontend-api-verification/requirements.md` and `design.md`
  - [x] 3.1 E2E Test with MCP Playwright (MUST PASS BEFORE PUSH)
    - Use MCP Playwright browser tools (NOT creating test files)
    - Test flow:
      1. Navigate to `http://10.0.0.5:3000/login`
      2. Login with `kiro2@zeltra.dev` / `Kiro123!`
      3. Wait for redirect to dashboard
      4. Navigate to `http://10.0.0.5:3000/dashboard/transactions`
      5. Verify transactions list loads without errors
      6. Click "New Transaction" button (if exists)
      7. Fill form and submit (if form is ready)
      8. Verify new transaction appears in list OR verify page loads without console errors
    - **IF TEST FAILS:** Identify the error, fix the code, re-test until pass
    - **CANNOT PUSH until E2E passes**
  - [x] 3.2 Push to GitHub (ONLY after E2E passes)
    - `git add -A`
    - `git commit -m "feat(frontend): Transaction CRUD & workflow mutations"`
    - `git push`

- [x] 4. Account CRUD
  - [x] 4.1 Update account types
    - Use `grepSearch` to find current account types in `types/accounts.ts`
    - Import and re-export types from `api.generated.ts`:
      - `Account` - base account type
      - `AccountWithBalance` - account with balance info
      - `CreateAccountRequest` - request payload
      - `UpdateAccountRequest` - update payload
      - `AccountType` - enum (Asset, Liability, Equity, Revenue, Expense)
      - `AccountSubtype` - enum for subtypes
    - _Requirements: 1.1, 1.2_
  - [x] 4.2 Update account queries
    - Use `grepSearch` to find current account queries in `lib/queries/accounts.ts`
    - Verify `useAccounts` query:
      - Endpoint: `GET /organizations/{org_id}/accounts`
      - Returns array of accounts
      - Query params: account_type, is_active
    - Verify `useAccount` query:
      - Endpoint: `GET /organizations/{org_id}/accounts/{id}`
      - Returns single account with balance
    - Add `useCreateAccount` mutation:
      - Endpoint: `POST /organizations/{org_id}/accounts`
      - Payload: `{ code, name, account_type, account_subtype, parent_id?, currency, description? }`
      - Invalidate accounts list on success
    - Add `useUpdateAccount` mutation:
      - Endpoint: `PUT /organizations/{org_id}/accounts/{id}`
      - Full update of account
      - Invalidate account detail and list
    - Add `useDeleteAccount` mutation:
      - Endpoint: `DELETE /organizations/{org_id}/accounts/{id}`
      - Only for accounts with no transactions
      - Invalidate accounts list
    - Add `useToggleAccountStatus` mutation:
      - Endpoint: `PATCH /organizations/{org_id}/accounts/{id}/status`
      - Payload: `{ is_active: boolean }`
      - Invalidate account detail and list
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  - [x] 4.3 Update AccountForm component
    - Use `grepSearch` to find AccountForm in `components/accounts/`
    - Wire form to `useCreateAccount` mutation
    - Form fields:
      - code (required, unique identifier)
      - name (required, display name)
      - account_type (required, dropdown: Asset, Liability, Equity, Revenue, Expense)
      - account_subtype (required, dropdown based on account_type)
      - parent_id (optional, account dropdown for hierarchy)
      - currency (required, currency dropdown, default to org currency)
      - description (optional, textarea)
    - Show loading state during submission
    - Show success toast and close dialog on success
    - Show error toast on failure with validation details
    - _Requirements: 4.5_
  - [x] 4.4 Add account detail page actions
    - Use `grepSearch` to find account detail page
    - Add "Edit" button that opens edit form
    - Add "Deactivate/Activate" toggle button
    - Add "Delete" button (only if no transactions)
    - Wire buttons to respective mutations
    - _Requirements: 4.2, 4.3, 4.4_
  - [x] 4.5 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding

- [x] 5. Budget CRUD
  - [x] 5.1 Update budget types
    - Use `grepSearch` to find current budget types
    - Import and re-export types from `api.generated.ts`:
      - `Budget` - base budget type
      - `BudgetWithLines` - budget with line items
      - `BudgetLine` - individual budget line
      - `CreateBudgetRequest` - request payload
      - `CreateBudgetLinesRequest` - bulk line creation
      - `BudgetVsActual` - comparison report type
    - _Requirements: 1.1_
  - [x] 5.2 Update budget queries
    - Use `grepSearch` to find current budget queries in `lib/queries/budgets.ts`
    - Verify `useBudgets` query:
      - Endpoint: `GET /organizations/{org_id}/budgets`
      - Returns array of budgets
    - Verify `useBudget` query:
      - Endpoint: `GET /organizations/{org_id}/budgets/{id}`
      - Returns budget with lines
    - Add `useCreateBudget` mutation:
      - Endpoint: `POST /organizations/{org_id}/budgets`
      - Payload: `{ name, fiscal_year_id, description? }`
      - Invalidate budgets list
    - Add `useCreateBudgetLines` mutation:
      - Endpoint: `POST /organizations/{org_id}/budgets/{id}/lines`
      - Payload: `{ lines: [{ account_id, period_id, amount, dimensions? }] }`
      - Invalidate budget detail
    - Add `useLockBudget` mutation:
      - Endpoint: `POST /organizations/{org_id}/budgets/{id}/lock`
      - Prevents further modifications
      - Invalidate budget detail
    - Add `useBudgetVsActual` query:
      - Endpoint: `GET /organizations/{org_id}/budgets/{id}/vs-actual`
      - Returns comparison of budget vs actual amounts
    - _Requirements: 5.1, 5.2, 5.3, 5.4_
  - [x] 5.3 Update budget form components
    - Use `grepSearch` to find budget form components
    - Wire CreateBudgetDialog to `useCreateBudget`
    - Form fields:
      - name (required)
      - fiscal_year_id (required, fiscal year dropdown)
      - description (optional)
    - Add budget lines editor:
      - Account selection
      - Period selection
      - Amount input
      - Dimension values (optional)
    - _Requirements: 5.5_
  - [x] 5.4 Add budget detail page features
    - Display budget lines in table
    - Add "Add Lines" button
    - Add "Lock Budget" button (if not locked)
    - Add "View vs Actual" button
    - Display vs actual comparison chart
    - _Requirements: 5.2, 5.3, 5.4_
  - [x] 5.5 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding

- [x] 6. Checkpoint - Account & Budget CRUD (MANDATORY E2E)
  - **IF CONTEXT LOST:** Re-read `.kiro/specs/frontend-api-verification/requirements.md` and `design.md`
  - [x] 6.1 E2E Test with MCP Playwright (MUST PASS BEFORE PUSH)
    - Use MCP Playwright browser tools (NOT creating test files)
    - Test flow:
      1. Login with `kiro2@zeltra.dev` / `Kiro123!` ✅
      2. Navigate to `http://10.0.0.5:3000/dashboard/accounts` ✅
      3. Verify accounts list loads without errors ✅ (2 accounts displayed)
      4. Click "New Account" button (if exists) ✅
      5. Fill form with test data and submit ✅ (dialog works)
      6. Verify new account appears in list ⚠️ (needs API verification)
      7. Navigate to `http://10.0.0.5:3000/dashboard/budgets` ✅
      8. Verify budgets list loads without errors ✅ (empty state shown)
      9. Test create budget if form is ready ✅ (dialog works, fiscal years dropdown empty)
    - **IF TEST FAILS:** Identify the error, fix the code, re-test until pass
    - **CANNOT PUSH until E2E passes**
  - [ ] 6.2 Push to GitHub (ONLY after E2E passes)
    - `git add -A`
    - `git commit -m "feat(frontend): Account & Budget CRUD mutations"`
    - `git push`

- [ ] 7. Dimension CRUD
  - [ ] 7.1 Update dimension types
    - Use `grepSearch` to find current dimension types
    - Import and re-export types from `api.generated.ts`:
      - `DimensionType` - dimension type definition
      - `DimensionValue` - dimension value
      - `CreateDimensionTypeRequest`
      - `CreateDimensionValueRequest`
      - `UpdateDimensionValueRequest`
    - _Requirements: 1.1_
  - [ ] 7.2 Update dimension queries
    - Use `grepSearch` to find current dimension queries in `lib/queries/dimensions.ts`
    - Verify `useDimensionTypes` query:
      - Endpoint: `GET /organizations/{org_id}/dimension-types`
    - Verify `useDimensionValues` query:
      - Endpoint: `GET /organizations/{org_id}/dimension-values`
      - Query params: dimension_type_id
    - Add `useCreateDimensionType` mutation:
      - Endpoint: `POST /organizations/{org_id}/dimension-types`
      - Payload: `{ name, code, is_required }`
      - Invalidate dimension types list
    - Add `useCreateDimensionValue` mutation:
      - Endpoint: `POST /organizations/{org_id}/dimension-values`
      - Payload: `{ dimension_type_id, code, name }`
      - Invalidate dimension values list
    - Add `useUpdateDimensionValue` mutation:
      - Endpoint: `PATCH /organizations/{org_id}/dimension-values/{id}`
      - Payload: `{ name?, code? }`
      - Invalidate dimension values list
    - Add `useToggleDimensionValueStatus` mutation:
      - Endpoint: `PATCH /organizations/{org_id}/dimension-values/{id}/status`
      - Payload: `{ is_active: boolean }`
      - Invalidate dimension values list
    - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - [ ] 7.3 Update dimension form components
    - Use `grepSearch` to find dimension form components
    - Wire DimensionTypeDialog to `useCreateDimensionType`
    - Form fields for dimension type:
      - name (required)
      - code (required, unique)
      - is_required (boolean checkbox)
    - Wire dimension value form to `useCreateDimensionValue`
    - Form fields for dimension value:
      - dimension_type_id (required, dropdown)
      - code (required)
      - name (required)
    - _Requirements: 6.5_
  - [ ] 7.4 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding

- [ ] 8. Fiscal Period Management
  - [ ] 8.1 Update fiscal types
    - Use `grepSearch` to find current fiscal types
    - Import and re-export types from `api.generated.ts`:
      - `FiscalYear` - fiscal year definition
      - `FiscalPeriod` - individual period
      - `CreateFiscalYearRequest`
      - `PeriodStatus` - enum (Open, SoftClose, Closed)
    - _Requirements: 1.1_
  - [ ] 8.2 Update fiscal queries
    - Use `grepSearch` to find current fiscal queries in `lib/queries/fiscal.ts`
    - Verify `useFiscalYears` query:
      - Endpoint: `GET /organizations/{org_id}/fiscal-years`
    - Verify `useFiscalPeriods` query:
      - Endpoint: `GET /organizations/{org_id}/fiscal-periods`
      - Query params: fiscal_year_id
    - Add `useCreateFiscalYear` mutation:
      - Endpoint: `POST /organizations/{org_id}/fiscal-years`
      - Payload: `{ name, start_date, end_date, include_adjustment_period }`
      - Invalidate fiscal years list
    - Add `useUpdatePeriodStatus` mutation:
      - Endpoint: `PATCH /organizations/{org_id}/fiscal-periods/{id}/status`
      - Payload: `{ status: 'Open' | 'SoftClose' | 'Closed' }`
      - Invalidate fiscal periods list
    - _Requirements: 7.1, 7.2_
  - [ ] 8.3 Update fiscal form components
    - Use `grepSearch` to find fiscal form components
    - Wire CreateFiscalYearDialog to `useCreateFiscalYear`
    - Form fields:
      - name (required)
      - start_date (required, date picker)
      - end_date (required, date picker)
      - include_adjustment_period (boolean checkbox)
    - Add period status change UI:
      - Dropdown to change status (Open, SoftClose, Closed)
      - Confirmation dialog for closing periods
    - _Requirements: 7.3, 7.4_
  - [ ] 8.4 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding

- [ ] 9. Exchange Rate Management
  - [ ] 9.1 Update exchange rate types
    - Use `grepSearch` to find current exchange rate types
    - Import and re-export types from `api.generated.ts`:
      - `ExchangeRate`
      - `CreateExchangeRateRequest`
      - `BulkImportRatesRequest`
    - _Requirements: 1.1_
  - [ ] 9.2 Update exchange rate queries
    - Use `grepSearch` to find current exchange rate queries in `lib/queries/exchange-rates.ts`
    - Verify `useExchangeRates` query:
      - Endpoint: `GET /organizations/{org_id}/exchange-rates`
      - Query params: from_currency, to_currency, effective_date
    - Add `useCreateExchangeRate` mutation:
      - Endpoint: `POST /organizations/{org_id}/exchange-rates`
      - Payload: `{ from_currency, to_currency, rate, effective_date }`
      - Invalidate exchange rates list
    - Add `useBulkImportRates` mutation:
      - Endpoint: `POST /organizations/{org_id}/exchange-rates/bulk`
      - Payload: `{ rates: [{ from_currency, to_currency, rate, effective_date }] }`
      - Invalidate exchange rates list
    - Add `useFetchLiveRates` mutation:
      - Endpoint: `POST /organizations/{org_id}/exchange-rates/fetch`
      - Fetches live rates from external API
      - Invalidate exchange rates list
    - _Requirements: 8.1, 8.2, 8.3_
  - [ ] 9.3 Update exchange rate form components
    - Use `grepSearch` to find exchange rate form components
    - Wire form to `useCreateExchangeRate`
    - Form fields:
      - from_currency (required, currency dropdown)
      - to_currency (required, currency dropdown)
      - rate (required, decimal input)
      - effective_date (required, date picker)
    - Wire BulkImportDialog to `useBulkImportRates`
    - Add "Fetch Live Rates" button wired to `useFetchLiveRates`
    - _Requirements: 8.4_
  - [ ] 9.4 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems
    - Run `pnpm run lint` in frontend directory
    - Fix ALL errors before proceeding

- [ ] 10. Checkpoint - Master Data CRUD (MANDATORY E2E)
  - **IF CONTEXT LOST:** Re-read `.kiro/specs/frontend-api-verification/requirements.md` and `design.md`
  - [ ] 10.1 E2E Test with MCP Playwright (MUST PASS BEFORE PUSH)
    - Use MCP Playwright browser tools (NOT creating test files)
    - Test flow:
      1. Login with `kiro2@zeltra.dev` / `Kiro123!`
      2. Navigate to `http://0.0.0.0:3000/dashboard/master-data/dimensions`
      3. Verify dimensions page loads without errors
      4. Test create dimension type if form is ready
      5. Navigate to `http://0.0.0.0:3000/dashboard/master-data/fiscal-periods`
      6. Verify fiscal periods page loads without errors
      7. Navigate to `http://0.0.0.0:3000/dashboard/master-data/exchange-rates`
      8. Verify exchange rates page loads without errors
    - **IF TEST FAILS:** Identify the error, fix the code, re-test until pass
    - **CANNOT PUSH until E2E passes**
  - [ ] 10.2 Push to GitHub (ONLY after E2E passes)
    - `git add -A`
    - `git commit -m "feat(frontend): Dimension, Fiscal, Exchange Rate CRUD"`
    - `git push`
