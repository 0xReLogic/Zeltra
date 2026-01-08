# Zeltra - Progress Tracker

Live status untuk sync antara Backend & Frontend.

---

## Current State

|                    | Backend                 | Frontend                                           |
| ------------------ | ----------------------- | -------------------------------------------------- |
| **Current Phase**  | 1                       | 1 (Done)                                           |
| **Last Task Done** | Auth API implementation | Phase 7: Advanced UI Features (Charts, Budgets) ✅ |
| **Next Task**      | Integration tests       | Phase 8: Final Polish & Simulation UI              |

**Last Updated:** 2026-01-08

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
| 2: Ledger        | ⬜      | -        |                                                |
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

| Endpoint                             | Status | Notes                     |
| ------------------------------------ | ------ | ------------------------- |
| POST /api/v1/organizations           | ✅     | Create org, user as owner |
| GET /api/v1/organizations/:id        | ✅     | Get org details           |
| GET /api/v1/organizations/:id/users  | ✅     | List org members          |
| POST /api/v1/organizations/:id/users | ✅     | Add user to org (admin+)  |
| PATCH /api/v1/organizations/:id      | ✅     | Update org settings       |

### Accounts

| Endpoint                  | Status | Notes  |
| ------------------------- | ------ | ------ |
| GET /accounts             | ✅     | Mocked |
| POST /accounts            | ✅     | Mocked |
| GET /accounts/:id/balance | ✅     | Mocked |
| GET /accounts/:id/ledger  | ✅     | Mocked |

### Transactions

| Endpoint                       | Status | Notes  |
| ------------------------------ | ------ | ------ |
| GET /transactions              | ✅     | Mocked |
| POST /transactions             | ✅     | Mocked |
| GET /transactions/:id          | ✅     | Mocked |
| POST /transactions/:id/submit  | ✅     | Mocked |
| POST /transactions/:id/approve | ✅     | Mocked |
| POST /transactions/:id/reject  | ✅     | Mocked |
| POST /transactions/:id/post    | ✅     | Mocked |
| POST /transactions/:id/void    | ✅     | Mocked |

### Master Data

| Endpoint                         | Status | Notes                                  |
| -------------------------------- | ------ | -------------------------------------- |
| GET /fiscal-years                | ✅     | Mocked                                 |
| POST /fiscal-years               | ✅     | Mocked (Auto-generate incl. Period 13) |
| GET /fiscal-periods              | ✅     | Mocked                                 |
| PATCH /fiscal-periods/:id/status | ✅     | Mocked                                 |
| GET /dimension-types             | ✅     | Mocked                                 |
| GET /dimension-values            | ✅     | Mocked                                 |
| GET /exchange-rates              | ✅     | Mocked                                 |
| POST /exchange-rates             | ✅     | Mocked                                 |

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
