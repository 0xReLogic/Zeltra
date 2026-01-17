# Implementation Plan: Common Schemas Fix (BUG-011)

## Overview

This plan implements fixes for 10 critical issues in the Common Schemas identified through comprehensive audits. The implementation is organized into three phases: OpenAPI specification updates, backend enhancements, and frontend improvements. Each task builds incrementally to ensure the system remains functional throughout the process.

## Tasks

- [x] 1. Fix OpenAPI specification violations
  - [x] 1.1 Add PaginationMeta schema to 11-common-schemas.yaml
    - Define schema with page (int64, 0-indexed), limit (int64), total (int64)
    - Mark all three fields as required
    - Add description: "Simple pagination metadata for transactions (0-indexed pages, no total_pages calculation)"
    - Add example values for each field
    - _Requirements: 2.1, 2.2_
  
  - [x] 1.2 Fix ApiError.details type definition
    - Change details field to type: object with additionalProperties: true
    - Add example showing retry_after and validation error structures
    - Update description to mention validation errors and retry information
    - _Requirements: 1.1, 8.4_
  
  - [x] 1.3 Enhance pagination schema descriptions
    - Update PageMeta description to indicate 1-indexed pages and standard REST usage
    - Update PaginationResponse description to indicate 0-indexed pages with total_pages
    - Update PaginationInfo description to indicate cursor-based pagination
    - Add indexing convention to page field descriptions (0-indexed or 1-indexed)
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 3.2_
  
  - [x] 1.4 Validate OpenAPI specification
    - Run openapi-generator validate on all spec files
    - Verify no errors or warnings
    - Verify all $ref references resolve correctly
    - _Requirements: 1.4_

- [x] 2. Update backend utoipa annotations
  - [x] 2.1 Add utoipa annotations to PaginationMeta in transactions.rs
    - Add #[schema(description = "...")] to struct
    - Add #[schema(example = ..., description = "...")] to each field
    - Ensure page field description mentions 0-indexed
    - _Requirements: 2.4_
  
  - [x] 2.2 Regenerate OpenAPI specification from backend
    - Run backend OpenAPI generation command
    - Run split-openapi.py to split and fix nullable syntax
    - Verify PaginationMeta appears in generated spec
    - Verify nullable fields use correct syntax (nullable: true)
    - _Requirements: 2.3, 2.4_
  
  - [x] 2.3 Write unit tests for PaginationMeta schema generation
    - Test that utoipa generates correct schema structure
    - Test that all fields have correct types and formats
    - Test that descriptions are present
    - _Requirements: 2.1, 2.2, 2.4_

- [x] 3. Checkpoint - Verify OpenAPI spec is valid and complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Update frontend type exports
  - [x] 4.1 Add missing pagination type exports to api-helpers.ts
    - Export PageMeta type
    - Export PageRequest type
    - Export PageResponse_ExchangeRateListItem type
    - Verify PaginationMeta export exists
    - Group all pagination exports together in the file
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  
  - [x] 4.2 Write unit tests for type exports
    - Test that all pagination types are importable from api-helpers
    - Test that types match generated schema definitions
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5. Fix frontend type mismatches
  - [x] 5.1 Fix dashboard activity query to use generated type
    - Remove custom ActivityResponse interface from dashboard.ts
    - Import RecentActivityResponse from api-helpers
    - Update useRecentActivity hook to use RecentActivityResponse type
    - Verify pagination field is accessible
    - _Requirements: 6.1, 6.2_
  
  - [x] 5.2 Write unit tests for dashboard activity type
    - Test that useRecentActivity returns correct type
    - Test that pagination field is present in response type
    - _Requirements: 6.1, 6.2_
  
  - [x] 5.3 Write property test for generated type usage
    - **Property 5: Generated Type Usage**
    - **Validates: Requirements 6.3, 6.4**
    - Test that all query hooks use generated types
    - Test that no custom interfaces duplicate API response types
    - Run 100+ iterations checking different query files

- [x] 6. Implement exchange rates pagination UI
  - [x] 6.1 Add page state management to exchange rates page
    - Add useState for page (1-indexed, default 1)
    - Add useState for perPage (default 20)
    - Update useExchangeRates hook to accept page and per_page parameters
    - Pass page and per_page to API query
    - _Requirements: 7.7, 7.8_
  
  - [x] 6.2 Update useExchangeRates query hook
    - Add params parameter with page and per_page fields
    - Include page and per_page in queryKey for proper caching
    - Pass parameters to API endpoint URL
    - _Requirements: 7.7, 7.8_
  
  - [x] 6.3 Implement pagination controls UI
    - Add pagination controls div after table
    - Display current page, total pages, and total count
    - Add Previous button with onClick handler
    - Add Next button with onClick handler
    - Disable Previous button when page === 1
    - Disable Next button when page >= total_pages
    - Only show controls when total_pages > 1
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_
  
  - [x] 6.4 Write E2E tests for exchange rates pagination
    - Test pagination controls are rendered when data has multiple pages
    - Test Previous button is disabled on first page
    - Test Next button is disabled on last page
    - Test clicking Next increments page
    - Test clicking Previous decrements page
    - Test page parameter is passed to API
    - Use Playwright MCP with credentials: corp@zeltra.io / qwertyui
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_

- [x] 7. E2E Testing with Playwright MCP
  - [x] 7.1 Test transactions pagination UI/UX
    - Login with corp@zeltra.io / qwertyui
    - Navigate to transactions page
    - Verify pagination controls display correctly
    - Test page navigation (next/prev buttons)
    - Test page size selector changes results
    - Verify pagination metadata displays correctly (current page, total)
    - Test edge cases (first page, last page, empty results)
    - Document any UI/UX issues or bugs found
    - _Requirements: 10.5_
  
  - [x] 7.2 Test exchange rates pagination UI/UX
    - Navigate to exchange rates page
    - Verify new pagination controls render correctly
    - Test Previous/Next button functionality
    - Test button disabled states (first/last page)
    - Verify page counter displays correctly
    - Test data loads correctly on page change
    - Document any UI/UX issues or bugs found
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 10.5_
  
  - [x] 7.3 Test reports pagination UI/UX
    - Navigate to reports section
    - Test trial balance pagination
    - Test balance sheet pagination
    - Test income statement pagination
    - Verify pagination controls work consistently across all reports
    - Document any UI/UX issues or bugs found
    - _Requirements: 10.5_
  
  - [x] 7.4 Test dashboard activity feed
    - Navigate to dashboard
    - Verify recent activity displays correctly
    - Test cursor-based pagination if applicable
    - Verify activity items render properly
    - Document any UI/UX issues or bugs found
    - _Requirements: 6.1, 6.2, 10.5_
  
  - [x] 7.5 Search Cognio for similar bugs before documenting
    - For each bug found during E2E testing:
      - Search Cognio project `zeltra-bug` for similar issues
      - Check if bug already documented
      - If new bug, document with clear reproduction steps
      - Include screenshots/error logs if applicable
      - Save to Cognio with proper tags and metadata

- [x] 8. Checkpoint - Verify frontend changes work correctly
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Write property tests for schema validation
  - [x] 8.1 Write property test for OpenAPI schema validation
    - **Property 1: OpenAPI Schema Validation**
    - **Validates: Requirements 1.3, 10.1, 10.2, 10.3, 10.4**
    - Test all paginated endpoints return responses matching OpenAPI schemas
    - Test error responses match ApiError schema
    - Run 100+ iterations with different page/limit combinations
  
  - [x] 8.2 Write property test for pagination format consistency
    - **Property 2: Pagination Format Consistency**
    - **Validates: Requirements 3.1**
    - Test all pagination schemas use consistent formats for similar fields
    - Test total fields use int64
    - Test page/limit fields use consistent formats within each schema
    - Run 100+ iterations checking all pagination schemas
  
  - [x] 8.3 Write property test for documentation completeness
    - **Property 3: Schema Documentation Completeness**
    - **Validates: Requirements 3.2, 4.1, 8.1, 8.3**
    - Test all pagination schema fields have non-empty descriptions
    - Test page fields document indexing convention (0 or 1-indexed)
    - Test schema-level descriptions explain use cases
    - Run 100+ iterations checking all schemas
  
  - [x] 8.4 Write property test for example value presence
    - **Property 4: Example Value Presence**
    - **Validates: Requirements 8.2**
    - Test all pagination schema fields have example values
    - Test examples are valid for the field type
    - Run 100+ iterations checking all fields
  
  - [x] 8.5 Write property test for backward compatibility
    - **Property 6: Backward Compatibility**
    - **Validates: Requirements 9.1, 9.2, 9.3**
    - Capture baseline responses before changes
    - Test responses match baseline structure after changes
    - Test field names and types unchanged
    - Test numeric values within format ranges
    - Run 100+ iterations with different endpoints

- [x] 9. Write integration tests
  - [x] 9.1 Write E2E test for transactions pagination
    - Test transactions page displays pagination correctly
    - Test pagination controls work
    - Test PaginationMeta structure in responses
    - _Requirements: 10.5_
  
  - [x] 9.2 Write E2E test for account ledger pagination
    - Test ledger page displays pagination correctly
    - Test PaginationResponse structure in responses
    - _Requirements: 10.5_
  
  - [x] 9.3 Write E2E test for dashboard activity cursor pagination
    - Test activity feed displays correctly
    - Test PaginationInfo structure in responses
    - Test has_more and next_cursor fields
    - _Requirements: 10.5_

- [x] 10. Bug Documentation & Final Verification
  - [x] 10.1 Document all bugs found to Cognio
    - Review all bugs found during implementation and E2E testing
    - For each bug:
      - Search Cognio project `zeltra-bug` first to check for duplicates
      - Document with clear title, description, and reproduction steps
      - Include relevant file paths and line numbers
      - Add screenshots or error logs if applicable
      - Tag appropriately (e.g., pagination, frontend, backend, openapi)
    - Save all new bugs to Cognio project `zeltra-bug`
  
  - [x] 10.2 Final checkpoint - Verify all changes work together
    - Ensure all tests pass, ask the user if questions arise
    - Verify OpenAPI spec is valid
    - Verify backend generates correct spec
    - Verify frontend types are correct
    - Verify all pagination UIs work
    - Verify backward compatibility maintained
    - Run full test suite (unit + integration + E2E)
  
  - [x] 10.3 Create summary report
    - Document all changes made
    - List all bugs found and fixed
    - List all bugs documented in Cognio
    - Verify all acceptance criteria met
    - Update documentation if needed

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties with 100+ iterations
- Unit tests validate specific examples and edge cases
- E2E tests validate end-to-end flows with Playwright
- The split-openapi.py script automatically fixes utoipa's nullable syntax bug
- All changes maintain backward compatibility - no breaking changes to existing endpoints
