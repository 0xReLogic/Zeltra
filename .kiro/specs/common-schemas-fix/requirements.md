# Requirements Document

## Introduction

This document specifies requirements for fixing critical issues in the Common Schemas (BUG-011) identified through comprehensive audits of the OpenAPI specification, backend implementation, and frontend integration. The system currently has 4 different pagination schemas with inconsistent usage, missing schema definitions, type specification violations, and incomplete frontend implementations.

## Glossary

- **OpenAPI_Spec**: The OpenAPI 3.0 specification files in `contracts/openapi-split/` that define API contracts
- **Backend**: The Rust-based API server in `backend/` that implements the API endpoints
- **Frontend**: The TypeScript/React application in `frontend/` that consumes the API
- **utoipa**: Rust library that generates OpenAPI specifications from Rust code annotations
- **Pagination_Schema**: A data structure defining how paginated API responses are formatted
- **Type_Format**: OpenAPI data type format specifiers (int32, int64, etc.)
- **API_Client**: Generated TypeScript client code that calls backend API endpoints

## Requirements

### Requirement 1: Fix OpenAPI Specification Violations

**User Story:** As an API consumer, I want the OpenAPI specification to be valid and complete, so that I can generate correct client code and understand the API contract.

#### Acceptance Criteria

1. WHEN the ApiError schema is defined, THE OpenAPI_Spec SHALL specify the `details` field type as `object` with `additionalProperties: true`
2. WHEN the PaginationMeta schema is referenced by an endpoint, THE OpenAPI_Spec SHALL include the PaginationMeta schema definition in the common schemas file
3. WHEN any schema is defined in the OpenAPI specification, THE OpenAPI_Spec SHALL include all required fields, types, and format specifiers
4. WHEN the OpenAPI specification is validated, THE OpenAPI_Spec SHALL pass validation without errors or warnings

### Requirement 2: Add Missing PaginationMeta Schema

**User Story:** As a developer, I want the PaginationMeta schema documented in the OpenAPI spec, so that the transactions endpoint contract is clear and client code can be generated correctly.

#### Acceptance Criteria

1. WHEN the PaginationMeta schema is added to the OpenAPI spec, THE OpenAPI_Spec SHALL define it with fields: `page` (int64, 0-indexed), `limit` (int64), and `total` (int64)
2. WHEN the PaginationMeta schema is defined, THE OpenAPI_Spec SHALL mark all three fields as required
3. WHEN the PaginatedTransactionsResponse schema references pagination metadata, THE OpenAPI_Spec SHALL reference the PaginationMeta schema using `$ref`
4. WHEN the OpenAPI spec is regenerated, THE Backend SHALL use utoipa annotations to generate the PaginationMeta schema definition

### Requirement 3: Standardize Pagination Type Formats

**User Story:** As an API maintainer, I want consistent type formats across pagination schemas, so that the API is predictable and easier to understand.

#### Acceptance Criteria

1. WHEN pagination schemas use numeric fields for counts or indices, THE OpenAPI_Spec SHALL use consistent format specifiers (int32 or int64) based on expected value ranges
2. WHEN a pagination schema field represents a page number or limit, THE OpenAPI_Spec SHALL document whether it is 0-indexed or 1-indexed
3. WHEN the PageMeta schema is defined, THE OpenAPI_Spec SHALL use int32 for `page`, `per_page`, and `total_pages` fields, and int64 for `total` field
4. WHEN the PaginationResponse schema is defined, THE OpenAPI_Spec SHALL use int64 for all numeric fields
5. WHEN the PaginationMeta schema is defined, THE OpenAPI_Spec SHALL use int64 for all numeric fields

### Requirement 4: Document Pagination Schema Usage

**User Story:** As a developer, I want clear documentation of when to use each pagination schema, so that I can implement new endpoints consistently.

#### Acceptance Criteria

1. WHEN multiple pagination schemas exist in the OpenAPI spec, THE OpenAPI_Spec SHALL include descriptions explaining the use case for each schema
2. WHEN the PageMeta schema is documented, THE OpenAPI_Spec SHALL indicate it is for standard offset-based pagination with 1-indexed pages
3. WHEN the PaginationResponse schema is documented, THE OpenAPI_Spec SHALL indicate it is for offset-based pagination with 0-indexed pages and total_pages calculation
4. WHEN the PaginationMeta schema is documented, THE OpenAPI_Spec SHALL indicate it is for simple offset-based pagination with 0-indexed pages without total_pages
5. WHEN the PaginationInfo schema is documented, THE OpenAPI_Spec SHALL indicate it is for cursor-based pagination with has_more flag

### Requirement 5: Fix Frontend Type Exports

**User Story:** As a frontend developer, I want all pagination types exported from the type helpers module, so that I can easily import and use them without navigating generated files.

#### Acceptance Criteria

1. WHEN pagination types are generated from the OpenAPI spec, THE Frontend SHALL export PageMeta type from the api-helpers module
2. WHEN pagination types are generated from the OpenAPI spec, THE Frontend SHALL export PageRequest type from the api-helpers module
3. WHEN pagination types are generated from the OpenAPI spec, THE Frontend SHALL export PageResponse_ExchangeRateListItem type from the api-helpers module
4. WHEN pagination types are generated from the OpenAPI spec, THE Frontend SHALL export PaginationMeta type from the api-helpers module
5. WHEN a developer imports pagination types, THE Frontend SHALL provide all types through a single import from api-helpers module

### Requirement 6: Fix Frontend Type Mismatches

**User Story:** As a frontend developer, I want API response types to match the OpenAPI specification, so that I have type safety and avoid runtime errors.

#### Acceptance Criteria

1. WHEN the dashboard activity query is defined, THE Frontend SHALL use the generated RecentActivityResponse type instead of a custom interface
2. WHEN the RecentActivityResponse type is used, THE Frontend SHALL include the pagination field of type PaginationInfo
3. WHEN API response types are defined in query hooks, THE Frontend SHALL use generated types from api.generated.ts or api-helpers.ts
4. WHEN custom interfaces are needed for API responses, THE Frontend SHALL extend generated types rather than redefining them

### Requirement 7: Implement Exchange Rates Pagination UI

**User Story:** As a user, I want to navigate through exchange rates using pagination controls, so that I can view large datasets efficiently without loading all data at once.

#### Acceptance Criteria

1. WHEN the exchange rates page displays data, THE Frontend SHALL render pagination controls showing current page and total pages
2. WHEN the exchange rates page displays data, THE Frontend SHALL show the total count of exchange rates
3. WHEN a user clicks the previous page button, THE Frontend SHALL load the previous page of exchange rates
4. WHEN a user clicks the next page button, THE Frontend SHALL load the next page of exchange rates
5. WHEN the user is on the first page, THE Frontend SHALL disable the previous page button
6. WHEN the user is on the last page, THE Frontend SHALL disable the next page button
7. WHEN the exchange rates query executes, THE Frontend SHALL pass the page parameter to the API endpoint
8. WHEN the exchange rates query executes, THE Frontend SHALL pass the per_page parameter to the API endpoint

### Requirement 8: Add Comprehensive Documentation

**User Story:** As a developer, I want clear documentation of pagination patterns and schema usage, so that I can implement features correctly and maintain consistency.

#### Acceptance Criteria

1. WHEN the OpenAPI specification defines pagination schemas, THE OpenAPI_Spec SHALL include description fields explaining each schema's purpose and usage
2. WHEN the OpenAPI specification defines pagination schemas, THE OpenAPI_Spec SHALL include example values for all fields
3. WHEN pagination schemas have indexing conventions, THE OpenAPI_Spec SHALL document whether pages are 0-indexed or 1-indexed in field descriptions
4. WHEN the ApiError.details field is defined, THE OpenAPI_Spec SHALL include an example showing the structure of error details

### Requirement 9: Maintain Backward Compatibility

**User Story:** As an API consumer, I want existing API endpoints to continue working after schema fixes, so that my integrations are not broken by the changes.

#### Acceptance Criteria

1. WHEN pagination schemas are updated in the OpenAPI spec, THE Backend SHALL continue returning the same response structure for existing endpoints
2. WHEN the ApiError schema is updated, THE Backend SHALL continue returning error responses in the same format
3. WHEN type formats are standardized, THE Backend SHALL ensure numeric field values remain within the specified format ranges
4. WHEN the PaginationMeta schema is added to the OpenAPI spec, THE Backend SHALL not change the transactions endpoint response structure

### Requirement 10: Validate Implementation Against Specification

**User Story:** As a quality assurance engineer, I want automated tests to verify the implementation matches the OpenAPI specification, so that schema drift is caught early.

#### Acceptance Criteria

1. WHEN the backend returns paginated responses, THE Backend SHALL return data structures matching the OpenAPI schema definitions
2. WHEN the backend returns error responses, THE Backend SHALL return ApiError structures matching the OpenAPI schema definition
3. WHEN the frontend makes API requests, THE Frontend SHALL send request parameters matching the OpenAPI schema definitions
4. WHEN the frontend receives API responses, THE Frontend SHALL handle response structures matching the OpenAPI schema definitions
5. WHEN end-to-end tests run, THE API_Client SHALL successfully parse all paginated responses using generated types
