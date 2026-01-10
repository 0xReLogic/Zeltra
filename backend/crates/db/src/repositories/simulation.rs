//! Simulation repository for historical data queries.
//!
//! Implements Requirements 10.1-10.5 for simulation historical data aggregation.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{
    chart_of_accounts, entry_dimensions, ledger_entries,
    sea_orm_active_enums::{AccountType, TransactionStatus},
    transactions,
};

/// Error types for simulation operations.
#[derive(Debug, thiserror::Error)]
pub enum SimulationRepoError {
    /// Invalid base period.
    #[error("Invalid base period: start {start} is after end {end}")]
    InvalidBasePeriod {
        /// Start date.
        start: NaiveDate,
        /// End date.
        end: NaiveDate,
    },

    /// No historical data found.
    #[error("No historical data found for the base period")]
    NoHistoricalData,

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Historical account data for simulation.
#[derive(Debug, Clone)]
pub struct HistoricalAccountData {
    /// Account ID.
    pub account_id: Uuid,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Account type (revenue, expense, etc.).
    pub account_type: String,
    /// Monthly amounts in the base period.
    pub monthly_amounts: Vec<Decimal>,
}

/// Simulation repository for historical data queries.
#[derive(Debug, Clone)]
pub struct SimulationRepository {
    db: DatabaseConnection,
}

impl SimulationRepository {
    /// Creates a new simulation repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Queries historical data for simulation.
    ///
    /// Requirements: 10.1, 10.2, 10.4
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Base period is invalid (start > end)
    /// - Database query fails
    pub async fn query_historical_data(
        &self,
        organization_id: Uuid,
        base_period_start: NaiveDate,
        base_period_end: NaiveDate,
        dimension_filters: &[Uuid],
    ) -> Result<Vec<HistoricalAccountData>, SimulationRepoError> {
        // Validate base period
        if base_period_start > base_period_end {
            return Err(SimulationRepoError::InvalidBasePeriod {
                start: base_period_start,
                end: base_period_end,
            });
        }

        // Get revenue and expense accounts (for simulation)
        let accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .filter(
                chart_of_accounts::Column::AccountType
                    .is_in([AccountType::Revenue, AccountType::Expense]),
            )
            .all(&self.db)
            .await?;

        if accounts.is_empty() {
            return Ok(vec![]);
        }

        // Get posted transaction IDs within base period (Requirement 10.1)
        let posted_tx_ids: Vec<Uuid> = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .filter(transactions::Column::TransactionDate.gte(base_period_start))
            .filter(transactions::Column::TransactionDate.lte(base_period_end))
            .select_only()
            .column(transactions::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await?;

        if posted_tx_ids.is_empty() {
            // Return accounts with empty monthly amounts (Requirement 10.5)
            return Ok(accounts
                .into_iter()
                .map(|a| HistoricalAccountData {
                    account_id: a.id,
                    account_code: a.code,
                    account_name: a.name,
                    account_type: account_type_to_string(&a.account_type),
                    monthly_amounts: vec![],
                })
                .collect());
        }

        // Get transactions with dates for monthly grouping
        let transactions_with_dates: Vec<(Uuid, NaiveDate)> = transactions::Entity::find()
            .filter(transactions::Column::Id.is_in(posted_tx_ids.clone()))
            .select_only()
            .column(transactions::Column::Id)
            .column(transactions::Column::TransactionDate)
            .into_tuple()
            .all(&self.db)
            .await?;

        let tx_date_map: HashMap<Uuid, NaiveDate> = transactions_with_dates.into_iter().collect();

        let mut result = Vec::with_capacity(accounts.len());

        for account in accounts {
            // Query ledger entries for this account
            let entries = ledger_entries::Entity::find()
                .filter(ledger_entries::Column::AccountId.eq(account.id))
                .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids.clone()))
                .all(&self.db)
                .await?;

            // Filter by dimensions if specified (Requirement 10.4)
            let filtered_entries = self
                .filter_entries_by_dimensions(entries, dimension_filters)
                .await?;

            // Group by month and calculate totals (Requirement 10.2)
            let mut monthly_totals: HashMap<(i32, u32), Decimal> = HashMap::new();

            for entry in filtered_entries {
                if let Some(tx_date) = tx_date_map.get(&entry.transaction_id) {
                    let year = tx_date.year();
                    let month = tx_date.month();

                    // Calculate amount based on account type
                    let amount = match account.account_type {
                        AccountType::Revenue => entry.credit - entry.debit,
                        _ => entry.debit - entry.credit,
                    };

                    *monthly_totals.entry((year, month)).or_insert(Decimal::ZERO) += amount;
                }
            }

            // Convert to sorted monthly amounts
            let mut months: Vec<(i32, u32)> = monthly_totals.keys().copied().collect();
            months.sort_unstable();

            let monthly_amounts: Vec<Decimal> = months
                .iter()
                .map(|key| monthly_totals.get(key).copied().unwrap_or(Decimal::ZERO))
                .collect();

            result.push(HistoricalAccountData {
                account_id: account.id,
                account_code: account.code,
                account_name: account.name,
                account_type: account_type_to_string(&account.account_type),
                monthly_amounts,
            });
        }

        Ok(result)
    }

    /// Filters ledger entries by dimension values.
    async fn filter_entries_by_dimensions(
        &self,
        entries: Vec<crate::entities::ledger_entries::Model>,
        dimension_filters: &[Uuid],
    ) -> Result<Vec<crate::entities::ledger_entries::Model>, SimulationRepoError> {
        if dimension_filters.is_empty() {
            return Ok(entries);
        }

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
        Ok(filtered)
    }
}

/// Converts account type enum to string.
fn account_type_to_string(account_type: &AccountType) -> String {
    match account_type {
        AccountType::Asset => "asset".to_string(),
        AccountType::Liability => "liability".to_string(),
        AccountType::Equity => "equity".to_string(),
        AccountType::Revenue => "revenue".to_string(),
        AccountType::Expense => "expense".to_string(),
    }
}
