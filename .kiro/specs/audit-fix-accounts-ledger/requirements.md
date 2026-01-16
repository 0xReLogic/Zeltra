# Requirements Document

## Introduction

This document specifies requirements for synchronizing the Accounts and Ledger domain between backend API implementation, OpenAPI specification, and frontend TypeScript types. The audit identified multiple mismatches that need to be resolved.

## Glossary

- **Backend**: Rust API server using Axum framework with utoipa for OpenAPI generation
- **OpenAPI**: The API specification in contracts/openapi.yaml
- **Frontend**: Next.js application with TypeScript and React Query
- **Generated_Types**: TypeScript types auto-generated from OpenAPI via openapi-typescript
- **Manual_Types**: Hand-written TypeScript interfaces in frontend/src/types/

## Requirements

### Requirement 1: Fix Backend OpenAPI Annotations

**User Story:** As a developer, I want the OpenAPI spec to accurately reflect the API behavior, so that generated types match actual responses.

#### Acceptance Criteria

1. WHEN list_accounts query parameters are defined THEN the Backend SHALL annotate them as `in: query` not `in: path`
2. WHEN list_accounts query parameters are defined THEN the Backend SHALL mark them as `required: false` since they are optional filters
3. WHEN list_accounts returns data THEN the Backend SHALL return a wrapper object `{ accounts: AccountResponse[] }` matching the actual implementation
4. THE Backend SHALL define a `GetAccountsResponse` schema with `accounts` array field

### Requirement 2: Fix OpenAPI Schema Accuracy

**User Story:** As a developer, I want OpenAPI schemas to match backend response structures, so that frontend types are correct.

#### Acceptance Criteria

1. THE OpenAPI SHALL include `GetAccountsResponse` schema with `accounts: AccountResponse[]` wrapper
2. WHEN list_accounts endpoint is documented THEN the OpenAPI SHALL reference `GetAccountsResponse` not raw array
3. THE OpenAPI SHALL correctly specify query parameters as optional with `in: query`

### Requirement 3: Migrate Frontend to Generated Types

**User Story:** As a developer, I want frontend to use generated types from OpenAPI, so that types stay in sync automatically.

#### Acceptance Criteria

1. THE Frontend SHALL use `AccountResponse` from generated types instead of manual `Account` interface
2. THE Frontend SHALL use `CreateAccountRequest` from generated types
3. THE Frontend SHALL use `UpdateAccountRequest` from generated types
4. THE Frontend SHALL use `AccountLedgerResponse` from generated types
5. THE Frontend SHALL use `LedgerEntryResponse` from generated types
6. WHEN manual types exist in types/accounts.ts THEN the Frontend SHALL remove or alias them to generated types

### Requirement 4: Fix Frontend CreateAccountRequest

**User Story:** As a developer, I want to create accounts with all optional fields, so that I can set is_active and allow_direct_posting.

#### Acceptance Criteria

1. THE Frontend CreateAccountRequest SHALL include optional `is_active` field
2. THE Frontend CreateAccountRequest SHALL include optional `allow_direct_posting` field
3. WHEN creating an account THEN the Frontend SHALL be able to pass these optional fields

### Requirement 5: Consolidate Ledger Types

**User Story:** As a developer, I want a single source of truth for ledger types, so that there's no confusion about which type to use.

#### Acceptance Criteria

1. THE Frontend SHALL remove duplicate `LedgerEntry` interface from accounts.ts
2. THE Frontend SHALL remove duplicate `GetLedgerResponse` interface from accounts.ts
3. THE Frontend SHALL use `AccountLedgerResponse` from generated types in all ledger queries
4. THE Frontend SHALL use `LedgerEntryResponse` from generated types

### Requirement 6: Update Frontend Queries

**User Story:** As a developer, I want queries to use correct types, so that TypeScript catches type errors.

#### Acceptance Criteria

1. THE useAccounts hook SHALL return `GetAccountsResponse` with accounts wrapper
2. THE useAccount hook SHALL return `AccountResponse` type
3. THE useAccountLedger hook SHALL return `AccountLedgerResponse` type
4. THE useCreateAccount hook SHALL accept `CreateAccountRequest` with all fields
5. THE useUpdateAccount hook SHALL accept `UpdateAccountRequest` type

### Requirement 7: E2E Verification

**User Story:** As a developer, I want to verify accounts functionality works end-to-end, so that I'm confident the sync is correct.

#### Acceptance Criteria

1. WHEN listing accounts THEN the System SHALL return accounts in wrapper object
2. WHEN creating an account THEN the System SHALL accept all optional fields
3. WHEN viewing account ledger THEN the System SHALL return full LedgerEntryResponse data
4. WHEN toggling account status THEN the System SHALL update is_active correctly
