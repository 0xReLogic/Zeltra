# Requirements Document: Audit & Fix 05-Reports Schema

## Introduction

This document specifies the requirements for fixing mismatches between the OpenAPI specification (05-reports-schemas.yaml, 21-reports-endpoints.yaml), backend Rust implementation (reports.rs), and frontend TypeScript types/queries for the Reports module. The audit identified critical issues where query parameters are incorrectly annotated as path parameters, and frontend types don't match actual backend response structures.

## Glossary

- **OpenAPI_Spec**: The OpenAPI 3.0 specification files defining API contracts
- **Utoipa**: Rust library for generating OpenAPI documentation from code annotations
- **IntoParams**: Utoipa derive macro for query/path parameter structs
- **Query_Parameter**: HTTP parameter passed in URL query string (?key=value)
- **Path_Parameter**: HTTP parameter embedded in URL path (/resource/{id})
- **Response_Schema**: OpenAPI schema defining the structure of API responses
- **Frontend_Types**: TypeScript interfaces defining expected API response shapes

## Requirements

### Requirement 1: Fix Query Parameter Annotations

**User Story:** As a developer, I want the OpenAPI spec to correctly identify query parameters, so that generated client code constructs URLs properly.

#### Acceptance Criteria

1. WHEN the TrialBalanceQuery struct is used, THE Backend SHALL annotate it with `#[into_params(parameter_in = Query)]`
2. WHEN the BalanceSheetQuery struct is used, THE Backend SHALL annotate it with `#[into_params(parameter_in = Query)]`
3. WHEN the IncomeStatementQuery struct is used, THE Backend SHALL annotate it with `#[into_params(parameter_in = Query)]`
4. WHEN the DimensionalReportQuery struct is used, THE Backend SHALL annotate it with `#[into_params(parameter_in = Query)]`
5. WHEN the AccountLedgerQuery struct is used, THE Backend SHALL annotate it with `#[into_params(parameter_in = Query)]`
6. WHEN the OpenAPI spec is regenerated, THE System SHALL show all report query parameters with `in: query` instead of `in: path`

### Requirement 2: Fix Frontend Trial Balance Types

**User Story:** As a frontend developer, I want the TrialBalance types to match the actual backend response, so that data renders correctly.

#### Acceptance Criteria

1. WHEN the backend returns a TrialBalanceResponse, THE Response SHALL contain `accounts` array (not `data`)
2. WHEN the backend returns a TrialBalanceResponse, THE Response SHALL contain `totals.total_debit` and `totals.total_credit` (not flat `total_debit`/`total_credit`)
3. WHEN the backend returns a TrialBalanceResponse, THE Response SHALL contain `totals.is_balanced` boolean
4. WHEN the frontend queries trial balance, THE Query SHALL use the correct response type matching backend structure
5. WHEN the trial balance page renders, THE Page SHALL access `data.accounts` instead of `data.data`
6. WHEN the trial balance page renders totals, THE Page SHALL access `data.totals.total_debit` and `data.totals.total_credit`

### Requirement 3: Fix Frontend Balance Sheet Types

**User Story:** As a frontend developer, I want the BalanceSheet types to match the actual backend response, so that data renders correctly.

#### Acceptance Criteria

1. WHEN the backend returns a BalanceSheetResponse, THE Response SHALL contain section objects with `accounts` array and `total` string
2. WHEN the backend returns a BalanceSheetResponse, THE Response SHALL contain `total_liabilities_and_equity` (not separate `total_liabilities` and `total_equity`)
3. WHEN the backend returns a BalanceSheetResponse, THE Response SHALL contain `is_balanced` boolean
4. WHEN the frontend queries balance sheet, THE Query SHALL use the correct response type matching backend structure
5. WHEN the balance sheet page renders, THE Page SHALL access section data correctly from `assets.accounts`, `liabilities.accounts`, `equity.accounts`

### Requirement 4: Fix Frontend Income Statement Types

**User Story:** As a frontend developer, I want the IncomeStatement types to match the actual backend response, so that data renders correctly.

#### Acceptance Criteria

1. WHEN the backend returns an IncomeStatementResponse, THE Response SHALL contain `revenue`, `cost_of_goods_sold`, `operating_expenses`, `other_income_expenses` sections
2. WHEN the backend returns an IncomeStatementResponse, THE Response SHALL contain `gross_profit`, `operating_income`, `net_income` calculated values
3. WHEN the backend returns an IncomeStatementResponse, THE Response SHALL NOT contain simplified `revenues`/`expenses` arrays
4. WHEN the frontend queries income statement, THE Query SHALL use the correct response type matching backend structure
5. WHEN the income statement page renders, THE Page SHALL display all sections including COGS, gross profit, operating income

### Requirement 5: Regenerate OpenAPI and Frontend Types

**User Story:** As a developer, I want the OpenAPI spec and frontend types regenerated, so that all contracts are in sync.

#### Acceptance Criteria

1. WHEN backend annotations are fixed, THE System SHALL regenerate OpenAPI spec using `cargo run --bin generate-openapi`
2. WHEN OpenAPI spec is regenerated, THE System SHALL run `python contracts/split-openapi.py` to update split files
3. WHEN split files are updated, THE System SHALL regenerate frontend types using `pnpm openapi-typescript`
4. WHEN frontend types are regenerated, THE Frontend_Types SHALL match the corrected OpenAPI schemas

### Requirement 6: Update Frontend Queries and Pages

**User Story:** As a frontend developer, I want queries and pages updated to use correct types, so that reports display properly.

#### Acceptance Criteria

1. WHEN the frontend queries reports, THE Queries SHALL import types from generated api.generated.ts or api-helpers.ts
2. WHEN the frontend queries reports, THE Queries SHALL remove manually defined interfaces that duplicate generated types
3. WHEN report pages render data, THE Pages SHALL use correct property accessors matching backend response structure
4. WHEN report pages export CSV/PDF, THE Export_Functions SHALL use correct property accessors

### Requirement 7: E2E Testing and UI/UX Verification

**User Story:** As a QA engineer, I want to verify all report pages work correctly, so that users can view financial reports.

#### Acceptance Criteria

1. WHEN a user navigates to Trial Balance page, THE Page SHALL display account balances with correct totals
2. WHEN a user navigates to Balance Sheet page, THE Page SHALL display assets, liabilities, and equity sections
3. WHEN a user navigates to Income Statement page, THE Page SHALL display revenue and expense sections with net income
4. WHEN a user exports a report to CSV, THE Export SHALL contain correct data
5. WHEN a user exports a report to PDF, THE Export SHALL contain correct data
6. WHEN reports are balanced, THE UI SHALL show "Balanced" indicator
