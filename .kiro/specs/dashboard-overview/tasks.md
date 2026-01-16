# Implementation Tasks

## Task 1: Fix OpenAPI Specification (BUG-004)

### Description
Fix utoipa-generated OpenAPI spec where query parameters are incorrectly marked as path parameters.

### Files to Modify
- `contracts/openapi-split/24-dashboard-endpoints.yaml`

### Steps
1. [x] Change `period_id` in `/dashboard/metrics` from `in: path` to `in: query`, `required: false`
2. [x] Change `months` in `/dashboard/cash-flow` from `in: path` to `in: query`, `required: false`
3. [x] Change `period_id` in `/dashboard/cash-flow` from `in: path` to `in: query`, `required: false`
4. [x] Change `limit` in `/dashboard/recent-activity` from `in: path` to `in: query`, `required: false`
5. [x] Change `type` in `/dashboard/recent-activity` from `in: path` to `in: query`, `required: false`
6. [x] Change `cursor` in `/dashboard/recent-activity` from `in: path` to `in: query`, `required: false`
7. [x] Change `budget_id` in `/dashboard/budget-vs-actual` from `in: path` to `in: query`, `required: false`
8. [x] Fix nullable types: change `type: [string, 'null']` to `type: string` with separate `nullable: true`

---

## Task 2: Fix Frontend Type Alignment (BUG-005)

### Description
Fix CashFlowDataPoint type mismatch - backend returns string (Decimal), frontend expects number.

### Files to Modify
- `frontend/src/lib/queries/dashboard.ts`

### Steps
1. [x] Update `CashFlowDataPoint` interface: change `inflow`, `outflow` from `number` to `string`
2. [x] Add `period_name` and `net` fields to `CashFlowDataPoint` interface
3. [x] Update `useCashFlowData` hook to parse string decimals to numbers for chart rendering
4. [x] Verify chart still renders correctly with parsed data

---

## Task 3: Add Budget vs Actual Hook

### Description
Create React Query hook for the budget-vs-actual endpoint.

### Files to Modify
- `frontend/src/lib/queries/dashboard.ts`

### Steps
1. [x] Add `BudgetVsActualResponse` interface matching backend schema
2. [x] Add `BudgetLineItem` interface for line items
3. [x] Add `BudgetSummary` interface for summary data
4. [x] Create `useBudgetVsActual(budgetId?: string)` hook with React Query

---

## Task 4: Create Budget vs Actual Component

### Description
Create the BudgetVsActual widget component for the dashboard.

### Files to Create
- `frontend/src/components/dashboard/BudgetVsActual.tsx`

### Steps
1. [x] Create component with Card layout
2. [x] Display budget name and summary (total budgeted, actual, variance)
3. [x] Show progress bars for top 5 line items
4. [x] Add warning indicator (red) when line item exceeds budget
5. [x] Implement empty state when no active budget exists
6. [x] Add loading state during data fetch
7. [x] Add error state with retry option

---

## Task 5: Update Dashboard Page Layout

### Description
Add BudgetVsActual widget to the dashboard page.

### Files to Modify
- `frontend/src/app/dashboard/page.tsx`

### Steps
1. [x] Import BudgetVsActual component
2. [x] Add BudgetVsActual widget below the existing Cash Flow / Recent Activity grid
3. [x] Ensure responsive layout works on mobile/tablet/desktop

---

## Task 6: E2E Testing

### Description
Verify all dashboard functionality works end-to-end using Playwright MCP.

### Test Credentials
- Email: `corp@zeltra.io`
- Password: `qwertyui`
- URL: `http://10.0.0.5:3000`

### Steps
1. [x] Navigate to dashboard page (login if needed)
2. [x] Verify 4 metric cards display with data
3. [x] Verify Cash Flow chart renders
4. [x] Verify Recent Activity feed shows items
5. [x] Verify Budget vs Actual widget displays (or empty state if no budget)
6. [x] Check console for errors
7. [x] Verify no TypeScript/lint errors

---

## Task 7: Save Bugs to Cognio Memory

### Description
Save discovered bugs to Cognio project `zeltra-bug` for learning and tracking.

### Steps
1. [x] Set active project to `zeltra-bug`
2. [x] Save BUG-004: OpenAPI spec query params incorrectly marked as path params (utoipa issue)
3. [x] Save BUG-005: CashFlowDataPoint type mismatch (string vs number)
4. [x] Save BUG-006: Missing Budget vs Actual widget (backend exists, frontend missing)
5. [x] Include context about root cause and fix approach
