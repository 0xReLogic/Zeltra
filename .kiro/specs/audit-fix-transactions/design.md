# Design Document: Audit Fix Transactions

## Overview

This design addresses the synchronization issues between the Rust backend API, OpenAPI specification, and Next.js frontend for the Transactions domain. The primary goals are:

1. Fix backend response structures to match OpenAPI contracts
2. Reorganize OpenAPI split files for proper domain separation
3. Update frontend types to use generated types consistently
4. Implement comprehensive E2E tests for transaction flows
5. Improve UI/UX for transaction management

## Architecture

### Current State Issues

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    Backend      │     │    OpenAPI      │     │    Frontend     │
│   (Rust/Axum)   │     │   (Contract)    │     │   (Next.js)     │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ void_transaction│ ──X──│ VoidResponse    │ ──X──│ VoidResponse   │
│ returns inline  │     │ expects full    │     │ expects full   │
│ partial JSON    │     │ TransactionResp │     │ TransactionResp│
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ PendingTxResp   │     │ Missing in      │     │ Manual type    │
│ struct defined  │     │ split file      │     │ definition     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Target State

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    Backend      │     │    OpenAPI      │     │    Frontend     │
│   (Rust/Axum)   │     │   (Contract)    │     │   (Next.js)     │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ void_transaction│ ────│ VoidResponse    │ ────│ VoidResponse   │
│ returns proper  │     │ full schema     │     │ generated type │
│ VoidResponse    │     │ in split file   │     │ from OpenAPI   │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ All structs     │     │ All schemas in  │     │ All types from │
│ with utoipa     │     │ correct domain  │     │ api.generated  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Components and Interfaces

### Backend Changes

#### 1. Fix void_transaction Response (transactions.rs)

**Current Implementation (Line ~1530):**
```rust
// Returns inline JSON with partial fields
(
    StatusCode::OK,
    Json(json!({
        "original_transaction": {
            "id": result.original_transaction.id,
            "status": status_to_string(&result.original_transaction.status),
            "voided_at": voided_at,
            // ... partial fields
        },
        "reversing_transaction": {
            "id": result.reversing_transaction.id,
            // ... partial fields
        }
    })),
)
```

**Fixed Implementation:**
```rust
// Returns proper VoidResponse struct with full TransactionResponse
async fn void_transaction(...) -> impl IntoResponse {
    // ... existing logic ...
    
    match workflow_repo.void_transaction(...).await {
        Ok(result) => {
            // Fetch full transaction data for both
            let original_full = tx_repo.get_transaction(org_id, result.original_transaction.id).await;
            let reversing_full = tx_repo.get_transaction(org_id, result.reversing_transaction.id).await;
            
            let response = VoidResponse {
                original_transaction: map_transaction_to_response(original_full),
                reversing_transaction: map_transaction_to_response(reversing_full),
            };
            
            (StatusCode::OK, Json(response)).into_response()
        }
        // ... error handling
    }
}
```

### OpenAPI Split File Changes

#### 2. Update split-openapi.py

Add transaction-related schemas to the TRANSACTION_SCHEMAS list:
```python
TRANSACTION_SCHEMAS = [
    'CreateTransactionRequest',
    'CreateEntryRequest',
    'TransactionResponse',
    'TransactionListItem',
    'UpdateTransactionRequest',
    'ApproveRequest',
    'RejectRequest',
    'VoidRequest',
    'VoidResponse',
    'BulkApproveRequest',
    'BulkApproveResponse',
    'BulkApproveItemResponse',
    'PayInvoiceRequest',
    'EntryResponse',
    # Add missing schemas:
    'PaginatedTransactionsResponse',
    'PaginationMeta',
    'PendingTransactionResponse',
]

REPORT_SCHEMAS = [
    # ... existing
    'LedgerEntryResponse',  # Move from transactions
]

DASHBOARD_SCHEMAS = [
    # ... existing
    'PendingApprovalsResponse',  # Keep only here, remove from transactions
]
```

#### 3. Schema Relocation

| Schema | From | To |
|--------|------|-----|
| LedgerEntryResponse | 02-transactions | 05-reports |
| PendingApprovalsResponse | 02-transactions | 09-dashboard (only) |
| PaginatedTransactionsResponse | (missing) | 02-transactions |
| PaginationMeta | (missing) | 02-transactions |
| PendingTransactionResponse | (missing) | 02-transactions |

### Frontend Changes

#### 4. Update api-helpers.ts

```typescript
// Add missing exports
export type PayInvoiceRequest = Schema<'PayInvoiceRequest'>
export type PendingTransactionResponse = Schema<'PendingTransactionResponse'>
export type PaginatedTransactionsResponse = Schema<'PaginatedTransactionsResponse'>
export type PaginationMeta = Schema<'PaginationMeta'>
```

#### 5. Update transactions.ts

```typescript
// Remove manual GetPendingTransactionsResponse interface
// Use generated type instead
import type { PendingTransactionResponse } from './api-helpers'

// Backend /transactions/pending returns { data: PendingTransactionResponse[] }
export type GetPendingTransactionsResponse = {
  data: PendingTransactionResponse[]
}
```

### E2E Test Structure

#### 6. Transaction E2E Tests (Playwright)

```
frontend/e2e/
├── transactions/
│   ├── create-transaction.spec.ts
│   ├── list-transactions.spec.ts
│   ├── void-transaction.spec.ts
│   ├── pending-approvals.spec.ts
│   └── bulk-approve.spec.ts
```

## Data Models

### VoidResponse (Corrected)

```typescript
interface VoidResponse {
  original_transaction: TransactionResponse;  // Full transaction, not partial
  reversing_transaction: TransactionResponse; // Full transaction, not partial
}
```

### PendingTransactionResponse

```typescript
interface PendingTransactionResponse {
  id: string;                    // UUID
  reference_number: string | null;
  type: string;                  // Transaction type
  transaction_date: string;
  description: string;
  status: string;
  total_amount: string;
  submitted_at: string | null;
  can_approve: boolean;          // Important for UI
}
```

### PaginatedTransactionsResponse

```typescript
interface PaginatedTransactionsResponse {
  transactions: TransactionListItem[];
  pagination: PaginationMeta;
}

interface PaginationMeta {
  page: number;
  limit: number;
  total: number;
}
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: VoidResponse Schema Compliance

*For any* valid transaction that is voided, the response SHALL contain both `original_transaction` and `reversing_transaction` fields, each containing ALL fields defined in the TransactionResponse schema (id, type, transaction_date, description, status, fiscal_period_id, created_by, created_at, updated_at, entries, total_debit, total_credit, timezone).

**Validates: Requirements 1.1, 1.2**

### Property 2: Error Response Schema Compliance

*For any* invalid void request (non-existent transaction, already voided transaction, unauthorized user), the backend SHALL return an error response matching the OpenAPI error schema with appropriate status code and error message.

**Validates: Requirements 1.3**

### Property 3: OpenAPI Reference Integrity

*For any* `$ref` reference in the split OpenAPI files, the referenced schema SHALL exist in one of the split files or the main openapi.yaml.

**Validates: Requirements 3.3**

### Property 4: Type Generation Round-Trip

*For any* schema defined in the OpenAPI specification, regenerating frontend types SHALL produce TypeScript types that match the structure of actual backend API responses.

**Validates: Requirements 4.4**

### Property 5: Pagination Response Structure

*For any* page number and limit combination when listing transactions, the response SHALL contain a `transactions` array and a `pagination` object with `page`, `limit`, and `total` fields where `total` represents the actual count of all transactions.

**Validates: Requirements 5.2**

### Property 6: Transaction Balance Validation

*For any* transaction creation request where the sum of debit entries does not equal the sum of credit entries, the UI SHALL prevent form submission and display a balance error message.

**Validates: Requirements 8.2**

## Error Handling

### Backend Error Responses

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `invalid_transition` | 400 | Invalid status transition (e.g., voiding a draft) |
| `not_found` | 404 | Transaction not found |
| `not_authorized` | 403 | User not authorized to perform action |
| `void_reason_required` | 400 | Void reason is required but missing |
| `cannot_modify_voided` | 400 | Cannot modify already voided transaction |
| `unbalanced_transaction` | 400 | Debits don't equal credits |

### Frontend Error Handling

```typescript
// Error handling pattern for transaction operations
try {
  const response = await voidTransaction(orgId, transactionId, { reason });
  // Handle success - display both transactions
  showVoidSuccess(response.original_transaction, response.reversing_transaction);
} catch (error) {
  if (error.response?.status === 400) {
    // Handle validation errors
    showValidationError(error.response.data.message);
  } else if (error.response?.status === 403) {
    // Handle authorization errors
    showAuthError("You don't have permission to void this transaction");
  } else {
    // Handle unexpected errors
    showGenericError("An error occurred. Please try again.");
  }
}
```

## Testing Strategy

### Unit Tests

Unit tests will cover:
- Backend response mapping functions (`map_transaction_to_response`)
- Frontend type guards and validators
- Form validation logic (balance checking, required fields)

### Property-Based Tests

Using **fast-check** for TypeScript property-based testing:

1. **VoidResponse Schema Test**: Generate random valid transactions, void them, verify response schema
2. **Pagination Test**: Generate random page/limit combinations, verify response structure
3. **Balance Validation Test**: Generate random entry combinations, verify balance check logic

Configuration:
- Minimum 100 iterations per property test
- Tag format: `Feature: audit-fix-transactions, Property N: {property_text}`

### E2E Tests (MCP Playwright)

E2E tests will be executed manually using MCP Playwright tool to interact with the running application. This allows real-time verification of UI behavior and API responses.

**Test Account:**
- Email: corp@zeltra.io
- Password: qwertyui

**E2E Test Flows:**

| Flow | Steps | Validates |
|------|-------|-----------|
| Create Transaction | Login → Navigate to transactions → Click create → Fill form → Submit → Verify response | Req 5.1, 7.1, 8.1-8.5 |
| List Transactions | Login → Navigate to transactions → Verify list loads → Test pagination controls | Req 5.2, 7.2 |
| Void Transaction | Login → Find posted transaction → Click void → Enter reason → Verify both transactions shown | Req 5.3, 7.3 |
| Pending Approvals | Login → Navigate to pending → Verify can_approve status shown correctly | Req 5.4, 7.4 |
| Bulk Approve | Login → Select multiple pending → Click bulk approve → Verify individual statuses | Req 5.5, 7.5 |

**E2E Verification Points:**
1. API responses match expected schema (check Network tab)
2. UI displays correct data from responses
3. Loading states appear during API calls
4. Error messages display correctly on failures
5. Data refreshes after mutations

### Verification Commands

```bash
# Backend tests
cd backend && cargo test

# Frontend type check
cd frontend && pnpm tsc --noEmit

# Frontend build
cd frontend && pnpm build

# E2E tests
cd frontend && pnpm playwright test

# Regenerate OpenAPI
cd backend && cargo build
cd contracts && python split-openapi.py

# Regenerate frontend types
cd frontend && pnpm openapi-typescript ../contracts/openapi.yaml -o src/types/api.generated.ts
```
