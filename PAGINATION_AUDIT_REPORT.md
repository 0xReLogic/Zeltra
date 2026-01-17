# Backend Pagination Schema Audit Report

## Executive Summary

This audit reveals **critical type mismatches** between the OpenAPI specification and the Rust backend implementation for pagination schemas. There are **4 different pagination implementations** in the backend, but only some match the OpenAPI spec.

## Key Findings

### 1. ❌ CRITICAL: Type Mismatches in `PageMeta`

**OpenAPI Spec** (`contracts/openapi-split/11-common-schemas.yaml`):
```yaml
PageMeta:
  properties:
    page: 
      type: integer
      format: int32          # ← 32-bit integer
    per_page: 
      type: integer
      format: int32          # ← 32-bit integer
    total: 
      type: integer
      format: int64          # ← 64-bit integer
    total_pages: 
      type: integer
      format: int32          # ← 32-bit integer
```

**Backend Implementation** (`backend/crates/shared/src/types/pagination.rs`):
```rust
pub struct PageMeta {
    pub page: u32,           // ✅ Matches (int32)
    pub per_page: u32,       // ✅ Matches (int32)
    pub total: u64,          // ✅ Matches (int64)
    pub total_pages: u32,    // ✅ Matches (int32)
}
```

**Status**: ✅ **MATCHES** - Backend correctly implements the spec.

---

### 2. ❌ CRITICAL: Type Mismatches in `PaginationResponse`

**OpenAPI Spec**:
```yaml
PaginationResponse:
  properties:
    page: 
      type: integer
      format: int64          # ← 64-bit integer
    limit: 
      type: integer
      format: int64          # ← 64-bit integer
    total: 
      type: integer
      format: int64          # ← 64-bit integer
    total_pages: 
      type: integer
      format: int64          # ← 64-bit integer
```

**Backend Implementation** (`backend/crates/api/src/routes/reports.rs:344`):
```rust
pub struct PaginationResponse {
    pub page: u64,           // ✅ Matches (int64)
    pub limit: u64,          // ✅ Matches (int64)
    pub total: u64,          // ✅ Matches (int64)
    pub total_pages: u64,    // ✅ Matches (int64)
}
```

**Status**: ✅ **MATCHES** - Backend correctly implements the spec.

**Used By**:
- `AccountLedgerResponse` in `/api/v1/organizations/{org_id}/reports/accounts/{account_id}/ledger`

---

### 3. ❌ CRITICAL: Type Mismatches in `PaginationMeta` (Transactions)

**OpenAPI Spec**: ❌ **NOT DEFINED** in OpenAPI spec

**Backend Implementation** (`backend/crates/api/src/routes/transactions.rs:318`):
```rust
pub struct PaginationMeta {
    pub page: u64,           // 0-indexed
    pub limit: u64,
    pub total: u64,
}
```

**Status**: ❌ **MISSING FROM OPENAPI SPEC**

**Used By**:
- `PaginatedTransactionsResponse` in `/api/v1/organizations/{org_id}/transactions`

**Issue**: This schema is used in production but not documented in OpenAPI spec.

---

### 4. ✅ `PaginationInfo` (Dashboard)

**OpenAPI Spec**:
```yaml
PaginationInfo:
  properties:
    limit: 
      type: integer
      format: int64
    has_more: 
      type: boolean
    next_cursor: 
      type: string
      nullable: true
```

**Backend Implementation** (`backend/crates/api/src/routes/dashboard.rs:204`):
```rust
pub struct PaginationInfo {
    pub limit: u64,          // ✅ Matches (int64)
    pub has_more: bool,      // ✅ Matches
    pub next_cursor: Option<String>, // ✅ Matches
}
```

**Status**: ✅ **MATCHES** - Backend correctly implements the spec.

**Used By**:
- `RecentActivityResponse` in `/api/v1/organizations/{org_id}/dashboard/activity`

---

### 5. ❌ Database Repository Pagination

**Implementation** (`backend/crates/db/src/repositories/account.rs:86`):
```rust
pub struct AccountListPagination {
    pub total: u64,
    pub page: u64,           // 1-indexed
    pub limit: u64,
    pub total_pages: u64,
}
```

**Status**: Internal use only, not exposed in API responses.

---

## Field Analysis

### `total_pages` Field

✅ **Backend DOES return `total_pages`** in:
1. `PageMeta` (u32) - Used by `PageResponse<T>`
2. `PaginationResponse` (u64) - Used by `AccountLedgerResponse`
3. Database layer `AccountListPagination` (u64)

❌ **Backend DOES NOT return `total_pages`** in:
- `PaginationMeta` (transactions) - Only has `page`, `limit`, `total`
- `PaginationInfo` (dashboard) - Uses cursor-based pagination instead

### Calculation Logic

All implementations correctly calculate `total_pages`:
```rust
let total_pages = if total == 0 { 
    1  // or 0 in some cases
} else { 
    total.div_ceil(limit) 
};
```

---

## ApiError.details Analysis

### OpenAPI Spec
```yaml
ApiError:
  properties:
    details:
      description: Additional error details.
      # ❌ NO TYPE SPECIFIED
```

### Backend Implementation
```rust
pub struct ApiError {
    pub details: Option<serde_json::Value>,  // ← Generic JSON value
}
```

### Actual Usage

**Found 2 uses of `.with_details()`**:

1. **Rate Limiting** (`backend/crates/api/src/error.rs:150`):
```rust
ApiError::rate_limited(60)
    .with_details(serde_json::json!({ "retry_after": 60 }))
```

2. **Tests** (`backend/crates/api/src/middleware/error_tests.rs:168`):
```rust
ApiError::new("validation_error", "Invalid input")
    .with_details(serde_json::json!({
        "field": "email",
        "reason": "invalid format"
    }))
```

**Status**: 
- ✅ Backend correctly uses `serde_json::Value` (flexible JSON)
- ❌ OpenAPI spec should specify type as `object` or provide examples
- ⚠️ Very limited production usage (only rate limiting)

---

## Endpoints Using Pagination

### 1. `PageResponse<T>` (Standard Pagination)
- ✅ `/api/v1/organizations/{org_id}/exchange-rates` → `PageResponse<ExchangeRateListItem>`

### 2. `PaginationResponse` (Custom)
- ✅ `/api/v1/organizations/{org_id}/reports/accounts/{account_id}/ledger` → `AccountLedgerResponse`

### 3. `PaginationMeta` (Transactions - Missing from OpenAPI)
- ❌ `/api/v1/organizations/{org_id}/transactions` → `PaginatedTransactionsResponse`

### 4. `PaginationInfo` (Cursor-based)
- ✅ `/api/v1/organizations/{org_id}/dashboard/activity` → `RecentActivityResponse`

---

## Critical Issues Summary

| Issue | Severity | Impact |
|-------|----------|--------|
| `PaginationMeta` missing from OpenAPI spec | 🔴 HIGH | Transactions endpoint not documented |
| `ApiError.details` has no type in OpenAPI | 🟡 MEDIUM | Unclear contract for error details |
| Multiple pagination schemas | 🟡 MEDIUM | Inconsistent API design |
| `PaginationMeta` missing `total_pages` | 🟡 MEDIUM | Inconsistent with other pagination |

---

## Recommendations

### 1. Add Missing Schema to OpenAPI
Add `PaginationMeta` to `11-common-schemas.yaml`:
```yaml
PaginationMeta:
  description: Pagination metadata for transactions.
  properties:
    page:
      description: Current page number (0-indexed).
      type: integer
      format: int64
      minimum: 0
    limit:
      description: Items per page.
      type: integer
      format: int64
      minimum: 0
    total:
      description: Total number of items.
      type: integer
      format: int64
      minimum: 0
  required:
    - page
    - limit
    - total
  type: object
```

### 2. Fix ApiError.details Type
Update OpenAPI spec:
```yaml
ApiError:
  properties:
    details:
      description: Additional error details.
      type: object
      additionalProperties: true
      example:
        retry_after: 60
```

### 3. Consider Standardization
- Consolidate to 2 pagination types:
  - **Offset-based**: Use `PageResponse<T>` with `PageMeta` everywhere
  - **Cursor-based**: Use `PaginationInfo` for real-time feeds

### 4. Add `total_pages` to `PaginationMeta`
Update transactions pagination to include `total_pages` for consistency.

---

## Verification Commands

```bash
# Find all pagination struct definitions
rg "struct.*Pagination" --type rust

# Find all endpoints returning paginated data
rg "body = (PageResponse|PaginationResponse|PaginationMeta|PaginationInfo)" --type rust

# Find ApiError.details usage
rg "\.with_details\(" --type rust
```
