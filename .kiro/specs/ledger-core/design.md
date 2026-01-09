# Design Document: Ledger Core

## Overview

The Ledger Core module is the heart of Zeltra's financial system, implementing enterprise-grade double-entry bookkeeping with multi-currency support and dimensional accounting. This design prioritizes data integrity above all else - a single incorrect balance can cascade into meaningless reports and dashboards.

### Key Design Principles

1. **Immutability**: Posted transactions cannot be modified; corrections are made via reversing entries
2. **Database-Level Enforcement**: Critical validations (balance check, period status) are enforced by PostgreSQL triggers
3. **Multi-Currency First**: Every entry stores source_amount, exchange_rate, and functional_amount
4. **Dimensional Flexibility**: Any entry can be tagged with multiple dimensions for analytical reporting
5. **Optimistic Locking**: Account versions prevent race conditions in concurrent environments
6. **Zero Float Policy**: All monetary calculations use `rust_decimal::Decimal` with explicit rounding

### Research Findings Incorporated

- **rust_decimal**: Using `RoundingStrategy::MidpointNearestEven` (Banker's Rounding) as default
- **SeaORM 1.1**: Transaction management via `db.begin()` / `txn.commit()` pattern
- **Double-Entry Rules**: Assets/Expenses increase with debits; Liabilities/Equity/Revenue increase with credits
- **Largest Remainder Method**: For fair allocation without losing cents

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           API Layer (Axum)                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────────┐  │
│  │ Transactions│ │  Accounts   │ │   Fiscal    │ │  Dimensions   │  │
│  │   Routes    │ │   Routes    │ │   Routes    │ │    Routes     │  │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └───────┬───────┘  │
└─────────┼───────────────┼───────────────┼─────────────────┼─────────┘
          │               │               │                 │
          ▼               ▼               ▼                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Core Layer (Pure Rust)                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                      LedgerService                           │    │
│  │  - validate_and_resolve()                                    │    │
│  │  - calculate_balance()                                       │    │
│  └─────────────────────────────────────────────────────────────┘    │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐    │
│  │ CurrencyService │ │ AllocationUtil  │ │  ValidationService  │    │
│  │ - get_rate()    │ │ - allocate_eq() │ │  - validate_entry() │    │
│  │ - convert()     │ │ - allocate_pct()│ │  - validate_dims()  │    │
│  └─────────────────┘ └─────────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
          │               │               │                 │
          ▼               ▼               ▼                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Database Layer (SeaORM)                       │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐    │
│  │TransactionRepo  │ │  AccountRepo    │ │  FiscalPeriodRepo   │    │
│  └─────────────────┘ └─────────────────┘ └─────────────────────┘    │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐    │
│  │ExchangeRateRepo │ │  DimensionRepo  │ │  LedgerEntryRepo    │    │
│  └─────────────────┘ └─────────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      PostgreSQL 16 + Triggers                        │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ check_transaction_balance    (DEFERRABLE INITIALLY DEFERRED) │    │
│  │ update_account_balance       (BEFORE INSERT)                 │    │
│  │ prevent_posted_modification  (BEFORE UPDATE)                 │    │
│  │ validate_fiscal_period       (BEFORE UPDATE on transactions) │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### Transaction Flow

```
User Request (POST /transactions)
         │
         ▼
┌────────────────────────┐
│   Parse & Validate     │ ─── Serde deserialization
│   Request Body         │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Find Fiscal Period    │ ─── Query fiscal_periods by date
│  for Transaction Date  │ ─── Validate period status
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Validate Accounts     │ ─── Check accounts exist
│                        │ ─── Check accounts are active
│                        │ ─── Check allow_direct_posting
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Validate Dimensions   │ ─── Check dimension values exist
│                        │ ─── Check required dimensions
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Resolve Exchange      │ ─── Lookup rates for each currency
│  Rates                 │ ─── Calculate functional_amount
│                        │ ─── Apply Banker's Rounding
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Validate Balance      │ ─── Sum debits in functional currency
│  (Debit = Credit)      │ ─── Sum credits in functional currency
│                        │ ─── Compare (must be equal)
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  BEGIN DB Transaction  │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Insert Transaction    │ ─── transactions table
│  Header                │ ─── status = 'draft'
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Insert Ledger         │ ─── ledger_entries table
│  Entries               │ ─── Trigger: update_account_balance
│                        │ ─── Sets account_version, balances
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  Insert Entry          │ ─── entry_dimensions table
│  Dimensions            │
└──────────┬─────────────┘
           │
           ▼
┌────────────────────────┐
│  COMMIT Transaction    │ ─── Trigger: check_transaction_balance
│                        │ ─── Fires at COMMIT (DEFERRABLE)
└──────────┬─────────────┘
           │
           ▼
    Response (201 Created)
```

## Components and Interfaces

### Core Domain Types

```rust
// crates/core/src/ledger/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::NaiveDate;

/// Entry type: either Debit or Credit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Debit,
    Credit,
}

/// Transaction type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Journal,
    Expense,
    Invoice,
    Bill,
    Payment,
    Transfer,
    Adjustment,
    OpeningBalance,
    Reversal,
}

/// Transaction status in workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Draft,
    Pending,
    Approved,
    Posted,
    Voided,
}

/// Fiscal period status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiscalPeriodStatus {
    Open,
    SoftClose,
    Closed,
}

/// Input for a single ledger entry
#[derive(Debug, Clone)]
pub struct LedgerEntryInput {
    pub account_id: Uuid,
    pub source_currency: String,
    pub source_amount: Decimal,
    pub entry_type: EntryType,
    pub memo: Option<String>,
    pub dimensions: Vec<Uuid>,
}

/// Input for creating a transaction
#[derive(Debug, Clone)]
pub struct CreateTransactionInput {
    pub organization_id: Uuid,
    pub transaction_type: TransactionType,
    pub transaction_date: NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub memo: Option<String>,
    pub entries: Vec<LedgerEntryInput>,
    pub created_by: Uuid,
}

/// Resolved entry with exchange rate applied
#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    pub account_id: Uuid,
    pub source_currency: String,
    pub source_amount: Decimal,
    pub exchange_rate: Decimal,
    pub functional_currency: String,
    pub functional_amount: Decimal,
    pub debit: Decimal,
    pub credit: Decimal,
    pub memo: Option<String>,
    pub dimensions: Vec<Uuid>,
}

/// Result of transaction creation
#[derive(Debug)]
pub struct TransactionResult {
    pub id: Uuid,
    pub reference_number: Option<String>,
    pub status: TransactionStatus,
    pub entries: Vec<ResolvedEntry>,
    pub totals: TransactionTotals,
}

/// Transaction totals for validation
#[derive(Debug)]
pub struct TransactionTotals {
    pub functional_debit: Decimal,
    pub functional_credit: Decimal,
    pub is_balanced: bool,
}
```

### Error Types

```rust
// crates/core/src/ledger/error.rs

use thiserror::Error;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Error)]
pub enum LedgerError {
    // Validation errors
    #[error("Transaction must have at least 2 entries")]
    InsufficientEntries,
    
    #[error("Transaction is not balanced. Debit: {debit}, Credit: {credit}")]
    UnbalancedTransaction { debit: Decimal, credit: Decimal },
    
    #[error("Entry amount cannot be zero")]
    ZeroAmount,
    
    #[error("Entry amount cannot be negative")]
    NegativeAmount,
    
    // Account errors
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),
    
    #[error("Account {0} is inactive")]
    AccountInactive(Uuid),
    
    #[error("Account {0} does not allow direct posting")]
    AccountNoDirectPosting(Uuid),
    
    // Fiscal period errors
    #[error("No fiscal period found for date {0}")]
    NoFiscalPeriod(NaiveDate),
    
    #[error("Fiscal period is closed, no posting allowed")]
    PeriodClosed,
    
    #[error("Fiscal period is soft-closed, only accountants can post")]
    PeriodSoftClosed,
    
    // Currency errors
    #[error("No exchange rate found for {from} to {to} on {date}")]
    NoExchangeRate { from: String, to: String, date: NaiveDate },
    
    // Dimension errors
    #[error("Invalid dimension value: {0}")]
    InvalidDimension(Uuid),
    
    #[error("Required dimension type missing: {0}")]
    RequiredDimensionMissing(String),
    
    // Transaction state errors
    #[error("Cannot modify posted transaction")]
    CannotModifyPosted,
    
    #[error("Cannot modify voided transaction")]
    CannotModifyVoided,
    
    #[error("Can only delete draft transactions")]
    CanOnlyDeleteDraft,
    
    // Concurrency errors
    #[error("Concurrent modification detected, please retry")]
    ConcurrentModification,
    
    // Database errors
    #[error("Database error: {0}")]
    Database(String),
}
```

### LedgerService Interface

```rust
// crates/core/src/ledger/service.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

pub struct LedgerService;

impl LedgerService {
    /// Validate and resolve a transaction before persisting
    /// Returns resolved entries with exchange rates applied
    pub fn validate_and_resolve(
        input: &CreateTransactionInput,
        org_base_currency: &str,
        exchange_rate_lookup: impl Fn(&str, &str, NaiveDate) -> Option<Decimal>,
        account_validator: impl Fn(Uuid) -> Result<AccountInfo, LedgerError>,
        dimension_validator: impl Fn(&[Uuid]) -> Result<(), LedgerError>,
    ) -> Result<Vec<ResolvedEntry>, LedgerError> {
        // 1. Validate minimum entries
        if input.entries.len() < 2 {
            return Err(LedgerError::InsufficientEntries);
        }

        // 2. Resolve each entry
        let mut resolved = Vec::with_capacity(input.entries.len());
        
        for entry in &input.entries {
            // Validate amount
            if entry.source_amount == Decimal::ZERO {
                return Err(LedgerError::ZeroAmount);
            }
            if entry.source_amount < Decimal::ZERO {
                return Err(LedgerError::NegativeAmount);
            }

            // Validate account
            let account_info = account_validator(entry.account_id)?;
            if !account_info.is_active {
                return Err(LedgerError::AccountInactive(entry.account_id));
            }
            if !account_info.allow_direct_posting {
                return Err(LedgerError::AccountNoDirectPosting(entry.account_id));
            }

            // Validate dimensions
            dimension_validator(&entry.dimensions)?;

            // Get exchange rate
            let exchange_rate = if entry.source_currency == org_base_currency {
                Decimal::ONE
            } else {
                exchange_rate_lookup(
                    &entry.source_currency,
                    org_base_currency,
                    input.transaction_date,
                ).ok_or_else(|| LedgerError::NoExchangeRate {
                    from: entry.source_currency.clone(),
                    to: org_base_currency.to_string(),
                    date: input.transaction_date,
                })?
            };

            // Calculate functional amount with Banker's Rounding
            let functional_amount = (entry.source_amount * exchange_rate)
                .round_dp_with_strategy(4, RoundingStrategy::MidpointNearestEven);

            // Determine debit/credit
            let (debit, credit) = match entry.entry_type {
                EntryType::Debit => (functional_amount, Decimal::ZERO),
                EntryType::Credit => (Decimal::ZERO, functional_amount),
            };

            resolved.push(ResolvedEntry {
                account_id: entry.account_id,
                source_currency: entry.source_currency.clone(),
                source_amount: entry.source_amount,
                exchange_rate,
                functional_currency: org_base_currency.to_string(),
                functional_amount,
                debit,
                credit,
                memo: entry.memo.clone(),
                dimensions: entry.dimensions.clone(),
            });
        }

        // 3. Validate balance
        let total_debit: Decimal = resolved.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = resolved.iter().map(|e| e.credit).sum();

        if total_debit != total_credit {
            return Err(LedgerError::UnbalancedTransaction {
                debit: total_debit,
                credit: total_credit,
            });
        }

        Ok(resolved)
    }
}
```

### CurrencyService Interface

```rust
// crates/core/src/currency/service.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use chrono::NaiveDate;

pub struct CurrencyService;

impl CurrencyService {
    /// Convert amount using exchange rate with Banker's Rounding
    pub fn convert(amount: Decimal, rate: Decimal) -> Decimal {
        (amount * rate).round_dp_with_strategy(4, RoundingStrategy::MidpointNearestEven)
    }

    /// Get exchange rate with triangulation fallback
    /// Priority: 1) Direct rate, 2) Inverse rate, 3) Triangulation via USD
    pub async fn get_rate(
        repo: &ExchangeRateRepository,
        org_id: Uuid,
        from: &str,
        to: &str,
        date: NaiveDate,
    ) -> Result<Decimal, CurrencyError> {
        // Same currency = 1.0
        if from == to {
            return Ok(Decimal::ONE);
        }

        // Try direct rate
        if let Some(rate) = repo.find_rate(org_id, from, to, date).await? {
            return Ok(rate);
        }

        // Try inverse rate
        if let Some(rate) = repo.find_rate(org_id, to, from, date).await? {
            return Ok(Decimal::ONE / rate);
        }

        // Try triangulation through USD
        if from != "USD" && to != "USD" {
            let from_usd = repo.find_rate(org_id, from, "USD", date).await?;
            let usd_to = repo.find_rate(org_id, "USD", to, date).await?;
            
            if let (Some(r1), Some(r2)) = (from_usd, usd_to) {
                return Ok(r1 * r2);
            }
        }

        Err(CurrencyError::NoRateFound { from: from.to_string(), to: to.to_string(), date })
    }
}
```

### AllocationUtil Interface

```rust
// crates/core/src/currency/allocation.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::*;

pub struct AllocationUtil;

impl AllocationUtil {
    /// Allocate amount equally using Largest Remainder Method
    /// Ensures sum of allocations EXACTLY equals total
    pub fn allocate_equal(total: Decimal, count: usize, decimal_places: u32) -> Vec<Decimal> {
        if count == 0 {
            return vec![];
        }
        if count == 1 {
            return vec![total];
        }

        let count_dec = Decimal::from(count as u64);
        let unit = Decimal::new(1, decimal_places);
        
        // Round down to get base allocation
        let base = (total / count_dec)
            .round_dp_with_strategy(decimal_places, RoundingStrategy::ToZero);
        
        // Calculate remainder
        let allocated = base * count_dec;
        let remainder = total - allocated;
        let extra_count = (remainder / unit).to_u64().unwrap_or(0) as usize;
        
        // Distribute: first N items get extra unit
        (0..count)
            .map(|i| if i < extra_count { base + unit } else { base })
            .collect()
    }

    /// Allocate by percentages using Largest Remainder Method
    pub fn allocate_by_percentages(
        total: Decimal,
        percentages: &[Decimal],
        decimal_places: u32,
    ) -> Vec<Decimal> {
        if percentages.is_empty() {
            return vec![];
        }

        let hundred = Decimal::from(100);
        let unit = Decimal::new(1, decimal_places);

        // Calculate exact allocations
        let exact: Vec<Decimal> = percentages
            .iter()
            .map(|p| total * *p / hundred)
            .collect();

        // Round down each
        let mut rounded: Vec<Decimal> = exact
            .iter()
            .map(|a| a.round_dp_with_strategy(decimal_places, RoundingStrategy::ToZero))
            .collect();

        // Calculate remainder to distribute
        let sum_rounded: Decimal = rounded.iter().copied().sum();
        let remainder = total - sum_rounded;
        let units_to_distribute = (remainder / unit).to_u64().unwrap_or(0) as usize;

        if units_to_distribute == 0 {
            return rounded;
        }

        // Sort by fractional remainder (largest first)
        let mut remainders: Vec<(usize, Decimal)> = exact
            .iter()
            .zip(rounded.iter())
            .enumerate()
            .map(|(i, (e, r))| (i, *e - *r))
            .collect();
        remainders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Give +1 unit to items with largest remainders
        for (idx, _) in remainders.iter().take(units_to_distribute) {
            rounded[*idx] += unit;
        }

        rounded
    }
}
```

## Data Models

### Entity Relationships

```
┌─────────────────┐       ┌─────────────────┐
│  organizations  │───────│  fiscal_years   │
└─────────────────┘       └────────┬────────┘
        │                          │
        │                          │
        ▼                          ▼
┌─────────────────┐       ┌─────────────────┐
│chart_of_accounts│       │ fiscal_periods  │
└────────┬────────┘       └────────┬────────┘
         │                         │
         │    ┌────────────────────┘
         │    │
         ▼    ▼
┌─────────────────┐       ┌─────────────────┐
│  transactions   │───────│ ledger_entries  │
└─────────────────┘       └────────┬────────┘
                                   │
                                   │
                                   ▼
                          ┌─────────────────┐
                          │entry_dimensions │
                          └────────┬────────┘
                                   │
                                   ▼
┌─────────────────┐       ┌─────────────────┐
│ dimension_types │───────│dimension_values │
└─────────────────┘       └─────────────────┘

┌─────────────────┐
│ exchange_rates  │
└─────────────────┘
```

### Key Database Tables

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `fiscal_years` | Accounting year container | id, organization_id, name, start_date, end_date, status |
| `fiscal_periods` | Monthly periods within year | id, fiscal_year_id, period_number, start_date, end_date, status |
| `chart_of_accounts` | Account master data | id, code, name, account_type, account_subtype, currency, is_active, allow_direct_posting |
| `transactions` | Transaction headers | id, organization_id, fiscal_period_id, transaction_type, transaction_date, status, created_by |
| `ledger_entries` | Individual debit/credit lines | id, transaction_id, account_id, source_currency, source_amount, exchange_rate, functional_amount, debit, credit, account_version, previous_balance, current_balance |
| `entry_dimensions` | Dimension tags per entry | id, ledger_entry_id, dimension_value_id |
| `dimension_types` | Dimension categories | id, organization_id, code, name, is_required |
| `dimension_values` | Dimension options | id, dimension_type_id, code, name, parent_id |
| `exchange_rates` | Currency conversion rates | id, organization_id, from_currency, to_currency, rate, effective_date |

### Balance Calculation Rules

| Account Type | Normal Balance | Debit Effect | Credit Effect |
|--------------|----------------|--------------|---------------|
| Asset | Debit | Increase (+) | Decrease (-) |
| Expense | Debit | Increase (+) | Decrease (-) |
| Liability | Credit | Decrease (-) | Increase (+) |
| Equity | Credit | Decrease (-) | Increase (+) |
| Revenue | Credit | Decrease (-) | Increase (+) |



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system - essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Transaction Balance Integrity

*For any* valid transaction with two or more entries, the sum of all debit amounts in functional currency SHALL equal the sum of all credit amounts in functional currency.

This is the fundamental double-entry bookkeeping invariant. If this property ever fails, the entire ledger becomes unreliable.

**Validates: Requirements 5.2, 6.6**

### Property 2: Account Type Balance Rules

*For any* ledger entry on an account:
- If the account is of type Asset or Expense, the balance change equals (debit - credit)
- If the account is of type Liability, Equity, or Revenue, the balance change equals (credit - debit)

This property ensures that the normal balance rules of accounting are correctly implemented.

**Validates: Requirements 8.4, 8.5**

### Property 3: Running Balance Consistency

*For any* account with N ledger entries, the `current_balance` of entry N SHALL equal the `previous_balance` of entry N plus the balance change (calculated per Property 2).

Additionally, the `previous_balance` of entry N SHALL equal the `current_balance` of entry N-1.

**Validates: Requirements 8.2, 8.3, 8.7**

### Property 4: Account Version Monotonicity

*For any* account, the `account_version` values across all ledger entries SHALL form a strictly increasing sequence starting from 1.

**Validates: Requirements 8.1**

### Property 5: Currency Conversion Correctness

*For any* ledger entry where source_currency differs from functional_currency:
- `functional_amount` SHALL equal `source_amount * exchange_rate` rounded to 4 decimal places using Banker's Rounding
- `exchange_rate` SHALL be positive

*For any* ledger entry where source_currency equals functional_currency:
- `exchange_rate` SHALL equal 1
- `functional_amount` SHALL equal `source_amount`

**Validates: Requirements 6.2, 6.3, 6.4**

### Property 6: Banker's Rounding Correctness

*For any* decimal value at a midpoint (e.g., 2.5, 3.5, 2.25, 2.35):
- Rounding SHALL produce the nearest even number
- 2.5 → 2, 3.5 → 4, 2.25 → 2.2, 2.35 → 2.4

**Validates: Requirements 12.1, 12.2**

### Property 7: Allocation Sum Invariant

*For any* allocation operation (equal or percentage-based):
- The sum of all allocated amounts SHALL exactly equal the original total
- No cents shall be lost or gained in the allocation

Example: `allocate_equal(100.00, 3, 2)` → [33.34, 33.33, 33.33] where sum = 100.00

**Validates: Requirements 12.3, 12.4, 12.5**

### Property 8: Exchange Rate Lookup Priority

*For any* exchange rate lookup from currency A to currency B on date D:
1. If a direct rate (A→B) exists on or before D, return the most recent one
2. Else if an inverse rate (B→A) exists, return 1/rate
3. Else if both A→USD and USD→B exist, return (A→USD) * (USD→B)
4. Else return error

**Validates: Requirements 4.6, 4.7**

### Property 9: Fiscal Period Posting Rules

*For any* transaction posting attempt:
- If period status is OPEN, all authorized users can post
- If period status is SOFT_CLOSE, only accountant/admin/owner roles can post
- If period status is CLOSED, no one can post

**Validates: Requirements 1.5, 1.6, 9.3, 9.4, 9.5**

### Property 10: Fiscal Year Date Validation

*For any* fiscal year:
- `start_date` SHALL be strictly before `end_date`
- The date range SHALL NOT overlap with any other fiscal year in the same organization

**Validates: Requirements 1.2, 1.3**

### Property 11: Uniqueness Constraints

*For any* organization:
- Account codes SHALL be unique
- Dimension type codes SHALL be unique
- Dimension value codes SHALL be unique within their dimension type

**Validates: Requirements 2.2, 3.2, 3.4**

### Property 12: Inactive Entity Rejection

*For any* transaction creation:
- Entries referencing inactive accounts SHALL be rejected
- Entries referencing inactive dimension values SHALL be rejected
- Entries referencing accounts with `allow_direct_posting = false` SHALL be rejected

**Validates: Requirements 2.7, 5.6, 5.7, 7.1**

### Property 13: Entry Validation Rules

*For any* ledger entry:
- `source_amount` SHALL be positive (> 0)
- `source_amount` SHALL NOT be zero
- The transaction SHALL have at least 2 entries

**Validates: Requirements 5.1, 5.3, 5.4**

### Property 14: Concurrent Balance Integrity (Stress Test)

*For any* sequence of N concurrent transactions on the same account:
- The final `current_balance` SHALL equal the mathematically expected value
- No balance drift shall occur regardless of execution order
- The system SHALL handle at least 1000 concurrent transactions without balance errors

**Validates: Requirements 14.1, 14.2, 14.3, 14.4**

### Property 15: Transaction Immutability

*For any* posted transaction:
- Updates (except to void) SHALL be rejected
- Deletions SHALL be rejected

*For any* voided transaction:
- All modifications SHALL be rejected

**Validates: Requirements 13.4, 13.5**

### Property 16: Multi-Currency Entry Completeness

*For any* ledger entry, all three currency fields SHALL be populated:
- `source_amount` (original amount)
- `exchange_rate` (conversion rate)
- `functional_amount` (converted amount)

**Validates: Requirements 6.5**

## Error Handling

### Error Response Format

All API errors follow a consistent JSON structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {},
    "request_id": "uuid"
  }
}
```

### Error Codes by Category

| Category | Code | HTTP Status | Description |
|----------|------|-------------|-------------|
| Validation | `INSUFFICIENT_ENTRIES` | 400 | Transaction has fewer than 2 entries |
| Validation | `UNBALANCED_TRANSACTION` | 400 | Debits ≠ Credits in functional currency |
| Validation | `ZERO_AMOUNT` | 400 | Entry amount is zero |
| Validation | `NEGATIVE_AMOUNT` | 400 | Entry amount is negative |
| Account | `ACCOUNT_NOT_FOUND` | 404 | Referenced account doesn't exist |
| Account | `ACCOUNT_INACTIVE` | 400 | Account is deactivated |
| Account | `ACCOUNT_NO_DIRECT_POSTING` | 400 | Account doesn't allow direct posting |
| Fiscal | `NO_FISCAL_PERIOD` | 400 | No period found for transaction date |
| Fiscal | `PERIOD_CLOSED` | 400 | Period is closed, no posting allowed |
| Fiscal | `PERIOD_SOFT_CLOSED` | 403 | Only accountants can post to soft-closed periods |
| Currency | `NO_EXCHANGE_RATE` | 400 | No rate found for currency pair/date |
| Dimension | `INVALID_DIMENSION` | 400 | Dimension value doesn't exist or is inactive |
| Dimension | `REQUIRED_DIMENSION_MISSING` | 400 | Required dimension type not provided |
| State | `CANNOT_MODIFY_POSTED` | 400 | Posted transactions are immutable |
| State | `CANNOT_MODIFY_VOIDED` | 400 | Voided transactions are immutable |
| State | `CAN_ONLY_DELETE_DRAFT` | 400 | Only draft transactions can be deleted |
| Concurrency | `CONCURRENT_MODIFICATION` | 409 | Optimistic lock failure, retry needed |

### Retry Strategy

For `CONCURRENT_MODIFICATION` errors:
1. Client should retry with exponential backoff
2. Maximum 3 retries recommended
3. If still failing, surface error to user

## Testing Strategy

### Dual Testing Approach

This module requires both unit tests and property-based tests:

1. **Unit Tests**: Verify specific examples, edge cases, and error conditions
2. **Property Tests**: Verify universal properties across randomly generated inputs

### Property-Based Testing Configuration

- **Library**: `proptest` crate for Rust
- **Minimum Iterations**: 100 per property test
- **Tag Format**: `// Feature: ledger-core, Property N: [property_text]`

### Test Categories

| Category | Type | Count Target |
|----------|------|--------------|
| Transaction Validation | Property | 20+ |
| Currency Conversion | Property | 15+ |
| Allocation | Property | 10+ |
| Balance Tracking | Property | 20+ |
| Fiscal Period Rules | Property | 15+ |
| Concurrent Access | Property | 10+ |
| API Integration | Unit | 30+ |
| Error Handling | Unit | 20+ |
| Database Triggers | Integration | 10+ |
| **Total** | | **150+** |

### Critical Test Scenarios

1. **Balance Drift Test**: Create 1000+ transactions on same account, verify final balance
2. **Concurrent Posting Test**: 100 concurrent transactions, verify no race conditions
3. **Multi-Currency Rounding Test**: Verify no cents lost in currency conversion
4. **Allocation Exhaustive Test**: Verify sum invariant for various amounts and counts
5. **Fiscal Period Boundary Test**: Transactions at period boundaries

### Test Data Generators

```rust
// Example proptest generators for ledger-core

use proptest::prelude::*;
use rust_decimal::Decimal;

// Generate valid positive amounts (0.0001 to 999,999,999.9999)
fn arb_amount() -> impl Strategy<Value = Decimal> {
    (1i64..=9_999_999_999_999i64)
        .prop_map(|n| Decimal::new(n, 4))
}

// Generate valid currency codes
fn arb_currency() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("USD".to_string()),
        Just("EUR".to_string()),
        Just("GBP".to_string()),
        Just("JPY".to_string()),
        Just("IDR".to_string()),
        Just("SGD".to_string()),
    ]
}

// Generate valid exchange rates (0.0001 to 100,000)
fn arb_exchange_rate() -> impl Strategy<Value = Decimal> {
    (1i64..=1_000_000_000i64)
        .prop_map(|n| Decimal::new(n, 4))
}

// Generate balanced transaction entries
fn arb_balanced_entries(count: usize) -> impl Strategy<Value = Vec<LedgerEntryInput>> {
    // Generate entries that sum to zero (balanced)
    // Implementation ensures total debits = total credits
}
```

### Integration Test Setup

```rust
// tests/ledger_integration.rs

use sqlx::PgPool;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;

async fn setup_test_db() -> PgPool {
    // Spin up PostgreSQL container
    // Run migrations
    // Seed test data
    // Return connection pool
}

#[tokio::test]
async fn test_balance_trigger_fires_on_commit() {
    let pool = setup_test_db().await;
    // Create unbalanced transaction
    // Verify commit fails with balance error
}

#[tokio::test]
async fn test_concurrent_balance_updates() {
    let pool = setup_test_db().await;
    // Spawn 100 concurrent tasks
    // Each creates a transaction on same account
    // Verify final balance is correct
}
```
