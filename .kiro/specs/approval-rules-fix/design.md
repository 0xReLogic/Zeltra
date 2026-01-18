# Design: Approval Rules Management (BUG-013)

## 1. Architecture Overview

This design addresses the complete implementation of Approval Rules management across three layers:
1. **OpenAPI Specification** - API contract with validation rules
2. **Backend (Rust)** - Business logic, database operations, API endpoints
3. **Frontend (Next.js/React)** - User interface for CRUD operations

### 1.1 System Context

```
┌─────────────────────────────────────────────────────────────┐
│                     Approval Rules System                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐      ┌──────────────┐      ┌───────────┐ │
│  │   Frontend   │─────▶│   Backend    │─────▶│ Database  │ │
│  │  (Next.js)   │◀─────│   (Rust)     │◀─────│(Postgres) │ │
│  └──────────────┘      └──────────────┘      └───────────┘ │
│         │                      │                             │
│         │                      │                             │
│         ▼                      ▼                             │
│  ┌──────────────┐      ┌──────────────┐                    │
│  │ React Query  │      │  OpenAPI     │                    │
│  │   + Zod      │      │    Spec      │                    │
│  └──────────────┘      └──────────────┘                    │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Component Responsibilities

**OpenAPI Specification**:
- Define API contract
- Specify validation rules
- Document error responses
- Provide examples

**Backend**:
- Enforce business logic
- Validate inputs
- Manage database operations
- Handle authentication/authorization
- Implement pagination
- Provide audit logging

**Frontend**:
- Render UI components
- Handle user interactions
- Validate forms client-side
- Manage API state with React Query
- Provide feedback (toasts, errors)

## 2. Data Model

### 2.1 Database Schema

```sql
CREATE TABLE approval_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    min_amount DECIMAL(19, 4),
    max_amount DECIMAL(19, 4),
    transaction_types transaction_type[] NOT NULL,
    required_role user_role NOT NULL,
    priority SMALLINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT min_max_amount_check CHECK (
        min_amount IS NULL OR max_amount IS NULL OR min_amount <= max_amount
    ),
    CONSTRAINT priority_range_check CHECK (priority >= 1 AND priority <= 100)
);

-- Indexes for performance
CREATE INDEX idx_approval_rules_org_priority 
ON approval_rules(organization_id, priority) 
WHERE is_active = true;

CREATE INDEX idx_approval_rules_tx_types 
ON approval_rules USING GIN(transaction_types) 
WHERE is_active = true;

CREATE INDEX idx_approval_rules_role 
ON approval_rules(organization_id, required_role) 
WHERE is_active = true;

CREATE INDEX idx_approval_rules_amounts 
ON approval_rules(organization_id, min_amount, max_amount) 
WHERE is_active = true;
```

### 2.2 Type Definitions

**Rust (Backend)**:
```rust
pub struct ApprovalRuleModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub transaction_types: Vec<TransactionType>,
    pub required_role: UserRole,
    pub priority: i16,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

pub enum TransactionType {
    Journal, Invoice, Bill, Payment, Expense, Transfer,
    Adjustment, OpeningBalance, Reversal, Accrual, 
    Revaluation, Intercompany
}

pub enum UserRole {
    Viewer, Submitter, Approver, Accountant, Admin, Owner
}
```

**TypeScript (Frontend)**:
```typescript
export type ApprovalRuleResponse = {
  id: string
  organization_id: string
  name: string
  description: string | null
  min_amount: string | null
  max_amount: string | null
  transaction_types: string[]
  required_role: string
  priority: number
  is_active: boolean
  created_at: string
  updated_at: string
}

export type CreateApprovalRuleRequest = {
  name: string
  description?: string | null
  min_amount?: string | null
  max_amount?: string | null
  transaction_types: string[]
  required_role: string
  priority: number
  is_active?: boolean
}
```

## 3. API Design

### 3.1 Endpoints

**List Approval Rules** (with pagination):
```
GET /organizations/{org_id}/approval-rules
Query Parameters:
  - page: integer (default: 1, min: 1)
  - per_page: integer (default: 20, min: 1, max: 100)
  - is_active: boolean (optional)
  - transaction_type: string (optional)
  - required_role: string (optional)
  - sort_by: enum [priority, created_at, name] (default: priority)
  - sort_order: enum [asc, desc] (default: asc)

Response 200:
{
  "data": [ApprovalRuleResponse],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 150,
    "total_pages": 8
  }
}
```

**Get Single Rule**:
```
GET /organizations/{org_id}/approval-rules/{rule_id}
Response 200: ApprovalRuleResponse
```

**Create Rule**:
```
POST /organizations/{org_id}/approval-rules
Body: CreateApprovalRuleRequest
Response 201: ApprovalRuleResponse
```

**Update Rule**:
```
PATCH /organizations/{org_id}/approval-rules/{rule_id}
Body: UpdateApprovalRuleRequest
Response 200: ApprovalRuleResponse
```

**Delete Rule** (soft delete):
```
DELETE /organizations/{org_id}/approval-rules/{rule_id}
Response 204: No Content
```

### 3.2 Error Responses

All endpoints return ApiError schema on error:
```json
{
  "error": "validation_error",
  "message": "Invalid amount format",
  "details": {
    "validation_errors": [
      {
        "field": "min_amount",
        "message": "Must match pattern ^[0-9]+(\\.[0-9]{1,2})?$"
      }
    ]
  }
}
```

Status codes:
- 400: Validation error, invalid input
- 401: Unauthorized (missing/invalid token)
- 403: Forbidden (insufficient permissions)
- 404: Rule not found
- 429: Rate limit exceeded
- 500: Internal server error

## 4. Frontend Design

### 4.1 Page Structure

```
/dashboard/settings/approval-rules
├── Header (title, description, create button)
├── Filters (status, transaction type, role)
├── Data Table
│   ├── Priority column (sortable, badge)
│   ├── Name column (sortable, bold)
│   ├── Transaction Types (badges, truncated)
│   ├── Required Role (badge with color)
│   ├── Amount Range (formatted currency)
│   ├── Status (toggle switch)
│   └── Actions (edit, delete icons)
└── Pagination Controls (prev, next, page info)
```

### 4.2 Component Hierarchy

```
ApprovalRulesPage
├── PageHeader
│   ├── Title
│   ├── Description
│   └── CreateButton → CreateApprovalRuleDialog
├── FiltersBar
│   ├── StatusFilter
│   ├── TransactionTypeFilter
│   └── RoleFilter
├── ApprovalRulesTable (TanStack Table)
│   ├── TableHeader (sortable columns)
│   ├── TableBody
│   │   └── ApprovalRuleRow (for each rule)
│   │       ├── PriorityBadge
│   │       ├── TransactionTypeBadges
│   │       ├── RoleBadge
│   │       ├── AmountRange
│   │       ├── StatusToggle
│   │       └── ActionButtons
│   │           ├── EditButton → EditApprovalRuleDialog
│   │           └── DeleteButton → DeleteConfirmDialog
│   └── EmptyState (when no rules)
└── PaginationControls
    ├── PreviousButton
    ├── PageInfo
    └── NextButton
```

### 4.3 Form Design

**Create/Edit Form Fields**:
1. Name (text input, required, max 255)
2. Description (textarea, optional, max 1000)
3. Transaction Types (multi-select, required, min 1)
4. Required Role (select, required)
5. Priority (number input, required, 1-100)
6. Min Amount (currency input, optional, pattern validation)
7. Max Amount (currency input, optional, pattern validation)
8. Is Active (toggle, default true)

**Validation Rules**:
- Name: required, 1-255 characters
- Transaction Types: at least 1 selected
- Priority: integer, 1-100
- Amounts: regex `/^\d+(\.\d{1,2})?$/`, min <= max
- All fields: sanitized for XSS

### 4.4 State Management

**React Query Keys**:
```typescript
const APPROVAL_RULE_KEYS = {
  all: ['approval-rules'],
  lists: () => [...APPROVAL_RULE_KEYS.all, 'list'],
  list: (filters) => [...APPROVAL_RULE_KEYS.lists(), filters],
  details: () => [...APPROVAL_RULE_KEYS.all, 'detail'],
  detail: (id) => [...APPROVAL_RULE_KEYS.details(), id],
}
```

**Cache Strategy**:
- List cache: 5 minutes
- Detail cache: 5 minutes
- Invalidate list on create/update/delete
- Invalidate detail on update
- Optimistic updates for toggle/delete

## 5. Validation Strategy

### 5.1 Three-Layer Validation

**Layer 1: OpenAPI Specification**
- Defines validation rules in schema
- Used by code generators
- Documentation for developers

**Layer 2: Frontend (Zod)**
- Client-side validation before API call
- Immediate feedback to user
- Prevents unnecessary API calls

**Layer 3: Backend (Rust)**
- Server-side validation (authoritative)
- Prevents invalid data in database
- Returns specific error messages

### 5.2 Validation Rules

**Name**:
- OpenAPI: minLength 1, maxLength 255
- Zod: `.min(1).max(255)`
- Rust: `if name.len() > 255 { return Err(...) }`

**Priority**:
- OpenAPI: minimum 1, maximum 100
- Zod: `.int().min(1).max(100)`
- Rust: `if priority < 1 || priority > 100 { return Err(...) }`

**Amounts**:
- OpenAPI: pattern `^[0-9]+(\.[0-9]{1,2})?$`
- Zod: `.regex(/^\d+(\.\d{1,2})?$/)`
- Rust: `AMOUNT_PATTERN.is_match(s)`

**Transaction Types**:
- OpenAPI: enum, minItems 1, maxItems 10
- Zod: `.array(z.enum(...)).min(1).max(10)`
- Rust: `parse_transaction_types()` with enum matching

## 6. Correctness Properties

### Property 1: Pagination Consistency
**Validates**: Requirements 2.1.2, 2.2.1

**Property**: For any valid page number p and page size s, the list endpoint returns exactly s items (or fewer on last page), and the total count matches the sum of all pages.

**Test Strategy**:
```rust
#[proptest]
fn pagination_consistency(
    #[strategy(1u32..=100)] page: u32,
    #[strategy(1u32..=100)] per_page: u32,
) {
    let (items, total) = list_rules_paginated(org_id, page, per_page).await?;
    
    // Items count <= per_page
    prop_assert!(items.len() <= per_page as usize);
    
    // Last page has remaining items
    let expected_items = if page * per_page > total {
        (total % per_page) as usize
    } else {
        per_page as usize
    };
    prop_assert_eq!(items.len(), expected_items);
    
    // Total pages calculation
    let total_pages = (total + per_page - 1) / per_page;
    prop_assert!(page <= total_pages);
}
```

### Property 2: Amount Range Validation
**Validates**: Requirements 2.1.4, 2.2.8, 2.3.3

**Property**: For any approval rule, if both min_amount and max_amount are specified, min_amount must be less than or equal to max_amount.

**Test Strategy**:
```rust
#[proptest]
fn amount_range_validation(
    #[strategy(any::<Decimal>())] min: Decimal,
    #[strategy(any::<Decimal>())] max: Decimal,
) {
    let input = CreateApprovalRuleInput {
        min_amount: Some(min),
        max_amount: Some(max),
        // ... other fields
    };
    
    let result = create_rule(org_id, input).await;
    
    if min > max {
        prop_assert!(result.is_err());
        prop_assert!(matches!(result, Err(ApprovalRuleError::InvalidAmountRange)));
    } else {
        prop_assert!(result.is_ok());
    }
}
```

### Property 3: Priority Range Enforcement
**Validates**: Requirements 2.1.6, 2.2.5, 2.3.3

**Property**: Priority must always be between 1 and 100 (inclusive). Any value outside this range must be rejected.

**Test Strategy**:
```rust
#[proptest]
fn priority_range_enforcement(
    #[strategy(any::<i16>())] priority: i16,
) {
    let input = CreateApprovalRuleInput {
        priority,
        // ... other fields
    };
    
    let result = create_rule(org_id, input).await;
    
    if priority < 1 || priority > 100 {
        prop_assert!(result.is_err());
        prop_assert!(matches!(result, Err(ApprovalRuleError::InvalidPriority)));
    } else {
        prop_assert!(result.is_ok());
    }
}
```

### Property 4: Transaction Type Completeness
**Validates**: Requirements 2.1.5, 2.2.3, 2.4.4

**Property**: All 12 transaction types must be parseable and matchable. No valid transaction type should be rejected.

**Test Strategy**:
```rust
#[test]
fn transaction_type_completeness() {
    let all_types = vec![
        "journal", "invoice", "bill", "payment", "expense", "transfer",
        "adjustment", "opening_balance", "reversal", "accrual", 
        "revaluation", "intercompany"
    ];
    
    for tx_type in all_types {
        let result = parse_transaction_type(tx_type);
        assert!(result.is_ok(), "Failed to parse: {}", tx_type);
    }
}
```

### Property 5: String Length Constraints
**Validates**: Requirements 2.1.6, 2.2.4, 2.3.3

**Property**: Name must be 1-255 characters, description must be ≤1000 characters. Values exceeding limits must be rejected.

**Test Strategy**:
```rust
#[proptest]
fn string_length_constraints(
    #[strategy("\\PC{0,300}")] name: String,
    #[strategy("\\PC{0,1100}")] description: String,
) {
    let input = CreateApprovalRuleInput {
        name: name.clone(),
        description: Some(description.clone()),
        // ... other fields
    };
    
    let result = create_rule(org_id, input).await;
    
    if name.is_empty() || name.len() > 255 {
        prop_assert!(result.is_err());
    } else if description.len() > 1000 {
        prop_assert!(result.is_err());
    } else {
        prop_assert!(result.is_ok());
    }
}
```

### Property 6: Enum Validation
**Validates**: Requirements 2.1.5, 2.3.3

**Property**: Only valid enum values for required_role and transaction_types must be accepted. Invalid values must be rejected.

**Test Strategy**:
```rust
#[proptest]
fn enum_validation(
    #[strategy(any::<String>())] role: String,
    #[strategy(prop::collection::vec(any::<String>(), 1..=10))] tx_types: Vec<String>,
) {
    let valid_roles = ["viewer", "submitter", "approver", "accountant", "admin", "owner"];
    let valid_types = ["journal", "invoice", "bill", "payment", "expense", "transfer",
                       "adjustment", "opening_balance", "reversal", "accrual", 
                       "revaluation", "intercompany"];
    
    let input = CreateApprovalRuleInput {
        required_role: role.clone(),
        transaction_types: tx_types.clone(),
        // ... other fields
    };
    
    let result = create_rule(org_id, input).await;
    
    let role_valid = valid_roles.contains(&role.to_lowercase().as_str());
    let types_valid = tx_types.iter().all(|t| valid_types.contains(&t.to_lowercase().as_str()));
    
    if !role_valid || !types_valid {
        prop_assert!(result.is_err());
    } else {
        prop_assert!(result.is_ok());
    }
}
```

### Property 7: Cache Invalidation
**Validates**: Requirements 2.3.10, 2.7.3

**Property**: After any mutation (create/update/delete), the list cache must be invalidated and subsequent queries must return fresh data.

**Test Strategy**:
```typescript
test('cache invalidation on create', async () => {
  const queryClient = new QueryClient()
  
  // Initial fetch
  const { data: initial } = await queryClient.fetchQuery({
    queryKey: ['approval-rules', 'list'],
    queryFn: fetchRules
  })
  
  // Create new rule
  await createRule(newRuleData)
  
  // Verify cache was invalidated
  const cacheState = queryClient.getQueryState(['approval-rules', 'list'])
  expect(cacheState?.isInvalidated).toBe(true)
  
  // Fetch again
  const { data: updated } = await queryClient.fetchQuery({
    queryKey: ['approval-rules', 'list'],
    queryFn: fetchRules
  })
  
  // New rule should be in list
  expect(updated.length).toBe(initial.length + 1)
})
```

### Property 8: Optimistic Update Rollback
**Validates**: Requirements 2.3.10

**Property**: If an optimistic update fails, the UI must rollback to the previous state.

**Test Strategy**:
```typescript
test('optimistic update rollback on error', async () => {
  const queryClient = new QueryClient()
  
  // Get initial state
  const initial = queryClient.getQueryData(['approval-rules', 'list'])
  
  // Trigger optimistic update that will fail
  const mutation = useMutation({
    mutationFn: () => Promise.reject(new Error('API Error')),
    onMutate: async () => {
      // Optimistic update
      queryClient.setQueryData(['approval-rules', 'list'], (old) => 
        old.map(r => r.id === ruleId ? { ...r, is_active: !r.is_active } : r)
      )
      return { previous: initial }
    },
    onError: (err, variables, context) => {
      // Rollback
      queryClient.setQueryData(['approval-rules', 'list'], context.previous)
    }
  })
  
  await mutation.mutateAsync()
  
  // Verify rollback
  const final = queryClient.getQueryData(['approval-rules', 'list'])
  expect(final).toEqual(initial)
})
```

### Property 9: Database Index Usage
**Validates**: Requirements 2.2.2, 2.7.2

**Property**: All queries must use appropriate indexes and complete in < 100ms.

**Test Strategy**:
```rust
#[tokio::test]
async fn database_index_usage() {
    let start = Instant::now();
    
    // Query that should use idx_approval_rules_org_priority
    let rules = list_rules(org_id).await?;
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Query took {}ms", duration.as_millis());
    
    // Verify EXPLAIN shows index usage
    let explain = db.execute_raw("EXPLAIN SELECT * FROM approval_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority").await?;
    assert!(explain.contains("idx_approval_rules_org_priority"));
}
```

### Property 10: Rate Limiting
**Validates**: Requirements 2.2.7

**Property**: Requests exceeding rate limits must return 429 with Retry-After header.

**Test Strategy**:
```rust
#[tokio::test]
async fn rate_limiting() {
    let client = TestClient::new();
    
    // Make 101 requests (limit is 100/minute)
    for i in 0..=100 {
        let response = client.get("/approval-rules").await;
        
        if i < 100 {
            assert_eq!(response.status(), 200);
        } else {
            assert_eq!(response.status(), 429);
            assert!(response.headers().contains_key("Retry-After"));
        }
    }
}
```

## 7. Performance Considerations

### 7.1 Database Optimization
- **Indexes**: 4 indexes for common query patterns
- **Partial indexes**: Only index active rules
- **GIN index**: For array column (transaction_types)
- **Query planning**: Use EXPLAIN to verify index usage

### 7.2 Caching Strategy
- **React Query**: 5-minute cache for list and detail
- **Stale-while-revalidate**: Show cached data while fetching fresh
- **Optimistic updates**: Instant UI feedback
- **Cache invalidation**: On mutations only

### 7.3 Frontend Optimization
- **Code splitting**: Lazy load approval rules page
- **Skeleton loaders**: Better perceived performance
- **Debounced search**: Reduce API calls
- **Pagination**: Limit data transfer

## 8. Security Considerations

### 8.1 Authentication & Authorization
- **JWT tokens**: Required for all endpoints
- **Role-based access**: Admin/owner only for CRUD
- **RLS**: Database-level isolation by organization
- **Token validation**: On every request

### 8.2 Input Validation
- **XSS prevention**: Sanitize all string inputs
- **SQL injection**: Parameterized queries only
- **Amount validation**: Regex pattern enforcement
- **Length limits**: Prevent DoS via large inputs

### 8.3 Rate Limiting
- **Per-user**: 100 requests/minute
- **Per-organization**: 1000 requests/minute
- **Global**: 10000 requests/minute
- **Backoff**: Exponential backoff on 429

## 9. Testing Strategy

### 9.1 Unit Tests
- Validation logic (Zod schemas)
- Amount parsing and formatting
- Enum parsing
- String length checks

### 9.2 Integration Tests
- API endpoints (request/response)
- Database operations (CRUD)
- Cache invalidation
- Error handling

### 9.3 Property-Based Tests
- 10 properties defined above
- 100+ iterations per property
- Edge case discovery
- Regression prevention

### 9.4 E2E Tests
- Create approval rule flow
- Edit approval rule flow
- Delete approval rule flow
- Toggle active status
- Filter and sort
- Pagination navigation

## 10. Deployment Strategy

### 10.1 Phase 1: Backend Quick Wins (Week 1)
- Add missing transaction types
- Add validation (length, range)
- Add database indexes
- Deploy to staging

### 10.2 Phase 2: MVP (Week 2)
- Backend: Basic CRUD without pagination
- Frontend: Core UI with forms and table
- Deploy MVP to production
- Gather user feedback

### 10.3 Phase 3: Pagination (Week 3)
- Backend: Implement pagination (breaking change)
- Frontend: Add pagination controls
- API versioning (v1 → v2)
- Deploy v2 to production

### 10.4 Phase 4: Polish (Week 4)
- Add remaining UX features
- Complete E2E tests
- Performance optimization
- Final QA and deployment

## 11. Monitoring & Observability

### 11.1 Metrics
- API response times (p50, p95, p99)
- Error rates by endpoint
- Cache hit rates
- Database query times

### 11.2 Logging
- Structured logs (JSON format)
- Audit logs for all mutations
- Error logs with stack traces
- Performance logs for slow queries

### 11.3 Alerts
- Error rate > 1%
- Response time > 2 seconds
- Database query > 100ms
- Rate limit exceeded

## 12. Documentation

### 12.1 API Documentation
- OpenAPI spec with examples
- Error response catalog
- Authentication guide
- Rate limiting policy

### 12.2 User Documentation
- How to create approval rules
- Transaction type explanations
- Role permissions matrix
- Troubleshooting guide

### 12.3 Developer Documentation
- Architecture overview
- Database schema
- API integration guide
- Testing guide


## 13. Known Issues & Workarounds

### 13.1 utoipa Nullable Type Syntax (BUG-007)

**Issue**: The utoipa Rust library (used for OpenAPI spec generation) generates nullable fields using OpenAPI 3.1 syntax `type: [T, 'null']` instead of the more widely supported OpenAPI 3.0 syntax `nullable: true`.

**Impact**:
- Many OpenAPI tools and validators expect `nullable: true` format
- Type generators may fail or produce incorrect types
- API documentation may display incorrectly

**Affected Fields in Approval Rules**:
- `description` (optional string)
- `min_amount` (optional decimal)
- `max_amount` (optional decimal)

**Workaround** (Task 1.7):
The `contracts/split-openapi.py` script **automatically fixes** this issue using the `fix_nullable_syntax()` function:

1. Run: `cd contracts && python3 split-openapi.py`
2. Script automatically converts `type: [T, 'null']` to `type: T, nullable: true`
3. Verify the output in `contracts/openapi-split/12-approval-rules-schemas.yaml`

**No manual intervention required** - the script handles this automatically for all schemas.

**Script Logic**:
```python
def fix_nullable_syntax(obj):
    """
    Recursively fix utoipa's OpenAPI 3.1 nullable syntax to OpenAPI 3.0 compatible.
    Converts: type: [string, 'null'] -> type: string, nullable: true
    """
    if isinstance(obj, dict):
        if 'type' in obj and isinstance(obj['type'], list):
            type_list = obj['type']
            non_null_types = [t for t in type_list if t != 'null']
            has_null = 'null' in type_list
            
            if has_null and len(non_null_types) == 1:
                obj['type'] = non_null_types[0]
                obj['nullable'] = True
    # ... recursively processes all schemas
```

**Example Fix** (Automatic):
```yaml
# BEFORE (utoipa generates this)
description:
  type: [string, 'null']
  maxLength: 1000

# AFTER (split-openapi.py automatically converts)
description:
  type: string
  nullable: true
  maxLength: 1000
```

**Automation**: ✅ **Already automated** in `contracts/split-openapi.py` via the `fix_nullable_syntax()` function. No manual intervention needed.

**Related Bugs**:
- BUG-007: OpenAPI Nullable Type Syntax (simulation-attachments)
- BUG-008: Missing Simulation Attachment Feature

**Upstream Issue**: This is a known limitation of utoipa when targeting OpenAPI 3.0 compatibility. The library prioritizes OpenAPI 3.1 spec compliance.
