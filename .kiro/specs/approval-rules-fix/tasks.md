# Implementation Plan: Approval Rules Management (BUG-013)

## Overview

This plan implements the complete Approval Rules management feature across OpenAPI specification, backend (Rust), and frontend (Next.js/React). The implementation is organized into 4 phases over 4 weeks, addressing 80 total issues (43 OpenAPI, 17 backend, 20 frontend).

**Testing Credentials**: corp@zeltra.io / qwertyui

## Tasks

- [ ] 1. Backend OpenAPI Annotations Update (Critical)
  - [x] 1.1 Update utoipa annotations for timestamp fields
    - File: `backend/crates/api/src/routes/approval_rules.rs`
    - Add `#[schema(value_type = String, format = "date-time", example = "2024-01-15T10:30:00Z")]` to created_at in ApprovalRuleResponse
    - Add `#[schema(value_type = String, format = "date-time", example = "2024-01-15T10:30:00Z")]` to updated_at in ApprovalRuleResponse
    - _Requirements: 2.1.1_
    - _Property: None_
  
  - [x] 1.2 Update utoipa annotations for amount fields
    - File: `backend/crates/api/src/routes/approval_rules.rs`
    - Add `#[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "1000.00")]` to min_amount in CreateApprovalRuleRequest
    - Add `#[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "5000.00")]` to max_amount in CreateApprovalRuleRequest
    - Add `#[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "1000.00")]` to min_amount in UpdateApprovalRuleRequest
    - Add `#[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "5000.00")]` to max_amount in UpdateApprovalRuleRequest
    - Add `#[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "1000.00")]` to min_amount in ApprovalRuleResponse
    - Add `#[schema(pattern = "^[0-9]+(\\.[0-9]{1,2})?$", example = "5000.00")]` to max_amount in ApprovalRuleResponse
    - Update field descriptions to mention format requirements
    - _Requirements: 2.1.4_
    - _Property: Property 2 (Amount Range Validation)_
  
  - [x] 1.3 Update utoipa annotations for enum constraints
    - File: `backend/crates/api/src/routes/approval_rules.rs`
    - Add `#[schema(inline, example = json!(["bill", "invoice"]))]` to transaction_types in all structs
    - Add inline enum documentation for required_role field
    - Update descriptions to list valid enum values
    - _Requirements: 2.1.5_
    - _Property: Property 4 (Transaction Type Completeness), Property 6 (Enum Validation)_
  
  - [x] 1.4 Update utoipa annotations for validation constraints
    - File: `backend/crates/api/src/routes/approval_rules.rs`
    - Add `#[schema(minimum = 1, maximum = 100, example = 1)]` to priority in all structs
    - Add `#[schema(min_length = 1, max_length = 255, example = "High Value Bills")]` to name in CreateApprovalRuleRequest
    - Add `#[schema(max_length = 1000)]` to description in all structs
    - Update field descriptions to mention valid ranges
    - _Requirements: 2.1.6_
    - _Property: Property 3 (Priority Range Enforcement), Property 5 (String Length Constraints)_
  
  - [x] 1.5 Add pagination support to list endpoint annotation
    - File: `backend/crates/api/src/routes/approval_rules.rs`
    - Update `#[utoipa::path]` annotation for list_approval_rules
    - Add query parameters: page, per_page, is_active, transaction_type, sort_by, sort_order
    - Change response from `body = [ApprovalRuleResponse]` to paginated response with data and meta
    - Add parameter descriptions and examples
    - Add 400, 401, 500 error responses with ApiError schema
    - _Requirements: 2.1.2_
    - _Property: Property 1 (Pagination Consistency)_
    - _Note: Backend implementation in Task 3, this is just OpenAPI annotation_
  
  - [x] 1.6 Add error response schemas to all endpoint annotations
    - File: `backend/crates/api/src/routes/approval_rules.rs`
    - Update all `#[utoipa::path]` annotations
    - Add 401 response: `(status = 401, description = "Unauthorized", body = ApiError)`
    - Add 500 response: `(status = 500, description = "Internal server error", body = ApiError)`
    - Update existing 400, 403, 404 responses to include `body = ApiError`
    - Add example error responses in descriptions
    - _Requirements: 2.1.3_
    - _Property: None_
  
  - [x] 1.7 Regenerate OpenAPI spec from backend
    - Run: `cargo run --bin generate-openapi` in backend/
    - This will generate `contracts/openapi.yaml` from utoipa annotations
    - Verify output shows "Successfully generated OpenAPI specification"
    - Verify `contracts/openapi.yaml` has been updated with new annotations
    - _Requirements: 2.1.1-2.1.6_
    - _Property: None_
  
  - [x] 1.8 Split and fix OpenAPI spec (automatic nullable fix)
    - Run: `python3 split-openapi.py` in contracts/
    - Script automatically:
      - Splits openapi.yaml into openapi-split/*.yaml files
      - Fixes utoipa nullable syntax: `type: [T, 'null']` → `type: T, nullable: true`
    - Verify `contracts/openapi-split/12-approval-rules-schemas.yaml` has correct nullable syntax
    - Verify `contracts/openapi-split/27-approval-rules-endpoints.yaml` has all updates
    - Common nullable fields: description, min_amount, max_amount
    - Verify no `type: [` patterns remain in approval rules schemas
    - _Requirements: 2.1.1_
    - _Property: None_
    - _Note: Script has built-in fix_nullable_syntax() function (BUG-007 workaround)_
  
  - [x] 1.9 Validate OpenAPI specification
    - Verify all changes are present in split files
    - Check timestamp formats are correct (format: date-time)
    - Check amount patterns are correct (pattern: ^[0-9]+(\\.[0-9]{1,2})?$)
    - Check enum constraints are correct (9 transaction types, 6 roles)
    - Check validation constraints are correct (priority 1-100, name 1-255, description max 1000)
    - Check pagination parameters are correct (page, per_page, filters, sorting)
    - Check error responses are correct (ApiError schema on all error responses)
    - Verify no `type: [` patterns remain (nullable syntax fixed)
    - _Requirements: 2.1.1-2.1.6_
    - _Property: None_

- [ ] 2. Backend Quick Wins (Non-Breaking)
  - [-] 2.1 Add missing transaction types to parser
    - Add "accrual" => Ok(TransactionType::Accrual) to parse_transaction_type
    - Add "revaluation" => Ok(TransactionType::Revaluation) to parse_transaction_type
    - Add "intercompany" => Ok(TransactionType::Intercompany) to parse_transaction_type
    - Update tests to verify all 12 types parse correctly
    - _Requirements: 2.2.3_
    - _Property: Property 4 (Transaction Type Completeness)_
  
  - [x] 2.2 Add string length validation
    - Add name length check (1-255) in create_approval_rule
    - Add name length check (1-255) in update_approval_rule
    - Add description length check (max 1000) in create_approval_rule
    - Add description length check (max 1000) in update_approval_rule
    - Return 400 error with specific message if exceeded
    - _Requirements: 2.2.4_
    - _Property: Property 5 (String Length Constraints)_
  
  - [x] 2.3 Add priority range validation
    - Add priority range check (1-100) in create_approval_rule
    - Add priority range check (1-100) in update_approval_rule
    - Return 400 error with specific message if outside range
    - _Requirements: 2.2.5_
    - _Property: Property 3 (Priority Range Enforcement)_
  
  - [x] 2.4 Add database indexes
    - Create index on (organization_id, priority) WHERE is_active = true
    - Create GIN index on transaction_types WHERE is_active = true
    - Create index on (organization_id, required_role) WHERE is_active = true
    - Create index on (organization_id, min_amount, max_amount) WHERE is_active = true
    - Run EXPLAIN to verify index usage
    - _Requirements: 2.2.2_
    - _Property: Property 9 (Database Index Usage)_
  
  - [ ] 2.5 Add input sanitization
    - Add ammonia::clean() for name field
    - Add ammonia::clean() for description field
    - Verify sanitized strings are not empty after cleaning
    - _Requirements: 3.1_
    - _Property: None_
  
  - [ ] 2.6 Add amount pattern validation
    - Add regex pattern validation for min_amount
    - Add regex pattern validation for max_amount
    - Add maximum amount check (999,999,999)
    - Verify min <= max validation
    - Return 400 error with specific message if invalid
    - _Requirements: 2.2.8_
    - _Property: Property 2 (Amount Range Validation)_

- [ ] 3. Backend Pagination Implementation (Breaking Change)
  - [ ] 3.1 Create pagination repository method
    - Add list_rules_paginated(org_id, offset, limit) method
    - Return (Vec<ApprovalRuleModel>, u32) with rules and total count
    - Use offset and limit in SQL query
    - Count total rules for organization
    - _Requirements: 2.2.1_
    - _Property: Property 1 (Pagination Consistency)_
  
  - [ ] 3.2 Add pagination parameters to route handler
    - Add PaginationParams struct with page and per_page fields
    - Add Query(params) extractor to list_approval_rules
    - Cap per_page at 100
    - Calculate offset from page and per_page
    - _Requirements: 2.2.1_
    - _Property: Property 1 (Pagination Consistency)_
  
  - [ ] 3.3 Update response structure
    - Change response to include data array and meta object
    - Calculate total_pages from total and per_page
    - Return JSON with data and meta fields
    - _Requirements: 2.2.1_
    - _Property: Property 1 (Pagination Consistency)_
  
  - [ ] 3.4 Add query parameters for filtering
    - Add is_active filter parameter
    - Add transaction_type filter parameter
    - Add required_role filter parameter
    - Apply filters in repository query
    - _Requirements: 2.2.6_
    - _Property: None_
  
  - [ ] 3.5 Add query parameters for sorting
    - Add sort_by parameter (priority, created_at, name)
    - Add sort_order parameter (asc, desc)
    - Apply sorting in repository query
    - Default to priority ascending
    - _Requirements: 2.2.6_
    - _Property: None_
  
  - [ ] 3.6 Write property test for pagination
    - **Property 1: Pagination Consistency**
    - **Validates: Requirements 2.1.2, 2.2.1**
    - Test that items count <= per_page
    - Test that last page has correct remaining items
    - Test that total_pages calculation is correct
    - Run 100+ iterations with different page/per_page values
    - _Requirements: 2.2.1_
    - _Property: Property 1 (Pagination Consistency)_

- [ ] 4. Backend Rate Limiting & Security
  - [ ] 4.1 Add rate limiting middleware
    - Add tower-governor dependency to Cargo.toml
    - Create rate limiting configuration (100 req/min per user)
    - Add GovernorLayer to router
    - Return 429 with Retry-After header when exceeded
    - _Requirements: 2.2.7_
    - _Property: Property 10 (Rate Limiting)_
  
  - [ ] 4.2 Add transaction wrapping
    - Wrap multi-step operations in database transactions
    - Add rollback on error
    - Test transaction rollback behavior
    - _Requirements: 3.1_
    - _Property: None_
  
  - [ ] 4.3 Add audit logging
    - Create audit_log module with structured logging
    - Log create operations with actor_id, org_id, resource_id, changes
    - Log update operations with actor_id, org_id, resource_id, changes
    - Log delete operations with actor_id, org_id, resource_id
    - Use structured JSON format
    - _Requirements: 2.2.9_
    - _Property: None_
  
  - [ ] 4.4 Write property test for rate limiting
    - **Property 10: Rate Limiting**
    - **Validates: Requirements 2.2.7**
    - Test that requests under limit return 200
    - Test that requests over limit return 429
    - Test that Retry-After header is present
    - Run test with 101 requests
    - _Requirements: 2.2.7_
    - _Property: Property 10 (Rate Limiting)_

- [ ] 5. Frontend Core Implementation
  - [ ] 5.1 Create React Query hooks
    - Create file: frontend/src/lib/queries/approval-rules.ts
    - Implement useApprovalRules() hook with pagination support
    - Implement useApprovalRule(id) hook
    - Implement useCreateApprovalRule() mutation
    - Implement useUpdateApprovalRule() mutation
    - Implement useDeleteApprovalRule() mutation
    - Define APPROVAL_RULE_KEYS for cache management
    - Add cache invalidation on mutations
    - _Requirements: 2.3.2_
    - _Property: Property 7 (Cache Invalidation)_
  
  - [ ] 5.2 Create Zod validation schema
    - Create file: frontend/src/lib/validations/approval-rule.ts
    - Define TRANSACTION_TYPES enum
    - Define ROLES enum
    - Create approvalRuleSchema with all field validations
    - Add cross-field validation (min <= max amount)
    - Add amount format validation with regex
    - Export ApprovalRuleFormValues type
    - _Requirements: 2.3.3_
    - _Property: Property 2, 3, 5, 6 (Validation Properties)_
  
  - [ ] 5.3 Create form component
    - Create file: frontend/src/components/approval-rules/ApprovalRuleForm.tsx
    - Add name input field with validation
    - Add description textarea with validation
    - Add transaction types multi-select
    - Add required role select
    - Add priority number input with validation
    - Add min/max amount currency inputs with validation
    - Add is_active toggle
    - Use React Hook Form with zodResolver
    - Display validation errors
    - _Requirements: 2.3.4_
    - _Property: Property 2, 3, 5, 6 (Validation Properties)_
  
  - [ ] 5.4 Create CRUD dialogs
    - Create CreateApprovalRuleDialog.tsx with form
    - Create EditApprovalRuleDialog.tsx with pre-filled form
    - Create DeleteApprovalRuleDialog.tsx with confirmation
    - Add success toast on create
    - Add success toast on update
    - Add success toast on delete
    - Add error toast on failures
    - _Requirements: 2.3.6_
    - _Property: None_
  
  - [ ] 5.5 Create main page
    - Create file: frontend/src/app/dashboard/settings/approval-rules/page.tsx
    - Add page header with title and create button
    - Add filters bar (status, transaction type, role)
    - Add data table with TanStack Table
    - Add empty state with icon and CTA
    - Add loading state with skeleton loaders
    - Add error state with retry button
    - _Requirements: 2.3.1, 2.3.5_
    - _Property: None_
  
  - [ ] 5.6 Create data table component
    - Create ApprovalRulesTable.tsx with TanStack Table
    - Add columns: Priority, Name, Transaction Types, Role, Amount Range, Status, Actions
    - Add sortable column headers
    - Add priority badge with color coding
    - Add transaction type badges (truncated)
    - Add role badge with color
    - Add amount range formatting
    - Add status toggle switch
    - Add edit and delete action buttons
    - _Requirements: 2.3.5_
    - _Property: None_

- [ ] 6. Frontend UX Enhancements
  - [ ] 6.1 Add optimistic updates
    - Implement optimistic update for toggle active status
    - Implement optimistic update for delete operation
    - Add rollback on error
    - Test rollback behavior
    - _Requirements: 2.3.10_
    - _Property: Property 8 (Optimistic Update Rollback)_
  
  - [ ] 6.2 Add pagination controls
    - Create PaginationControls component
    - Add Previous button (disabled on first page)
    - Add Next button (disabled on last page)
    - Add page info display (Page X of Y)
    - Add total count display
    - Sync with React Query pagination state
    - _Requirements: 2.3.5_
    - _Property: Property 1 (Pagination Consistency)_
  
  - [ ] 6.3 Add search and filter UI
    - Add search input for rule name
    - Add status filter dropdown (active/inactive/all)
    - Add transaction type filter dropdown
    - Add required role filter dropdown
    - Add clear filters button
    - Sync filters with React Query key
    - _Requirements: 2.3.5_
    - _Property: None_
  
  - [ ] 6.4 Add sorting UI
    - Add sortable table headers with icons
    - Add sort indicator (up/down arrow)
    - Sync sort state with React Query key
    - Default to priority ascending
    - _Requirements: 2.3.5_
    - _Property: None_
  
  - [ ] 6.5 Add navigation link
    - Update frontend/src/app/dashboard/settings/layout.tsx
    - Add "Approval Rules" link with Shield icon
    - Add to settings navigation menu
    - _Requirements: 2.3.1_
    - _Property: None_

- [ ] 7. Frontend Accessibility & Polish
  - [ ] 7.1 Add keyboard navigation
    - Add keyboard shortcuts (Ctrl+N for create, Escape for close)
    - Add tab navigation through form fields
    - Add Enter to submit forms
    - Add arrow keys for table navigation
    - _Requirements: 2.6.1_
    - _Property: None_
  
  - [ ] 7.2 Add ARIA labels
    - Add ARIA labels to all interactive elements
    - Add form field descriptions
    - Add error announcements
    - Add success announcements
    - _Requirements: 2.6.2_
    - _Property: None_
  
  - [ ] 7.3 Add focus management
    - Add focus trap in dialogs
    - Add focus return after dialog close
    - Add visible focus indicators
    - Test with keyboard only
    - _Requirements: 2.6.3_
    - _Property: None_
  
  - [ ] 7.4 Add mobile responsiveness
    - Add responsive table (card view on mobile)
    - Add touch-friendly buttons
    - Add mobile-optimized dialogs
    - Add responsive form layout
    - Test on mobile devices
    - _Requirements: 2.7.1_
    - _Property: None_

- [ ] 8. Property-Based Testing
  - [ ] 8.1 Write property test for amount range validation
    - **Property 2: Amount Range Validation**
    - **Validates: Requirements 2.1.4, 2.2.8, 2.3.3**
    - Test that min_amount <= max_amount is enforced
    - Test that invalid ranges are rejected
    - Test that valid ranges are accepted
    - Run 100+ iterations with random amounts
    - _Requirements: 2.1.4, 2.2.8, 2.3.3_
    - _Property: Property 2 (Amount Range Validation)_
  
  - [ ] 8.2 Write property test for priority range enforcement
    - **Property 3: Priority Range Enforcement**
    - **Validates: Requirements 2.1.6, 2.2.5, 2.3.3**
    - Test that priority 1-100 is accepted
    - Test that priority < 1 or > 100 is rejected
    - Run 100+ iterations with random priorities
    - _Requirements: 2.1.6, 2.2.5, 2.3.3_
    - _Property: Property 3 (Priority Range Enforcement)_
  
  - [ ] 8.3 Write property test for transaction type completeness
    - **Property 4: Transaction Type Completeness**
    - **Validates: Requirements 2.1.5, 2.2.3, 2.4.4**
    - Test that all 12 transaction types are parseable
    - Test that invalid types are rejected
    - Verify frontend, backend, and OpenAPI alignment
    - _Requirements: 2.1.5, 2.2.3, 2.4.4_
    - _Property: Property 4 (Transaction Type Completeness)_
  
  - [ ] 8.4 Write property test for string length constraints
    - **Property 5: String Length Constraints**
    - **Validates: Requirements 2.1.6, 2.2.4, 2.3.3**
    - Test that name 1-255 characters is accepted
    - Test that description <= 1000 characters is accepted
    - Test that exceeding limits is rejected
    - Run 100+ iterations with random strings
    - _Requirements: 2.1.6, 2.2.4, 2.3.3_
    - _Property: Property 5 (String Length Constraints)_
  
  - [ ] 8.5 Write property test for enum validation
    - **Property 6: Enum Validation**
    - **Validates: Requirements 2.1.5, 2.3.3**
    - Test that valid roles are accepted
    - Test that valid transaction types are accepted
    - Test that invalid values are rejected
    - Run 100+ iterations with random strings
    - _Requirements: 2.1.5, 2.3.3_
    - _Property: Property 6 (Enum Validation)_
  
  - [ ] 8.6 Write property test for cache invalidation
    - **Property 7: Cache Invalidation**
    - **Validates: Requirements 2.3.10, 2.7.3**
    - Test that create invalidates list cache
    - Test that update invalidates list and detail cache
    - Test that delete invalidates list cache
    - Test that subsequent queries return fresh data
    - _Requirements: 2.3.10, 2.7.3_
    - _Property: Property 7 (Cache Invalidation)_
  
  - [ ] 8.7 Write property test for optimistic update rollback
    - **Property 8: Optimistic Update Rollback**
    - **Validates: Requirements 2.3.10**
    - Test that failed optimistic update rolls back
    - Test that UI returns to previous state
    - Test with toggle active and delete operations
    - _Requirements: 2.3.10_
    - _Property: Property 8 (Optimistic Update Rollback)_
  
  - [ ] 8.8 Write property test for database index usage
    - **Property 9: Database Index Usage**
    - **Validates: Requirements 2.2.2, 2.7.2**
    - Test that queries complete in < 100ms
    - Test that EXPLAIN shows index usage
    - Test with different query patterns
    - _Requirements: 2.2.2, 2.7.2_
    - _Property: Property 9 (Database Index Usage)_

- [ ] 9. E2E Testing with Playwright MCP
  - [ ] 9.1 Test create approval rule flow
    - Login with corp@zeltra.io / qwertyui
    - Navigate to /dashboard/settings/approval-rules
    - Click "Create Rule" button
    - Fill in form fields (name, transaction types, role, priority, amounts)
    - Submit form
    - Verify success toast appears
    - Verify new rule appears in table
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: None_
  
  - [ ] 9.2 Test edit approval rule flow
    - Navigate to approval rules page
    - Click edit button on existing rule
    - Modify form fields
    - Submit form
    - Verify success toast appears
    - Verify changes reflected in table
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: None_
  
  - [ ] 9.3 Test delete approval rule flow
    - Navigate to approval rules page
    - Click delete button on existing rule
    - Confirm deletion in dialog
    - Verify success toast appears
    - Verify rule removed from table
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: None_
  
  - [ ] 9.4 Test toggle active status
    - Navigate to approval rules page
    - Click toggle switch on existing rule
    - Verify optimistic update (instant feedback)
    - Verify success toast appears
    - Verify status persisted after page refresh
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: Property 8 (Optimistic Update Rollback)_
  
  - [ ] 9.5 Test filter by status
    - Navigate to approval rules page
    - Select "Active" filter
    - Verify only active rules displayed
    - Select "Inactive" filter
    - Verify only inactive rules displayed
    - Select "All" filter
    - Verify all rules displayed
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: None_
  
  - [ ] 9.6 Test sort by priority
    - Navigate to approval rules page
    - Click priority column header
    - Verify rules sorted ascending
    - Click priority column header again
    - Verify rules sorted descending
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: None_
  
  - [ ] 9.7 Test pagination navigation
    - Navigate to approval rules page (with >20 rules)
    - Verify pagination controls displayed
    - Click "Next" button
    - Verify page 2 displayed
    - Click "Previous" button
    - Verify page 1 displayed
    - Verify page info displays correctly
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: Property 1 (Pagination Consistency)_
  
  - [ ] 9.8 Test form validation errors
    - Navigate to approval rules page
    - Click "Create Rule" button
    - Submit empty form
    - Verify validation errors displayed
    - Fill in invalid data (priority 101, invalid amount format)
    - Verify specific error messages displayed
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: Property 2, 3, 5, 6 (Validation Properties)_
  
  - [ ] 9.9 Test empty state
    - Create test organization with no approval rules
    - Navigate to approval rules page
    - Verify empty state displayed with icon and message
    - Verify "Create First Rule" button displayed
    - Click button and verify create dialog opens
    - Document any UI/UX issues found
    - _Requirements: 2.5.1_
    - _Property: None_
  
  - [ ] 9.10 Test accessibility with keyboard
    - Navigate to approval rules page
    - Tab through all interactive elements
    - Verify focus indicators visible
    - Press Enter on create button
    - Tab through form fields
    - Press Escape to close dialog
    - Verify focus returns to create button
    - Document any accessibility issues found
    - _Requirements: 2.5.1, 2.6.1, 2.6.2, 2.6.3_
    - _Property: None_

- [ ] 10. Bug Documentation & Final Verification
  - [ ] 10.1 Search Cognio for similar bugs
    - Search Cognio project `zeltra-bug` for approval rules issues
    - Search for pagination bugs
    - Search for validation bugs
    - Check if any bugs already documented
    - _Requirements: All_
    - _Property: None_
  
  - [ ] 10.2 Document all bugs found to Cognio
    - For each bug found during implementation and E2E testing:
      - Document with clear title and description
      - Include reproduction steps
      - Include relevant file paths and line numbers
      - Add screenshots or error logs if applicable
      - Tag appropriately (approval-rules, pagination, validation, etc.)
    - Save all new bugs to Cognio project `zeltra-bug`
    - _Requirements: All_
    - _Property: None_
  
  - [ ] 10.3 Final checkpoint - Verify all changes work together
    - Verify OpenAPI spec is valid
    - Verify backend generates correct spec
    - Verify frontend types are correct
    - Verify all CRUD operations work
    - Verify pagination works correctly
    - Verify validation works on all layers
    - Verify accessibility score > 90
    - Verify page load time < 2 seconds
    - Run full test suite (unit + integration + property + E2E)
    - _Requirements: All_
    - _Property: All Properties_
  
  - [ ] 10.4 Create summary report
    - Document all changes made (80 issues resolved)
    - List all bugs found and fixed
    - List all bugs documented in Cognio
    - Verify all acceptance criteria met
    - Update documentation if needed
    - Create deployment checklist
    - _Requirements: All_
    - _Property: None_

## Notes

- **Testing Credentials**: corp@zeltra.io / qwertyui
- **Backend URL**: http://10.0.0.5:8080
- **Frontend URL**: http://10.0.0.5:3000
- **Database**: PostgreSQL in Docker container `zeltra-postgres`
- **Property-Based Testing**: Use proptest for Rust, fast-check for TypeScript
- **E2E Testing**: Use Playwright MCP with provided credentials
- **Bug Tracking**: Save all bugs to Cognio project `zeltra-bug`
- **Breaking Change**: Pagination (Task 3) requires API versioning
- **Quick Wins**: Tasks 2.1-2.6 can be deployed immediately (5 hours, 0 breaking changes)
- **MVP Timeline**: Tasks 1-2, 5 (2 weeks, no pagination)
- **Complete Timeline**: All tasks (4 weeks)
- **Coordination**: Frontend pagination (Task 6.2) depends on backend pagination (Task 3)
- **Performance**: All database queries must complete in < 100ms (verified by Property 9)
- **Accessibility**: All interactive elements must have ARIA labels (Task 7.2)
- **Cache Strategy**: 5-minute cache, invalidate on mutations (Property 7)
- **Rate Limiting**: 100 req/min per user, 1000 req/min per org (Property 10)

## Success Criteria

- ✅ All 80 issues resolved (43 OpenAPI + 17 backend + 20 frontend)
- ✅ All 10 property-based tests passing (100+ iterations each)
- ✅ All E2E tests passing (10 test scenarios)
- ✅ Page load time < 2 seconds
- ✅ Database queries < 100ms
- ✅ Accessibility score > 90
- ✅ Zero type errors
- ✅ Zero runtime errors in production
- ✅ User can create/edit/delete rules successfully
- ✅ Approval workflow functions correctly
- ✅ All bugs documented in Cognio project `zeltra-bug`
