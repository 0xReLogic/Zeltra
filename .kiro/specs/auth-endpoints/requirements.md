# Auth Endpoints Bug Fixes - Requirements

**Feature**: Fix all authentication system bugs
**Date**: 2026-01-23
**Total Bugs**: 21 (Backend: 12, OpenAPI: 3, Frontend: 6)

---

## 1. Bug Summary

### Backend Bugs (12 total)
- **P0 (1)**: BUG-AUTH-007 - Default JWT secret risk
- **P1 (6)**: BUG-AUTH-003, 004, 005, 006, 011, 012
- **P2 (5)**: BUG-AUTH-001, 002, 008, 009, 010

### OpenAPI Bugs (3 total)
- **P1 (1)**: BUG-OPENAPI-001 - oneOf nullable syntax
- **P2 (1)**: BUG-OPENAPI-002 - Missing timezone default
- **P3 (1)**: BUG-OPENAPI-003 - Missing examples

### Frontend Bugs (6 total)
- **P1 (5)**: BUG-FRONTEND-002, 003, 004, 005, 006
- **P2 (1)**: BUG-FRONTEND-001 - Duplicate error toasts

---

## 2. Requirements by Priority

### 2.1 P0 - Critical (Must Fix First)

**REQ-1**: JWT Secret Validation
- Application must panic on startup if JWT secret is default value
- Only in production environment
- Clear error message

### 2.2 P1 - High Priority (Must Fix)

**REQ-2**: Input Validation
- Email format validation (backend + frontend)
- Password strength validation (backend + frontend)
- Frontend: uppercase, lowercase, number, special char required

**REQ-3**: Schema Annotations
- All `Option<T>` fields must have `#[schema(nullable = true)]`
- UpdateMemberRequest must use custom enum for nested Option
- OpenAPI oneOf patterns must be fixed

**REQ-4**: Frontend UX
- Logged-in users must be redirected from /login and /register
- Proactive token refresh (5 min before expiry)
- useRefresh hook must be created

**REQ-5**: Security
- SQL injection risk in RLS context must be mitigated
- Per-user rate limiting must be implemented

**REQ-6**: Middleware
- Document localStorage limitation OR implement cookie sync

### 2.3 P2 - Medium Priority (Should Fix)

**REQ-7**: Documentation
- Add examples to RegisterRequest, CreateOrganizationRequest, etc.
- Document timezone default value ("UTC")

**REQ-8**: Session Tracking
- Extract user agent from request headers
- Extract IP address from request
- Implement expired session cleanup job

**REQ-9**: Error Handling
- Remove duplicate error toasts (only toast in apiClient)

---

## 3. Acceptance Criteria

### 3.1 Security
- ✅ No default JWT secrets in production
- ✅ Email format validated
- ✅ Strong password requirements enforced
- ✅ SQL injection risks mitigated
- ✅ Per-user rate limiting implemented

### 3.2 Type Safety
- ✅ All nullable fields properly annotated
- ✅ OpenAPI spec matches backend types
- ✅ Frontend types generated correctly
- ✅ No nested Option<Option<T>> types

### 3.3 User Experience
- ✅ No duplicate error messages
- ✅ Logged-in users redirected from auth pages
- ✅ Seamless token refresh
- ✅ Clear validation error messages

### 3.4 Code Quality
- ✅ Backend: `cargo fmt` and `cargo clippy` pass
- ✅ Frontend: `pnpm lint` and `pnpm build` pass
- ✅ All tests pass
- ✅ No new warnings

---

## 4. Out of Scope (Future Tasks)

These will be added AFTER all bugs are fixed:
- Multi-Factor Authentication (2FA)
- OAuth social login
- Password reset flow
- Session management UI
- IP geolocation
- Biometric auth

---

## 5. Success Metrics

- ✅ All 21 bugs fixed and verified
- ✅ E2E tests pass for all auth flows
- ✅ Security audit passed
- ✅ No breaking API changes

---

**Status**: Ready for Design
