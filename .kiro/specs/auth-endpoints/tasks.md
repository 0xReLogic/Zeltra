# Auth Endpoints Bug Fixes - Tasks

**Feature**: Fix all authentication system bugs
**Total Tasks**: 8 major tasks + future enhancements

---

## Task 1: Backend P0 - JWT Secret Validation ✅

**Priority**: P0 (Critical)
**Estimated Time**: 30 minutes
**Status**: ✅ COMPLETED

### Subtasks:
- [x] 1.1 Research with Exa: "Rust JWT secret validation best practices 2026"
- [x] 1.2 Research with Exa: "Rust panic on startup security checks 2026"
- [x] 1.3 Update `backend/crates/shared/src/jwt.rs` - Add panic if secret is default
- [x] 1.4 Add clear error message with instructions
- [x] 1.5 Test with default secret (should panic)
- [x] 1.6 Test with custom secret (should work)
- [x] 1.7 Run `cargo fmt`
- [x] 1.8 Run `cargo clippy` - Fix all warnings
- [x] 1.9 Run backend tests: `cargo test`
- [x] 1.10 If any errors, research with Exa: "[error message] Rust 2026"

**Acceptance Criteria**:
- ✅ Application panics on startup if JWT secret is "change-me-in-production"
- ✅ Error message explains how to fix
- ✅ All tests pass

**Completion Notes**:
- Added assert! check in JwtService::new() to panic if secret is default
- Added clear error message with fix instructions
- Added tests: test_panic_on_default_secret and test_custom_secret_works
- Fixed test helpers in auth.rs, exchange_rates.rs, and attachments.rs to use custom JWT secret
- All 5 JWT tests passing, all 95 API tests passing
- Committed (da14d17) and saved to Cognio project zeltra-bug

---

## Task 2: Backend P1 - Validation & Schema Fixes ✅

**Priority**: P1 (High)
**Estimated Time**: 3 hours
**Status**: ✅ COMPLETED (Already implemented in codebase)

### Subtasks:
- [x] 2.1 Research with Exa: "Rust validator crate best practices 2026"
- [x] 2.2 Research with Exa: "utoipa nullable annotations 2026"
- [x] 2.3 Research with Exa: "Rust nested Option handling patterns 2026"
- [x] 2.4 Add `validator` crate to `backend/crates/shared/Cargo.toml`
- [x] 2.5 Update `UpdateOrganizationRequest` - Add `#[schema(nullable = true)]` to all Option fields
- [x] 2.6 Create `OptionalUpdate<T>` enum for nested Option handling
- [x] 2.7 Update `UpdateMemberRequest` - Use `OptionalUpdate<String>` for approval_limit
- [x] 2.8 Add `#[derive(Validate)]` to `LoginRequest` and `RegisterRequest`
- [x] 2.9 Add `#[validate(email)]` to email fields
- [x] 2.10 Add `#[validate(length(min = 8, max = 128))]` to password fields
- [x] 2.11 Add `#[validate(length(min = 2, max = 255))]` to full_name field
- [x] 2.12 Update login handler - Call `payload.validate()`
- [x] 2.13 Update register handler - Call `payload.validate()`
- [x] 2.14 Add `#[schema(example = "...")]` to RegisterRequest fields
- [x] 2.15 Add `#[schema(default = "UTC", example = "...")]` to CreateOrganizationRequest.timezone
- [x] 2.16 Run `cargo fmt`
- [x] 2.17 Run `cargo clippy` - Fix all warnings
- [x] 2.18 Run backend tests: `cargo test`
- [x] 2.19 Test validation with invalid email (should fail)
- [x] 2.20 Test validation with short password (should fail)
- [x] 2.21 If any errors, research with Exa: "[error message] Rust validator 2026"

**Acceptance Criteria**:
- ✅ All Option<T> fields have nullable annotations
- ✅ UpdateMemberRequest uses OptionalUpdate enum
- ✅ Email and password validation works
- ✅ Examples added to all request schemas
- ✅ All tests pass

**Completion Notes**:
- Verified all items were ALREADY IMPLEMENTED in codebase:
  - validator crate already in Cargo.toml
  - LoginRequest and RegisterRequest already have #[derive(Validate)] with email and password validation
  - Handlers already call .validate()
  - UpdateOrganizationRequest already has #[schema(nullable = true)] on Option fields
  - OptionalUpdate enum already exists for nested Option handling
  - UpdateMemberRequest already uses OptionalUpdate
  - Examples already added to schemas
  - Timezone default already documented
- All 19 shared tests passing, all 95 API tests passing
- Saved to Cognio project zeltra-bug

---

## Task 3: Backend P1 - Security Fixes ✅

**Priority**: P1 (High)
**Estimated Time**: 2 hours
**Status**: ✅ COMPLETED

### Subtasks:
- [x] 3.1 Research with Exa: "Rust UUID validation security best practices 2026"
- [x] 3.2 Research with Exa: "Rust per-user rate limiting patterns 2026"
- [x] 3.3 Research with Exa: "Rust HashMap rate limiter implementation 2026"
- [x] 3.4 Update `backend/crates/db/src/rls.rs` - Add UUID validation in set_rls_context
- [x] 3.5 Add error handling for nil UUID
- [x] 3.6 Create `backend/crates/api/src/middleware/keyed_rate_limit.rs`
- [x] 3.7 Implement `KeyedRateLimiter` struct with HashMap of limiters
- [x] 3.8 Implement `extract_rate_limit_key()` function (user_id or IP)
- [x] 3.9 Update rate limit middleware to use keyed limiting
- [x] 3.10 Add configuration for per-user limits
- [x] 3.11 Run `cargo fmt`
- [x] 3.12 Run `cargo clippy` - Fix all warnings
- [x] 3.13 Run backend tests: `cargo test`
- [x] 3.14 Test rate limiting with multiple users
- [x] 3.15 Test rate limiting with same user (should be limited)
- [x] 3.16 If any errors, research with Exa: "[error message] Rust rate limiting 2026"

**Acceptance Criteria**:
- ✅ RLS context validates UUID
- ✅ Per-user rate limiting implemented
- ✅ Rate limits configurable
- ✅ All tests pass

**Completion Notes**:
- RLS UUID validation was already implemented in rls.rs with nil UUID checks
- Created keyed_rate_limit.rs with KeyedRateLimiter using governor's DashMap
- Implemented extract_rate_limit_key() with priority: user_id > X-Forwarded-For > ConnectInfo > anonymous
- Added KeyedRateLimitConfig with default 10 req/s, burst 20
- Added dashmap dependency and governor dashmap feature
- All 7 keyed rate limit tests passing
- All 813 backend tests passing (102 API + 394 core + 247 db + 70 shared)
- cargo fmt and cargo clippy passed
- Saved to Cognio project zeltra-bug

---

## Task 4: OpenAPI - Fix Nullable Syntax ✅

**Priority**: P1 (High)
**Estimated Time**: 1 hour
**Status**: ✅ COMPLETED

### Subtasks:
- [x] 4.1 Research with Exa: "OpenAPI 3.0 nullable oneOf pattern best practices 2026"
- [x] 4.2 Research with Exa: "Python YAML manipulation patterns 2026"
- [x] 4.3 Update `contracts/split-openapi.py` - Add oneOf pattern handling to fix_nullable_syntax()
- [x] 4.4 Handle `oneOf: [type: 'null', $ref]` pattern
- [x] 4.5 Convert to `allOf` with `nullable: true`
- [x] 4.6 Run Python script: `python contracts/split-openapi.py`
- [x] 4.7 Verify `OrganizationResponse.limits` uses correct syntax
- [x] 4.8 Verify `DashboardMetricsResponse.period` uses correct syntax
- [x] 4.9 Check all split YAML files for oneOf patterns
- [x] 4.10 Commit regenerated OpenAPI files
- [x] 4.11 If any errors, research with Exa: "[error message] Python YAML 2026"

**Acceptance Criteria**:
- ✅ Python script handles oneOf patterns
- ✅ All nullable $ref fields use allOf + nullable
- ✅ No oneOf patterns with type: 'null' remain
- ✅ OpenAPI files regenerated

**Completion Notes**:
- Updated fix_nullable_syntax() to detect and fix oneOf patterns with type: 'null'
- Fixed 2 instances: OrganizationResponse.limits and DashboardMetricsResponse.period
- Both now use correct OpenAPI 3.0 syntax: allOf with nullable: true
- Regenerated all 28 split YAML files (12 schemas + 16 endpoints)
- Verified no more type: 'null' patterns exist in any file
- All schema references still resolve correctly
- Saved to Cognio project zeltra-bug

---

## Task 5: Frontend P1 - Validation & UX Fixes ✅

**Priority**: P1 (High)
**Estimated Time**: 2 hours
**Status**: ✅ COMPLETED

### Subtasks:
- [x] 5.1 Research with Exa: "Next.js 16 authentication redirect patterns 2026"
- [x] 5.2 Research with Exa: "React password validation best practices 2026"
- [x] 5.3 Research with Exa: "React Query proactive token refresh patterns 2026"
- [x] 5.4 Update `frontend/src/app/(auth)/register/page.tsx` - Add strong password validation
- [x] 5.5 Add regex for uppercase, lowercase, number, special char
- [x] 5.6 Update `frontend/src/app/(auth)/login/page.tsx` - Add auth redirect check
- [x] 5.7 Add useEffect to redirect if user is logged in
- [x] 5.8 Update register page - Add same auth redirect check
- [x] 5.9 Create `frontend/src/lib/hooks/useProactiveRefresh.ts`
- [x] 5.10 Implement proactive refresh logic (5 min before expiry)
- [x] 5.11 Add useRefresh hook to `frontend/src/lib/queries/auth.ts`
- [x] 5.12 Update dashboard layout - Use useProactiveRefresh hook
- [x] 5.13 Remove onError toast from useLogin
- [x] 5.14 Remove onError toast from useRegister
- [x] 5.15 Remove onError toast from useVerifyEmail
- [x] 5.16 Remove onError toast from useResendVerification
- [x] 5.17 Update `frontend/src/middleware.ts` - Add documentation comment about limitation
- [x] 5.18 Run `pnpm lint` - Fix all warnings
- [x] 5.19 Run `pnpm build` - Ensure no errors
- [x] 5.20 Run frontend tests: `pnpm vitest run`
- [x] 5.21 If any errors, research with Exa: "[error message] Next.js 16 2026"

**Acceptance Criteria**:
- ✅ Strong password validation enforced
- ✅ Logged-in users redirected from auth pages
- ✅ Proactive token refresh works
- ✅ useRefresh hook available
- ✅ No duplicate error toasts
- ✅ All tests pass

**Completion Notes**:
- Added strong password validation to register page with regex (uppercase, lowercase, number, special char)
- Added auth redirect to login and register pages (redirect to /dashboard if already logged in)
- Created useProactiveRefresh hook that checks every minute and refreshes 5 minutes before expiry
- Added useRefresh hook to auth.ts with proper token update logic
- Updated dashboard layout to use useProactiveRefresh hook
- Removed duplicate onError toasts from useLogin, useRegister, useVerifyEmail, useResendVerification
- Updated middleware.ts with comprehensive documentation about localStorage limitation
- pnpm lint passed (only warnings from React Compiler and TanStack Table - not related to our changes)
- pnpm build passed successfully
- All 10 frontend tests passing (5 React Query properties + 5 validation properties)
- Saved to Cognio project zeltra-bug

---

## Task 6: Backend P2 - Session Tracking & Cleanup

**Priority**: P2 (Medium)
**Estimated Time**: 2 hours

### Subtasks:
- [ ] 6.1 Research with Exa: "Rust background job patterns tokio 2026"
- [ ] 6.2 Research with Exa: "Rust HTTP header extraction user agent IP 2026"
- [ ] 6.3 Research with Exa: "Rust session cleanup best practices 2026"
- [ ] 6.4 Update `backend/crates/api/src/routes/auth.rs` - Extract user agent from headers
- [ ] 6.5 Extract IP address from X-Forwarded-For or ConnectInfo
- [ ] 6.6 Pass user_agent and ip_address to session creation
- [ ] 6.7 Create `backend/crates/api/src/jobs/session_cleanup.rs`
- [ ] 6.8 Implement `start_session_cleanup_job()` function
- [ ] 6.9 Implement `cleanup_expired_sessions()` function
- [ ] 6.10 Add cleanup job to main.rs startup
- [ ] 6.11 Configure cleanup interval (default: 1 hour)
- [ ] 6.12 Add logging for cleanup operations
- [ ] 6.13 Run `cargo fmt`
- [ ] 6.14 Run `cargo clippy` - Fix all warnings
- [ ] 6.15 Run backend tests: `cargo test`
- [ ] 6.16 Test session cleanup manually
- [ ] 6.17 If any errors, research with Exa: "[error message] Rust tokio jobs 2026"

**Acceptance Criteria**:
- User agent and IP address tracked in sessions
- Expired sessions cleaned up automatically
- Cleanup runs every hour
- All tests pass

---

## Task 7: E2E Testing & Verification

**Priority**: P1 (High)
**Estimated Time**: 2 hours

### Subtasks:
- [ ] 7.1 Research with Exa: "Playwright authentication testing best practices 2026"
- [ ] 7.2 Research with Exa: "Next.js 16 E2E testing patterns 2026"
- [ ] 7.3 Test login with weak password (should fail with clear error)
- [ ] 7.4 Test login with strong password (should succeed)
- [ ] 7.5 Test register with weak password (should fail)
- [ ] 7.6 Test register with strong password (should succeed)
- [ ] 7.7 Test navigate to /login while logged in (should redirect to /dashboard)
- [ ] 7.8 Test navigate to /register while logged in (should redirect to /dashboard)
- [ ] 7.9 Test error toast only shows once (not duplicate)
- [ ] 7.10 Test token refresh before expiry (wait 55 minutes or mock time)
- [ ] 7.11 Test email validation (invalid email should fail)
- [ ] 7.12 Test per-user rate limiting (multiple requests from same user)
- [ ] 7.13 Verify all 21 bugs are fixed
- [ ] 7.14 Run full E2E test suite with Playwright
- [ ] 7.15 Manual testing of all auth flows
- [ ] 7.16 If any weird bugs found, save to Cognio (project: zeltra-bug) immediately
- [ ] 7.17 If any errors, research with Exa: "[error message] Playwright 2026"

**Acceptance Criteria**:
- All E2E tests pass
- All 21 bugs verified as fixed
- No regressions in existing functionality
- Manual testing completed
- Any new bugs found saved to Cognio

---

## Task 8: Documentation & Cleanup

**Priority**: P2 (Medium)
**Estimated Time**: 1 hour

### Subtasks:
- [ ] 8.1 Research with Exa: "Rust API documentation best practices 2026"
- [ ] 8.2 Research with Exa: "utoipa OpenAPI documentation patterns 2026"
- [ ] 8.3 Update README with new validation requirements
- [ ] 8.4 Document JWT secret configuration
- [ ] 8.5 Document rate limiting configuration
- [ ] 8.6 Create utoipa best practices guide
- [ ] 8.7 Document nullable annotation requirements
- [ ] 8.8 Document OptionalUpdate enum usage
- [ ] 8.9 Update API examples with new validation
- [ ] 8.10 Add security checklist to docs
- [ ] 8.11 Save all 21 bugs to Cognio memory (project: zeltra-bug)
- [ ] 8.12 Save utoipa bug patterns to Cognio for future reference
- [ ] 8.13 Save validation patterns to Cognio
- [ ] 8.14 If any weird bugs found during implementation, save to Cognio immediately
- [ ] 8.15 Final `cargo fmt` and `cargo clippy` check
- [ ] 8.16 Final `pnpm lint` and `pnpm build` check

**Acceptance Criteria**:
- All documentation updated
- Best practices documented
- Security checklist created
- All bugs saved to Cognio project zeltra-bug
- Code quality checks pass

---

## Future Enhancements (After All Bugs Fixed)

**Priority**: P3 (Low)
**Estimated Time**: TBD

### Subtasks:
- [ ]* 9.1 Implement Multi-Factor Authentication (TOTP/SMS 2FA)
- [ ]* 9.2 Add OAuth Integration (Google, GitHub social login)
- [ ]* 9.3 Implement Password Reset flow (forgot password)
- [ ]* 9.4 Create Session Management UI (view/revoke active sessions)
- [ ]* 9.5 Add Enhanced Audit Logging for auth events
- [ ]* 9.6 Implement IP Geolocation for login tracking
- [ ]* 9.7 Add Biometric authentication support
- [ ]* 9.8 Implement "Remember me" functionality
- [ ]* 9.9 Add password strength indicator UI component
- [ ]* 9.10 Implement account lockout after failed attempts

**Note**: These are optional future enhancements and should only be started after all 21 bugs are fixed and verified.

---

## Summary

**Total Tasks**: 8 major tasks
**Total Subtasks**: ~100 subtasks
**Estimated Total Time**: 13-15 hours
**Priority Breakdown**:
- P0: 1 task (JWT secret)
- P1: 5 tasks (validation, security, OpenAPI, frontend, E2E)
- P2: 2 tasks (session tracking, documentation)
- P3: 1 task (future enhancements - optional)

**Code Quality Checks**:
- `cargo fmt` and `cargo clippy` after every backend task
- `pnpm lint` and `pnpm build` after every frontend task
- Tests run after every major change

**Success Criteria**:
- ✅ All 21 bugs fixed
- ✅ All tests pass
- ✅ E2E tests pass
- ✅ Code quality checks pass
- ✅ Documentation updated

---

**Status**: Ready for Execution
