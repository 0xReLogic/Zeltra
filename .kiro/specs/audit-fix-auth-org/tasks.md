# Implementation Plan: Audit & Fix 01-auth-org-schemas

## Overview

Hands-on audit and fix for `01-auth-org-schemas.yaml` (18 schemas). Research backend/frontend, identify bugs, fix them, regenerate OpenAPI/types, verify, and generate report.

## Tasks

- [x] 1. Research backend implementations for all 18 schemas
  - Use grepSearch to find Rust structs in `backend/crates/` with `#[derive(ToSchema)]`
  - For each schema: LoginRequest, LoginResponse, RegisterRequest, VerifyEmailRequest, VerifyEmailResponse, ResendVerificationRequest, LogoutRequest, RefreshRequest, RefreshResponse, CreateOrganizationRequest, OrganizationResponse, UpdateOrganizationRequest, AddUserRequest, UpdateMemberRequest, UpdateUserRoleRequest, OrgUserResponse, MembershipResponse, TierLimitsResponse, UserInfo, UserOrganization
  - Read struct definitions with readFile
  - Document: file path, fields, types, Option<T> for optional fields
  - _Requirements: 1.1, 1.2_

- [x] 2. Compare backend structs with OpenAPI schemas
  - For each schema, compare Rust fields vs OpenAPI properties
  - Identify missing fields (in Rust but not OpenAPI, or vice versa)
  - Identify type mismatches (String vs Option<String>, i64 vs integer)
  - Identify missing structs (schema exists but no Rust struct)
  - Document all mismatches with severity (critical, high, medium, low)
  - _Requirements: 1.3, 1.4, 1.5, 1.6_

- [x] 3. Research frontend usage for all 18 schemas
  - Use grepSearch to find imports in `frontend/src/` from `@/types/api.generated`
  - Search for custom type definitions in `@/types/auth.ts` and `@/types/organizations.ts`
  - Identify which components use which schemas
  - Find `as any` type casts that bypass type safety
  - Document: imported (yes/no), custom type used, components using schema
  - _Requirements: 2.1, 2.2, 2.5_

- [x] 4. Compare frontend usage with OpenAPI schemas
  - For each schema, check if frontend expects fields not in OpenAPI
  - Identify custom types that should use api.generated
  - Identify unused schemas (defined but never imported)
  - Document all mismatches with severity
  - _Requirements: 2.3, 2.4_

- [x] 5. Research OpenAPI best practices
  - Use Tavily to search "OpenAPI 3.0 authentication schema best practices JWT"
  - Use Tavily to search "OpenAPI user management organization schema patterns"
  - Use Exa to find well-designed auth/org schema examples
  - Check if schemas have field descriptions
  - Check if schemas have examples for complex types
  - Check if sensitive fields (password, token) are properly annotated
  - Document recommendations with sources
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 6. Checkpoint - Review findings
  - Present all findings to user
  - Show critical issues (missing structs, type mismatches)
  - Show high priority issues (custom types, missing fields)
  - Show medium/low issues (missing descriptions, missing examples)
  - Ask user which issues to fix
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Fix critical backend issues
  - [x] 7.1 Add missing RefreshResponse struct
    - Create struct in `backend/crates/shared/src/auth.rs`
    - Add fields: `access_token: String`, `expires_in: i64`
    - Add `#[derive(Serialize, ToSchema)]`
    - Use strReplace to insert after RefreshRequest
    - _Requirements: 4.1, 4.5_
  
  - [x] 7.2 Add missing fields to UserInfo
    - Add `email: String` field
    - Add `organizations: Vec<UserOrganization>` field
    - Use strReplace to update struct
    - Preserve existing fields and comments
    - _Requirements: 4.2, 4.5_
  
  - [x] 7.3 Fix other missing structs/fields
    - Check if UpdateUserRoleRequest exists (should be UpdateMemberRequest)
    - Add any other missing fields identified in step 2
    - Use strReplace for each fix
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 8. Fix frontend type issues
  - [x] 8.1 Refactor custom types to use api.generated
    - Update `frontend/src/types/auth.ts` to import from api.generated
    - Update `frontend/src/types/organizations.ts` to import from api.generated
    - Remove custom type definitions that duplicate api.generated
    - Use strReplace for each file
    - _Requirements: 5.1, 5.4_
  
  - [x] 8.2 Remove `as any` type casts
    - Find all `as any` in transaction and approval components
    - Replace with proper types from api.generated
    - Use strReplace for each fix
    - _Requirements: 5.2, 5.4_
  
  - [x] 8.3 Update components to match OpenAPI
    - If frontend expects fields not in OpenAPI, update components
    - Use strReplace to fix component code
    - _Requirements: 5.3, 5.4_

- [x] 9. Regenerate OpenAPI specification
  - Run `cd backend && cargo run --bin generate-openapi`
  - Capture stdout/stderr for logging
  - Verify `contracts/openapi.yaml` is updated (check file modification time)
  - If regeneration fails, rollback backend changes and report error
  - _Requirements: 6.1, 6.4_

- [x] 10. Regenerate frontend types
  - Run `cd frontend && pnpm run generate:types`
  - Capture stdout/stderr for logging
  - Verify `frontend/src/types/api.generated.ts` is updated
  - If generation fails, rollback OpenAPI and backend changes, report error
  - _Requirements: 6.2, 6.4_

- [x] 11. Verify frontend builds successfully
  - Run `cd frontend && pnpm run build`
  - Capture build output
  - If build fails, rollback all changes and report error with build log
  - If build succeeds, log success
  - _Requirements: 6.3, 6.4_

- [x] 12. Re-audit to check for remaining issues
  - Re-run backend checker (step 1-2)
  - Re-run frontend checker (step 3-4)
  - Compare before/after issue counts
  - Verify critical issues are resolved
  - Document any remaining issues
  - _Requirements: 6.4_

- [x] 13. Generate comprehensive audit report
  - Create markdown report at `contracts/openapi-split/audits/01-auth-org-schemas-audit.md`
  - Include executive summary (total schemas, issues found, fixes applied, status)
  - For each of 18 schemas, include:
    - Status (✅ Valid, ⚠️ Warnings, ❌ Issues)
    - Backend implementation (file path, struct code)
    - Frontend usage (components, import source)
    - Issues found (with severity)
    - Fixes applied (with before/after code snippets)
  - Include "Fixes Applied" section with all patches
  - Include verification results (build passed, types generated)
  - Include remaining issues that need manual review
  - Include best practice recommendations
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

- [x] 14. Final review and cleanup
  - Review audit report for completeness
  - Verify all 18 schemas are documented
  - Verify all fixes are listed with code snippets
  - Verify verification results are included
  - Present report to user for review
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- This is a hands-on task executed directly by Kiro agent (no separate script)
- Use Sequential Thinking MCP for complex decisions (e.g., which fixes are safe)
- Use Tavily/Exa MCP for best practices research
- All fixes use strReplace to preserve code structure
- Rollback strategy: if any step fails, undo previous changes
- Focus on 18 schemas only: LoginRequest, LoginResponse, RegisterRequest, VerifyEmailRequest, VerifyEmailResponse, ResendVerificationRequest, LogoutRequest, RefreshRequest, RefreshResponse, CreateOrganizationRequest, OrganizationResponse, UpdateOrganizationRequest, AddUserRequest, UpdateMemberRequest, UpdateUserRoleRequest, OrgUserResponse, MembershipResponse, TierLimitsResponse, UserInfo, UserOrganization

