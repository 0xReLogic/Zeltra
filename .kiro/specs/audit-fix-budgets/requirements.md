# Audit & Fix 04-Budgets Schema

## Summary

Audit menemukan mismatch antara backend implementation, OpenAPI annotations, dan frontend types untuk Budget API. Root cause: Utoipa annotations tidak match dengan actual JSON response yang di-return handler.

## Issues Found

### Issue 1: list_budgets - Response Wrapper Mismatch

**Location**: `backend/crates/api/src/routes/budgets.rs` line 238

**Problem**:
- Annotation: `body = [BudgetResponse]` (raw array)
- Actual response: `Json(json!({ "budgets": response }))` (wrapper object)

**OpenAPI Generated** (contracts/openapi-split/20-budgets-endpoints.yaml):
```yaml
responses:
  '200':
    content:
      application/json:
        schema:
          items:
            $ref: '#/components/schemas/BudgetResponse'
          type: array
```

**Frontend Workaround** (frontend/src/app/dashboard/budgets/page.tsx line 63):
```typescript
const budgets = Array.isArray(data) ? data : []
```

**Fix**: Add `GetBudgetsResponse` wrapper struct dengan field `budgets: Vec<BudgetResponse>`

---

### Issue 2: list_budget_lines - Missing Response Schema

**Location**: `backend/crates/api/src/routes/budgets.rs` line 640

**Problem**:
- Annotation: No response body defined (hanya `description = "List of budget lines"`)
- Actual response: `Json(json!({ "lines": response }))`

**Fix**: Add `GetBudgetLinesResponse` wrapper struct dengan field `lines: Vec<BudgetLineResponse>`

---

### Issue 3: create_budget_lines - Missing Response Schema

**Location**: `backend/crates/api/src/routes/budgets.rs` line 720

**Problem**:
- Annotation: No response body defined (hanya `description = "Budget lines created successfully"`)
- Actual response: `Json(json!({ "lines": response }))`

**Fix**: Add response schema `body = GetBudgetLinesResponse`

---

### Issue 4: get_budget_vs_actual - Query Params as Path Params

**Location**: `backend/crates/api/src/routes/budgets.rs` line 830

**Problem**:
- `BudgetVsActualQuery` struct missing `#[into_params(parameter_in = Query)]`
- OpenAPI shows `fiscal_period_id` dan `dimensions` as `in: path` (should be `in: query`)

**OpenAPI Generated** (contracts/openapi-split/20-budgets-endpoints.yaml):
```yaml
- description: Filter by fiscal period ID.
  in: path  # WRONG - should be query
  name: fiscal_period_id
  required: true  # WRONG - should be optional
```

**Fix**: Add `#[into_params(parameter_in = Query)]` to `BudgetVsActualQuery` struct

---

### Issue 5: get_budget_vs_actual - Field Name Mismatch

**Location**: `backend/crates/api/src/routes/budgets.rs` line 920-930

**Problem**:
Backend returns:
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

OpenAPI schema expects (contracts/openapi-split/04-budgets-schemas.yaml):
```yaml
BudgetVsActualResponse:
  properties:
    budget_id: ...
    budget_name: ...
    line_items: ...  # Backend returns "lines"
    summary:
      $ref: '#/components/schemas/BudgetSummary'

BudgetSummary:
  properties:
    total_budgeted: ...
    total_actual: ...
    variance: ...           # Backend returns "total_variance"
    variance_percent: ...   # Backend returns "overall_utilization"
```

**Fix Options**:
1. Update backend to match OpenAPI schema names (preferred - less frontend changes)
2. Update OpenAPI schema to match backend (requires frontend type changes)

**Recommended**: Update backend field names to match OpenAPI:
- `lines` → `line_items`
- `total_variance` → `variance`
- `overall_utilization` → `variance_percent`
- Add `budget_name` field

---

### Issue 6: BudgetLineResponse Missing Schema

**Problem**: Backend returns budget line objects dengan structure:
```json
{
  "id": "...",
  "account_id": "...",
  "fiscal_period_id": "...",
  "amount": "...",
  "notes": "...",
  "dimensions": [...]
}
```

Tapi tidak ada `BudgetLineResponse` schema di OpenAPI, hanya `BudgetLineInput` (untuk request) dan `BudgetLineItemResponse` (untuk vs-actual).

**Fix**: Add `BudgetLineResponse` schema untuk GET /lines endpoint

---

## Requirements

### REQ-1: Fix list_budgets Response Schema
- Add `GetBudgetsResponse` wrapper struct dengan `#[derive(Serialize, utoipa::ToSchema)]`
- Update utoipa annotation: `body = GetBudgetsResponse`
- Regenerate OpenAPI

### REQ-2: Fix list_budget_lines Response Schema
- Add `GetBudgetLinesResponse` wrapper struct
- Add `BudgetLineResponse` schema untuk individual line
- Update utoipa annotation dengan response body

### REQ-3: Fix create_budget_lines Response Schema
- Update utoipa annotation dengan response body `GetBudgetLinesResponse`

### REQ-4: Fix get_budget_vs_actual Query Params
- Add `#[into_params(parameter_in = Query)]` to `BudgetVsActualQuery`
- Regenerate OpenAPI to show params as `in: query`

### REQ-5: Fix get_budget_vs_actual Response Fields
- Update backend response field names to match OpenAPI schema
- Add `budget_name` field to response
- Change `lines` → `line_items`
- Change `total_variance` → `variance`
- Change `overall_utilization` → `variance_percent`

### REQ-6: Update Frontend Types
- Update `frontend/src/types/budgets.ts` to use generated types
- Update `frontend/src/lib/queries/budgets.ts` to handle wrapper responses
- Remove workarounds di `frontend/src/app/dashboard/budgets/page.tsx`

### REQ-7: E2E Testing
- Test list budgets flow
- Test create budget flow
- Test budget detail view
- Test budget vs actual (if data available)

---

## Files to Modify

### Backend
- `backend/crates/api/src/routes/budgets.rs`

### Contracts
- `contracts/openapi.yaml` (regenerated)
- `contracts/openapi-split/04-budgets-schemas.yaml` (regenerated)
- `contracts/openapi-split/20-budgets-endpoints.yaml` (regenerated)

### Frontend
- `frontend/src/types/budgets.ts`
- `frontend/src/lib/queries/budgets.ts`
- `frontend/src/app/dashboard/budgets/page.tsx`
- `frontend/src/app/dashboard/budgets/[id]/page.tsx`
