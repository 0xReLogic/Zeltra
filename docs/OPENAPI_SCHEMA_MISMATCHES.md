# OpenAPI Schema Mismatches - Detailed Analysis

This document complements the main validation report by identifying specific schema mismatches between the OpenAPI specification and backend implementation.

---

## Field Name Differences

### CreateAccountRequest

**OpenAPI (`contracts/openapi.yaml`):**
```yaml
CreateAccountRequest:
  type: object
  required: [code, name, account_type]
  properties:
    code: string
    name: string
    account_type: AccountType
    account_subtype: AccountSubtype  # ← Different name
    parent_id: uuid (nullable)
    is_active: boolean (default: true)
    currency: string
```

**Backend (`backend/crates/api/src/routes/accounts.rs`):**
```rust
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,  // ← Missing in OpenAPI
    #[serde(rename = "type")]
    pub account_type: String,
    pub subtype: Option<String>,  // ← Different name (not account_subtype)
    pub parent_id: Option<Uuid>,
    pub currency: String,
    pub is_active: Option<bool>,
    pub allow_direct_posting: Option<bool>,  // ← Missing in OpenAPI
}
```

**Mismatches:**
1. ❌ Field name: `account_subtype` (OpenAPI) vs `subtype` (Backend)
2. ❌ Missing field in OpenAPI: `description`
3. ❌ Missing field in OpenAPI: `allow_direct_posting`
4. ⚠️ `currency` is not required in OpenAPI but is in backend

---

## Required Fields Discrepancies

### LoginRequest

**OpenAPI:**
```yaml
LoginRequest:
  type: object
  required: [email, password]
  properties:
    email:
      type: string
      format: email
    password:
      type: string
      minLength: 8  # ← Validation rule in OpenAPI
```

**Backend:**
```rust
pub struct LoginRequest {
    pub email: String,
    pub password: String,  // ← No minLength validation in type
}
```

**Status:** ✅ Match (validation happens at application layer, not type level)

---

## Missing Routes Causing Schema Issues

Since many routes are missing from the OpenAPI spec, the following schemas are documented but may not be accurate:

### Accounts Module (0% route coverage)
- `CreateAccountRequest` - Field name mismatch
- `UpdateAccountRequest` - Likely has mismatches
- `Account` response type - Should be verified

### Transactions Module (17% route coverage)
- `CreateTransactionRequest` - Not documented for org-scoped route
- `UpdateTransactionRequest` - Not documented for org-scoped route
- `Transaction` response type - May have mismatches

### Budgets Module (0% route coverage)
- `CreateBudgetRequest` - Not documented for org-scoped route
- `BudgetLineRequest` - Not documented for org-scoped route
- All budget response types - Should be verified

### Dimensions Module (0% route coverage)
- `CreateDimensionTypeRequest` - Not documented
- `CreateDimensionValueRequest` - Not documented
- All dimension response types - Should be verified

### Reports Module (0% route coverage)
- All report response types (`TrialBalanceResponse`, `BalanceSheetResponse`, etc.) - Should be verified

### Fiscal Module (0% route coverage)
- `CreateFiscalYearRequest` - Not documented for org-scoped route
- Fiscal period types - Should be verified

---

## Known Schema Matches

These schemas have been verified to match between OpenAPI and backend:

### Auth Module ✅
- `LoginRequest` - ✅ Match
- `LoginResponse` - ✅ Match (with minor field name convention)
- `RegisterRequest` - ✅ Match
- `RefreshRequest` - ✅ Match
- `LogoutRequest` - ✅ Match

### Exchange Rates Module ✅
- `CreateExchangeRateRequest` - ✅ Match
- `ExchangeRateResponse` - ✅ Match
- `FetchRatesRequest` - ✅ Match
- `FetchRatesResponse` - ✅ Match

### Attachments Module ✅
- `RequestUploadRequest` - ✅ Match
- `RequestUploadResponse` - ✅ Match
- `ConfirmUploadRequest` - ✅ Match
- `AttachmentResponse` - ✅ Match

### Approval Rules Module ✅
- `CreateApprovalRuleRequest` - ✅ Match
- `UpdateApprovalRuleRequest` - ✅ Match
- `ApprovalRule` response - ✅ Match

---

## Recommendations

### For Backend Team

1. **Standardize field naming**
   - Use `subtype` consistently (not `account_subtype`)
   - Document all optional fields in OpenAPI

2. **Add missing routes to OpenAPI**
   - This will force documentation of all request/response schemas
   - Use correct org-scoped paths

3. **Add OpenAPI generation from code**
   - Consider using tools like `utoipa` (Rust crate) to generate OpenAPI from code annotations
   - This ensures schemas always match implementation

### For Frontend Team

1. **Trust backend type definitions over OpenAPI**
   - When in doubt, refer to Rust structs in `backend/crates/api/src/routes/`
   - Test API requests in development to verify actual schema

2. **Use TypeScript generators with caution**
   - If generating TypeScript types from OpenAPI, verify critical schemas manually
   - Focus on modules with 100% route coverage (auth, exchange_rates, attachments, approval_rules)

3. **Report schema mismatches**
   - Create issues for any discrepancies found during development
   - Help maintain this validation report

---

## Testing Recommendations

### Integration Test Examples

```typescript
// Example: Testing account creation with correct schema
describe('Account API', () => {
  it('should create account with correct field names', async () => {
    const request = {
      code: '1001',
      name: 'Test Account',
      description: 'Test description',  // Include this field
      type: 'asset',
      subtype: 'current_asset',  // Use 'subtype', not 'account_subtype'
      currency: 'USD',
      is_active: true,
      allow_direct_posting: true,  // Include this field
    };
    
    const response = await api.post(
      '/organizations/{org_id}/accounts',  // Use org-scoped path
      request
    );
    
    expect(response.status).toBe(201);
  });
});
```

### Schema Validation Tests

```typescript
// Create schema validation tests for critical types
import { z } from 'zod';

const CreateAccountRequestSchema = z.object({
  code: z.string(),
  name: z.string(),
  description: z.string().optional(),
  type: z.enum(['asset', 'liability', 'equity', 'revenue', 'expense']),
  subtype: z.string().optional(),  // NOT account_subtype
  parent_id: z.string().uuid().optional(),
  currency: z.string(),
  is_active: z.boolean().optional(),
  allow_direct_posting: z.boolean().optional(),
});

// Use this to validate requests before sending
const validatedRequest = CreateAccountRequestSchema.parse(request);
```

---

## Future Improvements

1. **Automated Schema Testing**
   - Set up CI to compare OpenAPI schemas against backend types
   - Fail builds on mismatch

2. **OpenAPI Auto-generation**
   - Generate OpenAPI spec from backend code annotations
   - Ensures 100% accuracy

3. **Contract Testing**
   - Use tools like Pact or Dredd for contract testing
   - Validate actual API responses against OpenAPI spec

4. **Documentation**
   - Keep this validation report updated
   - Document all schema changes in changelog

---

**Last Updated**: 2026-01-13  
**Next Review**: After backend updates OpenAPI spec
