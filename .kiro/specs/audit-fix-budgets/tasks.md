# Implementation Tasks: Audit & Fix 04-Budgets Schema

## Notes

- **E2E Testing**: Use Playwright MCP (bukan script), IP `10.0.0.5` bukan localhost
- **Login Credentials**: `corp@zeltra.io` / `qwertyui`
- **Lint Check**: Setiap selesai coding, cek Problems panel (getDiagnostics)
- **Search Feature**: Gunakan search/filter di UI untuk testing
- **Root Cause**: Backend response tidak sesuai OpenAPI annotations (utoipa generates from annotations, not runtime code)

## Tasks

- [x] 1. Fix Backend Utoipa Annotations
  - [x] 1.1 Add wrapper response structs (GetBudgetsResponse, GetBudgetLinesResponse, BudgetLineResponse)
  - [x] 1.2 Update list_budgets annotation
  - [x] 1.3 Update list_budget_lines annotation
  - [x] 1.4 Update create_budget_lines annotation
  - [x] 1.5 Fix BudgetVsActualQuery params
  - [x] 1.6 Fix get_budget_vs_actual response fields

- [x] 2. Regenerate OpenAPI and Frontend Types
  - [x] 2.1 Regenerate OpenAPI spec
  - [x] 2.2 Run split-openapi script
  - [x] 2.3 Regenerate frontend types

- [x] 3. Update Frontend Types and Queries
  - [x] 3.1 Update frontend/src/types/budgets.ts
  - [x] 3.2 Update frontend/src/lib/queries/budgets.ts

- [x] 4. Update Frontend Pages
  - [x] 4.1 Update frontend/src/app/dashboard/budgets/page.tsx
  - [x] 4.2 Update frontend/src/app/dashboard/budgets/[id]/page.tsx

- [x] 5. Checkpoint - Build and Lint Check
  - Backend build: ✅
  - Frontend build: ✅

- [x] 6. E2E Testing (Playwright MCP, IP: 10.0.0.5)
  - [x] 6.1 Test budget list flow - ✅ Page renders, summary cards work
  - [x] 6.2 Test create budget flow - ✅ Created "E2E Test Budget 2026"
  - [x] 6.3 Test budget detail flow - ✅ Detail page renders with budget info

- [x] 7. UI/UX Verification
  - [x] 7.1 Verify loading states - ✅ Pages load correctly
  - [x] 7.2 Verify error handling - ✅ Toast notifications work
  - [x] 7.3 Verify data refresh - ✅ List updates after create

- [x] 8. Final Checkpoint
  - All E2E tests passed
  - Ready for commit
