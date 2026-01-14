# Design Document: Phase 7 Remaining Feature Verification

## Overview

This document outlines the design for verifying and fixing the remaining Phase 6-7 frontend features that have UI implementations but have not been tested against the real backend API. The goal is to ensure all frontend components correctly integrate with backend services using proper OpenAPI types and endpoint paths.

## Architecture

The frontend uses a standard architecture:
- **TanStack Query** for data fetching and caching
- **apiClient** wrapper for authenticated API calls (auto-adds org_id prefix)
- **OpenAPI generated types** from `contracts/openapi.yaml`
- **Shadcn/UI** components for UI

### Current Issues Identified

1. **Simulation Page**: Uses hardcoded `/api/v1/simulation/run` instead of org-scoped endpoint
2. **Dimensional Report**: Uses custom types instead of OpenAPI types
3. **Account Ledger**: Uses wrong response type (`GetTransactionsResponse`)
4. **Fiscal Year Creation**: Needs verification of request/response types
5. **Attachments**: No frontend implementation exists yet

## Components and Interfaces

### 1. Simulation Feature

**Current State:**
```typescript
// frontend/src/app/dashboard/simulation/page.tsx
const runSimulation = async (params: SimulationRequest): Promise<SimulationResult> => {
  const res = await fetch('/api/v1/simulation/run', { ... }) // WRONG: hardcoded path
}
```

**Required Changes:**
- Create `frontend/src/lib/queries/simulation.ts` with proper mutation
- Use `apiClient` which auto-prefixes `/organizations/{org_id}`
- Use OpenAPI types: `RunSimulationRequest`, `SimulationResponse`

**API Endpoint:**
```
POST /organizations/{org_id}/simulation/run
Request: RunSimulationRequest
Response: SimulationResponse
```

### 2. Attachments Feature

**Current State:** No frontend implementation

**Required Implementation:**
- Create `frontend/src/lib/queries/attachments.ts`
- Create `frontend/src/types/attachments.ts` with OpenAPI type exports
- Update transaction detail page to show attachments
- Implement upload flow with presigned URLs

**API Endpoints:**
```
POST /organizations/{org_id}/transactions/{transaction_id}/attachments/upload
  Request: RequestUploadRequest (filename, content_type, file_size)
  Response: UploadUrlResponse (upload_url, attachment_id)

POST /organizations/{org_id}/transactions/{transaction_id}/attachments
  Request: ConfirmUploadRequest (attachment_id, attachment_type)
  Response: AttachmentResponse

GET /organizations/{org_id}/transactions/{transaction_id}/attachments
  Response: AttachmentResponse[]

GET /organizations/{org_id}/attachments/{attachment_id}
  Response: AttachmentResponse (with download_url)

DELETE /organizations/{org_id}/attachments/{attachment_id}
  Response: 204 No Content
```

### 3. Account Ledger Feature

**Current State:**
```typescript
// frontend/src/lib/queries/ledger.ts
export function useLedger(accountId: string) {
  return useQuery({
    queryKey: ['ledger', accountId],
    queryFn: () => apiClient<GetTransactionsResponse>(`/accounts/${accountId}/ledger`), // WRONG type
  })
}
```

**Required Changes:**
- Create proper `LedgerResponse` type from OpenAPI
- Add date range filter parameters
- Update response handling for running balance display

**API Endpoint:**
```
GET /organizations/{org_id}/accounts/{account_id}/ledger
  Query params: start_date?, end_date?
  Response: AccountLedgerResponse (entries with running_balance)
```

### 4. Dimensional Reports Feature

**Current State:**
```typescript
// frontend/src/lib/queries/reports.ts
export interface DimensionalReportData { ... } // Custom type, not from OpenAPI
```

**Required Changes:**
- Use OpenAPI types: `DimensionalReportResponse`, `DimensionalReportRowResponse`
- Verify query parameter names match backend expectations

**API Endpoint:**
```
GET /organizations/{org_id}/reports/dimensional
  Query params: start_date, end_date, dimension_type_id?, dimension_value_id?
  Response: DimensionalReportResponse
```

### 5. Fiscal Year Creation Feature

**Current State:**
```typescript
// frontend/src/lib/queries/fiscal.ts
export function useCreateFiscalYear() {
  return useMutation({
    mutationFn: (data: CreateFiscalYearRequest) =>
      apiClient<FiscalYear>('/fiscal-years', { ... })
  })
}
```

**Required Verification:**
- Verify `CreateFiscalYearRequest` matches OpenAPI schema
- Verify response type matches `FiscalYearResponse`
- Test with real API

## Data Models

### OpenAPI Types to Use

```typescript
// From api.generated.ts
export type RunSimulationRequest = {
  base_period_start: string
  base_period_end: string
  projection_months: number
  revenue_growth_rate?: string | null
  expense_growth_rate?: string | null
  account_adjustments?: Record<string, string> | null
  dimension_filters?: string[] | null
}

export type SimulationResponse = {
  simulation_id: string
  parameters_hash: string
  cached: boolean
  projections: AccountProjectionResponse[]
  annual_summary: AnnualSummaryResponse
  monthly_summary: MonthlySummaryResponse[]
}

export type AttachmentResponse = {
  id: string
  attachment_type: string
  filename: string
  file_size: number
  mime_type: string
  storage_provider: string
  uploaded_by: string
  created_at: string
  transaction_id?: string | null
  download_url?: string | null
  download_url_expires_at?: string | null
}

export type DimensionalReportResponse = {
  rows: DimensionalReportRowResponse[]
  total_amount: string
  currency: string
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Simulation API Integration
*For any* valid simulation parameters, calling the simulation mutation SHALL result in a request to `POST /organizations/{org_id}/simulation/run` with the correct payload structure.
**Validates: Requirements 1.1, 1.4**

### Property 2: Simulation Response Display
*For any* successful simulation response, the UI SHALL display projection data including annual_summary and monthly_summary values.
**Validates: Requirements 1.2**

### Property 3: Attachment Upload Flow
*For any* file upload attempt, the Frontend SHALL first request a presigned URL, then upload directly to storage, then confirm the upload.
**Validates: Requirements 2.1, 2.2, 2.3**

### Property 4: Attachment List Display
*For any* transaction with attachments, viewing the transaction detail SHALL display all linked attachments.
**Validates: Requirements 2.4**

### Property 5: Account Ledger Running Balance
*For any* account ledger query, the response SHALL include entries with running_balance that reflects cumulative balance.
**Validates: Requirements 3.1, 3.2**

### Property 6: Dimensional Report Filtering
*For any* dimensional report query with dimension type filter, the response SHALL only include data for that dimension type.
**Validates: Requirements 4.1, 4.4**

### Property 7: Fiscal Year Creation
*For any* fiscal year creation request, the API SHALL create the fiscal year and auto-generate 12 monthly periods.
**Validates: Requirements 5.1, 5.5**

## Error Handling

### API Error Responses
- **400 Bad Request**: Display validation error message from response body
- **401 Unauthorized**: Redirect to login (handled by apiClient)
- **403 Forbidden**: Display "Access denied" message
- **404 Not Found**: Display "Resource not found" message
- **500 Internal Server Error**: Display generic error with retry option

### File Upload Errors
- **File type not allowed**: Display "File type not supported. Allowed: PDF, PNG, JPG, JPEG, DOC, DOCX"
- **File too large**: Display "File size exceeds 10MB limit"
- **Upload failed**: Display "Upload failed. Please try again."

## Testing Strategy

### Unit Tests
- Test query/mutation hooks with mocked responses
- Test error handling for each API call
- Test file validation logic (type, size)

### Integration Tests (E2E with Playwright)
- Test simulation flow: enter parameters → run → view results
- Test attachment flow: upload → view → download → delete
- Test account ledger: select account → view entries with running balance
- Test dimensional report: select filters → view chart and table
- Test fiscal year creation: fill form → submit → verify in list

### Property-Based Tests
- Not applicable for this verification spec (UI integration testing)
