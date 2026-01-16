# Design Document

## Overview

Simple, hands-on audit and fix for `01-auth-org-schemas.yaml` (18 schemas). Research backend Rust code, frontend TypeScript usage, and OpenAPI best practices. Identify bugs and mismatches. Fix them. Regenerate OpenAPI and types. Verify everything works. Generate report.

## Architecture

**Simple Linear Flow:**
```
1. Research Phase
   ├─ Check backend Rust structs (grepSearch, readFile)
   ├─ Check frontend TypeScript usage (grepSearch, readFile)
   └─ Research best practices (Tavily, Exa)

2. Analysis Phase
   ├─ Compare backend vs OpenAPI
   ├─ Compare frontend vs OpenAPI
   └─ Identify bugs and mismatches

3. Fix Phase
   ├─ Fix backend (strReplace to patch Rust files)
   ├─ Fix frontend (strReplace to patch TypeScript files)
   └─ Regenerate OpenAPI + types (executeBash)

4. Verification Phase
   ├─ Verify frontend builds (executeBash: pnpm run build)
   └─ Re-check for remaining issues

5. Report Phase
   └─ Generate audit report (fsWrite markdown)
```

## Components and Interfaces

### 1. Backend Checker

**Purpose:** Find Rust structs and compare with OpenAPI schemas

**Methods:**
- `findStruct(schemaName)`: Search for Rust struct in `backend/crates/`
- `parseFields(structCode)`: Extract fields, types, and annotations
- `compareWithOpenAPI(rustStruct, openAPISchema)`: Find mismatches

**Tools Used:**
- `grepSearch` to find structs with `#[derive(ToSchema)]`
- `readFile` to read struct definitions
- Manual parsing of Rust syntax

**Output:**
```typescript
{
  schemaName: "LoginRequest",
  found: true,
  filePath: "backend/crates/shared/src/auth.rs",
  fields: [
    { name: "email", rustType: "String", openAPIType: "string", match: true },
    { name: "password", rustType: "String", openAPIType: "string", match: true }
  ],
  issues: []
}
```

### 2. Frontend Checker

**Purpose:** Find TypeScript imports and compare with OpenAPI schemas

**Methods:**
- `findImports(schemaName)`: Search for imports in `frontend/src/`
- `analyzeUsage(schemaName)`: Find which components use the schema
- `detectCustomTypes()`: Find custom type definitions that bypass `api.generated`

**Tools Used:**
- `grepSearch` to find imports from `@/types/api.generated`
- `grepSearch` to find custom type definitions
- `readFile` to analyze component usage

**Output:**
```typescript
{
  schemaName: "LoginRequest",
  imported: true,
  importedFrom: "api.generated", // or "custom"
  usedInComponents: ["frontend/src/app/(auth)/login/page.tsx"],
  issues: ["Custom type used instead of api.generated"]
}
```

### 3. Best Practices Researcher

**Purpose:** Research OpenAPI best practices for auth/org schemas

**Methods:**
- `searchBestPractices()`: Use Tavily to search for best practices
- `findExamples()`: Use Exa to find well-designed schema examples
- `validateSchema(schema)`: Check if schema follows best practices

**Tools Used:**
- `mcp_tavily_tavily_search` for current best practices
- `mcp_exa_web_search_exa` for authoritative examples

**Output:**
```typescript
{
  recommendations: [
    {
      topic: "JWT token response",
      recommendation: "Include expires_in field",
      source: "https://...",
      appliesTo: ["LoginResponse", "RefreshResponse"]
    }
  ]
}
```

### 4. Code Fixer

**Purpose:** Patch backend and frontend code to fix issues

**Methods:**
- `fixBackend(issue)`: Patch Rust files using `strReplace`
- `fixFrontend(issue)`: Patch TypeScript files using `strReplace`
- `addMissingStruct(schemaName)`: Create new Rust struct

**Tools Used:**
- `strReplace` to modify code
- `fsWrite` to create new files if needed

**Fix Strategies:**
- **Missing struct:** Create new struct with `#[derive(Serialize, ToSchema)]`
- **Missing field:** Add field to existing struct
- **Type mismatch:** Update Rust type (e.g., `String` → `Option<String>`)
- **Custom type:** Replace import with `api.generated` import

### 5. Regenerator

**Purpose:** Regenerate OpenAPI and frontend types after fixes

**Methods:**
- `regenerateOpenAPI()`: Run `cargo run --bin generate-openapi`
- `regenerateFrontendTypes()`: Run `pnpm run generate:types`
- `verifyBuild()`: Run `pnpm run build` to check for errors

**Tools Used:**
- `executeBash` to run commands
- Capture stdout/stderr for logging

### 6. Report Generator

**Purpose:** Generate comprehensive audit report

**Methods:**
- `generateReport(findings, fixes)`: Create markdown report
- `formatSchemaSection(schema)`: Format individual schema analysis
- `formatFixSection(fix)`: Show before/after code snippets

**Tools Used:**
- `fsWrite` to save report as markdown

**Report Structure:**
```markdown
# Audit Report: 01-auth-org-schemas.yaml

## Executive Summary
- Total schemas: 18
- Issues found: X
- Fixes applied: Y
- Status: ✅ All synced / ⚠️ Warnings / ❌ Issues remain

## Schema Analysis

### LoginRequest
**Status:** ✅ Valid

**Backend:** `backend/crates/shared/src/auth.rs`
```rust
#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
```

**Frontend:** Used in `frontend/src/app/(auth)/login/page.tsx`
- Import: ✅ From api.generated
- Usage: Form data → API call

**Issues:** None

### RefreshResponse
**Status:** ❌ Critical Issues

**Backend:** ❌ Struct not found

**Frontend:** Used in `frontend/src/lib/api/client.ts`
- Import: ⚠️ Inline type, not imported
- Usage: API response

**Issues:**
- 🔴 Critical: Backend struct missing
- 🟠 High: Frontend expects but not in OpenAPI

**Fix Applied:**
```rust
// Added to backend/crates/shared/src/auth.rs
#[derive(Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}
```

## Fixes Applied

### Fix 1: Add RefreshResponse struct
- **File:** `backend/crates/shared/src/auth.rs`
- **Type:** Add missing struct
- **Before:** Struct did not exist
- **After:** [code snippet]

### Fix 2: Add email field to UserInfo
- **File:** `backend/crates/shared/src/auth.rs`
- **Type:** Add missing field
- **Before:** [code snippet]
- **After:** [code snippet]

## Verification

- ✅ OpenAPI regenerated successfully
- ✅ Frontend types regenerated successfully
- ✅ Frontend build passed
- ✅ No new issues introduced

## Remaining Issues

[List any issues that need manual review]

## Next Steps

1. Review fixes in git diff
2. Run backend tests: `cd backend && cargo test`
3. Test authentication flow manually
4. Commit changes if everything looks good
```

## Data Models

### Issue

```typescript
interface Issue {
  severity: 'critical' | 'high' | 'medium' | 'low';
  category: 'missing-struct' | 'missing-field' | 'type-mismatch' | 'custom-type' | 'unused-schema';
  schemaName: string;
  fieldName?: string;
  message: string;
  location: {
    backend?: string; // file path
    frontend?: string; // file path
    openapi?: string; // schema path
  };
}
```

### Fix

```typescript
interface Fix {
  issueId: string;
  type: 'add-struct' | 'add-field' | 'fix-type' | 'refactor-import';
  filePath: string;
  before: string; // code before fix
  after: string; // code after fix
  applied: boolean;
  error?: string; // if fix failed
}
```

## Correctness Properties

*Properties are formal statements about what the system should do, serving as the bridge between requirements and verification.*

### Property 1: Backend-OpenAPI Field Consistency
*For any* schema in OpenAPI, all fields in the corresponding Rust struct should be documented in OpenAPI, and all required OpenAPI fields should be non-Optional in Rust.
**Validates: Requirements 1.2, 1.3, 1.4, 1.5**

### Property 2: Frontend-OpenAPI Import Consistency
*For any* schema used in frontend, it should be imported from `api.generated` and not defined as a custom type.
**Validates: Requirements 2.2, 2.3**

### Property 3: Type Regeneration Idempotence
*For any* set of backend fixes, regenerating OpenAPI then types should produce consistent output on repeated runs.
**Validates: Requirements 6.1, 6.2**

### Property 4: Fix Preservation
*For any* code fix applied, existing code structure and comments should be preserved.
**Validates: Requirements 4.5, 5.4**

### Property 5: Verification Completeness
*For any* audit execution, the report should include findings for all 18 schemas.
**Validates: Requirements 7.1**

## Error Handling

### Backend Struct Not Found
- Log warning: "Schema X not found in backend"
- Mark as critical issue
- Attempt to create struct if safe
- If uncertain, flag for manual review

### Frontend Build Fails After Fix
- Rollback all changes using git
- Log error with build output
- Mark fix as failed in report
- Provide manual fix recommendation

### OpenAPI Regeneration Fails
- Check if `generate-openapi` binary exists
- Check for Rust compilation errors
- Rollback backend changes
- Report error with cargo output

### Type Generation Fails
- Check if `pnpm run generate:types` script exists
- Check for TypeScript errors in generated file
- Rollback OpenAPI changes
- Report error with pnpm output

## Testing Strategy

### Manual Testing Approach

Since this is a one-time audit + fix task, we'll use manual testing:

1. **Test Backend Checker:**
   - Run on known schemas (LoginRequest, UserResponse)
   - Verify it finds structs correctly
   - Verify it detects missing fields

2. **Test Frontend Checker:**
   - Run on known components (login page, org settings)
   - Verify it finds imports correctly
   - Verify it detects custom types

3. **Test Code Fixer:**
   - Apply fix to test file
   - Verify code is patched correctly
   - Verify original structure is preserved

4. **Test Regeneration:**
   - Run `cargo run --bin generate-openapi`
   - Verify `contracts/openapi.yaml` is updated
   - Run `pnpm run generate:types`
   - Verify `frontend/src/types/api.generated.ts` is updated

5. **Test End-to-End:**
   - Run full audit + fix on `01-auth-org-schemas.yaml`
   - Verify all 18 schemas are checked
   - Verify fixes are applied
   - Verify report is generated
   - Verify frontend builds successfully

### Verification Checklist

- [ ] All 18 schemas analyzed
- [ ] Backend structs found for all schemas
- [ ] Frontend usage documented for all schemas
- [ ] All critical issues fixed
- [ ] OpenAPI regenerated successfully
- [ ] Frontend types regenerated successfully
- [ ] Frontend build passes
- [ ] Audit report generated
- [ ] No regressions introduced

## Implementation Notes

### Technology Stack

- **Scripting:** Direct Kiro agent execution (no separate script needed)
- **Tools:** grepSearch, readFile, strReplace, executeBash, fsWrite
- **Research:** Tavily MCP, Exa MCP
- **Thinking:** Sequential Thinking MCP for complex decisions

### Execution Flow

1. **Start:** User asks to audit + fix `01-auth-org-schemas.yaml`
2. **Research:** Use tools to gather data about all 18 schemas
3. **Analyze:** Compare backend, OpenAPI, frontend
4. **Fix:** Apply patches using strReplace
5. **Regenerate:** Run cargo and pnpm commands
6. **Verify:** Check build and re-audit
7. **Report:** Generate markdown report
8. **Done:** Present report to user

### Known Limitations

- Cannot fix complex business logic issues
- Cannot handle breaking API changes automatically
- May miss dynamic imports in frontend
- Rust macro-generated code not analyzed

### Future Enhancements

- Extend to other schema files (02-transactions, 03-accounts, etc.)
- Add automated testing after fixes
- Generate PR with fixes automatically
- Track fixes over time for metrics

