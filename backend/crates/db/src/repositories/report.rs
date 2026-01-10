//! Report repository for financial report database operations.
//!
//! Implements Requirements 5.1-5.7, 6.1-6.7, 7.1-7.8, 8.1-8.6, 9.1-9.7 for report generation.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use uuid::Uuid;

use crate::entities::{
    chart_of_accounts, dimension_types, dimension_values, entry_dimensions, ledger_entries,
    sea_orm_active_enums::{AccountSubtype, AccountType, TransactionStatus},
    transactions,
};

/// Error types for report operations.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Invalid date range.
    #[error("Invalid date range: start {start} is after end {end}")]
    InvalidDateRange {
        /// Start date.
        start: NaiveDate,
        /// End date.
        end: NaiveDate,
    },

    /// No data found.
    #[error("No data found for the specified criteria")]
    NoDataFound,

    /// Invalid dimension type.
    #[error("Invalid dimension type: {0}")]
    InvalidDimensionType(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Account balance for reports.
#[derive(Debug, Clone)]
pub struct AccountBalance {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub code: String,
    /// Account name.
    pub name: String,
    /// Account type.
    pub account_type: AccountType,
    /// Account subtype.
    pub account_subtype: Option<AccountSubtype>,
    /// Total debit amount.
    pub total_debit: Decimal,
    /// Total credit amount.
    pub total_credit: Decimal,
    /// Net balance.
    pub balance: Decimal,
}

/// Account ledger entry for detailed reports.
#[derive(Debug, Clone)]
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
    /// Running balance (from account_current_balance).
    pub running_balance: Decimal,
    /// Dimension values.
    pub dimensions: Vec<DimensionInfo>,
}

/// Dimension information for reports.
#[derive(Debug, Clone)]
pub struct DimensionInfo {
    /// Dimension type name.
    pub dimension_type: String,
    /// Dimension value code.
    pub code: String,
    /// Dimension value name.
    pub name: String,
}

/// Dimensional report row.
#[derive(Debug, Clone)]
pub struct DimensionalReportRow {
    /// Dimension values for this row.
    pub dimensions: Vec<DimensionInfo>,
    /// Total debit.
    pub total_debit: Decimal,
    /// Total credit.
    pub total_credit: Decimal,
    /// Net balance.
    pub balance: Decimal,
}

/// Report repository for financial report queries.
#[derive(Debug, Clone)]
pub struct ReportRepository {
    db: DatabaseConnection,
}

impl ReportRepository {
    /// Creates a new report repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ========================================================================
    // Trial Balance Query (Requirements 5.1-5.7)
    // ========================================================================

    /// Queries account balances for trial balance report.
    ///
    /// Requirements: 5.1, 5.2, 5.6, 5.7
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_trial_balance(
        &self,
        organization_id: Uuid,
        as_of: NaiveDate,
        dimension_filters: &[Uuid],
    ) -> Result<Vec<AccountBalance>, ReportError> {
        // Get all accounts for the organization
        let accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .order_by_asc(chart_of_accounts::Column::Code)
            .all(&self.db)
            .await?;

        // Get posted transaction IDs up to as_of date
        let posted_tx_ids = self
            .get_posted_transaction_ids(organization_id, None, Some(as_of))
            .await?;

        if posted_tx_ids.is_empty() {
            // Return accounts with zero balances
            return Ok(accounts
                .into_iter()
                .map(|a| AccountBalance {
                    account_id: a.id,
                    code: a.code,
                    name: a.name,
                    account_type: a.account_type,
                    account_subtype: a.account_subtype,
                    total_debit: Decimal::ZERO,
                    total_credit: Decimal::ZERO,
                    balance: Decimal::ZERO,
                })
                .collect());
        }

        let mut result = Vec::with_capacity(accounts.len());

        for account in accounts {
            let (total_debit, total_credit) = self
                .calculate_account_totals(account.id, &posted_tx_ids, dimension_filters)
                .await?;

            // Calculate balance based on account type (Requirement 5.3)
            let balance = calculate_balance(&account.account_type, total_debit, total_credit);

            result.push(AccountBalance {
                account_id: account.id,
                code: account.code,
                name: account.name,
                account_type: account.account_type,
                account_subtype: account.account_subtype,
                total_debit,
                total_credit,
                balance,
            });
        }

        Ok(result)
    }

    // ========================================================================
    // Balance Sheet Query (Requirements 6.1-6.7)
    // ========================================================================

    /// Queries account balances for balance sheet report.
    ///
    /// Requirements: 6.1, 6.3, 6.4
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_balance_sheet(
        &self,
        organization_id: Uuid,
        as_of: NaiveDate,
    ) -> Result<Vec<AccountBalance>, ReportError> {
        // Get balance sheet accounts (Asset, Liability, Equity)
        let accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .filter(chart_of_accounts::Column::AccountType.is_in([
                AccountType::Asset,
                AccountType::Liability,
                AccountType::Equity,
            ]))
            .order_by_asc(chart_of_accounts::Column::Code)
            .all(&self.db)
            .await?;

        // Get posted transaction IDs up to as_of date
        let posted_tx_ids = self
            .get_posted_transaction_ids(organization_id, None, Some(as_of))
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok(accounts
                .into_iter()
                .map(|a| AccountBalance {
                    account_id: a.id,
                    code: a.code,
                    name: a.name,
                    account_type: a.account_type,
                    account_subtype: a.account_subtype,
                    total_debit: Decimal::ZERO,
                    total_credit: Decimal::ZERO,
                    balance: Decimal::ZERO,
                })
                .collect());
        }

        let mut result = Vec::with_capacity(accounts.len());

        for account in accounts {
            let (total_debit, total_credit) = self
                .calculate_account_totals(account.id, &posted_tx_ids, &[])
                .await?;

            let balance = calculate_balance(&account.account_type, total_debit, total_credit);

            result.push(AccountBalance {
                account_id: account.id,
                code: account.code,
                name: account.name,
                account_type: account.account_type,
                account_subtype: account.account_subtype,
                total_debit,
                total_credit,
                balance,
            });
        }

        Ok(result)
    }

    // ========================================================================
    // Income Statement Query (Requirements 7.1-7.8)
    // ========================================================================

    /// Queries account balances for income statement report.
    ///
    /// Requirements: 7.1, 7.2
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_income_statement(
        &self,
        organization_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        dimension_filters: &[Uuid],
    ) -> Result<Vec<AccountBalance>, ReportError> {
        // Validate date range
        if from > to {
            return Err(ReportError::InvalidDateRange {
                start: from,
                end: to,
            });
        }

        // Get income statement accounts (Revenue, Expense)
        let accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .filter(
                chart_of_accounts::Column::AccountType
                    .is_in([AccountType::Revenue, AccountType::Expense]),
            )
            .order_by_asc(chart_of_accounts::Column::Code)
            .all(&self.db)
            .await?;

        // Get posted transaction IDs within date range
        let posted_tx_ids = self
            .get_posted_transaction_ids(organization_id, Some(from), Some(to))
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok(accounts
                .into_iter()
                .map(|a| AccountBalance {
                    account_id: a.id,
                    code: a.code,
                    name: a.name,
                    account_type: a.account_type,
                    account_subtype: a.account_subtype,
                    total_debit: Decimal::ZERO,
                    total_credit: Decimal::ZERO,
                    balance: Decimal::ZERO,
                })
                .collect());
        }

        let mut result = Vec::with_capacity(accounts.len());

        for account in accounts {
            let (total_debit, total_credit) = self
                .calculate_account_totals(account.id, &posted_tx_ids, dimension_filters)
                .await?;

            let balance = calculate_balance(&account.account_type, total_debit, total_credit);

            result.push(AccountBalance {
                account_id: account.id,
                code: account.code,
                name: account.name,
                account_type: account.account_type,
                account_subtype: account.account_subtype,
                total_debit,
                total_credit,
                balance,
            });
        }

        Ok(result)
    }

    // ========================================================================
    // Account Ledger Query (Requirements 8.1-8.6)
    // ========================================================================

    /// Queries ledger entries for a specific account.
    ///
    /// Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6
    ///
    /// # Errors
    ///
    /// Returns an error if the account is not found or database query fails.
    pub async fn query_account_ledger(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<AccountLedgerEntry>, u64), ReportError> {
        // Validate date range
        if from > to {
            return Err(ReportError::InvalidDateRange {
                start: from,
                end: to,
            });
        }

        // Verify account exists
        let _account = chart_of_accounts::Entity::find_by_id(account_id)
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(ReportError::AccountNotFound(account_id))?;

        // Get posted transaction IDs within date range
        let posted_tx_ids = self
            .get_posted_transaction_ids(organization_id, Some(from), Some(to))
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok((vec![], 0));
        }

        // Count total entries for pagination
        let total_count = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids.clone()))
            .count(&self.db)
            .await?;

        // Query entries with pagination (Requirement 8.5)
        // Order by transaction date and entry creation (Requirement 8.6)
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids))
            .order_by_asc(ledger_entries::Column::CreatedAt)
            .offset(page * limit)
            .limit(limit)
            .all(&self.db)
            .await?;

        // Get transaction details and dimensions for each entry
        let mut result = Vec::with_capacity(entries.len());

        for entry in entries {
            // Get transaction for date and description
            let transaction = transactions::Entity::find_by_id(entry.transaction_id)
                .one(&self.db)
                .await?;

            let (transaction_date, description) = transaction
                .map(|t| (t.transaction_date, t.description))
                .unwrap_or((from, String::new()));

            // Get dimensions for this entry (Requirement 8.4)
            let dimensions = self.get_entry_dimensions(entry.id).await?;

            result.push(AccountLedgerEntry {
                id: entry.id,
                transaction_id: entry.transaction_id,
                transaction_date,
                description,
                source_currency: entry.source_currency,
                source_amount: entry.source_amount,
                exchange_rate: entry.exchange_rate,
                functional_amount: entry.functional_amount,
                debit: entry.debit,
                credit: entry.credit,
                // Running balance from account_current_balance (Requirement 8.3)
                running_balance: entry.account_current_balance,
                dimensions,
            });
        }

        Ok((result, total_count))
    }

    // ========================================================================
    // Dimensional Report Query (Requirements 9.1-9.7)
    // ========================================================================

    /// Queries ledger entries grouped by dimensions.
    ///
    /// Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_dimensional_report(
        &self,
        organization_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        group_by: &[String],
        account_type_filter: Option<AccountType>,
        dimension_filters: &[Uuid],
    ) -> Result<(Vec<DimensionalReportRow>, Decimal), ReportError> {
        // Validate date range
        if from > to {
            return Err(ReportError::InvalidDateRange {
                start: from,
                end: to,
            });
        }

        if group_by.is_empty() {
            return Err(ReportError::InvalidDimensionType(
                "At least one group_by dimension is required".to_string(),
            ));
        }

        // Validate dimension types exist
        for dim_type in group_by {
            let exists = dimension_types::Entity::find()
                .filter(dimension_types::Column::OrganizationId.eq(organization_id))
                .filter(dimension_types::Column::Name.eq(dim_type))
                .one(&self.db)
                .await?;

            if exists.is_none() {
                return Err(ReportError::InvalidDimensionType(dim_type.clone()));
            }
        }

        // Get posted transaction IDs within date range
        let posted_tx_ids = self
            .get_posted_transaction_ids(organization_id, Some(from), Some(to))
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok((vec![], Decimal::ZERO));
        }

        // Get accounts filtered by type if specified
        let mut account_query = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true));

        if let Some(acc_type) = account_type_filter {
            account_query =
                account_query.filter(chart_of_accounts::Column::AccountType.eq(acc_type));
        }

        let accounts = account_query.all(&self.db).await?;
        let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

        if account_ids.is_empty() {
            return Ok((vec![], Decimal::ZERO));
        }

        // Query all relevant ledger entries
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.is_in(account_ids))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids))
            .all(&self.db)
            .await?;

        // Group entries by dimension combinations
        let mut dimension_groups: std::collections::HashMap<Vec<Uuid>, (Decimal, Decimal)> =
            std::collections::HashMap::new();

        for entry in entries {
            // Get dimensions for this entry
            let entry_dims = entry_dimensions::Entity::find()
                .filter(entry_dimensions::Column::LedgerEntryId.eq(entry.id))
                .all(&self.db)
                .await?;

            // Filter by dimension filters if specified (Requirement 9.5)
            if !dimension_filters.is_empty() {
                let entry_dim_ids: Vec<Uuid> =
                    entry_dims.iter().map(|d| d.dimension_value_id).collect();
                let has_all = dimension_filters.iter().all(|f| entry_dim_ids.contains(f));
                if !has_all {
                    continue;
                }
            }

            // Get dimension values that match the group_by types
            let mut group_key = Vec::new();
            for dim in &entry_dims {
                let dim_value = dimension_values::Entity::find_by_id(dim.dimension_value_id)
                    .one(&self.db)
                    .await?;

                if let Some(dv) = dim_value {
                    let dim_type = dimension_types::Entity::find_by_id(dv.dimension_type_id)
                        .one(&self.db)
                        .await?;

                    if let Some(dt) = dim_type
                        && group_by.contains(&dt.name)
                    {
                        group_key.push(dim.dimension_value_id);
                    }
                }
            }

            // Sort key for consistent grouping
            group_key.sort();

            // Aggregate totals
            let totals = dimension_groups
                .entry(group_key)
                .or_insert((Decimal::ZERO, Decimal::ZERO));
            totals.0 += entry.debit;
            totals.1 += entry.credit;
        }

        // Build result rows using helper
        self.build_dimensional_rows(dimension_groups).await
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Gets posted transaction IDs for an organization within a date range.
    async fn get_posted_transaction_ids(
        &self,
        organization_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<Uuid>, ReportError> {
        let mut query = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted));

        if let Some(from_date) = from {
            query = query.filter(transactions::Column::TransactionDate.gte(from_date));
        }

        if let Some(to_date) = to {
            query = query.filter(transactions::Column::TransactionDate.lte(to_date));
        }

        let tx_ids: Vec<Uuid> = query
            .select_only()
            .column(transactions::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await?;

        Ok(tx_ids)
    }

    /// Calculates total debit and credit for an account from ledger entries.
    async fn calculate_account_totals(
        &self,
        account_id: Uuid,
        transaction_ids: &[Uuid],
        dimension_filters: &[Uuid],
    ) -> Result<(Decimal, Decimal), ReportError> {
        if transaction_ids.is_empty() {
            return Ok((Decimal::ZERO, Decimal::ZERO));
        }

        // Query ledger entries for this account
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .filter(ledger_entries::Column::TransactionId.is_in(transaction_ids.to_vec()))
            .all(&self.db)
            .await?;

        // Filter by dimensions if specified (Requirement 5.6)
        let filtered_entries = if dimension_filters.is_empty() {
            entries
        } else {
            let mut filtered = Vec::new();
            for entry in entries {
                let entry_dims: Vec<Uuid> = entry_dimensions::Entity::find()
                    .filter(entry_dimensions::Column::LedgerEntryId.eq(entry.id))
                    .all(&self.db)
                    .await?
                    .into_iter()
                    .map(|d| d.dimension_value_id)
                    .collect();

                let has_all = dimension_filters.iter().all(|f| entry_dims.contains(f));
                if has_all {
                    filtered.push(entry);
                }
            }
            filtered
        };

        let total_debit: Decimal = filtered_entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = filtered_entries.iter().map(|e| e.credit).sum();

        Ok((total_debit, total_credit))
    }

    /// Gets dimension information for a ledger entry.
    async fn get_entry_dimensions(
        &self,
        entry_id: Uuid,
    ) -> Result<Vec<DimensionInfo>, ReportError> {
        let dims = entry_dimensions::Entity::find()
            .filter(entry_dimensions::Column::LedgerEntryId.eq(entry_id))
            .all(&self.db)
            .await?;

        let mut result = Vec::with_capacity(dims.len());

        for dim in dims {
            let dim_value = dimension_values::Entity::find_by_id(dim.dimension_value_id)
                .one(&self.db)
                .await?;

            if let Some(dv) = dim_value {
                let dim_type = dimension_types::Entity::find_by_id(dv.dimension_type_id)
                    .one(&self.db)
                    .await?;

                result.push(DimensionInfo {
                    dimension_type: dim_type.map(|t| t.name).unwrap_or_default(),
                    code: dv.code,
                    name: dv.name,
                });
            }
        }

        Ok(result)
    }

    /// Builds dimensional report rows from grouped dimension data.
    async fn build_dimensional_rows(
        &self,
        dimension_groups: std::collections::HashMap<Vec<Uuid>, (Decimal, Decimal)>,
    ) -> Result<(Vec<DimensionalReportRow>, Decimal), ReportError> {
        let mut result = Vec::with_capacity(dimension_groups.len());
        let mut grand_total = Decimal::ZERO;

        for (dim_ids, (total_debit, total_credit)) in dimension_groups {
            let mut dimensions = Vec::new();
            for dim_id in &dim_ids {
                let dim_value = dimension_values::Entity::find_by_id(*dim_id)
                    .one(&self.db)
                    .await?;

                if let Some(dv) = dim_value {
                    let dim_type = dimension_types::Entity::find_by_id(dv.dimension_type_id)
                        .one(&self.db)
                        .await?;

                    dimensions.push(DimensionInfo {
                        dimension_type: dim_type.map(|t| t.name).unwrap_or_default(),
                        code: dv.code,
                        name: dv.name,
                    });
                }
            }

            let balance = total_debit - total_credit;
            grand_total += balance;

            result.push(DimensionalReportRow {
                dimensions,
                total_debit,
                total_credit,
                balance,
            });
        }

        Ok((result, grand_total))
    }
}

// ============================================================================
// Balance Calculation Helper
// ============================================================================

/// Calculates balance based on account type.
///
/// - Asset/Expense (debit-normal): balance = debit - credit
/// - Liability/Equity/Revenue (credit-normal): balance = credit - debit
#[must_use]
pub fn calculate_balance(
    account_type: &AccountType,
    total_debit: Decimal,
    total_credit: Decimal,
) -> Decimal {
    match account_type {
        AccountType::Asset | AccountType::Expense => total_debit - total_credit,
        AccountType::Liability | AccountType::Equity | AccountType::Revenue => {
            total_credit - total_debit
        }
    }
}

/// Determines if an account type is debit-normal.
#[must_use]
pub fn is_debit_normal(account_type: &AccountType) -> bool {
    matches!(account_type, AccountType::Asset | AccountType::Expense)
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
