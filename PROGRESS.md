# Zeltra - Progress Tracker

Live status untuk sync antara Backend & Frontend.

---

## Current State

|                    | Backend                                                 | Frontend                                          |
| ------------------ | ------------------------------------------------------- | ------------------------------------------------- |
| **Current Phase**  | 2 (Done)                                                | 1 (Done)                                          |
| **Last Task Done** | Phase 2 - Ledger Core Complete (229 tests passing)      | Phase 7 - Dashboard & Reports (Wire up mock data) |
| **Next Task**      | Phase 3 - Transaction Workflow                          | Phase 8: Final Polish & Simulation UI             |

**Last Updated:** 2026-01-09

---

## Legend

- ⬜ Not Started
- 🟡 In Progress
- ✅ Done
- ❌ Blocked

---

## Phase Status

| Phase            | Backend | Frontend | Notes                                          |
| ---------------- | ------- | -------- | ---------------------------------------------- |
| 0: Foundation    | ✅      | ⬜       | BE workspace setup done                        |
| 1: Auth          | ✅      | ✅       | BE Auth API complete                           |
| 2: Ledger        | ✅      | -        | 229 tests, 1000+ concurrent stress test passed |
| 3: Workflow      | ⬜      | -        |                                                |
| 4: Reports       | ⬜      | -        |                                                |
| 5: Polish        | ⬜      | -        |                                                |
| 6: FE Foundation | -       | ✅       | Auth & Org Management complete                 |
| 7: FE Features   | -       | ✅       | Ledger, Reports, Budgets, & Dashboard complete |
| 8: Launch        | ⬜      | ⬜       |                                                |

---

## Phase 0 Tasks (Backend)

| Task                      | Status | Notes                                       |
| ------------------------- | ------ | ------------------------------------------- |
| Setup Rust workspace      | ✅     | Cargo workspace with 4 crates               |
| Create crate structure    | ✅     | api, core, db, shared                       |
| Setup rust-toolchain.toml | ✅     | Rust 1.92                                   |
| Setup .cargo/config.toml  | ✅     | Float arithmetic denied                     |
| Core domain types         | ✅     | Money, IDs, Pagination                      |
| Ledger types              | ✅     | Entry, Transaction, Balance                 |
| Currency types            | ✅     | Exchange rate, conversion                   |
| Fiscal types              | ✅     | FiscalYear, FiscalPeriod                    |
| Budget types              | ✅     | Variance calculations                       |
| Simulation types          | ✅     | Scenario, Engine                            |
| Health endpoint           | ✅     | GET /api/v1/health                          |
| Docker compose            | ✅     | PostgreSQL 16                               |
| Config files              | ✅     | default.toml, development.toml              |
| Database migrations       | ✅     | All tables, views, triggers, RLS, seed data |
| SeaORM entity generation  | ✅     | 21 entities generated from schema           |
| Seeder CLI                | ✅     | Exchange rates, dimensions seeded           |
| Integration tests setup   | ⬜     |                                             |

---

## Phase 1 Tasks (Backend - Auth)

| Task                        | Status | Notes                                       |
| --------------------------- | ------ | ------------------------------------------- |
| JWT Service                 | ✅     | Access & refresh token generation           |
| Password hashing            | ✅     | Argon2id with secure defaults               |
| User repository             | ✅     | CRUD, find by email, get organizations      |
| Organization repository     | ✅     | CRUD, membership management, role checks    |
| Session repository          | ✅     | Create, revoke, cleanup sessions            |
| Email verification repo     | ✅     | Create/verify tokens, invalidate, cleanup   |
| Email service               | ✅     | SMTP via lettre, verification emails        |
| Auth middleware             | ✅     | JWT validation, claims extraction           |
| Auth routes                 | ✅     | Login, register, refresh, logout            |
| Email verification routes   | ✅     | verify-email, resend-verification           |
| Organization routes         | ✅     | Create, get, list users, add user           |
| RLS context per request     | ✅     | `RlsConnection` wrapper, `SET LOCAL` helper |
| Test cross-tenant isolation | ✅     | 3 integration tests, non-superuser role     |

---

## Phase 1 Tasks (Backend - Subscription & Tier)

| Task                        | Status | Notes                                   |
| --------------------------- | ------ | --------------------------------------- |
| Seed tier_limits table      | ✅     | Already seeded in initial migration     |
| Set default subscription    | ✅     | starter tier, trialing status           |
| check_tier_limit() function | ✅     | SubscriptionRepository::check_limit()   |
| has_feature() function      | ✅     | SubscriptionRepository::has_feature()   |
| organization_usage tracking | ✅     | get_or_create, increment counters       |
| Trial expiry check          | ✅     | is_trial_expired()                      |
| Tier upgrade function       | ✅     | upgrade_tier()                          |
| Test cross-tenant isolation | ✅     | 3 integration tests, non-superuser role |

---

## Phase 2 Tasks (Backend - Ledger Core)

| Task                                    | Status | Notes                                    |
| --------------------------------------- | ------ | ---------------------------------------- |
| Fiscal years and periods CRUD           | ✅     | FiscalYearRepository                     |
| Chart of accounts CRUD                  | ✅     | AccountRepository                        |
| Dimension types and values CRUD         | ✅     | DimensionRepository                      |
| Exchange rates CRUD                     | ✅     | ExchangeRateRepository                   |
| Transaction creation (single currency)  | ✅     | TransactionRepository                    |
| Validate debit = credit                 | ✅     | LedgerService + DB trigger               |
| Validate minimum 2 entries              | ✅     | Property test 13                         |
| Validate no zero/negative amounts       | ✅     | Property test 13                         |
| Account version increment               | ✅     | DB trigger                               |
| Running balance tracking                | ✅     | DB trigger (bug fixed!)                  |
| Concurrent transaction stress test      | ✅     | 1000+ transactions, no drift             |
| Exchange rate lookup + triangulation    | ✅     | Property test 8                          |
| Currency conversion (Banker's Rounding) | ✅     | Property test 6                          |
| Allocation (Largest Remainder Method)   | ✅     | Property test 7                          |
| Dimensional accounting                  | ✅     | entry_dimensions table                   |
| Fiscal period validation                | ✅     | Property test 9, DB trigger              |
| Transaction API endpoints               | ✅     | All CRUD endpoints                       |
| Master data API endpoints               | ✅     | All endpoints                            |
| Database trigger tests                  | ✅     | 8 integration tests                      |
| **Total Tests**                         | ✅     | **229 tests passing** (target was 150+)  |

---

## API Endpoints Status

Frontend cek di sini untuk tau endpoint mana yang udah ready.

### Health

| Endpoint           | Status | Notes                    |
| ------------------ | ------ | ------------------------ |
| GET /api/v1/health | ✅     | Returns status & version |

### Auth

| Endpoint                              | Status | Notes                              |
| ------------------------------------- | ------ | ---------------------------------- |
| POST /api/v1/auth/register            | ✅     | Creates user, sends verification   |
| POST /api/v1/auth/login               | ✅     | Returns tokens + user info         |
| POST /api/v1/auth/refresh             | ✅     | Returns new access token           |
| POST /api/v1/auth/logout              | ✅     | Revokes session, invalidates token |
| POST /api/v1/auth/verify-email        | ✅     | Verify email with token            |
| POST /api/v1/auth/resend-verification | ✅     | Resend verification email          |

### Organizations

| Endpoint                                    | Status | Notes                                    |
| ------------------------------------------- | ------ | ---------------------------------------- |
| POST /api/v1/organizations                  | ✅     | Create org, user as owner                |
| GET /api/v1/organizations/:id               | ✅     | Get org details                          |
| GET /api/v1/organizations/:id/users         | ✅     | List org members                         |
| POST /api/v1/organizations/:id/users        | ✅     | Add user to org (admin+)                 |
| PATCH /api/v1/organizations/:id             | ✅     | Update org settings (name/currency/tz)   |
| PATCH /api/v1/organizations/:id/users/:id   | ✅     | Update user role/approval_limit (admin+) |
| DELETE /api/v1/organizations/:id/users/:id  | ✅     | Remove user from org (admin+, hierarchy) |

### Accounts

| Endpoint                   | Status | Notes                                |
| -------------------------- | ------ | ------------------------------------ |
| GET /accounts              | ✅     | Real API - list with balances        |
| POST /accounts             | ✅     | Real API - create account            |
| GET /accounts/:id          | ✅     | Real API - get account detail        |
| PUT /accounts/:id          | ✅     | Real API - update account            |
| DELETE /accounts/:id       | ✅     | Real API - soft delete               |
| GET /accounts/:id/balance  | ✅     | Real API - balance at date           |
| GET /accounts/:id/ledger   | ✅     | Real API - ledger entries with range |

### Transactions

| Endpoint                       | Status | Notes                                    |
| ------------------------------ | ------ | ---------------------------------------- |
| GET /transactions              | ✅     | Real API - list with filters             |
| POST /transactions             | ✅     | Real API - create draft                  |
| GET /transactions/:id          | ✅     | Real API - detail with entries           |
| PATCH /transactions/:id        | ✅     | Real API - update draft only             |
| DELETE /transactions/:id       | ✅     | Real API - delete draft only             |
| POST /transactions/:id/submit  | ⬜     | Phase 3 - Workflow                       |
| POST /transactions/:id/approve | ⬜     | Phase 3 - Workflow                       |
| POST /transactions/:id/reject  | ⬜     | Phase 3 - Workflow                       |
| POST /transactions/:id/post    | ⬜     | Phase 3 - Workflow                       |
| POST /transactions/:id/void    | ⬜     | Phase 3 - Workflow                       |

### Master Data

| Endpoint                         | Status | Notes                                |
| -------------------------------- | ------ | ------------------------------------ |
| GET /fiscal-years                | ✅     | Real API - list with nested periods  |
| POST /fiscal-years               | ✅     | Real API - create with auto-periods  |
| GET /fiscal-periods              | ✅     | Real API - list periods              |
| PATCH /fiscal-periods/:id/status | ✅     | Real API - update status             |
| GET /dimension-types             | ✅     | Real API - list types                |
| POST /dimension-types            | ✅     | Real API - create type               |
| GET /dimension-values            | ✅     | Real API - list with filters         |
| POST /dimension-values           | ✅     | Real API - create value              |
| GET /exchange-rates              | ✅     | Real API - get rate for pair/date    |
| POST /exchange-rates             | ✅     | Real API - create/update rate        |
| GET /currencies                  | ✅     | Real API - list all currencies       |

### Reports

| Endpoint                      | Status | Notes  |
| ----------------------------- | ------ | ------ |
| GET /reports/trial-balance    | ✅     | Mocked |
| GET /reports/balance-sheet    | ✅     | Mocked |
| GET /reports/income-statement | ✅     | Mocked |
| GET /reports/dimensional      | ✅     | Mocked |
| GET /budgets/:id/vs-actual    | ✅     | Mocked |

### Budgets

| Endpoint                  | Status | Notes                 |
| ------------------------- | ------ | --------------------- |
| GET /budgets              | ✅     | Mocked                |
| POST /budgets             | ✅     | Mocked (Create)       |
| GET /budgets/:id          | ✅     | Mocked (Detail+Lines) |
| POST /budgets/:id/lines   | ✅     | Mocked (Add Line)     |
| PATCH /budgets/:id/status | ✅     | Mocked (Lock/Unlock)  |

### Dashboard

| Endpoint                       | Status | Notes                           |
| ------------------------------ | ------ | ------------------------------- |
| GET /dashboard/metrics         | ✅     | Mocked (Cash Flow, Utilization) |
| GET /dashboard/recent-activity | ⬜     |                                 |

### Simulation

| Endpoint             | Status | Notes |
| -------------------- | ------ | ----- |
| POST /simulation/run | ⬜     |       |

---

## Blockers

| Issue | Reporter | Status | Resolution |
| ----- | -------- | ------ | ---------- |
| -     | -        | -      | -          |

---

## Notes

- **Frontend gak perlu nunggu Backend** - Pake MSW mock API
- Backend update status endpoint setelah implement
- Frontend cek status, kalau ⬜ pake mock, kalau ✅ test real API
- Gradually replace mock dengan real API pas Backend catch up

---

## Frontend-Backend Schema Sync Analysis

**Last Verified:** 2026-01-08

### Compatibility Status: ✅ COMPATIBLE

Frontend mock structures align with database schema. Notes for API implementation:

| Area           | Status | Notes                                                        |
| -------------- | ------ | ------------------------------------------------------------ |
| Accounts       | ⚠️     | `balance` is computed field (from ledger_entries), not in DB |
| Transactions   | ⚠️     | `entries[]` needs JOIN with `ledger_entries` table           |
| Fiscal Years   | ⚠️     | `periods[]` needs JOIN with `fiscal_periods` table           |
| Dimensions     | ⚠️     | `values[]` needs JOIN with `dimension_values` table          |
| Exchange Rates | ⚠️     | Field name: DB `effective_date` → API `date`                 |
| Enums          | ⚠️     | DB enums need lowercase string conversion for JSON           |

### API Response Mapping Required

```
DB account_type ENUM → lowercase string ("asset", "liability", "equity", "revenue", "expense")
DB transaction_status ENUM → lowercase string ("draft", "pending", "approved", "posted", "voided")
DB transaction_type ENUM → lowercase string ("journal", "expense", "revenue", "transfer")
```
