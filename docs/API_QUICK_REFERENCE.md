# Quick Reference: Correct API Endpoints

**⚠️ IMPORTANT**: This is a quick reference guide for frontend developers. The OpenAPI spec (`contracts/openapi.yaml`) is outdated and should NOT be trusted as the source of truth.

---

## 🚨 Critical Information

### DO NOT USE These Endpoints (They don't exist!)
```
❌ GET    /accounts
❌ POST   /accounts
❌ GET    /accounts/{id}
❌ GET    /transactions
❌ POST   /transactions
❌ GET    /budgets
❌ POST   /budgets
❌ GET    /dimension-types
❌ GET    /reports/trial-balance
❌ POST   /simulation/run
```

### ✅ Use These Instead (Organization-scoped)
```
✅ GET    /organizations/{org_id}/accounts
✅ POST   /organizations/{org_id}/accounts
✅ GET    /organizations/{org_id}/accounts/{account_id}
✅ GET    /organizations/{org_id}/transactions
✅ POST   /organizations/{org_id}/transactions
✅ GET    /organizations/{org_id}/budgets
✅ POST   /organizations/{org_id}/budgets
✅ GET    /organizations/{org_id}/dimension-types
✅ GET    /organizations/{org_id}/reports/trial-balance
✅ POST   /organizations/{org_id}/simulation/run
```

---

## 📚 Complete API Reference

### Authentication (No org_id needed)
```
POST   /auth/register
POST   /auth/login
POST   /auth/refresh
POST   /auth/logout
POST   /auth/verify-email
POST   /auth/resend-verification
```

### Organizations
```
POST   /organizations                                    - Create new organization
GET    /organizations/{org_id}                          - Get organization details
PATCH  /organizations/{org_id}                          - Update organization
GET    /organizations/{org_id}/users                    - List organization users
POST   /organizations/{org_id}/users                    - Add user to organization
PATCH  /organizations/{org_id}/users/{user_id}         - Update member role
DELETE /organizations/{org_id}/users/{user_id}         - Remove user
```

### Accounts
```
GET    /organizations/{org_id}/accounts                         - List accounts
POST   /organizations/{org_id}/accounts                         - Create account
GET    /organizations/{org_id}/accounts/{account_id}           - Get account
PUT    /organizations/{org_id}/accounts/{account_id}           - Update account
DELETE /organizations/{org_id}/accounts/{account_id}           - Delete account
PATCH  /organizations/{org_id}/accounts/{account_id}/status    - Toggle active status
GET    /organizations/{org_id}/accounts/{account_id}/balance   - Get balance
GET    /organizations/{org_id}/accounts/{account_id}/ledger    - Get ledger entries
```

### Transactions
```
GET    /organizations/{org_id}/transactions                               - List transactions
POST   /organizations/{org_id}/transactions                               - Create transaction
GET    /organizations/{org_id}/transactions/pending                       - Get pending transactions
POST   /organizations/{org_id}/transactions/bulk-approve                  - Bulk approve
GET    /organizations/{org_id}/transactions/{transaction_id}             - Get transaction
PATCH  /organizations/{org_id}/transactions/{transaction_id}             - Update transaction
DELETE /organizations/{org_id}/transactions/{transaction_id}             - Delete transaction
POST   /organizations/{org_id}/transactions/{transaction_id}/submit      - Submit for approval
POST   /organizations/{org_id}/transactions/{transaction_id}/approve     - Approve transaction
POST   /organizations/{org_id}/transactions/{transaction_id}/reject      - Reject transaction
POST   /organizations/{org_id}/transactions/{transaction_id}/post        - Post to ledger
POST   /organizations/{org_id}/transactions/{transaction_id}/void        - Void transaction
```

### Budgets
```
GET    /organizations/{org_id}/budgets                           - List budgets
POST   /organizations/{org_id}/budgets                           - Create budget
GET    /organizations/{org_id}/budgets/{budget_id}              - Get budget
PUT    /organizations/{org_id}/budgets/{budget_id}              - Update budget
GET    /organizations/{org_id}/budgets/{budget_id}/lines        - Get budget lines
POST   /organizations/{org_id}/budgets/{budget_id}/lines        - Add budget lines
POST   /organizations/{org_id}/budgets/{budget_id}/lock         - Lock budget
GET    /organizations/{org_id}/budgets/{budget_id}/vs-actual    - Budget vs actual report
```

### Dimensions
```
GET    /organizations/{org_id}/dimension-types                      - List dimension types
POST   /organizations/{org_id}/dimension-types                      - Create dimension type
GET    /organizations/{org_id}/dimension-values                     - List dimension values
POST   /organizations/{org_id}/dimension-values                     - Create dimension value
PATCH  /organizations/{org_id}/dimension-values/{value_id}         - Update dimension value
PATCH  /organizations/{org_id}/dimension-values/{value_id}/status  - Toggle status
```

### Dashboard
```
GET    /organizations/{org_id}/dashboard/metrics           - Get dashboard metrics
GET    /organizations/{org_id}/dashboard/cash-flow         - Get cash flow data
GET    /organizations/{org_id}/dashboard/recent-activity   - Get recent activity
GET    /organizations/{org_id}/dashboard/budget-vs-actual  - Get budget vs actual summary
```

### Reports
```
GET    /organizations/{org_id}/reports/trial-balance      - Trial balance report
GET    /organizations/{org_id}/reports/balance-sheet      - Balance sheet report
GET    /organizations/{org_id}/reports/income-statement   - Income statement report
GET    /organizations/{org_id}/reports/dimensional        - Dimensional analysis report
GET    /organizations/{org_id}/accounts/{account_id}/ledger  - Account ledger report
```

### Simulation
```
POST   /organizations/{org_id}/simulation/run             - Run budget simulation
```

### Exchange Rates
```
GET    /organizations/{org_id}/exchange-rates             - Get exchange rate
POST   /organizations/{org_id}/exchange-rates             - Create exchange rate
POST   /organizations/{org_id}/exchange-rates/fetch       - Fetch rates from API
POST   /organizations/{org_id}/exchange-rates/bulk        - Bulk import rates
```

### Fiscal Years & Periods
```
GET    /organizations/{org_id}/fiscal-years                         - List fiscal years
POST   /organizations/{org_id}/fiscal-years                         - Create fiscal year
PATCH  /organizations/{org_id}/fiscal-periods/{period_id}/status   - Update period status
```

### Attachments
```
POST   /organizations/{org_id}/transactions/{transaction_id}/attachments/upload  - Request upload URL
POST   /organizations/{org_id}/transactions/{transaction_id}/attachments         - Confirm upload
GET    /organizations/{org_id}/transactions/{transaction_id}/attachments         - List attachments
GET    /organizations/{org_id}/attachments/{attachment_id}                       - Get attachment
DELETE /organizations/{org_id}/attachments/{attachment_id}                       - Delete attachment
```

### Approval Rules
```
GET    /organizations/{org_id}/approval-rules               - List approval rules
POST   /organizations/{org_id}/approval-rules               - Create approval rule
GET    /organizations/{org_id}/approval-rules/{rule_id}     - Get approval rule
PATCH  /organizations/{org_id}/approval-rules/{rule_id}     - Update approval rule
DELETE /organizations/{org_id}/approval-rules/{rule_id}     - Delete approval rule
```

### Currencies (Global - No org_id)
```
GET    /currencies                                         - List supported currencies
```

### Health Check (No org_id)
```
GET    /health                                             - Health check
```

---

## 🎯 Common Patterns

### Getting the Organization ID

The `org_id` is typically obtained from:
1. User's authentication context (JWT token)
2. User's current/selected organization
3. Organization list endpoint after login

```typescript
// Example: Get org_id from user context
const currentOrg = user.organizations[0]; // or user.currentOrganization
const orgId = currentOrg.id;

// Use in API calls
const accounts = await api.get(`/organizations/${orgId}/accounts`);
```

### Authentication Headers

All protected endpoints (except `/auth/*` and `/health`) require authentication:

```typescript
headers: {
  'Authorization': `Bearer ${accessToken}`,
  'Content-Type': 'application/json'
}
```

### Error Handling

All endpoints return errors in this format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request body",
    "details": {},
    "request_id": "uuid"
  }
}
```

---

## 📝 Example Usage

### Create an Account

```typescript
const response = await fetch(
  `${API_BASE_URL}/organizations/${orgId}/accounts`,
  {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${accessToken}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      code: '1001',
      name: 'Cash',
      description: 'Main cash account',
      type: 'asset',
      subtype: 'current_asset',  // Note: 'subtype', not 'account_subtype'
      currency: 'USD',
      is_active: true,
      allow_direct_posting: true
    })
  }
);

const account = await response.json();
```

### List Transactions

```typescript
const response = await fetch(
  `${API_BASE_URL}/organizations/${orgId}/transactions?status=pending&limit=20`,
  {
    headers: {
      'Authorization': `Bearer ${accessToken}`
    }
  }
);

const data = await response.json();
const transactions = data.data; // List is in 'data' field
const pagination = data.pagination;
```

### Submit Transaction for Approval

```typescript
const response = await fetch(
  `${API_BASE_URL}/organizations/${orgId}/transactions/${transactionId}/submit`,
  {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${accessToken}`
    }
  }
);
```

---

## 🔍 Finding More Information

### If you need detailed schema information:

1. **Backend Source Code** (Most reliable):
   - Request types: `backend/crates/api/src/routes/*.rs`
   - Shared types: `backend/crates/shared/src/*.rs`
   
2. **Validation Report**:
   - See `docs/OPENAPI_VALIDATION_REPORT.md` for complete analysis
   - See `docs/OPENAPI_SCHEMA_MISMATCHES.md` for schema details

3. **Test the API**:
   - Use Postman/Insomnia to test endpoints
   - Check actual request/response in browser DevTools
   - API examples: `contracts/api-examples.http`

---

## ⚠️ Known Schema Issues

### Account Creation
- Use `subtype` (not `account_subtype`)
- Include `description` and `allow_direct_posting` fields
- `currency` is required

### Common Field Name Conventions
- Use snake_case for all field names
- IDs end with `_id` (e.g., `account_id`, `transaction_id`)
- Timestamps end with `_at` (e.g., `created_at`, `updated_at`)
- Boolean fields start with `is_` or `has_` (e.g., `is_active`, `has_children`)

---

**Last Updated**: 2026-01-13  
**Maintained By**: Backend Team  
**For Questions**: Check `docs/OPENAPI_VALIDATION_REPORT.md` or ask backend team
