# Design Document

## Overview

This document describes the technical design for the Dashboard Overview feature, addressing bugs found during audit and implementing missing functionality.

## Architecture

### Component Hierarchy

```
DashboardPage
├── MetricsGrid (4 cards)
│   ├── CashPositionCard
│   ├── BurnRateCard
│   ├── RunwayCard
│   └── PendingApprovalsCard
├── ContentGrid
│   ├── CashFlowChart (col-span-4)
│   ├── RecentActivity (col-span-3) [existing]
│   └── BudgetVsActualWidget (col-span-4) [NEW]
```

### Data Flow

```
Backend (Rust/Axum)
    │
    ▼ (JSON with string decimals)
OpenAPI Spec (contracts/openapi-split/24-dashboard-endpoints.yaml)
    │
    ▼ (Generated types)
Frontend Types (lib/queries/dashboard.ts)
    │
    ▼ (React Query hooks)
Dashboard Components
    │
    ▼ (parseFloat for display)
UI Rendering
```

## Detailed Design

### 1. OpenAPI Specification Fixes (BUG-004)

**Problem**: utoipa generates query params as path params with `required: true`

**File**: `contracts/openapi-split/24-dashboard-endpoints.yaml`

**Changes Required**:

| Endpoint | Parameter | Current | Fixed |
|----------|-----------|---------|-------|
| `/dashboard/metrics` | `period_id` | `in: path, required: true` | `in: query, required: false` |
| `/dashboard/cash-flow` | `months` | `in: path, required: true` | `in: query, required: false` |
| `/dashboard/cash-flow` | `period_id` | `in: path, required: true` | `in: query, required: false` |
| `/dashboard/recent-activity` | `limit` | `in: path, required: true` | `in: query, required: false` |
| `/dashboard/recent-activity` | `type` | `in: path, required: true` | `in: query, required: false` |
| `/dashboard/recent-activity` | `cursor` | `in: path, required: true` | `in: query, required: false` |
| `/dashboard/budget-vs-actual` | `budget_id` | `in: path, required: true` | `in: query, required: false` |

**Schema Fix**: Change `type: [string, 'null']` to `type: string` with `nullable: true`

### 2. Frontend Type Alignment (BUG-005)

**Problem**: `CashFlowDataPoint` expects `number` but backend returns `string` (Decimal)

**File**: `frontend/src/lib/queries/dashboard.ts`

**Current Type**:
```typescript
export interface CashFlowDataPoint {
  month: string
  inflow: number  // WRONG
  outflow: number // WRONG
}
```

**Fixed Type** (matching backend):
```typescript
export interface CashFlowDataPoint {
  month: string
  period_name: string
  inflow: string   // Backend returns Decimal as string
  outflow: string  // Backend returns Decimal as string
  net: string      // Backend returns Decimal as string
}
```

**Hook Update**: Parse strings to numbers in the hook for chart consumption:
```typescript
export function useCashFlowData() {
  return useQuery({
    queryKey: ['dashboard', 'cash-flow'],
    queryFn: async () => {
      const response = await apiClient<CashFlowResponse>('/dashboard/cash-flow')
      // Parse string decimals to numbers for chart rendering
      return (response.data || []).map(point => ({
        ...point,
        inflow: parseFloat(point.inflow) || 0,
        outflow: parseFloat(point.outflow) || 0,
        net: parseFloat(point.net) || 0,
      }))
    },
  })
}
```

### 3. Budget vs Actual Widget (BUG-006)

**Problem**: Backend endpoint exists but frontend has no hook or component

#### 3.1 New Hook

**File**: `frontend/src/lib/queries/dashboard.ts`

```typescript
export interface BudgetVsActualResponse {
  budget_id: string | null
  budget_name: string | null
  summary: {
    total_budgeted: string
    total_actual: string
    variance: string
    variance_percent: number
  }
  line_items: BudgetLineItem[]
}

export interface BudgetLineItem {
  account_id: string
  account_code: string
  account_name: string
  budgeted: string
  actual: string
  variance: string
  variance_percent: number
}

export function useBudgetVsActual(budgetId?: string) {
  return useQuery({
    queryKey: ['dashboard', 'budget-vs-actual', budgetId],
    queryFn: () => apiClient<BudgetVsActualResponse>(
      `/dashboard/budget-vs-actual${budgetId ? `?budget_id=${budgetId}` : ''}`
    ),
  })
}
```

#### 3.2 New Component

**File**: `frontend/src/components/dashboard/BudgetVsActual.tsx`

**Design**:
- Card with title "Budget vs Actual"
- Summary section showing total budgeted, actual, variance
- Progress bars for top 5 line items
- Warning indicator (red) when over budget (variance < 0)
- Empty state when no active budget exists

**UI Mockup**:
```
┌─────────────────────────────────────────┐
│ Budget vs Actual                        │
├─────────────────────────────────────────┤
│ Annual Budget 2026                      │
│                                         │
│ Total Budgeted: $50,000.00              │
│ Total Actual:   $35,000.00              │
│ Variance:       $15,000.00 (30%)   ✓    │
├─────────────────────────────────────────┤
│ Office Expenses                         │
│ ████████████░░░░░░░░ $3,000 / $5,000    │
│                                         │
│ Marketing                               │
│ ██████████████████░░ $9,000 / $10,000   │
│                                         │
│ Travel                          ⚠️      │
│ ████████████████████ $6,000 / $5,000    │
└─────────────────────────────────────────┘
```

### 4. Dashboard Page Layout Update

**File**: `frontend/src/app/dashboard/page.tsx`

Add BudgetVsActual widget below the existing grid:

```tsx
<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
  <Card className="col-span-4">
    {/* Cash Flow Chart */}
  </Card>
  <RecentActivity /> {/* col-span-3 */}
</div>

{/* NEW: Budget vs Actual section */}
<div className="grid gap-4 md:grid-cols-1">
  <BudgetVsActualWidget />
</div>
```

## Type Definitions Summary

### Backend Response Types (Rust → JSON)

| Field | Rust Type | JSON Type | Notes |
|-------|-----------|-----------|-------|
| `balance` | `Decimal` | `string` | "50000.0000" |
| `inflow` | `Decimal` | `string` | "10000.0000" |
| `outflow` | `Decimal` | `string` | "8000.0000" |
| `change_percent` | `f64` | `number` | 11.11 |
| `runway_days` | `i32` | `number` | 180 |
| `count` | `i32` | `number` | 5 |

### Frontend Parsing Strategy

All monetary values come as strings from backend. Parse at hook level:
- Use `parseFloat()` for chart data
- Keep as strings for display (use `formatCurrency()`)

## Correctness Properties

### P1: Type Safety
- All API response types must match OpenAPI schema
- No `any` types in dashboard code
- Strict TypeScript compilation must pass

### P2: Data Integrity
- Monetary values must not lose precision during parsing
- Empty/null responses must be handled gracefully
- Error states must be displayed to user

### P3: UI Consistency
- All currency values use `formatCurrency()` helper
- Loading states shown during data fetch
- Error boundaries catch component failures

### P4: Accessibility
- All cards have proper ARIA labels
- Chart has accessible data table alternative
- Color contrast meets WCAG 2.1 AA

## Testing Strategy

### Unit Tests
- Hook parsing logic (string → number conversion)
- formatCurrency edge cases
- Empty state rendering

### Integration Tests
- Dashboard page renders all widgets
- API error handling displays error state
- Loading states appear during fetch

### E2E Tests (Playwright)
- Login → Dashboard loads all 4 metric cards
- Cash flow chart renders with data
- Recent activity shows items
- Budget vs actual widget displays (or empty state)

## Migration Notes

1. Fix OpenAPI spec first (manual edit, utoipa limitation)
2. Regenerate frontend types if using openapi-typescript
3. Update hooks with correct types
4. Add BudgetVsActual component
5. Update dashboard page layout
6. Run E2E tests to verify

## Files to Modify

| File | Change Type | Description |
|------|-------------|-------------|
| `contracts/openapi-split/24-dashboard-endpoints.yaml` | Fix | Change `in: path` to `in: query` for optional params |
| `frontend/src/lib/queries/dashboard.ts` | Fix + Add | Fix CashFlowDataPoint types, add useBudgetVsActual hook |
| `frontend/src/components/dashboard/BudgetVsActual.tsx` | New | Create Budget vs Actual widget component |
| `frontend/src/app/dashboard/page.tsx` | Update | Add BudgetVsActual widget to layout |
