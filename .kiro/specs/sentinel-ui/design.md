# Design Document: Sentinel Intelligence UI

## Overview

This design document outlines the frontend implementation for Zeltra's Sentinel Intelligence module. The backend APIs are already complete - this focuses on building React/Next.js components to consume those APIs and provide a polished user experience for enterprise accounting features.

The implementation follows Zeltra's existing design patterns using shadcn/ui components, TanStack Query for data fetching, and React Hook Form with Zod for form validation.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Sentinel UI Layer                         │
├─────────────────────────────────────────────────────────────────┤
│  Pages                                                           │
│  ├── /dashboard/accruals      → AccrualsPage                    │
│  ├── /dashboard/revaluation   → RevaluationPage                 │
│  └── /dashboard/intercompany  → IntercompanyPage                │
├─────────────────────────────────────────────────────────────────┤
│  Components                                                      │
│  ├── CreateAccrualDialog      → Form for new accrual schedule   │
│  ├── AccrualScheduleTable     → DataTable for schedules         │
│  ├── RevaluationLogTable      → DataTable for reval logs        │
│  ├── IntercompanyMappingTable → DataTable for mappings          │
│  └── CreateMappingDialog      → Form for new IC mapping         │
├─────────────────────────────────────────────────────────────────┤
│  Query Layer (TanStack Query)                                    │
│  ├── useAccrualSchedules()    → GET /accrual-schedules          │
│  ├── useCreateAccrualSchedule()→ POST /accrual-schedules        │
│  ├── useRevaluationLogs()     → GET /revaluation-logs           │
│  ├── useIntercompanyMappings()→ GET /intercompany/mappings      │
│  └── useCreateIntercompanyMapping() → POST /intercompany/connect│
├─────────────────────────────────────────────────────────────────┤
│  Types (from api-helpers.ts)                                     │
│  ├── AccrualScheduleResponse                                     │
│  ├── CreateAccrualScheduleRequest                                │
│  ├── RevaluationLogResponse                                      │
│  ├── IntercompanyMappingResponse                                 │
│  └── CreateIntercompanyMappingRequest                            │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Accruals Page Components

```typescript
// AccrualsPage - Main page component
interface AccrualsPageProps {
  // No props - uses hooks internally
}

// AccrualScheduleTable - Data table for schedules
interface AccrualScheduleTableProps {
  schedules: AccrualScheduleResponse[]
  isLoading: boolean
  onRowClick: (schedule: AccrualScheduleResponse) => void
}

// CreateAccrualDialog - Form dialog
interface CreateAccrualDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

// Zod schema for form validation
const createAccrualSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  description: z.string().optional(),
  total_amount: z.string().refine(val => parseFloat(val) > 0, 'Amount must be positive'),
  currency_id: z.string().min(1, 'Currency is required'),
  debit_account_id: z.string().uuid('Invalid account'),
  credit_account_id: z.string().uuid('Invalid account'),
  start_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, 'Invalid date format'),
  end_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, 'Invalid date format'),
  frequency: z.enum(['daily', 'weekly', 'monthly', 'quarterly', 'yearly']),
  total_periods: z.number().int().positive(),
})
```

### 2. Revaluation Page Components

```typescript
// RevaluationPage - Main page component
interface RevaluationPageProps {
  // No props - uses hooks internally
}

// RevaluationLogTable - Data table for logs
interface RevaluationLogTableProps {
  logs: RevaluationLogResponse[]
  isLoading: boolean
}

// DateRangeFilter - Filter component
interface DateRangeFilterProps {
  startDate: Date | null
  endDate: Date | null
  onDateChange: (start: Date | null, end: Date | null) => void
}
```

### 3. Intercompany Page Components

```typescript
// IntercompanyPage - Main page component
interface IntercompanyPageProps {
  // No props - uses hooks internally
}

// IntercompanyMappingTable - Data table for mappings
interface IntercompanyMappingTableProps {
  mappings: IntercompanyMappingResponse[]
  isLoading: boolean
}

// CreateMappingDialog - Form dialog
interface CreateMappingDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

// Zod schema for form validation
const createMappingSchema = z.object({
  source_account_id: z.string().uuid('Invalid account'),
  target_org_id: z.string().uuid('Invalid organization'),
  target_account_id: z.string().uuid('Invalid account'),
})
```

## Data Models

### API Response Types (from OpenAPI)

```typescript
// AccrualScheduleResponse
interface AccrualScheduleResponse {
  id: string                    // UUID
  organization_id: string       // UUID
  name: string
  description: string | null
  total_amount: string          // Decimal as string
  currency_id: string           // ISO currency code
  debit_account_id: string      // UUID
  credit_account_id: string     // UUID
  start_date: string            // YYYY-MM-DD
  end_date: string              // YYYY-MM-DD
  frequency: string             // daily|weekly|monthly|quarterly|yearly
  total_periods: number
  periods_processed: number
  status: string                // active|completed|paused
  next_run_date: string | null  // YYYY-MM-DD
  created_at: string            // ISO timestamp
}

// RevaluationLogResponse
interface RevaluationLogResponse {
  id: string                    // UUID
  organization_id: string       // UUID
  account_id: string            // UUID
  revaluation_date: string      // YYYY-MM-DD
  currency_id: string           // ISO currency code
  balance_in_currency: string   // Decimal as string
  old_exchange_rate: string     // Decimal as string
  new_exchange_rate: string     // Decimal as string
  unrealized_gain_loss: string  // Decimal as string (positive=gain, negative=loss)
  transaction_id: string | null // UUID of posted adjustment
  created_at: string            // ISO timestamp
}

// IntercompanyMappingResponse
interface IntercompanyMappingResponse {
  id: string                    // UUID
  source_org_id: string         // UUID
  source_account_id: string     // UUID
  target_org_id: string         // UUID
  target_account_id: string     // UUID
  mapping_type: string          // elimination|mirror
  auto_post: boolean
  created_at: string            // ISO timestamp
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Gain/Loss Color Consistency
*For any* revaluation log displayed in the table, if the unrealized_gain_loss value is positive, the text color SHALL be green; if negative, the text color SHALL be red.
**Validates: Requirements 2.2**

### Property 2: Form Validation Completeness
*For any* form submission attempt with invalid data (empty required fields, invalid formats), the form SHALL display appropriate validation errors and prevent submission.
**Validates: Requirements 5.6**

### Property 3: Tier Gating Consistency
*For any* Sentinel page (Accruals, Revaluation, Intercompany), if the user's organization lacks the required tier feature, the page SHALL display an upgrade prompt instead of the feature content.
**Validates: Requirements 1.5, 2.4, 3.4**

### Property 4: Query Cache Invalidation
*For any* successful mutation (create accrual, create mapping), the related list query SHALL be invalidated and refetched to show the new data.
**Validates: Requirements 4.7**

## Error Handling

### API Error Handling

```typescript
// Error response structure from backend
interface ApiErrorResponse {
  error: string       // Error code
  message: string     // Human-readable message
  details?: unknown   // Additional context
}

// Error handling in mutations
onError: (error: ApiError) => {
  if (error.status === 400) {
    toast.error(error.message || 'Invalid input')
  } else if (error.status === 403) {
    toast.error('You do not have permission for this action')
  } else if (error.status === 404) {
    toast.error('Resource not found')
  } else {
    toast.error('An unexpected error occurred')
    console.error('Sentinel API error:', error)
  }
}
```

### Tier Gating Error Handling

```typescript
// Check tier features before rendering content
const { data: org } = useOrganization()

const hasAccruals = org?.tier_features?.has_auto_accruals ?? false
const hasMultiCurrency = org?.tier_features?.has_multi_currency ?? false
const hasIntercompany = org?.tier_features?.has_intercompany_hub ?? false

// Render upgrade prompt if feature not available
if (!hasAccruals) {
  return <UpgradePrompt feature="Automated Accruals" requiredTier="Enterprise" />
}
```

## Testing Strategy

### Unit Tests
- Zod schema validation for form inputs
- Utility functions for formatting amounts, dates
- Component rendering with mock data

### Integration Tests
- React Query hooks with MSW mocks
- Form submission flows
- Error state handling

### E2E Tests (Playwright MCP)
- **Test Environment**: IP 10.0.0.5, Port 3000
- **Credentials**: corp@zeltra.io / qwertyui
- **Test Cases**:
  1. Navigate to Accruals page, verify table renders
  2. Open Create Accrual dialog, fill form, submit
  3. Navigate to Revaluation page, verify logs display
  4. Navigate to Intercompany page, verify mappings display
  5. Test tier gating with non-enterprise user (if available)

### Property-Based Tests
- Form validation with random invalid inputs
- Gain/loss color logic with random positive/negative values
