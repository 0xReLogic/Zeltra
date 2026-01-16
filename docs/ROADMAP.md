# Roadmap

Enterprise-grade B2B Expense & Budgeting Engine development timeline.

Start Date: January 2026

---

## AI Research Notes (WAJIB BACA)

> **PENTING:** Untuk topik-topik critical di bawah, AI WAJIB research dulu pake Exa/Tavily sebelum implement. Jangan halu!

### Kapan WAJIB Pake Exa/Tavily:

| Topic                        | Search Query                                                     | Why                                       |
| ---------------------------- | ---------------------------------------------------------------- | ----------------------------------------- |
| **SeaORM CLI**               | `sea-orm-cli generate entity 2025 2026 tutorial`                 | Syntax berubah tiap version               |
| **SeaORM 1.1 Entity Format** | `SeaORM 1.1 entity format derive macro`                          | Format baru di 2.0                        |
| **SeaORM Migration**         | `sea-orm-migration 2.0 create table example`                     | Migration API                             |
| **Axum 0.8 Router**          | `Axum 0.8 router state extractor 2025`                           | Breaking changes dari 0.7                 |
| **Axum Middleware**          | `Axum 0.8 tower middleware layer example`                        | Middleware pattern                        |
| **Double-Entry Accounting**  | `double entry bookkeeping debit credit rules assets liabilities` | Accounting rules biar gak salah           |
| **Trial Balance**            | `trial balance calculation example accounting`                   | Report logic                              |
| **Balance Sheet**            | `balance sheet format assets liabilities equity GAAP`            | Report structure                          |
| **Income Statement**         | `income statement P&L format revenue expenses`                   | Report structure                          |
| **Currency Revaluation**     | `foreign currency revaluation unrealized gain loss accounting`   | Complex accounting                        |
| **Fiscal Period Close**      | `month end close accounting process soft close hard close`       | Period management                         |
| **Dimensional Accounting**   | `dimensional accounting cost center department reporting`        | Enterprise feature                        |
| **JWT + Refresh Token**      | `JWT refresh token rotation Rust 2025 best practice`             | Security pattern                          |
| **Argon2id**                 | `Argon2id password hashing Rust example`                         | Password security                         |
| **PostgreSQL RLS**           | `PostgreSQL row level security multi-tenant example`             | RLS setup                                 |
| **Rust Decimal**             | `rust_decimal arithmetic precision money calculation`            | Money handling                            |
| **Concurrent Testing**       | `Rust tokio concurrent test race condition`                      | Stress test pattern                       |
| **Rounding Strategy**        | `bankers rounding half even rust_decimal`                        | Pajak/Diskon butuh rounding spesifik      |
| **Rounding Difference**      | `handling rounding difference split transaction accounting`      | $100/3 = $33.34 + $33.33 + $33.33         |
| **Frankfurter API**          | `https://frankfurter.dev/`                                       | Live exchange rates dari ECB, self-hosted |

### Research Template:

Sebelum implement fitur complex, jalankan:

```
1. Exa: `mcp_exa_get_code_context_exa` - untuk code examples
2. Tavily: `mcp_tavily_tavily_search` - untuk concepts/tutorials
```

---

## Philosophy: LEDGER-FIRST

> "Kalau Ledger lu salah, Dashboard lu cuma hiasan sampah."

This roadmap prioritizes:

1. Database integrity and correctness
2. Ledger engine with bulletproof double-entry
3. API layer WITH each feature (Vertical Slice)
4. Frontend LAST (after backend is rock-solid)

No frontend work until Phase 6. Backend must be battle-tested first.

---

## Phase 0: Foundation + Seeders (Week 1-2)

> **RESEARCH REQUIRED:**
>
> - SeaORM CLI: `sea-orm-cli generate entity latest tutorial`
> - SeaORM 1.1 migration: `sea-orm-migration 2.0 example`
> - Docker Postgres 16: `docker compose postgres 16 volume setup`

### Infrastructure Setup

- [x] Setup Rust workspace structure
- [x] Docker Compose for local development (Postgres 16)
- [x] CI/CD pipeline (GitHub Actions) - Rust tests only
- [ ] Heroku Postgres database setup (dev)
- [ ] Database backup strategy

### Database Foundation

- [x] Execute complete DDL schema
- [x] Verify all constraints and triggers work correctly
- [x] Test RLS policies manually
- [x] Setup SeaORM entity generation from schema
- [x] Create migration system (sea-orm-cli)

### Seeders (CRITICAL for Testing)

- [x] Seed `currencies` table (USD, EUR, GBP, JPY, IDR, SGD, etc.)
- [x] Seed `exchange_rates` with mock data (USD base rates for 30 days)
- [x] Seed `dimension_types` (DEPARTMENT, PROJECT, COST_CENTER)
- [x] Seed `dimension_values` (sample departments, projects)
- [x] Create seeder CLI command: `cargo run --bin seeder`

### Project Skeleton

- [x] Rust workspace: `api`, `core`, `db`, `shared` crates
- [x] Core crate: zero external dependencies (pure business logic)
- [x] Shared crate: types, errors
- [x] Basic Axum server that connects to DB

**Deliverable:** Database running with seeded data. Rust project compiles.

**Exit Criteria:**

- All DDL executes without error
- Triggers fire correctly (test manually)
- RLS blocks cross-tenant access
- SeaORM entities generated
- `exchange_rates` has 30 days of mock data
- Seeder runs without error

---

## Phase 1: Auth & Organization (Week 3-4)

> **WHY FIRST?** Transaction butuh `created_by` dan `organization_id` dari hari pertama. RLS butuh user context.

> **RESEARCH REQUIRED:**
>
> - JWT best practice: `JWT access token refresh token Rust 2025 security`
> - Argon2id: `Argon2id password hashing Rust argon2 crate example`
> - PostgreSQL RLS: `PostgreSQL row level security set config current_setting`

### Authentication

- [x] User registration (email, password, full_name)
- [x] Password hashing (Argon2id)
- [x] Email verification flow
- [x] JWT generation and validation
- [x] Refresh token flow
- [x] Session management

### Organization & Multi-Tenancy

- [x] Create organization (name, slug, base_currency, timezone)
- [x] Add user to organization with role (`user_role` enum)
- [x] Set `approval_limit` per user (for approver role)
- [x] Set RLS context per request (`app.current_organization_id`)
- [x] Test cross-tenant isolation

### Subscription & Tier Logic

- [x] Seed `tier_limits` table with Starter/Growth/Enterprise
- [x] Set default `subscription_tier = 'starter'` dan `subscription_status = 'trialing'`
- [x] Set `trial_ends_at` (14 days from org creation)
- [x] Implement `check_tier_limit()` function (users, transactions, dimensions)
- [x] Implement `has_feature()` function (simulation, multi_currency, sso, etc.)
- [x] Create `organization_usage` tracking (monthly counters)
- [x] Test tier limit enforcement
- [x] Test feature flag checks

### API Endpoints (Vertical Slice)

- [x] `POST /auth/register`
- [x] `POST /auth/login`
- [x] `POST /auth/refresh`
- [x] `POST /auth/logout`
- [x] `POST /auth/verify-email`
- [x] `POST /auth/resend-verification`
- [x] `POST /organizations`
- [x] `GET /organizations/:id`
- [x] `PATCH /organizations/:id` (update settings: name, base_currency, timezone)
- [x] `POST /organizations/:id/users`
- [x] `GET /organizations/:id/users`
- [x] `PATCH /organizations/:id/users/:userId` (update role, approval_limit)
- [x] `DELETE /organizations/:id/users/:userId` (remove user)

### Tests

```
tests/
├── auth/
│   ├── test_registration.rs
│   ├── test_login.rs
│   ├── test_jwt.rs
│   └── test_refresh_token.rs
├── organization/
│   ├── test_create_org.rs
│   ├── test_add_user.rs
│   └── test_rls_isolation.rs
```

**Deliverable:** Auth system working. Users can login and belong to organizations.

**Exit Criteria:**

- JWT flow works end-to-end
- Refresh token rotation works
- RLS isolates tenants completely
- 50+ tests passing

---

## Phase 2a: Ledger Core + API (Week 5-8) - BATTLE TESTED ✅

This is the MOST CRITICAL phase. It ensures the mathematical foundation of Zeltra is bulletproof.

> **RESEARCH REQUIRED (ACCOUNTING):**
>
> - Double-entry basics: `double entry bookkeeping debit credit rules`
> - Account types: `chart of accounts asset liability equity revenue expense normal balance`
> - Transaction posting: `journal entry posting general ledger accounting`
> - Balance calculation: `account balance debit credit calculation`
> - Multi-currency (Basic): `multi currency accounting functional currency translation`
> - Rounding: `bankers rounding half even accounting`

### Week 5-6: Basic Ledger Operations

#### Master Data CRUD + API

- [x] Fiscal years and periods CRUD
- [x] Chart of accounts CRUD
- [x] Dimension types and values CRUD
- [x] Exchange rates CRUD (manual entry)

#### Master Data API Endpoints

- [x] `POST /fiscal-years` + `GET /fiscal-years`
- [x] `POST /fiscal-periods` + `GET /fiscal-periods`
- [x] `PATCH /fiscal-periods/:id/status` (OPEN/SOFT_CLOSE/CLOSED)
- [x] `POST /accounts` + `GET /accounts` (List)
- [x] `GET /accounts/:id` (Detail) + `PUT /accounts/:id` (Update) + `DELETE /accounts/:id` (Delete)
- [x] `POST /dimension-types` + `GET /dimension-types`
- [x] `POST /dimension-values` + `GET /dimension-values`
- [x] `POST /exchange-rates` + `GET /exchange-rates`

#### Transaction Creation

- [x] Create transaction with entries (single currency)
- [x] Validate debit = credit (in functional currency)
- [x] Validate minimum 2 entries
- [x] Validate no zero amounts
- [x] Validate account exists and is active
- [x] Validate account allows direct posting

#### Account Balance Tracking

- [x] Implement `account_version` increment
- [x] Implement `previous_balance` / `current_balance` tracking
- [x] Test concurrent transactions on same account (race condition)
- [x] Verify balance never drifts (write stress test)

#### Database Trigger Verification

- [x] Test `check_transaction_balance` trigger
- [x] Test `update_account_balance` trigger
- [x] Test with DEFERRABLE constraint (multi-entry insert)

### Week 7-8: Multi-Currency & Dimensions

#### Multi-Currency

- [x] Exchange rate lookup function
- [x] Currency conversion (source → functional)
- [x] Store all 3 values (source_amount, exchange_rate, functional_amount)
- [x] Test conversion accuracy (no floating point errors)
- [x] Test missing exchange rate error handling
- [x] Implement rounding strategy (Banker's Rounding)
- [x] Handle rounding differences in split transactions

#### Dimensional Accounting

- [x] Validate dimension values exist
- [x] Link entries to dimensions
- [x] Test required dimension enforcement

#### Fiscal Period Validation

- [x] Find fiscal period for transaction date
- [x] Validate period status (OPEN/SOFT_CLOSE/CLOSED)
- [x] Test posting to closed period (must fail)
- [x] Test soft-close with different user roles

#### Transaction API Endpoints (Vertical Slice)

- [x] `POST /organizations/{org_id}/transactions` (create draft)
- [x] `GET /organizations/{org_id}/transactions` (list with filters)
- [x] `GET /organizations/{org_id}/transactions/:id` (detail with entries)
- [x] `PATCH /organizations/{org_id}/transactions/:id` (update draft only)
- [x] `DELETE /organizations/{org_id}/transactions/:id` (delete draft only)

---

## Phase 2b: Advanced Ledger Core (Sentinel Intelligence) 🟢 STRATEGIC

This adds intelligence and automation to the core ledger.

### Automation & Intelligence

- [x] **Real-time Revaluation Engine**: Daily background job to revalue foreign balances (unrealized G/L).
- [x] **Automated Accruals Engine**: Schedule-based recognition for prepayments/deferred revenue.
- [x] **Intercompany Transactions**: Automated mirrored entries between linked organizations.
- [ ] **Usage-Based Ledger API**: Real-time posting from telemetry/usage data.

### Compliance & ESG

- [x] **ESG Reporting Metadata**: Structure support for carbon/social impact tagging in ledger.
- [x] **Pillar Two Readiness**: Schema support for global minimum tax per jurisdiction.

### API Endpoints

- [x] `GET /organizations/{org_id}/revaluation-logs`
- [x] `POST /organizations/{org_id}/accrual-schedules`
- [x] `POST /organizations/{org_id}/intercompany/connect`

---

## Phase 3: Transaction Workflow + API (Week 9-10)

> **RESEARCH REQUIRED:**
>
> - Reversing entry: `reversing entry accounting void transaction journal`
> - Approval workflow: `approval workflow state machine Rust`
> - Immutable ledger: `immutable audit log accounting best practice`

### Status Transitions

- [x] Draft → Pending (submit)
- [x] Pending → Approved (approve)
- [x] Pending → Draft (reject, with reason)
- [x] Approved → Posted (post)
- [x] Posted → Voided (void, with reversing entry)

### Void with Reversing Entry

- [x] Create reversing transaction automatically
- [x] Link original ↔ reversing transaction
- [x] Verify balances after void

### Approval Rules Engine

- [x] CRUD for approval rules
- [x] Match transaction to approval rules
- [x] Check amount thresholds
- [x] Check user role hierarchy
- [x] Check user approval limit

### Immutability Enforcement

- [x] Test: Cannot UPDATE posted transaction
- [x] Test: Cannot DELETE posted transaction
- [x] Test: Cannot UPDATE voided transaction

### Workflow API Endpoints

- [x] `POST /transactions/:id/submit` (draft → pending)
- [x] `POST /transactions/:id/approve` (pending → approved)
- [x] `POST /transactions/:id/reject` (pending → draft)
- [x] `POST /transactions/:id/post` (approved → posted)
- [x] `POST /transactions/:id/void` (posted → voided)
- [x] `GET /transactions/pending` (approval queue)
- [x] `POST /transactions/bulk-approve` (approve multiple at once)
- [x] `POST /approval-rules` + `GET /approval-rules`

### Tests

```
tests/
├── workflow/
│   ├── test_status_transitions.rs
│   ├── test_void_reversing_entry.rs
│   ├── test_approval_rules.rs
│   └── test_immutability.rs
├── api/
│   └── test_workflow_api.rs
```

**Deliverable:** Complete transaction lifecycle with audit trail. API working.

**Exit Criteria:**

- [x] All status transitions work correctly
- [x] Void creates accurate reversing entry
- [x] Approval rules match correctly
- [x] Immutability cannot be bypassed
- [x] 50+ tests passing (515 tests!)

---

## Phase 4: Reports & Simulation + API (Week 11-13) ✅ COMPLETE

> **RESEARCH REQUIRED (ACCOUNTING REPORTS - CRITICAL!):**
>
> - Trial Balance: `trial balance report format debit credit totals`
> - Balance Sheet: `balance sheet format GAAP assets liabilities equity`
> - Income Statement: `income statement P&L format revenue expenses net income`
> - Account Ledger: `general ledger report format running balance`
> - Budget variance: `budget vs actual variance analysis favorable unfavorable`

> **RESEARCH REQUIRED (RUST):**
>
> - Rayon parallel: `Rayon parallel iterator Rust example`
> - Report caching: `Rust caching strategy moka cache`

### Budget Management

- [x] Budget CRUD
- [x] Budget lines with dimensions
- [x] Actual calculation from ledger
- [x] Variance calculation

### Core Reports

- [x] Trial Balance
- [x] Balance Sheet
- [x] Income Statement (P&L)
- [x] Account Ledger (with running balance)
- [x] Dimensional Report (slice by any dimension)

### Simulation Engine

- [x] Historical data aggregation
- [x] Baseline calculation
- [x] Projection with adjustments
- [x] Rayon parallel processing
- [x] Result caching

### Report & Simulation API Endpoints

- [x] `POST /budgets` + `GET /budgets`
- [x] `GET /budgets/:id` (budget detail)
- [x] `POST /budgets/:id/lines` + `GET /budgets/:id/lines`
- [x] `POST /budgets/:id/lock` (lock budget)
- [x] `GET /budgets/:id/vs-actual` (budget vs actual comparison)
- [x] `GET /reports/trial-balance`
- [x] `GET /reports/balance-sheet`
- [x] `GET /reports/income-statement`
- [x] `GET /reports/account-ledger/:account_id`
- [x] `GET /reports/dimensional`
- [x] `GET /reports/budget-vs-actual`
- [x] `POST /simulation/run`

### Tests

```
tests/
├── reports/
│   ├── test_trial_balance.rs
│   ├── test_balance_sheet.rs
│   ├── test_income_statement.rs
│   └── test_dimensional_report.rs
├── simulation/
│   ├── test_projection.rs
│   └── test_simulation_accuracy.rs
├── api/
│   └── test_reports_api.rs
```

**Deliverable:** All financial reports and simulation working via API.

**Exit Criteria:**

- [x] Reports match expected output (verified by accountant if possible)
- [x] Simulation produces reasonable projections
- [x] 716 tests passing (76 new integration tests)
- [x] Performance acceptable (<2s for 12-month simulation) - Benchmark: ~8ms for 12-month, ~123ms worst case (1000 accounts, 60 months)
- 50+ tests passing

---

## Phase 5: Attachments & API Polish (Week 14-15)

> **RESEARCH REQUIRED:**
>
> - Apache OpenDAL: `opendal` crate - unified storage API for 40+ backends
> - File upload: `Axum multipart file upload example`
> - OpenAPI generation: `Rust OpenAPI utoipa Axum 0.8 swagger`
> - Load testing: `k6 load testing REST API tutorial`

### Storage Architecture (Vendor-Agnostic with OpenDAL)

> **Why OpenDAL?** Apache project, production-ready, zero vendor lock-in.
> Switch storage backend via config only - no code changes needed.

```
┌─────────────────────────────────────────┐
│           Apache OpenDAL                │
│         (Unified Storage API)           │
├─────────────────────────────────────────┤
│ op.write("key", data)                   │
│ op.read("key")                          │
│ op.delete("key")                        │
│ op.presign_read("key", duration)        │
└─────────────────────────────────────────┘
          ▲           ▲           ▲
          │           │           │
    ┌─────┴───┐ ┌─────┴───┐ ┌─────┴───┐
    │  Azure  │ │   R2    │ │  Local  │
    │  Blob   │ │  (S3)   │ │  Disk   │
    │ (Free)  │ │ (10GB)  │ │ (Dev)   │
    └─────────┘ └─────────┘ └─────────┘
```

**Supported Backends (via config):**

- `azblob` - Azure Blob Storage (Student: Free 1yr, 5GB)
- `s3` - Cloudflare R2, AWS S3, MinIO (R2: Free 10GB, needs CC)
- `fs` - Local filesystem (development/testing)
- `gcs` - Google Cloud Storage (future)

**Config Example:**

```env
# Azure Blob (current - free student tier)
STORAGE_TYPE=azblob
AZURE_STORAGE_ACCOUNT=zeltradev
AZURE_STORAGE_ACCESS_KEY=xxx
AZURE_CONTAINER=attachments

# Cloudflare R2 (future - when need more storage)
# STORAGE_TYPE=s3
# S3_ENDPOINT=https://xxx.r2.cloudflarestorage.com
# S3_BUCKET=attachments
# S3_ACCESS_KEY_ID=xxx
# S3_SECRET_ACCESS_KEY=xxx
# S3_REGION=auto

# Local (development)
# STORAGE_TYPE=fs
# FS_ROOT=./storage/attachments
```

### Attachments Implementation

- [x] Add `opendal` dependency with features: `services-azblob`, `services-s3`, `services-fs`
- [x] Create `StorageService` wrapper around OpenDAL Operator
- [x] Config-based backend selection (env: `STORAGE_TYPE`)
- [x] Presigned URL generation for direct upload/download
- [x] Link attachments to transactions (`attachments` table)
- [x] File type validation (PDF, images, common docs)
- [x] Size limits (configurable, default 10MB)

### Attachment API Endpoints

- [x] `POST /attachments/upload` (get presigned URL for direct upload)
- [x] `POST /attachments` (confirm upload, link to transaction)
- [x] `GET /attachments/:id` (get presigned download URL)
- [x] `DELETE /attachments/:id`
- [x] `GET /transactions/:id/attachments`

### Live Exchange Rates (Frankfurter Integration)

> **RESEARCH REQUIRED:**
>
> - Frankfurter API docs: `https://frankfurter.dev/`
> - Self-hosting: `docker run -d -p 8080:8080 lineofflight/frankfurter`
> - Rust client (optional): `frankfurte-rs` crate atau bikin sendiri pake `reqwest`

- [x] Setup Frankfurter Docker container (self-hosted)
- [x] Create `ExchangeRateFetcher` service
- [x] Implement daily rate fetch (cron job)
- [x] Store fetched rates ke `exchange_rates` table
- [x] Config: pilih source (mock/frankfurter/manual)
- [x] Fallback ke last known rate kalau API down
- [x] API endpoint: `POST /exchange-rates/fetch` (manual trigger)
- [x] API endpoint: `POST /exchange-rates/bulk` (bulk import rates)

### Missing Master Data APIs (Frontend Mocked)

- [x] `PATCH /accounts/:id/status` (Toggle active/inactive)
- [x] `PATCH /dimension-values/:id` (Update value name/code)
- [x] `PATCH /dimension-values/:id/status` (Toggle active/inactive)

### Dashboard Analytics

- [x] Dashboard API: `GET /api/v1/dashboard/metrics`
  - Cash position, burn rate, runway days
  - Pending approvals count & amount
  - Budget status & utilization
  - Top expenses by department
  - Currency exposure
  - Cash flow chart data (weekly inflow/outflow)
  - Utilization chart data (budgeted vs actual by department)
- [x] Cash Flow API: `GET /api/v1/dashboard/cash-flow`
  - Monthly inflow/outflow data for charts
  - Optional period_id and months parameters
- [x] Activity Log API: `GET /api/v1/dashboard/recent-activity`
  - Transaction events (created, submitted, approved, rejected, posted, voided)
  - Budget events (created, updated, locked)
  - User events (invited, role changed)
  - Pagination with cursor
- [x] Budget Summary API: `GET /api/v1/dashboard/budget-vs-actual`
- [ ] Frontend Integration: Replace mock data with real-time API responses

### API Polish

- [x] OpenAPI/Swagger spec generation (utoipa)
- [x] API documentation
- [x] Consistent error format across all endpoints
- [x] Rate limiting
- [x] Request logging

### Load & Security Testing

- [x] Setup k6 or similar (Integration via concurrent internal tests)
- [x] Test concurrent transaction creation
- [x] Test report generation under load
- [x] SQL injection attempts
- [x] Cross-tenant access attempts
- [x] Invalid JWT handling

**Deliverable:** Production-ready API with comprehensive test coverage.

**Exit Criteria:**

- All API endpoints documented
- Load test: 100 concurrent users, <500ms p95
- Security tests passing
- 200+ total integration tests

---

## Phase 6: Frontend Foundation (Week 16-17) 🟡 IN PROGRESS

NOW we start frontend, because backend is solid.

> **RESEARCH REQUIRED:**
>
> - Next.js 16: `Next.js 16 app router setup 2026`
> - TanStack Query v5: `TanStack Query v5 React 19 setup`
> - Zustand: `Zustand React 19 store pattern`
> - Shadcn/UI: `Shadcn UI Next.js 16 setup`

### Setup

- [x] Next.js 16 project
- [x] Shadcn/UI components
- [x] TanStack Query configuration
- [x] Zustand stores
- [x] API client with typed responses

### Auth Pages (✅ Real API)

- [x] Login ✅ Real API
- [x] Register ✅ Real API
- [x] Forgot password (UI only)
- [x] Organization selector ✅ Real API
- [x] Logout ✅ Real API
- [x] Email verification page (`/verify-email?token=xxx`) ✅ Real API
- [x] Resend verification email UI ✅ Real API

### Core Layout

- [x] Sidebar navigation
- [x] Header with user menu
- [x] Responsive design

### Organization UI (✅ Real API)

> **DB Tables:** `organizations`, `organization_users` > **DB Fields:** `base_currency`, `timezone`, `role`, `approval_limit`

- [x] Organization Settings Page ✅ Real API
  - [x] Update base currency (`organizations.base_currency`)
  - [x] Update timezone (`organizations.timezone`)
  - [x] View subscription tier & status
- [x] User/Team Management Page ✅ Real API
  - [x] List organization users (`organization_users`)
  - [x] Invite new user (email + role)
  - [x] Update user role (`user_role` enum: viewer, accountant, approver, admin, owner, submitter) ✅ 6 roles
  - [x] Set approval limit for approvers (`approval_limit` field)
  - [x] Remove user from organization
- [x] Organization Creation UI ✅ Real API (2026-01-13)

### Real API Integration Progress (2026-01-13)

- [x] Mock API dependencies removed (MSW disabled)
- [x] API client optimized (30s timeout, 401 refresh logic)
- [x] Organization creation UI added
- [x] OpenAPI types generated from contracts/openapi.yaml
- [x] Auth flows (login, register, logout, refresh) ✅ Real API
- [x] Organization CRUD ✅ Real API
- [x] User/Team management ✅ Real API

**Deliverable:** Frontend skeleton with auth working.

**Status:** Auth & Organization = Real API ✅ | Other features = Need verification

### Technical Handover & Strategic Alignment 🟢 IMPORTANT

> [!IMPORTANT] > **PENTING UNTUK SELURUH TIM:** Hasil audit tier enforcement (Januari 2026) dan strategi monetisasi via UI gating ("Golden Lock"):

> [!IMPORTANT] > **PENTING UNTUK FE:** Hasil bug-fix di backend (Januari 2026) memerlukan penyesuaian di UI untuk menjaga integritas data:

1.  **Field Timezone Wajib**:
    - API `POST /organizations/{org_id}/transactions` sekarang butuh field `timezone` (e.g., `"Asia/Jakarta"`).
    - FE wajib ambil timezone user (browser settings) dan kirim ke BE.
    - Tanpa ini, API akan return **400 Bad Request**.
2.  **Transaction Response Update**:
    - Semua endpoint transaksi (Get, List, Create) sekarang mengembalikan field `timezone`.
    - Gunakan ini untuk menampilkan waktu transaksi yang akurat di UI sesuai input asli user.
3.  **Budget Dimension Validation**:
    - Jika API balikin error **400**, cek body response. BE akan mengirim list `missing_dimensions`.
    - Tampilkan pesan error spesifik: _"Dimensi 'Project' wajib diisi karena akun ini terikat budget."_
4.  **UI Button Protection**:
    - **Void Button**: Disable tombol Void jika status transaksi adalah `Reversal`. Transaksi pembatalan (void) bersifat final dan tidak boleh di-void kembali.
    - **Approve Button**: Jika `approval_limit` user di DB adalah `NULL`, user dianggap memiliki limit $0 (Paling Aman). Jangan izinkan user menekan approve jika amount transaksi > limit.
5.  **Audit Trail Consistency**:
    - Tabel ledger sekarang dijamin punya `account_version` yang sekuensial (1, 2, 3...) tanpa bolong (no gaps).
    - Ini menjamin _Running Balance_ yang ditampilkan di UI akan selalu akurat saat ditarik dari history.
6.  **Rounding Accuracy**: Total Debit/Credit di UI akan selalu balance sempurna karena BE sudah menggunakan _Residual Adjustment_ (pembulatan otomatis 0.01 error).
7.  **Exchange Rate List**: Gunakan endpoint `GET /exchange-rates/list` untuk histori kurs.
8.  **Auto Forex Gain/Loss**: Pakai `POST /transactions/pay-invoice` (BUKAN create transaction biasa) untuk pembayaran invoice mata uang asing. Selisih kurs dihitung otomatis.
9.  **Idempotency Key (Double-Posting Protection)**:
    - Request `POST /organizations/{org_id}/transactions` dan `POST /organizations/{org_id}/transactions/pay-invoice` mendukung field `idempotency_key` (UUID).
    - FE disarankan generate UUID di client untuk setiap transaksi baru guna mencegah duplikasi data jika terjadi masalah jaringan atau _double-click_.
10. **Sentinel Intelligence (ESG & Pillar Two)**:
    - Field `compliance_metadata` (JSON) kini tersedia di setiap `LedgerEntry`.
    - FE bisa mengirim object `esg` (carbon_offset, water_usage, social_impact) dan `pillar_two` (jurisdiction, local_tax_amount) untuk reporting global otomatis.
11. **OpenAPI Schema Sync**: Jalankan `pnpm run generate-api` untuk mendapatkan tipe data terbaru. Skema `ComplianceMetadata` dan endpoint Sentinel sudah masuk (`contracts/openapi.yaml`).
12. **Sentinel Intelligence Endpoints (LIVE)**:
    - **Revaluation**: `GET /organizations/{org_id}/revaluation-logs` - Riwayat gain/loss selisih kurs.
    - **Accruals**: `POST /organizations/{org_id}/accrual-schedules` - Registrasi biaya dibayar dimuka (Prepaid) / Accrual.
    - **Intercompany**: `POST /organizations/{org_id}/intercompany/connect` - Mapping akun antar cabang.
    - **Manual Override**: `LedgerEntryInput` kini mendukung field `functional_amount`. FE bisa override nilai kurs manual (misal untuk penyesuaian audit/pajak).
13. **Sentinel Tier Enforcement (Gembok Emas Backend) 🔒**:
    - Backend kini menerapkan pembatasan fitur secara ketat berdasarkan `subscription_tier`.
    - **User & Transaction Limits**: Endpoint `POST /users` dan `POST /transactions` akan return **402 Payment Required** jika kuota `max_users` atau `max_transactions_per_month` tercapai.
    - Jika user mencoba akses fitur premium (Auto-Accruals, Intercompany Hub, Revaluation) di tier yang kaga mendukung, API juga return **402**.
    - FE wajib handle error 402 ini dengan menampilkan modal upgrade.
14. **Exposed Feature Flags & Resource Quotas**:
    - Object `OrganizationResponse` kini punya field baru `limits: TierLimitsResponse`.
    - **Update Schema**: Field baru `max_users` (Option<i32>) dan `max_transactions_per_month` (Option<i32>) sudah tersedia. Null = Unlimited.
    - Gunakan field ini untuk menampilkan **Usage Meter** (e.g. "Users: 3 / 5") dan secara proaktif mengatur UI (hide/disable menu) sebelum user klik.
15. **Dimension Quotas (Starter Tier Limit)**:
    - Tier **Starter** dibatasi maksimal **2 Dimensi**. Cek field `limits.max_dimensions` untuk validasi sisi client.

---

### UI/UX Strategy: The "Golden Lock" �

Kita menggunakan strategi **"Show but Lock"** (Gembok Emas) daripada menyembunyikan fitur. Tujuannya adalah **Product-Led Growth (PLG)** - membiarkan user melihat potensi penuh Zeltra agar mereka terdorong untuk upgrade secara organik.

#### 🚩 Gating Rules by Tier

| Feature Category | Feature                |    🛡️ STARTER    | 🚀 GROWTH | 👑 ENTERPRISE |
| :--------------- | :--------------------- | :--------------: | :-------: | :-----------: |
| **Dimensions**   | Project & Cost Centers |    🔒 (Max 2)    |    ✅     |      ✅       |
| **Global**       | Auto-Sync (ECB)        | 🔒 (Manual Only) |    ✅     |      ✅       |
| **Global**       | Real-Time Revaluation  |        🔒        |    ✅     |      ✅       |
| **Intelligence** | Accruals Engine        |        🔒        |    🔒     |      ✅       |
| **Intelligence** | Budget Simulation      |        🔒        |    🔒     |      ✅       |
| **Intelligence** | Intercompany Hub       |        🔒        |    🔒     |      ✅       |
| **Reporting**    | Full Analytics Dash    |        🔒        |    ✅     |      ✅       |
| **Reporting**    | Forensic Suite (AI)    |        🔒        |    🔒     |      ✅       |
| **Budgets**      | Budget Creation        |     🔒 (> 3)     |    ✅     |      ✅       |

#### 💡 Implementation for Frontend:

- **Visual**: Gunakan ikon gembok warna **Gold/Amber** di samping menu/tombol yang terkunci.
- **Interaction**: Saat diklik, tampilkan **Upgrade Modal** (Upsell screen).
- **Resource Quotas**: Hitung limit (Users, Tx, Budgets) secara dinamis menggunakan field `limits` dari `OrganizationResponse`. Jika quota penuh, ubah UI menjadi **Red/Warning State** + Disable action dengan ikon 🔒.

### Master Data UI (✅ Real API - Verified via E2E)

> **DB Tables:** `chart_of_accounts`, `fiscal_years`, `fiscal_periods`, `dimension_types`, `dimension_values`, `exchange_rates`

- [x] Chart of Accounts management ✅ Real API
  - [x] List accounts with hierarchy (parent_id)
  - [x] Account type/subtype display
  - [x] Create new account (`POST /accounts`)
  - [x] Edit account (`PATCH /accounts/:id`)
  - [x] Delete account (`DELETE /accounts/:id`)
  - [x] Toggle account active status
- [x] Fiscal period management ✅ Real API
  - [x] List fiscal years with nested periods
  - [x] Period status badges (OPEN/SOFT_CLOSE/CLOSED)
  - [x] Change period status (`PATCH /fiscal-periods/:id/status`)
  - [x] Create fiscal year with auto-generated periods (`POST /fiscal-years`)
- [x] Dimension management ✅ Real API
  - [x] List dimension types with nested values
  - [x] Add dimension value (`POST /dimensions/:typeId/values`)
  - [x] Create dimension type (`POST /dimension-types`)
  - [x] Edit dimension value
  - [x] Toggle dimension active status
- [x] Exchange rate management ✅ Real API
  - [x] List exchange rates
  - [x] Add exchange rate (`POST /exchange-rates`)
  - [x] Bulk import exchange rates

### Transaction UI (✅ Real API - Verified via E2E)

> **DB Tables:** `transactions`, `ledger_entries`, `entry_dimensions`

- [x] Transaction list with filters ✅ Real API
  - [x] Filter by status (draft/pending/approved/posted/voided)
  - [x] Filter by date range
  - [x] Filter by transaction type
  - [x] Filter by dimension (department/project)
- [x] Transaction entry form ✅ Real API
  - [x] Multi-line journal entry (debit/credit)
  - [x] Account selector
  - [x] Dimension assignment per entry (`entry_dimensions`)
  - [x] Multi-currency support (source_currency, exchange_rate)
  - [x] Attachment upload
- [x] Approval queue ✅ Real API
  - [x] List pending transactions
  - [x] Approve/Reject actions
  - [x] Bulk approve
- [x] Transaction detail ✅ Real API
  - [x] View entries with debit/credit
  - [x] View dimension assignments
  - [x] View attachments
  - [x] Audit trail (submitted_by, approved_by, posted_by timestamps)

### Dashboard (✅ Real API - Verified via E2E)

> **Computed from:** `ledger_entries`, `budgets`, `budget_lines`

- [x] Key metrics ✅ Real API
  - [x] Cash position (sum of cash accounts)
  - [x] Burn rate (daily/monthly)
  - [x] Runway days
  - [x] Pending approvals count
- [x] Budget vs actual ✅ Real API
  - [x] Department budget cards
  - [x] Progress bars (actual/budget)
  - [x] Variance highlighting (favorable/unfavorable)
- [x] Charts (Recharts) ✅ Real API
  - [x] Expense trend chart
  - [x] Cash flow chart
  - [x] Budget utilization by department
- [x] Recent Activity Widget ✅ Real API
  - [x] Feed-style list of latest transaction & budget actions
  - [x] Uses `GET /organizations/{org_id}/dashboard/recent-activity` endpoint
  - [x] Activity type icons (created, approved, posted, voided, etc.)
  - [x] Relative timestamps ("2 hours ago")
  - [x] Click to navigate to transaction/budget detail

### Reports UI (✅ Real API - Verified via E2E)

> **Computed from:** `ledger_entries`, `chart_of_accounts`, `entry_dimensions`

- [x] Report viewer ✅ Real API
  - [x] Trial Balance
  - [x] Balance Sheet
  - [x] Income Statement (P&L)
- [x] Export functionality (UI done)
  - [x] CSV export
  - [x] PDF export
- [x] **Account Ledger View** ✅ Real API
  - [x] Select account from dropdown (via Account List)
  - [x] Show all entries for account (`ledger_entries.account_id`)
  - [x] Running balance column (`account_current_balance`)
  - [x] Date range filter
  - [x] `GET /organizations/{org_id}/accounts/:id/ledger`
- [x] **Dimensional Reports UI** ✅ Real API
  - [x] Filter by dimension type (Department/Project/Cost Center)
  - [x] Filter by dimension value (Department/Project/Cost Center)
  - [x] Group expenses by dimension (Chart & Table)
  - [x] Compare across dimensions
  - [x] `GET /organizations/{org_id}/reports/dimensional`

### Advanced Features (✅ Real API - Partial Verification)

> **DB Tables:** `budgets`, `budget_lines`, `budget_line_dimensions`

- [x] **Fiscal Year Creation UI** ✅ Real API
  - [x] Form: name, start_date, end_date
  - [x] Auto-generate 12 monthly periods
  - [x] Option for adjustment period (period 13)
  - [x] `POST /organizations/{org_id}/fiscal-years`
- [x] **Attachments UI** ✅ Real API
  - [x] Upload file with presigned URL
  - [x] List attachments on transaction detail
  - [x] Download attachment
  - [x] Delete attachment
  - [x] File type and size validation
- [x] **Budget Management UI** ✅ Real API
  - [x] Create budget (`budgets` table)
  - [x] Add budget lines per account/period (`budget_lines`)
  - [x] Assign dimensions to budget lines (`budget_line_dimensions`)
  - [x] Lock/unlock budget
  - [x] `POST /organizations/{org_id}/budgets`, `POST /organizations/{org_id}/budgets/:id/lines`
- [x] Simulation/Forecasting UI ✅ Real API
  - [x] Historical data selection
  - [x] Adjustment parameters
  - [x] Projection results

### Real API Integration Status (2026-01-13) - Updated via E2E Testing

**✅ Verified Real API (Playwright E2E Tested - 2026-01-13):**

- Auth (login, register, logout, refresh)
- Organization CRUD
- User/Team management
- Role management (6 roles)
- Dashboard (metrics, cash flow, recent activity)
- Transactions list & CRUD (create, workflow actions)
- Accounts list & CRUD (create, edit, delete, toggle status)
- Budgets list & CRUD (create, add lines, lock, vs-actual)
- Dimensions list & CRUD (create types, add values, edit, toggle status)
- Fiscal periods (list, create year, change period status)
- Exchange rates (list, add rate, bulk import)
- Reports/Trial Balance (shows empty table with $0.00 totals)

**✅ OpenAPI Types Migration (2026-01-13):**

- All frontend types now use auto-generated types from `api.generated.ts`
- Type helper utilities in `types/api-helpers.ts`
- Backward compatibility maintained with type aliases
- Created dedicated type files: `transactions.ts`, `accounts.ts`, `budgets.ts`, `dimensions.ts`, `fiscal.ts`, `exchange-rates.ts`

**✅ Query/Mutation Updates (2026-01-13):**

- Transaction queries: CRUD + workflow mutations (submit, approve, reject, post, void, bulk-approve)
- Account queries: CRUD + toggle status mutation
- Budget queries: CRUD + lines + lock + vs-actual
- Dimension queries: types + values CRUD + toggle status
- Fiscal queries: years + periods + create year + update period status
- Exchange rate queries: list + create + bulk import + fetch live rates

**⚠️ Need Further Verification:**

- Forensic Suite (partially verified)
- Mobile responsiveness (Phase 10)

**🔧 Response Type Fixes Applied (2026-01-13 - 2026-01-15):**

- Transactions: Changed from `{ data: [], pagination: {} }` to `TransactionListItem[]`
- Accounts: Changed from `{ data: [] }` to `Account[]`
- Budgets: Changed from `{ data: [] }` to `Budget[]`
- CashFlow: Added wrapper type to extract `data` array

**✅ All Org-Scoped Endpoints Updated:**

- Dashboard: `/organizations/{org_id}/dashboard/*`
- Transactions: `/organizations/{org_id}/transactions`
- Accounts: `/organizations/{org_id}/accounts`
- Budgets: `/organizations/{org_id}/budgets`
- Dimensions: `/organizations/{org_id}/dimension-types`
- Reports: `/organizations/{org_id}/reports/*`

**Deliverable:** Complete frontend application.

**Status:** UI Complete | Real API Integration = ✅ Core Features Working (Auth + Org + Dashboard + Lists + CRUD Operations + Simulation + Attachments + Account Ledger + Dimensional Reports + Fiscal Year Creation)

- Transactions (CRUD + workflow)
- Dashboard (metrics, cash flow, recent activity)
- Reports (trial balance, balance sheet, income statement, dimensional) ✅ Real API
- Budgets (CRUD + lines)
- Simulation ✅ Real API
- Attachments ✅ Real API
- Account Ledger ✅ Real API
- Dimensional Reports ✅ Real API
- Fiscal Year Creation ✅ Real API

**Deliverable:** Complete frontend application.

**Status:** UI Complete | Real API Integration = Partial (Auth + Org only)

---

## Phase 8: Polish & Launch (Week 21-22)

> **RESEARCH REQUIRED:**
>
> - Playwright E2E: `Playwright Next.js 16 E2E testing setup`
> - Vercel deploy: `Vercel Next.js 16 deployment environment variables`
> - DigitalOcean Docker: `DigitalOcean Docker container deploy Rust`
> - Monitoring: `Rust application monitoring Prometheus Grafana`
> - Error tracking: `Sentry Rust Axum error tracking setup`
> - Billing: `Stripe subscription webhook Rust example` atau `LemonSqueezy API integration`

### Billing & Subscription (Payment Provider Agnostic)

> **Provider Options:** Stripe, LemonSqueezy, Paddle, atau manual invoicing untuk enterprise.
> Arsitektur harus agnostic - gampang switch provider.

#### Payment Provider Abstraction

- [ ] Create `PaymentProvider` trait/interface di Rust
- [ ] Implement `StripeProvider` (atau provider pilihan)
- [ ] Config-based provider selection (env var)
- [ ] Webhook signature validation per provider

#### Integration Tasks

- [ ] Setup products & pricing tiers di provider dashboard
- [ ] Map provider price IDs ke `subscription_tier` enum (config file)
- [ ] Checkout flow: frontend redirect ke provider checkout
- [ ] Webhook handler: subscription created → update org tier & status
- [ ] Webhook handler: subscription updated → sync tier/status
- [ ] Webhook handler: subscription cancelled → set status cancelled
- [ ] Webhook handler: payment failed → set status past_due
- [ ] Update `organizations.payment_*` fields
- [ ] Customer portal redirect (manage subscription)
- [ ] Trial expiry cron job (trialing → expired after 14 days)
- [ ] Grace period handling (past_due → expired after 7 days)

#### Manual/Enterprise Billing

- [ ] Support `payment_provider = 'manual'` untuk enterprise deals
- [ ] Admin endpoint to manually set tier (for invoiced customers)

### Testing

- [ ] E2E tests (Playwright)
- [ ] Cross-browser testing
- [ ] Mobile responsiveness

### Production Setup

- [ ] Vercel deployment
- [ ] DigitalOcean production server
- [ ] Production database
- [ ] Cloudflare R2 for attachments
- [ ] Monitoring and alerting

### Launch

- [ ] Beta testing
- [ ] Bug fixes
- [ ] Public launch

**Deliverable:** Live production system with billing.

### Accounting Readiness (MVP) - 🟢 CRITICAL ADDITION

> **Moved from Phase 12:** Required for Day 1 Accountant usability.

- [ ] **Bank Reconciliation (Manual MVP):**
  - [ ] Schema: `external_statements` table implementation.
  - [ ] UI: Manual CSV statement upload (Bank Format Agnostic).
  - [ ] UI: Split-screen matching interface (Ledger vs Statement).
  - [ ] Report: Reconciliation Discrepancy Report (PDF).
- [ ] **Onboarding Templates (CoA):**
  - [ ] Pre-seeded templates: "SaaS Startup", "Agency", "Retail".
  - [ ] Auto-mapping to IFRS/GAAP categories during setup.
