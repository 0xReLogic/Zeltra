# Core Features

Enterprise-grade financial features with multi-currency support, dimensional accounting, and strict fiscal period management.

## 1. Ledger Service

### Overview

The ledger service handles all financial transactions using double-entry bookkeeping with:
- Multi-currency support (source, exchange rate, functional amounts)
- Dimensional accounting (cost center, department, project tagging)
- Fiscal period validation
- Historical balance tracking per entry

### Transaction Flow

```
User Request
     │
     ▼
┌──────────────────────┐
│   Validate Input     │ ─── Zod/Serde validation
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Validate Fiscal      │ ─── Check period status (OPEN/SOFT_CLOSE)
│ Period               │ ─── Check user role permissions
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Validate Dimensions  │ ─── Check dimension values exist in master data
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Currency Conversion  │ ─── Lookup exchange rate for transaction date
│                      │ ─── Calculate functional_amount
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Check Budget         │ ─── Optional: warn if over budget
│ (by dimension)       │ ─── Check per account + dimension combination
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Begin DB Transaction │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Insert Transaction   │ ─── transactions table
│ Header               │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Insert Ledger        │ ─── ledger_entries table
│ Entries              │ ─── Trigger updates account_version & balances
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Insert Entry         │ ─── entry_dimensions table
│ Dimensions           │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Commit Transaction   │ ─── Balance check trigger fires at COMMIT
└──────────┬───────────┘
           │
           ▼
    Response (with balances)
```

### Domain Types (Rust)

```rust
// core/src/ledger/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{NaiveDate, DateTime, Utc};

/// Represents a single line item in a transaction
#[derive(Debug, Clone)]
pub struct LedgerEntryInput {
    pub account_id: Uuid,
    
    // Multi-currency: source amount in original currency
    pub source_currency: String,  // ISO 4217 (e.g., "USD", "EUR", "IDR")
    pub source_amount: Decimal,
    
    // Debit or Credit (mutually exclusive)
    pub entry_type: EntryType,
    
    // Optional memo for this line
    pub memo: Option<String>,
    
    // Dimensional tags
    pub dimensions: Vec<Uuid>,  // dimension_value_ids
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Debit,
    Credit,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Draft,
    Pending,
    Approved,
    Posted,
    Voided,
}

/// Input for creating a new transaction
#[derive(Debug)]
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
#[derive(Debug)]
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
```

### Ledger Error Types

```rust
// core/src/ledger/error.rs

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LedgerError {
    // Validation errors
    #[error("Transaction must have at least 2 entries")]
    InsufficientEntries,
    
    #[error("Transaction is not balanced. Debit: {debit}, Credit: {credit}")]
    UnbalancedTransaction {
        debit: rust_decimal::Decimal,
        credit: rust_decimal::Decimal,
    },
    
    #[error("Amount cannot be negative")]
    NegativeAmount,
    
    #[error("Entry cannot have both debit and credit")]
    BothDebitAndCredit,
    
    #[error("Entry amount cannot be zero")]
    ZeroEntry,
    
    // Fiscal period errors
    #[error("Fiscal period is closed, no posting allowed")]
    PeriodClosed,
    
    #[error("Fiscal period is soft-closed, only accountants can post")]
    PeriodSoftClosed,
    
    #[error("No fiscal period found for date {0}")]
    NoFiscalPeriod(chrono::NaiveDate),
    
    // Currency errors
    #[error("No exchange rate found for {from} to {to} on {date}")]
    NoExchangeRate {
        from: String,
        to: String,
        date: chrono::NaiveDate,
    },
    
    #[error("Currency mismatch: account expects {expected}, got {got}")]
    CurrencyMismatch {
        expected: String,
        got: String,
    },
    
    // Dimension errors
    #[error("Invalid dimension value: {0}")]
    InvalidDimension(Uuid),
    
    #[error("Required dimension type missing: {0}")]
    RequiredDimensionMissing(String),
    
    // Account errors
    #[error("Account {0} does not allow direct posting")]
    AccountNoDirectPosting(Uuid),
    
    #[error("Account {0} is inactive")]
    AccountInactive(Uuid),
    
    // Transaction state errors
    #[error("Cannot modify posted transaction")]
    CannotModifyPosted,
    
    #[error("Cannot modify voided transaction")]
    CannotModifyVoided,
    
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Concurrent modification detected, please retry")]
    ConcurrentModification,
}
```

### Ledger Service Implementation

```rust
// core/src/ledger/service.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::ledger::types::*;
use crate::ledger::error::LedgerError;

pub struct LedgerService;

impl LedgerService {
    /// Validate and resolve a transaction before persisting
    pub fn validate_and_resolve(
        input: &CreateTransactionInput,
        org_base_currency: &str,
        exchange_rate_lookup: impl Fn(&str, &str, NaiveDate) -> Option<Decimal>,
    ) -> Result<Vec<ResolvedEntry>, LedgerError> {
        // 1. Basic validation
        if input.entries.len() < 2 {
            return Err(LedgerError::InsufficientEntries);
        }

        // 2. Resolve each entry with exchange rates
        let mut resolved: Vec<ResolvedEntry> = Vec::with_capacity(input.entries.len());
        
        for entry in &input.entries {
            // Validate amount
            if entry.source_amount <= Decimal::ZERO {
                return Err(LedgerError::NegativeAmount);
            }

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

            // Calculate functional amount
            let functional_amount = entry.source_amount * exchange_rate;
            
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

        // 3. Validate balance (in functional currency)
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

### Repository Implementation (SeaORM)

```rust
// db/src/repositories/ledger_repo.rs

use sea_orm::{
    DatabaseConnection, TransactionTrait, ActiveModelTrait, 
    EntityTrait, QueryFilter, ColumnTrait, Set,
};
use uuid::Uuid;

use crate::entities::{transactions, ledger_entries, entry_dimensions, fiscal_periods};
use zeltra_core::ledger::{
    CreateTransactionInput, ResolvedEntry, LedgerError,
    TransactionStatus, FiscalPeriodStatus,
};

pub struct LedgerRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> LedgerRepository<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Find fiscal period for a given date
    pub async fn find_fiscal_period(
        &self,
        org_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<fiscal_periods::Model, LedgerError> {
        fiscal_periods::Entity::find()
            .filter(fiscal_periods::Column::OrganizationId.eq(org_id))
            .filter(fiscal_periods::Column::StartDate.lte(date))
            .filter(fiscal_periods::Column::EndDate.gte(date))
            .one(self.db)
            .await?
            .ok_or(LedgerError::NoFiscalPeriod(date))
    }

    /// Validate fiscal period allows posting
    pub fn validate_period_status(
        period: &fiscal_periods::Model,
        user_role: &str,
    ) -> Result<(), LedgerError> {
        match period.status.as_str() {
            "CLOSED" => Err(LedgerError::PeriodClosed),
            "SOFT_CLOSE" => {
                if !["owner", "admin", "accountant"].contains(&user_role) {
                    Err(LedgerError::PeriodSoftClosed)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Create a complete transaction with entries and dimensions
    pub async fn create_transaction(
        &self,
        input: CreateTransactionInput,
        resolved_entries: Vec<ResolvedEntry>,
        fiscal_period_id: Uuid,
    ) -> Result<transactions::Model, LedgerError> {
        let db_txn = self.db.begin().await?;

        // 1. Insert transaction header
        let transaction = transactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(input.organization_id),
            fiscal_period_id: Set(fiscal_period_id),
            transaction_type: Set(input.transaction_type.to_string()),
            transaction_date: Set(input.transaction_date),
            description: Set(input.description),
            reference_number: Set(input.reference_number),
            memo: Set(input.memo),
            status: Set(TransactionStatus::Draft.to_string()),
            created_by: Set(input.created_by),
            ..Default::default()
        };
        let transaction = transaction.insert(&db_txn).await?;

        // 2. Insert ledger entries
        for entry in resolved_entries {
            let ledger_entry = ledger_entries::ActiveModel {
                id: Set(Uuid::new_v4()),
                transaction_id: Set(transaction.id),
                account_id: Set(entry.account_id),
                source_currency: Set(entry.source_currency),
                source_amount: Set(entry.source_amount),
                exchange_rate: Set(entry.exchange_rate),
                functional_currency: Set(entry.functional_currency),
                functional_amount: Set(entry.functional_amount),
                debit: Set(entry.debit),
                credit: Set(entry.credit),
                memo: Set(entry.memo),
                // account_version, previous_balance, current_balance 
                // are set by database trigger
                ..Default::default()
            };
            let ledger_entry = ledger_entry.insert(&db_txn).await?;

            // 3. Insert entry dimensions
            for dim_value_id in entry.dimensions {
                let entry_dim = entry_dimensions::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    ledger_entry_id: Set(ledger_entry.id),
                    dimension_value_id: Set(dim_value_id),
                    ..Default::default()
                };
                entry_dim.insert(&db_txn).await?;
            }
        }

        // 4. Commit - balance check trigger fires here
        db_txn.commit().await?;

        Ok(transaction)
    }

    /// Post a transaction (change status from approved to posted)
    pub async fn post_transaction(
        &self,
        transaction_id: Uuid,
        posted_by: Uuid,
    ) -> Result<transactions::Model, LedgerError> {
        let txn = transactions::Entity::find_by_id(transaction_id)
            .one(self.db)
            .await?
            .ok_or(LedgerError::Database(sea_orm::DbErr::RecordNotFound(
                "Transaction not found".to_string(),
            )))?;

        if txn.status == "posted" {
            return Err(LedgerError::CannotModifyPosted);
        }
        if txn.status == "voided" {
            return Err(LedgerError::CannotModifyVoided);
        }

        let mut active: transactions::ActiveModel = txn.into();
        active.status = Set("posted".to_string());
        active.posted_by = Set(Some(posted_by));
        active.posted_at = Set(Some(chrono::Utc::now()));

        let updated = active.update(self.db).await?;
        Ok(updated)
    }

    /// Void a posted transaction by creating a reversing entry
    pub async fn void_transaction(
        &self,
        transaction_id: Uuid,
        voided_by: Uuid,
        reason: String,
    ) -> Result<transactions::Model, LedgerError> {
        // Implementation creates a reversing transaction
        // with all debits/credits swapped
        todo!("Implement void with reversing entry")
    }
}
```


---

## 2. Multi-Currency Engine

### Overview

Every transaction stores three values for complete audit trail and currency revaluation support:
1. `source_amount` - Original amount in transaction currency
2. `exchange_rate` - Rate at transaction date
3. `functional_amount` - Converted to organization base currency

### Exchange Rate Service

```rust
// core/src/currency/service.rs

use rust_decimal::Decimal;
use chrono::NaiveDate;
use uuid::Uuid;

pub struct ExchangeRateService;

impl ExchangeRateService {
    /// Get exchange rate for a specific date
    /// Returns the most recent rate on or before the given date
    pub async fn get_rate(
        db: &DatabaseConnection,
        org_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        date: NaiveDate,
    ) -> Result<Decimal, CurrencyError> {
        if from_currency == to_currency {
            return Ok(Decimal::ONE);
        }

        // Try direct rate
        if let Some(rate) = Self::find_direct_rate(db, org_id, from_currency, to_currency, date).await? {
            return Ok(rate);
        }

        // Try inverse rate
        if let Some(rate) = Self::find_direct_rate(db, org_id, to_currency, from_currency, date).await? {
            return Ok(Decimal::ONE / rate);
        }

        // Try triangulation through USD
        if from_currency != "USD" && to_currency != "USD" {
            let from_usd = Self::get_rate(db, org_id, from_currency, "USD", date).await?;
            let usd_to = Self::get_rate(db, org_id, "USD", to_currency, date).await?;
            return Ok(from_usd * usd_to);
        }

        Err(CurrencyError::NoRateFound {
            from: from_currency.to_string(),
            to: to_currency.to_string(),
            date,
        })
    }

    /// Convert amount from one currency to another
    pub fn convert(
        amount: Decimal,
        rate: Decimal,
    ) -> Decimal {
        (amount * rate).round_dp(4)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurrencyError {
    #[error("No exchange rate found for {from} to {to} on {date}")]
    NoRateFound {
        from: String,
        to: String,
        date: chrono::NaiveDate,
    },
    
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
```

### Currency Revaluation (Month-End)

```rust
// core/src/currency/revaluation.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::NaiveDate;

/// Revalue foreign currency accounts at period end
/// Creates adjustment entries for unrealized gains/losses
pub struct CurrencyRevaluation;

impl CurrencyRevaluation {
    pub async fn revalue_period(
        db: &DatabaseConnection,
        org_id: Uuid,
        period_end_date: NaiveDate,
    ) -> Result<Vec<RevaluationResult>, CurrencyError> {
        // 1. Find all accounts with foreign currency balances
        // 2. Get current exchange rate at period end
        // 3. Calculate unrealized gain/loss
        // 4. Create adjustment journal entries
        
        // Debit/Credit: Unrealized Currency Gain/Loss account
        // Credit/Debit: The foreign currency account
        
        todo!("Implement currency revaluation")
    }
}

pub struct RevaluationResult {
    pub account_id: Uuid,
    pub original_functional_balance: Decimal,
    pub revalued_balance: Decimal,
    pub gain_loss: Decimal,
    pub adjustment_transaction_id: Uuid,
}
```

### Rounding Strategy

Di accounting, pembagian amount TIDAK boleh kehilangan sen. Setiap sen harus ter-account.

#### The Problem

```
$100.00 dibagi 3 orang:
- SALAH: $33.33 + $33.33 + $33.33 = $99.99 (hilang $0.01)
- BENAR: $33.34 + $33.33 + $33.33 = $100.00
```

#### rust_decimal Rounding Strategies

`rust_decimal` menyediakan beberapa rounding strategy via `RoundingStrategy` enum:

```rust
use rust_decimal::RoundingStrategy;

// Available strategies:
RoundingStrategy::MidpointNearestEven    // Banker's Rounding (DEFAULT)
RoundingStrategy::MidpointAwayFromZero   // Traditional rounding (0.5 → 1)
RoundingStrategy::MidpointTowardZero     // 0.5 → 0
RoundingStrategy::ToZero                 // Truncate toward zero
RoundingStrategy::AwayFromZero           // Always round away from zero
RoundingStrategy::ToNegativeInfinity     // Floor
RoundingStrategy::ToPositiveInfinity     // Ceiling

// Aliases
RoundingStrategy::BankersRounding        // = MidpointNearestEven
RoundingStrategy::RoundHalfUp            // = MidpointAwayFromZero
RoundingStrategy::RoundHalfDown          // = MidpointTowardZero
RoundingStrategy::RoundDown              // = ToZero
RoundingStrategy::RoundUp                // = AwayFromZero
```

#### Banker's Rounding (Half-Even) - DEFAULT

`rust_decimal` default menggunakan Banker's Rounding (IEEE 754 standard):

```rust
use rust_decimal_macros::dec;

// Midpoint values round to EVEN number
dec!(2.5).round_dp(0)  // → 2 (round to even)
dec!(3.5).round_dp(0)  // → 4 (round to even)
dec!(2.25).round_dp(1) // → 2.2 (round to even)
dec!(2.35).round_dp(1) // → 2.4 (round to even)

// Non-midpoint values round normally
dec!(2.4).round_dp(0)  // → 2
dec!(2.6).round_dp(0)  // → 3
```

Kenapa Banker's Rounding?
- Mengurangi bias kumulatif (0.5 kadang naik, kadang turun)
- Standard di financial industry
- IEEE 754 compliant

#### Explicit Rounding Strategy

```rust
use rust_decimal::prelude::*;
use rust_decimal::RoundingStrategy;

let amount = dec!(2.555);

// Banker's Rounding (default)
amount.round_dp(2)  // → 2.56

// Explicit strategy
amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)  // → 2.56
amount.round_dp_with_strategy(2, RoundingStrategy::MidpointTowardZero)    // → 2.55
amount.round_dp_with_strategy(2, RoundingStrategy::ToZero)                // → 2.55
```

#### Largest Remainder Method (Hamilton Method)

Untuk allocation yang fair, gunakan Largest Remainder Method:

1. Bagi total dengan jumlah recipient, dapat integer + remainder
2. Berikan integer ke semua recipient
3. Sort by remainder descending
4. Distribute sisa seats ke yang punya remainder terbesar

```rust
// core/src/currency/allocation.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Allocate amount equally across N recipients using Largest Remainder Method
/// Ensures sum of allocations EXACTLY equals total (no penny lost)
pub fn allocate_equal(total: Decimal, count: usize, decimal_places: u32) -> Vec<Decimal> {
    if count == 0 {
        return vec![];
    }
    if count == 1 {
        return vec![total];
    }

    let count_dec = Decimal::from(count as u64);
    
    // Calculate exact quotient (high precision)
    let exact_each = total / count_dec;
    
    // Round down to get base allocation
    let base = exact_each.round_dp_with_strategy(decimal_places, RoundingStrategy::ToZero);
    
    // Calculate how much we've allocated
    let allocated = base * count_dec;
    
    // Calculate remainder to distribute
    let remainder = total - allocated;
    
    // How many recipients get +1 smallest unit?
    let unit = Decimal::new(1, decimal_places);
    let extra_count = (remainder / unit).to_u64().unwrap_or(0) as usize;
    
    // Build result: first N items get the extra unit
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        if i < extra_count {
            result.push(base + unit);
        } else {
            result.push(base);
        }
    }
    
    result
}

/// Allocate by percentages using Largest Remainder Method
/// Percentages should sum to 100
pub fn allocate_by_percentages(
    total: Decimal,
    percentages: &[Decimal],
    decimal_places: u32,
) -> Vec<Decimal> {
    if percentages.is_empty() {
        return vec![];
    }

    let hundred = dec!(100);
    let unit = Decimal::new(1, decimal_places);

    // Step 1: Calculate exact allocations (high precision)
    let exact: Vec<Decimal> = percentages
        .iter()
        .map(|p| total * *p / hundred)
        .collect();

    // Step 2: Round down each allocation
    let mut rounded: Vec<Decimal> = exact
        .iter()
        .map(|a| a.round_dp_with_strategy(decimal_places, RoundingStrategy::ToZero))
        .collect();

    // Step 3: Calculate remainder
    let sum_rounded: Decimal = rounded.iter().copied().sum();
    let remainder = total - sum_rounded;
    
    // Step 4: How many units to distribute?
    let units_to_distribute = (remainder / unit).to_u64().unwrap_or(0) as usize;
    
    if units_to_distribute == 0 {
        return rounded;
    }

    // Step 5: Calculate fractional remainders for each allocation
    let mut remainders: Vec<(usize, Decimal)> = exact
        .iter()
        .zip(rounded.iter())
        .enumerate()
        .map(|(i, (exact, rounded))| {
            // Fractional part that was lost in rounding
            (i, *exact - *rounded)
        })
        .collect();

    // Step 6: Sort by remainder descending (largest first)
    remainders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Step 7: Give +1 unit to items with largest remainders
    for (idx, _) in remainders.iter().take(units_to_distribute) {
        rounded[*idx] += unit;
    }

    rounded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_100_by_3() {
        let result = allocate_equal(dec!(100.00), 3, 2);
        
        // Should be $33.34, $33.33, $33.33 (first gets extra penny)
        assert_eq!(result[0], dec!(33.34));
        assert_eq!(result[1], dec!(33.33));
        assert_eq!(result[2], dec!(33.33));
        
        // Sum MUST equal original
        let sum: Decimal = result.iter().copied().sum();
        assert_eq!(sum, dec!(100.00));
    }

    #[test]
    fn test_allocate_100_by_7() {
        let result = allocate_equal(dec!(100.00), 7, 2);
        
        // 100 / 7 = 14.285714...
        // Base = 14.28, allocated = 99.96, remainder = 0.04 = 4 pennies
        // First 4 get 14.29, last 3 get 14.28
        assert_eq!(result[0], dec!(14.29));
        assert_eq!(result[1], dec!(14.29));
        assert_eq!(result[2], dec!(14.29));
        assert_eq!(result[3], dec!(14.29));
        assert_eq!(result[4], dec!(14.28));
        assert_eq!(result[5], dec!(14.28));
        assert_eq!(result[6], dec!(14.28));
        
        let sum: Decimal = result.iter().copied().sum();
        assert_eq!(sum, dec!(100.00));
    }

    #[test]
    fn test_allocate_by_percentages_exact() {
        // 50%, 30%, 20% of $100 - no remainder
        let result = allocate_by_percentages(
            dec!(100.00),
            &[dec!(50), dec!(30), dec!(20)],
            2,
        );
        
        assert_eq!(result, vec![dec!(50.00), dec!(30.00), dec!(20.00)]);
        assert_eq!(result.iter().copied().sum::<Decimal>(), dec!(100.00));
    }

    #[test]
    fn test_allocate_by_percentages_with_remainder() {
        // 33.33%, 33.33%, 33.34% of $100
        let result = allocate_by_percentages(
            dec!(100.00),
            &[dec!(33.33), dec!(33.33), dec!(33.34)],
            2,
        );
        
        // Sum MUST equal $100.00
        let sum: Decimal = result.iter().copied().sum();
        assert_eq!(sum, dec!(100.00));
    }

    #[test]
    fn test_allocate_expense_by_department() {
        // Marketing 40%, Engineering 35%, Sales 25% of $1000
        let result = allocate_by_percentages(
            dec!(1000.00),
            &[dec!(40), dec!(35), dec!(25)],
            2,
        );
        
        assert_eq!(result, vec![dec!(400.00), dec!(350.00), dec!(250.00)]);
        assert_eq!(result.iter().copied().sum::<Decimal>(), dec!(1000.00));
    }

    #[test]
    fn test_allocate_tricky_percentages() {
        // 33%, 33%, 34% of $99.99
        let result = allocate_by_percentages(
            dec!(99.99),
            &[dec!(33), dec!(33), dec!(34)],
            2,
        );
        
        // Sum MUST equal $99.99
        let sum: Decimal = result.iter().copied().sum();
        assert_eq!(sum, dec!(99.99));
    }
}
```

#### Usage in Ledger Service

```rust
use crate::currency::allocation::{allocate_equal, allocate_by_percentages};

// Split expense equally across 3 cost centers
let allocations = allocate_equal(dec!(1000.00), 3, 4);
// → [333.3334, 333.3333, 333.3333] (sum = 1000.0000)

// Split by department percentages
let dept_allocations = allocate_by_percentages(
    dec!(1000.00),
    &[dec!(40), dec!(35), dec!(25)],  // Marketing, Engineering, Sales
    4,  // 4 decimal places for functional_amount
);

// Create ledger entries
for (i, amount) in dept_allocations.iter().enumerate() {
    // Each entry gets its allocated portion
    // Total will ALWAYS equal original amount
}
```

#### Key Rules

1. **NEVER** use `f64` for money calculations
2. **ALWAYS** use `rust_decimal::Decimal`
3. **ALWAYS** verify sum after allocation equals original
4. Use `round_dp(4)` for `functional_amount` (4 decimal places)
5. Use `round_dp(2)` for display/reporting
6. Use **Largest Remainder Method** for fair distribution
7. Default to **Banker's Rounding** (`MidpointNearestEven`)
8. For tax calculations, check local regulations (some require `RoundHalfUp`)

---

## 3. Simulation Engine

### Overview

The simulation engine projects future financial states based on historical data and user-defined parameters. Supports dimensional filtering for granular projections.

### Simulation Parameters

```rust
// core/src/simulation/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SimulationParams {
    /// Base period for historical data
    pub base_period_start: chrono::NaiveDate,
    pub base_period_end: chrono::NaiveDate,
    
    /// How many months to project forward
    pub projection_months: u32,
    
    /// Global adjustments
    pub revenue_growth_rate: Decimal,      // e.g., 0.15 for 15% growth
    pub expense_growth_rate: Decimal,      // e.g., 0.05 for 5% inflation
    
    /// Per-account adjustments (override global)
    pub account_adjustments: HashMap<Uuid, Decimal>,
    
    /// Per-dimension adjustments
    pub dimension_adjustments: HashMap<Uuid, Decimal>,  // dimension_value_id -> adjustment
    
    /// Filter by dimensions (only include these in projection)
    pub dimension_filters: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ProjectionPeriod {
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub period_name: String,  // e.g., "2026-Q1", "2026-02"
}

#[derive(Debug)]
pub struct ProjectionResult {
    pub period: ProjectionPeriod,
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    
    /// Historical baseline (average or trend)
    pub baseline_amount: Decimal,
    
    /// Projected amount after adjustments
    pub projected_amount: Decimal,
    
    /// Breakdown by dimension (if filtered)
    pub dimension_breakdown: HashMap<Uuid, Decimal>,
}

#[derive(Debug)]
pub struct SimulationSummary {
    pub total_projected_revenue: Decimal,
    pub total_projected_expenses: Decimal,
    pub projected_net_income: Decimal,
    pub projections: Vec<ProjectionResult>,
}
```

### Simulation Engine Implementation

```rust
// core/src/simulation/engine.rs

use rust_decimal::Decimal;
use rayon::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::simulation::types::*;

pub struct SimulationEngine;

impl SimulationEngine {
    /// Run simulation with given parameters
    /// Uses Rayon for parallel computation across accounts
    pub fn run(
        historical_data: Vec<HistoricalAccountData>,
        params: &SimulationParams,
    ) -> SimulationSummary {
        // Group by account and compute in parallel
        let projections: Vec<ProjectionResult> = historical_data
            .par_iter()
            .flat_map(|account_data| {
                Self::project_account(account_data, params)
            })
            .collect();

        // Calculate summary
        let total_projected_revenue: Decimal = projections
            .iter()
            .filter(|p| p.account_type == "revenue")
            .map(|p| p.projected_amount)
            .sum();

        let total_projected_expenses: Decimal = projections
            .iter()
            .filter(|p| p.account_type == "expense")
            .map(|p| p.projected_amount)
            .sum();

        SimulationSummary {
            total_projected_revenue,
            total_projected_expenses,
            projected_net_income: total_projected_revenue - total_projected_expenses,
            projections,
        }
    }

    fn project_account(
        data: &HistoricalAccountData,
        params: &SimulationParams,
    ) -> Vec<ProjectionResult> {
        // Calculate baseline using linear regression or simple average
        let baseline = Self::calculate_baseline(&data.monthly_amounts);
        
        // Get adjustment factor
        let adjustment = params
            .account_adjustments
            .get(&data.account_id)
            .copied()
            .unwrap_or_else(|| {
                // Use global rate based on account type
                if data.account_type == "revenue" {
                    params.revenue_growth_rate
                } else {
                    params.expense_growth_rate
                }
            });

        // Generate projections for each future period
        let mut results = Vec::with_capacity(params.projection_months as usize);
        let mut current_date = params.base_period_end;

        for month in 1..=params.projection_months {
            current_date = Self::add_months(current_date, 1);
            
            // Compound growth
            let growth_factor = (Decimal::ONE + adjustment).powd(Decimal::from(month));
            let projected = baseline * growth_factor;

            results.push(ProjectionResult {
                period: ProjectionPeriod {
                    period_start: Self::month_start(current_date),
                    period_end: Self::month_end(current_date),
                    period_name: current_date.format("%Y-%m").to_string(),
                },
                account_id: data.account_id,
                account_code: data.account_code.clone(),
                account_name: data.account_name.clone(),
                account_type: data.account_type.clone(),
                baseline_amount: baseline,
                projected_amount: projected.round_dp(4),
                dimension_breakdown: HashMap::new(),
            });
        }

        results
    }

    fn calculate_baseline(monthly_amounts: &[Decimal]) -> Decimal {
        if monthly_amounts.is_empty() {
            return Decimal::ZERO;
        }
        
        // Simple average for now
        // TODO: Implement linear regression for trend-based projection
        let sum: Decimal = monthly_amounts.iter().sum();
        sum / Decimal::from(monthly_amounts.len())
    }

    fn add_months(date: chrono::NaiveDate, months: u32) -> chrono::NaiveDate {
        // Implementation using chrono
        date.checked_add_months(chrono::Months::new(months))
            .unwrap_or(date)
    }

    fn month_start(date: chrono::NaiveDate) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
    }

    fn month_end(date: chrono::NaiveDate) -> chrono::NaiveDate {
        Self::month_start(Self::add_months(date, 1))
            .pred_opt()
            .unwrap_or(date)
    }
}

#[derive(Debug)]
pub struct HistoricalAccountData {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub monthly_amounts: Vec<Decimal>,
}
```

### Performance Considerations

1. Pre-aggregate historical data using materialized views in PostgreSQL
2. Cache simulation results with parameter hash as cache key (Redis)
3. Use Rayon for parallel computation across accounts
4. Stream results for large projections using async iterators
5. Use `tokio::task::spawn_blocking` for CPU-intensive simulation
6. Consider WebAssembly compilation for client-side preview simulations

---

## 4. Dimensional Reporting

### Overview

Slice and dice financial data by any combination of dimensions (Department, Project, Cost Center, Location, etc.).

### Query Builder

```rust
// core/src/reporting/dimensional.rs

use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct DimensionalQuery {
    pub organization_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    
    /// Group by these dimension types
    pub group_by_dimensions: Vec<String>,  // e.g., ["DEPARTMENT", "PROJECT"]
    
    /// Filter to specific dimension values
    pub dimension_filters: Vec<Uuid>,  // dimension_value_ids
    
    /// Filter to specific accounts
    pub account_filters: Vec<Uuid>,
    
    /// Filter by account type
    pub account_type_filter: Option<String>,
}

#[derive(Debug)]
pub struct DimensionalReportRow {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub dimensions: Vec<DimensionValue>,
    pub debit_total: Decimal,
    pub credit_total: Decimal,
    pub balance: Decimal,
}

#[derive(Debug)]
pub struct DimensionValue {
    pub dimension_type: String,
    pub dimension_code: String,
    pub dimension_name: String,
}
```

### SQL Query Generation

```sql
-- Example: P&L by Department for Q1 2026
SELECT 
    coa.code AS account_code,
    coa.name AS account_name,
    coa.account_type,
    dv.code AS department_code,
    dv.name AS department_name,
    SUM(le.debit) AS total_debit,
    SUM(le.credit) AS total_credit,
    CASE 
        WHEN coa.account_type IN ('asset', 'expense') 
            THEN SUM(le.debit) - SUM(le.credit)
        ELSE SUM(le.credit) - SUM(le.debit)
    END AS balance
FROM ledger_entries le
JOIN transactions t ON t.id = le.transaction_id
JOIN chart_of_accounts coa ON coa.id = le.account_id
JOIN entry_dimensions ed ON ed.ledger_entry_id = le.id
JOIN dimension_values dv ON dv.id = ed.dimension_value_id
JOIN dimension_types dt ON dt.id = dv.dimension_type_id
WHERE t.organization_id = $1
  AND t.status = 'posted'
  AND t.transaction_date BETWEEN $2 AND $3
  AND dt.code = 'DEPARTMENT'
  AND coa.account_type IN ('revenue', 'expense')
GROUP BY coa.id, coa.code, coa.name, coa.account_type, dv.code, dv.name
ORDER BY coa.code, dv.code;
```

---

## 5. Approval Workflow

### States

```
draft ──► pending ──► approved ──► posted
                │
                └──► rejected ──► draft (can resubmit)
                
posted ──► voided (with reversing entry)
```

### Approval Rules Engine

```rust
// core/src/workflow/approval.rs

use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug)]
pub struct ApprovalRule {
    pub id: Uuid,
    pub name: String,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub transaction_types: Vec<String>,
    pub required_role: String,
    pub priority: i16,
}

pub struct ApprovalEngine;

impl ApprovalEngine {
    /// Determine required approver role for a transaction
    pub fn get_required_approval(
        rules: &[ApprovalRule],
        transaction_type: &str,
        total_amount: Decimal,
    ) -> Option<String> {
        // Sort by priority (lower = higher priority)
        let mut applicable: Vec<_> = rules
            .iter()
            .filter(|r| r.transaction_types.contains(&transaction_type.to_string()))
            .filter(|r| {
                let above_min = r.min_amount.map_or(true, |min| total_amount >= min);
                let below_max = r.max_amount.map_or(true, |max| total_amount <= max);
                above_min && below_max
            })
            .collect();

        applicable.sort_by_key(|r| r.priority);
        applicable.first().map(|r| r.required_role.clone())
    }

    /// Check if user can approve a transaction
    pub fn can_approve(
        user_role: &str,
        user_approval_limit: Option<Decimal>,
        required_role: &str,
        transaction_amount: Decimal,
    ) -> bool {
        let role_hierarchy = ["viewer", "submitter", "approver", "accountant", "admin", "owner"];
        
        let user_level = role_hierarchy.iter().position(|&r| r == user_role).unwrap_or(0);
        let required_level = role_hierarchy.iter().position(|&r| r == required_role).unwrap_or(999);

        if user_level < required_level {
            return false;
        }

        // Check approval limit
        if let Some(limit) = user_approval_limit {
            if transaction_amount > limit {
                return false;
            }
        }

        true
    }
}
```

---

## 6. Dashboard Metrics

### Key Metrics

1. Burn Rate: Total expenses / days in period
2. Runway: Cash balance / daily burn rate
3. Budget Utilization: Actual / Budget per account (with dimensional breakdown)
4. Top Spenders: By department, project, or cost center
5. Pending Approvals: Count and total amount awaiting approval
6. Currency Exposure: Balances by currency with unrealized gain/loss

### Real-time Updates (Frontend)

```typescript
// lib/queries/useDashboard.ts

import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';

interface DashboardMetrics {
  burnRate: number;
  runwayDays: number;
  cashBalance: number;
  pendingApprovals: {
    count: number;
    totalAmount: number;
  };
  budgetUtilization: {
    accountId: string;
    accountName: string;
    budgeted: number;
    actual: number;
    variance: number;
    utilizationPercent: number;
  }[];
  topSpenders: {
    dimensionName: string;
    amount: number;
    percentOfTotal: number;
  }[];
}

export function useDashboardMetrics(fiscalPeriodId: string) {
  return useQuery({
    queryKey: ['dashboard', 'metrics', fiscalPeriodId],
    queryFn: () => api.get<DashboardMetrics>(`/dashboard/metrics?period=${fiscalPeriodId}`),
    staleTime: 30 * 1000,
    refetchInterval: 60 * 1000,
    gcTime: 5 * 60 * 1000,
  });
}

export function useBudgetVsActual(fiscalPeriodId: string, dimensionValueId?: string) {
  return useQuery({
    queryKey: ['dashboard', 'budget-vs-actual', fiscalPeriodId, dimensionValueId],
    queryFn: () => api.get('/reports/budget-vs-actual', {
      params: { period: fiscalPeriodId, dimension: dimensionValueId }
    }),
    staleTime: 60 * 1000,
  });
}
```

### Zustand Store (Client State)

```typescript
// lib/stores/dashboardStore.ts

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface DashboardFilters {
  selectedPeriodId: string | null;
  selectedDimensionType: string | null;
  selectedDimensionValues: string[];
  dateRange: {
    start: Date | null;
    end: Date | null;
  };
}

interface DashboardStore extends DashboardFilters {
  setSelectedPeriod: (periodId: string) => void;
  setDimensionType: (type: string | null) => void;
  toggleDimensionValue: (valueId: string) => void;
  setDateRange: (start: Date | null, end: Date | null) => void;
  resetFilters: () => void;
}

const initialState: DashboardFilters = {
  selectedPeriodId: null,
  selectedDimensionType: null,
  selectedDimensionValues: [],
  dateRange: { start: null, end: null },
};

export const useDashboardStore = create<DashboardStore>()(
  persist(
    (set) => ({
      ...initialState,
      
      setSelectedPeriod: (periodId) => set({ selectedPeriodId: periodId }),
      
      setDimensionType: (type) => set({ 
        selectedDimensionType: type,
        selectedDimensionValues: [], // Reset values when type changes
      }),
      
      toggleDimensionValue: (valueId) => set((state) => ({
        selectedDimensionValues: state.selectedDimensionValues.includes(valueId)
          ? state.selectedDimensionValues.filter(id => id !== valueId)
          : [...state.selectedDimensionValues, valueId],
      })),
      
      setDateRange: (start, end) => set({ dateRange: { start, end } }),
      
      resetFilters: () => set(initialState),
    }),
    {
      name: 'dashboard-filters',
    }
  )
);
```
