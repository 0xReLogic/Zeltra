# Zeltra - Progress Tracker

Live status untuk sync antara Backend & Frontend.

---

## Current State

|                    | Backend           | Frontend                     |
| ------------------ | ----------------- | ---------------------------- |
| **Current Phase**  | 0 (External)      | 1 (Done)                     |
| **Last Task Done** | -                 | Phase 1: Auth ✅ (FE Mocked) |
| **Next Task**      | External AI Agent | Phase 6/7 Foundations        |

**Last Updated:** -

---

## Legend

- ⬜ Not Started
- 🟡 In Progress
- ✅ Done
- ❌ Blocked

---

## Phase Status

| 7: FE Features | - | ⬜ | |
| 8: Launch | ⬜ | ⬜ | |

---

## API Endpoints Status

Frontend cek di sini untuk tau endpoint mana yang udah ready.

### Auth

| Endpoint            | Status | Notes |
| ------------------- | ------ | ----- |
| POST /auth/register | ⬜     |       |
| POST /auth/login    | ⬜     |       |
| POST /auth/refresh  | ⬜     |       |
| POST /auth/logout   | ⬜     |       |

### Organizations

| Endpoint                      | Status | Notes |
| ----------------------------- | ------ | ----- |
| GET /organizations            | ⬜     |       |
| POST /organizations           | ⬜     |       |
| POST /organizations/:id/users | ⬜     |       |

### Accounts

| Endpoint                  | Status | Notes |
| ------------------------- | ------ | ----- |
| GET /accounts             | ⬜     |       |
| POST /accounts            | ⬜     |       |
| GET /accounts/:id/balance | ⬜     |       |
| GET /accounts/:id/ledger  | ⬜     |       |

### Transactions

| Endpoint                       | Status | Notes |
| ------------------------------ | ------ | ----- |
| GET /transactions              | ⬜     |       |
| POST /transactions             | ⬜     |       |
| GET /transactions/:id          | ⬜     |       |
| POST /transactions/:id/submit  | ⬜     |       |
| POST /transactions/:id/approve | ⬜     |       |
| POST /transactions/:id/reject  | ⬜     |       |
| POST /transactions/:id/post    | ⬜     |       |
| POST /transactions/:id/void    | ⬜     |       |

### Master Data

| Endpoint                         | Status | Notes |
| -------------------------------- | ------ | ----- |
| GET /fiscal-years                | ⬜     |       |
| POST /fiscal-years               | ⬜     |       |
| GET /fiscal-periods              | ⬜     |       |
| PATCH /fiscal-periods/:id/status | ⬜     |       |
| GET /dimension-types             | ⬜     |       |
| GET /dimension-values            | ⬜     |       |
| GET /exchange-rates              | ⬜     |       |
| POST /exchange-rates             | ⬜     |       |

### Reports

| Endpoint                      | Status | Notes |
| ----------------------------- | ------ | ----- |
| GET /reports/trial-balance    | ⬜     |       |
| GET /reports/balance-sheet    | ⬜     |       |
| GET /reports/income-statement | ⬜     |       |
| GET /reports/dimensional      | ⬜     |       |
| GET /budgets/:id/vs-actual    | ⬜     |       |

### Dashboard

| Endpoint                       | Status | Notes |
| ------------------------------ | ------ | ----- |
| GET /dashboard/metrics         | ⬜     |       |
| GET /dashboard/recent-activity | ⬜     |       |

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
