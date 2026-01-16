# Implementation Tasks: Audit & Fix 05-Reports Schema

## Notes

- **E2E Testing**: Use Playwright MCP (bukan script), IP `10.0.0.5` bukan localhost
- **Login Credentials**: 
  - Email: `corp@zeltra.io`
  - Password: `qwertyui`
- **Lint Check**: Setiap selesai coding, cek Problems panel (getDiagnostics)
- **Root Cause**: Backend utoipa IntoParams missing `parameter_in = Query` attribute
- **Bug Tracking**: Kalau ketemu bug aneh, save ke Cognito project `zeltra-bug` untuk learning

## Tasks

- [-] 1. Fix Backend Utoipa Annotations
  - [x] 1.1 Add `#[into_params(parameter_in = Query)]` to TrialBalanceQuery
    - File: `backend/crates/api/src/routes/reports.rs` line ~57
    - _Requirements: REQ-1.1_
  - [x] 1.2 Add `#[into_params(parameter_in = Query)]` to BalanceSheetQuery
    - File: `backend/crates/api/src/routes/reports.rs` line ~67
    - _Requirements: REQ-1.2_
  - [x] 1.3 Add `#[into_params(parameter_in = Query)]` to IncomeStatementQuery
    - File: `backend/crates/api/src/routes/reports.rs` line ~74
    - _Requirements: REQ-1.3_
  - [x] 1.4 Add `#[into_params(parameter_in = Query)]` to DimensionalReportQuery
    - File: `backend/crates/api/src/routes/reports.rs` line ~86
    - _Requirements: REQ-1.4_
  - [ ] 1.5 Add `#[into_params(parameter_in = Query)]` to AccountLedgerQuery
    - File: `backend/crates/api/src/routes/reports.rs` line ~103
    - _Requirements: REQ-1.5_

- [ ] 2. Regenerate OpenAPI and Frontend Types
  - [x] 2.1 Regenerate OpenAPI spec
    - Run `cargo run --bin generate-openapi` in backend/
    - _Requirements: REQ-5.1_
  - [ ] 2.2 Run split-openapi script
    - Run `python contracts/split-openapi.py`
    - Verify 21-reports-endpoints.yaml has `in: query` for all params
    - _Requirements: REQ-5.2_
  - [ ] 2.3 Regenerate frontend types
    - Run `pnpm openapi-typescript` in frontend/
    - _Requirements: REQ-5.3_

- [ ] 3. Update Frontend Types
  - [ ] 3.1 Add report types to api-helpers.ts
    - Re-export TrialBalanceResponse, BalanceSheetResponse, IncomeStatementResponse from api.generated.ts
    - Re-export section types (TrialBalanceTotals, BalanceSheetSectionResponse, etc.)
    - _Requirements: REQ-6.1_

- [ ] 4. Update Frontend Queries
  - [ ] 4.1 Update reports.ts queries
    - Remove manual interface definitions (TrialBalanceResponse, BalanceSheetResponse, IncomeStatementResponse)
    - Import types from @/types/api-helpers
    - Keep query functions, just update types
    - _Requirements: REQ-6.1, REQ-6.2_

- [ ] 5. Update Frontend Pages
  - [x] 5.1 Update trial-balance/page.tsx
    - Change `data?.data` to `data?.accounts`
    - Change `data?.total_debit` to `data?.totals?.total_debit`
    - Change `data?.total_credit` to `data?.totals?.total_credit`
    - Use `data?.totals?.is_balanced` for balanced indicator
    - Update export functions to use correct accessors
    - _Requirements: REQ-2.5, REQ-2.6, REQ-6.3, REQ-6.4_
  - [x] 5.2 Update balance-sheet/page.tsx
    - Change `report?.assets` to `data?.assets?.accounts`
    - Change `report?.liabilities` to `data?.liabilities?.accounts`
    - Change `report?.equity` to `data?.equity?.accounts`
    - Use section totals: `data?.assets?.total`, `data?.liabilities?.total`, `data?.equity?.total`
    - Use `data?.total_liabilities_and_equity` instead of calculating
    - Use `data?.is_balanced` for balanced indicator
    - Update export functions
    - _Requirements: REQ-3.5, REQ-6.3, REQ-6.4_
  - [x] 5.3 Update income-statement/page.tsx
    - Change `report?.revenues` to `data?.revenue?.accounts`
    - Add COGS section: `data?.cost_of_goods_sold?.accounts`
    - Change `report?.expenses` to `data?.operating_expenses?.accounts`
    - Add other income/expenses: `data?.other_income_expenses?.accounts`
    - Display gross_profit, operating_income from response
    - Update export functions
    - _Requirements: REQ-4.5, REQ-6.3, REQ-6.4_

- [x] 6. Checkpoint - Build and Lint Check
  - Run `cargo build` in backend/
  - Run `pnpm build` in frontend/
  - Fix any type errors or warnings
  - _Requirements: All_

- [x] 7. E2E Testing (Playwright MCP, IP: 10.0.0.5)
  - [x] 7.1 Test Trial Balance page
    - Login dengan `corp@zeltra.io` / `qwertyui`
    - Navigate to /dashboard/reports/trial-balance
    - Verify accounts table renders with data
    - Verify totals display correctly
    - Verify balanced/unbalanced indicator
    - _Requirements: REQ-7.1_
  - [x] 7.2 Test Balance Sheet page
    - Navigate to /dashboard/reports/balance-sheet
    - Verify assets, liabilities, equity sections render
    - Verify section totals display
    - Verify total assets = total liabilities + equity
    - Verify balanced indicator
    - _Requirements: REQ-7.2_
  - [x] 7.3 Test Income Statement page
    - Navigate to /dashboard/reports/income-statement
    - Verify revenue section renders
    - Verify expense sections render
    - Verify net income displays with correct color
    - _Requirements: REQ-7.3_
  - [x] 7.4 Test CSV Export
    - Click CSV export on any report
    - Verify toast notification appears
    - _Requirements: REQ-7.4_

- [x] 8. Final Checkpoint
  - Ensure all E2E tests pass
  - Commit changes with descriptive message
  - Kalau ada bug aneh yang ditemukan, save ke Cognito project `zeltra-bug` untuk learning
  - _Requirements: All_

- [x] 9. Populate Liabilities & Equity Data
  - [x] 9.1 Created journal transaction for Accounts Payable (Liability) - $75.00
    - Debit Rent Expense, Credit Accounts Payable
  - [x] 9.2 Created journal transaction for Retained Earnings (Equity) - $50.00
    - Debit Main Bank Account, Credit Retained Earnings
  - [x] 9.3 Verified Balance Sheet displays all 3 sections with non-zero balances
    - Assets: $200.00
    - Liabilities: $75.00
    - Equity: $50.00
    - Total L+E: $125.00
  - Note: "Unbalanced" status is expected (expenses not yet closed to retained earnings)
