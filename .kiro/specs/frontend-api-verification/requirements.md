# Requirements Document

## Introduction

This document specifies requirements for verifying and fixing frontend API integration with the real backend. The backend API is complete and documented via OpenAPI spec. The frontend has UI components built but many CRUD operations are unverified against the real API. This spec focuses on ensuring all frontend features work correctly with the real backend.

## Glossary

- **Frontend**: Next.js 16 application with React 19, TanStack Query, Zustand
- **Backend**: Rust Axum API server with PostgreSQL database
- **OpenAPI_Types**: Auto-generated TypeScript types from `contracts/openapi.yaml` in `frontend/src/types/api.generated.ts`
- **Org_Scoped_Endpoint**: API endpoint prefixed with `/organizations/{org_id}/`
- **CRUD**: Create, Read, Update, Delete operations
- **Workflow_Action**: Transaction state transitions (submit, approve, reject, post, void)

## Requirements

### Requirement 1: Use OpenAPI Generated Types

**User Story:** As a developer, I want to use auto-generated OpenAPI types, so that frontend types always match backend API contracts.

#### Acceptance Criteria

1. THE Frontend SHALL import types from `api.generated.ts` instead of manually defined types where available
2. WHEN a type mismatch exists between manual types and OpenAPI types, THE Frontend SHALL use the OpenAPI type
3. THE Frontend SHALL maintain backward compatibility with existing components during migration

### Requirement 2: Transaction CRUD Operations

**User Story:** As a user, I want to create, edit, and delete transactions, so that I can manage my financial records.

#### Acceptance Criteria

1. WHEN a user creates a transaction, THE System SHALL POST to `/organizations/{org_id}/transactions` with correct payload
2. WHEN a user edits a draft transaction, THE System SHALL PATCH to `/organizations/{org_id}/transactions/{id}`
3. WHEN a user deletes a draft transaction, THE System SHALL DELETE to `/organizations/{org_id}/transactions/{id}`
4. WHEN creating a transaction, THE System SHALL include entries array with account_id, debit, credit, and optional dimensions
5. IF the API returns an error, THEN THE System SHALL display the error message to the user

### Requirement 3: Transaction Workflow Actions

**User Story:** As a user, I want to submit, approve, reject, post, and void transactions, so that I can follow the approval workflow.

#### Acceptance Criteria

1. WHEN a user submits a draft transaction, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/submit`
2. WHEN an approver approves a pending transaction, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/approve`
3. WHEN an approver rejects a pending transaction, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/reject` with reason
4. WHEN a user posts an approved transaction, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/post`
5. WHEN a user voids a posted transaction, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/void` with reason
6. WHEN bulk approving transactions, THE System SHALL POST to `/organizations/{org_id}/transactions/bulk-approve`

### Requirement 4: Account CRUD Operations

**User Story:** As a user, I want to create, edit, and delete accounts, so that I can manage my chart of accounts.

#### Acceptance Criteria

1. WHEN a user creates an account, THE System SHALL POST to `/organizations/{org_id}/accounts` with correct payload
2. WHEN a user edits an account, THE System SHALL PUT to `/organizations/{org_id}/accounts/{id}`
3. WHEN a user deletes an account, THE System SHALL DELETE to `/organizations/{org_id}/accounts/{id}`
4. WHEN a user toggles account status, THE System SHALL PATCH to `/organizations/{org_id}/accounts/{id}/status`
5. THE Create_Account_Form SHALL include: code, name, account_type, account_subtype, parent_id (optional), currency

### Requirement 5: Budget CRUD Operations

**User Story:** As a user, I want to create budgets and budget lines, so that I can plan my expenses.

#### Acceptance Criteria

1. WHEN a user creates a budget, THE System SHALL POST to `/organizations/{org_id}/budgets`
2. WHEN a user adds budget lines, THE System SHALL POST to `/organizations/{org_id}/budgets/{id}/lines`
3. WHEN a user locks a budget, THE System SHALL POST to `/organizations/{org_id}/budgets/{id}/lock`
4. WHEN a user views budget vs actual, THE System SHALL GET from `/organizations/{org_id}/budgets/{id}/vs-actual`
5. THE Budget_Form SHALL include: name, fiscal_year_id, description

### Requirement 6: Dimension CRUD Operations

**User Story:** As a user, I want to create dimension types and values, so that I can categorize transactions.

#### Acceptance Criteria

1. WHEN a user creates a dimension type, THE System SHALL POST to `/organizations/{org_id}/dimension-types`
2. WHEN a user creates a dimension value, THE System SHALL POST to `/organizations/{org_id}/dimension-values`
3. WHEN a user updates a dimension value, THE System SHALL PATCH to `/organizations/{org_id}/dimension-values/{id}`
4. WHEN a user toggles dimension value status, THE System SHALL PATCH to `/organizations/{org_id}/dimension-values/{id}/status`
5. THE Dimension_Type_Form SHALL include: name, code, is_required

### Requirement 7: Fiscal Period Management

**User Story:** As a user, I want to create fiscal years and manage period status, so that I can control posting periods.

#### Acceptance Criteria

1. WHEN a user creates a fiscal year, THE System SHALL POST to `/organizations/{org_id}/fiscal-years`
2. WHEN a user changes period status, THE System SHALL PATCH to `/organizations/{org_id}/fiscal-periods/{id}/status`
3. THE Fiscal_Year_Form SHALL include: name, start_date, end_date, include_adjustment_period (boolean)
4. THE Period_Status_Change SHALL support: OPEN, SOFT_CLOSE, CLOSED

### Requirement 8: Exchange Rate Management

**User Story:** As a user, I want to manage exchange rates, so that I can handle multi-currency transactions.

#### Acceptance Criteria

1. WHEN a user adds an exchange rate, THE System SHALL POST to `/organizations/{org_id}/exchange-rates`
2. WHEN a user bulk imports rates, THE System SHALL POST to `/organizations/{org_id}/exchange-rates/bulk`
3. WHEN a user fetches live rates, THE System SHALL POST to `/organizations/{org_id}/exchange-rates/fetch`
4. THE Exchange_Rate_Form SHALL include: from_currency, to_currency, rate, effective_date

### Requirement 9: Simulation

**User Story:** As a user, I want to run financial simulations, so that I can forecast future scenarios.

#### Acceptance Criteria

1. WHEN a user runs a simulation, THE System SHALL POST to `/organizations/{org_id}/simulation/run`
2. THE Simulation_Request SHALL include: baseline_months, projection_months, adjustments array
3. THE System SHALL display simulation results with projected values

### Requirement 10: Approval Rules Management

**User Story:** As an admin, I want to manage approval rules, so that I can control transaction approval workflow.

#### Acceptance Criteria

1. WHEN a user lists approval rules, THE System SHALL GET from `/organizations/{org_id}/approval-rules`
2. WHEN a user creates an approval rule, THE System SHALL POST to `/organizations/{org_id}/approval-rules`
3. WHEN a user updates an approval rule, THE System SHALL PATCH to `/organizations/{org_id}/approval-rules/{id}`
4. WHEN a user deletes an approval rule, THE System SHALL DELETE to `/organizations/{org_id}/approval-rules/{id}`
5. THE Approval_Rule_Form SHALL include: name, min_amount, max_amount, required_role, is_active

### Requirement 11: Attachments

**User Story:** As a user, I want to upload and manage attachments on transactions, so that I can store supporting documents.

#### Acceptance Criteria

1. WHEN a user requests upload URL, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/attachments/upload`
2. WHEN a user confirms upload, THE System SHALL POST to `/organizations/{org_id}/transactions/{id}/attachments`
3. WHEN a user downloads attachment, THE System SHALL GET from `/organizations/{org_id}/attachments/{id}`
4. WHEN a user deletes attachment, THE System SHALL DELETE to `/organizations/{org_id}/attachments/{id}`
5. THE System SHALL support file types: PDF, PNG, JPG, JPEG

### Requirement 12: Account Ledger View

**User Story:** As a user, I want to view account ledger with running balance, so that I can see transaction history.

#### Acceptance Criteria

1. WHEN a user views account ledger, THE System SHALL GET from `/organizations/{org_id}/accounts/{id}/ledger`
2. THE Ledger_View SHALL display: date, description, debit, credit, running_balance
3. THE System SHALL support date range filtering via query params

### Requirement 13: Currency List

**User Story:** As a user, I want to see available currencies, so that I can select currencies in forms.

#### Acceptance Criteria

1. WHEN the app loads currency dropdowns, THE System SHALL GET from `/currencies`
2. THE Currency_List SHALL include: code, name, symbol, decimal_places

### Requirement 14: Error Handling

**User Story:** As a user, I want to see clear error messages, so that I can understand and fix issues.

#### Acceptance Criteria

1. WHEN the API returns a 400 error, THE System SHALL display the validation error message
2. WHEN the API returns a 403 error, THE System SHALL display "Permission denied"
3. WHEN the API returns a 404 error, THE System SHALL display "Resource not found"
4. WHEN the API returns a 500 error, THE System SHALL display "Server error, please try again"
5. THE System SHALL use toast notifications for error display
