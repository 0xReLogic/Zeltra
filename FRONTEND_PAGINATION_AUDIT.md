# Frontend Pagination & Error Types Audit Report

**Date:** 2024
**Scope:** Frontend implementation vs OpenAPI specification

---

## Executive Summary

This audit compares frontend TypeScript types against the OpenAPI specification for pagination schemas and error types. The frontend has **partial implementation** of pagination types with some missing exports and inconsistent usage patterns.

### Key Findings:
- ✅ **ApiError**: Properly defined and used
- ⚠️ **Pagination Types**: 3 of 4 schemas properly exported, 1 missing from helpers
- ⚠️ **Type Mismatches**: Found inconsistencies in pagination field naming
- ⚠️ **Missing Pagination**: Exchange rates page doesn't implement pagination UI

---

## 1. Pagination Type Definitions

### OpenAPI Spec (4 Schemas)

The OpenAPI specification defines **4 distinct pagination schemas**:

#### 1.1 `PaginationResponse` (Offset-based with total pages)
**Location:** `contracts/openapi-split/11-common-schemas.yaml`
```yaml
PaginationResponse:
  description: Pagination response.
  properties:
    limit: integer (int64)      # Items per page
    page: integer (int64)        # Current page (0-indexed)
    total: integer (int64)       # Total items
    total_pages: integer (int64) # Total pages
  required: [page, limit, total, total_pages]
```
**Used by:** `AccountLedgerResponse`

#### 1.2 `PaginationInfo` (Cursor-based)
**Location:** `contracts/openapi-split/11-common-schemas.yaml`
```yaml
PaginationInfo:
  description: Pagination info.
  properties:
    has_more: boolean           # Has more results
    limit: integer (int64)      # Limit
    next_cursor: string | null  # Next cursor
  required: [limit, has_more]
```
**Used by:** `RecentActivityResponse`

#### 1.3 `PaginationMeta` (Simple offset-based)
**Location:** `contracts/openapi-split/02-transactions-schemas.yaml`
```yaml
PaginationMeta:
  description: Pagination metadata.
  properties:
    limit: integer (int64)  # Items per page
    page: integer (int64)   # Current page (0-indexed)
    total: integer (int64)  # Total items
  required: [page, limit, total]
```
**Used by:** `PaginatedTransactionsResponse`

#### 1.4 `PageMeta` (1-indexed page-based)
**Location:** `contracts/openapi-split/11-common-schemas.yaml`
```yaml
PageMeta:
  description: Pagination metadata.
  properties:
    page: integer (int32)       # Current page (1-indexed)
    per_page: integer (int32)   # Items per page
    total: integer (int64)      # Total items
    total_pages: integer (int32) # Total pages
  required: [page, per_page, total, total_pages]
```
**Used by:** `PageResponse_ExchangeRateListItem`

---

### Frontend Implementation

#### Generated Types (✅ All 4 Present)
**Location:** `frontend/src/types/api.generated.ts`

All 4 pagination schemas are correctly generated:
- ✅ `PaginationResponse` (lines 2998-3020)
- ✅ `PaginationInfo` (lines 2968-2979)
- ✅ `PaginationMeta` (lines 2980-2997)
- ✅ `PageMeta` (lines 2886-2912)

#### Type Helpers Export (⚠️ 1 Missing)
**Location:** `frontend/src/types/api-helpers.ts`

```typescript
// ✅ Exported (lines 176-177)
export type PaginationResponse = Schema<'PaginationResponse'>
export type PaginationInfo = Schema<'PaginationInfo'>

// ⚠️ NOT EXPORTED - Only used in PaginatedTransactionsResponse
// PaginationMeta is available via Schema<'PaginationMeta'> but not re-exported

// ❌ NOT EXPORTED - Missing from api-helpers.ts
// PageMeta is only available via direct import from api.generated.ts
```

**Issue:** `PageMeta` and `PageRequest` are not re-exported in `api-helpers.ts`, making them less discoverable for developers.

---

## 2. Pagination Usage by Component

### 2.1 Transactions Page (✅ Correct)
**File:** `frontend/src/app/dashboard/transactions/page.tsx`
**Type Used:** `PaginationMeta` (via `PaginatedTransactionsResponse`)

```typescript
// Query hook
const { data } = useTransactions({ page, limit: 50, ... })

// Type: PaginatedTransactionsResponse
// Structure: { transactions: [], pagination: PaginationMeta }

// UI Implementation (lines 211, 217)
Page {page + 1} of {Math.ceil(data.pagination.total / data.pagination.limit)}
disabled={page >= Math.ceil(data.pagination.total / data.pagination.limit) - 1}
```

**Status:** ✅ Correctly implements 0-indexed pagination with proper calculations

---

### 2.2 Account Ledger Page (✅ Correct)
**File:** `frontend/src/app/dashboard/accounts/[id]/ledger/page.tsx`
**Type Used:** `PaginationResponse` (via `AccountLedgerResponse`)

```typescript
// Query hook
const { data: ledger } = useLedger({ accountId, page, limit, ... })

// Type: AccountLedgerResponse
// Structure: { entries: [], pagination: PaginationResponse }

// UI Implementation (lines 182-200)
{ledger?.pagination && ledger.pagination.total_pages > 1 && (
  <div>
    Page {ledger.pagination.page} of {ledger.pagination.total_pages}
    <Button onClick={() => setPage(page - 1)} disabled={page === 1} />
    <Button onClick={() => setPage(page + 1)} 
            disabled={page >= ledger.pagination.total_pages} />
  </div>
)}
```

**Status:** ✅ Correctly uses `total_pages` field from `PaginationResponse`

---

### 2.3 Dashboard Recent Activity (⚠️ Type Mismatch)
**File:** `frontend/src/lib/queries/dashboard.ts`
**Type Used:** Custom `ActivityResponse` interface (lines 68-87)

```typescript
// ❌ ISSUE: Custom interface instead of generated type
interface ActivityResponse {
    activities: {
        id: string
        // ... fields
    }[]
    // Missing pagination field!
}

export function useRecentActivity() {
    return useQuery({
        queryKey: ['dashboard', 'recent-activity'],
        queryFn: () => apiClient<ActivityResponse>('/dashboard/recent-activity'),
    })
}
```

**OpenAPI Spec:** `RecentActivityResponse` should have:
```yaml
RecentActivityResponse:
  properties:
    activities: array
    pagination: PaginationInfo  # ← Missing in frontend type!
```

**Status:** ❌ Type mismatch - custom interface missing `pagination: PaginationInfo` field

---

### 2.4 Exchange Rates Page (⚠️ No Pagination UI)
**File:** `frontend/src/app/dashboard/master-data/exchange-rates/page.tsx`
**Type Used:** `PageResponse_ExchangeRateListItem` (with `PageMeta`)

```typescript
// Query hook (correct type)
const { data } = useExchangeRates()
// Type: PageResponse_ExchangeRateListItem
// Structure: { data: [], meta: PageMeta }

// ❌ ISSUE: No pagination UI implemented
{(data?.data && Array.isArray(data.data) ? data.data : [])
  .sort((a, b) => b.effective_date.localeCompare(a.effective_date))
  .map((rate, index) => (
    <TableRow key={...}>...</TableRow>
  ))
}
```

**Status:** ⚠️ Type is correct but pagination UI is not implemented. All results are displayed without page controls.

**Recommendation:** Add pagination controls using `data.meta` fields:
```typescript
// Should implement:
Page {data.meta.page} of {data.meta.total_pages}
Showing {data.data.length} of {data.meta.total} rates
```

---

## 3. ApiError Type Audit

### OpenAPI Spec
**Location:** `contracts/openapi-split/11-common-schemas.yaml`

```yaml
ApiError:
  description: Standard API error response.
  properties:
    details: unknown              # Additional error details
    error: string                 # Error code (required)
    message: string               # Human-readable message (required)
    request_id: string | null     # Request ID for tracing
  required: [error, message]
```

---

### Frontend Implementation

#### 3.1 Generated Type (✅ Correct)
**Location:** `frontend/src/types/api.generated.ts` (lines 1408-1427)

```typescript
ApiError: {
    details?: unknown;
    error: string;              // ✅ Required
    message: string;            // ✅ Required
    request_id?: string | null; // ✅ Optional
}
```

**Status:** ✅ Matches OpenAPI spec exactly

---

#### 3.2 Type Helper Export (✅ Correct)
**Location:** `frontend/src/types/api-helpers.ts` (line 180)

```typescript
export type ApiError = Schema<'ApiError'>
```

**Status:** ✅ Properly exported

---

#### 3.3 Custom Error Classes (✅ Well Implemented)
**Location:** `frontend/src/lib/api/client.ts` (lines 54-77)

```typescript
// Base error class
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: string,
    public details?: Record<string, string[]>
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

// Specialized error classes
export class PermissionDeniedError extends ApiError {
  constructor(message: string = 'Permission denied') {
    super(message, 403, 'PERMISSION_DENIED')
  }
}

export class UnauthorizedError extends ApiError {
  constructor(message: string = 'Unauthorized') {
    super(message, 401, 'UNAUTHORIZED')
  }
}
```

**Status:** ✅ Well-designed error hierarchy with proper inheritance

---

#### 3.4 Error Handling (✅ Comprehensive)
**Location:** `frontend/src/lib/api/client.ts` (lines 230-320)

```typescript
// Error response parsing
const errorBody = await res.json().catch(() => ({}))
const message = errorBody.error?.message || `API Error: ${res.status}`
const code = errorBody.error?.code
const details = errorBody.error?.details

// Status-specific handling
switch (res.status) {
  case 400: toast.error(message); break
  case 401: throw new UnauthorizedError(message)
  case 402: throw new ApiError(message, 402, 'PAYMENT_REQUIRED')
  case 403: throw new PermissionDeniedError(message)
  case 404: toast.error('Resource not found'); break
  case 409: toast.error(message); break
  case 422: // Validation errors with details
    if (details) {
      const detailMessages = Object.entries(details)
        .map(([field, errors]) => `${field}: ${errors.join(', ')}`)
        .join('\n')
      toast.error(detailMessages || message)
    }
    break
  default:
    if (res.status >= 500) toast.error('Server error')
}

throw new ApiError(message, res.status, code, details)
```

**Status:** ✅ Comprehensive error handling with:
- Proper parsing of OpenAPI `ApiError` schema
- User-friendly toast notifications
- Specialized error classes for common cases
- Validation error detail extraction

---

## 4. Type Mismatches & Issues

### 4.1 Critical Issues

#### Issue #1: Missing PageMeta Export
**Severity:** Medium
**Location:** `frontend/src/types/api-helpers.ts`

```typescript
// ❌ Missing exports
export type PageMeta = Schema<'PageMeta'>
export type PageRequest = Schema<'PageRequest'>
export type PageResponse_ExchangeRateListItem = Schema<'PageResponse_ExchangeRateListItem'>
```

**Impact:** Developers must import directly from `api.generated.ts` or use verbose `Schema<'PageMeta'>` syntax.

**Fix:** Add to api-helpers.ts under "Pagination" section

---

#### Issue #2: Dashboard Activity Type Mismatch
**Severity:** High
**Location:** `frontend/src/lib/queries/dashboard.ts`

```typescript
// ❌ Current: Custom interface
interface ActivityResponse {
    activities: { ... }[]
    // Missing: pagination field
}

// ✅ Should use generated type
import type { RecentActivityResponse } from '@/types/api-helpers'

export function useRecentActivity() {
    return useQuery({
        queryFn: () => apiClient<RecentActivityResponse>('/dashboard/recent-activity'),
    })
}
```

**Impact:** 
- Type safety lost for pagination field
- Cannot access `pagination.has_more` or `pagination.next_cursor`
- Potential runtime errors if backend adds pagination

**Fix:** Replace custom interface with generated `RecentActivityResponse` type

---

#### Issue #3: Exchange Rates Missing Pagination UI
**Severity:** Medium
**Location:** `frontend/src/app/dashboard/master-data/exchange-rates/page.tsx`

**Impact:** All exchange rates loaded at once, no pagination controls

**Fix:** Implement pagination UI using `data.meta` fields:
```typescript
{data?.meta && (
  <div className="flex items-center justify-between mt-4">
    <div className="text-sm text-muted-foreground">
      Page {data.meta.page} of {data.meta.total_pages}
      ({data.meta.total} total rates)
    </div>
    <div className="flex gap-2">
      <Button onClick={() => setPage(page - 1)} disabled={page === 1}>
        Previous
      </Button>
      <Button onClick={() => setPage(page + 1)} 
              disabled={page >= data.meta.total_pages}>
        Next
      </Button>
    </div>
  </div>
)}
```

---

### 4.2 Naming Inconsistencies

| OpenAPI Schema | Field Name | Frontend Usage | Status |
|----------------|------------|----------------|--------|
| `PaginationResponse` | `page` (0-indexed) | ✅ Used correctly | OK |
| `PaginationMeta` | `page` (0-indexed) | ✅ Used correctly | OK |
| `PageMeta` | `page` (1-indexed) | ⚠️ No UI implementation | Missing |
| `PageMeta` | `per_page` | ⚠️ Not used | Missing |
| `PaginationInfo` | `next_cursor` | ⚠️ Not used | Missing |

**Note:** Different pagination schemas use different indexing:
- `PaginationResponse`, `PaginationMeta`: 0-indexed pages
- `PageMeta`: 1-indexed pages (standard REST convention)

---

## 5. Component-Type Mapping

| Component | Endpoint | Response Type | Pagination Schema | Status |
|-----------|----------|---------------|-------------------|--------|
| Transactions Page | `/transactions` | `PaginatedTransactionsResponse` | `PaginationMeta` | ✅ Correct |
| Account Ledger | `/accounts/{id}/ledger` | `AccountLedgerResponse` | `PaginationResponse` | ✅ Correct |
| Recent Activity | `/dashboard/recent-activity` | `RecentActivityResponse` | `PaginationInfo` | ❌ Type mismatch |
| Exchange Rates | `/exchange-rates/list` | `PageResponse_ExchangeRateListItem` | `PageMeta` | ⚠️ No UI |

---

## 6. Recommendations

### High Priority

1. **Fix Dashboard Activity Type**
   - Replace custom `ActivityResponse` interface with generated `RecentActivityResponse`
   - Add pagination support if backend returns `PaginationInfo`

2. **Export Missing Pagination Types**
   - Add `PageMeta`, `PageRequest`, `PageResponse_ExchangeRateListItem` to `api-helpers.ts`

3. **Implement Exchange Rates Pagination UI**
   - Add page controls using `data.meta` fields
   - Support `per_page` parameter in query

### Medium Priority

4. **Standardize Pagination Patterns**
   - Create reusable pagination component
   - Document which pagination type to use for new features

5. **Add Type Tests**
   - Verify frontend types match OpenAPI schemas
   - Catch type drift early

### Low Priority

6. **Consider Pagination Consolidation**
   - Evaluate if 4 different pagination schemas are necessary
   - Consider standardizing on 1-2 patterns across API

---

## 7. Summary Table

| Aspect | OpenAPI Spec | Frontend | Status |
|--------|--------------|----------|--------|
| **Pagination Schemas** | 4 types | 4 types generated | ✅ Complete |
| **Type Exports** | N/A | 2 of 4 exported | ⚠️ Incomplete |
| **ApiError Schema** | Defined | Matches exactly | ✅ Correct |
| **Error Handling** | N/A | Comprehensive | ✅ Excellent |
| **Transactions Pagination** | `PaginationMeta` | Implemented | ✅ Correct |
| **Ledger Pagination** | `PaginationResponse` | Implemented | ✅ Correct |
| **Activity Pagination** | `PaginationInfo` | Type mismatch | ❌ Broken |
| **Exchange Rates Pagination** | `PageMeta` | No UI | ⚠️ Missing |

---

## 8. Action Items

### Immediate (Fix Bugs)
- [ ] Fix `RecentActivityResponse` type in `dashboard.ts`
- [ ] Export `PageMeta` and `PageRequest` in `api-helpers.ts`

### Short Term (Improve UX)
- [ ] Implement pagination UI for exchange rates page
- [ ] Add pagination controls for recent activity (if backend supports)

### Long Term (Technical Debt)
- [ ] Create reusable pagination component
- [ ] Add automated type checking tests
- [ ] Document pagination patterns in developer guide

---

**Audit Completed:** All pagination and error types reviewed
**Overall Assessment:** Frontend implementation is mostly correct with 3 medium-priority issues to address
