# Requirements Document: Sentinel Intelligence UI

## Introduction

Implement the frontend UI for Zeltra's Sentinel Intelligence module, which provides enterprise-grade features for automated accruals, currency revaluation, and intercompany transaction management. The backend APIs are already complete - this spec focuses on building the React/Next.js frontend to consume those APIs.

## Glossary

- **Sentinel**: Zeltra's enterprise intelligence module for advanced accounting automation
- **Accrual_Schedule**: Automated recurring journal entries for prepaid expenses/deferred revenue
- **Revaluation**: Currency adjustment for accounts denominated in non-functional currencies
- **Intercompany_Hub**: Cross-entity transaction matching and elimination system
- **ESG_Metadata**: Environmental, Social, Governance data attached to transactions
- **Tier_Gating**: Feature access control based on subscription tier (Starter/Growth/Enterprise)

## Requirements

### Requirement 1: Accruals Management Page

**User Story:** As an enterprise accountant, I want to manage automated accrual schedules, so that I can automate recurring journal entries for prepaid expenses and deferred revenue.

#### Acceptance Criteria

1. WHEN a user navigates to /dashboard/accruals, THE Accruals_Page SHALL display a list of accrual schedules with columns: Name, Total Amount, Progress, Frequency, Status, Next Run Date
2. WHEN a user clicks "Create Schedule", THE Accruals_Page SHALL open a dialog with form fields: Name, Description, Total Amount, Currency, Debit Account, Credit Account, Start Date, End Date, Frequency, Total Periods
3. WHEN a user submits a valid accrual schedule form, THE System SHALL call POST /organizations/{org_id}/accrual-schedules and display success toast
4. WHEN a user clicks on a schedule row, THE Accruals_Page SHALL display schedule details including processing history
5. IF the user's organization tier does not have `has_auto_accruals` feature, THEN THE Accruals_Page SHALL display an upgrade prompt instead of the schedule list
6. WHEN the accrual list is loading, THE Accruals_Page SHALL display a loading skeleton
7. WHEN the accrual list is empty, THE Accruals_Page SHALL display an empty state with guidance

### Requirement 2: Currency Revaluation Page

**User Story:** As a multi-currency accountant, I want to view and manage currency revaluations, so that I can track unrealized gains/losses from exchange rate fluctuations.

#### Acceptance Criteria

1. WHEN a user navigates to /dashboard/revaluation, THE Revaluation_Page SHALL display a list of revaluation logs with columns: Date, Account, Currency, Old Rate, New Rate, Gain/Loss Amount
2. WHEN displaying gain/loss amounts, THE Revaluation_Page SHALL show gains in green and losses in red
3. WHEN a user filters by date range, THE Revaluation_Page SHALL filter the revaluation logs accordingly
4. IF the user's organization tier does not have `has_multi_currency` feature, THEN THE Revaluation_Page SHALL display an upgrade prompt
5. WHEN the revaluation list is loading, THE Revaluation_Page SHALL display a loading skeleton
6. WHEN the revaluation list is empty, THE Revaluation_Page SHALL display an empty state explaining no revaluations have occurred

### Requirement 3: Intercompany Hub Page

**User Story:** As a consolidation accountant, I want to manage intercompany mappings, so that I can automate elimination entries and cross-entity transaction mirroring.

#### Acceptance Criteria

1. WHEN a user navigates to /dashboard/intercompany, THE Intercompany_Page SHALL display a list of intercompany mappings with columns: Source Org, Source Account, Target Org, Target Account, Mapping Type, Auto-Post Status
2. WHEN a user clicks "Connect Organizations", THE Intercompany_Page SHALL open a dialog to create a new mapping with fields: Source Account, Target Organization, Target Account
3. WHEN a user submits a valid mapping form, THE System SHALL call POST /organizations/{org_id}/intercompany/connect and display success toast
4. IF the user's organization tier does not have `has_intercompany_hub` feature, THEN THE Intercompany_Page SHALL display an upgrade prompt
5. WHEN the mapping list is loading, THE Intercompany_Page SHALL display a loading skeleton
6. WHEN the mapping list is empty, THE Intercompany_Page SHALL display an empty state with setup guidance

### Requirement 4: API Integration Layer

**User Story:** As a developer, I want well-typed React Query hooks for Sentinel APIs, so that I can efficiently fetch and mutate Sentinel data with proper caching.

#### Acceptance Criteria

1. THE System SHALL export Sentinel types (AccrualScheduleResponse, RevaluationLogResponse, IntercompanyMappingResponse) from api-helpers.ts
2. THE System SHALL provide useAccrualSchedules() hook that fetches GET /organizations/{org_id}/accrual-schedules
3. THE System SHALL provide useCreateAccrualSchedule() mutation hook for POST /organizations/{org_id}/accrual-schedules
4. THE System SHALL provide useRevaluationLogs() hook that fetches GET /organizations/{org_id}/revaluation-logs
5. THE System SHALL provide useIntercompanyMappings() hook that fetches GET /organizations/{org_id}/intercompany/mappings
6. THE System SHALL provide useCreateIntercompanyMapping() mutation hook for POST /organizations/{org_id}/intercompany/connect
7. WHEN a mutation succeeds, THE System SHALL invalidate related queries to refresh the list

### Requirement 5: UI/UX Standards

**User Story:** As a user, I want consistent and accessible UI across all Sentinel pages, so that I can efficiently navigate and use the features.

#### Acceptance Criteria

1. THE Sentinel_Pages SHALL follow the existing dashboard design patterns (Card, Table, Dialog components)
2. THE Sentinel_Pages SHALL be fully responsive on mobile and desktop viewports
3. THE Sentinel_Pages SHALL include proper loading states with skeleton loaders
4. THE Sentinel_Pages SHALL include proper error states with retry options
5. THE Sentinel_Pages SHALL include proper empty states with helpful guidance
6. THE Form_Dialogs SHALL validate inputs using Zod schemas before submission
7. THE Sentinel_Pages SHALL support keyboard navigation and screen readers (WCAG 2.1 AA)

### Requirement 6: E2E Testing

**User Story:** As a QA engineer, I want automated E2E tests for Sentinel features, so that I can ensure the UI works correctly with the backend.

#### Acceptance Criteria

1. THE E2E_Tests SHALL verify the Accruals page loads and displays data correctly
2. THE E2E_Tests SHALL verify the Revaluation page loads and displays data correctly
3. THE E2E_Tests SHALL verify the Intercompany page loads and displays data correctly
4. THE E2E_Tests SHALL verify tier gating shows upgrade prompt for non-enterprise users
5. THE E2E_Tests SHALL use Playwright MCP with IP 10.0.0.5 and credentials corp@zeltra.io / qwertyui
