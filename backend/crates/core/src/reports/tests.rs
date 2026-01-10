//! Property-based tests for reports module.

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use super::service::ReportService;
use super::types::AccountBalance;

proptest! {
    /// Feature: reports-simulation, Property 1: Trial Balance Debits Equal Credits
    /// For any valid ledger with posted transactions, the sum of all debit balances
    /// SHALL equal the sum of all credit balances in the trial balance report.
    #[test]
    fn test_trial_balance_totals_calculation(
        num_accounts in 1usize..20,
    ) {
        // Generate accounts with random debits and credits
        let accounts: Vec<AccountBalance> = (0..num_accounts)
            .map(|i| {
                let debit = Decimal::from(i as i64 * 1000 + 500);
                let credit = Decimal::from(i as i64 * 800 + 300);
                AccountBalance {
                    account_id: Uuid::new_v4(),
                    code: format!("{}", 1000 + i),
                    name: format!("Account {}", i),
                    account_type: if i % 2 == 0 { "asset" } else { "liability" }.to_string(),
                    account_subtype: None,
                    total_debit: debit,
                    total_credit: credit,
                    balance: debit - credit,
                }
            })
            .collect();

        let expected_total_debit: Decimal = accounts.iter().map(|a| a.total_debit).sum();
        let expected_total_credit: Decimal = accounts.iter().map(|a| a.total_credit).sum();

        let report = ReportService::generate_trial_balance(accounts);

        // Verify totals are correctly calculated
        prop_assert_eq!(report.totals.total_debit, expected_total_debit);
        prop_assert_eq!(report.totals.total_credit, expected_total_credit);

        // Verify is_balanced flag is correct
        prop_assert_eq!(
            report.totals.is_balanced,
            expected_total_debit == expected_total_credit
        );
    }

    /// Feature: reports-simulation, Property 1: Trial Balance Debits Equal Credits
    /// When we create balanced accounts, the trial balance should be balanced.
    #[test]
    fn test_trial_balance_balanced_when_debits_equal_credits(
        num_accounts in 2usize..20,
    ) {
        // Generate balanced accounts (total debits = total credits)
        let mut accounts = Vec::with_capacity(num_accounts);
        let mut running_debit = Decimal::ZERO;
        let mut running_credit = Decimal::ZERO;

        for i in 0..num_accounts - 1 {
            let debit = Decimal::from(i as i64 * 1000 + 500);
            let credit = Decimal::from(i as i64 * 800 + 300);
            running_debit += debit;
            running_credit += credit;

            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("{}", 1000 + i),
                name: format!("Account {}", i),
                account_type: if i % 2 == 0 { "asset" } else { "liability" }.to_string(),
                account_subtype: None,
                total_debit: debit,
                total_credit: credit,
                balance: debit - credit,
            });
        }

        // Add balancing account
        let diff = running_debit - running_credit;
        let (final_debit, final_credit) = if diff > Decimal::ZERO {
            (Decimal::ZERO, diff)
        } else {
            (-diff, Decimal::ZERO)
        };

        accounts.push(AccountBalance {
            account_id: Uuid::new_v4(),
            code: "9999".to_string(),
            name: "Balancing Account".to_string(),
            account_type: "equity".to_string(),
            account_subtype: None,
            total_debit: final_debit,
            total_credit: final_credit,
            balance: final_debit - final_credit,
        });

        let report = ReportService::generate_trial_balance(accounts);

        // Trial balance must be balanced
        prop_assert!(report.totals.is_balanced);
        prop_assert_eq!(report.totals.total_debit, report.totals.total_credit);
    }

    /// Feature: reports-simulation, Property 2: Balance Sheet Accounting Equation
    /// For any valid ledger, Assets = Liabilities + Equity.
    #[test]
    fn test_balance_sheet_equation(
        asset_balance in 0i64..1_000_000_000,
        liability_balance in 0i64..500_000_000,
    ) {
        let asset_balance = Decimal::from(asset_balance);
        let liability_balance = Decimal::from(liability_balance);
        let equity_balance = asset_balance - liability_balance; // A = L + E

        let accounts = vec![
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "1000".to_string(),
                name: "Cash".to_string(),
                account_type: "asset".to_string(),
                account_subtype: Some("current_asset".to_string()),
                total_debit: asset_balance,
                total_credit: Decimal::ZERO,
                balance: asset_balance,
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "2000".to_string(),
                name: "Accounts Payable".to_string(),
                account_type: "liability".to_string(),
                account_subtype: Some("current_liability".to_string()),
                total_debit: Decimal::ZERO,
                total_credit: liability_balance,
                balance: liability_balance,
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
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
        prop_assert_eq!(report.total_assets, report.total_liabilities + report.total_equity);
    }

    /// Feature: reports-simulation, Property 2: Balance Sheet Accounting Equation
    /// Section totals should equal sum of account balances.
    #[test]
    fn test_balance_sheet_section_totals(
        num_assets in 1usize..10,
        num_liabilities in 1usize..10,
        num_equity in 1usize..5,
    ) {
        let mut accounts = Vec::new();
        let mut expected_assets = Decimal::ZERO;
        let mut expected_liabilities = Decimal::ZERO;
        let mut expected_equity = Decimal::ZERO;

        for i in 0..num_assets {
            let balance = Decimal::from(i as i64 * 1000 + 100);
            expected_assets += balance;
            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("1{:03}", i),
                name: format!("Asset {}", i),
                account_type: "asset".to_string(),
                account_subtype: None,
                total_debit: balance,
                total_credit: Decimal::ZERO,
                balance,
            });
        }

        for i in 0..num_liabilities {
            let balance = Decimal::from(i as i64 * 500 + 50);
            expected_liabilities += balance;
            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("2{:03}", i),
                name: format!("Liability {}", i),
                account_type: "liability".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: balance,
                balance,
            });
        }

        for i in 0..num_equity {
            let balance = Decimal::from(i as i64 * 200 + 20);
            expected_equity += balance;
            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("3{:03}", i),
                name: format!("Equity {}", i),
                account_type: "equity".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: balance,
                balance,
            });
        }

        let report = ReportService::generate_balance_sheet(accounts);

        prop_assert_eq!(report.total_assets, expected_assets);
        prop_assert_eq!(report.total_liabilities, expected_liabilities);
        prop_assert_eq!(report.total_equity, expected_equity);
    }

    /// Feature: reports-simulation, Property 3: Income Statement Net Income Calculation
    /// Net Income = Revenue - COGS - Operating Expenses - Other Expenses
    #[test]
    fn test_income_statement_net_income(
        revenue_amount in 0i64..1_000_000_000,
        cogs_amount in 0i64..500_000_000,
        opex_amount in 0i64..300_000_000,
        other_amount in 0i64..100_000_000,
    ) {
        let revenue = Decimal::from(revenue_amount);
        let cogs = Decimal::from(cogs_amount);
        let opex = Decimal::from(opex_amount);
        let other = Decimal::from(other_amount);

        let accounts = vec![
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "4000".to_string(),
                name: "Sales Revenue".to_string(),
                account_type: "revenue".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: revenue,
                balance: revenue,
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "5000".to_string(),
                name: "Cost of Goods Sold".to_string(),
                account_type: "expense".to_string(),
                account_subtype: Some("cost_of_goods_sold".to_string()),
                total_debit: cogs,
                total_credit: Decimal::ZERO,
                balance: cogs,
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "6000".to_string(),
                name: "Operating Expenses".to_string(),
                account_type: "expense".to_string(),
                account_subtype: Some("operating_expense".to_string()),
                total_debit: opex,
                total_credit: Decimal::ZERO,
                balance: opex,
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "7000".to_string(),
                name: "Other Expenses".to_string(),
                account_type: "expense".to_string(),
                account_subtype: Some("other_expense".to_string()),
                total_debit: other,
                total_credit: Decimal::ZERO,
                balance: other,
            },
        ];

        let report = ReportService::generate_income_statement(accounts);

        // Verify calculations
        let expected_gross_profit = revenue - cogs;
        let expected_operating_income = expected_gross_profit - opex;
        let expected_net_income = expected_operating_income - other;

        prop_assert_eq!(report.gross_profit, expected_gross_profit);
        prop_assert_eq!(report.operating_income, expected_operating_income);
        prop_assert_eq!(report.net_income, expected_net_income);
    }

    /// Feature: reports-simulation, Property 3: Income Statement Net Income Calculation
    /// Section totals should equal sum of account balances.
    #[test]
    fn test_income_statement_section_totals(
        num_revenue in 1usize..5,
        num_cogs in 1usize..3,
        num_opex in 1usize..5,
    ) {
        let mut accounts = Vec::new();
        let mut expected_revenue = Decimal::ZERO;
        let mut expected_cogs = Decimal::ZERO;
        let mut expected_opex = Decimal::ZERO;

        for i in 0..num_revenue {
            let balance = Decimal::from(i as i64 * 10000 + 5000);
            expected_revenue += balance;
            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("4{:03}", i),
                name: format!("Revenue {}", i),
                account_type: "revenue".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: balance,
                balance,
            });
        }

        for i in 0..num_cogs {
            let balance = Decimal::from(i as i64 * 3000 + 1000);
            expected_cogs += balance;
            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("5{:03}", i),
                name: format!("COGS {}", i),
                account_type: "expense".to_string(),
                account_subtype: Some("cost_of_goods_sold".to_string()),
                total_debit: balance,
                total_credit: Decimal::ZERO,
                balance,
            });
        }

        for i in 0..num_opex {
            let balance = Decimal::from(i as i64 * 2000 + 500);
            expected_opex += balance;
            accounts.push(AccountBalance {
                account_id: Uuid::new_v4(),
                code: format!("6{:03}", i),
                name: format!("OpEx {}", i),
                account_type: "expense".to_string(),
                account_subtype: Some("operating_expense".to_string()),
                total_debit: balance,
                total_credit: Decimal::ZERO,
                balance,
            });
        }

        let report = ReportService::generate_income_statement(accounts);

        prop_assert_eq!(report.revenue.total, expected_revenue);
        prop_assert_eq!(report.cost_of_goods_sold.total, expected_cogs);
        prop_assert_eq!(report.operating_expenses.total, expected_opex);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_trial_balance_empty_accounts() {
        let report = ReportService::generate_trial_balance(vec![]);

        assert_eq!(report.totals.total_debit, dec!(0));
        assert_eq!(report.totals.total_credit, dec!(0));
        assert!(report.totals.is_balanced);
    }

    #[test]
    fn test_balance_sheet_empty_accounts() {
        let report = ReportService::generate_balance_sheet(vec![]);

        assert_eq!(report.total_assets, dec!(0));
        assert_eq!(report.total_liabilities, dec!(0));
        assert_eq!(report.total_equity, dec!(0));
        assert!(report.is_balanced);
    }

    #[test]
    fn test_income_statement_empty_accounts() {
        let report = ReportService::generate_income_statement(vec![]);

        assert_eq!(report.revenue.total, dec!(0));
        assert_eq!(report.cost_of_goods_sold.total, dec!(0));
        assert_eq!(report.gross_profit, dec!(0));
        assert_eq!(report.operating_income, dec!(0));
        assert_eq!(report.net_income, dec!(0));
    }

    #[test]
    fn test_balance_sheet_ignores_revenue_expense() {
        let accounts = vec![
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "4000".to_string(),
                name: "Revenue".to_string(),
                account_type: "revenue".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: dec!(10000),
                balance: dec!(10000),
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "5000".to_string(),
                name: "Expense".to_string(),
                account_type: "expense".to_string(),
                account_subtype: None,
                total_debit: dec!(5000),
                total_credit: Decimal::ZERO,
                balance: dec!(5000),
            },
        ];

        let report = ReportService::generate_balance_sheet(accounts);

        // Revenue and expense should not appear in balance sheet
        assert_eq!(report.total_assets, dec!(0));
        assert_eq!(report.total_liabilities, dec!(0));
        assert_eq!(report.total_equity, dec!(0));
    }

    #[test]
    fn test_income_statement_ignores_balance_sheet_accounts() {
        let accounts = vec![
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "1000".to_string(),
                name: "Cash".to_string(),
                account_type: "asset".to_string(),
                account_subtype: None,
                total_debit: dec!(10000),
                total_credit: Decimal::ZERO,
                balance: dec!(10000),
            },
            AccountBalance {
                account_id: Uuid::new_v4(),
                code: "2000".to_string(),
                name: "Payable".to_string(),
                account_type: "liability".to_string(),
                account_subtype: None,
                total_debit: Decimal::ZERO,
                total_credit: dec!(5000),
                balance: dec!(5000),
            },
        ];

        let report = ReportService::generate_income_statement(accounts);

        // Asset and liability should not appear in income statement
        assert_eq!(report.revenue.total, dec!(0));
        assert_eq!(report.cost_of_goods_sold.total, dec!(0));
        assert_eq!(report.net_income, dec!(0));
    }
}
