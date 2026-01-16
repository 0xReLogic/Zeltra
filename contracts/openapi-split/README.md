# OpenAPI Spec - Split for Auditing

This directory contains the OpenAPI specification split into **28 manageable chunks** for systematic auditing.

## 📁 File Structure

### **Schemas (Data Models)**

| File | Schemas | Description |
|------|---------|-------------|
| `01-auth-org-schemas.yaml` | 18 | Auth, User, Organization, Membership types |
| `02-transactions-schemas.yaml` | 16 | Transaction, Entry, Approval, Void types |
| `03-accounts-ledger-schemas.yaml` | 5 | Account, Ledger types |
| `04-budgets-schemas.yaml` | 8 | Budget, Budget Line types |
| `05-reports-schemas.yaml` | 8 | Trial Balance, Balance Sheet, Income Statement, Dimensional |
| `06-sentinel-schemas.yaml` | 5 | Accruals, Revaluation, Intercompany |
| `07-forensic-schemas.yaml` | 5 | Benford, Reconciliation, Health Score |
| `08-master-data-schemas.yaml` | 22 | Fiscal, Dimensions, Exchange Rates, Currencies |
| `09-dashboard-schemas.yaml` | 7 | Dashboard metrics, Cash flow, Activity |
| `10-simulation-attachments-schemas.yaml` | 9 | Simulation, Attachments |
| `11-common-schemas.yaml` | 11 | ApiError, Pagination, Page helpers, Health, Toggle |
| `12-approval-rules-schemas.yaml` | 3 | Approval Rules CRUD types |

### **Endpoints (API Routes)**

| File | Endpoints | Description |
|------|-----------|-------------|
| `13-auth-endpoints.yaml` | 6 | Login, Register, Refresh, Logout, Email Verification |
| `14-org-endpoints.yaml` | 4 | Organization CRUD, Users Management |
| `15-transaction-endpoints.yaml` | 10 | Transaction CRUD, Workflow (submit, approve, reject, post, void), Bulk Approve, Pay Invoice |
| `16-accounts-endpoints.yaml` | 5 | Accounts CRUD, Ledger, Balance, Toggle Status |
| `17-fiscal-endpoints.yaml` | 2 | Fiscal Years, Fiscal Periods, Update Period Status |
| `18-dimensions-endpoints.yaml` | 4 | Dimension Types, Dimension Values, Toggle Status |
| `19-exchange-rates-endpoints.yaml` | 5 | Exchange Rates CRUD, Bulk Import, Fetch Live Rates, Currencies |
| `20-budgets-endpoints.yaml` | 5 | Budgets CRUD, Budget Lines, Lock, Vs Actual |
| `21-reports-endpoints.yaml` | 4 | Trial Balance, Balance Sheet, Income Statement, Dimensional |
| `22-sentinel-endpoints.yaml` | 5 | Revaluation Logs, Accrual Schedules, Intercompany Mappings |
| `23-forensic-endpoints.yaml` | 2 | Benford Analysis, Reconciliation, Health Score |
| `24-dashboard-endpoints.yaml` | 4 | Dashboard Metrics, Cash Flow, Recent Activity, Budget vs Actual |
| `25-simulation-endpoints.yaml` | 1 | Simulation Run |
| `26-attachments-endpoints.yaml` | 3 | Attachments CRUD, Upload |
| `27-approval-rules-endpoints.yaml` | 2 | Approval Rules CRUD |
| `28-health-endpoints.yaml` | 1 | Health Check |

---

## 🎯 Audit Strategy

### **Phase 1: Schema Validation** (Week 1)
Audit each schema file to ensure:
- ✅ All required fields are present
- ✅ Field types match backend implementation
- ✅ Descriptions are clear and accurate
- ✅ Examples are provided where helpful

**Order:**
1. `02-transactions-schemas.yaml` (HIGHEST PRIORITY - most complex)
2. `06-sentinel-schemas.yaml` (Accruals, Revaluation, Intercompany)
3. `03-accounts-ledger-schemas.yaml`
4. `04-budgets-schemas.yaml`
5. `08-master-data-schemas.yaml`
6. Others...

### **Phase 2: Endpoint Validation** (Week 2)
Audit each endpoint file to ensure:
- ✅ Request/response schemas are correct
- ✅ HTTP methods are appropriate
- ✅ Error responses (400, 402, 404, etc.) are documented
- ✅ Path parameters are validated
- ✅ Query parameters are documented

**Order:**
1. `14-transaction-endpoints.yaml` (HIGHEST PRIORITY - 10 endpoints)
2. `21-sentinel-endpoints.yaml` (Sentinel Intelligence - 5 endpoints)
3. `15-accounts-endpoints.yaml` (5 endpoints)
4. `19-budgets-endpoints.yaml` (5 endpoints)
5. `20-reports-endpoints.yaml` (4 endpoints)
6. Others...

### **Phase 3: Frontend Integration** (Week 3)
For each audited section:
- ✅ Regenerate frontend types: `pnpm run generate:types`
- ✅ Update component imports
- ✅ Fix type mismatches
- ✅ Test API calls

---

## 🔄 Regeneration

To regenerate the full OpenAPI spec from backend:

```bash
cd backend
cargo run --bin generate-openapi
```

To re-split the spec:

```bash
cd contracts
python3 split-openapi.py
```

---

## 📝 Audit Checklist Template

For each file, create an audit report:

```markdown
# Audit: [filename]

## ✅ Validated Schemas/Endpoints
- [x] SchemaName1 - All fields correct
- [x] SchemaName2 - Added missing description

## ⚠️ Issues Found
1. **SchemaName3**: Missing `timezone` field
   - **Fix**: Added `timezone: string` field
   - **Impact**: Frontend needs to send timezone

## 🔧 Recommended Changes
1. Add example values for complex schemas
2. Improve error response documentation

## 📊 Stats
- Total schemas/endpoints: X
- Issues found: Y
- Issues fixed: Z
```

---

## 🚀 Next Steps

1. **Start with Transaction schemas** (`02-transactions-schemas.yaml`)
2. **Audit systematically** - one file at a time
3. **Document findings** - create audit reports
4. **Fix issues** - update backend, regenerate OpenAPI
5. **Sync frontend** - regenerate types, update components

---

**Generated:** 2026-01-15
**Total Files:** 28 (12 schemas + 16 endpoints)
**Total Schemas:** 117 (ALL CAPTURED ✅)
**Total Endpoints:** 63
**Largest File:** ~14KB (transaction-endpoints)
**Smallest File:** ~328B (health-endpoint)
