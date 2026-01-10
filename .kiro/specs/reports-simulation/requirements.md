# Requirements Document

## Introduction

This document specifies the requirements for the Reports & Simulation module of Zeltra - a B2B Expense & Budgeting Engine. This module provides financial reporting capabilities (Trial Balance, Balance Sheet, Income Statement, Account Ledger, Dimensional Reports), budget management with variance analysis, and a simulation engine for financial projections. These features enable organizations to understand their financial position, track budget performance, and forecast future scenarios.

## Glossary

- **Report_Service**: The core service responsible for generating financial reports from ledger data.
- **Budget_Service**: The service responsible for managing budgets, budget lines, and calculating variance against actual ledger data.
- **Simulation_Engine**: The service responsible for projecting future financial states based on historical data and user-defined parameters.
- **Trial_Balance**: A report listing all accounts with their debit and credit totals, verifying that total debits equal total credits.
- **Balance_Sheet**: A point-in-time report showing assets, liabilities, and equity, following the equation: Assets = Liabilities + Equity.
- **Income_Statement**: A period report showing revenue, expenses, and net income (also called Profit & Loss or P&L).
- **Account_Ledger**: A detailed report showing all entries for a specific account with running balance.
- **Dimensional_Report**: A report that slices financial data by dimension values (e.g., expenses by department).
- **Budget**: A financial plan for a fiscal year containing expected amounts per account and period.
- **Budget_Line**: A single line item in a budget specifying the budgeted amount for an account in a specific fiscal period.
- **Variance**: The difference between budgeted and actual amounts (Budgeted - Actual).
- **Favorable_Variance**: When actual results are better than budgeted (lower expenses or higher revenue).
- **Unfavorable_Variance**: When actual results are worse than budgeted (higher expenses or lower revenue).
- **Utilization_Percent**: The percentage of budget consumed (Actual / Budgeted * 100).
- **Baseline**: The historical average or trend used as the starting point for projections.
- **Projection**: A forecasted future value based on baseline and adjustment parameters.
- **Growth_Rate**: A percentage adjustment applied to baseline values for projection (e.g., 0.15 for 15% growth).

## Requirements

### Requirement 1: Budget Management - CRUD Operations

**User Story:** As an accountant, I want to create and manage budgets, so that I can plan and track financial performance against targets.

#### Acceptance Criteria

1. WHEN a user creates a budget with name, fiscal_year_id, budget_type, and currency, THE Budget_Service SHALL create the budget record.
2. WHEN creating a budget, THE Budget_Service SHALL validate that the fiscal_year_id exists and belongs to the same organization.
3. WHEN creating a budget, THE Budget_Service SHALL validate that the currency matches the organization's base_currency.
4. WHEN creating a budget, THE Budget_Service SHALL validate that the budget name is unique within the organization and fiscal year.
5. WHEN listing budgets, THE Budget_Service SHALL return budgets with summary totals (total_budgeted, total_actual, total_variance).
6. WHEN a user locks a budget, THE Budget_Service SHALL set is_locked to true and prevent further modifications to budget lines.
7. WHEN a user attempts to modify a locked budget, THE Budget_Service SHALL reject the modification with a clear error.

### Requirement 2: Budget Lines Management

**User Story:** As an accountant, I want to add budget lines for specific accounts and periods, so that I can define expected amounts at a granular level.

#### Acceptance Criteria

1. WHEN a user creates a budget line with budget_id, account_id, fiscal_period_id, and amount, THE Budget_Service SHALL create the budget line record.
2. WHEN creating a budget line, THE Budget_Service SHALL validate that the account_id exists and belongs to the same organization.
3. WHEN creating a budget line, THE Budget_Service SHALL validate that the fiscal_period_id belongs to the budget's fiscal year.
4. WHEN creating a budget line, THE Budget_Service SHALL validate that the amount is non-negative.
5. WHEN creating a budget line, THE Budget_Service SHALL enforce uniqueness of (budget_id, account_id, fiscal_period_id).
6. WHEN a user creates budget lines in bulk, THE Budget_Service SHALL process all lines in a single transaction.
7. WHEN listing budget lines, THE Budget_Service SHALL return lines with calculated actual amounts from posted ledger entries.

### Requirement 3: Budget Line Dimensions

**User Story:** As an accountant, I want to assign dimensions to budget lines, so that I can track budget performance by department, project, or cost center.

#### Acceptance Criteria

1. WHEN a user creates a budget line with dimension values, THE Budget_Service SHALL validate that all dimension value IDs exist and are active.
2. WHEN a user creates a budget line with dimension values, THE Budget_Service SHALL create budget_line_dimensions records.
3. WHEN calculating actual amounts for a budget line with dimensions, THE Budget_Service SHALL filter ledger entries by matching dimension values.
4. WHEN listing budget lines, THE Budget_Service SHALL include associated dimension values.

### Requirement 4: Budget vs Actual Comparison

**User Story:** As a manager, I want to compare budgeted amounts against actual spending, so that I can identify variances and take corrective action.

#### Acceptance Criteria

1. WHEN a user requests budget vs actual for a budget, THE Budget_Service SHALL calculate actual amounts from posted ledger entries within the period date range.
2. WHEN calculating actual amounts for expense accounts, THE Budget_Service SHALL sum (debit - credit) from ledger entries.
3. WHEN calculating actual amounts for revenue accounts, THE Budget_Service SHALL sum (credit - debit) from ledger entries.
4. WHEN calculating variance, THE Budget_Service SHALL compute (budgeted - actual) for expense accounts.
5. WHEN calculating variance, THE Budget_Service SHALL compute (actual - budgeted) for revenue accounts.
6. WHEN variance is positive for expenses (under budget), THE Budget_Service SHALL mark it as favorable.
7. WHEN variance is negative for expenses (over budget), THE Budget_Service SHALL mark it as unfavorable.
8. WHEN calculating utilization_percent, THE Budget_Service SHALL compute (actual / budgeted * 100), handling zero budget gracefully.
9. WHEN a user filters budget vs actual by dimension, THE Budget_Service SHALL filter both budget lines and actual entries by the specified dimension values.

### Requirement 5: Trial Balance Report

**User Story:** As an accountant, I want to generate a trial balance report, so that I can verify that total debits equal total credits and identify any imbalances.

#### Acceptance Criteria

1. WHEN a user requests a trial balance as of a specific date, THE Report_Service SHALL aggregate all posted ledger entries up to that date.
2. WHEN generating trial balance, THE Report_Service SHALL group entries by account and calculate total_debit and total_credit per account.
3. WHEN calculating account balance, THE Report_Service SHALL apply normal balance rules based on account type.
4. WHEN generating trial balance, THE Report_Service SHALL calculate grand totals for all debits and all credits.
5. WHEN total debits equal total credits, THE Report_Service SHALL mark the trial balance as balanced (is_balanced = true).
6. WHEN a user filters trial balance by dimension, THE Report_Service SHALL include only entries with matching dimension values.
7. WHEN returning trial balance, THE Report_Service SHALL include account_id, code, name, account_type, total_debit, total_credit, and balance for each account.

### Requirement 6: Balance Sheet Report

**User Story:** As a manager, I want to generate a balance sheet report, so that I can understand the organization's financial position at a point in time.

#### Acceptance Criteria

1. WHEN a user requests a balance sheet as of a specific date, THE Report_Service SHALL aggregate all posted ledger entries up to that date.
2. WHEN generating balance sheet, THE Report_Service SHALL group accounts into Assets, Liabilities, and Equity sections.
3. WHEN generating balance sheet, THE Report_Service SHALL further categorize assets into Current Assets and Fixed Assets based on account_subtype.
4. WHEN generating balance sheet, THE Report_Service SHALL further categorize liabilities into Current Liabilities and Long-term Liabilities based on account_subtype.
5. WHEN generating balance sheet, THE Report_Service SHALL calculate section totals (total_assets, total_liabilities, total_equity).
6. WHEN generating balance sheet, THE Report_Service SHALL verify that Assets = Liabilities + Equity (is_balanced = true).
7. WHEN returning balance sheet, THE Report_Service SHALL include hierarchical structure with sections, subsections, and individual accounts.

### Requirement 7: Income Statement Report

**User Story:** As a manager, I want to generate an income statement (P&L) report, so that I can understand profitability over a period.

#### Acceptance Criteria

1. WHEN a user requests an income statement for a date range, THE Report_Service SHALL aggregate posted ledger entries within that range.
2. WHEN generating income statement, THE Report_Service SHALL group accounts into Revenue, Cost of Goods Sold, Operating Expenses, and Other Income/Expense sections.
3. WHEN calculating revenue totals, THE Report_Service SHALL sum (credit - debit) for revenue accounts.
4. WHEN calculating expense totals, THE Report_Service SHALL sum (debit - credit) for expense accounts.
5. WHEN generating income statement, THE Report_Service SHALL calculate Gross Profit = Revenue - Cost of Goods Sold.
6. WHEN generating income statement, THE Report_Service SHALL calculate Operating Income = Gross Profit - Operating Expenses.
7. WHEN generating income statement, THE Report_Service SHALL calculate Net Income = Operating Income + Other Income - Other Expenses.
8. WHEN a user filters income statement by dimension, THE Report_Service SHALL include only entries with matching dimension values.

### Requirement 8: Account Ledger Report

**User Story:** As an accountant, I want to view all entries for a specific account with running balance, so that I can audit account activity.

#### Acceptance Criteria

1. WHEN a user requests an account ledger for an account_id and date range, THE Report_Service SHALL return all ledger entries for that account within the range.
2. WHEN returning account ledger entries, THE Report_Service SHALL include transaction_id, transaction_date, description, source_currency, source_amount, exchange_rate, functional_amount, debit, credit, and running_balance.
3. WHEN calculating running_balance, THE Report_Service SHALL use the current_balance stored on each ledger entry.
4. WHEN returning account ledger entries, THE Report_Service SHALL include dimension values associated with each entry.
5. WHEN returning account ledger, THE Report_Service SHALL support pagination with page and limit parameters.
6. WHEN returning account ledger, THE Report_Service SHALL order entries by transaction_date and entry creation order.

### Requirement 9: Dimensional Report

**User Story:** As a manager, I want to analyze financial data by dimension (department, project, cost center), so that I can understand spending patterns across organizational units.

#### Acceptance Criteria

1. WHEN a user requests a dimensional report with group_by dimensions, THE Report_Service SHALL aggregate ledger entries grouped by the specified dimension types.
2. WHEN generating dimensional report, THE Report_Service SHALL support grouping by multiple dimension types simultaneously (e.g., DEPARTMENT and PROJECT).
3. WHEN generating dimensional report, THE Report_Service SHALL calculate total_debit, total_credit, and balance for each dimension combination.
4. WHEN a user filters dimensional report by account_type, THE Report_Service SHALL include only accounts of that type.
5. WHEN a user filters dimensional report by specific dimension values, THE Report_Service SHALL include only entries with those dimension values.
6. WHEN returning dimensional report, THE Report_Service SHALL include dimension type, code, and name for each grouping.
7. WHEN returning dimensional report, THE Report_Service SHALL calculate grand_total across all dimension combinations.

### Requirement 10: Simulation Engine - Historical Data Aggregation

**User Story:** As a financial analyst, I want to aggregate historical data as a baseline for projections, so that forecasts are grounded in actual performance.

#### Acceptance Criteria

1. WHEN a user specifies a base period (start_date, end_date), THE Simulation_Engine SHALL aggregate posted ledger entries within that period.
2. WHEN aggregating historical data, THE Simulation_Engine SHALL group entries by account and calculate monthly totals.
3. WHEN calculating baseline, THE Simulation_Engine SHALL compute the average monthly amount per account.
4. WHEN a user filters by dimensions, THE Simulation_Engine SHALL include only entries with matching dimension values in the baseline calculation.
5. WHEN an account has no entries in the base period, THE Simulation_Engine SHALL use zero as the baseline.

### Requirement 11: Simulation Engine - Projection Calculation

**User Story:** As a financial analyst, I want to project future financial states with adjustable growth rates, so that I can model different scenarios.

#### Acceptance Criteria

1. WHEN a user specifies projection_months, THE Simulation_Engine SHALL generate projections for that many future months.
2. WHEN projecting revenue accounts, THE Simulation_Engine SHALL apply the revenue_growth_rate to the baseline.
3. WHEN projecting expense accounts, THE Simulation_Engine SHALL apply the expense_growth_rate to the baseline.
4. WHEN a user specifies account_adjustments for specific accounts, THE Simulation_Engine SHALL use those rates instead of global rates.
5. WHEN calculating projected amounts, THE Simulation_Engine SHALL apply compound growth: baseline * (1 + rate)^month.
6. WHEN returning projections, THE Simulation_Engine SHALL include period_name, period_start, period_end, baseline_amount, and projected_amount for each account and period.
7. WHEN returning projections, THE Simulation_Engine SHALL calculate summary totals: total_projected_revenue, total_projected_expenses, projected_net_income.

### Requirement 12: Simulation Engine - Performance

**User Story:** As a system, I want simulation calculations to complete quickly, so that users can interactively explore different scenarios.

#### Acceptance Criteria

1. WHEN running simulation with many accounts, THE Simulation_Engine SHALL use parallel processing (Rayon) to compute projections concurrently.
2. WHEN simulation parameters are identical to a previous run, THE Simulation_Engine SHALL return cached results if available.
3. WHEN caching simulation results, THE Simulation_Engine SHALL use a hash of the parameters as the cache key.
4. WHEN simulation completes, THE Simulation_Engine SHALL return results within 2 seconds for a 12-month projection with 100+ accounts.

### Requirement 13: Budget API Endpoints

**User Story:** As a frontend developer, I want REST API endpoints for budget management, so that I can build the budget UI.

#### Acceptance Criteria

1. WHEN a POST request is made to /budgets, THE API SHALL create a new budget and return the budget record.
2. WHEN a GET request is made to /budgets, THE API SHALL return a paginated list of budgets with summary totals.
3. WHEN a GET request is made to /budgets/:id, THE API SHALL return the budget with all budget lines.
4. WHEN a POST request is made to /budgets/:id/lines, THE API SHALL create budget lines in bulk.
5. WHEN a GET request is made to /budgets/:id/lines, THE API SHALL return budget lines with actual amounts.
6. WHEN a POST request is made to /budgets/:id/lock, THE API SHALL lock the budget.
7. WHEN a GET request is made to /budgets/:id/vs-actual, THE API SHALL return budget vs actual comparison with variance analysis.

### Requirement 14: Report API Endpoints

**User Story:** As a frontend developer, I want REST API endpoints for financial reports, so that I can build the reports UI.

#### Acceptance Criteria

1. WHEN a GET request is made to /reports/trial-balance with as_of parameter, THE API SHALL return the trial balance report.
2. WHEN a GET request is made to /reports/balance-sheet with as_of parameter, THE API SHALL return the balance sheet report.
3. WHEN a GET request is made to /reports/income-statement with from and to parameters, THE API SHALL return the income statement report.
4. WHEN a GET request is made to /accounts/:id/ledger with from, to, page, and limit parameters, THE API SHALL return the account ledger report.
5. WHEN a GET request is made to /reports/dimensional with from, to, group_by, and optional filters, THE API SHALL return the dimensional report.
6. WHEN report endpoints receive a dimension filter parameter, THE API SHALL filter results by the specified dimension values.

### Requirement 15: Simulation API Endpoints

**User Story:** As a frontend developer, I want REST API endpoints for simulation, so that I can build the forecasting UI.

#### Acceptance Criteria

1. WHEN a POST request is made to /simulation/run with base_period, projection_months, and parameters, THE API SHALL run the simulation and return projections.
2. WHEN returning simulation results, THE API SHALL include simulation_id, parameters_hash, projections array, and annual_summary.
3. WHEN simulation results are cached, THE API SHALL include cached: true in the response.
4. WHEN simulation parameters are invalid, THE API SHALL return a 400 error with details about the validation failure.

### Requirement 16: Dashboard Metrics API

**User Story:** As a frontend developer, I want a dashboard metrics API, so that I can display key financial indicators on the dashboard.

#### Acceptance Criteria

1. WHEN a GET request is made to /dashboard/metrics with period_id, THE API SHALL return aggregated dashboard metrics.
2. WHEN calculating cash_position, THE API SHALL sum balances of all cash and bank accounts (account_subtype = 'cash', 'bank').
3. WHEN calculating burn_rate, THE API SHALL compute total expenses divided by days in the period.
4. WHEN calculating runway_days, THE API SHALL compute cash_position divided by daily burn_rate.
5. WHEN calculating pending_approvals, THE API SHALL count and sum transactions with status = 'pending'.
6. WHEN calculating budget_status, THE API SHALL return total_budgeted, total_spent, and utilization_percent for the period.
7. WHEN calculating top_expenses_by_department, THE API SHALL return the top N departments by expense amount.
8. WHEN calculating currency_exposure, THE API SHALL return balances grouped by currency with functional value equivalents.

### Requirement 17: Recent Activity API

**User Story:** As a frontend developer, I want a recent activity API, so that I can display a feed of recent financial events on the dashboard.

#### Acceptance Criteria

1. WHEN a GET request is made to /dashboard/recent-activity with limit parameter, THE API SHALL return recent activity events.
2. WHEN returning activity events, THE API SHALL include transaction events (created, submitted, approved, rejected, posted, voided).
3. WHEN returning activity events, THE API SHALL include budget events (created, updated, locked).
4. WHEN returning activity events, THE API SHALL include user information (id, full_name) for each event.
5. WHEN returning activity events, THE API SHALL order by timestamp descending (most recent first).
6. WHEN returning activity events, THE API SHALL support cursor-based pagination with has_more and next_cursor.
