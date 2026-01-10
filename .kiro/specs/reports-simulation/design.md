# Design Document

## Overview

The Reports & Simulation module provides comprehensive financial reporting, budget management, and forecasting capabilities for Zeltra. This module builds on top of the Ledger Core, aggregating posted ledger entries to generate standard financial reports (Trial Balance, Balance Sheet, Income Statement), budget variance analysis, and future projections.

The architecture follows a layered approach:
1. **Report Service** (core crate) - Pure business logic for report generation
2. **Budget Service** (core crate) - Budget management and variance calculation
3. **Simulation Engine** (core crate) - Projection calculations with Rayon parallelism
4. **Report Repository** (db crate) - Database queries using SeaORM
5. **Report Routes** (api crate) - REST API endpoints

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Layer (Axum)                        │
│  /reports/*  /budgets/*  /simulation/*  /dashboard/*            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Core Services                              │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐        │
│  │ReportService│  │BudgetService│  │SimulationEngine  │        │
│  │             │  │             │  │  (Rayon parallel)│        │
│  └─────────────┘  └─────────────┘  └──────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Repository Layer                            │
│  ┌────────────────┐  ┌────────────────┐  ┌─────────────────┐   │
│  │ReportRepository│  │BudgetRepository│  │DashboardRepository│  │
│  └────────────────┘  └────────────────┘  └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    PostgreSQL Database                           │
│  ┌──────────────────┐  ┌─────────────────────┐                  │
│  │trial_balance_view│  │budget_vs_actual_view│                  │
│  └──────────────────┘  └─────────────────────┘                  │
│  ┌──────────────────┐  ┌─────────────────────┐                  │
│  │ledger_entries    │  │budgets, budget_lines│                  │
│  └──────────────────┘  └─────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Report Service

```rust
// core/src/reports/service.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::NaiveDate;

pub struct ReportService;

impl ReportService {
    /// Generate trial balance as of a specific date
    pub fn generate_trial_balance(
        accounts: Vec<AccountBalance>,
    ) -> TrialBalanceReport {
        let total_debit: Decimal = accounts.iter().map(|a| a.total_debit).sum();
        let total_credit: Decimal = accounts.iter().map(|a| a.total_credit).sum();
        
        TrialBalanceReport {
            accounts,
            totals: TrialBalanceTotals {
                total_debit,
                total_credit,
                is_balanced: total_debit == total_credit,
            },
        }
    }

    /// Generate balance sheet as of a specific date
    pub fn generate_balance_sheet(
        accounts: Vec<AccountBalance>,
    ) -> BalanceSheetReport {
        let mut assets = BalanceSheetSection::default();
        let mut liabilities = BalanceSheetSection::default();
        let mut equity = BalanceSheetSection::default();

        for account in accounts {
            match account.account_type.as_str() {
                "asset" => Self::add_to_section(&mut assets, account),
                "liability" => Self::add_to_section(&mut liabilities, account),
                "equity" => Self::add_to_section(&mut equity, account),
                _ => {}
            }
        }

        let total_assets = assets.total;
        let total_liabilities = liabilities.total;
        let total_equity = equity.total;

        BalanceSheetReport {
            assets,
            liabilities,
            equity,
            total_assets,
            total_liabilities,
            total_equity,
            liabilities_and_equity: total_liabilities + total_equity,
            is_balanced: total_assets == total_liabilities + total_equity,
        }
    }

    /// Generate income statement for a date range
    pub fn generate_income_statement(
        accounts: Vec<AccountBalance>,
    ) -> IncomeStatementReport {
        let mut revenue = IncomeStatementSection::default();
        let mut cogs = IncomeStatementSection::default();
        let mut operating_expenses = IncomeStatementSection::default();
        let mut other = IncomeStatementSection::default();

        for account in accounts {
            match (account.account_type.as_str(), account.account_subtype.as_deref()) {
                ("revenue", _) => Self::add_to_income_section(&mut revenue, account),
                ("expense", Some("cost_of_goods_sold")) => Self::add_to_income_section(&mut cogs, account),
                ("expense", Some("operating_expense")) => Self::add_to_income_section(&mut operating_expenses, account),
                ("expense", _) => Self::add_to_income_section(&mut other, account),
                _ => {}
            }
        }

        let gross_profit = revenue.total - cogs.total;
        let operating_income = gross_profit - operating_expenses.total;
        let net_income = operating_income - other.total;

        IncomeStatementReport {
            revenue,
            cost_of_goods_sold: cogs,
            gross_profit,
            operating_expenses,
            operating_income,
            other_income_expense: other,
            net_income,
        }
    }

    fn add_to_section(section: &mut BalanceSheetSection, account: AccountBalance) {
        section.total += account.balance;
        section.accounts.push(account);
    }

    fn add_to_income_section(section: &mut IncomeStatementSection, account: AccountBalance) {
        section.total += account.balance.abs();
        section.accounts.push(account);
    }
}
```

### Budget Service

```rust
// core/src/budget/service.rs

use rust_decimal::Decimal;
use uuid::Uuid;

pub struct BudgetService;

impl BudgetService {
    /// Calculate variance between budgeted and actual amounts
    pub fn calculate_variance(
        budgeted: Decimal,
        actual: Decimal,
        account_type: &str,
    ) -> VarianceResult {
        let variance = match account_type {
            "expense" => budgeted - actual,  // Positive = under budget (favorable)
            "revenue" => actual - budgeted,  // Positive = over target (favorable)
            _ => budgeted - actual,
        };

        let status = if variance > Decimal::ZERO {
            VarianceStatus::Favorable
        } else if variance < Decimal::ZERO {
            VarianceStatus::Unfavorable
        } else {
            VarianceStatus::OnBudget
        };

        let utilization_percent = if budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (actual / budgeted * Decimal::from(100)).round_dp(2)
        };

        VarianceResult {
            budgeted,
            actual,
            variance,
            variance_percent: if budgeted.is_zero() {
                Decimal::ZERO
            } else {
                (variance / budgeted * Decimal::from(100)).round_dp(2)
            },
            utilization_percent,
            status,
        }
    }

    /// Validate budget line creation
    pub fn validate_budget_line(
        budget: &Budget,
        account_id: Uuid,
        fiscal_period_id: Uuid,
        amount: Decimal,
    ) -> Result<(), BudgetError> {
        if budget.is_locked {
            return Err(BudgetError::BudgetLocked);
        }

        if amount < Decimal::ZERO {
            return Err(BudgetError::NegativeAmount);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarianceStatus {
    Favorable,
    Unfavorable,
    OnBudget,
}
```

### Simulation Engine

```rust
// core/src/simulation/engine.rs

use rust_decimal::Decimal;
use rayon::prelude::*;
use uuid::Uuid;
use chrono::NaiveDate;

pub struct SimulationEngine;

impl SimulationEngine {
    /// Run simulation with parallel processing
    pub fn run(
        historical_data: Vec<HistoricalAccountData>,
        params: &SimulationParams,
    ) -> SimulationResult {
        // Use Rayon for parallel computation across accounts
        let projections: Vec<AccountProjection> = historical_data
            .par_iter()
            .flat_map(|account| Self::project_account(account, params))
            .collect();

        // Calculate summary totals
        let mut total_revenue = Decimal::ZERO;
        let mut total_expenses = Decimal::ZERO;

        for projection in &projections {
            match projection.account_type.as_str() {
                "revenue" => total_revenue += projection.projected_amount,
                "expense" => total_expenses += projection.projected_amount,
                _ => {}
            }
        }

        SimulationResult {
            simulation_id: Uuid::new_v4(),
            parameters_hash: Self::hash_params(params),
            projections,
            annual_summary: AnnualSummary {
                total_projected_revenue: total_revenue,
                total_projected_expenses: total_expenses,
                projected_net_income: total_revenue - total_expenses,
            },
            cached: false,
        }
    }

    fn project_account(
        data: &HistoricalAccountData,
        params: &SimulationParams,
    ) -> Vec<AccountProjection> {
        let baseline = Self::calculate_baseline(&data.monthly_amounts);
        
        // Get growth rate (account-specific or global)
        let growth_rate = params
            .account_adjustments
            .get(&data.account_id)
            .copied()
            .unwrap_or_else(|| {
                if data.account_type == "revenue" {
                    params.revenue_growth_rate
                } else {
                    params.expense_growth_rate
                }
            });

        let mut projections = Vec::with_capacity(params.projection_months as usize);
        let mut current_date = params.base_period_end;

        for month in 1..=params.projection_months {
            current_date = Self::add_months(current_date, 1);
            
            // Compound growth: baseline * (1 + rate)^month
            let growth_factor = (Decimal::ONE + growth_rate)
                .powd(Decimal::from(month));
            let projected = (baseline * growth_factor).round_dp(4);

            projections.push(AccountProjection {
                period_name: current_date.format("%Y-%m").to_string(),
                period_start: Self::month_start(current_date),
                period_end: Self::month_end(current_date),
                account_id: data.account_id,
                account_code: data.account_code.clone(),
                account_name: data.account_name.clone(),
                account_type: data.account_type.clone(),
                baseline_amount: baseline,
                projected_amount: projected,
                change_percent: if baseline.is_zero() {
                    Decimal::ZERO
                } else {
                    ((projected - baseline) / baseline * Decimal::from(100)).round_dp(2)
                },
            });
        }

        projections
    }

    fn calculate_baseline(monthly_amounts: &[Decimal]) -> Decimal {
        if monthly_amounts.is_empty() {
            return Decimal::ZERO;
        }
        
        let sum: Decimal = monthly_amounts.iter().sum();
        (sum / Decimal::from(monthly_amounts.len() as u64)).round_dp(4)
    }

    fn hash_params(params: &SimulationParams) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        // Hash key parameters
        params.base_period_start.hash(&mut hasher);
        params.base_period_end.hash(&mut hasher);
        params.projection_months.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
        date.checked_add_months(chrono::Months::new(months))
            .unwrap_or(date)
    }

    fn month_start(date: NaiveDate) -> NaiveDate {
        NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
    }

    fn month_end(date: NaiveDate) -> NaiveDate {
        Self::month_start(Self::add_months(date, 1))
            .pred_opt()
            .unwrap_or(date)
    }
}
```

## Data Models

### Report Types

```rust
// core/src/reports/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub account_subtype: Option<String>,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceReport {
    pub report_type: String,
    pub as_of: chrono::NaiveDate,
    pub currency: String,
    pub accounts: Vec<AccountBalance>,
    pub totals: TrialBalanceTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceTotals {
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceSheetSection {
    pub total: Decimal,
    pub accounts: Vec<AccountBalance>,
    pub subsections: Vec<BalanceSheetSubsection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetSubsection {
    pub name: String,
    pub total: Decimal,
    pub accounts: Vec<AccountBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetReport {
    pub report_type: String,
    pub as_of: chrono::NaiveDate,
    pub currency: String,
    pub assets: BalanceSheetSection,
    pub liabilities: BalanceSheetSection,
    pub equity: BalanceSheetSection,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub total_equity: Decimal,
    pub liabilities_and_equity: Decimal,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncomeStatementSection {
    pub total: Decimal,
    pub accounts: Vec<AccountBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatementReport {
    pub report_type: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub currency: String,
    pub revenue: IncomeStatementSection,
    pub cost_of_goods_sold: IncomeStatementSection,
    pub gross_profit: Decimal,
    pub operating_expenses: IncomeStatementSection,
    pub operating_income: Decimal,
    pub other_income_expense: IncomeStatementSection,
    pub net_income: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedgerEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub transaction_date: chrono::NaiveDate,
    pub description: String,
    pub source_currency: String,
    pub source_amount: Decimal,
    pub exchange_rate: Decimal,
    pub functional_amount: Decimal,
    pub debit: Decimal,
    pub credit: Decimal,
    pub running_balance: Decimal,
    pub dimensions: Vec<DimensionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionInfo {
    pub dimension_type: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalReportRow {
    pub dimensions: Vec<DimensionInfo>,
    pub accounts: Vec<AccountBalance>,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalReport {
    pub report_type: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub group_by: Vec<String>,
    pub data: Vec<DimensionalReportRow>,
    pub grand_total: Decimal,
}
```

### Budget Types

```rust
// core/src/budget/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetType {
    Annual,
    Quarterly,
    Monthly,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub budget_type: BudgetType,
    pub currency: String,
    pub is_active: bool,
    pub is_locked: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLine {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub account_id: Uuid,
    pub fiscal_period_id: Uuid,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub dimensions: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLineWithActual {
    pub id: Uuid,
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub fiscal_period_id: Uuid,
    pub period_name: String,
    pub budgeted: Decimal,
    pub actual: Decimal,
    pub variance: Decimal,
    pub utilization_percent: Decimal,
    pub status: String,
    pub dimensions: Vec<DimensionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub id: Uuid,
    pub name: String,
    pub fiscal_year: String,
    pub budget_type: BudgetType,
    pub currency: String,
    pub is_locked: bool,
    pub total_budgeted: Decimal,
    pub total_actual: Decimal,
    pub total_variance: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVsActualReport {
    pub budget_id: Uuid,
    pub budget_name: String,
    pub period: String,
    pub lines: Vec<BudgetLineWithActual>,
    pub summary: BudgetVsActualSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetVsActualSummary {
    pub total_budgeted: Decimal,
    pub total_actual: Decimal,
    pub total_variance: Decimal,
    pub overall_utilization: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceResult {
    pub budgeted: Decimal,
    pub actual: Decimal,
    pub variance: Decimal,
    pub variance_percent: Decimal,
    pub utilization_percent: Decimal,
    pub status: VarianceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarianceStatus {
    Favorable,
    Unfavorable,
    OnBudget,
}

#[derive(Debug, Clone)]
pub struct CreateBudgetInput {
    pub organization_id: Uuid,
    pub fiscal_year_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub budget_type: BudgetType,
    pub currency: String,
    pub created_by: Uuid,
}

#[derive(Debug, Clone)]
pub struct CreateBudgetLineInput {
    pub budget_id: Uuid,
    pub account_id: Uuid,
    pub fiscal_period_id: Uuid,
    pub amount: Decimal,
    pub notes: Option<String>,
    pub dimensions: Vec<Uuid>,
}
```

### Simulation Types

```rust
// core/src/simulation/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::NaiveDate;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    pub base_period_start: NaiveDate,
    pub base_period_end: NaiveDate,
    pub projection_months: u32,
    pub revenue_growth_rate: Decimal,
    pub expense_growth_rate: Decimal,
    pub account_adjustments: HashMap<Uuid, Decimal>,
    pub dimension_filters: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct HistoricalAccountData {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub monthly_amounts: Vec<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProjection {
    pub period_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub baseline_amount: Decimal,
    pub projected_amount: Decimal,
    pub change_percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualSummary {
    pub total_projected_revenue: Decimal,
    pub total_projected_expenses: Decimal,
    pub projected_net_income: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub simulation_id: Uuid,
    pub parameters_hash: String,
    pub projections: Vec<AccountProjection>,
    pub annual_summary: AnnualSummary,
    pub cached: bool,
}
```

### Dashboard Types

```rust
// core/src/dashboard/types.rs

use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub period: PeriodInfo,
    pub cash_position: CashPosition,
    pub burn_rate: BurnRate,
    pub runway_days: i32,
    pub pending_approvals: PendingApprovals,
    pub budget_status: BudgetStatus,
    pub top_expenses_by_department: Vec<DepartmentExpense>,
    pub currency_exposure: Vec<CurrencyExposure>,
    pub cash_flow_chart: CashFlowChart,
    pub utilization_chart: UtilizationChart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInfo {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPosition {
    pub balance: Decimal,
    pub currency: String,
    pub change_from_last_period: Decimal,
    pub change_percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRate {
    pub daily: Decimal,
    pub monthly: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApprovals {
    pub count: i32,
    pub total_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub total_budgeted: Decimal,
    pub total_spent: Decimal,
    pub utilization_percent: Decimal,
    pub days_remaining: i32,
    pub projected_end_of_period: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentExpense {
    pub department: String,
    pub amount: Decimal,
    pub percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExposure {
    pub currency: String,
    pub balance: Decimal,
    pub functional_value: Decimal,
    pub percent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowChart {
    pub labels: Vec<String>,
    pub inflow: Vec<Decimal>,
    pub outflow: Vec<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationChart {
    pub labels: Vec<String>,
    pub budgeted: Vec<Decimal>,
    pub actual: Vec<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: Uuid,
    pub event_type: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub description: String,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub user: UserInfo,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivityResponse {
    pub activities: Vec<ActivityEvent>,
    pub pagination: ActivityPagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPagination {
    pub limit: i32,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}
```



## Error Handling

### Budget Errors

```rust
// core/src/budget/error.rs

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("Budget not found: {0}")]
    NotFound(Uuid),

    #[error("Budget is locked and cannot be modified")]
    BudgetLocked,

    #[error("Budget name already exists for this fiscal year")]
    DuplicateName,

    #[error("Fiscal year not found: {0}")]
    FiscalYearNotFound(Uuid),

    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    #[error("Fiscal period does not belong to budget's fiscal year")]
    PeriodNotInFiscalYear,

    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    #[error("Budget line already exists for this account and period")]
    DuplicateBudgetLine,

    #[error("Amount cannot be negative")]
    NegativeAmount,

    #[error("Currency mismatch: expected {expected}, got {got}")]
    CurrencyMismatch { expected: String, got: String },

    #[error("Invalid dimension value: {0}")]
    InvalidDimension(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
```

### Report Errors

```rust
// core/src/reports/error.rs

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    #[error("Invalid date range: start {start} is after end {end}")]
    InvalidDateRange {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },

    #[error("Invalid dimension type: {0}")]
    InvalidDimensionType(String),

    #[error("No data found for the specified criteria")]
    NoDataFound,

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
```

### Simulation Errors

```rust
// core/src/simulation/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("Invalid base period: start {start} is after end {end}")]
    InvalidBasePeriod {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },

    #[error("Projection months must be between 1 and 60")]
    InvalidProjectionMonths,

    #[error("Growth rate must be between -1.0 and 10.0")]
    InvalidGrowthRate,

    #[error("No historical data found for the base period")]
    NoHistoricalData,

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system - essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Trial Balance Debits Equal Credits

*For any* valid ledger with posted transactions, the sum of all debit balances SHALL equal the sum of all credit balances in the trial balance report.

**Validates: Requirements 5.4, 5.5**

### Property 2: Balance Sheet Accounting Equation

*For any* valid ledger with posted transactions, the balance sheet SHALL satisfy: Total Assets = Total Liabilities + Total Equity.

**Validates: Requirements 6.5, 6.6**

### Property 3: Income Statement Net Income Calculation

*For any* income statement, Net Income SHALL equal Revenue - Cost of Goods Sold - Operating Expenses - Other Expenses + Other Income.

**Validates: Requirements 7.5, 7.6, 7.7**

### Property 4: Budget Variance Calculation by Account Type

*For any* budget line with budgeted amount B and actual amount A:
- For expense accounts: variance = B - A, favorable if variance > 0
- For revenue accounts: variance = A - B, favorable if variance > 0

**Validates: Requirements 4.4, 4.5, 4.6**

### Property 5: Actual Amount Calculation by Account Type

*For any* set of posted ledger entries for an account:
- For expense/asset accounts: actual = sum(debit) - sum(credit)
- For revenue/liability/equity accounts: actual = sum(credit) - sum(debit)

**Validates: Requirements 4.2, 4.3**

### Property 6: Utilization Percent Calculation

*For any* budget line with budgeted amount B and actual amount A:
- If B > 0: utilization_percent = (A / B) * 100
- If B = 0: utilization_percent = 0

**Validates: Requirements 4.8**

### Property 7: Budget Lock Prevents Modification

*For any* locked budget, all attempts to add, update, or delete budget lines SHALL be rejected.

**Validates: Requirements 1.6, 1.7**

### Property 8: Budget Line Uniqueness

*For any* budget, there SHALL be at most one budget line for each (account_id, fiscal_period_id) combination.

**Validates: Requirements 2.5**

### Property 9: Simulation Projection Count

*For any* simulation with projection_months = N, the result SHALL contain exactly N projection periods per account.

**Validates: Requirements 11.1**

### Property 10: Simulation Compound Growth Formula

*For any* account with baseline B and growth rate R, the projected amount for month M SHALL equal B * (1 + R)^M, rounded to 4 decimal places.

**Validates: Requirements 11.5**

### Property 11: Simulation Growth Rate Override

*For any* account with a specific account_adjustment rate, that rate SHALL be used instead of the global revenue_growth_rate or expense_growth_rate.

**Validates: Requirements 11.4**

### Property 12: Simulation Summary Totals

*For any* simulation result, total_projected_revenue SHALL equal the sum of all projected amounts for revenue accounts, and total_projected_expenses SHALL equal the sum of all projected amounts for expense accounts.

**Validates: Requirements 11.7**

### Property 13: Account Ledger Running Balance

*For any* account ledger entry, the running_balance SHALL equal the current_balance stored on the ledger entry (pre-calculated by database trigger).

**Validates: Requirements 8.3**

### Property 14: Account Ledger Ordering

*For any* account ledger, entries SHALL be ordered by transaction_date ascending, then by entry creation order.

**Validates: Requirements 8.6**

### Property 15: Dimensional Report Grand Total

*For any* dimensional report, the grand_total SHALL equal the sum of all individual dimension combination totals.

**Validates: Requirements 9.7**

### Property 16: Dimensional Report Grouping

*For any* dimensional report with group_by dimensions D1, D2, ..., Dn, each row SHALL have exactly one value for each dimension type in the group_by list.

**Validates: Requirements 9.1, 9.2**

### Property 17: Budget Summary Totals

*For any* budget, total_budgeted SHALL equal the sum of all budget line amounts, and total_actual SHALL equal the sum of all calculated actual amounts.

**Validates: Requirements 1.5**

### Property 18: Simulation Baseline Calculation

*For any* account with monthly amounts [A1, A2, ..., An] in the base period, the baseline SHALL equal (A1 + A2 + ... + An) / n, rounded to 4 decimal places.

**Validates: Requirements 10.3**

## Testing Strategy

### Property-Based Testing Library

Use `proptest` crate for property-based testing in Rust.

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.4"
```

### Test Configuration

- Minimum 100 iterations per property test
- Each property test must reference its design document property
- Tag format: `Feature: reports-simulation, Property N: {property_text}`

### Unit Tests

Unit tests should cover:
- Specific examples that demonstrate correct behavior
- Edge cases (zero amounts, empty data, boundary values)
- Error conditions (invalid inputs, missing data)

### Property Tests

Property tests should cover:
- All 18 correctness properties defined above
- Use generators for random valid inputs
- Verify invariants hold across all generated inputs

### Integration Tests

Integration tests should cover:
- Full API endpoint flows
- Database interactions
- Multi-step workflows (create budget → add lines → lock → query vs actual)

### Test File Structure

```
backend/crates/core/src/
├── budget/
│   ├── mod.rs
│   ├── service.rs
│   ├── types.rs
│   ├── error.rs
│   └── tests.rs          # Unit + property tests
├── reports/
│   ├── mod.rs
│   ├── service.rs
│   ├── types.rs
│   ├── error.rs
│   └── tests.rs          # Unit + property tests
├── simulation/
│   ├── mod.rs
│   ├── engine.rs
│   ├── types.rs
│   ├── error.rs
│   └── tests.rs          # Unit + property tests
└── dashboard/
    ├── mod.rs
    ├── types.rs
    └── tests.rs          # Unit tests

backend/crates/db/src/repositories/
├── budget_repo.rs
├── report_repo.rs
├── simulation_repo.rs
└── dashboard_repo.rs

backend/tests/
├── budget/
│   ├── test_budget_crud.rs
│   ├── test_budget_lines.rs
│   ├── test_budget_vs_actual.rs
│   └── test_budget_lock.rs
├── reports/
│   ├── test_trial_balance.rs
│   ├── test_balance_sheet.rs
│   ├── test_income_statement.rs
│   ├── test_account_ledger.rs
│   └── test_dimensional_report.rs
├── simulation/
│   ├── test_baseline_calculation.rs
│   ├── test_projection.rs
│   └── test_simulation_accuracy.rs
└── api/
    ├── test_budget_api.rs
    ├── test_reports_api.rs
    ├── test_simulation_api.rs
    └── test_dashboard_api.rs
```

### Example Property Test

```rust
// core/src/budget/tests.rs

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::budget::service::BudgetService;
use crate::budget::types::VarianceStatus;

proptest! {
    /// Feature: reports-simulation, Property 4: Budget Variance Calculation by Account Type
    #[test]
    fn test_variance_calculation_expense(
        budgeted in 0i64..1_000_000,
        actual in 0i64..1_000_000,
    ) {
        let budgeted = Decimal::from(budgeted);
        let actual = Decimal::from(actual);
        
        let result = BudgetService::calculate_variance(budgeted, actual, "expense");
        
        // For expenses: variance = budgeted - actual
        prop_assert_eq!(result.variance, budgeted - actual);
        
        // Favorable if under budget (variance > 0)
        if result.variance > Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Favorable);
        } else if result.variance < Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Unfavorable);
        } else {
            prop_assert_eq!(result.status, VarianceStatus::OnBudget);
        }
    }

    /// Feature: reports-simulation, Property 4: Budget Variance Calculation by Account Type
    #[test]
    fn test_variance_calculation_revenue(
        budgeted in 0i64..1_000_000,
        actual in 0i64..1_000_000,
    ) {
        let budgeted = Decimal::from(budgeted);
        let actual = Decimal::from(actual);
        
        let result = BudgetService::calculate_variance(budgeted, actual, "revenue");
        
        // For revenue: variance = actual - budgeted
        prop_assert_eq!(result.variance, actual - budgeted);
        
        // Favorable if over target (variance > 0)
        if result.variance > Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Favorable);
        } else if result.variance < Decimal::ZERO {
            prop_assert_eq!(result.status, VarianceStatus::Unfavorable);
        } else {
            prop_assert_eq!(result.status, VarianceStatus::OnBudget);
        }
    }

    /// Feature: reports-simulation, Property 6: Utilization Percent Calculation
    #[test]
    fn test_utilization_percent(
        budgeted in 1i64..1_000_000,  // Avoid zero to test normal case
        actual in 0i64..1_000_000,
    ) {
        let budgeted = Decimal::from(budgeted);
        let actual = Decimal::from(actual);
        
        let result = BudgetService::calculate_variance(budgeted, actual, "expense");
        
        let expected = (actual / budgeted * dec!(100)).round_dp(2);
        prop_assert_eq!(result.utilization_percent, expected);
    }

    /// Feature: reports-simulation, Property 6: Utilization Percent Calculation (zero budget)
    #[test]
    fn test_utilization_percent_zero_budget(
        actual in 0i64..1_000_000,
    ) {
        let budgeted = Decimal::ZERO;
        let actual = Decimal::from(actual);
        
        let result = BudgetService::calculate_variance(budgeted, actual, "expense");
        
        // Zero budget should result in zero utilization
        prop_assert_eq!(result.utilization_percent, Decimal::ZERO);
    }
}
```

### Example Simulation Property Test

```rust
// core/src/simulation/tests.rs

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::simulation::engine::SimulationEngine;
use crate::simulation::types::*;

proptest! {
    /// Feature: reports-simulation, Property 9: Simulation Projection Count
    #[test]
    fn test_projection_count(
        projection_months in 1u32..=60,
        num_accounts in 1usize..=10,
    ) {
        let historical_data: Vec<HistoricalAccountData> = (0..num_accounts)
            .map(|i| HistoricalAccountData {
                account_id: uuid::Uuid::new_v4(),
                account_code: format!("ACC{}", i),
                account_name: format!("Account {}", i),
                account_type: "expense".to_string(),
                monthly_amounts: vec![dec!(1000)],
            })
            .collect();

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months,
            revenue_growth_rate: dec!(0.1),
            expense_growth_rate: dec!(0.05),
            account_adjustments: std::collections::HashMap::new(),
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(historical_data, &params);
        
        // Each account should have exactly projection_months projections
        let projections_per_account = result.projections.len() / num_accounts;
        prop_assert_eq!(projections_per_account, projection_months as usize);
    }

    /// Feature: reports-simulation, Property 10: Simulation Compound Growth Formula
    #[test]
    fn test_compound_growth_formula(
        baseline in 100i64..100_000,
        growth_rate_percent in -50i32..100,  // -50% to +100%
        month in 1u32..=12,
    ) {
        let baseline = Decimal::from(baseline);
        let growth_rate = Decimal::from(growth_rate_percent) / dec!(100);
        
        let historical_data = vec![HistoricalAccountData {
            account_id: uuid::Uuid::new_v4(),
            account_code: "TEST".to_string(),
            account_name: "Test Account".to_string(),
            account_type: "expense".to_string(),
            monthly_amounts: vec![baseline],
        }];

        let params = SimulationParams {
            base_period_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            projection_months: month,
            revenue_growth_rate: dec!(0),
            expense_growth_rate: growth_rate,
            account_adjustments: std::collections::HashMap::new(),
            dimension_filters: vec![],
        };

        let result = SimulationEngine::run(historical_data, &params);
        
        // Get the last projection (month M)
        let last_projection = result.projections.last().unwrap();
        
        // Expected: baseline * (1 + rate)^month
        let expected = (baseline * (Decimal::ONE + growth_rate).powd(Decimal::from(month))).round_dp(4);
        
        prop_assert_eq!(last_projection.projected_amount, expected);
    }
}
```

### Example Report Property Test

```rust
// core/src/reports/tests.rs

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::reports::service::ReportService;
use crate::reports::types::*;

proptest! {
    /// Feature: reports-simulation, Property 1: Trial Balance Debits Equal Credits
    #[test]
    fn test_trial_balance_balanced(
        num_accounts in 2usize..=20,
    ) {
        // Generate balanced accounts (total debits = total credits)
        let mut accounts = Vec::with_capacity(num_accounts);
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;
        
        for i in 0..num_accounts {
            let debit = Decimal::from(i as i64 * 1000);
            let credit = Decimal::from((num_accounts - i - 1) as i64 * 1000);
            
            accounts.push(AccountBalance {
                account_id: uuid::Uuid::new_v4(),
                code: format!("{}", 1000 + i),
                name: format!("Account {}", i),
                account_type: if i % 2 == 0 { "asset" } else { "liability" }.to_string(),
                account_subtype: None,
                total_debit: debit,
                total_credit: credit,
                balance: debit - credit,
            });
            
            total_debit += debit;
            total_credit += credit;
        }
        
        // Adjust last account to balance
        if let Some(last) = accounts.last_mut() {
            let diff = total_debit - total_credit;
            if diff > Decimal::ZERO {
                last.total_credit += diff;
            } else {
                last.total_debit -= diff;
            }
        }
        
        let report = ReportService::generate_trial_balance(accounts);
        
        // Trial balance must be balanced
        prop_assert!(report.totals.is_balanced);
        prop_assert_eq!(report.totals.total_debit, report.totals.total_credit);
    }

    /// Feature: reports-simulation, Property 2: Balance Sheet Accounting Equation
    #[test]
    fn test_balance_sheet_equation(
        asset_balance in 0i64..1_000_000,
        liability_balance in 0i64..500_000,
    ) {
        let asset_balance = Decimal::from(asset_balance);
        let liability_balance = Decimal::from(liability_balance);
        let equity_balance = asset_balance - liability_balance;  // A = L + E
        
        let accounts = vec![
            AccountBalance {
                account_id: uuid::Uuid::new_v4(),
                code: "1000".to_string(),
                name: "Cash".to_string(),
                account_type: "asset".to_string(),
                account_subtype: Some("current_asset".to_string()),
                total_debit: asset_balance,
                total_credit: Decimal::ZERO,
                balance: asset_balance,
            },
            AccountBalance {
                account_id: uuid::Uuid::new_v4(),
                code: "2000".to_string(),
                name: "Accounts Payable".to_string(),
                account_type: "liability".to_string(),
                account_subtype: Some("current_liability".to_string()),
                total_debit: Decimal::ZERO,
                total_credit: liability_balance,
                balance: liability_balance,
            },
            AccountBalance {
                account_id: uuid::Uuid::new_v4(),
                code: "3000".to_string(),
                name: "Retained Earnings".to_string(),
                account_type: "equity".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: equity_balance,
                balance: equity_balance,
            },
        ];
        
        let report = ReportService::generate_balance_sheet(accounts);
        
        // Assets = Liabilities + Equity
        prop_assert!(report.is_balanced);
        prop_assert_eq!(report.total_assets, report.liabilities_and_equity);
    }
}
```


## Research Findings (2025-2026)

### Rayon Parallel Processing Best Practices

Based on research from 2025-2026 sources:

1. **Use `par_iter()` for CPU-bound operations**:
```rust
use rayon::prelude::*;

// Good: Parallel map-reduce for aggregations
let total: Decimal = accounts
    .par_iter()
    .map(|a| a.balance)
    .sum();

// Good: Parallel filter and collect
let projections: Vec<AccountProjection> = historical_data
    .par_iter()
    .flat_map(|account| project_account(account, params))
    .collect();
```

2. **Minimum load size threshold** - Don't parallelize small workloads:
```rust
const MIN_LOAD_SIZE: usize = 5;

if accounts.len() < MIN_LOAD_SIZE {
    // Use sequential processing for small datasets
    return accounts.iter().map(|a| process(a)).collect();
}

// Use parallel for larger datasets
accounts.par_iter().map(|a| process(a)).collect()
```

3. **Avoid mixing sync/async** - Rayon is for CPU-bound, Tokio for I/O-bound:
```rust
// DON'T: Mix blocking calls in async context
// DO: Use Rayon for pure computation, Tokio for database/network
```

### SeaORM Aggregation Patterns

Based on SeaORM 1.1 documentation:

1. **GROUP BY with aggregates**:
```rust
use sea_orm::sea_query::{Expr, Alias};

// Complex aggregation query
let results = Entity::find()
    .select_only()
    .column_as(Column::AccountId.count(), "count")
    .column_as(Column::Amount.sum(), "total_amount")
    .group_by(Column::AccountType)
    .having(Expr::col(Alias::new("count")).gt(0))
    .into_model::<AggregateResult>()
    .all(&db)
    .await?;
```

2. **Raw SQL for complex views**:
```rust
// Use database views for complex aggregations
let results: Vec<TrialBalanceRow> = TrialBalanceRow::find_by_statement(
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT * FROM trial_balance_view WHERE organization_id = $1 AND as_of_date <= $2"#,
        [org_id.into(), as_of_date.into()]
    )
)
.all(&db)
.await?;
```

### Moka Caching for Simulation Results

Based on moka crate patterns (2025):

```rust
use moka::future::Cache;
use std::time::Duration;

// Create cache with TTL and max capacity
let simulation_cache: Cache<String, SimulationResult> = Cache::builder()
    .max_capacity(100)
    .time_to_live(Duration::from_secs(300)) // 5 minutes
    .build();

// Cache simulation results by parameter hash
pub async fn run_simulation_cached(
    cache: &Cache<String, SimulationResult>,
    params: &SimulationParams,
    historical_data: Vec<HistoricalAccountData>,
) -> SimulationResult {
    let cache_key = hash_params(params);
    
    cache.get_with(cache_key.clone(), async {
        let mut result = SimulationEngine::run(historical_data, params);
        result.cached = false;
        result
    }).await
}
```

### rust_decimal Best Practices for Financial Calculations

Based on 2025 security guidelines:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// CORRECT: Use Decimal for all money calculations
fn calculate_variance(budgeted: Decimal, actual: Decimal) -> Decimal {
    budgeted - actual
}

// CORRECT: Explicit rounding with round_dp
fn calculate_percent(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator.is_zero() {
        return Decimal::ZERO;
    }
    (numerator / denominator * dec!(100)).round_dp(2)
}

// CORRECT: Use checked operations for overflow protection
fn safe_multiply(a: Decimal, b: Decimal) -> Option<Decimal> {
    a.checked_mul(b)
}

// WRONG: Never use float for money
// fn bad_calc(amount: f64) -> f64 { amount * 0.1 }
```

### Proptest Strategies for Financial Data

Based on property-based testing patterns:

```rust
use proptest::prelude::*;
use rust_decimal::Decimal;

// Strategy for generating valid Decimal amounts
fn decimal_amount() -> impl Strategy<Value = Decimal> {
    (0i64..1_000_000_000i64).prop_map(|v| Decimal::from(v))
}

// Strategy for generating valid growth rates (-50% to +100%)
fn growth_rate() -> impl Strategy<Value = Decimal> {
    (-50i32..=100i32).prop_map(|v| Decimal::from(v) / Decimal::from(100))
}

// Strategy for generating account types
fn account_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("asset".to_string()),
        Just("liability".to_string()),
        Just("equity".to_string()),
        Just("revenue".to_string()),
        Just("expense".to_string()),
    ]
}

proptest! {
    #[test]
    fn test_variance_calculation(
        budgeted in decimal_amount(),
        actual in decimal_amount(),
        account_type in account_type(),
    ) {
        let result = BudgetService::calculate_variance(budgeted, actual, &account_type);
        
        // Verify variance formula based on account type
        match account_type.as_str() {
            "expense" => prop_assert_eq!(result.variance, budgeted - actual),
            "revenue" => prop_assert_eq!(result.variance, actual - budgeted),
            _ => prop_assert_eq!(result.variance, budgeted - actual),
        }
    }
}
```

### Dependencies to Add

```toml
# Cargo.toml additions for Phase 4
[dependencies]
rayon = "1.10"
moka = { version = "0.12", features = ["future"] }

[dev-dependencies]
proptest = "1.5"
```
