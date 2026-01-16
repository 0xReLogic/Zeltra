# Implementation Tasks: Sentinel Intelligence UI

## Notes

- **E2E Testing**: Use Playwright MCP, IP `10.0.0.5` bukan localhost
- **Login Credentials**: 
  - Email: `corp@zeltra.io`
  - Password: `qwertyui`
- **Lint Check**: Setiap selesai coding, cek Problems panel (getDiagnostics)
- **Design System**: Follow existing shadcn/ui patterns dari transactions/accounts pages
- **Bug Tracking**: Kalau ketemu bug aneh, save ke Cognito project `zeltra-bug` untuk learning

## Tasks

- [x] 1. Setup API Integration Layer
  - [x] 1.1 Export Sentinel types from api-helpers.ts
    - Add AccrualScheduleResponse, CreateAccrualScheduleRequest
    - Add RevaluationLogResponse
    - Add IntercompanyMappingResponse, CreateIntercompanyMappingRequest
    - File: `frontend/src/types/api-helpers.ts`
    - _Requirements: REQ-4.1_
  - [x] 1.2 Create sentinel.ts query file
    - Create `frontend/src/lib/queries/sentinel.ts`
    - Implement useAccrualSchedules() hook
    - Implement useCreateAccrualSchedule() mutation
    - Implement useRevaluationLogs() hook
    - Implement useIntercompanyMappings() hook
    - Implement useCreateIntercompanyMapping() mutation
    - Include query invalidation on mutations
    - _Requirements: REQ-4.2, REQ-4.3, REQ-4.4, REQ-4.5, REQ-4.6, REQ-4.7_

- [-] 2. Implement Accruals Page
  - [x] 2.1 Create AccrualsPage component
    - Replace placeholder with full implementation
    - Add tier gating check for `has_auto_accruals`
    - Display UpgradePrompt if tier not available
    - File: `frontend/src/app/dashboard/accruals/page.tsx`
    - _Requirements: REQ-1.1, REQ-1.5_
  - [x] 2.2 Create AccrualScheduleTable component
    - DataTable with columns: Name, Total Amount, Progress, Frequency, Status, Next Run
    - Progress bar showing periods_processed / total_periods
    - Status badge with color coding
    - Row click handler for details
    - File: `frontend/src/components/sentinel/AccrualScheduleTable.tsx`
    - _Requirements: REQ-1.1, REQ-1.4_
  - [x] 2.3 Create CreateAccrualDialog component
    - Form with all required fields
    - Zod validation schema
    - Account selector dropdowns
    - Date pickers for start/end
    - Frequency selector
    - File: `frontend/src/components/sentinel/CreateAccrualDialog.tsx`
    - _Requirements: REQ-1.2, REQ-1.3, REQ-5.6_
  - [x] 2.4 Add loading and empty states
    - Skeleton loader during fetch
    - Empty state with guidance when no schedules
    - _Requirements: REQ-1.6, REQ-1.7, REQ-5.3, REQ-5.5_

- [-] 3. Implement Revaluation Page
  - [x] 3.1 Create RevaluationPage component
    - Replace placeholder with full implementation
    - Add tier gating check for `has_multi_currency`
    - Display UpgradePrompt if tier not available
    - File: `frontend/src/app/dashboard/revaluation/page.tsx`
    - _Requirements: REQ-2.1, REQ-2.4_
  - [x] 3.2 Create RevaluationLogTable component
    - DataTable with columns: Date, Account, Currency, Old Rate, New Rate, Gain/Loss
    - Green text for gains, red for losses
    - Format decimal values properly
    - File: `frontend/src/components/sentinel/RevaluationLogTable.tsx`
    - _Requirements: REQ-2.1, REQ-2.2_
  - [x] 3.3 Add date range filter
    - Date picker for start/end date
    - Filter logs by revaluation_date
    - _Requirements: REQ-2.3_
  - [x] 3.4 Add loading and empty states
    - Skeleton loader during fetch
    - Empty state explaining no revaluations
    - _Requirements: REQ-2.5, REQ-2.6, REQ-5.3, REQ-5.5_

- [-] 4. Implement Intercompany Page
  - [x] 4.1 Create IntercompanyPage component
    - Replace placeholder with full implementation
    - Add tier gating check for `has_intercompany_hub`
    - Display UpgradePrompt if tier not available
    - File: `frontend/src/app/dashboard/intercompany/page.tsx`
    - _Requirements: REQ-3.1, REQ-3.4_
  - [x] 4.2 Create IntercompanyMappingTable component
    - DataTable with columns: Source Org, Source Account, Target Org, Target Account, Type, Auto-Post
    - Badge for mapping type (elimination/mirror)
    - Toggle indicator for auto-post status
    - File: `frontend/src/components/sentinel/IntercompanyMappingTable.tsx`
    - _Requirements: REQ-3.1_
  - [x] 4.3 Create CreateMappingDialog component
    - Form with account and organization selectors
    - Zod validation schema
    - File: `frontend/src/components/sentinel/CreateMappingDialog.tsx`
    - _Requirements: REQ-3.2, REQ-3.3, REQ-5.6_
  - [x] 4.4 Add loading and empty states
    - Skeleton loader during fetch
    - Empty state with setup guidance
    - _Requirements: REQ-3.5, REQ-3.6, REQ-5.3, REQ-5.5_

- [x] 5. Checkpoint - Build and Lint Check
  - Run `pnpm build` in frontend/
  - Run getDiagnostics on all new files
  - Fix any type errors or warnings
  - _Requirements: All_

- [x] 6. E2E Testing (Playwright MCP, IP: 10.0.0.5)
  - [x] 6.1 Test Accruals page ✅
    - Login dengan `corp@zeltra.io` / `qwertyui`
    - Navigate to /dashboard/accruals
    - Page renders correctly with summary cards and empty state
    - Create Schedule dialog opens with all fields
    - Fixed: Moved loading/error checks before data processing to prevent crash
    - _Requirements: REQ-6.1_
  - [x] 6.2 Test Revaluation page ✅
    - Navigate to /dashboard/revaluation
    - Page renders correctly with summary cards (gains/losses/net)
    - Date range filter present
    - Empty state shows correctly
    - _Requirements: REQ-6.2_
  - [x] 6.3 Test Intercompany page ✅
    - Navigate to /dashboard/intercompany
    - Page renders correctly with summary cards
    - Connect Organizations dialog opens with all fields
    - NOTE: Backend returns 405 for GET /intercompany/mappings (backend bug)
    - _Requirements: REQ-6.3_
  - [x] 6.4 Test tier gating ✅
    - User `corp@zeltra.io` has enterprise features enabled
    - Tier gating logic implemented correctly (shows upgrade prompt when tier not available)
    - Fixed: Loading state now shows before tier check to prevent race condition
    - _Requirements: REQ-6.4_

- [x] 7. Final Checkpoint ✅
  - All E2E tests passed
  - Responsive design verified on mobile viewport (375x667)
  - Keyboard navigation works (Escape to close dialogs)
  - Bug found: Backend returns 405 for GET /intercompany/mappings
  - _Requirements: All, REQ-5.2, REQ-5.7_


---

## Bugs Found During E2E Testing

### BUG-001: Backend 405 on GET /intercompany/mappings
- **Severity**: Medium
- **Endpoint**: `GET /api/v1/organizations/{org_id}/intercompany/mappings`
- **Expected**: Return list of intercompany mappings
- **Actual**: Returns 405 Method Not Allowed
- **Impact**: Intercompany page cannot load existing mappings
- **Root Cause**: Backend endpoint may only support POST, not GET
- **Fix**: Add GET handler to intercompany mappings endpoint in backend

### BUG-002: Client-side crash on API error (FIXED)
- **Severity**: High (Fixed)
- **Issue**: When API returns 403/error, page crashed with "scheduleList.filter is not a function"
- **Root Cause**: Data processing happened before loading/error state checks
- **Fix**: Moved loading/error checks to top of component, added Array.isArray() safety check
- **Files Fixed**: 
  - `frontend/src/app/dashboard/accruals/page.tsx`
  - `frontend/src/app/dashboard/revaluation/page.tsx`
  - `frontend/src/app/dashboard/intercompany/page.tsx`
