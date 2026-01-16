# Design Document: Audit & Fix 05-Reports Schema

## Overview

This design document outlines the technical approach to fix mismatches between the OpenAPI specification, backend Rust implementation, and frontend TypeScript types for the Reports module. The primary issues are:

1. **Backend Utoipa**: Query parameters incorrectly generated as path parameters (missing `parameter_in = Query`)
2. **Frontend Types**: Manual type definitions don't match actual backend response structures
3. **Split OpenAPI**: Generated split files (05-reports-schemas.yaml, 21-reports-endpoints.yaml) have incorrect parameter locations

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CURRENT STATE (BROKEN)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Backend (reports.rs)          OpenAPI Spec              Frontend           │
│  ┌─────────────────┐          ┌─────────────────┐      ┌─────────────────┐ │
│  │ #[derive(       │  ──X──>  │ parameters:     │      │ interface       │ │
│  │   IntoParams)]  │          │   - in: path    │      │ TrialBalance {  │ │
│  │ struct Query {  │          │     name: as_of │      │   data: []      │ │
│  │   as_of: Option │          │   - in: path    │      │   total_debit   │ │
│  │   dimensions    │          │     name: dims  │      │ }               │ │
│  │ }               │          │                 │      │                 │ │
│  └─────────────────┘          └─────────────────┘      └─────────────────┘ │
│         │                            │                        │            │
│         │ Missing:                   │ Wrong:                 │ Wrong:     │
│         │ parameter_in=Query         │ in: path               │ data.data  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           TARGET STATE (FIXED)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Backend (reports.rs)          OpenAPI Spec              Frontend           │
│  ┌─────────────────┐          ┌─────────────────┐      ┌─────────────────┐ │
│  │ #[into_params(  │  ────>   │ parameters:     │      │ // Use generated│ │
│  │  parameter_in = │          │   - in: query   │      │ // types from   │ │
│  │  Query)]        │          │     name: as_of │      │ // api.generated│ │
│  │ #[derive(       │          │   - in: query   │      │ TrialBalance    │ │
│  │   IntoParams)]  │          │     name: dims  │      │ Response {      │ │
│  │ struct Query {  │          │                 │      │   accounts: []  │ │
│  │   as_of: Option │          │                 │      │   totals: {}    │ │
│  │ }               │          │                 │      │ }               │ │
│  └─────────────────┘          └─────────────────┘      └─────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Backend Utoipa Annotations (reports.rs)

**Current State:**
```rust
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TrialBalanceQuery {
    pub as_of: Option<NaiveDate>,
    pub dimensions: Option<String>,
}
```

**Target State:**
```rust
#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TrialBalanceQuery {
    pub as_of: Option<NaiveDate>,
    pub dimensions: Option<String>,
}
```

**Structs to Fix:**
| Struct | Line | Parameters |
|--------|------|------------|
| TrialBalanceQuery | ~57 | as_of, dimensions |
| BalanceSheetQuery | ~67 | as_of |
| IncomeStatementQuery | ~74 | from, to, dimensions |
| DimensionalReportQuery | ~86 | from, to, group_by, account_type, dimensions |
| AccountLedgerQuery | ~103 | from, to, page, limit |

### 2. Split OpenAPI Files

**Files to Update (auto-generated):**
- `contracts/openapi-split/21-reports-endpoints.yaml` - Parameter locations will change from `in: path` to `in: query`
- `contracts/openapi-split/05-reports-schemas.yaml` - No changes needed (schemas are correct)

**Current (Wrong):**
```yaml
parameters:
  - description: As of date (defaults to today).
    in: path  # WRONG
    name: as_of
    required: true
```

**Target (Correct):**
```yaml
parameters:
  - description: As of date (defaults to today).
    in: query  # CORRECT
    name: as_of
    required: false
```

### 3. Frontend Types (api-helpers.ts)

**Add Report Types:**
```typescript
// Re-export from generated types
export type { 
  TrialBalanceResponse,
  TrialBalanceTotals,
  AccountBalanceResponse,
  BalanceSheetResponse,
  BalanceSheetSectionResponse,
  IncomeStatementResponse,
  IncomeStatementSectionResponse,
  DimensionalReportResponse,
  DimensionalReportRowResponse,
} from './api.generated';
```

### 4. Frontend Queries (reports.ts)

**Current (Wrong):**
```typescript
export interface TrialBalanceResponse {
  data: TrialBalanceItem[]  // WRONG - backend returns 'accounts'
  total_debit: string       // WRONG - backend returns 'totals.total_debit'
  total_credit: string
}
```

**Target (Correct):**
```typescript
import type { 
  TrialBalanceResponse,
  BalanceSheetResponse,
  IncomeStatementResponse,
} from '@/types/api-helpers';

// Remove manual interface definitions
// Use generated types directly
```

### 5. Frontend Pages

**Trial Balance Page - Current (Wrong):**
```typescript
const report = data?.data || []  // WRONG
const totalDebit = parseFloat(data?.total_debit || '0')  // WRONG
```

**Trial Balance Page - Target (Correct):**
```typescript
const report = data?.accounts || []  // CORRECT
const totalDebit = parseFloat(data?.totals?.total_debit || '0')  // CORRECT
const isBalanced = data?.totals?.is_balanced ?? false  // CORRECT
```

**Balance Sheet Page - Current (Wrong):**
```typescript
const totalLiabilities = parseFloat(report?.total_liabilities || '0')  // WRONG
```

**Balance Sheet Page - Target (Correct):**
```typescript
// Use section totals from backend
const totalLiabilities = parseFloat(data?.liabilities?.total || '0')  // CORRECT
const totalEquity = parseFloat(data?.equity?.total || '0')  // CORRECT
```

**Income Statement Page - Current (Wrong):**
```typescript
const revenues = report?.revenues || []  // WRONG
const expenses = report?.expenses || []  // WRONG
```

**Income Statement Page - Target (Correct):**
```typescript
// Use full structure from backend
const revenue = data?.revenue?.accounts || []  // CORRECT
const cogs = data?.cost_of_goods_sold?.accounts || []  // CORRECT
const operatingExpenses = data?.operating_expenses?.accounts || []  // CORRECT
const otherIncomeExpenses = data?.other_income_expenses?.accounts || []  // CORRECT
```

## Data Models

### Backend Response Structures (Correct - No Changes)

**TrialBalanceResponse:**
```typescript
{
  report_type: "trial_balance",
  as_of: "2026-01-16",
  currency: "USD",
  accounts: AccountBalanceResponse[],
  totals: {
    total_debit: string,
    total_credit: string,
    is_balanced: boolean
  }
}
```

**BalanceSheetResponse:**
```typescript
{
  report_type: "balance_sheet",
  as_of: "2026-01-16",
  currency: "USD",
  assets: { accounts: AccountBalanceResponse[], total: string },
  liabilities: { accounts: AccountBalanceResponse[], total: string },
  equity: { accounts: AccountBalanceResponse[], total: string },
  total_assets: string,
  total_liabilities_and_equity: string,
  is_balanced: boolean
}
```

**IncomeStatementResponse:**
```typescript
{
  report_type: "income_statement",
  period_start: "2026-01-01",
  period_end: "2026-01-16",
  currency: "USD",
  revenue: { accounts: AccountBalanceResponse[], total: string },
  cost_of_goods_sold: { accounts: AccountBalanceResponse[], total: string },
  gross_profit: string,
  operating_expenses: { accounts: AccountBalanceResponse[], total: string },
  operating_income: string,
  other_income_expenses: { accounts: AccountBalanceResponse[], total: string },
  net_income: string
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system.*

### Property 1: Query Parameters Location
*For any* report endpoint with optional parameters, the OpenAPI spec SHALL define those parameters with `in: query` (not `in: path`).
**Validates: Requirements 1.1-1.6**

### Property 2: Trial Balance Data Accessor
*For any* trial balance response, accessing `response.accounts` SHALL return the array of account balances.
**Validates: Requirements 2.1, 2.5**

### Property 3: Trial Balance Totals Accessor
*For any* trial balance response, accessing `response.totals.total_debit` and `response.totals.total_credit` SHALL return the correct totals.
**Validates: Requirements 2.2, 2.6**

### Property 4: Balance Sheet Section Structure
*For any* balance sheet response, each section (assets, liabilities, equity) SHALL contain `accounts` array and `total` string.
**Validates: Requirements 3.1, 3.5**

### Property 5: Income Statement Full Structure
*For any* income statement response, the response SHALL contain all sections: revenue, cost_of_goods_sold, operating_expenses, other_income_expenses.
**Validates: Requirements 4.1, 4.2**

### Property 6: Type Consistency
*For any* frontend query, the TypeScript type used SHALL match the actual backend response structure.
**Validates: Requirements 6.1, 6.2**

## Error Handling

| Error Scenario | Handling |
|----------------|----------|
| Missing query params | Backend uses defaults (today's date, no filters) |
| Invalid date format | Backend returns 400 Bad Request |
| Invalid date range (from > to) | Backend returns 400 Bad Request |
| Empty group_by for dimensional | Backend returns 400 Bad Request |
| Unauthorized access | Backend returns 403 Forbidden |
| Organization not found | Backend returns 404 Not Found |

## Testing Strategy

### Unit Tests
- Verify query parameter annotations generate correct OpenAPI
- Verify frontend types match generated types

### Integration Tests
- Test each report endpoint with query parameters
- Verify response structure matches TypeScript types

### E2E Tests (Playwright MCP)
1. **Trial Balance Page**
   - Navigate to /dashboard/reports/trial-balance
   - Verify accounts table renders
   - Verify totals display correctly
   - Verify balanced/unbalanced indicator
   - Test CSV export
   - Test PDF export

2. **Balance Sheet Page**
   - Navigate to /dashboard/reports/balance-sheet
   - Verify assets, liabilities, equity sections render
   - Verify totals display correctly
   - Verify balanced indicator

3. **Income Statement Page**
   - Navigate to /dashboard/reports/income-statement
   - Verify revenue and expense sections render
   - Verify net income displays correctly
   - Verify profit/loss color coding

### Test Configuration
- IP: `10.0.0.5` (not localhost)
- Login: `corp@zeltra.io` / `qwertyui`
- Use Playwright MCP (not scripts)
