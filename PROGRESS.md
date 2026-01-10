# Zeltra - Progress Tracker

Live status untuk sync antara Backend & Frontend.

---

## Current State

|                    | Backend                                    | Frontend                              |
| ------------------ | ------------------------------------------ | ------------------------------------- |
| **Current Phase**  | 4 (Reports & Simulation) ✅ COMPLETE       | 8 (Transaction Enhancements)          |
| **Last Task Done** | Phase 4 - All Tasks Complete (716 tests)   | Phase 8 - Transaction Enhancements ✅ |
| **Next Task**      | Phase 5 - TBD                              | Playwright E2E                        |

**Last Updated:** 2026-01-10

---

## Legend

- ⬜ Not Started
- 🟡 In Progress
- ✅ Done (Real API)
- ⚠️ Mocked (Frontend Only)
- ❌ Blocked

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

| Endpoint                                   | Status | Notes                                    |
| ------------------------------------------ | ------ | ---------------------------------------- |
| POST /api/v1/organizations                 | ✅     | Create org, user as owner                |
| GET /api/v1/organizations/:id              | ✅     | Get org details                          |
| GET /api/v1/organizations/:id/users        | ✅     | List org members                         |
| POST /api/v1/organizations/:id/users       | ✅     | Add user to org (admin+)                 |
| PATCH /api/v1/organizations/:id            | ✅     | Update org settings (name/currency/tz)   |
| PATCH /api/v1/organizations/:id/users/:id  | ✅     | Update user role/approval_limit (admin+) |
| DELETE /api/v1/organizations/:id/users/:id | ✅     | Remove user from org (admin+, hierarchy) |

### Accounts

| Endpoint                  | Status | Notes                                |
| ------------------------- | ------ | ------------------------------------ |
| GET /accounts             | ✅     | Real API - list with balances        |
| POST /accounts            | ✅     | Real API - create account            |
| GET /accounts/:id         | ✅     | Real API - get account detail        |
| PUT /accounts/:id         | ✅     | Real API - update account            |
| DELETE /accounts/:id      | ✅     | Real API - soft delete               |
| GET /accounts/:id/balance | ✅     | Real API - balance at date           |
| GET /accounts/:id/ledger  | ✅     | Real API - ledger entries with range |

### Transactions

| Endpoint                           | Status | Notes                          |
| ---------------------------------- | ------ | ------------------------------ |
| GET /transactions                  | ✅     | Real API - list with filters   |
| POST /transactions                 | ✅     | Real API - create draft        |
| GET /transactions/:id              | ✅     | Real API - detail with entries |
| PATCH /transactions/:id            | ✅     | Real API - update draft only   |
| DELETE /transactions/:id           | ✅     | Real API - delete draft only   |
| POST /transactions/:id/submit      | ✅     | Real API - draft → pending     |
| POST /transactions/:id/approve     | ✅     | Real API - pending → approved  |
| POST /transactions/:id/reject      | ✅     | Real API - pending → draft     |
| POST /transactions/:id/post        | ✅     | Real API - approved → posted   |
| POST /transactions/:id/void        | ✅     | Real API - posted → voided     |
| GET /transactions/pending          | ✅     | Real API - approval queue      |
| POST /transactions/bulk-approve    | ✅     | Real API - batch approval      |
| POST /transactions/:id/attachments | ⚠️     | Mocked - Upload file           |
| GET /transactions/:id/attachments  | ⚠️     | Mocked - List files            |

### Master Data

| Endpoint                           | Status | Notes                               |
| ---------------------------------- | ------ | ----------------------------------- |
| GET /fiscal-years                  | ✅     | Real API - list with nested periods |
| POST /fiscal-years                 | ✅     | Real API - create with auto-periods |
| GET /fiscal-periods                | ✅     | Real API - list periods             |
| PATCH /fiscal-periods/:id/status   | ✅     | Real API - update status            |
| GET /dimension-types               | ✅     | Real API - list types               |
| POST /dimension-types              | ✅     | Real API - create type              |
| GET /dimension-values              | ✅     | Real API - list with filters        |
| POST /dimension-values             | ✅     | Real API - create value             |
| GET /exchange-rates                | ✅     | Real API - get rate for pair/date   |
| POST /exchange-rates               | ✅     | Real API - create/update rate       |
| GET /currencies                    | ✅     | Real API - list all currencies      |
| PATCH /accounts/:id/status         | ⚠️     | Mocked (Needs BE)                   |
| POST /dimension-types              | ⚠️     | Mocked (Needs BE)                   |
| PATCH /dimension-values/:id        | ⚠️     | Mocked (Needs BE)                   |
| PATCH /dimension-values/:id/status | ⚠️     | Mocked (Needs BE)                   |
| POST /exchange-rates/bulk          | ⚠️     | Mocked (Needs BE)                   |

### Approval Rules

| Endpoint                                     | Status | Notes                           |
| -------------------------------------------- | ------ | ------------------------------- |
| GET /organizations/:id/approval-rules        | ✅     | Real API - list rules           |
| POST /organizations/:id/approval-rules       | ✅     | Real API - create rule (admin+) |
| GET /organizations/:id/approval-rules/:id    | ✅     | Real API - get rule detail      |
| PATCH /organizations/:id/approval-rules/:id  | ✅     | Real API - update rule (admin+) |
| DELETE /organizations/:id/approval-rules/:id | ✅     | Real API - soft delete (admin+) |

### Reports

| Endpoint                      | Status | Notes                                |
| ----------------------------- | ------ | ------------------------------------ |
| GET /reports/trial-balance    | ✅     | Real API - as_of, dimension filters  |
| GET /reports/balance-sheet    | ✅     | Real API - as_of date                |
| GET /reports/income-statement | ✅     | Real API - from/to, dimension filter |
| GET /reports/dimensional      | ✅     | Real API - group_by dimensions       |
| GET /budgets/:id/vs-actual    | ✅     | Real API - variance analysis         |

### Budgets

| Endpoint                  | Status | Notes                                |
| ------------------------- | ------ | ------------------------------------ |
| GET /budgets              | ✅     | Real API - list with summary         |
| POST /budgets             | ✅     | Real API - create budget             |
| GET /budgets/:id          | ✅     | Real API - detail with lines         |
| PUT /budgets/:id          | ✅     | Real API - update budget             |
| GET /budgets/:id/lines    | ✅     | Real API - list budget lines         |
| POST /budgets/:id/lines   | ✅     | Real API - bulk create lines         |
| POST /budgets/:id/lock    | ✅     | Real API - lock budget               |

### Dashboard

| Endpoint                       | Status | Notes                                |
| ------------------------------ | ------ | ------------------------------------ |
| GET /dashboard/metrics         | ✅     | Real API - cash, burn rate, runway   |
| GET /dashboard/recent-activity | ✅     | Real API - cursor pagination         |

### Simulation

| Endpoint             | Status | Notes                                |
| -------------------- | ------ | ------------------------------------ |
| POST /simulation/run | ✅     | Real API - projections with caching  |

---

## Blockers

| Issue        | Reporter | Status | Resolution                                                                           |
| ------------ | -------- | ------ | ------------------------------------------------------------------------------------ |
| Missing APIs | Frontend | 🟡     | Needs BE implementation: Toggle Account, Create Dim Type, Edit Dim Value, Bulk Rates |

---

## Notes

- **Frontend gak perlu nunggu Backend** - Pake MSW mock API
- Backend update status endpoint setelah implement
- Frontend cek status, kalau ⚠️ pake mock, kalau ✅ test real API
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
