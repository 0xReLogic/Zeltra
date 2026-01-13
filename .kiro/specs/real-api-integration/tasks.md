# Implementation Plan: Real API Integration

## Overview

This plan converts the Zeltra frontend from mock API to real backend integration. Tasks are ordered to ensure incremental progress with early validation.

## Tasks

- [x] 1. Remove Mock API Dependencies
  - [x] 1.1 Remove MOCK_DATA constant and fallback logic from `frontend/src/lib/api/client.ts`
    - Delete the entire MOCK_DATA object
    - Remove all mock fallback logic in the catch block
    - Remove NEXT_PUBLIC_API_MOCK environment variable checks
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - [x] 1.2 Disable MSW initialization in `frontend/src/mocks/browser.ts`
    - Comment out or remove MSW worker.start() call
    - Update MSWProvider to not initialize MSW
    - _Requirements: 1.1_
  - [x] 1.3 Write property test for no mock fallback
    - **Property 1: No Mock Fallback**
    - **Validates: Requirements 1.2, 1.5**
  - [x] 1.4 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 2. Fix Role Type Mismatch
  - [x] 2.1 Update OrganizationUser role type in `frontend/src/types/organizations.ts`
    - Add 'submitter' to UserRole type
    - Ensure exactly 6 roles: owner, admin, approver, accountant, viewer, submitter
    - _Requirements: 2.1_
  - [x] 2.2 Update role selection UI components
    - Update role dropdown options in user invite form
    - Update role dropdown options in user role update form
    - Add submitter role to all role-related UI
    - _Requirements: 2.2, 2.3, 2.4_
  - [x] 2.3 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 3. Optimize API Client
  - [x] 3.1 Update API client configuration in `frontend/src/lib/api/client.ts`
    - Change timeout from 3 seconds to 30 seconds
    - Ensure baseUrl always points to real backend
    - _Requirements: 5.6_
  - [x] 3.2 Implement proper header handling
    - Ensure Authorization header is always set for authenticated requests
    - Ensure X-Organization-ID header is always set for org-scoped requests
    - _Requirements: 5.1, 5.2_
  - [x] 3.3 Implement 401 token refresh logic
    - Detect 401 response
    - Attempt token refresh using refresh_token
    - Retry original request with new token
    - Redirect to login if refresh fails
    - _Requirements: 5.3_
  - [x] 3.4 Implement error handling improvements
    - Handle 403 with permission denied error
    - Handle network errors with user-friendly messages
    - _Requirements: 5.4, 5.5_
  - [x] 3.5 Write property tests for API client headers
    - **Property 2: Authentication Headers**
    - **Property 3: Organization Context Headers**
    - **Validates: Requirements 5.1, 5.2**
  - [x] 3.6 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 4. Checkpoint - Verify API Client
  - Ensure all tests pass, ask the user if questions arise.
  - Test API client with real backend manually

- [x] 5. Authentication Integration
  - [x] 5.1 Update login mutation in `frontend/src/lib/queries/auth.ts`
    - Ensure POST to /api/v1/auth/login
    - Store access_token, refresh_token, and expires_in
    - Update auth store with token expiration tracking
    - _Requirements: 4.1, 4.2_
  - [x] 5.2 Update register mutation
    - Ensure POST to /api/v1/auth/register
    - Handle registration response
    - _Requirements: 4.3_
  - [x] 5.3 Update logout mutation
    - Ensure POST to /api/v1/auth/logout
    - Clear all stored tokens and state
    - _Requirements: 4.5_
  - [x] 5.4 Implement token refresh in auth store
    - Add tokenExpiresAt tracking
    - Add isTokenExpired() method
    - Add refreshAccessToken() method
    - _Requirements: 4.4_
  - [x] 5.5 Update error handling for auth failures
    - Display backend error messages on login failure
    - Display backend error messages on register failure
    - _Requirements: 4.6_
  - [x] 5.6 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 6. Organization Creation UI
  - [x] 6.1 Create CreateOrganizationRequest type
    - Add to `frontend/src/types/organizations.ts`
    - Include name, slug, base_currency, timezone fields
    - _Requirements: 3.2_
  - [x] 6.2 Create useCreateOrganization mutation
    - Add to `frontend/src/lib/queries/organizations.ts`
    - POST to /api/v1/organizations
    - Handle success and error responses
    - _Requirements: 3.3, 3.5_
  - [x] 6.3 Create CreateOrganizationDialog component
    - Create form with name, slug, base_currency, timezone fields
    - Add slug validation (lowercase, numbers, hyphens only)
    - Add currency selector with common currencies
    - Add timezone selector
    - _Requirements: 3.2, 3.6_
  - [x] 6.4 Add "Create Organization" button to organization settings
    - Add button to organization settings page
    - Wire up dialog open/close
    - Handle success callback to switch to new org
    - _Requirements: 3.1, 3.4_
  - [x] 6.5 Write property test for slug validation
    - **Property 4: Slug Validation**
    - **Validates: Requirements 3.6**
  - [x] 6.6 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 7. Checkpoint - Verify Core Integration
  - Ensure all tests pass, ask the user if questions arise.
  - Test login/logout flow with real backend
  - Test organization creation with real backend

- [x] 8. Update E2E Tests for Real API
  - [x] 8.1 Update auth.spec.ts for real API
    - Remove mock dependencies
    - Test with real backend credentials
    - Verify token storage
    - _Requirements: 4.1, 4.2, 4.5_
  - [x] 8.2 Update smoke.spec.ts for real API
    - Verify pages load with real data
    - Check for console errors
    - _Requirements: 1.2_
  - [x] 8.3 Add organization creation E2E test
    - Test create organization flow
    - Verify new org appears in list
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - [x] 8.4 Add role management E2E test
    - Test inviting user with submitter role
    - Test changing user role to submitter
    - _Requirements: 2.2, 2.3, 2.4_
  - [x] 8.5 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 9. Final Checkpoint
  - [x] Ensure all tests pass, ask the user if questions arise.
  - [x] Verify all mock dependencies removed
  - [x] Verify all 6 roles working
  - [x] Verify organization creation working
  - [x] Update PROGRESS.md and ROADMAP.md

- [x] 10. Self-Test with MCP Playwright
  - [x] 10.1 Test Auth Flow with Real Backend ✅
    - Login successful with real backend
    - Token stored correctly
    - Redirect to dashboard works
    - **ISSUE FOUND:** New users can't login without org (403 no_organization)
    - **WORKAROUND:** Manual DB insert to add user to org
    - _Requirements: 4.1, 4.2_
  - [x] 10.2 Test Organization Creation Flow ✅
    - Create Organization dialog works
    - Form validation works (name, slug, currency, timezone)
    - Org created and verified in DB: "Kiro Test Corp"
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - [x] 10.3 Test Role Management Flow ✅
    - Invite User dialog shows 5 roles: Admin, Accountant, Approver, Submitter, Viewer
    - Owner role excluded (correct - auto-assigned to creator)
    - Submitter role present ✅
    - _Requirements: 2.2, 2.3, 2.4_
  - [x] 10.4 Test Dashboard Loads with Real Data ⚠️ PARTIAL
    - Dashboard page loads
    - **ISSUE:** 404 errors on `/api/v1/dashboard/metrics` and `/api/v1/dashboard/cash-flow`
    - **ROOT CAUSE:** Frontend calls non-org-scoped endpoints, backend uses org-scoped
    - **FIX NEEDED:** Update frontend to use `/organizations/{org_id}/dashboard/*`
    - _Requirements: 1.2_
  - [x] 10.5 Test Transaction List Loads ⚠️ PARTIAL
    - Transactions page loads
    - **ISSUE:** 404 on `/api/v1/transactions`
    - **ROOT CAUSE:** Frontend calls `/transactions`, backend expects `/organizations/{org_id}/transactions`
    - **FIX NEEDED:** Update all API calls to use org-scoped endpoints
    - _Requirements: 1.2_
  - [x] 10.6 Check problems and run lint ✅
    - Lint passed (0 errors, 6 warnings)
    - Build successful

## Issues Found During Self-Test

### Critical Issues (Need Fix)

1. **Auth Flow - No Org Login Block** ✅ FIXED
   - New registered users can now login even without org
   - Backend returns `organizations: []` for users without org
   - Frontend redirects to `/onboarding/create-organization` when `organizations.length === 0`
   - User creates org → becomes owner → redirected to dashboard

2. **API Path Mismatch - Non-Org-Scoped Endpoints**
   - Frontend calls: `/api/v1/transactions`, `/api/v1/dashboard/metrics`
   - Backend expects: `/organizations/{org_id}/transactions`, `/organizations/{org_id}/dashboard/metrics`
   - Affected: Transactions, Dashboard, Budgets, Dimensions, Reports
   - Need: Update all frontend API calls to use org-scoped paths

### Fixed Issues

1. **Email Verification** ✅ FIXED
   - Added `smtp_tls` config option to backend
   - Set `smtp_tls = false` for development (MailHog doesn't support TLS)
   - MailHog running on Docker: `docker run -d --name mailhog -p 1025:1025 -p 8025:8025 mailhog/mailhog`
   - Verification emails now sent successfully

2. **Register Response Mismatch** ✅ FIXED
   - Frontend expected `AuthResponse` with tokens
   - Backend returns `RegisterResponse` without tokens (email not verified yet)
   - Added `RegisterResponse` type and updated `useRegister` mutation

3. **No Org Login Block** ✅ FIXED
   - Backend now allows login without org (returns nil UUID for org_id in token)
   - Frontend checks `organizations.length` and redirects to onboarding
   - Created `/onboarding/create-organization` page

### Minor Issues

1. **Team Management** - Shows "No users found" even when user exists (refresh issue)

- [x] 11. Fix API Path Mismatch - Org-Scoped Endpoints
  - [x] 11.1 Update API client to support org-scoped paths
    - Modify `apiClient` function to auto-prefix org-scoped endpoints with `/organizations/{org_id}`
    - Add helper function `orgScopedEndpoint(path)` that prepends org context
    - Keep auth endpoints (`/auth/*`) without org prefix
    - _Requirements: 5.2_
  - [x] 11.2 Update dashboard queries
    - Change `/dashboard/metrics` → `/organizations/{org_id}/dashboard/metrics`
    - Change `/dashboard/cash-flow` → `/organizations/{org_id}/dashboard/cash-flow`
    - Change `/dashboard/recent-activity` → `/organizations/{org_id}/dashboard/recent-activity`
    - Update `frontend/src/lib/queries/dashboard.ts`
    - _Requirements: 1.2_
  - [x] 11.3 Update transactions queries
    - Change `/transactions` → `/organizations/{org_id}/transactions`
    - Change `/transactions/{id}` → `/organizations/{org_id}/transactions/{id}`
    - Change `/transactions/{id}/approve` → `/organizations/{org_id}/transactions/{id}/approve`
    - Change `/transactions/{id}/reject` → `/organizations/{org_id}/transactions/{id}/reject`
    - Update `frontend/src/lib/queries/transactions.ts`
    - _Requirements: 1.2_
  - [x] 11.4 Update accounts queries
    - Change `/accounts` → `/organizations/{org_id}/accounts`
    - Change `/accounts/{id}` → `/organizations/{org_id}/accounts/{id}`
    - Change `/accounts/{id}/ledger` → `/organizations/{org_id}/accounts/{id}/ledger`
    - Change `/accounts/{id}/status` → `/organizations/{org_id}/accounts/{id}/status`
    - Update `frontend/src/lib/queries/accounts.ts`
    - _Requirements: 1.2_
  - [x] 11.5 Update budgets queries
    - Change `/budgets` → `/organizations/{org_id}/budgets`
    - Change `/budgets/{id}` → `/organizations/{org_id}/budgets/{id}`
    - Change `/budgets/{id}/lines` → `/organizations/{org_id}/budgets/{id}/lines`
    - Change `/budgets/{id}/status` → `/organizations/{org_id}/budgets/{id}/status`
    - Update `frontend/src/lib/queries/budgets.ts`
    - _Requirements: 1.2_
  - [x] 11.6 Update dimensions queries
    - Change `/dimensions` → `/organizations/{org_id}/dimension-types` (with values)
    - Change `/dimension-types` → `/organizations/{org_id}/dimension-types`
    - Change `/dimensions/{typeId}/values` → `/organizations/{org_id}/dimension-values`
    - Update `frontend/src/lib/queries/dimensions.ts`
    - _Requirements: 1.2_
  - [x] 11.7 Update reports queries
    - Change `/reports/trial-balance` → `/organizations/{org_id}/reports/trial-balance`
    - Change `/reports/income-statement` → `/organizations/{org_id}/reports/income-statement`
    - Change `/reports/balance-sheet` → `/organizations/{org_id}/reports/balance-sheet`
    - Change `/reports/dimensional` → `/organizations/{org_id}/reports/dimensional`
    - Update `frontend/src/lib/queries/reports.ts`
    - _Requirements: 1.2_
  - [x] 11.8 Check problems and run lint
    - Use getDiagnostics tool to check for TypeScript/lint problems in modified files FIRST
    - Fix any type errors or problems found
    - Then run `npm run lint` in frontend directory
    - Ensure zero problems before proceeding

- [x] 12. Self-Test Org-Scoped Endpoints with MCP Playwright
  - [x] 12.1 Start backend and frontend servers
    - Start PostgreSQL via Docker if not running
    - Start backend: `cargo run` in backend directory
    - Start frontend: `pnpm run dev` in frontend directory
    - Verify both servers are healthy
  - [x] 12.2 Test Dashboard with Real Data ✅
    - Navigate to dashboard after login
    - Verify metrics load without 404
    - Verify cash flow chart loads
    - Verify recent activity loads
    - _Requirements: 1.2_
  - [x] 12.3 Test Transactions Page ✅
    - Navigate to transactions page
    - Verify transaction list loads (shows "No transactions found")
    - Fixed response type mismatch (backend returns array, not paginated object)
    - _Requirements: 1.2_
  - [x] 12.4 Test Accounts Page ✅
    - Navigate to accounts page
    - Verify accounts list loads (empty, shows "New Account" button)
    - Fixed response type mismatch (backend returns array, not `{ data: [] }`)
    - _Requirements: 1.2_
  - [x] 12.5 Test Budgets Page ✅
    - Navigate to budgets page
    - Verify budgets list loads (shows $0 totals)
    - Fixed response type mismatch (backend returns array, not `{ data: [] }`)
    - _Requirements: 1.2_
  - [x] 12.6 Test Dimensions Page ✅
    - Navigate to dimensions page
    - Verify dimension types load (shows "No dimensions defined yet")
    - Added safe array check with useMemo
    - _Requirements: 1.2_
  - [x] 12.7 Test Reports Page ✅
    - Navigate to reports page
    - Verify trial balance loads (shows empty table with $0.00 totals)
    - _Requirements: 1.2_
  - [x] 12.8 Check problems and run lint ✅
    - Lint passed (0 errors, 7 warnings - all unused imports)
    - All pages load without crashes

- [x] 13. Final Integration Checkpoint
  - [x] Ensure all API calls use org-scoped paths
  - [x] Ensure no 404 errors on any page
  - [x] Verified via Playwright E2E testing (manual)
  - [x] Update ROADMAP.md with Real API status (2026-01-13)

## Response Type Fixes Applied

During testing, discovered that backend returns arrays directly for list endpoints, not wrapped in `{ data: [] }`:

1. **Transactions**: `GetTransactionsResponse` changed from `{ data: Transaction[], pagination: {...} }` to `TransactionListItem[]`
2. **Accounts**: `GetAccountsResponse` changed from `{ data: Account[] }` to `Account[]`
3. **Budgets**: `GetBudgetsResponse` changed from `{ data: Budget[] }` to `Budget[]`
4. **CashFlow**: Added `CashFlowResponse` wrapper type and extract `data` array in query

Files modified:
- `frontend/src/types/transactions.ts`
- `frontend/src/types/accounts.ts`
- `frontend/src/lib/queries/transactions.ts`
- `frontend/src/lib/queries/budgets.ts`
- `frontend/src/lib/queries/dashboard.ts`
- `frontend/src/app/dashboard/transactions/page.tsx`
- `frontend/src/app/dashboard/accounts/page.tsx`
- `frontend/src/app/dashboard/budgets/page.tsx`
- `frontend/src/app/dashboard/page.tsx`
- `frontend/src/app/dashboard/master-data/dimensions/page.tsx`
- `frontend/src/components/transactions/CreateTransactionDialog.tsx`

## Notes

- All tasks including property tests are required
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Every major task ends with lint check to ensure clean code
- Use Exa/Tavily search for best practices if knowledge is insufficient
