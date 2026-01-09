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

## Phase 2: Ledger Core + API (Week 5-8)

This is the MOST CRITICAL phase. Take your time. Get it right.

> **RESEARCH REQUIRED (ACCOUNTING - JANGAN HALU!):**
>
> - Double-entry basics: `double entry bookkeeping debit credit rules`
> - Account types: `chart of accounts asset liability equity revenue expense normal balance`
> - Transaction posting: `journal entry posting general ledger accounting`
> - Balance calculation: `account balance debit credit calculation`
> - Multi-currency: `multi currency accounting functional currency translation`
> - Exchange rate: `foreign exchange rate accounting spot rate`
> - Rounding: `bankers rounding half even accounting`

> **RESEARCH REQUIRED (RUST):**
>
> - SeaORM transactions: `SeaORM 1.1 database transaction begin commit rollback`
> - Decimal arithmetic: `rust_decimal checked_add checked_sub example`
> - Concurrent access: `Rust PostgreSQL concurrent update optimistic locking`

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

- [x] `POST /transactions` (create draft)
- [x] `GET /transactions` (list with filters)
- [x] `GET /transactions/:id` (detail with entries)
- [x] `PATCH /transactions/:id` (update draft only)
- [x] `DELETE /transactions/:id` (delete draft only)

### Ledger Tests (CRITICAL)

```
tests/
├── ledger/
│   ├── test_create_transaction.rs
│   ├── test_balance_tracking.rs
│   ├── test_concurrent_transactions.rs
│   ├── test_multi_currency.rs
│   ├── test_rounding.rs
│   ├── test_dimensions.rs
│   ├── test_fiscal_period.rs
│   └── test_balance_never_drifts.rs  # Stress test
├── api/
│   ├── test_transactions_api.rs
│   ├── test_accounts_api.rs
│   └── test_fiscal_periods_api.rs
```

**Deliverable:** Ledger engine that NEVER produces incorrect balances. API endpoints working.

**Exit Criteria:**

- [x] 150+ unit tests passing (229 tests!)
- [x] Concurrent transaction stress test passing
- [x] Balance drift test passing (1000+ transactions)
- [x] Multi-currency conversion accurate to 4 decimal places
- [x] All fiscal period rules enforced
- [x] All API endpoints return correct responses
- [x] Postman/curl testing works

---

## Phase 3: Transaction Workflow + API (Week 9-10)

> **RESEARCH REQUIRED:**
>
> - Reversing entry: `reversing entry accounting void transaction journal`
> - Approval workflow: `approval workflow state machine Rust`
> - Immutable ledger: `immutable audit log accounting best practice`

### Status Transitions

- [ ] Draft → Pending (submit)
- [ ] Pending → Approved (approve)
- [ ] Pending → Draft (reject, with reason)
- [ ] Approved → Posted (post)
- [ ] Posted → Voided (void, with reversing entry)

### Void with Reversing Entry

- [ ] Create reversing transaction automatically
- [ ] Link original ↔ reversing transaction
- [ ] Verify balances after void

### Approval Rules Engine

- [ ] CRUD for approval rules
- [ ] Match transaction to approval rules
- [ ] Check amount thresholds
- [ ] Check user role hierarchy
- [ ] Check user approval limit

### Immutability Enforcement

- [ ] Test: Cannot UPDATE posted transaction
- [ ] Test: Cannot DELETE posted transaction
- [ ] Test: Cannot UPDATE voided transaction

### Workflow API Endpoints

- [ ] `POST /transactions/:id/submit` (draft → pending)
- [ ] `POST /transactions/:id/approve` (pending → approved)
- [ ] `POST /transactions/:id/reject` (pending → draft)
- [ ] `POST /transactions/:id/post` (approved → posted)
- [ ] `POST /transactions/:id/void` (posted → voided)
- [ ] `GET /transactions/pending` (approval queue)
- [ ] `POST /approval-rules` + `GET /approval-rules`

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

- All status transitions work correctly
- Void creates accurate reversing entry
- Approval rules match correctly
- Immutability cannot be bypassed
- 50+ tests passing

---

## Phase 4: Reports & Simulation + API (Week 11-13)

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

- [ ] Budget CRUD
- [ ] Budget lines with dimensions
- [ ] Actual calculation from ledger
- [ ] Variance calculation

### Core Reports

- [ ] Trial Balance
- [ ] Balance Sheet
- [ ] Income Statement (P&L)
- [ ] Account Ledger (with running balance)
- [ ] Dimensional Report (slice by any dimension)

### Simulation Engine

- [ ] Historical data aggregation
- [ ] Baseline calculation
- [ ] Projection with adjustments
- [ ] Rayon parallel processing
- [ ] Result caching

### Report & Simulation API Endpoints

- [ ] `POST /budgets` + `GET /budgets`
- [ ] `GET /budgets/:id` (budget detail)
- [ ] `POST /budgets/:id/lines` + `GET /budgets/:id/lines`
- [ ] `POST /budgets/:id/lock` (lock budget)
- [ ] `GET /budgets/:id/vs-actual` (budget vs actual comparison)
- [ ] `GET /reports/trial-balance`
- [ ] `GET /reports/balance-sheet`
- [ ] `GET /reports/income-statement`
- [ ] `GET /reports/account-ledger/:account_id`
- [ ] `GET /reports/dimensional`
- [ ] `GET /reports/budget-vs-actual`
- [ ] `POST /simulation/project`

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

- Reports match expected output (verified by accountant if possible)
- Simulation produces reasonable projections
- Performance acceptable (<2s for 12-month simulation)
- 50+ tests passing

---

## Phase 5: Attachments & API Polish (Week 14-15)

> **RESEARCH REQUIRED:**
>
> - Cloudflare R2: `Cloudflare R2 S3 compatible Rust SDK presigned URL`
> - File upload: `Axum multipart file upload example`
> - OpenAPI generation: `Rust OpenAPI utoipa Axum 0.8 swagger`
> - Load testing: `k6 load testing REST API tutorial`

### Attachments

- [ ] File upload to Cloudflare R2
- [ ] Presigned URL generation
- [ ] Link attachments to transactions
- [ ] File type validation
- [ ] Size limits

### Attachment API Endpoints

- [ ] `POST /attachments/upload` (get presigned URL)
- [ ] `POST /attachments` (confirm upload, link to transaction)
- [ ] `GET /attachments/:id` (get download URL)
- [ ] `DELETE /attachments/:id`
- [ ] `GET /transactions/:id/attachments`

### Live Exchange Rates (Frankfurter Integration)

> **RESEARCH REQUIRED:**
>
> - Frankfurter API docs: `https://frankfurter.dev/`
> - Self-hosting: `docker run -d -p 8080:8080 lineofflight/frankfurter`
> - Rust client (optional): `frankfurte-rs` crate atau bikin sendiri pake `reqwest`

- [ ] Setup Frankfurter Docker container (self-hosted)
- [ ] Create `ExchangeRateFetcher` service
- [ ] Implement daily rate fetch (cron job)
- [ ] Store fetched rates ke `exchange_rates` table
- [ ] Config: pilih source (mock/frankfurter/manual)
- [ ] Fallback ke last known rate kalau API down
- [ ] API endpoint: `POST /exchange-rates/fetch` (manual trigger)
- [ ] API endpoint: `POST /exchange-rates/bulk` (bulk import rates)

### Dashboard Analytics

- [ ] Dashboard API: `GET /api/v1/dashboard/metrics`
  - Cash position, burn rate, runway days
  - Pending approvals count & amount
  - Budget status & utilization
  - Top expenses by department
  - Currency exposure
  - Cash flow chart data (weekly inflow/outflow)
  - Utilization chart data (budgeted vs actual by department)
- [ ] Activity Log API: `GET /api/v1/dashboard/recent-activity`
  - Transaction events (created, submitted, approved, rejected, posted, voided)
  - Budget events (created, updated, locked)
  - User events (invited, role changed)
  - Pagination with cursor
- [ ] Budget Summary API: `GET /api/v1/dashboard/budget-vs-actual`
  - Summary for dashboard widget
  - Period-based comparison
- [ ] Frontend Integration: Replace mock data with real-time API responses

### API Polish

- [ ] OpenAPI/Swagger spec generation (utoipa)
- [ ] API documentation
- [ ] Consistent error format across all endpoints
- [ ] Rate limiting
- [ ] Request logging

### Load & Security Testing

- [ ] Setup k6 or similar
- [ ] Test concurrent transaction creation
- [ ] Test report generation under load
- [ ] SQL injection attempts
- [ ] Cross-tenant access attempts
- [ ] Invalid JWT handling

**Deliverable:** Production-ready API with comprehensive test coverage.

**Exit Criteria:**

- All API endpoints documented
- Load test: 100 concurrent users, <500ms p95
- Security tests passing
- 200+ total integration tests

---

## Phase 6: Frontend Foundation (Week 16-17)

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

### Auth Pages

- [x] Login
- [x] Register
- [x] Forgot password
- [x] Organization selector
- [x] Logout (UI ready, mock API connected - needs real backend integration)
- [x] Email verification page (`/verify-email?token=xxx`)
- [x] Resend verification email UI (in register success / login blocked state)

### Core Layout

- [x] Sidebar navigation
- [x] Header with user menu
- [x] Responsive design

### Organization UI

> **DB Tables:** `organizations`, `organization_users` > **DB Fields:** `base_currency`, `timezone`, `role`, `approval_limit`

- [x] Organization Settings Page
  - [x] Update base currency (`organizations.base_currency`)
  - [x] Update timezone (`organizations.timezone`)
  - [x] View subscription tier & status
- [x] User/Team Management Page
  - [x] List organization users (`organization_users`)
  - [x] Invite new user (email + role)
  - [x] Update user role (`user_role` enum: viewer, accountant, approver, admin, owner)
  - [x] Set approval limit for approvers (`approval_limit` field)
  - [x] Remove user from organization

**Deliverable:** Frontend skeleton with auth working.

---

## Phase 7: Frontend Features (Week 18-20)

> **RESEARCH REQUIRED:**
>
> - React Hook Form: `React Hook Form Zod validation Next.js 16`
> - Data tables: `TanStack Table React 19 sorting filtering pagination`
> - Charts: `Recharts React 19 bar chart line chart example`
> - Optimistic updates: `TanStack Query v5 optimistic update mutation`

### Master Data UI

> **DB Tables:** `chart_of_accounts`, `fiscal_years`, `fiscal_periods`, `dimension_types`, `dimension_values`, `exchange_rates`

- [x] Chart of Accounts management
  - [x] List accounts with hierarchy (parent_id)
  - [x] Account type/subtype display
  - [x] Create new account (`POST /accounts`)
  - [x] Edit account (`PATCH /accounts/:id`)
  - [x] Delete account (`DELETE /accounts/:id`)
  - [ ] Toggle account active status
- [x] Fiscal period management
  - [x] List fiscal years with nested periods
  - [x] Period status badges (OPEN/SOFT_CLOSE/CLOSED)
  - [x] Change period status (`PATCH /fiscal-periods/:id/status`)
  - [x] Create fiscal year with auto-generated periods (`POST /fiscal-years`) (Mocked)
- [x] Dimension management
  - [x] List dimension types with nested values
  - [x] Add dimension value (`POST /dimensions/:typeId/values`)
  - [ ] Create dimension type (`POST /dimension-types`)
  - [ ] Edit dimension value
  - [ ] Toggle dimension active status
- [x] Exchange rate management
  - [x] List exchange rates
  - [x] Add exchange rate (`POST /exchange-rates`)
  - [ ] Bulk import exchange rates

### Transaction UI

> **DB Tables:** `transactions`, `ledger_entries`, `entry_dimensions`

- [x] Transaction list with filters
  - [x] Filter by status (draft/pending/approved/posted/voided)
  - [x] Filter by date range
  - [x] Filter by transaction type
  - [x] Filter by dimension (department/project)
- [x] Transaction entry form
  - [x] Multi-line journal entry (debit/credit)
  - [x] Account selector
  - [x] Dimension assignment per entry (`entry_dimensions`)
  - [ ] Multi-currency support (source_currency, exchange_rate)
  - [ ] Attachment upload
- [x] Approval queue
  - [x] List pending transactions
  - [x] Approve/Reject actions
  - [ ] Bulk approve
- [x] Transaction detail
  - [x] View entries with debit/credit
  - [ ] View dimension assignments
  - [ ] View attachments
  - [ ] Audit trail (submitted_by, approved_by, posted_by timestamps)

### Dashboard

> **Computed from:** `ledger_entries`, `budgets`, `budget_lines`

- [x] Key metrics
  - [x] Cash position (sum of cash accounts)
  - [x] Burn rate (daily/monthly)
  - [x] Runway days
  - [x] Pending approvals count
- [x] Budget vs actual
  - [x] Department budget cards
  - [x] Progress bars (actual/budget)
  - [x] Variance highlighting (favorable/unfavorable)
- [x] Charts (Recharts)
  - [x] Expense trend chart
  - [x] Cash flow chart
  - [x] Budget utilization by department
- [ ] Recent Activity Widget
  - [ ] Feed-style list of latest transaction & budget actions
  - [ ] Uses `GET /api/v1/dashboard/recent-activity` endpoint
  - [ ] Activity type icons (created, approved, posted, voided, etc.)
  - [ ] Relative timestamps ("2 hours ago")
  - [ ] Click to navigate to transaction/budget detail

### Reports UI

> **Computed from:** `ledger_entries`, `chart_of_accounts`, `entry_dimensions`

- [x] Report viewer
  - [x] Trial Balance
  - [x] Balance Sheet
  - [x] Income Statement (P&L)
- [x] Export functionality
  - [x] CSV export
  - [x] PDF export
- [x] **Account Ledger View**
  - [x] Select account from dropdown (via Account List)
  - [x] Show all entries for account (`ledger_entries.account_id`)
  - [x] Running balance column (`account_current_balance`)
  - [x] Date range filter (Mocked)
  - [x] Mock: `GET /accounts/:id/ledger`
- [x] **Dimensional Reports UI** (MISSING - DB Ready)
  - [x] Filter by dimension type (Department/Project/Cost Center)
  - [x] Filter by dimension value (Department/Project/Cost Center)
  - [x] Group expenses by dimension (Chart & Table)
  - [x] Compare across dimensions
  - [x] Mock: `GET /reports/dimensional`

### Advanced Features

> **DB Tables:** `budgets`, `budget_lines`, `budget_line_dimensions`

- [x] **Fiscal Year Creation UI** (Mocked - Auto-generate periods)
  - [x] Form: name, start_date, end_date
  - [x] Auto-generate 12 monthly periods
  - [x] Option for adjustment period (period 13)
  - [x] Mock: `POST /fiscal-years`
- [x] **Budget Management UI** (Mocked - Create, Detail, Lines)
  - [x] Create budget (`budgets` table)
  - [x] Add budget lines per account/period (`budget_lines`)
  - [x] Assign dimensions to budget lines (`budget_line_dimensions`)
  - [x] Lock/unlock budget
  - [x] Mock: `POST /budgets`, `POST /budgets/:id/lines`
- [ ] Simulation/Forecasting UI (Phase 4 backend dependency)
  - [ ] Historical data selection
  - [ ] Adjustment parameters
  - [ ] Projection results

### Mock Handlers Needed

Add these to `frontend/src/mocks/handlers.ts`:

```typescript
// Account Ledger
GET /api/v1/accounts/:id/ledger → { entries: [...], running_balance }

// Dimensional Report
GET /api/v1/reports/dimensional → { data: [...], grouped_by_dimension }

// Fiscal Year Creation
POST /api/v1/fiscal-years → { id, name, periods: [...] }

// Organization Settings
PATCH /api/v1/organizations/:id → { success: true }

// Team Management
GET /api/v1/organizations/:id/users → { users: [...] }
POST /api/v1/organizations/:id/users → { user_id, role }
PATCH /api/v1/organizations/:id/users/:userId → { role, approval_limit }
DELETE /api/v1/organizations/:id/users/:userId → { success: true }

// Budget Management
POST /api/v1/budgets → { id, name, fiscal_year_id }
POST /api/v1/budgets/:id/lines → { id, account_id, amount }
```

**Deliverable:** Complete frontend application.

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

---

## Timeline Summary (REVISED - Vertical Slice)

| Phase                        | Duration | Focus                                   | End Date     |
| ---------------------------- | -------- | --------------------------------------- | ------------ |
| Phase 0: Foundation          | 2 weeks  | Infra + DB + **Seeders**                | Jan 21, 2026 |
| Phase 1: Auth & Org          | 2 weeks  | **Auth FIRST** + API                    | Feb 4, 2026  |
| Phase 2: Ledger Core         | 4 weeks  | Double-entry + Multi-currency + **API** | Mar 4, 2026  |
| Phase 3: Workflow            | 2 weeks  | Transaction lifecycle + **API**         | Mar 18, 2026 |
| Phase 4: Reports             | 3 weeks  | Financial reports, simulation + **API** | Apr 8, 2026  |
| Phase 5: Attachments         | 2 weeks  | File storage + API polish               | Apr 22, 2026 |
| Phase 6: Frontend Foundation | 2 weeks  | Next.js setup, auth UI                  | May 6, 2026  |
| Phase 7: Frontend Features   | 3 weeks  | Full UI                                 | May 27, 2026 |
| Phase 8: Polish & Launch     | 2 weeks  | Testing, deploy                         | Jun 10, 2026 |

**Total: 22 weeks (~5.5 months)**

**Backend complete: Week 15 (Apr 22, 2026)**
**Frontend starts: Week 16**
**GO LIVE: June 10, 2026**

---

## Key Changes from Original Roadmap

### 1. Auth BEFORE Ledger (Fatal Dependency Fix)

- Phase 1 sekarang Auth & Organization
- Transaction punya `created_by` dan `organization_id` dari hari pertama
- RLS bisa ditest dengan real users

### 2. Vertical Slice (API Integration Fix)

- Setiap phase langsung bikin API endpoint
- Gak ada "API Phase" terpisah di akhir
- Bisa test via Postman/curl real-time

### 3. Seeders di Phase 0 (Forex Data Fix)

- `exchange_rates` di-seed dengan mock data
- Multi-currency logic bisa langsung dites
- Gak perlu nunggu live API

### 4. Rounding Strategy (New)

- Research Banker's Rounding
- Handle split difference explicitly
- $100/3 = $33.34 + $33.33 + $33.33

---

## Critical Success Factors

### Phase 2 is Make-or-Break

Spend extra time here if needed. Do NOT rush.

Checklist before leaving Phase 2:

- [x] Balance NEVER drifts (stress tested)
- [x] Concurrent transactions handled correctly
- [x] Multi-currency conversion accurate
- [x] Rounding differences handled correctly
- [x] Fiscal period rules enforced
- [x] All API endpoints working
- [x] All edge cases covered with tests

### Test Coverage Requirements

| Phase         | Minimum Tests           |
| ------------- | ----------------------- |
| Phase 1       | 50+ tests               |
| Phase 2       | 150+ tests              |
| Phase 3       | 50+ tests               |
| Phase 4       | 50+ tests               |
| Phase 5       | 50+ tests (integration) |
| Total Backend | 350+ tests              |

### No Shortcuts

- No skipping tests to "save time"
- No "we'll fix it later" for ledger bugs
- No frontend until backend is solid
- No launch until load tested

---

## Risk Mitigation

| Risk                      | Mitigation                                      |
| ------------------------- | ----------------------------------------------- |
| Ledger bugs in production | Extensive testing in Phase 2, no shortcuts      |
| Balance drift             | Stress test with 10,000+ transactions           |
| Race conditions           | Test concurrent access explicitly               |
| Currency errors           | Use Decimal everywhere, never float             |
| Rounding errors           | Banker's Rounding + explicit remainder handling |
| Auth dependency           | Auth moved to Phase 1 (before Ledger)           |
| API integration hell      | Vertical slice - API with each feature          |
| Scope creep               | No new features until MVP launch                |
| Burnout                   | Realistic timeline, take breaks                 |

---

## Definition of Done (Per Phase)

### Phase 1 Done When:

- Auth flow works end-to-end
- RLS isolates tenants
- All API endpoints working
- 50+ tests passing

### Phase 2 Done When:

- All ledger tests pass
- Stress test passes (1000 concurrent transactions)
- API endpoints working via Postman
- 150+ tests passing
- No known bugs

### Phase 5 Done When:

- All API tests pass
- Load test passes (100 concurrent users)
- Security tests pass
- API docs complete

### Phase 8 Done When:

- E2E tests pass
- Production deployed
- Monitoring active
- First customer onboarded
