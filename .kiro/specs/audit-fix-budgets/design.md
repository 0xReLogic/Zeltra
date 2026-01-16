# Design: Audit & Fix 04-Budgets Schema

## Overview

Fix mismatch antara backend Utoipa annotations, OpenAPI schema, dan frontend types untuk Budget API endpoints. Approach: update backend annotations dan response structures untuk match OpenAPI schema yang sudah ada, kemudian regenerate dan sync frontend.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Backend (Rust/Axum)                       │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ budgets.rs - Route Handlers                                 ││
│  │  - Add wrapper response structs (GetBudgetsResponse, etc)   ││
│  │  - Fix utoipa annotations to match actual responses         ││
│  │  - Fix BudgetVsActualQuery with #[into_params]              ││
│  │  - Align vs-actual field names with OpenAPI schema          ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ cargo run --bin generate-openapi
┌─────────────────────────────────────────────────────────────────┐
│                     OpenAPI (contracts/)                         │
│  - openapi.yaml (regenerated)                                   │
│  - openapi-split/04-budgets-schemas.yaml                        │
│  - openapi-split/20-budgets-endpoints.yaml                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ pnpm openapi-typescript
┌─────────────────────────────────────────────────────────────────┐
│                      Frontend (Next.js)                          │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ types/budgets.ts - Use generated types                      ││
│  │ queries/budgets.ts - Handle wrapper responses               ││
│  │ budgets/page.tsx - Remove workarounds                       ││
│  │ budgets/[id]/page.tsx - Update for new types                ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Backend Response Structs (New)

```rust
/// Response wrapper for list budgets.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GetBudgetsResponse {
    /// List of budgets.
    pub budgets: Vec<BudgetResponse>,
}

/// Response wrapper for budget lines.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GetBudgetLinesResponse {
    /// List of budget lines.
    pub lines: Vec<BudgetLineResponse>,
}

/// Response for a single budget line.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BudgetLineResponse {
    /// Line ID.
    pub id: Uuid,
    /// Account ID.
    pub account_id: Uuid,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Budgeted amount.
    pub amount: String,
    /// Notes.
    pub notes: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}
```

### Backend Query Params Fix

```rust
/// Query parameters for budget vs actual.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]  // ADD THIS LINE
pub struct BudgetVsActualQuery {
    /// Filter by fiscal period ID.
    pub fiscal_period_id: Option<Uuid>,
    /// Filter by dimension value IDs (comma-separated).
    pub dimensions: Option<String>,
}
```

### Backend vs-actual Response Fix

Current response:
```json
{
  "budget_id": "...",
  "lines": [...],
  "summary": {
    "total_budgeted": "...",
    "total_actual": "...",
    "total_variance": "...",
    "overall_utilization": "..."
  }
}
```

Fixed response (match OpenAPI schema):
```json
{
  "budget_id": "...",
  "budget_name": "...",
  "line_items": [...],
  "summary": {
    "total_budgeted": "...",
    "total_actual": "...",
    "variance": "...",
    "variance_percent": 0.0
  }
}
```

### Frontend Type Updates

```typescript
// types/budgets.ts - Use generated wrapper types
import type {
  GetBudgetsResponse,
  GetBudgetLinesResponse,
  BudgetResponse,
  BudgetLineResponse,
  // ... other types
} from './api-helpers'

export type { GetBudgetsResponse, GetBudgetLinesResponse, BudgetResponse, BudgetLineResponse }
```

```typescript
// queries/budgets.ts - Handle wrapper responses
export function useBudgets(filters?: BudgetFilters) {
  return useQuery({
    queryKey: BUDGET_KEYS.list(filters),
    queryFn: async () => {
      const response = await apiClient<GetBudgetsResponse>(`/budgets...`)
      return response.budgets  // Extract from wrapper
    },
  })
}
```

## Data Models

### OpenAPI Schema Alignment

| Endpoint | Current OpenAPI | Backend Actual | Fix |
|----------|-----------------|----------------|-----|
| GET /budgets | `[BudgetResponse]` | `{ budgets: [...] }` | Add `GetBudgetsResponse` |
| GET /budgets/{id}/lines | No body | `{ lines: [...] }` | Add `GetBudgetLinesResponse` |
| POST /budgets/{id}/lines | No body | `{ lines: [...] }` | Add `GetBudgetLinesResponse` |
| GET /budgets/{id}/vs-actual | `BudgetVsActualResponse` | Different fields | Align field names |

### Field Name Mapping (vs-actual)

| OpenAPI Schema | Backend Current | Action |
|----------------|-----------------|--------|
| `line_items` | `lines` | Change backend to `line_items` |
| `summary.variance` | `summary.total_variance` | Change backend to `variance` |
| `summary.variance_percent` | `summary.overall_utilization` | Change backend to `variance_percent` |
| `budget_name` | (missing) | Add to backend response |

## Error Handling

No changes needed - existing error handling is correct.

## Testing Strategy

### E2E Tests (Playwright)

1. **Budget List Flow**
   - Login and navigate to /dashboard/budgets
   - Verify budget list renders correctly
   - Verify summary cards show correct totals

2. **Create Budget Flow**
   - Click "New Budget" button
   - Fill form and submit
   - Verify new budget appears in list

3. **Budget Detail Flow**
   - Click on a budget from list
   - Verify detail page renders with lines

4. **Budget vs Actual Flow** (if data available)
   - Navigate to vs-actual view
   - Verify comparison data renders

### Manual Verification

- Loading states work correctly
- Error toasts show on failures
- Data refreshes without full page reload
- Responsive design works on mobile
