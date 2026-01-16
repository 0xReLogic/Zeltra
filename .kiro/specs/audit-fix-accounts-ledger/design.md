# Design Document: Audit Fix Accounts Ledger

## Overview

This design addresses synchronization issues between the backend Rust API, OpenAPI specification, and frontend TypeScript types for the Accounts and Ledger domain. The primary goal is to ensure type consistency across all layers by fixing backend annotations, updating OpenAPI schemas, and migrating frontend to use generated types.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Backend       │────▶│   OpenAPI       │────▶│   Frontend      │
│   (Rust/Axum)   │     │   (YAML)        │     │   (TypeScript)  │
│                 │     │                 │     │                 │
│ - utoipa macros │     │ - Schemas       │     │ - api.generated │
│ - Response      │     │ - Endpoints     │     │ - React Query   │
│   structs       │     │                 │     │   hooks         │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Components and Interfaces

### Backend Changes

#### 1. Fix ListAccountsQuery Annotation

Current (incorrect):
```rust
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListAccountsQuery {
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub active: Option<bool>,
    pub currency: Option<String>,
}
```

The utoipa macro generates these as path params. Need to add `#[into_params(parameter_in = Query)]` attribute.

#### 2. Add GetAccountsResponse Wrapper

```rust
/// Response wrapper for list accounts.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GetAccountsResponse {
    /// List of accounts.
    pub accounts: Vec<AccountResponse>,
}
```

#### 3. Update list_accounts Endpoint

Change response annotation from `body = [AccountResponse]` to `body = GetAccountsResponse`.

### Frontend Changes

#### 1. Update types/accounts.ts

Replace manual types with re-exports from generated types:

```typescript
// Re-export from generated types
export type { 
  AccountResponse as Account,
  CreateAccountRequest,
  UpdateAccountRequest,
} from './api-helpers'

// Wrapper type for list response
export interface GetAccountsResponse {
  accounts: AccountResponse[]
}
```

#### 2. Update types/ledger.ts

Already using generated types - no changes needed.

#### 3. Update queries/accounts.ts

- Update return types to use generated types
- Remove duplicate LedgerEntry and GetLedgerResponse interfaces
- Use AccountLedgerResponse from generated types

## Data Models

### AccountResponse (Generated)

```typescript
interface AccountResponse {
  id: string                    // UUID
  code: string
  name: string
  type: string                  // asset|liability|equity|revenue|expense
  subtype?: string | null
  currency: string
  balance: string
  is_active: boolean
  allow_direct_posting: boolean
  parent_id?: string | null
  description?: string | null
}
```

### CreateAccountRequest (Generated)

```typescript
interface CreateAccountRequest {
  code: string
  name: string
  type: string
  currency: string
  subtype?: string | null
  parent_id?: string | null
  description?: string | null
  is_active?: boolean | null      // NEW - was missing
  allow_direct_posting?: boolean | null  // NEW - was missing
}
```

### AccountLedgerResponse (Generated)

```typescript
interface AccountLedgerResponse {
  account_id: string
  code: string
  name: string
  entries: LedgerEntryResponse[]
  pagination: PaginationResponse
}
```

### LedgerEntryResponse (Generated)

```typescript
interface LedgerEntryResponse {
  id: string
  transaction_id: string
  transaction_date: string
  reference_number?: string | null
  description: string
  source_currency: string
  source_amount: string
  exchange_rate: string
  functional_amount: string
  debit: string
  credit: string
  running_balance: string
  dimensions: DimensionValueResponse[]
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Response Wrapper Consistency

*For any* call to list_accounts endpoint, the response SHALL contain an `accounts` key with an array value, never a raw array at the root level.

**Validates: Requirements 1.3, 2.1, 2.2**

### Property 2: Type Field Naming

*For any* AccountResponse, the account type field SHALL be named `type` in JSON (not `account_type`), matching the serde rename attribute.

**Validates: Requirements 2.1, 3.1**

### Property 3: Optional Query Parameters

*For any* call to list_accounts, all query parameters (type, active, currency) SHALL be optional and the endpoint SHALL return results even when no filters are provided.

**Validates: Requirements 1.1, 1.2**

### Property 4: Generated Type Alignment

*For any* frontend type used in accounts domain, it SHALL be derived from or alias to the generated types in api.generated.ts, ensuring automatic sync with OpenAPI changes.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

## Error Handling

### Backend Errors

| Error | Status | Response |
|-------|--------|----------|
| Invalid account type | 400 | `{ error: "invalid_account_type", message: "..." }` |
| Account not found | 404 | `{ error: "not_found", message: "..." }` |
| Duplicate code | 409 | `{ error: "duplicate_code", message: "..." }` |
| Forbidden | 403 | `{ error: "forbidden", message: "..." }` |

### Frontend Error Handling

- Use React Query's error handling
- Display toast notifications for API errors
- Type errors caught at compile time via TypeScript

## Testing Strategy

### Unit Tests

- Verify utoipa generates correct OpenAPI for query params
- Verify response wrapper serialization

### Integration Tests

- Test list_accounts returns wrapper object
- Test create_account accepts all optional fields
- Test account ledger returns full entry data

### E2E Tests

- Login and navigate to accounts page
- Create account with optional fields
- View account ledger
- Toggle account status

### Property Tests

- Property 1: Response wrapper consistency - verify all list responses have wrapper
- Property 3: Optional query params - verify endpoint works with/without filters
