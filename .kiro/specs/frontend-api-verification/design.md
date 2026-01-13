# Design Document: Frontend API Verification

## Overview

This design document outlines the approach for verifying and fixing frontend API integration with the real backend. The primary strategy is to migrate from manually-defined types to OpenAPI-generated types and ensure all CRUD operations work correctly with the real API.

## Architecture

### Type System Migration

```
┌─────────────────────────────────────────────────────────────┐
│                    Current State                             │
├─────────────────────────────────────────────────────────────┤
│  Manual Types (types/*.ts)  →  API Client  →  Components    │
│  - May not match backend                                     │
│  - Maintenance burden                                        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Target State                              │
├─────────────────────────────────────────────────────────────┤
│  OpenAPI Types (api.generated.ts)  →  API Client  →  Comps  │
│  - Always matches backend                                    │
│  - Auto-generated from contracts/openapi.yaml                │
└─────────────────────────────────────────────────────────────┘
```

### API Client Pattern

```typescript
// Current pattern in lib/api/client.ts
export async function apiClient<T>(
  endpoint: string,
  options?: RequestInit & { skipAuth?: boolean }
): Promise<T>

// Usage with OpenAPI types
import { components, operations } from '@/types/api.generated'

type Account = components['schemas']['Account']
type CreateAccountRequest = operations['create_account']['requestBody']['content']['application/json']
type CreateAccountResponse = operations['create_account']['responses']['201']['content']['application/json']
```

## Components and Interfaces

### Type Imports Strategy

For each feature, import types from `api.generated.ts`:

```typescript
// Example: Account types
import { components } from '@/types/api.generated'

type Account = components['schemas']['Account']
type AccountWithBalance = components['schemas']['AccountWithBalance']
type CreateAccountRequest = components['schemas']['CreateAccountRequest']
```

### Query/Mutation Pattern

Each feature follows this pattern:

```typescript
// lib/queries/[feature].ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from '@/lib/api/client'
import { useAuthStore } from '@/lib/stores/authStore'
import { components } from '@/types/api.generated'

// Types from OpenAPI
type Entity = components['schemas']['Entity']
type CreateRequest = components['schemas']['CreateEntityRequest']

// Query keys
const QUERY_KEYS = {
  list: (orgId: string) => ['entities', orgId] as const,
  detail: (orgId: string, id: string) => ['entities', orgId, id] as const,
}

// List query
export function useEntities() {
  const orgId = useAuthStore((state) => state.currentOrgId)
  
  return useQuery({
    queryKey: QUERY_KEYS.list(orgId!),
    queryFn: () => apiClient<Entity[]>(`/organizations/${orgId}/entities`),
    enabled: !!orgId,
  })
}

// Create mutation
export function useCreateEntity() {
  const queryClient = useQueryClient()
  const orgId = useAuthStore((state) => state.currentOrgId)
  
  return useMutation({
    mutationFn: (data: CreateRequest) => 
      apiClient<Entity>(`/organizations/${orgId}/entities`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.list(orgId!) })
    },
  })
}
```

## Data Models

### OpenAPI Schema Types

Key types from `api.generated.ts`:

```typescript
// Accounts
Account, AccountWithBalance, CreateAccountRequest, UpdateAccountRequest

// Transactions
Transaction, TransactionWithEntries, CreateTransactionRequest, LedgerEntry

// Budgets
Budget, BudgetWithLines, BudgetLine, CreateBudgetRequest, CreateBudgetLinesRequest

// Dimensions
DimensionType, DimensionValue, CreateDimensionTypeRequest, CreateDimensionValueRequest

// Fiscal
FiscalYear, FiscalPeriod, CreateFiscalYearRequest, PeriodStatus

// Exchange Rates
ExchangeRate, CreateExchangeRateRequest, BulkImportRatesRequest

// Simulation
SimulationRequest, SimulationResult

// Approval Rules
ApprovalRule, CreateApprovalRuleRequest

// Attachments
Attachment, UploadUrlRequest, UploadUrlResponse
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system.*

### Property 1: Org-Scoped Endpoint Prefix
*For any* org-scoped API call (transactions, accounts, budgets, dimensions, fiscal-years, exchange-rates, approval-rules, attachments, simulation, reports), the endpoint SHALL be prefixed with `/organizations/{org_id}/`.
**Validates: Requirements 2.1, 2.2, 2.3, 3.1-3.6, 4.1-4.5, 5.1-5.4, 6.1-6.5, 7.1-7.4, 8.1-8.4, 9.1, 10.1-10.5, 11.1-11.4, 12.1**

### Property 2: Error Message Display
*For any* API error response (400, 403, 404, 500), the System SHALL display an appropriate error message via toast notification matching the error type.
**Validates: Requirements 14.1, 14.2, 14.3, 14.4, 14.5**

### Property 3: Transaction Workflow State Transitions
*For any* transaction workflow action, the transaction status SHALL transition correctly according to the state machine:
- submit: draft → pending
- approve: pending → approved
- reject: pending → draft
- post: approved → posted
- void: posted → voided
**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5**

### Property 4: Query Cache Invalidation
*For any* successful create, update, or delete mutation, the related query cache SHALL be invalidated to ensure data freshness.
**Validates: Requirements 2.1, 2.2, 2.3, 4.1, 4.2, 4.3, 5.1, 5.2, 6.1, 6.2, 6.3, 10.1, 10.2, 10.3**

## Error Handling

### API Error Response Format

Backend returns errors in this format:
```json
{
  "error": {
    "code": "validation_error",
    "message": "Human readable message",
    "details": { ... }
  }
}
```

### Error Handling Strategy

```typescript
// In API client
if (!res.ok) {
  const error = await res.json()
  const message = error.error?.message || 'An error occurred'
  
  // Show toast based on status code
  if (res.status === 400) {
    toast.error(message)
  } else if (res.status === 403) {
    toast.error('Permission denied')
  } else if (res.status === 404) {
    toast.error('Resource not found')
  } else if (res.status >= 500) {
    toast.error('Server error, please try again')
  }
  
  throw new Error(message)
}
```

## Testing Strategy

### Unit Tests
- Test type compatibility between manual types and OpenAPI types
- Test query key generation
- Test error message formatting

### Integration Tests (E2E with Playwright)
- Test each CRUD operation with real backend
- Test transaction workflow state transitions
- Test file upload flow
- Test error scenarios (validation errors, permission errors)

### Property-Based Tests
- Property 1: Response type validation (mock API responses against OpenAPI schema)
- Property 2: Endpoint prefix validation (all org-scoped calls include org_id)
- Property 5: Transaction state machine validation

### Test Configuration
- Minimum 100 iterations per property test
- Use `fast-check` for property-based testing in TypeScript
- Tag format: **Feature: frontend-api-verification, Property {number}: {property_text}**
