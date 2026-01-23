# Auth Endpoints Bug Fixes - Design

**Feature**: Fix all authentication system bugs
**Date**: 2026-01-23

---

## 1. Architecture Overview

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│   Frontend  │─────▶│   OpenAPI    │◀─────│   Backend   │
│  (Next.js)  │      │  (utoipa)    │      │   (Rust)    │
└─────────────┘      └──────────────┘      └─────────────┘
      │                      │                      │
      │                      │                      │
   6 bugs                 3 bugs                12 bugs
```

All bugs will be fixed in their respective layers with proper validation and testing.

---

## 2. Backend Fixes (12 bugs)

### 2.1 BUG-AUTH-007 (P0): JWT Secret Validation

**Location**: `backend/crates/shared/src/jwt.rs`

**Solution**:
```rust
impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        // Panic if default secret in production
        if config.secret == "change-me-in-production" {
            panic!(
                "SECURITY ERROR: JWT secret must be changed in production! \
                 Set JWT_SECRET environment variable."
            );
        }
        
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            access_token_expiry: config.access_token_expiry,
            refresh_token_expiry: config.refresh_token_expiry,
        }
    }
}
```

### 2.2 BUG-AUTH-003 (P1): UpdateOrganizationRequest Nullable Annotations

**Location**: `backend/crates/shared/src/auth.rs`

**Solution**:
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

### 2.3 BUG-AUTH-004 (P1): UpdateMemberRequest Nested Option

**Location**: `backend/crates/shared/src/auth.rs`

**Solution**: Create custom enum
```rust
#[derive(Debug, Clone, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum OptionalUpdate<T> {
    /// Field not provided - don't update
    #[serde(skip)]
    Unset,
    /// Field provided with value or null
    Set(Option<T>),
}

impl<T> Default for OptionalUpdate<T> {
    fn default() -> Self {
        Self::Unset
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateMemberRequest {
    /// Role (optional).
    #[schema(nullable = true)]
    pub role: Option<String>,
    
    /// Approval limit (optional, can be cleared).
    #[serde(default)]
    pub approval_limit: OptionalUpdate<String>,
}
```

### 2.4 BUG-AUTH-005 (P1): Email Format Validation

**Location**: `backend/crates/shared/src/auth.rs`

**Solution**: Add validator crate
```rust
use validator::Validate;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema, Validate)]
pub struct LoginRequest {
    /// User email.
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// User password.
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema, Validate)]
pub struct RegisterRequest {
    /// User email.
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// User password (min 8 characters).
    #[validate(length(min = 8, max = 128))]
    #[schema(example = "SecurePass123!")]
    pub password: String,
    
    /// User full name.
    #[validate(length(min = 2, max = 255))]
    #[schema(example = "John Doe")]
    pub full_name: String,
}
```

**Handler Update**:
```rust
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate input
    payload.validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;
    
    // ... rest of handler
}
```

### 2.5 BUG-AUTH-006 (P1): Password Strength Validation

**Solution**: Already handled by validator crate in BUG-AUTH-005
- Backend validates minimum length (8 chars)
- Frontend will validate complexity (see Frontend section)

### 2.6 BUG-AUTH-011 (P1): SQL Injection in RLS Context

**Location**: `backend/crates/db/src/rls.rs`

**Solution**: Add explicit UUID validation
```rust
pub async fn set_rls_context(
    txn: &DatabaseTransaction,
    organization_id: Uuid,
) -> Result<(), DbErr> {
    // Validate UUID is not nil
    if organization_id.is_nil() {
        return Err(DbErr::Custom("Invalid organization ID".to_string()));
    }
    
    // UUID type system already prevents injection, but be explicit
    let sql = format!("SET LOCAL app.current_organization_id = '{organization_id}'");
    txn.execute_unprepared(&sql).await?;
    Ok(())
}
```

### 2.7 BUG-AUTH-012 (P1): Per-User Rate Limiting

**Location**: `backend/crates/api/src/middleware/rate_limit.rs`

**Solution**: Implement keyed rate limiting
```rust
use governor::{Quota, RateLimiter};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct KeyedRateLimiter {
    limiters: Arc<RwLock<HashMap<String, RateLimiter<...>>>>,
    quota: Quota,
}

impl KeyedRateLimiter {
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let mut limiters = self.limiters.write().await;
        let limiter = limiters
            .entry(key.to_string())
            .or_insert_with(|| RateLimiter::direct(self.quota));
        
        limiter.check().map_err(|_| RateLimitError::TooManyRequests)
    }
}

// Extract key from request (user_id or IP)
pub fn extract_rate_limit_key(
    headers: &HeaderMap,
    user_id: Option<Uuid>,
) -> String {
    if let Some(id) = user_id {
        format!("user:{}", id)
    } else if let Some(ip) = headers.get("x-forwarded-for") {
        format!("ip:{}", ip.to_str().unwrap_or("unknown"))
    } else {
        "anonymous".to_string()
    }
}
```

### 2.8 BUG-AUTH-001 (P2): Missing Field Examples

**Solution**: Add examples to RegisterRequest (already shown in BUG-AUTH-005)

### 2.9 BUG-AUTH-002 (P2): Timezone Default Not Documented

**Location**: `backend/crates/shared/src/auth.rs`

**Solution**:
```rust
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateOrganizationRequest {
    // ... other fields
    
    /// Timezone (IANA format, defaults to UTC).
    #[serde(default = "default_timezone")]
    #[schema(default = "UTC", example = "America/New_York")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_string()
}
```

### 2.10 BUG-AUTH-008 (P2): User Agent Not Extracted

**Location**: `backend/crates/api/src/routes/auth.rs`

**Solution**:
```rust
use axum::http::HeaderMap;

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Extract user agent
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    
    // ... use in session creation
}
```

### 2.11 BUG-AUTH-009 (P2): IP Address Not Extracted

**Solution**: Extract from X-Forwarded-For or ConnectInfo
```rust
let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.split(',').next())
    .map(|s| s.trim().to_string());
```

### 2.12 BUG-AUTH-010 (P2): No Expired Session Cleanup

**Location**: New file `backend/crates/api/src/jobs/session_cleanup.rs`

**Solution**:
```rust
use tokio::time::{interval, Duration};

pub async fn start_session_cleanup_job(db: DatabaseConnection) {
    let mut interval = interval(Duration::from_secs(3600)); // Every hour
    
    loop {
        interval.tick().await;
        
        if let Err(e) = cleanup_expired_sessions(&db).await {
            tracing::error!("Session cleanup failed: {}", e);
        }
    }
}

async fn cleanup_expired_sessions(db: &DatabaseConnection) -> Result<(), DbErr> {
    let now = chrono::Utc::now();
    
    let result = Sessions::delete_many()
        .filter(sessions::Column::ExpiresAt.lt(now))
        .exec(db)
        .await?;
    
    tracing::info!("Cleaned up {} expired sessions", result.rows_affected);
    Ok(())
}
```

---

## 3. OpenAPI Fixes (3 bugs)

### 3.1 BUG-OPENAPI-001 (P1): oneOf Nullable Syntax

**Location**: `contracts/split-openapi.py`

**Solution**: Update `fix_nullable_syntax()` function
```python
def fix_nullable_syntax(obj):
    if isinstance(obj, dict):
        # Existing type array handling...
        
        # NEW: Handle oneOf with type: 'null' and $ref
        if 'oneOf' in obj and isinstance(obj['oneOf'], list):
            has_null_type = any(
                isinstance(item, dict) and item.get('type') == 'null' 
                for item in obj['oneOf']
            )
            if has_null_type:
                non_null_items = [
                    item for item in obj['oneOf'] 
                    if not (isinstance(item, dict) and item.get('type') == 'null')
                ]
                
                if len(non_null_items) == 1:
                    # Single non-null item: use allOf + nullable
                    single_item = non_null_items[0]
                    del obj['oneOf']
                    obj['allOf'] = [single_item]
                    obj['nullable'] = True
                else:
                    # Multiple non-null items: keep oneOf + nullable
                    obj['oneOf'] = non_null_items
                    obj['nullable'] = True
        
        # Recursively process...
```

**Regeneration Steps**:
1. Fix Python script
2. Run: `python contracts/split-openapi.py`
3. Verify all oneOf patterns fixed
4. Commit regenerated files

### 3.2 BUG-OPENAPI-002 (P2): Timezone Default Not Documented

**Solution**: Already fixed in backend (BUG-AUTH-002)
- Backend adds `#[schema(default = "UTC")]`
- OpenAPI will be regenerated automatically

### 3.3 BUG-OPENAPI-003 (P3): Missing Examples

**Solution**: Already fixed in backend (BUG-AUTH-001, BUG-AUTH-005)
- Backend adds `#[schema(example = "...")]` to all request structs
- OpenAPI will be regenerated automatically

---

## 4. Frontend Fixes (6 bugs)

### 4.1 BUG-FRONTEND-001 (P2): Duplicate Error Toasts

**Location**: `frontend/src/lib/queries/auth.ts`

**Solution**: Remove toast from mutation onError
```typescript
export function useLogin() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const router = useRouter()

  return useMutation({
    mutationFn: (data: LoginRequest) => 
      apiClient<AuthResponse>('/auth/login', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: (data) => {
      setAuth(data.user, data.access_token, data.refresh_token, data.expires_in)
      toast.success('Login successful')
      
      if (data.user.organizations.length === 0) {
        router.push('/onboarding/create-organization')
      } else {
        router.push('/dashboard')
      }
    },
    // REMOVE onError - apiClient already shows toast
  })
}
```

Apply same fix to: useRegister, useVerifyEmail, useResendVerification

### 4.2 BUG-FRONTEND-002 (P1): Missing useRefresh Hook

**Location**: `frontend/src/lib/queries/auth.ts`

**Solution**:
```typescript
export function useRefresh() {
  const setTokens = useAuthStore((state) => state.setTokens)
  const refreshToken = useAuthStore((state) => state.refreshToken)

  return useMutation({
    mutationFn: () => 
      apiClient<RefreshResponse>('/auth/refresh', {
        method: 'POST',
        body: JSON.stringify({ refresh_token: refreshToken || '' }),
      }),
    onSuccess: (data) => {
      setTokens(data.access_token, data.refresh_token, data.expires_in)
    },
  })
}
```

### 4.3 BUG-FRONTEND-003 (P1): No Proactive Token Refresh

**Location**: New file `frontend/src/lib/hooks/useProactiveRefresh.ts`

**Solution**:
```typescript
import { useEffect } from 'react'
import { useAuthStore } from '../stores/authStore'
import { useRefresh } from '../queries/auth'

export function useProactiveRefresh() {
  const tokenExpiresAt = useAuthStore((state) => state.tokenExpiresAt)
  const refreshToken = useAuthStore((state) => state.refreshToken)
  const { mutate: refresh } = useRefresh()

  useEffect(() => {
    if (!tokenExpiresAt || !refreshToken) return

    const checkInterval = setInterval(() => {
      const now = Date.now()
      const timeUntilExpiry = tokenExpiresAt - now
      const fiveMinutes = 5 * 60 * 1000

      // Refresh 5 minutes before expiry
      if (timeUntilExpiry > 0 && timeUntilExpiry < fiveMinutes) {
        console.log('🔄 Proactive token refresh triggered')
        refresh()
      }
    }, 60000) // Check every minute

    return () => clearInterval(checkInterval)
  }, [tokenExpiresAt, refreshToken, refresh])
}
```

**Usage in dashboard layout**:
```typescript
// frontend/src/app/dashboard/layout.tsx
import { useProactiveRefresh } from '@/lib/hooks/useProactiveRefresh'

export default function DashboardLayout({ children }: { children: React.Node }) {
  useProactiveRefresh() // Enable proactive refresh
  
  return <div>{children}</div>
}
```

### 4.4 BUG-FRONTEND-004 (P1): Weak Password Validation

**Location**: `frontend/src/app/(auth)/register/page.tsx` and `login/page.tsx`

**Solution**:
```typescript
const registerSchema = z.object({
  full_name: z.string().min(2, 'Name must be at least 2 characters'),
  email: z.string().email('Invalid email address'),
  password: z.string()
    .min(8, 'Password must be at least 8 characters')
    .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
    .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
    .regex(/[0-9]/, 'Password must contain at least one number')
    .regex(/[^A-Za-z0-9]/, 'Password must contain at least one special character'),
})
```

### 4.5 BUG-FRONTEND-005 (P1): Middleware Cannot Protect Routes

**Location**: `frontend/src/middleware.ts`

**Solution**: Document limitation
```typescript
// NOTE: This middleware cannot access localStorage where Zustand persists auth state.
// Route protection relies on client-side guards in page components.
// For production, consider syncing auth token to httpOnly cookies.

export function middleware(request: NextRequest) {
  // Middleware is limited - client-side protection is primary
  return NextResponse.next()
}
```

### 4.6 BUG-FRONTEND-006 (P1): No Auth Redirect on Login/Register

**Location**: `frontend/src/app/(auth)/login/page.tsx` and `register/page.tsx`

**Solution**:
```typescript
'use client'

import { useAuthStore } from '@/lib/stores/authStore'
import { useRouter } from 'next/navigation'
import { useEffect } from 'react'

export default function LoginPage() {
  const router = useRouter()
  const user = useAuthStore((state) => state.user)
  const accessToken = useAuthStore((state) => state.accessToken)
  
  useEffect(() => {
    // Redirect to dashboard if already logged in
    if (user && accessToken) {
      router.push('/dashboard')
    }
  }, [user, accessToken, router])
  
  // ... rest of component
}
```

Apply same fix to register page.

---

## 5. Testing Strategy

### 5.1 Backend Tests
- Unit tests for validation logic
- Integration tests for auth endpoints
- Test JWT secret validation panic
- Test rate limiting with multiple users

### 5.2 Frontend Tests
- Unit tests for password validation
- Test useRefresh hook
- Test proactive refresh logic
- Test auth redirect behavior

### 5.3 E2E Tests
- Login with weak password (should fail)
- Login with strong password (should succeed)
- Navigate to /login while logged in (should redirect)
- Token refresh before expiry (should be seamless)
- Duplicate error toast (should only show once)

---

## 6. Migration Plan

### Phase 1: Backend (P0 + P1)
1. Add validator crate dependency
2. Fix JWT secret validation
3. Add nullable annotations
4. Implement validation
5. Run `cargo fmt` and `cargo clippy`
6. Run tests

### Phase 2: OpenAPI
1. Fix Python script
2. Regenerate OpenAPI files
3. Verify schemas
4. Commit changes

### Phase 3: Frontend (P1)
1. Add password validation
2. Add auth redirect
3. Add useRefresh hook
4. Add proactive refresh
5. Remove duplicate toasts
6. Run `pnpm lint` and `pnpm build`
7. Run tests

### Phase 4: Backend (P2)
1. Add session tracking
2. Add cleanup job
3. Run tests

### Phase 5: E2E Testing
1. Run all E2E tests
2. Verify all bugs fixed
3. Manual testing

---

## 7. Rollback Plan

If issues arise:
1. Revert to previous commit
2. Fix issue in development
3. Re-test before deployment

---

**Design Status**: Ready for Implementation
