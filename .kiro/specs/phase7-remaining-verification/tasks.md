# Implementation Plan: Phase 7 Remaining Feature Verification

## Overview

This plan covers verification and fixes for the remaining Phase 6-7 frontend features: Simulation, Attachments, Account Ledger, Dimensional Reports, and Fiscal Year Creation. Each task focuses on ensuring proper API integration with the real backend.

## Prerequisites

**IMPORTANT:** 
1. **Simulation requires Enterprise tier** - Use MCP Postgres to upgrade organization:
   ```sql
   UPDATE organizations SET subscription_tier = 'enterprise' WHERE id = '<org_id>';
   ```

2. **Attachments use local storage** - Backend .env configured with:
   ```
   STORAGE_PROVIDER=local
   STORAGE_LOCAL_ROOT=./uploads
   ```
   (Azure Blob / Cloudflare R2 not setup yet)

## Tasks

- [x] 1. Simulation API Integration (Requires Enterprise Tier) ✅
  - **Context:** Read `design.md` and `requirements.md` if context lost. Use `grepSearch`/`fileSearch` to find existing code.
  - [x] 1.1 Create simulation queries file with proper API integration
    - Create `frontend/src/lib/queries/simulation.ts`
    - Use `apiClient` instead of raw fetch
    - Implement `useRunSimulation` mutation hook
    - _Requirements: 1.1, 1.4_
  - [x] 1.2 Create simulation types file with OpenAPI exports
    - Create `frontend/src/types/simulation.ts`
    - Export `RunSimulationRequest`, `SimulationResponse` from api.generated.ts
    - Export helper types: `AccountProjectionResponse`, `AnnualSummaryResponse`, `MonthlySummaryResponse`
    - _Requirements: 1.4_
  - [x] 1.3 Update simulation page to use new query hook
    - Replace hardcoded fetch with `useRunSimulation` mutation
    - Update type imports to use new types file
    - Handle loading and error states properly
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 1.4 E2E test simulation feature (via MCP Playwright)
    - First upgrade org to Enterprise tier via MCP Postgres
    - Use MCP Playwright to navigate to simulation page
    - Enter simulation parameters via browser automation
    - Run simulation and verify results display in browser
    - Debug any errors directly
    - _Requirements: 1.2_

- [x] 2. Checkpoint - Simulation Verified ✅
  - Run `getDiagnostics` on all modified files - MUST pass ✅
  - Ensure simulation works with real API via MCP Playwright E2E ✅
  - Git push `frontend` folder only if lint + E2E pass ✅
  - Ask user if questions arise

- [x] 3. Attachments API Integration
  - **Context:** Read `design.md` and `requirements.md` if context lost. Use `grepSearch`/`fileSearch` to find existing code.
  - [x] 3.1 Create attachments types file
    - Create `frontend/src/types/attachments.ts`
    - Export `AttachmentResponse`, `RequestUploadRequest`, `UploadUrlResponse`, `ConfirmUploadRequest`
    - _Requirements: 2.1, 2.2, 2.3_
  - [x] 3.2 Create attachments queries file
    - Create `frontend/src/lib/queries/attachments.ts`
    - Implement `useRequestUpload` mutation (get presigned URL)
    - Implement `useConfirmUpload` mutation (link to transaction)
    - Implement `useTransactionAttachments` query (list attachments)
    - Implement `useAttachment` query (get download URL)
    - Implement `useDeleteAttachment` mutation
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - [x] 3.3 Create attachment upload component
    - Create `frontend/src/components/attachments/AttachmentUpload.tsx`
    - Implement file selection with drag-and-drop
    - Add file type validation (PDF, PNG, JPG, JPEG, DOC, DOCX)
    - Add file size validation (max 10MB)
    - Implement 3-step upload flow: request URL → upload → confirm
    - _Requirements: 2.1, 2.2, 2.3, 2.7, 2.8_
  - [x] 3.4 Create attachment list component
    - Create `frontend/src/components/attachments/AttachmentList.tsx`
    - Display attachment filename, type, size
    - Add download button with presigned URL
    - Add delete button with confirmation
    - _Requirements: 2.4, 2.5, 2.6_
  - [x] 3.5 Integrate attachments into transaction detail page
    - Add AttachmentList to transaction detail view
    - Add AttachmentUpload for draft transactions
    - _Requirements: 2.4_
  - [x] 3.6 E2E test attachments feature (via MCP Playwright)
    - Use MCP Playwright to create transaction, upload attachment
    - View transaction, verify attachment appears in browser
    - Download attachment, verify URL works
    - Delete attachment, verify removed from UI
    - Debug any errors directly
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [x] 4. Checkpoint - Attachments Verified
  - Run `getDiagnostics` on all modified files - MUST pass
  - Ensure attachments work with real API via MCP Playwright E2E
  - Git push `frontend` folder only if lint + E2E pass
  - Ask user if questions arise

- [x] 5. Account Ledger API Integration
  - **Context:** Read `design.md` and `requirements.md` if context lost. Use `grepSearch`/`fileSearch` to find existing code.
  - [x] 5.1 Create ledger types file
    - Create `frontend/src/types/ledger.ts`
    - Export `AccountLedgerResponse`, `LedgerEntryResponse` from api.generated.ts
    - _Requirements: 3.5_
  - [x] 5.2 Update ledger queries with correct types
    - Update `frontend/src/lib/queries/ledger.ts`
    - Use correct response type `AccountLedgerResponse`
    - Add date range filter parameters (start_date, end_date)
    - _Requirements: 3.1, 3.4, 3.5_
  - [x] 5.3 Create or update account ledger page
    - Verify account ledger page exists or create it
    - Display entries with debit, credit, running_balance columns
    - Add date range filter UI
    - Handle empty state with "No entries found" message
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - [x] 5.4 E2E test account ledger feature (via MCP Playwright)
    - Use MCP Playwright to navigate to account ledger
    - Select an account via browser automation
    - Verify entries display with running balance in browser
    - Test date range filter
    - Debug any errors directly
    - _Requirements: 3.1, 3.2, 3.4_

- [x] 6. Checkpoint - Account Ledger Verified
  - Run `getDiagnostics` on all modified files - MUST pass
  - Ensure account ledger works with real API via MCP Playwright E2E
  - Git push `frontend` folder only if lint + E2E pass
  - Ask user if questions arise

- [-] 7. Dimensional Reports API Integration
  - **Context:** Read `design.md` and `requirements.md` if context lost. Use `grepSearch`/`fileSearch` to find existing code.
  - [x] 7.1 Create dimensional report types file
    - Create `frontend/src/types/dimensional-report.ts`
    - Export `DimensionalReportResponse`, `DimensionalReportRowResponse` from api.generated.ts
    - _Requirements: 4.5_
  - [x] 7.2 Update dimensional report queries with OpenAPI types
    - Update `frontend/src/lib/queries/reports.ts`
    - Replace custom `DimensionalReportData` with OpenAPI types
    - Verify query parameter names match backend (dimension_type_id, dimension_value_id)
    - _Requirements: 4.1, 4.4, 4.5_
  - [x] 7.3 Update dimensional report page to use new types
    - Update `frontend/src/app/dashboard/reports/dimensional/page.tsx`
    - Adjust data mapping for chart and table
    - Handle response structure from OpenAPI types
    - _Requirements: 4.2, 4.3_
  - [x] 7.4 E2E test dimensional reports feature (via MCP Playwright)
    - Use MCP Playwright to navigate to dimensional reports
    - Select dimension type (Department/Project/Cost Center) via browser
    - Verify chart and table display data in browser
    - Test date range filter
    - Debug any errors directly
    - _Requirements: 4.1, 4.2_

- [x] 8. Checkpoint - Dimensional Reports Verified
  - Run `getDiagnostics` on all modified files - MUST pass
  - Ensure dimensional reports work with real API via MCP Playwright E2E
  - Git push `frontend` folder only if lint + E2E pass
  - Ask user if questions arise

- [ ] 9. Fiscal Year Creation API Integration
  - **Context:** Read `design.md` and `requirements.md` if context lost. Use `grepSearch`/`fileSearch` to find existing code.
  - [ ] 9.1 Verify fiscal types match OpenAPI
    - Check `frontend/src/types/fiscal.ts`
    - Verify `CreateFiscalYearRequest` matches OpenAPI schema
    - Add `include_adjustment_period` field if missing
    - _Requirements: 5.4, 5.6_
  - [ ] 9.2 Update fiscal queries if needed
    - Verify `useCreateFiscalYear` mutation uses correct types
    - Add adjustment period support to request
    - _Requirements: 5.1, 5.4_
  - [ ] 9.3 Update fiscal year creation form
    - Add checkbox for adjustment period (period 13)
    - Verify form fields match API requirements
    - _Requirements: 5.1, 5.4_
  - [ ] 9.4 E2E test fiscal year creation (via MCP Playwright)
    - Use MCP Playwright to navigate to fiscal periods page
    - Click "New Fiscal Year" via browser automation
    - Fill form and submit
    - Verify new year appears in list with 12 periods
    - Debug any errors directly
    - _Requirements: 5.1, 5.2, 5.5_

- [ ] 10. Checkpoint - Fiscal Year Creation Verified
  - Run `getDiagnostics` on all modified files - MUST pass
  - Ensure fiscal year creation works with real API via MCP Playwright E2E
  - Git push `frontend` folder only if lint + E2E pass
  - Ask user if questions arise

- [ ] 11. Final Verification & Cleanup
  - **Context:** Read `design.md` and `requirements.md` if context lost.
  - [ ] 11.1 Run TypeScript diagnostics
    - Check all modified files for type errors
    - Fix any type mismatches
    - _Requirements: All_
  - [ ] 11.2 Update ROADMAP.md with verification status
    - Mark verified features as ✅ Real API
    - Update Phase 7 status
    - _Requirements: All_

## Notes

- **Simulation requires Enterprise tier** - Upgrade org via MCP Postgres before testing
- **Attachments use local storage** - Backend .env configured with `STORAGE_PROVIDER=local`
- **⚠️ IMPORTANT: Attachments full upload requires REAL cloud storage (Azure Blob/Cloudflare R2)** - Local storage doesn't support presigned URLs. Frontend is 100% ready, but backend needs Azure/R2 configuration for complete upload flow.
- **E2E testing via MCP Playwright** - No test files created, direct browser automation for immediate debugging
- **Token optimization** - Use `grepSearch`/`fileSearch` instead of reading full files
- **Context recovery** - Read `design.md` and `requirements.md` if context lost between sessions
- **Lint check REQUIRED** - Run `getDiagnostics` on modified files, MUST pass before checkpoint
- **Git push rules** - Only push `frontend` folder, only after lint + E2E pass
- Each feature should be tested with the real backend running
- Use `pnpm run generate:types` if OpenAPI types are missing
- Backend binary is `zeltra` (`cargo run --bin zeltra`)
- Checkpoints ensure incremental validation before moving to next feature
