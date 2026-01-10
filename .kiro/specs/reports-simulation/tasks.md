# Implementation Plan: Reports & Simulation

## Overview

This implementation plan covers Phase 4 of Zeltra Backend - Reports & Simulation module. The plan follows a layered approach: core business logic first, then repository layer, then API endpoints. Property-based tests are integrated throughout to ensure correctness.

## Tasks

- [x] 1. Set up module structure and dependencies
  - Create directory structure in core, db, and api crates
  - Add dependencies: rayon, moka, proptest
  - Create mod.rs files for budget, reports, simulation, dashboard modules
  - _Requirements: All_

- [x] 2. Implement Budget Core Types and Service
  - [x] 2.1 Create budget types (Budget, BudgetLine, BudgetType, VarianceResult, VarianceStatus)
    - Define all structs and enums in `core/src/budget/types.rs`
    - Implement Serialize/Deserialize for API responses
    - _Requirements: 1.1, 2.1, 4.1_

  - [x] 2.2 Create budget error types
    - Define BudgetError enum in `core/src/budget/error.rs`
    - Include: NotFound, BudgetLocked, DuplicateName, NegativeAmount, etc.
    - _Requirements: 1.7, 2.4, 2.5_

  - [x] 2.3 Implement BudgetService variance calculation
    - Implement `calculate_variance(budgeted, actual, account_type)` function
    - Handle expense vs revenue variance logic
    - Calculate utilization_percent with zero-budget handling
    - _Requirements: 4.4, 4.5, 4.6, 4.7, 4.8_

  - [x] 2.4 Write property tests for variance calculation
    - **Property 4: Budget Variance Calculation by Account Type**
    - **Property 6: Utilization Percent Calculation**
    - **Validates: Requirements 4.4, 4.5, 4.6, 4.8**

  - [x] 2.5 Implement BudgetService validation functions
    - Implement `validate_budget_line(budget, account_id, fiscal_period_id, amount)`
    - Check locked status, negative amounts
    - _Requirements: 1.6, 1.7, 2.4_

  - [x] 2.6 Write property test for budget lock validation
    - **Property 7: Budget Lock Prevents Modification**
    - **Validates: Requirements 1.6, 1.7**

- [x] 3. Checkpoint - Budget Core Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]` unless absolutely necessary)
  - Ensure all budget core tests pass with `cargo test`
  - **CONTEXT REFRESH**: Re-read `.kiro/specs/reports-simulation/design.md` and `requirements.md` before continuing
  - Ask the user if questions arise

- [x] 4. Implement Report Core Types and Service
  - [x] 4.1 Create report types (AccountBalance, TrialBalanceReport, BalanceSheetReport, IncomeStatementReport)
    - Define all structs in `core/src/reports/types.rs`
    - Include section types (BalanceSheetSection, IncomeStatementSection)
    - _Requirements: 5.7, 6.7, 7.1_

  - [x] 4.2 Create report error types
    - Define ReportError enum in `core/src/reports/error.rs`
    - Include: AccountNotFound, InvalidDateRange, NoDataFound, etc.
    - _Requirements: 5.1, 7.1_

  - [x] 4.3 Implement ReportService trial balance generation
    - Implement `generate_trial_balance(accounts)` function
    - Calculate total_debit, total_credit, is_balanced
    - _Requirements: 5.2, 5.4, 5.5_

  - [x] 4.4 Write property test for trial balance
    - **Property 1: Trial Balance Debits Equal Credits**
    - **Validates: Requirements 5.4, 5.5**

  - [x] 4.5 Implement ReportService balance sheet generation
    - Implement `generate_balance_sheet(accounts)` function
    - Group accounts into Assets, Liabilities, Equity sections
    - Calculate section totals and verify accounting equation
    - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.6_

  - [x] 4.6 Write property test for balance sheet
    - **Property 2: Balance Sheet Accounting Equation**
    - **Validates: Requirements 6.5, 6.6**

  - [x] 4.7 Implement ReportService income statement generation
    - Implement `generate_income_statement(accounts)` function
    - Group into Revenue, COGS, Operating Expenses, Other sections
    - Calculate Gross Profit, Operating Income, Net Income
    - _Requirements: 7.2, 7.3, 7.4, 7.5, 7.6, 7.7_

  - [x] 4.8 Write property test for income statement
    - **Property 3: Income Statement Net Income Calculation**
    - **Validates: Requirements 7.5, 7.6, 7.7**

- [x] 5. Checkpoint - Report Core Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]` unless absolutely necessary)
  - Ensure all report core tests pass with `cargo test`
  - **CONTEXT REFRESH**: Re-read `.kiro/specs/reports-simulation/design.md` and `requirements.md` before continuing
  - Ask the user if questions arise

- [x] 6. Implement Simulation Engine
  - [x] 6.1 Create simulation types (SimulationParams, HistoricalAccountData, AccountProjection, SimulationResult)
    - Define all structs in `core/src/simulation/types.rs`
    - Include AnnualSummary for totals
    - _Requirements: 10.1, 11.1, 11.6_

  - [x] 6.2 Create simulation error types
    - Define SimulationError enum in `core/src/simulation/error.rs`
    - Include: InvalidBasePeriod, InvalidProjectionMonths, InvalidGrowthRate, NoHistoricalData
    - _Requirements: 11.1_

  - [x] 6.3 Implement SimulationEngine baseline calculation
    - Implement `calculate_baseline(monthly_amounts)` function
    - Calculate average monthly amount, handle empty data
    - _Requirements: 10.3, 10.5_

  - [x] 6.4 Write property test for baseline calculation
    - **Property 18: Simulation Baseline Calculation**
    - **Validates: Requirements 10.3**

  - [x] 6.5 Implement SimulationEngine projection calculation
    - Implement `project_account(data, params)` function
    - Apply compound growth formula: baseline * (1 + rate)^month
    - Support account-specific growth rate overrides
    - _Requirements: 11.2, 11.3, 11.4, 11.5_

  - [x] 6.6 Write property tests for projection calculation
    - **Property 9: Simulation Projection Count**
    - **Property 10: Simulation Compound Growth Formula**
    - **Property 11: Simulation Growth Rate Override**
    - **Validates: Requirements 11.1, 11.4, 11.5**

  - [x] 6.7 Implement SimulationEngine parallel run
    - Implement `run(historical_data, params)` function
    - Use Rayon `par_iter()` for parallel account processing
    - Calculate summary totals (total_projected_revenue, total_projected_expenses, projected_net_income)
    - _Requirements: 11.7, 12.1_

  - [x] 6.8 Write property test for simulation summary totals
    - **Property 12: Simulation Summary Totals**
    - **Validates: Requirements 11.7**

  - [x] 6.9 Implement simulation caching with Moka
    - Create cache with TTL and max capacity
    - Hash simulation parameters for cache key
    - Return cached results when available
    - _Requirements: 12.2, 12.3_

- [x] 7. Checkpoint - Simulation Engine Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]` unless absolutely necessary)
  - Ensure all simulation tests pass with `cargo test`
  - **CONTEXT REFRESH**: Re-read `.kiro/specs/reports-simulation/design.md` and `requirements.md` before continuing
  - Ask the user if questions arise

- [ ] 8. Implement Dashboard Types
  - [ ] 8.1 Create dashboard types (DashboardMetrics, CashPosition, BurnRate, BudgetStatus, etc.)
    - Define all structs in `core/src/dashboard/types.rs`
    - Include ActivityEvent, RecentActivityResponse
    - _Requirements: 16.1, 17.1_

- [ ] 9. Implement Budget Repository
  - [ ] 9.1 Create budget SeaORM entities
    - Generate or create entities for budgets, budget_lines, budget_line_dimensions tables
    - _Requirements: 1.1, 2.1, 3.1_

  - [ ] 9.2 Implement BudgetRepository CRUD operations
    - Implement create_budget, get_budget, list_budgets, update_budget, lock_budget
    - Include summary totals calculation in list query
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

  - [ ] 9.3 Implement BudgetRepository budget line operations
    - Implement create_budget_lines (bulk), get_budget_lines, update_budget_line, delete_budget_line
    - Enforce uniqueness constraint
    - _Requirements: 2.1, 2.2, 2.3, 2.5, 2.6_

  - [ ] 9.4 Write property test for budget line uniqueness
    - **Property 8: Budget Line Uniqueness**
    - **Validates: Requirements 2.5**

  - [ ] 9.5 Implement BudgetRepository dimension operations
    - Implement create_budget_line_dimensions, get_budget_line_dimensions
    - Validate dimension value IDs
    - _Requirements: 3.1, 3.2, 3.4_

  - [ ] 9.6 Implement BudgetRepository actual amount calculation
    - Query posted ledger entries for budget period
    - Apply account type rules (debit-credit vs credit-debit)
    - Filter by dimensions when specified
    - _Requirements: 4.1, 4.2, 4.3, 4.9_

  - [ ] 9.7 Write property test for actual amount calculation
    - **Property 5: Actual Amount Calculation by Account Type**
    - **Validates: Requirements 4.2, 4.3**

- [ ] 10. Checkpoint - Budget Repository Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]` unless absolutely necessary)
  - Ensure all budget repository tests pass with `cargo test`
  - **CONTEXT REFRESH**: Re-read `.kiro/specs/reports-simulation/design.md` and `requirements.md` before continuing
  - Ask the user if questions arise

- [ ] 11. Implement Report Repository
  - [ ] 11.1 Implement ReportRepository trial balance query
    - Query trial_balance_view or aggregate ledger entries
    - Support as_of date filter
    - Support dimension filter
    - _Requirements: 5.1, 5.2, 5.6_

  - [ ] 11.2 Implement ReportRepository balance sheet query
    - Query accounts with balances as of date
    - Include account_type and account_subtype for categorization
    - _Requirements: 6.1, 6.3, 6.4_

  - [ ] 11.3 Implement ReportRepository income statement query
    - Query revenue and expense accounts for date range
    - Include account_subtype for COGS vs Operating Expense categorization
    - _Requirements: 7.1, 7.2_

  - [ ] 11.4 Implement ReportRepository account ledger query
    - Query ledger entries for specific account and date range
    - Include running_balance from current_balance field
    - Include dimension values via joins
    - Support pagination
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_

  - [ ] 11.5 Write property tests for account ledger
    - **Property 13: Account Ledger Running Balance**
    - **Property 14: Account Ledger Ordering**
    - **Validates: Requirements 8.3, 8.6**

  - [ ] 11.6 Implement ReportRepository dimensional report query
    - Query ledger entries grouped by dimension values
    - Support multiple group_by dimensions
    - Calculate totals per dimension combination
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_

  - [ ] 11.7 Write property tests for dimensional report
    - **Property 15: Dimensional Report Grand Total**
    - **Property 16: Dimensional Report Grouping**
    - **Validates: Requirements 9.7, 9.1, 9.2**

- [ ] 12. Implement Simulation Repository
  - [ ] 12.1 Implement SimulationRepository historical data query
    - Query posted ledger entries for base period
    - Group by account and month
    - Support dimension filters
    - _Requirements: 10.1, 10.2, 10.4_

- [ ] 13. Implement Dashboard Repository
  - [ ] 13.1 Implement DashboardRepository metrics queries
    - Query cash position (sum of cash/bank accounts)
    - Query pending approvals count and total
    - Query budget status from budget_vs_actual_view
    - Query top expenses by department
    - Query currency exposure
    - _Requirements: 16.2, 16.3, 16.4, 16.5, 16.6, 16.7, 16.8_

  - [ ] 13.2 Implement DashboardRepository recent activity query
    - Query audit_logs for transaction and budget events
    - Include user information via join
    - Support cursor-based pagination
    - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6_

- [ ] 14. Checkpoint - All Repositories Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]` unless absolutely necessary)
  - Ensure all repository tests pass with `cargo test`
  - **CONTEXT REFRESH**: Re-read `.kiro/specs/reports-simulation/design.md` and `requirements.md` before continuing
  - Ask the user if questions arise

- [ ] 15. Implement Budget API Routes
  - [ ] 15.1 Implement POST /budgets endpoint
    - Create budget with validation
    - Return created budget
    - _Requirements: 13.1_

  - [ ] 15.2 Implement GET /budgets endpoint
    - List budgets with pagination
    - Include summary totals
    - _Requirements: 13.2_

  - [ ] 15.3 Implement GET /budgets/:id endpoint
    - Get budget with all budget lines
    - _Requirements: 13.3_

  - [ ] 15.4 Implement POST /budgets/:id/lines endpoint
    - Create budget lines in bulk
    - Validate all lines in transaction
    - _Requirements: 13.4_

  - [ ] 15.5 Implement GET /budgets/:id/lines endpoint
    - Get budget lines with actual amounts
    - _Requirements: 13.5_

  - [ ] 15.6 Implement POST /budgets/:id/lock endpoint
    - Lock budget
    - _Requirements: 13.6_

  - [ ] 15.7 Implement GET /budgets/:id/vs-actual endpoint
    - Get budget vs actual comparison
    - Include variance analysis
    - _Requirements: 13.7_

- [ ] 16. Implement Report API Routes
  - [ ] 16.1 Implement GET /reports/trial-balance endpoint
    - Accept as_of query parameter
    - Support dimension filter
    - _Requirements: 14.1, 14.6_

  - [ ] 16.2 Implement GET /reports/balance-sheet endpoint
    - Accept as_of query parameter
    - _Requirements: 14.2_

  - [ ] 16.3 Implement GET /reports/income-statement endpoint
    - Accept from and to query parameters
    - Support dimension filter
    - _Requirements: 14.3, 14.6_

  - [ ] 16.4 Implement GET /accounts/:id/ledger endpoint
    - Accept from, to, page, limit query parameters
    - _Requirements: 14.4_

  - [ ] 16.5 Implement GET /reports/dimensional endpoint
    - Accept from, to, group_by query parameters
    - Support dimension and account_type filters
    - _Requirements: 14.5, 14.6_

- [ ] 17. Implement Simulation API Routes
  - [ ] 17.1 Implement POST /simulation/run endpoint
    - Accept base_period, projection_months, growth rates, account_adjustments
    - Return projections with summary
    - Include cached flag
    - _Requirements: 15.1, 15.2, 15.3, 15.4_

- [ ] 18. Implement Dashboard API Routes
  - [ ] 18.1 Implement GET /dashboard/metrics endpoint
    - Accept period_id query parameter
    - Return all dashboard metrics
    - _Requirements: 16.1_

  - [ ] 18.2 Implement GET /dashboard/recent-activity endpoint
    - Accept limit and cursor query parameters
    - Return activity events with pagination
    - _Requirements: 17.1_

- [ ] 19. Checkpoint - All API Routes Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]` unless absolutely necessary)
  - Ensure all API tests pass with `cargo test`
  - **CONTEXT REFRESH**: Re-read `.kiro/specs/reports-simulation/design.md` and `requirements.md` before continuing
  - Ask the user if questions arise

- [ ] 20. Integration Tests
  - [ ] 20.1 Write budget workflow integration tests
    - Test: create budget → add lines → lock → query vs actual
    - _Requirements: 1.1-1.7, 2.1-2.7, 4.1-4.9_

  - [ ] 20.2 Write report generation integration tests
    - Test: post transactions → generate trial balance, balance sheet, income statement
    - Verify accounting equation holds
    - _Requirements: 5.1-5.7, 6.1-6.7, 7.1-7.8_

  - [ ] 20.3 Write simulation integration tests
    - Test: historical data → run simulation → verify projections
    - Test caching behavior
    - _Requirements: 10.1-10.5, 11.1-11.7, 12.1-12.4_

  - [ ] 20.4 Write dashboard integration tests
    - Test: metrics calculation with real data
    - Test: recent activity pagination
    - _Requirements: 16.1-16.8, 17.1-17.6_

- [ ] 21. Final Checkpoint - Phase 4 Complete
  - Run `cargo fmt` to format all code
  - Run `cargo clippy -- -D warnings` and fix ALL warnings (NO `#[allow(dead_code)]`)
  - Ensure ALL tests pass with `cargo test` (target: 50+ new tests)
  - Update PROGRESS.md with Phase 4 completion status
  - Verify all API endpoints match contracts/openapi.yaml
  - **FINAL CONTEXT CHECK**: Verify implementation matches design.md and all requirements.md criteria are met

## Notes

- All tasks including property-based tests are required (comprehensive testing)
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- Use `proptest` crate for property-based testing
- Use `moka` crate for simulation caching
- Use `rayon` crate for parallel simulation processing

## Important Rules

1. **At every checkpoint**:
   - Run `cargo fmt` to format code
   - Run `cargo clippy -- -D warnings` and fix ALL warnings
   - NO `#[allow(dead_code)]` unless absolutely necessary with justification
   - Re-read `design.md` and `requirements.md` to refresh context

2. **Before starting each major task group (2, 4, 6, 9, 11, 15, 16, 17, 18, 20)**:
   - Re-read relevant sections of `design.md` and `requirements.md`
   - Don't lose context - we have plenty of time

3. **Code quality**:
   - NO `unwrap()` in production code
   - NO unnecessary `.clone()`
   - Use `rust_decimal::Decimal` for all money calculations
   - Follow existing codebase patterns
