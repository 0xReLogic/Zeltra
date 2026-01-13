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

## Notes

- All tasks including property tests are required
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Backend must be running at http://localhost:8080 for testing
- Every major task ends with lint check to ensure clean code
- Use Exa/Tavily search for best practices if knowledge is insufficient
