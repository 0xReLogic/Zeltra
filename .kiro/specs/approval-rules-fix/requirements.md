# Requirements: Approval Rules Management (BUG-013)

## 1. Overview

This specification addresses the complete implementation gap and critical issues in the Approval Rules feature across OpenAPI specification, backend implementation, and frontend UI. The feature allows organizations to define automated approval workflows based on transaction type, amount thresholds, and required approver roles.

**Status**: 🔴 CRITICAL - Frontend completely missing, backend has 17 issues, OpenAPI has 43 issues

**Audit Summary**:
- **OpenAPI**: 43 issues (6 critical, 9 high, 28 medium)
- **Backend**: 17 issues (5 critical, 7 high, 5 medium)
- **Frontend**: 20 issues (8 critical, 7 high, 5 medium)
- **Total**: 80 issues across all layers

## 2. Acceptance Criteria

### 2.1 OpenAPI Specification Requirements

**AC 2.1.1**: Timestamp fields MUST have `format: date-time` specification
- created_at and updated_at fields must specify ISO 8601 format
- Example values must be provided
- **Known Issue (BUG-007)**: utoipa generates nullable fields as `type: [T, 'null']` instead of `nullable: true`
- Manual fix required after OpenAPI spec generation

**AC 2.1.2**: List endpoint MUST support pagination
- Response structure: `{ data: [], meta: { page, per_page, total, total_pages } }`
- Query parameters: page, per_page, is_active, transaction_type, sort_by, sort_order
- Breaking change requires API versioning

**AC 2.1.3**: All error responses MUST reference ApiError schema
- 400, 401, 403, 404, 500 responses must have schema definitions
- Example error responses must be provided

**AC 2.1.4**: Amount fields MUST have pattern validation
- Pattern: `^[0-9]+(\.[0-9]{1,2})?$` for 2 decimal places
- Examples must match pattern (e.g., "1000.00")

**AC 2.1.5**: Enum constraints MUST be defined
- required_role: [viewer, submitter, approver, accountant, admin, owner]
- transaction_types items: [bill, invoice, journal, payment, expense, transfer, accrual, revaluation, intercompany]

**AC 2.1.6**: Validation constraints MUST be specified
- priority: minimum 1, maximum 100
- name: minLength 1, maxLength 255
- description: maxLength 1000
- transaction_types: minItems 1, maxItems 10

### 2.2 Backend Implementation Requirements

**AC 2.2.1**: Pagination MUST be implemented
- Repository method: `list_rules_paginated(org_id, offset, limit) -> (Vec<Rule>, u32)`
- Route handler must accept page and per_page query parameters
- Response must include pagination metadata

**AC 2.2.2**: Database indexes MUST be created
- Index on (organization_id, priority) for sorting
- GIN index on transaction_types for filtering
- Index on (organization_id, required_role)
- Index on (organization_id, min_amount, max_amount)

**AC 2.2.3**: Transaction type parsing MUST be complete
- Parser must handle all 12 types including accrual, revaluation, intercompany
- Error message must specify invalid type

**AC 2.2.4**: String length validation MUST be enforced
- name: max 255 characters
- description: max 1000 characters
- Return 400 error with specific message if exceeded

**AC 2.2.5**: Priority range validation MUST be enforced
- Valid range: 1-100
- Return 400 error if outside range

**AC 2.2.6**: Query parameters MUST support filtering and sorting
- Filter by: is_active, transaction_type, required_role
- Sort by: priority, created_at, name
- Sort order: asc, desc

**AC 2.2.7**: Rate limiting MUST be implemented
- Per-user: 100 requests/minute
- Per-organization: 1000 requests/minute
- Return 429 with Retry-After header

**AC 2.2.8**: Amount validation MUST use regex pattern
- Pattern: `^[0-9]+(\.[0-9]{1,2})?$`
- Maximum amount: 999,999,999
- Validate min <= max

**AC 2.2.9**: Audit logging MUST be implemented
- Log create, update, delete operations
- Include actor_id, organization_id, resource_id, changes
- Structured logging format

### 2.3 Frontend Implementation Requirements

**AC 2.3.1**: Management page MUST exist
- Location: `/dashboard/settings/approval-rules`
- Navigation link in settings sidebar
- Requires admin/owner role

**AC 2.3.2**: React Query hooks MUST be implemented
- useApprovalRules() - list with pagination
- useApprovalRule(id) - get single rule
- useCreateApprovalRule() - create mutation
- useUpdateApprovalRule() - update mutation
- useDeleteApprovalRule() - delete mutation

**AC 2.3.3**: Zod validation schema MUST be defined
- All field validations matching backend
- Cross-field validation (min <= max amount)
- Amount format validation with regex

**AC 2.3.4**: Form component MUST be created
- Fields: name, description, transaction_types, required_role, priority, min_amount, max_amount, is_active
- React Hook Form with zodResolver
- Real-time validation feedback

**AC 2.3.5**: Data table MUST display all rules
- Columns: Priority, Name, Transaction Types, Required Role, Amount Range, Status, Actions
- Sortable columns
- Filterable by status
- Pagination controls (when backend ready)

**AC 2.3.6**: CRUD dialogs MUST be implemented
- Create dialog with form
- Edit dialog with pre-filled form
- Delete confirmation dialog
- Success/error toast notifications

**AC 2.3.7**: Empty state MUST be displayed
- Icon and helpful message
- "Create First Rule" CTA button
- Link to documentation

**AC 2.3.8**: Error handling MUST be comprehensive
- API error toasts with specific messages
- Form validation error messages
- Error boundaries for component errors

**AC 2.3.9**: Loading states MUST be implemented
- Skeleton loaders for table
- Loading spinners during mutations
- Disabled buttons during operations

**AC 2.3.10**: Optimistic updates MUST be implemented
- Toggle active status
- Delete operation
- Rollback on error

### 2.4 Integration Requirements

**AC 2.4.1**: Type safety MUST be maintained
- Frontend uses generated TypeScript types
- Backend Rust types match OpenAPI schema
- No type mismatches

**AC 2.4.2**: API contract MUST be consistent
- Frontend requests match OpenAPI spec
- Backend responses match OpenAPI spec
- Error responses follow ApiError schema

**AC 2.4.3**: Pagination MUST be coordinated
- Backend implements pagination first
- Frontend implements pagination controls after
- Both use same page/per_page parameters

**AC 2.4.4**: Transaction types MUST be aligned
- Backend supports all 9 types
- Frontend multi-select shows all 9 types
- OpenAPI enum lists all 9 types

### 2.5 Testing Requirements

**AC 2.5.1**: E2E tests MUST cover all CRUD operations
- Create approval rule
- Edit approval rule
- Delete approval rule
- Toggle active status
- Filter by status
- Sort by priority

**AC 2.5.2**: Property-based tests MUST validate business logic
- Amount range validation (min <= max)
- Priority range validation (1-100)
- Transaction type matching
- Rule evaluation order

**AC 2.5.3**: Integration tests MUST verify API contract
- Request/response structure
- Error response format
- Pagination metadata
- Cache invalidation

**AC 2.5.4**: Unit tests MUST cover validation logic
- Zod schema validation
- Amount format validation
- Enum validation
- String length validation

### 2.6 Accessibility Requirements

**AC 2.6.1**: Keyboard navigation MUST be supported
- Tab through form fields
- Enter to submit
- Escape to close dialogs
- Arrow keys for table navigation

**AC 2.6.2**: Screen reader support MUST be implemented
- ARIA labels on all interactive elements
- Form field descriptions
- Error announcements
- Success announcements

**AC 2.6.3**: Focus management MUST be correct
- Focus trap in dialogs
- Focus return after dialog close
- Visible focus indicators

### 2.7 Performance Requirements

**AC 2.7.1**: Page load time MUST be < 2 seconds
- Initial data fetch
- Table rendering
- Form initialization

**AC 2.7.2**: Database queries MUST be < 100ms
- List rules query with indexes
- Get single rule query
- Filter/sort queries

**AC 2.7.3**: Caching MUST be implemented
- React Query cache for 5 minutes
- Invalidate on mutations
- Optimistic updates for instant feedback

## 3. Non-Functional Requirements

### 3.1 Security
- Admin/owner role required for all operations
- RLS (Row Level Security) enforced at database level
- Input sanitization to prevent XSS
- SQL injection prevention via parameterized queries

### 3.2 Scalability
- Pagination prevents memory issues with large datasets
- Database indexes ensure query performance
- Caching reduces database load

### 3.3 Maintainability
- Type-safe code (Rust + TypeScript)
- Comprehensive test coverage (>80%)
- Clear error messages for debugging
- Audit logs for troubleshooting

### 3.4 Usability
- Intuitive UI with clear labels
- Helpful error messages
- Success feedback for all actions
- Empty states with guidance

## 4. Out of Scope

- Bulk operations (create/update/delete multiple rules)
- Rule templates or presets
- Rule versioning or history
- Rule testing/simulation
- Advanced rule conditions (AND/OR logic)
- Rule scheduling (time-based activation)
- Notification system for rule matches
- Rule analytics or reporting

## 5. Dependencies

### 5.1 Backend Dependencies
- SeaORM for database operations
- Axum for HTTP routing
- Rust-decimal for amount handling
- Tower-governor for rate limiting

### 5.2 Frontend Dependencies
- React Query for API state management
- React Hook Form for form handling
- Zod for validation
- TanStack Table for data table
- Shadcn UI components

### 5.3 External Dependencies
- PostgreSQL database
- OpenAPI specification
- Authentication system

## 6. Success Metrics

- All 80 issues resolved
- E2E tests passing (100%)
- Page load time < 2 seconds
- Accessibility score > 90
- Zero type errors
- Zero runtime errors in production
- User can create/edit/delete rules successfully
- Approval workflow functions correctly

## 7. Risks and Mitigations

### Risk 1: Pagination Breaking Change
**Impact**: Existing API consumers will break
**Mitigation**: API versioning (v1 → v2), 6-month deprecation timeline

### Risk 2: Frontend-Backend Coordination
**Impact**: Frontend blocked until backend pagination ready
**Mitigation**: MVP without pagination first, add pagination later

### Risk 3: Performance with Large Datasets
**Impact**: Slow queries without indexes
**Mitigation**: Add indexes in Week 1 (quick win)

### Risk 4: Complex Form Validation
**Impact**: Poor UX if validation is confusing
**Mitigation**: Clear error messages, real-time feedback

## 8. Timeline

- **Week 1**: OpenAPI fixes + Backend quick wins + Frontend core
- **Week 2**: Backend pagination + Frontend UX features
- **Week 3**: Integration + Testing
- **Week 4**: QA + Deployment

**Total**: 4 weeks for complete implementation
**MVP**: 2 weeks for basic functionality (no pagination)
