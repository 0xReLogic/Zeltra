# Requirements Document

## Introduction

Audit and fix the 18 authentication and organization schemas in `01-auth-org-schemas.yaml` to ensure backend, OpenAPI, and frontend are synchronized. Identify bugs, mismatches, and missing implementations, then fix them to achieve full BE ↔ OpenAPI ↔ FE consistency.

## Glossary

- **Auth_Org_Schemas**: The 18 schemas in `contracts/openapi-split/01-auth-org-schemas.yaml` for authentication and organization management
- **Backend**: Rust code in `backend/crates/` that implements the API
- **Frontend**: TypeScript/React code in `frontend/src/` that consumes the API
- **OpenAPI**: The single source of truth for API contracts (`contracts/openapi.yaml`)
- **Sync**: Backend structs match OpenAPI schemas, and frontend types match OpenAPI schemas

## Requirements

### Requirement 1: Identify Backend-OpenAPI Mismatches

**User Story:** As a developer, I want to find all mismatches between backend Rust structs and OpenAPI schemas, so that I can fix them.

#### Acceptance Criteria

1. WHEN checking a schema, THE Audit SHALL find the corresponding Rust struct in `backend/crates/`
2. WHEN a struct is found, THE Audit SHALL compare all fields between Rust and OpenAPI
3. WHEN a field exists in Rust but not in OpenAPI, THE Audit SHALL flag it as "Missing in OpenAPI"
4. WHEN a field exists in OpenAPI but not in Rust, THE Audit SHALL flag it as "Missing in Backend"
5. WHEN field types don't match, THE Audit SHALL flag it as "Type Mismatch"
6. WHEN a struct is missing entirely, THE Audit SHALL flag it as "Backend Not Implemented"

### Requirement 2: Identify Frontend-OpenAPI Mismatches

**User Story:** As a developer, I want to find all mismatches between frontend TypeScript types and OpenAPI schemas, so that I can fix them.

#### Acceptance Criteria

1. WHEN checking a schema, THE Audit SHALL search for imports in `frontend/src/`
2. WHEN frontend uses custom types instead of `api.generated`, THE Audit SHALL flag it as "Custom Type Used"
3. WHEN frontend expects fields not in OpenAPI, THE Audit SHALL flag it as "Frontend Expects Missing Field"
4. WHEN frontend doesn't use a schema, THE Audit SHALL flag it as "Unused Schema"
5. WHEN frontend uses `as any` casts, THE Audit SHALL flag it as "Type Safety Bypass"

### Requirement 3: Research Best Practices

**User Story:** As a developer, I want to know if schemas follow OpenAPI best practices, so that I can improve API quality.

#### Acceptance Criteria

1. THE Audit SHALL use Tavily to search for "OpenAPI authentication schema best practices"
2. THE Audit SHALL use Exa to find examples of well-designed auth/org schemas
3. THE Audit SHALL check if schemas have descriptions for all fields
4. THE Audit SHALL check if schemas have examples for complex types
5. THE Audit SHALL check if sensitive fields (password, token) are properly annotated

### Requirement 4: Fix Backend Issues

**User Story:** As a developer, I want to fix backend bugs and missing implementations, so that backend matches OpenAPI.

#### Acceptance Criteria

1. WHEN a struct is missing, THE Fix SHALL create it with proper `#[derive(Serialize, ToSchema)]`
2. WHEN a field is missing in Rust, THE Fix SHALL add it to the struct
3. WHEN a field type is wrong, THE Fix SHALL update the Rust type
4. WHEN a field should be optional, THE Fix SHALL wrap it in `Option<T>`
5. THE Fix SHALL preserve existing code structure and comments

### Requirement 5: Fix Frontend Issues

**User Story:** As a developer, I want to fix frontend type mismatches, so that frontend uses generated types correctly.

#### Acceptance Criteria

1. WHEN frontend uses custom types, THE Fix SHALL refactor to use `api.generated` imports
2. WHEN frontend has `as any` casts, THE Fix SHALL remove them and use proper types
3. WHEN frontend expects missing fields, THE Fix SHALL update components to match OpenAPI
4. THE Fix SHALL preserve existing component logic and UI

### Requirement 6: Regenerate and Verify

**User Story:** As a developer, I want to regenerate OpenAPI and types after fixes, so that changes propagate correctly.

#### Acceptance Criteria

1. WHEN backend is fixed, THE System SHALL run `cargo run --bin generate-openapi`
2. WHEN OpenAPI is regenerated, THE System SHALL run `pnpm run generate:types`
3. WHEN types are regenerated, THE System SHALL verify frontend builds successfully
4. WHEN verification fails, THE System SHALL report the error and rollback changes

### Requirement 7: Generate Audit Report

**User Story:** As a developer, I want a clear report showing what was found and what was fixed, so that I can review the changes.

#### Acceptance Criteria

1. THE Report SHALL list all 18 schemas with their status (✅ Valid, ⚠️ Warnings, ❌ Issues)
2. THE Report SHALL show backend implementation details (file path, struct definition)
3. THE Report SHALL show frontend usage (which components use which schemas)
4. THE Report SHALL list all issues found with severity (Critical, High, Medium, Low)
5. THE Report SHALL list all fixes applied with before/after code snippets
6. THE Report SHALL include verification results (build passed, types generated)
7. THE Report SHALL be saved to `contracts/openapi-split/audits/01-auth-org-schemas-audit.md`

