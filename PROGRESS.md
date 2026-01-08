# Zeltra - Progress Tracker

Live status untuk sync antara Backend & Frontend.

---

## Current State

|                    | Backend                 | Frontend                                  |
| ------------------ | ----------------------- | ----------------------------------------- |
| **Current Phase**  | 0                       | 1 (Done)                                  |
| **Last Task Done** | Seeder CLI complete     | Phase 7: Master Data & Export Features ✅ |
| **Next Task**      | Integration tests setup | Phase 8: Backend Development (Supabase)   |

**Last Updated:** 2026-01-08

---

## Legend

- ⬜ Not Started
- 🟡 In Progress
- ✅ Done
- ❌ Blocked

---

## Phase Status

| Phase            | Backend | Frontend | Notes                   |
| ---------------- | ------- | -------- | ----------------------- |
| 0: Foundation    | 🟡      | ⬜       | BE workspace setup done |
| 1: Auth          | ⬜      | ✅       | FE mocked               |
| 2: Ledger        | ⬜      | -        |                         |
| 3: Workflow      | ⬜      | -        |                         |
| 4: Reports       | ⬜      | -        |                         |
| 5: Polish        | ⬜      | -        |                         |
| 6: FE Foundation | -       | ⬜       |                         |
| 7: FE Features   | -       | ✅       | Accounts, Reports, MD   |
| 8: Launch        | ⬜      | ⬜       |                         |

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

## API Endpoints Status

Frontend cek di sini untuk tau endpoint mana yang udah ready.

### Health

| Endpoint           | Status | Notes                    |
| ------------------ | ------ | ------------------------ |
| GET /api/v1/health | ✅     | Returns status & version |

### Auth

| Endpoint            | Status | Notes  |
| ------------------- | ------ | ------ |
| POST /auth/register | ⬜     |        |
| POST /auth/login    | ✅     | Mocked |
| POST /auth/refresh  | ⬜     |        |
| POST /auth/logout   | ⬜     |        |

### Organizations

| Endpoint                      | Status | Notes |
| ----------------------------- | ------ | ----- |
| GET /organizations            | ⬜     |       |
| POST /organizations           | ⬜     |       |
| POST /organizations/:id/users | ⬜     |       |

### Accounts

| Endpoint                  | Status | Notes  |
| ------------------------- | ------ | ------ |
| GET /accounts             | ✅     | Mocked |
| POST /accounts            | ⬜     |        |
| GET /accounts/:id/balance | ⬜     |        |
| GET /accounts/:id/ledger  | ⬜     |        |

### Transactions

| Endpoint                       | Status | Notes  |
| ------------------------------ | ------ | ------ |
| GET /transactions              | ✅     | Mocked |
| POST /transactions             | ✅     | Mocked |
| GET /transactions/:id          | ✅     | Mocked |
| POST /transactions/:id/submit  | ⬜     |        |
| POST /transactions/:id/approve | ✅     | Mocked |
| POST /transactions/:id/reject  | ✅     | Mocked |
| POST /transactions/:id/post    | ⬜     |        |
| POST /transactions/:id/void    | ⬜     |        |

### Master Data

| Endpoint                         | Status | Notes  |
| -------------------------------- | ------ | ------ |
| GET /fiscal-years                | ✅     | Mocked |
| POST /fiscal-years               | ⬜     |        |
| GET /fiscal-periods              | ✅     | Mocked |
| PATCH /fiscal-periods/:id/status | ✅     | Mocked |
| GET /dimension-types             | ✅     | Mocked |
| GET /dimension-values            | ✅     | Mocked |
| GET /exchange-rates              | ✅     | Mocked |
| POST /exchange-rates             | ✅     | Mocked |

### Reports

| Endpoint                      | Status | Notes  |
| ----------------------------- | ------ | ------ |
| GET /reports/trial-balance    | ✅     | Mocked |
| GET /reports/balance-sheet    | ✅     | Mocked |
| GET /reports/income-statement | ✅     | Mocked |
| GET /reports/dimensional      | ⬜     |        |
| GET /budgets/:id/vs-actual    | ✅     | Mocked |

### Dashboard

| Endpoint                       | Status | Notes  |
| ------------------------------ | ------ | ------ |
| GET /dashboard/metrics         | ✅     | Mocked |
| GET /dashboard/recent-activity | ⬜     |        |

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
