# Auth Endpoints Bug Fixes - Tasks

**Feature**: Fix all authentication system bugs
**Total Tasks**: 8 major tasks + future enhancements

---

## Task 1: Backend P0 - JWT Secret Validation

**Priority**: P0 (Critical)
**Estimated Time**: 30 minutes

### Subtasks:
- [ ] 1.1 Update `backend/crates/shared/src/jwt.rs` - Add panic if secret is default
- [ ] 1.2 Add clear error message with instructions
- [ ] 1.3 Test with default secret (should panic)
- [ ] 1.4 Test with custom secret (should work)
- [ ] 1.5 Run `cargo fmt`
- [ ] 1.6 Run `cargo clippy` - Fix all warnings
- [ ] 1.7 Run backend tests: `cargo test`

**Acceptance Criteria**:
- Application panics on startup if JWT secret is "change-me-in-production"
- Error message explains how to fix
- All tests pass

---

## Task 2: Backend P1 - Validation & Schema Fixes

**Priority**: P1 (High)
**Estimated Time**: 3 hours

### Subtasks:
- [ ] 2.1 Add `validator` crate to `backend/crates/shared/Cargo.toml`
- [ ] 2.2 Update `UpdateOrganizationRequest` - Add `#[schema(nullable = true)]` to all Option fields
- [ ] 2.3 Create `OptionalUpdate<T>` enum for nested Option handling
- [ ] 2.4 Update `UpdateMemberRequest` - Use `OptionalUpdate<String>` for approval_limit
- [ ] 2.5 Add `#[derive(Validate)]` to `LoginRequest` and `RegisterRequest`
- [ ] 2.6 Add `#[validate(email)]` to email fields
- [ ] 2.7 Add `#[validate(length(min = 8, max = 128))]` to password fields
- [ ] 2.8 Add `#[validate(length(min = 2, max = 255))]` to full_name field
- [ ] 2.9 Update login handler - Call `payload.validate()`
- [ ] 2.10 Update register handler - Call `payload.validate()`
- [ ] 2.11 Add `#[schema(example = "...")]` to RegisterRequest fields
- [ ] 2.12 Add `#[schema(default = "UTC", example = "...")]` to CreateOrganizationRequest.timezone
- [ ] 2.13 Run `cargo fmt`
- [ ] 2.14 Run `cargo clippy` - Fix all warnings
- [ ] 2.15 Run backend tests: `cargo test`
- [ ] 2.16 Test validation with invalid email (should fail)
- [ ] 2.17 Test validation with short password (should fail)

**Acceptance Criteria**:
- All Option<T> fields have nullable annotations
- UpdateMemberRequest uses OptionalUpdate enum
- Email and password validation works
- Examples added to all request schemas
- All tests pass

---

## Task 3: Backend P1 - Security Fixes

**Priority**: P1 (High)
**Estimated Time**: 2 hours

### Subtasks:
- [ ] 3.1 Update `backend/crates/db/src/rls.rs` - Add UUID validation in set_rls_context
- [ ] 3.2 Add error handling for nil UUID
- [ ] 3.3 Create `backend/crates/api/src/middleware/keyed_rate_limit.rs`
- [ ] 3.4 Implement `KeyedRateLimiter` struct with HashMap of limiters
- [ ] 3.5 Implement `extract_rate_limit_key()` function (user_id or IP)
- [ ] 3.6 Update rate limit middleware to use keyed limiting
- [ ] 3.7 Add configuration for per-user limits
- [ ] 3.8 Run `cargo fmt`
- [ ] 3.9 Run `cargo clippy` - Fix all warnings
- [ ] 3.10 Run backend tests: `cargo test`
- [ ] 3.11 Test rate limiting with multiple users
- [ ] 3.12 Test rate limiting with same user (should be limited)

**Acceptance Criteria**:
- RLS context validates UUID
- Per-user rate limiting implemented
- Rate limits configurable
- All tests pass

---

## Task 4: OpenAPI - Fix Nullable Syntax

**Priority**: P1 (High)
**Estimated Time**: 1 hour

### Subtasks:
- [ ] 4.1 Update `contracts/split-openapi.py` - Add oneOf pattern handling to fix_nullable_syntax()
- [ ] 4.2 Handle `oneOf: [type: 'null', $ref]` pattern
- [ ] 4.3 Convert to `allOf` with `nullable: true`
- [ ] 4.4 Run Python script: `python contracts/split-openapi.py`
- [ ] 4.5 Verify `OrganizationResponse.limits` uses correct syntax
- [ ] 4.6 Verify `DashboardMetricsResponse.period` uses correct syntax
- [ ] 4.7 Check all split YAML files for oneOf patterns
- [ ] 4.8 Commit regenerated OpenAPI files

**Acceptance Criteria**:
- Python script handles oneOf patterns
- All nullable $ref fields use allOf + nullable
- No oneOf patterns with type: 'null' remain
- OpenAPI files regenerated

---

## Task 5: Frontend P1 - Validation & UX Fixes

**Priority**: P1 (High)
**Estimated Time**: 2 hours

### Subtasks:
- [ ] 5.1 Update `frontend/src/app/(auth)/register/page.tsx` - Add strong password validation
- [ ] 5.2 Add regex for uppercase, lowercase, number, special char
- [ ] 5.3 Update `frontend/src/app/(auth)/login/page.tsx` - Add auth redirect check
- [ ] 5.4 Add useEffect to redirect if user is logged in
- [ ] 5.5 Update register page - Add same auth redirect check
- [ ] 5.6 Create `frontend/src/lib/hooks/useProactiveRefresh.ts`
- [ ] 5.7 Implement proactive refresh logic (5 min before expiry)
- [ ] 5.8 Add useRefresh hook to `frontend/src/lib/queries/auth.ts`
- [ ] 5.9 Update dashboard layout - Use useProactiveRefresh hook
- [ ] 5.10 Remove onError toast from useLogin
- [ ] 5.11 Remove onError toast from useRegister
- [ ] 5.12 Remove onError toast from useVerifyEmail
- [ ] 5.13 Remove onError toast from useResendVerification
- [ ] 5.14 Update `frontend/src/middleware.ts` - Add documentation comment about limitation
- [ ] 5.15 Run `pnpm lint` - Fix all warnings
- [ ] 5.16 Run `pnpm build` - Ensure no errors
- [ ] 5.17 Run frontend tests: `pnpm test`

**Acceptance Criteria**:
- Strong password validation enforced
- Logged-in users redirected from auth pages
- Proactive token refresh works
- useRefresh hook available
- No duplicate error toasts
- All tests pass

---

## Task 6: Backend P2 - Session Tracking & Cleanup

**Priority**: P2 (Medium)
**Estimated Time**: 2 hours

### Subtasks:
- [ ] 6.1 Update `backend/crates/api/src/routes/auth.rs` - Extract user agent from headers
- [ ] 6.2 Extract IP address from X-Forwarded-For or ConnectInfo
- [ ] 6.3 Pass user_agent and ip_address to session creation
- [ ] 6.4 Create `backend/crates/api/src/jobs/session_cleanup.rs`
- [ ] 6.5 Implement `start_session_cleanup_job()` function
- [ ] 6.6 Implement `cleanup_expired_sessions()` function
- [ ] 6.7 Add cleanup job to main.rs startup
- [ ] 6.8 Configure cleanup interval (default: 1 hour)
- [ ] 6.9 Add logging for cleanup operations
- [ ] 6.10 Run `cargo fmt`
- [ ] 6.11 Run `cargo clippy` - Fix all warnings
- [ ] 6.12 Run backend tests: `cargo test`
- [ ] 6.13 Test session cleanup manually

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
- [ ] 7.1 Test login with weak password (should fail with clear error)
- [ ] 7.2 Test login with strong password (should succeed)
- [ ] 7.3 Test register with weak password (should fail)
- [ ] 7.4 Test register with strong password (should succeed)
- [ ] 7.5 Test navigate to /login while logged in (should redirect to /dashboard)
- [ ] 7.6 Test navigate to /register while logged in (should redirect to /dashboard)
- [ ] 7.7 Test error toast only shows once (not duplicate)
- [ ] 7.8 Test token refresh before expiry (wait 55 minutes or mock time)
- [ ] 7.9 Test email validation (invalid email should fail)
- [ ] 7.10 Test per-user rate limiting (multiple requests from same user)
- [ ] 7.11 Verify all 21 bugs are fixed
- [ ] 7.12 Run full E2E test suite with Playwright
- [ ] 7.13 Manual testing of all auth flows

**Acceptance Criteria**:
- All E2E tests pass
- All 21 bugs verified as fixed
- No regressions in existing functionality
- Manual testing completed

---

## Task 8: Documentation & Cleanup

**Priority**: P2 (Medium)
**Estimated Time**: 1 hour

### Subtasks:
- [ ] 8.1 Update README with new validation requirements
- [ ] 8.2 Document JWT secret configuration
- [ ] 8.3 Document rate limiting configuration
- [ ] 8.4 Create utoipa best practices guide
- [ ] 8.5 Document nullable annotation requirements
- [ ] 8.6 Document OptionalUpdate enum usage
- [ ] 8.7 Update API examples with new validation
- [ ] 8.8 Add security checklist to docs
- [ ] 8.9 Final `cargo fmt` and `cargo clippy` check
- [ ] 8.10 Final `pnpm lint` and `pnpm build` check

**Acceptance Criteria**:
- All documentation updated
- Best practices documented
- Security checklist created
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
