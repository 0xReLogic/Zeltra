# OpenAPI Specification Validation Report

**Date**: 2026-01-13  
**Repository**: 0xReLogic/Zeltra  
**Backend**: Rust (Axum framework)  
**OpenAPI Spec**: `contracts/openapi.yaml`

---

## Executive Summary

This report provides a comprehensive comparison between the OpenAPI specification and the actual backend implementation to prevent schema mismatches for frontend development.

### Key Findings

- ✅ **28 routes** (36.8%) are correctly documented in OpenAPI
- ❌ **48 routes** are implemented in backend but MISSING from OpenAPI
- ⚠️ **38 routes** are documented in OpenAPI but NOT implemented (deprecated/legacy)

### Critical Issue

**The OpenAPI specification is significantly out of date.** The backend has been refactored to use organization-scoped routes (`/organizations/{org_id}/...`), but the OpenAPI spec still documents old unscoped routes (`/accounts`, `/transactions`, etc.) that are NO LONGER IMPLEMENTED.

---

## Detailed Analysis

### 1. Missing Routes in OpenAPI (Backend ✅ → OpenAPI ❌)

These routes ARE implemented in the backend but are NOT documented in the OpenAPI spec:

#### 🏢 Organizations Module (14% coverage)
- ❌ `GET /organizations/{org_id}` - Get organization details
- ❌ `PATCH /organizations/{org_id}` - Update organization
- ❌ `GET /organizations/{org_id}/users` - List organization users
- ❌ `POST /organizations/{org_id}/users` - Add user to organization
- ❌ `PATCH /organizations/{org_id}/users/{user_id}` - Update member role
- ❌ `DELETE /organizations/{org_id}/users/{user_id}` - Remove user from organization

#### 💰 Accounts Module (0% coverage)
- ❌ `GET /organizations/{org_id}/accounts` - List accounts
- ❌ `POST /organizations/{org_id}/accounts` - Create account
- ❌ `GET /organizations/{org_id}/accounts/{account_id}` - Get account details
- ❌ `PUT /organizations/{org_id}/accounts/{account_id}` - Update account
- ❌ `DELETE /organizations/{org_id}/accounts/{account_id}` - Delete account
- ❌ `PATCH /organizations/{org_id}/accounts/{account_id}/status` - Toggle account status
- ❌ `GET /organizations/{org_id}/accounts/{account_id}/balance` - Get account balance
- ❌ `GET /organizations/{org_id}/accounts/{account_id}/ledger` - Get account ledger

#### 📝 Transactions Module (17% coverage)
- ❌ `GET /organizations/{org_id}/transactions` - List transactions
- ❌ `POST /organizations/{org_id}/transactions` - Create transaction
- ❌ `GET /organizations/{org_id}/transactions/{transaction_id}` - Get transaction details
- ❌ `PATCH /organizations/{org_id}/transactions/{transaction_id}` - Update transaction
- ❌ `DELETE /organizations/{org_id}/transactions/{transaction_id}` - Delete transaction
- ❌ `POST /organizations/{org_id}/transactions/{transaction_id}/submit` - Submit for approval
- ❌ `POST /organizations/{org_id}/transactions/{transaction_id}/approve` - Approve transaction
- ❌ `POST /organizations/{org_id}/transactions/{transaction_id}/reject` - Reject transaction
- ❌ `POST /organizations/{org_id}/transactions/{transaction_id}/post` - Post transaction to ledger
- ❌ `POST /organizations/{org_id}/transactions/{transaction_id}/void` - Void transaction

#### 💵 Budgets Module (0% coverage)
- ❌ `GET /organizations/{org_id}/budgets` - List budgets
- ❌ `POST /organizations/{org_id}/budgets` - Create budget
- ❌ `GET /organizations/{org_id}/budgets/{budget_id}` - Get budget details
- ❌ `PUT /organizations/{org_id}/budgets/{budget_id}` - Update budget
- ❌ `GET /organizations/{org_id}/budgets/{budget_id}/lines` - Get budget lines
- ❌ `POST /organizations/{org_id}/budgets/{budget_id}/lines` - Add budget lines
- ❌ `POST /organizations/{org_id}/budgets/{budget_id}/lock` - Lock budget
- ❌ `GET /organizations/{org_id}/budgets/{budget_id}/vs-actual` - Budget vs actual report

#### 🏷️ Dimensions Module (0% coverage)
- ❌ `GET /organizations/{org_id}/dimension-types` - List dimension types
- ❌ `POST /organizations/{org_id}/dimension-types` - Create dimension type
- ❌ `GET /organizations/{org_id}/dimension-values` - List dimension values
- ❌ `POST /organizations/{org_id}/dimension-values` - Create dimension value
- ❌ `PATCH /organizations/{org_id}/dimension-values/{value_id}` - Update dimension value
- ❌ `PATCH /organizations/{org_id}/dimension-values/{value_id}/status` - Toggle dimension value status

#### 📊 Reports Module (0% coverage)
- ❌ `GET /organizations/{org_id}/reports/trial-balance` - Trial balance report
- ❌ `GET /organizations/{org_id}/reports/balance-sheet` - Balance sheet report
- ❌ `GET /organizations/{org_id}/reports/income-statement` - Income statement report
- ❌ `GET /organizations/{org_id}/reports/dimensional` - Dimensional report
- ❌ `GET /organizations/{org_id}/accounts/{account_id}/ledger` - Account ledger (duplicate of accounts endpoint)

#### 🎯 Simulation Module (0% coverage)
- ❌ `POST /organizations/{org_id}/simulation/run` - Run budget simulation

#### 📅 Fiscal Module (0% coverage)
- ❌ `GET /organizations/{org_id}/fiscal-years` - List fiscal years
- ❌ `POST /organizations/{org_id}/fiscal-years` - Create fiscal year
- ❌ `PATCH /organizations/{org_id}/fiscal-periods/{period_id}/status` - Update fiscal period status

#### 🏥 Health Module (0% coverage)
- ❌ `GET /health` - Health check endpoint

---

### 2. Deprecated Routes in OpenAPI (OpenAPI ✅ → Backend ❌)

These routes are documented in OpenAPI but are **NOT IMPLEMENTED** in the backend. They appear to be legacy routes from before the org-scoping refactor:

#### Legacy Unscoped Routes (Should be removed or marked deprecated)

**Accounts:**
- ⚠️ `GET /accounts` → Should be `GET /organizations/{org_id}/accounts`
- ⚠️ `POST /accounts` → Should be `POST /organizations/{org_id}/accounts`
- ⚠️ `GET /accounts/{id}` → Should be `GET /organizations/{org_id}/accounts/{account_id}`
- ⚠️ `PUT /accounts/{id}` → Should be `PUT /organizations/{org_id}/accounts/{account_id}`
- ⚠️ `DELETE /accounts/{id}` → Should be `DELETE /organizations/{org_id}/accounts/{account_id}`
- ⚠️ `PATCH /accounts/{id}/status` → Should be `PATCH /organizations/{org_id}/accounts/{account_id}/status`
- ⚠️ `GET /accounts/{id}/balance` → Should be `GET /organizations/{org_id}/accounts/{account_id}/balance`
- ⚠️ `GET /accounts/{id}/ledger` → Should be `GET /organizations/{org_id}/accounts/{account_id}/ledger`

**Transactions:**
- ⚠️ `GET /transactions` → Should be `GET /organizations/{org_id}/transactions`
- ⚠️ `POST /transactions` → Should be `POST /organizations/{org_id}/transactions`
- ⚠️ `GET /transactions/{id}` → Should be `GET /organizations/{org_id}/transactions/{transaction_id}`
- ⚠️ `PATCH /transactions/{id}` → Should be `PATCH /organizations/{org_id}/transactions/{transaction_id}`
- ⚠️ `DELETE /transactions/{id}` → Should be `DELETE /organizations/{org_id}/transactions/{transaction_id}`
- ⚠️ `GET /transactions/{id}/attachments` → Should be `GET /organizations/{org_id}/transactions/{transaction_id}/attachments`

**Budgets:**
- ⚠️ `GET /budgets` → Should be `GET /organizations/{org_id}/budgets`
- ⚠️ `POST /budgets` → Should be `POST /organizations/{org_id}/budgets`
- ⚠️ `GET /budgets/{id}` → Should be `GET /organizations/{org_id}/budgets/{budget_id}`
- ⚠️ `POST /budgets/{id}/lines` → Should be `POST /organizations/{org_id}/budgets/{budget_id}/lines`
- ⚠️ `POST /budgets/{id}/lock` → Should be `POST /organizations/{org_id}/budgets/{budget_id}/lock`
- ⚠️ `GET /budgets/{id}/vs-actual` → Should be `GET /organizations/{org_id}/budgets/{budget_id}/vs-actual`

**Dimensions:**
- ⚠️ `GET /dimension-types` → Should be `GET /organizations/{org_id}/dimension-types`
- ⚠️ `POST /dimension-types` → Should be `POST /organizations/{org_id}/dimension-types`
- ⚠️ `GET /dimension-values` → Should be `GET /organizations/{org_id}/dimension-values`
- ⚠️ `POST /dimension-values` → Should be `POST /organizations/{org_id}/dimension-values`
- ⚠️ `PATCH /dimension-values/{id}` → Should be `PATCH /organizations/{org_id}/dimension-values/{value_id}`
- ⚠️ `PATCH /dimension-values/{id}/status` → Should be `PATCH /organizations/{org_id}/dimension-values/{value_id}/status`

**Reports:**
- ⚠️ `GET /reports/trial-balance` → Should be `GET /organizations/{org_id}/reports/trial-balance`
- ⚠️ `GET /reports/balance-sheet` → Should be `GET /organizations/{org_id}/reports/balance-sheet`
- ⚠️ `GET /reports/income-statement` → Should be `GET /organizations/{org_id}/reports/income-statement`
- ⚠️ `GET /reports/dimensional` → Should be `GET /organizations/{org_id}/reports/dimensional`

**Dashboard:**
- ⚠️ `GET /dashboard/metrics` → Should be `GET /organizations/{org_id}/dashboard/metrics`
- ⚠️ `GET /dashboard/cash-flow` → Should be `GET /organizations/{org_id}/dashboard/cash-flow`
- ⚠️ `GET /dashboard/recent-activity` → Should be `GET /organizations/{org_id}/dashboard/recent-activity`
- ⚠️ `GET /dashboard/budget-vs-actual` → Should be `GET /organizations/{org_id}/dashboard/budget-vs-actual`

**Fiscal:**
- ⚠️ `GET /fiscal-years` → Should be `GET /organizations/{org_id}/fiscal-years`
- ⚠️ `POST /fiscal-years` → Should be `POST /organizations/{org_id}/fiscal-years`
- ⚠️ `PATCH /fiscal-periods/{id}/status` → Should be `PATCH /organizations/{org_id}/fiscal-periods/{period_id}/status`

**Simulation:**
- ⚠️ `POST /simulation/run` → Should be `POST /organizations/{org_id}/simulation/run`

---

### 3. Correctly Documented Routes (Backend ✅ → OpenAPI ✅)

These routes are correctly documented and match the backend implementation:

#### ✅ Auth Module (100% coverage)
- ✅ `POST /auth/register`
- ✅ `POST /auth/login`
- ✅ `POST /auth/refresh`
- ✅ `POST /auth/logout`
- ✅ `POST /auth/verify-email`
- ✅ `POST /auth/resend-verification`

#### ✅ Organizations Module (14% coverage)
- ✅ `POST /organizations`

#### ✅ Transactions Module (17% coverage)
- ✅ `GET /organizations/{org_id}/transactions/pending`
- ✅ `POST /organizations/{org_id}/transactions/bulk-approve`

#### ✅ Dashboard Module (100% coverage)
- ✅ `GET /organizations/{org_id}/dashboard/metrics`
- ✅ `GET /organizations/{org_id}/dashboard/cash-flow`
- ✅ `GET /organizations/{org_id}/dashboard/recent-activity`
- ✅ `GET /organizations/{org_id}/dashboard/budget-vs-actual`

#### ✅ Exchange Rates Module (100% coverage)
- ✅ `GET /organizations/{org_id}/exchange-rates`
- ✅ `POST /organizations/{org_id}/exchange-rates`
- ✅ `POST /organizations/{org_id}/exchange-rates/fetch`
- ✅ `POST /organizations/{org_id}/exchange-rates/bulk`

#### ✅ Attachments Module (100% coverage)
- ✅ `POST /organizations/{org_id}/transactions/{transaction_id}/attachments/upload`
- ✅ `POST /organizations/{org_id}/transactions/{transaction_id}/attachments`
- ✅ `GET /organizations/{org_id}/transactions/{transaction_id}/attachments`
- ✅ `GET /organizations/{org_id}/attachments/{attachment_id}`
- ✅ `DELETE /organizations/{org_id}/attachments/{attachment_id}`

#### ✅ Approval Rules Module (100% coverage)
- ✅ `GET /organizations/{org_id}/approval-rules`
- ✅ `POST /organizations/{org_id}/approval-rules`
- ✅ `GET /organizations/{org_id}/approval-rules/{rule_id}`
- ✅ `PATCH /organizations/{org_id}/approval-rules/{rule_id}`
- ✅ `DELETE /organizations/{org_id}/approval-rules/{rule_id}`

#### ✅ Currencies Module (100% coverage)
- ✅ `GET /currencies`

---

## Recommendations for Frontend Team

### 🚨 Critical Actions Required

1. **DO NOT use the unscoped routes** documented in OpenAPI (like `/accounts`, `/transactions`, etc.)
   - These routes are NOT implemented in the backend
   - They will return 404 errors

2. **Always use organization-scoped routes** with `/organizations/{org_id}/...` prefix
   - Example: Use `GET /organizations/{org_id}/accounts` instead of `GET /accounts`
   - The `org_id` should be obtained from the user's current organization context

3. **Verify all endpoints before implementation**
   - Cross-reference this report with the OpenAPI spec
   - Test endpoints in development environment
   - Focus on the "Correctly Documented Routes" section for safe implementations

### 📝 Schema Validation Notes

While this report focuses on route availability, you should also:

1. **Verify request/response schemas** - Some schemas in OpenAPI may not match backend types
2. **Check field names** - Parameter names might differ (e.g., `account_id` vs `id`)
3. **Validate data types** - Ensure types match (e.g., UUID format, decimal precision)
4. **Test error responses** - Error structures might differ from OpenAPI spec

### 🔄 Next Steps

**For Backend Team:**
1. Update `contracts/openapi.yaml` to add all missing org-scoped routes
2. Remove or mark as deprecated all old unscoped routes
3. Add the missing `/health` endpoint documentation
4. Verify all request/response schemas match actual implementation

**For Frontend Team:**
1. Use this report as source of truth until OpenAPI is updated
2. Start with modules that have 100% coverage (auth, dashboard, exchange_rates, attachments, approval_rules, currencies)
3. For other modules, refer to backend Rust code in `backend/crates/api/src/routes/` for accurate API definitions
4. Create integration tests to catch schema mismatches early

---

## Appendix: Backend Route Inventory

### Complete List of Implemented Backend Routes

```
Auth (6 routes)
├── POST   /auth/login
├── POST   /auth/register
├── POST   /auth/refresh
├── POST   /auth/logout
├── POST   /auth/verify-email
└── POST   /auth/resend-verification

Organizations (7 routes)
├── POST   /organizations
├── GET    /organizations/{org_id}
├── PATCH  /organizations/{org_id}
├── GET    /organizations/{org_id}/users
├── POST   /organizations/{org_id}/users
├── PATCH  /organizations/{org_id}/users/{user_id}
└── DELETE /organizations/{org_id}/users/{user_id}

Accounts (8 routes)
├── GET    /organizations/{org_id}/accounts
├── POST   /organizations/{org_id}/accounts
├── GET    /organizations/{org_id}/accounts/{account_id}
├── PUT    /organizations/{org_id}/accounts/{account_id}
├── DELETE /organizations/{org_id}/accounts/{account_id}
├── PATCH  /organizations/{org_id}/accounts/{account_id}/status
├── GET    /organizations/{org_id}/accounts/{account_id}/balance
└── GET    /organizations/{org_id}/accounts/{account_id}/ledger

Transactions (12 routes)
├── GET    /organizations/{org_id}/transactions
├── POST   /organizations/{org_id}/transactions
├── GET    /organizations/{org_id}/transactions/pending
├── POST   /organizations/{org_id}/transactions/bulk-approve
├── GET    /organizations/{org_id}/transactions/{transaction_id}
├── PATCH  /organizations/{org_id}/transactions/{transaction_id}
├── DELETE /organizations/{org_id}/transactions/{transaction_id}
├── POST   /organizations/{org_id}/transactions/{transaction_id}/submit
├── POST   /organizations/{org_id}/transactions/{transaction_id}/approve
├── POST   /organizations/{org_id}/transactions/{transaction_id}/reject
├── POST   /organizations/{org_id}/transactions/{transaction_id}/post
└── POST   /organizations/{org_id}/transactions/{transaction_id}/void

Budgets (8 routes)
├── GET    /organizations/{org_id}/budgets
├── POST   /organizations/{org_id}/budgets
├── GET    /organizations/{org_id}/budgets/{budget_id}
├── PUT    /organizations/{org_id}/budgets/{budget_id}
├── GET    /organizations/{org_id}/budgets/{budget_id}/lines
├── POST   /organizations/{org_id}/budgets/{budget_id}/lines
├── POST   /organizations/{org_id}/budgets/{budget_id}/lock
└── GET    /organizations/{org_id}/budgets/{budget_id}/vs-actual

Dimensions (6 routes)
├── GET    /organizations/{org_id}/dimension-types
├── POST   /organizations/{org_id}/dimension-types
├── GET    /organizations/{org_id}/dimension-values
├── POST   /organizations/{org_id}/dimension-values
├── PATCH  /organizations/{org_id}/dimension-values/{value_id}
└── PATCH  /organizations/{org_id}/dimension-values/{value_id}/status

Dashboard (4 routes)
├── GET    /organizations/{org_id}/dashboard/metrics
├── GET    /organizations/{org_id}/dashboard/cash-flow
├── GET    /organizations/{org_id}/dashboard/recent-activity
└── GET    /organizations/{org_id}/dashboard/budget-vs-actual

Reports (5 routes)
├── GET    /organizations/{org_id}/reports/trial-balance
├── GET    /organizations/{org_id}/reports/balance-sheet
├── GET    /organizations/{org_id}/reports/income-statement
├── GET    /organizations/{org_id}/reports/dimensional
└── GET    /organizations/{org_id}/accounts/{account_id}/ledger

Simulation (1 route)
└── POST   /organizations/{org_id}/simulation/run

Exchange Rates (4 routes)
├── GET    /organizations/{org_id}/exchange-rates
├── POST   /organizations/{org_id}/exchange-rates
├── POST   /organizations/{org_id}/exchange-rates/fetch
└── POST   /organizations/{org_id}/exchange-rates/bulk

Fiscal (3 routes)
├── GET    /organizations/{org_id}/fiscal-years
├── POST   /organizations/{org_id}/fiscal-years
└── PATCH  /organizations/{org_id}/fiscal-periods/{period_id}/status

Attachments (5 routes)
├── POST   /organizations/{org_id}/transactions/{transaction_id}/attachments/upload
├── POST   /organizations/{org_id}/transactions/{transaction_id}/attachments
├── GET    /organizations/{org_id}/transactions/{transaction_id}/attachments
├── GET    /organizations/{org_id}/attachments/{attachment_id}
└── DELETE /organizations/{org_id}/attachments/{attachment_id}

Approval Rules (5 routes)
├── GET    /organizations/{org_id}/approval-rules
├── POST   /organizations/{org_id}/approval-rules
├── GET    /organizations/{org_id}/approval-rules/{rule_id}
├── PATCH  /organizations/{org_id}/approval-rules/{rule_id}
└── DELETE /organizations/{org_id}/approval-rules/{rule_id}

Currencies (1 route)
└── GET    /currencies

Health (1 route)
└── GET    /health
```

**Total: 76 implemented backend routes**

---

## Contact

For questions or clarifications about this report, please:
- Review the backend source code in `backend/crates/api/src/routes/`
- Check the OpenAPI specification in `contracts/openapi.yaml`
- Consult with the backend team for schema details

**Generated**: 2026-01-13  
**Tool**: Automated OpenAPI validation script
