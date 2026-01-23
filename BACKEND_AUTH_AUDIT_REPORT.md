# Backend Auth Audit Report

**Date**: 2026-01-23  
**Auditor**: Sub-Agent 1  
**Project**: Zeltra - Financial ERP System  
**Scope**: Backend authentication implementation, database schema, validation, and security

---

## Executive Summary

Comprehensive audit of backend authentication system covering:
- ✅ Rust struct definitions with utoipa annotations
- ✅ Database schema alignment
- ✅ OpenAPI specification correctness
- ✅ Security implementation (password hashing, JWT, session management)
- ⚠️ **3 Critical Issues Found** (Option<T> nullable annotations, nested struct issues)

---

## 1. Struct Annotations Audit

### 1.1 Auth Structs (`backend/crates/shared/src/auth.rs`)

| Struct | utoipa::ToSchema | Nullable Fields | Issues |
|--------|------------------|-----------------|--------|
| `LoginRequest` | ✅ Present | None | ✅ No issues - proper examples |
| `RegisterRequest` | ✅ Present | None | ⚠️ **BUG-AUTH-001**: Missing field examples |
| `RegisterResponse` | ✅ Present | None | ✅ No issues |
| `LoginResponse` | ✅ Present | None | ✅ No issues |
| `UserInfo` | ✅ Present | None | ✅ Proper examples |
| `UserOrganization` | ✅ Present | None | ✅ No issues |
| `RefreshRequest` | ✅ Present | None | ✅ No issues |
| `RefreshResponse` | ✅ Present | None | ✅ No issues |
| `LogoutRequest` | ✅ Present | None | ✅ No issues |
| `VerifyEmailRequest` | ✅ Present | None | ✅ No issues |
| `VerifyEmailResponse` | ✅ Present | None | ✅ No issues |
| `ResendVerificationRequest` | ✅ Present | None | ✅ No issues |
| `ResendVerificationResponse` | ✅ Present | None | ✅ No issues |
| `CreateOrganizationRequest` | ✅ Present | `timezone` (has default) | ⚠️ **BUG-AUTH-002**: `timezone` not marked nullable in OpenAPI |
| `AddUserRequest` | ✅ Present | `approval_limit: Option<String>` | ✅ Correctly marked nullable in OpenAPI |
| `UpdateOrganizationRequest` | ✅ Present | All fields `Option<T>` | ⚠️ **BUG-AUTH-003**: Missing `#[schema(nullable = true)]` annotations |
| `UpdateMemberRequest` | ✅ Present | `role`, `approval_limit` both `Option<T>` | ⚠️ **BUG-AUTH-004**: Nested Option not handled correctly |

### 1.2 Internal Structs (Not in OpenAPI)

| Struct | utoipa::ToSchema | Purpose | Issues |
|--------|------------------|---------|--------|
| `Claims` | ❌ Not present | Internal JWT claims | ✅ Correct - not exposed in API |
| `TokenPair` | ❌ Not present | Internal token wrapper | ✅ Correct - not exposed in API |

---

## 2. Database Schema Audit

### 2.1 Users Table

| Field | DB Type | Nullable | Rust Type | Constraints | Issues |
|-------|---------|----------|-----------|-------------|--------|
| `id` | uuid | NO | Uuid | PRIMARY KEY, default: gen_random_uuid() | ✅ Aligned |
| `email` | varchar(255) | NO | String | UNIQUE | ✅ Aligned |
| `password_hash` | varchar(255) | NO | String | - | ✅ Aligned |
| `full_name` | varchar(255) | NO | String | - | ✅ Aligned |
| `is_active` | boolean | NO | bool | default: true | ✅ Aligned |
| `email_verified_at` | timestamptz | YES | Option<DateTimeWithTimeZone> | - | ✅ Aligned |
| `created_at` | timestamptz | NO | DateTimeWithTimeZone | default: now() | ✅ Aligned |
| `updated_at` | timestamptz | NO | DateTimeWithTimeZone | default: now() | ✅ Aligned |

**Indexes**:
- ✅ `users_pkey` (PRIMARY KEY on id)
- ✅ `users_email_key` (UNIQUE on email)
- ✅ `idx_users_email` (btree on email WHERE is_active = true) - **Excellent for login queries**

**Assessment**: ✅ Perfect alignment between DB and Rust entity

### 2.2 Sessions Table

| Field | DB Type | Nullable | Rust Type | Constraints | Issues |
|-------|---------|----------|-----------|-------------|--------|
| `id` | uuid | NO | Uuid | PRIMARY KEY, default: gen_random_uuid() | ✅ Aligned |
| `user_id` | uuid | NO | Uuid | FOREIGN KEY → users(id) | ✅ Aligned |
| `organization_id` | uuid | NO | Uuid | FOREIGN KEY → organizations(id) | ✅ Aligned |
| `refresh_token_hash` | varchar(64) | NO | String | - | ✅ Aligned |
| `user_agent` | text | YES | Option<String> | - | ✅ Aligned |
| `ip_address` | varchar(45) | YES | Option<String> | - | ✅ Aligned |
| `expires_at` | timestamptz | NO | DateTimeWithTimeZone | CHECK: expires_at > now() | ✅ Aligned |
| `revoked_at` | timestamptz | YES | Option<DateTimeWithTimeZone> | - | ✅ Aligned |
| `created_at` | timestamptz | NO | DateTimeWithTimeZone | default: now() | ✅ Aligned |
| `updated_at` | timestamptz | NO | DateTimeWithTimeZone | default: now() | ✅ Aligned |

**Indexes**:
- ✅ `sessions_pkey` (PRIMARY KEY on id)
- ✅ `idx_sessions_token_hash` (btree on refresh_token_hash WHERE revoked_at IS NULL) - **Excellent for token lookup**
- ✅ `idx_sessions_user` (btree on user_id, created_at DESC WHERE revoked_at IS NULL) - **Excellent for user session queries**
- ✅ `idx_sessions_expires` (btree on expires_at WHERE revoked_at IS NULL) - **Good for cleanup queries**
- ✅ `idx_sessions_org` (btree on organization_id, created_at DESC) - **Good for org session queries**

**Assessment**: ✅ Excellent schema design with proper indexes for all auth queries

### 2.3 Email Verification Tokens Table

| Field | DB Type | Nullable | Rust Type | Constraints | Issues |
|-------|---------|----------|-----------|-------------|--------|
| `id` | uuid | NO | Uuid | PRIMARY KEY | ✅ Aligned |
| `user_id` | uuid | NO | Uuid | FOREIGN KEY → users(id) | ✅ Aligned |
| `token_hash` | varchar(64) | NO | String | - | ✅ Aligned |
| `expires_at` | timestamptz | NO | DateTimeWithTimeZone | - | ✅ Aligned |
| `used_at` | timestamptz | YES | Option<DateTimeWithTimeZone> | - | ✅ Aligned |
| `created_at` | timestamptz | NO | DateTimeWithTimeZone | default: CURRENT_TIMESTAMP | ✅ Aligned |

**Indexes**:
- ✅ `email_verification_tokens_pkey` (PRIMARY KEY on id)
- ✅ `idx_email_verification_tokens_hash` (btree on token_hash) - **Good for token lookup**
- ✅ `idx_email_verification_tokens_user` (btree on user_id) - **Good for user token queries**

**Assessment**: ✅ Proper schema design with appropriate indexes

---

## 3. Validation Audit

### 3.1 Endpoint Validation

| Endpoint | Validation | Issues |
|----------|------------|--------|
| `POST /auth/login` | ✅ Email lookup, password verification, account active check | ⚠️ **Missing**: Email format validation |
| `POST /auth/register` | ✅ Email uniqueness check, password hashing | ⚠️ **BUG-AUTH-005**: No email format validation<br>⚠️ **BUG-AUTH-006**: No password strength validation |
| `POST /auth/refresh` | ✅ Token validation, session expiry check, revocation check | ✅ Comprehensive |
| `POST /auth/logout` | ✅ Token revocation (idempotent) | ✅ Proper implementation |
| `POST /auth/verify-email` | ✅ Token validation, expiry check, usage check | ✅ Comprehensive |
| `POST /auth/resend-verification` | ✅ User lookup, already-verified check | ✅ Proper security (no email enumeration) |

### 3.2 Input Validation Gaps

**Critical Missing Validations**:
1. ❌ **Email format validation** - No regex/format check before DB operations
2. ❌ **Password strength requirements** - No minimum length, complexity checks
3. ❌ **Full name validation** - No length limits or character restrictions
4. ❌ **Organization slug validation** - No format/uniqueness checks in auth context

---

## 4. Security Audit

### 4.1 Password Security

| Aspect | Implementation | Assessment |
|--------|----------------|------------|
| **Hashing Algorithm** | Argon2id (default config) | ✅ **Excellent** - Industry standard, memory-hard |
| **Salt Generation** | `SaltString::generate(&mut OsRng)` | ✅ **Excellent** - Cryptographically secure random |
| **Hash Storage** | PHC string format | ✅ **Excellent** - Standard format |
| **Verification** | Constant-time comparison via Argon2 | ✅ **Excellent** - Timing attack resistant |
| **Password in Transit** | Plain text in JSON | ⚠️ **Requires HTTPS** - Ensure TLS enforced |

**Code Reference**: `backend/crates/core/src/auth/password.rs`

### 4.2 JWT Token Security

| Aspect | Implementation | Assessment |
|--------|----------------|------------|
| **Signing Algorithm** | HS256 (HMAC-SHA256) | ✅ Acceptable for symmetric keys |
| **Secret Key** | Configurable via environment | ⚠️ **BUG-AUTH-007**: Default is "change-me-in-production" |
| **Access Token Expiry** | 15 minutes (configurable) | ✅ **Excellent** - Short-lived |
| **Refresh Token Expiry** | 7 days (configurable) | ✅ Good balance |
| **Token Validation** | Signature + expiry check | ✅ Proper implementation |
| **Claims Structure** | user_id, org_id, role, iat, exp | ✅ Comprehensive |

**Code Reference**: `backend/crates/shared/src/jwt.rs`

### 4.3 Session Management

| Aspect | Implementation | Assessment |
|--------|----------------|------------|
| **Token Storage** | Hashed in database | ✅ **Excellent** - Not storing plain tokens |
| **Session Revocation** | `revoked_at` timestamp | ✅ Proper implementation |
| **Expiry Enforcement** | Checked on refresh | ✅ Proper implementation |
| **User Agent Tracking** | Optional field | ⚠️ **BUG-AUTH-008**: Not extracted from request headers |
| **IP Address Tracking** | Optional field | ⚠️ **BUG-AUTH-009**: Not extracted from request |
| **Cleanup Strategy** | No automatic cleanup | ⚠️ **BUG-AUTH-010**: No expired session cleanup job |

**Code Reference**: `backend/crates/api/src/routes/auth.rs` (lines 176-189)

### 4.4 SQL Injection Protection

| Aspect | Implementation | Assessment |
|--------|----------------|------------|
| **ORM Usage** | SeaORM for all queries | ✅ **Excellent** - Parameterized queries |
| **Raw SQL** | Only in migrations | ✅ Safe - No user input in raw queries |
| **RLS Context** | Uses `execute_unprepared` with UUID | ⚠️ **BUG-AUTH-011**: Potential SQL injection in RLS context setting |

**Issue Details**: In `backend/crates/db/src/rls.rs` line 63:
```rust
let sql = format!("SET LOCAL app.current_organization_id = '{organization_id}'");
txn.execute_unprepared(&sql).await?;
```
While UUID format is validated by type system, using string formatting is risky.

### 4.5 Rate Limiting

| Aspect | Implementation | Assessment |
|--------|----------------|------------|
| **Global Rate Limit** | ✅ Implemented with `governor` crate | ✅ Present |
| **Configuration** | 100 req/s, burst 200 (default) | ✅ Reasonable defaults |
| **Applied to Auth** | ✅ Applied globally via layer | ✅ Protects auth endpoints |
| **Per-User Limits** | ❌ Not implemented | ⚠️ **BUG-AUTH-012**: No per-user/per-IP rate limiting |

**Code Reference**: `backend/crates/api/src/middleware/rate_limit.rs`

### 4.6 Error Handling Security

| Aspect | Implementation | Assessment |
|--------|----------------|------------|
| **Password Errors** | Generic "Invalid email or password" | ✅ **Excellent** - No user enumeration |
| **Account Status** | Separate error for disabled accounts | ⚠️ Reveals account existence |
| **Token Errors** | Generic "Invalid or expired token" | ✅ Good |
| **Email Verification** | Generic message on resend | ✅ **Excellent** - No email enumeration |
| **Error Logging** | Detailed logs with tracing | ✅ Good for debugging |

---

## 5. OpenAPI Schema Alignment

### 5.1 Schema Comparison: Rust vs OpenAPI

| Schema | Rust Struct | OpenAPI Schema | Alignment |
|--------|-------------|----------------|-----------|
| `LoginRequest` | ✅ email, password | ✅ email, password | ✅ Perfect |
| `RegisterRequest` | ✅ email, password, full_name | ✅ email, password, full_name | ✅ Perfect |
| `LoginResponse` | ✅ user, access_token, refresh_token, expires_in | ✅ user, access_token, refresh_token, expires_in | ✅ Perfect |
| `UserInfo` | ✅ id, email, full_name, organizations | ✅ id, email, full_name, organizations | ✅ Perfect |
| `AddUserRequest` | ✅ approval_limit: Option<String> | ✅ approval_limit: nullable | ✅ Perfect |
| `UpdateOrganizationRequest` | ⚠️ name, base_currency, timezone all Option<T> | ✅ All marked nullable | ⚠️ **Missing Rust annotations** |
| `UpdateMemberRequest` | ⚠️ approval_limit: Option<Option<String>> | ⚠️ approval_limit: nullable | ⚠️ **Nested Option issue** |

### 5.2 Known utoipa Bug Patterns Found

1. **Option<T> without nullable annotation**: `UpdateOrganizationRequest` fields
2. **Nested Option<Option<T>>**: `UpdateMemberRequest.approval_limit` - utoipa cannot handle this
3. **Default values not reflected**: `CreateOrganizationRequest.timezone` has default but not in OpenAPI

---

## 6. Bugs Found

### BUG-AUTH-001: Missing Field Examples in RegisterRequest
- **Severity**: P2 (Low)
- **Location**: `backend/crates/shared/src/auth.rs` lines 78-84
- **Issue**: `RegisterRequest` fields lack `#[schema(example = "...")]` annotations
- **Impact**: OpenAPI docs less helpful for API consumers
- **Root Cause**: Inconsistent annotation style
- **Fix**: Add examples like in `LoginRequest`

### BUG-AUTH-002: CreateOrganizationRequest timezone not nullable
- **Severity**: P2 (Low)
- **Location**: `backend/crates/shared/src/auth.rs` lines 158-169
- **Issue**: `timezone` has `#[serde(default)]` but not marked nullable in OpenAPI
- **Impact**: Frontend may send null instead of omitting field
- **Root Cause**: utoipa doesn't detect serde defaults
- **Fix**: Either make it `Option<String>` or document that it's required with default

### BUG-AUTH-003: UpdateOrganizationRequest missing nullable annotations
- **Severity**: P1 (Medium)
- **Location**: `backend/crates/shared/src/auth.rs` lines 177-185
- **Issue**: All fields are `Option<T>` but lack `#[schema(nullable = true)]`
- **Impact**: OpenAPI spec may not mark fields as nullable, causing frontend validation errors
- **Root Cause**: Known utoipa bug - Option<T> not automatically marked nullable
- **Fix**: Add `#[schema(nullable = true)]` to all Option fields

```rust
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateOrganizationRequest {
    /// Organization name (optional).
    #[schema(nullable = true)]
    pub name: Option<String>,
    /// Base currency (optional, ISO 4217 code).
    #[schema(nullable = true)]
    pub base_currency: Option<String>,
    /// Timezone (optional, IANA format).
    #[schema(nullable = true)]
    pub timezone: Option<String>,
}
```

### BUG-AUTH-004: UpdateMemberRequest nested Option not handled
- **Severity**: P1 (Medium)
- **Location**: `backend/crates/shared/src/auth.rs` lines 188-194
- **Issue**: `approval_limit: Option<Option<String>>` - utoipa cannot handle nested Options
- **Impact**: OpenAPI spec incorrect, frontend cannot distinguish between "don't update" and "clear value"
- **Root Cause**: utoipa limitation with nested Option types
- **Fix**: Use a custom enum or wrapper type:

```rust
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum OptionalUpdate<T> {
    Unset,
    Set(Option<T>),
}

pub struct UpdateMemberRequest {
    #[schema(nullable = true)]
    pub role: Option<String>,
    #[schema(nullable = true)]
    pub approval_limit: OptionalUpdate<String>,
}
```

### BUG-AUTH-005: No Email Format Validation
- **Severity**: P1 (Medium)
- **Location**: `backend/crates/api/src/routes/auth.rs` - login and register handlers
- **Issue**: No email format validation before database operations
- **Impact**: Invalid emails can be registered, causing issues with email sending
- **Root Cause**: Missing validation layer
- **Fix**: Add email validation using `validator` crate or regex

```rust
use validator::Validate;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    // ...
}
```

### BUG-AUTH-006: No Password Strength Validation
- **Severity**: P1 (Medium)
- **Location**: `backend/crates/api/src/routes/auth.rs` register handler
- **Issue**: No password strength requirements enforced
- **Impact**: Weak passwords can be registered, security risk
- **Root Cause**: Missing validation layer
- **Fix**: Add password validation

```rust
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    // ...
}
```

### BUG-AUTH-007: Default JWT Secret in Production Risk
- **Severity**: P0 (Critical)
- **Location**: `backend/crates/shared/src/jwt.rs` lines 27-32
- **Issue**: Default JWT secret is "change-me-in-production"
- **Impact**: If deployed with default, all tokens can be forged
- **Root Cause**: Convenience default for development
- **Fix**: Panic on startup if secret is default value in production

```rust
impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        if config.secret == "change-me-in-production" {
            panic!("JWT secret must be changed in production!");
        }
        // ...
    }
}
```

### BUG-AUTH-008: User Agent Not Extracted
- **Severity**: P2 (Low)
- **Location**: `backend/crates/api/src/routes/auth.rs` line 178
- **Issue**: TODO comment - user agent not extracted from request headers
- **Impact**: Cannot track sessions by device/browser
- **Root Cause**: Not implemented yet
- **Fix**: Extract from request headers

```rust
use axum::http::HeaderMap;

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    // ...
}
```

### BUG-AUTH-009: IP Address Not Extracted
- **Severity**: P2 (Low)
- **Location**: `backend/crates/api/src/routes/auth.rs` line 179
- **Issue**: TODO comment - IP address not extracted from request
- **Impact**: Cannot track sessions by IP, no IP-based rate limiting
- **Root Cause**: Not implemented yet
- **Fix**: Extract from request using ConnectInfo or X-Forwarded-For header

### BUG-AUTH-010: No Expired Session Cleanup
- **Severity**: P2 (Low)
- **Location**: Session management - no cleanup job
- **Issue**: Expired sessions accumulate in database
- **Impact**: Database bloat over time
- **Root Cause**: No background job implemented
- **Fix**: Add periodic cleanup job or use database trigger

### BUG-AUTH-011: Potential SQL Injection in RLS Context
- **Severity**: P1 (Medium)
- **Location**: `backend/crates/db/src/rls.rs` line 63
- **Issue**: Using string formatting for SQL with UUID
- **Impact**: While UUID type prevents injection, pattern is risky
- **Root Cause**: No parameterized query support for SET LOCAL
- **Fix**: Add UUID validation or use prepared statement if possible

```rust
// Validate UUID format explicitly
if !organization_id.is_nil() {
    let sql = format!("SET LOCAL app.current_organization_id = '{organization_id}'");
    txn.execute_unprepared(&sql).await?;
}
```

### BUG-AUTH-012: No Per-User Rate Limiting
- **Severity**: P1 (Medium)
- **Location**: Rate limiting middleware
- **Issue**: Only global rate limiting, no per-user or per-IP limits
- **Impact**: Single user can exhaust rate limit for all users
- **Root Cause**: Tracked in BUG-014 (from approval rules)
- **Fix**: Implement keyed rate limiting with user_id or IP extraction

---

## 7. Recommendations

### 7.1 Immediate Fixes (P0/P1)

1. **BUG-AUTH-007**: Add JWT secret validation on startup
2. **BUG-AUTH-003**: Add nullable annotations to UpdateOrganizationRequest
3. **BUG-AUTH-004**: Refactor UpdateMemberRequest to handle nested Option
4. **BUG-AUTH-005**: Add email format validation
5. **BUG-AUTH-006**: Add password strength validation
6. **BUG-AUTH-011**: Validate UUID format in RLS context
7. **BUG-AUTH-012**: Implement per-user rate limiting

### 7.2 Prevention Measures

1. **Validation Layer**: Integrate `validator` crate for all request structs
2. **utoipa Linting**: Create custom lint to detect Option<T> without nullable annotation
3. **Security Checklist**: Add pre-deployment security checklist
4. **Automated Testing**: Add property-based tests for auth flows
5. **Documentation**: Document all utoipa bug patterns and workarounds

### 7.3 Future Enhancements

1. **Multi-Factor Authentication**: Add TOTP/SMS 2FA support
2. **OAuth Integration**: Add social login (Google, GitHub)
3. **Password Reset**: Implement forgot password flow
4. **Session Management UI**: Allow users to view/revoke active sessions
5. **Audit Logging**: Enhanced audit trail for auth events
6. **IP Geolocation**: Track login locations for security alerts

---

## 8. Conclusion

### Summary

The backend authentication implementation is **generally solid** with excellent security practices:

✅ **Strengths**:
- Argon2id password hashing with secure defaults
- Proper JWT implementation with short-lived access tokens
- Comprehensive session management with revocation
- Good database schema design with appropriate indexes
- SeaORM prevents SQL injection
- Rate limiting implemented globally
- No user enumeration in error messages

⚠️ **Critical Issues**:
- Missing input validation (email format, password strength)
- utoipa nullable annotation bugs in 3 structs
- Default JWT secret risk
- No per-user rate limiting

🔧 **Recommended Actions**:
1. Fix P0/P1 bugs immediately (7 bugs)
2. Add validation layer with `validator` crate
3. Implement per-user rate limiting
4. Add automated security testing

### Risk Assessment

- **Current Risk Level**: Medium
- **With Fixes Applied**: Low
- **Deployment Readiness**: Ready with fixes

---

**Report Generated**: 2026-01-23  
**Next Review**: After P0/P1 fixes implemented
