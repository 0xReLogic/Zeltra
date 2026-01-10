//! Report generation service.

use rust_decimal::Decimal;

use super::types::{
    AccountBalance, BalanceSheetReport, BalanceSheetSection, IncomeStatementReport,
    IncomeStatementSection, TrialBalanceReport, TrialBalanceTotals,
};

/// Service for generating financial reports.
pub struct ReportService;

impl ReportService {
    /// Generates a trial balance report from account balances.
    ///
    /// The trial balance verifies that total debits equal total credits.
    #[must_use]
    pub fn generate_trial_balance(accounts: Vec<AccountBalance>) -> TrialBalanceReport {
        let total_debit: Decimal = accounts.iter().map(|a| a.total_debit).sum();
        let total_credit: Decimal = accounts.iter().map(|a| a.total_credit).sum();

        TrialBalanceReport {
            report_type: "trial_balance".to_string(),
            as_of: chrono::Utc::now().date_naive(),
            currency: "USD".to_string(),
            accounts,
            totals: TrialBalanceTotals {
                total_debit,
                total_credit,
                is_balanced: total_debit == total_credit,
            },
        }
    }

    /// Generates a balance sheet report from account balances.
    ///
    /// The balance sheet verifies that Assets = Liabilities + Equity.
    #[must_use]
    pub fn generate_balance_sheet(accounts: Vec<AccountBalance>) -> BalanceSheetReport {
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
        let liabilities_and_equity = total_liabilities + total_equity;

        BalanceSheetReport {
            report_type: "balance_sheet".to_string(),
            as_of: chrono::Utc::now().date_naive(),
            currency: "USD".to_string(),
            assets,
            liabilities,
            equity,
            total_assets,
            total_liabilities,
            total_equity,
            liabilities_and_equity,
            is_balanced: total_assets == liabilities_and_equity,
        }
    }

    /// Generates an income statement report from account balances.
    ///
    /// Calculates gross profit, operating income, and net income.
    #[must_use]
    pub fn generate_income_statement(accounts: Vec<AccountBalance>) -> IncomeStatementReport {
        let mut revenue = IncomeStatementSection::default();
        let mut cogs = IncomeStatementSection::default();
        let mut operating_expenses = IncomeStatementSection::default();
        let mut other = IncomeStatementSection::default();

        for account in accounts {
            match (
                account.account_type.as_str(),
                account.account_subtype.as_deref(),
            ) {
                ("revenue", _) => Self::add_to_income_section(&mut revenue, account),
                ("expense", Some("cost_of_goods_sold")) => {
                    Self::add_to_income_section(&mut cogs, account);
                }
                ("expense", Some("operating_expense")) => {
                    Self::add_to_income_section(&mut operating_expenses, account);
                }
                ("expense", _) => Self::add_to_income_section(&mut other, account),
                _ => {}
            }
        }

        let gross_profit = revenue.total - cogs.total;
        let operating_income = gross_profit - operating_expenses.total;
        let net_income = operating_income - other.total;

        IncomeStatementReport {
            report_type: "income_statement".to_string(),
            period_start: chrono::Utc::now().date_naive(),
            period_end: chrono::Utc::now().date_naive(),
            currency: "USD".to_string(),
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
