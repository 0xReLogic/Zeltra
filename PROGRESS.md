# Zeltra - Progress Tracker

Live status untuk sync antara Backend & Frontend.

---

## Current State

|                    | Backend                                    | Frontend                              |
| ------------------ | ------------------------------------------ | ------------------------------------- |
| **Current Phase**  | 5 (API Polish & Attachments) ✅ COMPLETE   | 6-7 (Frontend Features) 🟡 IN PROGRESS |
| **Last Task Done** | Phase 5 - All Tasks Complete (773 tests)   | Auth + Org Real API Integration ✅    |
| **Next Task**      | Phase 6 - TBD                              | Verify remaining features with Real API |

**Last Updated:** 2026-01-13

---

## 🟡 Real API Integration Progress

**Date:** 2026-01-13

**✅ Done (Real API Verified):**
- Mock API dependencies removed (MSW disabled, MOCK_DATA deleted)
- Role type mismatch fixed (6 roles: owner, admin, approver, accountant, viewer, submitter)
- Organization creation UI added
- API client optimized (30s timeout, 401 refresh, proper error handling)
- Auth integration complete (login, register, logout, refresh, token expiration tracking)
- Organization CRUD (create, update, list)
- User/Team management (invite, update role, remove)
- E2E tests updated for real API
- OpenAPI types generated from `contracts/openapi.yaml`

**⚠️ Need Verification (UI done, Real API unverified):**
- Master Data (accounts, fiscal periods, dimensions, exchange rates)
- Transactions (CRUD + workflow: submit/approve/reject/post/void)
- Dashboard (metrics, cash flow, recent activity)
- Reports (trial balance, balance sheet, income statement, dimensional)
- Budgets (CRUD + lines + lock)
- Simulation
- Attachments

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

| Endpoint               | Status | Notes                    |
| ---------------------- | ------ | ------------------------ |
| GET /api/v1/health     | ✅     | Returns status & version |

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
| GET /api/v1/accounts      | ✅     | Real API - list with balances        |
| POST /api/v1/accounts     | ✅     | Real API - create account            |
| GET /api/v1/accounts/:id  | ✅     | Real API - get account detail        |
| PUT /api/v1/accounts/:id  | ✅     | Real API - update account            |
| DELETE /api/v1/accounts/:id | ✅     | Real API - soft delete               |
| GET /api/v1/accounts/:id/balance | ✅     | Real API - balance at date           |
| GET /api/v1/accounts/:id/ledger  | ✅     | Real API - ledger entries with range |

### Transactions

| Endpoint                           | Status | Notes                          |
| ---------------------------------- | ------ | ------------------------------ |
| GET /api/v1/transactions           | ✅     | Real API - list with filters   |
| POST /api/v1/transactions          | ✅     | Real API - create draft        |
| GET /api/v1/transactions/:id       | ✅     | Real API - detail with entries |
| PATCH /api/v1/transactions/:id     | ✅     | Real API - update draft only   |
| DELETE /api/v1/transactions/:id    | ✅     | Real API - delete draft only   |
| POST /api/v1/transactions/:id/submit      | ✅     | Real API - draft → pending     |
| POST /api/v1/transactions/:id/approve     | ✅     | Real API - pending → approved  |
| POST /api/v1/transactions/:id/reject      | ✅     | Real API - pending → draft     |
| POST /api/v1/transactions/:id/post        | ✅     | Real API - approved → posted   |
| POST /api/v1/transactions/:id/void        | ✅     | Real API - posted → voided     |
| GET /api/v1/transactions/pending          | ✅     | Real API - approval queue      |
| POST /api/v1/transactions/bulk-approve    | ✅     | Real API - batch approval      |

### Attachments (Phase 5)

| Endpoint                                              | Status | Notes                          |
| ----------------------------------------------------- | ------ | ------------------------------ |
| POST /api/v1/transactions/:id/attachments/upload      | ✅     | Real API - presigned upload URL |
| POST /api/v1/transactions/:id/attachments             | ✅     | Real API - confirm upload       |
| GET /api/v1/transactions/:id/attachments              | ✅     | Real API - list attachments     |
| GET /api/v1/attachments/:id                           | ✅     | Real API - get with download URL |
| DELETE /api/v1/attachments/:id                        | ✅     | Real API - delete attachment    |

### Master Data

| Endpoint                           | Status | Notes                               |
| ---------------------------------- | ------ | ----------------------------------- |
| GET /api/v1/fiscal-years           | ✅     | Real API - list with nested periods |
| POST /api/v1/fiscal-years          | ✅     | Real API - create with auto-periods |
| GET /api/v1/fiscal-periods         | ✅     | Real API - list periods             |
| PATCH /api/v1/fiscal-periods/:id/status | ✅     | Real API - update status            |
| GET /api/v1/dimension-types         | ✅     | Real API - list types               |
| POST /api/v1/dimension-types       | ✅     | Real API - create type              |
| GET /api/v1/dimension-values        | ✅     | Real API - list with filters        |
| POST /api/v1/dimension-values       | ✅     | Real API - create value             |
| PATCH /api/v1/dimension-values/:id  | ✅     | Real API - update name/code         |
| PATCH /api/v1/dimension-values/:id/status | ✅     | Real API - toggle is_active         |
| GET /api/v1/exchange-rates          | ✅     | Real API - get rate for pair/date   |
| POST /api/v1/exchange-rates         | ✅     | Real API - create/update rate       |
| POST /api/v1/exchange-rates/fetch   | ✅     | Real API - fetch from Frankfurter   |
| POST /api/v1/exchange-rates/bulk    | ✅     | Real API - bulk import rates        |
| GET /api/v1/currencies              | ✅     | Real API - list all currencies      |
| PATCH /api/v1/accounts/:id/status   | ✅     | Real API - toggle is_active         |

### Approval Rules

| Endpoint                                     | Status | Notes                           |
| -------------------------------------------- | ------ | ------------------------------- |
| GET /api/v1/organizations/:id/approval-rules  | ✅     | Real API - list rules           |
| POST /api/v1/organizations/:id/approval-rules | ✅     | Real API - create rule (admin+) |
| GET /api/v1/organizations/:id/approval-rules/:id | ✅     | Real API - get rule detail      |
| PATCH /api/v1/organizations/:id/approval-rules/:id | ✅     | Real API - update rule (admin+) |
| DELETE /api/v1/organizations/:id/approval-rules/:id | ✅     | Real API - soft delete (admin+) |

### Reports

| Endpoint                      | Status | Notes                                |
| ----------------------------- | ------ | ------------------------------------ |
| GET /api/v1/reports/trial-balance    | ✅     | Real API - as_of, dimension filters  |
| GET /api/v1/reports/balance-sheet    | ✅     | Real API - as_of date                |
| GET /api/v1/reports/income-statement | ✅     | Real API - from/to, dimension filter |
| GET /api/v1/reports/dimensional      | ✅     | Real API - group_by dimensions       |
| GET /api/v1/budgets/:id/vs-actual    | ✅     | Real API - variance analysis         |

### Budgets

| Endpoint                  | Status | Notes                                |
| ------------------------- | ------ | ------------------------------------ |
| GET /api/v1/budgets       | ✅     | Real API - list with summary         |
| POST /api/v1/budgets      | ✅     | Real API - create budget             |
| GET /api/v1/budgets/:id   | ✅     | Real API - detail with lines         |
| PUT /api/v1/budgets/:id   | ✅     | Real API - update budget             |
| GET /api/v1/budgets/:id/lines | ✅     | Real API - list budget lines         |
| POST /api/v1/budgets/:id/lines | ✅     | Real API - bulk create lines         |
| POST /api/v1/budgets/:id/lock   | ✅     | Real API - lock budget               |

### Dashboard (Phase 5 - New Org-Scoped Endpoints)

| Endpoint                                          | Status | Notes                                |
| ------------------------------------------------- | ------ | ------------------------------------ |
| GET /api/v1/organizations/:id/dashboard/metrics   | ✅     | Real API - cash, burn rate, runway   |
| GET /api/v1/organizations/:id/dashboard/cash-flow | ✅     | Real API - monthly inflow/outflow    |
| GET /api/v1/organizations/:id/dashboard/recent-activity | ✅     | Real API - cursor pagination         |
| GET /api/v1/organizations/:id/dashboard/budget-vs-actual | ✅     | Real API - variance summary          |

### Dashboard (Deprecated)

| Endpoint                       | Status | Notes                                |
| ------------------------------ | ------ | ------------------------------------ |
| GET /api/v1/dashboard/metrics         | ⚠️     | DEPRECATED - use org-scoped endpoint |
| GET /api/v1/dashboard/recent-activity | ⚠️     | DEPRECATED - use org-scoped endpoint |

### Simulation

| Endpoint             | Status | Notes                                |
| -------------------- | ------ | ------------------------------------ |
| POST /api/v1/simulation/run | ✅     | Real API - projections with caching  |

---

## Blockers

| Issue        | Reporter | Status | Resolution                                                                           |
| ------------ | -------- | ------ | ------------------------------------------------------------------------------------ |
| ~~Missing APIs~~ | Frontend | ✅     | Phase 5 complete: Toggle Account, Edit Dim Value, Bulk Rates, Attachments all done |

---

## Notes

- **Frontend gak perlu nunggu Backend** - Pake MSW mock API
- Backend update status endpoint setelah implement
- Frontend cek status, kalau ⚠️ pake mock, kalau ✅ test real API
- Gradually replace mock dengan real API pas Backend catch up

---

## Frontend-Backend Schema Sync Analysis

**Last Verified:** 2026-01-13

### Compatibility Status: ✅ FULLY INTEGRATED

Frontend now uses real backend API. Mock API has been disabled. Notes:

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
