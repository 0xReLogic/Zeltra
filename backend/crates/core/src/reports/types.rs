//! Report data types.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Account balance for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub code: String,
    /// Account name.
    pub name: String,
    /// Account type (asset, liability, equity, revenue, expense).
    pub account_type: String,
    /// Account subtype (current_asset, fixed_asset, etc.).
    pub account_subtype: Option<String>,
    /// Total debit amount.
    pub total_debit: Decimal,
    /// Total credit amount.
    pub total_credit: Decimal,
    /// Net balance.
    pub balance: Decimal,
}

/// Trial balance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceReport {
    /// Report type identifier.
    pub report_type: String,
    /// As of date.
    pub as_of: NaiveDate,
    /// Currency code.
    pub currency: String,
    /// Account balances.
    pub accounts: Vec<AccountBalance>,
    /// Totals.
    pub totals: TrialBalanceTotals,
}

/// Trial balance totals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceTotals {
    /// Total debit.
    pub total_debit: Decimal,
    /// Total credit.
    pub total_credit: Decimal,
    /// Whether debits equal credits.
    pub is_balanced: bool,
}

/// Balance sheet section (assets, liabilities, equity).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceSheetSection {
    /// Section total.
    pub total: Decimal,
    /// Accounts in this section.
    pub accounts: Vec<AccountBalance>,
    /// Subsections (current assets, fixed assets, etc.).
    pub subsections: Vec<BalanceSheetSubsection>,
}

/// Balance sheet subsection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetSubsection {
    /// Subsection name.
    pub name: String,
    /// Subsection total.
    pub total: Decimal,
    /// Accounts in this subsection.
    pub accounts: Vec<AccountBalance>,
}

/// Balance sheet report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetReport {
    /// Report type identifier.
    pub report_type: String,
    /// As of date.
    pub as_of: NaiveDate,
    /// Currency code.
    pub currency: String,
    /// Assets section.
    pub assets: BalanceSheetSection,
    /// Liabilities section.
    pub liabilities: BalanceSheetSection,
    /// Equity section.
    pub equity: BalanceSheetSection,
    /// Total assets.
    pub total_assets: Decimal,
    /// Total liabilities.
    pub total_liabilities: Decimal,
    /// Total equity.
    pub total_equity: Decimal,
    /// Liabilities plus equity.
    pub liabilities_and_equity: Decimal,
    /// Whether assets equal liabilities plus equity.
    pub is_balanced: bool,
}

/// Income statement section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncomeStatementSection {
    /// Section total.
    pub total: Decimal,
    /// Accounts in this section.
    pub accounts: Vec<AccountBalance>,
}

/// Income statement report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatementReport {
    /// Report type identifier.
    pub report_type: String,
    /// Period start date.
    pub period_start: NaiveDate,
    /// Period end date.
    pub period_end: NaiveDate,
    /// Currency code.
    pub currency: String,
    /// Revenue section.
    pub revenue: IncomeStatementSection,
    /// Cost of goods sold section.
    pub cost_of_goods_sold: IncomeStatementSection,
    /// Gross profit (revenue - COGS).
    pub gross_profit: Decimal,
    /// Operating expenses section.
    pub operating_expenses: IncomeStatementSection,
    /// Operating income (gross profit - operating expenses).
    pub operating_income: Decimal,
    /// Other income/expense section.
    pub other_income_expense: IncomeStatementSection,
    /// Net income.
    pub net_income: Decimal,
}

/// Account ledger entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedgerEntry {
    /// Entry ID.
    pub id: Uuid,
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Transaction date.
    pub transaction_date: NaiveDate,
    /// Description.
    pub description: String,
    /// Source currency.
    pub source_currency: String,
    /// Source amount.
    pub source_amount: Decimal,
    /// Exchange rate.
    pub exchange_rate: Decimal,
    /// Functional amount.
    pub functional_amount: Decimal,
    /// Debit amount.
    pub debit: Decimal,
    /// Credit amount.
    pub credit: Decimal,
    /// Running balance.
    pub running_balance: Decimal,
    /// Dimension values.
    pub dimensions: Vec<DimensionInfo>,
}

/// Dimension information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionInfo {
    /// Dimension type.
    pub dimension_type: String,
    /// Dimension code.
    pub code: String,
    /// Dimension name.
    pub name: String,
}

/// Dimensional report row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalReportRow {
    /// Dimension values for this row.
    pub dimensions: Vec<DimensionInfo>,
    /// Account balances.
    pub accounts: Vec<AccountBalance>,
    /// Row total.
    pub total: Decimal,
}

/// Dimensional report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalReport {
    /// Report type identifier.
    pub report_type: String,
    /// Period start date.
    pub period_start: NaiveDate,
    /// Period end date.
    pub period_end: NaiveDate,
    /// Group by dimensions.
    pub group_by: Vec<String>,
    /// Report data rows.
    pub data: Vec<DimensionalReportRow>,
    /// Grand total.
    pub grand_total: Decimal,
}
