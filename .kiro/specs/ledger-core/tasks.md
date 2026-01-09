# Implementation Plan: Ledger Core

## Overview

This implementation plan breaks down the Ledger Core module into discrete, incremental tasks. Each task builds on previous work and includes property-based tests to verify correctness. The plan follows a bottom-up approach: core domain types → services → repositories → API routes.

**Language:** Rust (edition 2024)
**Testing Framework:** proptest for property-based tests, tokio::test for async tests

## Tasks

- [x] 1. Core Domain Types and Validation
  - [x] 1.1 Create ledger domain types in `crates/core/src/ledger/types.rs`
    - Define `EntryType`, `TransactionType`, `TransactionStatus`, `FiscalPeriodStatus` enums
    - Define `LedgerEntryInput`, `CreateTransactionInput`, `ResolvedEntry`, `TransactionResult`, `TransactionTotals` structs
    - Use `rust_decimal::Decimal` for all monetary values
    - _Requirements: 5.1-5.9, 6.1-6.6_

  - [x] 1.2 Create ledger error types in `crates/core/src/ledger/error.rs`
    - Define `LedgerError` enum with all validation and state errors
    - Implement `thiserror::Error` derive
    - _Requirements: 5.1-5.9, 8.6, 9.1-9.5_

  - [x] 1.3 Write property test for entry validation rules
    - **Property 13: Entry Validation Rules**
    - Test zero amount rejection, negative amount rejection, minimum 2 entries
    - **Validates: Requirements 5.1, 5.3, 5.4**

- [x] 2. Currency Service and Allocation
  - [x] 2.1 Create currency service in `crates/core/src/currency/service.rs`
    - Implement `convert()` with Banker's Rounding to 4 decimal places
    - Use `RoundingStrategy::MidpointNearestEven`
    - _Requirements: 6.2, 6.3, 12.1, 12.2_

  - [x] 2.2 Write property test for Banker's Rounding
    - **Property 6: Banker's Rounding Correctness**
    - Test midpoint values: 2.5→2, 3.5→4, 2.25→2.2, 2.35→2.4
    - **Validates: Requirements 12.1, 12.2**

  - [x] 2.3 Create allocation utility in `crates/core/src/currency/allocation.rs`
    - Implement `allocate_equal()` using Largest Remainder Method
    - Implement `allocate_by_percentages()` using Largest Remainder Method
    - Ensure sum invariant: allocated amounts exactly equal original
    - _Requirements: 12.3, 12.4, 12.5_

  - [x] 2.4 Write property test for allocation sum invariant
    - **Property 7: Allocation Sum Invariant**
    - Test: for any amount and count, sum of allocations == original
    - Test: 100/3 = [33.34, 33.33, 33.33], sum = 100.00
    - **Validates: Requirements 12.3, 12.4, 12.5**

- [x] 3. Checkpoint - Core utilities complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Ledger Service Implementation
  - [x] 4.1 Create LedgerService in `crates/core/src/ledger/service.rs`
    - Implement `validate_and_resolve()` function
    - Validate minimum entries, amounts, accounts, dimensions
    - Resolve exchange rates and calculate functional amounts
    - Validate transaction balance (debits = credits)
    - _Requirements: 5.1-5.9, 6.1-6.6, 7.1-7.4_

  - [x] 4.2 Write property test for transaction balance integrity
    - **Property 1: Transaction Balance Integrity**
    - Test: for any valid transaction, sum(debits) == sum(credits) in functional currency
    - **Validates: Requirements 5.2, 6.6**

  - [x] 4.3 Write property test for currency conversion correctness
    - **Property 5: Currency Conversion Correctness**
    - Test: functional_amount == source_amount * exchange_rate (rounded)
    - Test: same currency → rate=1, functional=source
    - **Validates: Requirements 6.2, 6.3, 6.4**

  - [x] 4.4 Write property test for multi-currency entry completeness
    - **Property 16: Multi-Currency Entry Completeness**
    - Test: all three fields (source_amount, exchange_rate, functional_amount) are populated
    - **Validates: Requirements 6.5**

- [x] 5. Checkpoint - Ledger service complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Database Repositories - Master Data
  - [x] 6.1 Create FiscalYearRepository in `crates/db/src/repositories/fiscal.rs`
    - Implement `create_fiscal_year()` with auto-generated periods
    - Implement `list_fiscal_years()` with nested periods
    - Implement `update_period_status()` with validation
    - _Requirements: 1.1-1.7_

  - [x] 6.2 Write property test for fiscal year date validation
    - **Property 10: Fiscal Year Date Validation**
    - Test: start_date < end_date, no overlapping date ranges
    - **Validates: Requirements 1.2, 1.3**

  - [x] 6.3 Create AccountRepository in `crates/db/src/repositories/account.rs`
    - Implement `create_account()` with validation (unique code, valid currency, valid parent)
    - Implement `list_accounts()` with computed balances
    - Implement `find_account_by_id()` for detail view
    - Implement `update_account()` with type change restriction (reject if has entries)
    - Implement `delete_account()` (soft delete via is_active = false, reject if has entries)
    - _Requirements: 2.1-2.7_

  - [x] 6.4 Write property test for uniqueness constraints
    - **Property 11: Uniqueness Constraints**
    - Test: duplicate account codes rejected, duplicate dimension codes rejected
    - **Validates: Requirements 2.2, 3.2, 3.4**

  - [x] 6.5 Create DimensionRepository in `crates/db/src/repositories/dimension.rs`
    - Implement `create_dimension_type()` and `create_dimension_value()`
    - Implement `list_dimension_types()` and `list_dimension_values()`
    - Support filtering by type and active status
    - _Requirements: 3.1-3.6_

  - [x] 6.6 Create ExchangeRateRepository in `crates/db/src/repositories/exchange_rate.rs`
    - Implement `create_or_update_rate()` (upsert behavior)
    - Implement `find_rate()` with date lookup (most recent on or before)
    - Implement triangulation fallback through USD
    - _Requirements: 4.1-4.8_

  - [x] 6.7 Write property test for exchange rate lookup priority
    - **Property 8: Exchange Rate Lookup Priority**
    - Test: direct rate → inverse rate → triangulation → error
    - **Validates: Requirements 4.6, 4.7**

- [x] 7. Checkpoint - Master data repositories complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Database Repositories - Transactions
  - [x] 8.1 Create TransactionRepository in `crates/db/src/repositories/transaction.rs`
    - Implement `create_transaction()` with entries and dimensions
    - Use SeaORM transaction: `db.begin()` / `txn.commit()`
    - Set initial status to "draft"
    - _Requirements: 5.8, 5.9, 7.4_

  - [x] 8.2 Implement balance tracking in ledger entry insertion
    - Increment `account_version` per account
    - Calculate and store `previous_balance` and `current_balance`
    - Apply account type balance rules (debit/credit effects)
    - _Requirements: 8.1-8.5_

  - [x] 8.3 Write property test for account type balance rules
    - **Property 2: Account Type Balance Rules**
    - Test: Asset/Expense → balance += debit - credit
    - Test: Liability/Equity/Revenue → balance += credit - debit
    - **Validates: Requirements 8.4, 8.5**

  - [x] 8.4 Write property test for running balance consistency
    - **Property 3: Running Balance Consistency**
    - Test: current_balance[N] == previous_balance[N] + change
    - Test: previous_balance[N] == current_balance[N-1]
    - **Validates: Requirements 8.2, 8.3, 8.7**

  - [x] 8.5 Write property test for account version monotonicity
    - **Property 4: Account Version Monotonicity**
    - Test: account_version forms strictly increasing sequence
    - **Validates: Requirements 8.1**

  - [x] 8.6 Implement transaction query methods
    - Implement `list_transactions()` with filters (status, date, type, dimension)
    - Implement `get_transaction()` with entries and dimensions
    - Implement `update_transaction()` for draft only
    - Implement `delete_transaction()` for draft only
    - _Requirements: 10.2-10.7_

  - [x] 8.7 Write property test for transaction immutability
    - **Property 15: Transaction Immutability**
    - Test: posted transactions reject updates (except void)
    - Test: voided transactions reject all modifications
    - **Validates: Requirements 13.4, 13.5**

- [x] 9. Checkpoint - Transaction repository complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Fiscal Period Validation
  - [x] 10.1 Implement fiscal period lookup and validation
    - Find period containing transaction date
    - Validate period status against user role
    - Return appropriate errors for closed/soft-closed periods
    - _Requirements: 9.1-9.5_

  - [x] 10.2 Write property test for fiscal period posting rules
    - **Property 9: Fiscal Period Posting Rules**
    - Test: OPEN → all users can post
    - Test: SOFT_CLOSE → only accountant/admin/owner
    - Test: CLOSED → no one can post
    - **Validates: Requirements 1.5, 1.6, 9.3, 9.4, 9.5**

  - [x] 10.3 Write property test for inactive entity rejection
    - **Property 12: Inactive Entity Rejection**
    - Test: inactive accounts rejected
    - Test: inactive dimensions rejected
    - Test: no-direct-posting accounts rejected
    - **Validates: Requirements 2.7, 5.6, 5.7, 7.1**

- [x] 11. Checkpoint - Fiscal period validation complete
  - Ensure all tests pass, ask the user if questions arise.

- [x] 12. API Routes - Master Data
  - [x] 12.1 Create fiscal routes in `crates/api/src/routes/fiscal.rs`
    - GET /fiscal-years - list with nested periods
    - POST /fiscal-years - create with auto-generated periods
    - PATCH /fiscal-periods/:id/status - update status
    - _Requirements: 11.1-11.3_

  - [x] 12.2 Create account routes in `crates/api/src/routes/accounts.rs`
    - GET /accounts - list with balances (query: type, active, currency)
    - POST /accounts - create account
    - GET /accounts/:id - get account detail
    - PUT /accounts/:id - update account
    - DELETE /accounts/:id - delete account (soft delete)
    - GET /accounts/:id/balance - get balance at date (query: as_of)
    - GET /accounts/:id/ledger - get ledger entries (query: from, to, page, limit)
    - _Requirements: 11.4-11.6_

  - [x] 12.3 Create dimension routes in `crates/api/src/routes/dimensions.rs`
    - GET /dimension-types - list types
    - POST /dimension-types - create type
    - GET /dimension-values - list with filters (query: type, active)
    - POST /dimension-values - create value
    - _Requirements: 11.7-11.10_

  - [x] 12.4 Create exchange rate routes in `crates/api/src/routes/exchange_rates.rs`
    - GET /exchange-rates - get rate for currency pair and date (query: from, to, date)
    - POST /exchange-rates - create/update rate
    - _Requirements: 11.11-11.12_

  - [x] 12.5 Create currency routes in `crates/api/src/routes/currencies.rs`
    - GET /currencies - list all currencies
    - _Requirements: 4.3, 4.4_

- [x] 13. API Routes - Transactions
  - [x] 13.1 Create transaction routes in `crates/api/src/routes/transactions.rs`
    - GET /transactions - list with filters (query: status, from, to, type, dimension, page, limit)
    - POST /transactions - create draft transaction
    - GET /transactions/:id - get with entries and dimensions
    - PATCH /transactions/:id - update draft only
    - DELETE /transactions/:id - delete draft only
    - _Requirements: 10.1-10.7_

  - [x] 13.2 Write integration tests for transaction API
    - Test create transaction with valid entries
    - Test create transaction with unbalanced entries (expect error)
    - Test create transaction with inactive account (expect error)
    - Test update posted transaction (expect error)
    - _Requirements: 10.1-10.7_

- [x] 14. Checkpoint - API routes complete
  - All API routes implemented and tested

- [x] 15. Database Trigger Verification
  - [x] 15.1 Write integration tests for database triggers
    - Test `check_transaction_balance` trigger rejects unbalanced commits
    - Test `update_account_balance` trigger sets version and balances
    - Test `prevent_posted_modification` trigger rejects posted updates
    - Test `validate_fiscal_period_posting` trigger enforces period rules
    - _Requirements: 13.1-13.6_

- [x] 16. Concurrent Access Testing
  - [x] 16.1 Implement concurrent transaction stress test
    - Create 100+ concurrent transactions on same account
    - Verify final balance is mathematically correct
    - Verify no balance drift occurs
    - _Requirements: 14.1-14.4_

  - [x] 16.2 Write property test for concurrent balance integrity
    - **Property 14: Concurrent Balance Integrity (Stress Test)**
    - Test: 1000+ concurrent transactions, final balance correct
    - Test: no balance drift regardless of execution order
    - **Validates: Requirements 14.1, 14.2, 14.3, 14.4**

  - [x] 16.3 Fix trigger bug: MAX(balance) → ORDER BY version DESC LIMIT 1
    - Fixed `update_account_balance` trigger to get balance from entry with highest version
    - Previous bug: MAX(account_current_balance) returned highest balance value, not latest
    - All concurrent tests now pass (100, 1000 transactions)

- [x] 17. Final Checkpoint - All tests pass
  - All 229 tests pass:
    - zeltra-core: 132 unit tests + 5 doctests
    - zeltra-db lib: 71 unit tests
    - concurrent_test: 5 integration tests (1000+ concurrent transactions)
    - trigger_test: 8 integration tests
    - transaction_test: 8 integration tests
  - Stress test with 1000+ concurrent transactions passes
  - Fixed trigger bug: MAX(balance) → ORDER BY version DESC LIMIT 1

## Notes

- All tasks including property-based tests are required
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- Integration tests verify database triggers and API behavior

## Scope Notes

This spec covers **Phase 2: Ledger Core** only. The following are handled in separate specs:
- **Phase 3 (Transaction Workflow)**: Submit/approve/reject/post/void workflow, approval rules
- **Phase 4 (Reports)**: Trial balance, balance sheet, income statement, dimensional reports, budgets
- **Phase 5 (API Polish)**: Dashboard metrics, recent activity, bulk import, attachments
