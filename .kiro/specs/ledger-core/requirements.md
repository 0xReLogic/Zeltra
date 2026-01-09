# Requirements Document

## Introduction

This document specifies the requirements for the Ledger Core module of Zeltra - a B2B Expense & Budgeting Engine. The Ledger Core is the most critical component of the system, implementing double-entry bookkeeping with multi-currency support, dimensional accounting, and strict fiscal period management. This module must ensure absolute data integrity - if the ledger is wrong, all reports and dashboards become meaningless.

## Glossary

- **Ledger_Service**: The core service responsible for creating, validating, and managing financial transactions using double-entry bookkeeping principles.
- **Transaction**: A financial event consisting of two or more ledger entries where total debits must equal total credits in functional currency.
- **Ledger_Entry**: A single line item within a transaction, representing either a debit or credit to a specific account.
- **Account**: A record in the Chart of Accounts representing a category for tracking financial activity (asset, liability, equity, revenue, or expense).
- **Fiscal_Period**: A defined time range (typically monthly) within a fiscal year that controls when transactions can be posted.
- **Fiscal_Year**: A 12-month accounting period containing multiple fiscal periods.
- **Dimension**: A classification tag (e.g., Department, Project, Cost Center) that can be attached to ledger entries for analytical reporting.
- **Dimension_Value**: A specific value within a dimension type (e.g., "Engineering" within "Department").
- **Exchange_Rate**: The conversion rate between two currencies on a specific date.
- **Functional_Currency**: The organization's base currency used for consolidated reporting (stored in `organizations.base_currency`).
- **Source_Currency**: The original currency of a transaction before conversion.
- **Source_Amount**: The original amount in the source currency.
- **Functional_Amount**: The converted amount in the organization's functional currency.
- **Debit**: An entry that increases asset/expense accounts or decreases liability/equity/revenue accounts.
- **Credit**: An entry that decreases asset/expense accounts or increases liability/equity/revenue accounts.
- **Normal_Balance**: The side (debit or credit) that increases an account based on its type.
- **Account_Version**: A monotonically increasing counter per account used for optimistic locking and historical balance tracking.
- **Running_Balance**: The account balance after each entry, enabling point-in-time balance queries.
- **Banker_Rounding**: IEEE 754 rounding strategy where midpoint values round to the nearest even number (also called half-even rounding).
- **Largest_Remainder_Method**: An allocation algorithm that distributes amounts fairly while ensuring the sum exactly equals the original total.

## Requirements

### Requirement 1: Master Data Management - Fiscal Years and Periods

**User Story:** As an accountant, I want to manage fiscal years and periods, so that I can control when transactions can be posted and organize financial data by time periods.

#### Acceptance Criteria

1. WHEN a user creates a fiscal year with name, start_date, and end_date, THE Ledger_Service SHALL create the fiscal year record and auto-generate monthly fiscal periods.
2. WHEN a fiscal year is created, THE Ledger_Service SHALL validate that start_date is before end_date.
3. WHEN a fiscal year is created, THE Ledger_Service SHALL validate that the date range does not overlap with existing fiscal years for the same organization.
4. WHEN listing fiscal years, THE Ledger_Service SHALL return fiscal years with their nested fiscal periods.
5. WHEN a user changes a fiscal period status to SOFT_CLOSE, THE Ledger_Service SHALL allow only users with accountant, admin, or owner roles to post transactions to that period.
6. WHEN a user changes a fiscal period status to CLOSED, THE Ledger_Service SHALL prevent all posting to that period regardless of user role.
7. WHEN a user attempts to close a fiscal period, THE Ledger_Service SHALL validate that all earlier periods in the same fiscal year are already closed.

### Requirement 2: Master Data Management - Chart of Accounts

**User Story:** As an accountant, I want to manage the chart of accounts, so that I can categorize and track all financial activities.

#### Acceptance Criteria

1. WHEN a user creates an account with code, name, account_type, and currency, THE Ledger_Service SHALL create the account record.
2. WHEN creating an account, THE Ledger_Service SHALL validate that the account code is unique within the organization.
3. WHEN creating an account, THE Ledger_Service SHALL validate that the currency exists in the currencies table.
4. WHEN creating an account with a parent_id, THE Ledger_Service SHALL validate that the parent account exists and belongs to the same organization.
5. WHEN listing accounts, THE Ledger_Service SHALL return accounts with their current balance calculated from posted ledger entries.
6. WHEN a user updates an account, THE Ledger_Service SHALL prevent changing the account_type if the account has any ledger entries.
7. WHEN a user deactivates an account, THE Ledger_Service SHALL prevent new transactions from posting to that account.

### Requirement 3: Master Data Management - Dimensions

**User Story:** As an accountant, I want to manage dimension types and values, so that I can tag transactions for analytical reporting by department, project, or cost center.

#### Acceptance Criteria

1. WHEN a user creates a dimension type with code and name, THE Ledger_Service SHALL create the dimension type record.
2. WHEN creating a dimension type, THE Ledger_Service SHALL validate that the code is unique within the organization.
3. WHEN a user creates a dimension value with code, name, and dimension_type_id, THE Ledger_Service SHALL create the dimension value record.
4. WHEN creating a dimension value, THE Ledger_Service SHALL validate that the code is unique within the dimension type.
5. WHEN creating a dimension value with a parent_id, THE Ledger_Service SHALL validate that the parent belongs to the same dimension type.
6. WHEN listing dimension values, THE Ledger_Service SHALL support filtering by dimension type and active status.

### Requirement 4: Master Data Management - Exchange Rates

**User Story:** As an accountant, I want to manage exchange rates, so that multi-currency transactions can be converted to the functional currency.

#### Acceptance Criteria

1. WHEN a user creates an exchange rate with from_currency, to_currency, rate, and effective_date, THE Ledger_Service SHALL create the exchange rate record.
2. WHEN creating an exchange rate, THE Ledger_Service SHALL validate that the rate is positive.
3. WHEN creating an exchange rate, THE Ledger_Service SHALL validate that from_currency and to_currency are different.
4. WHEN creating an exchange rate, THE Ledger_Service SHALL validate that both currencies exist in the currencies table.
5. WHEN an exchange rate already exists for the same currency pair and effective_date, THE Ledger_Service SHALL update the existing rate instead of creating a duplicate.
6. WHEN looking up an exchange rate for a date, THE Ledger_Service SHALL return the most recent rate on or before that date.
7. IF no direct exchange rate exists, THEN THE Ledger_Service SHALL attempt to calculate the rate via triangulation through USD.
8. IF no exchange rate can be found (direct or triangulated), THEN THE Ledger_Service SHALL return a clear error indicating the missing rate.

### Requirement 5: Transaction Creation - Single Currency

**User Story:** As a user, I want to create financial transactions with multiple entries, so that I can record business events using double-entry bookkeeping.

#### Acceptance Criteria

1. WHEN a user creates a transaction with entries, THE Ledger_Service SHALL validate that there are at least 2 entries.
2. WHEN a user creates a transaction, THE Ledger_Service SHALL validate that total debits equal total credits in functional currency.
3. WHEN a user creates a transaction, THE Ledger_Service SHALL validate that no entry has a zero amount.
4. WHEN a user creates a transaction, THE Ledger_Service SHALL validate that no entry has a negative amount.
5. WHEN a user creates a transaction, THE Ledger_Service SHALL validate that each entry specifies either debit or credit, not both.
6. WHEN a user creates a transaction, THE Ledger_Service SHALL validate that all referenced accounts exist and are active.
7. WHEN a user creates a transaction, THE Ledger_Service SHALL validate that all referenced accounts allow direct posting.
8. WHEN a transaction is created, THE Ledger_Service SHALL assign it a status of "draft".
9. WHEN a transaction is created, THE Ledger_Service SHALL find and validate the fiscal period for the transaction date.

### Requirement 6: Transaction Creation - Multi-Currency

**User Story:** As a user, I want to create transactions in foreign currencies, so that I can record international business events with proper currency conversion.

#### Acceptance Criteria

1. WHEN a user creates an entry with a source_currency different from the functional currency, THE Ledger_Service SHALL look up the exchange rate for the transaction date.
2. WHEN an exchange rate is found, THE Ledger_Service SHALL calculate the functional_amount by multiplying source_amount by exchange_rate.
3. WHEN calculating functional_amount, THE Ledger_Service SHALL use Banker's Rounding (half-even) and round to 4 decimal places.
4. WHEN the source_currency equals the functional currency, THE Ledger_Service SHALL set exchange_rate to 1 and functional_amount equal to source_amount.
5. WHEN storing a ledger entry, THE Ledger_Service SHALL store all three values: source_amount, exchange_rate, and functional_amount.
6. WHEN validating transaction balance, THE Ledger_Service SHALL compare total debits and credits in functional currency only.

### Requirement 7: Transaction Creation - Dimensional Tagging

**User Story:** As a user, I want to tag transaction entries with dimensions, so that I can analyze expenses by department, project, or cost center.

#### Acceptance Criteria

1. WHEN a user creates an entry with dimension values, THE Ledger_Service SHALL validate that all dimension value IDs exist and are active.
2. WHEN a user creates an entry with dimension values, THE Ledger_Service SHALL validate that dimension values belong to the same organization.
3. WHEN a dimension type is marked as required, THE Ledger_Service SHALL validate that entries include at least one value from that dimension type.
4. WHEN storing entry dimensions, THE Ledger_Service SHALL create entry_dimensions records linking the ledger entry to dimension values.

### Requirement 8: Account Balance Tracking

**User Story:** As a system, I want to track running balances for each account, so that I can provide point-in-time balance queries and detect concurrent modification issues.

#### Acceptance Criteria

1. WHEN a ledger entry is inserted, THE Ledger_Service SHALL increment the account_version for that account.
2. WHEN a ledger entry is inserted, THE Ledger_Service SHALL calculate and store the previous_balance (balance before this entry).
3. WHEN a ledger entry is inserted, THE Ledger_Service SHALL calculate and store the current_balance (balance after this entry).
4. WHEN calculating balance changes for asset or expense accounts, THE Ledger_Service SHALL add debits and subtract credits.
5. WHEN calculating balance changes for liability, equity, or revenue accounts, THE Ledger_Service SHALL add credits and subtract debits.
6. WHEN two transactions attempt to modify the same account concurrently, THE Ledger_Service SHALL handle the race condition using optimistic locking based on account_version.
7. FOR ALL sequences of transactions on an account, the final balance SHALL equal the sum of all debits minus credits (or vice versa based on account type).

### Requirement 9: Fiscal Period Validation

**User Story:** As an accountant, I want the system to enforce fiscal period rules, so that transactions cannot be posted to closed periods.

#### Acceptance Criteria

1. WHEN a user creates a transaction, THE Ledger_Service SHALL find the fiscal period that contains the transaction_date.
2. IF no fiscal period exists for the transaction_date, THEN THE Ledger_Service SHALL return an error indicating no fiscal period found.
3. WHEN posting a transaction to an OPEN period, THE Ledger_Service SHALL allow all authorized users to post.
4. WHEN posting a transaction to a SOFT_CLOSE period, THE Ledger_Service SHALL allow only users with accountant, admin, or owner roles to post.
5. WHEN posting a transaction to a CLOSED period, THE Ledger_Service SHALL reject the posting with a clear error message.

### Requirement 10: Transaction API Endpoints

**User Story:** As a frontend developer, I want REST API endpoints for transaction management, so that I can build the transaction UI.

#### Acceptance Criteria

1. WHEN a POST request is made to /transactions, THE API SHALL create a new transaction in draft status and return the transaction with resolved entries.
2. WHEN a GET request is made to /transactions, THE API SHALL return a paginated list of transactions with optional filters for status, date range, type, and dimension.
3. WHEN a GET request is made to /transactions/:id, THE API SHALL return the transaction with all entries, dimensions, and audit information.
4. WHEN a PATCH request is made to /transactions/:id for a draft transaction, THE API SHALL update the transaction.
5. WHEN a PATCH request is made to /transactions/:id for a non-draft transaction, THE API SHALL reject the update with an appropriate error.
6. WHEN a DELETE request is made to /transactions/:id for a draft transaction, THE API SHALL delete the transaction.
7. WHEN a DELETE request is made to /transactions/:id for a non-draft transaction, THE API SHALL reject the deletion with an appropriate error.

### Requirement 11: Master Data API Endpoints

**User Story:** As a frontend developer, I want REST API endpoints for master data management, so that I can build the configuration UI.

#### Acceptance Criteria

1. WHEN a POST request is made to /fiscal-years, THE API SHALL create a fiscal year with auto-generated periods.
2. WHEN a GET request is made to /fiscal-years, THE API SHALL return fiscal years with nested periods.
3. WHEN a PATCH request is made to /fiscal-periods/:id/status, THE API SHALL update the period status with proper validation.
4. WHEN a POST request is made to /accounts, THE API SHALL create a new account.
5. WHEN a GET request is made to /accounts, THE API SHALL return accounts with current balances.
6. WHEN a PATCH request is made to /accounts/:id, THE API SHALL update the account with proper validation.
7. WHEN a POST request is made to /dimension-types, THE API SHALL create a new dimension type.
8. WHEN a GET request is made to /dimension-types, THE API SHALL return dimension types.
9. WHEN a POST request is made to /dimension-values, THE API SHALL create a new dimension value.
10. WHEN a GET request is made to /dimension-values, THE API SHALL return dimension values with optional type filter.
11. WHEN a POST request is made to /exchange-rates, THE API SHALL create or update an exchange rate.
12. WHEN a GET request is made to /exchange-rates, THE API SHALL return the exchange rate for the specified currency pair and date.

### Requirement 12: Rounding and Allocation

**User Story:** As an accountant, I want amounts to be rounded and allocated correctly, so that no cents are lost in calculations.

#### Acceptance Criteria

1. WHEN converting currency amounts, THE Ledger_Service SHALL use Banker's Rounding (MidpointNearestEven) as the default rounding strategy.
2. WHEN rounding functional_amount, THE Ledger_Service SHALL round to 4 decimal places.
3. WHEN allocating an amount equally across N recipients, THE Ledger_Service SHALL use the Largest Remainder Method to ensure the sum exactly equals the original.
4. WHEN allocating by percentages, THE Ledger_Service SHALL use the Largest Remainder Method to distribute any rounding remainder.
5. FOR ALL allocation operations, the sum of allocated amounts SHALL exactly equal the original amount.

### Requirement 13: Database Trigger Verification

**User Story:** As a system administrator, I want database triggers to enforce data integrity, so that invalid data cannot be stored even if application logic fails.

#### Acceptance Criteria

1. WHEN a transaction is posted, THE check_transaction_balance trigger SHALL verify that total debits equal total credits.
2. IF the check_transaction_balance trigger detects an imbalance, THEN THE database SHALL reject the commit with an error.
3. WHEN a ledger entry is inserted, THE update_account_balance trigger SHALL update the account_version, previous_balance, and current_balance fields.
4. WHEN a user attempts to update a posted transaction (except to void it), THE prevent_posted_modification trigger SHALL reject the update.
5. WHEN a user attempts to update a voided transaction, THE prevent_posted_modification trigger SHALL reject the update.
6. WHEN posting to a fiscal period, THE validate_fiscal_period_posting trigger SHALL enforce period status rules.

### Requirement 14: Concurrent Transaction Handling

**User Story:** As a system, I want to handle concurrent transactions correctly, so that account balances never drift due to race conditions.

#### Acceptance Criteria

1. WHEN multiple transactions attempt to post to the same account simultaneously, THE Ledger_Service SHALL serialize the balance updates using database row locking.
2. WHEN a concurrent modification is detected via account_version mismatch, THE Ledger_Service SHALL retry the operation or return a clear error.
3. FOR ALL stress tests with 1000+ concurrent transactions, the final account balances SHALL be mathematically correct.
4. FOR ALL sequences of transactions, the account balance SHALL never drift from the expected value.
