//! Dashboard repository for metrics and activity queries.
//!
//! Implements Requirements 16.1-16.8, 17.1-17.6 for dashboard data.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::entities::{
    budget_lines, budgets, chart_of_accounts, dimension_types, dimension_values, entry_dimensions,
    ledger_entries, organizations,
    sea_orm_active_enums::{AccountSubtype, AccountType, TransactionStatus},
    transactions, users,
};

/// Error types for dashboard operations.
#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    /// Organization not found.
    #[error("Organization not found: {0}")]
    OrganizationNotFound(Uuid),

    /// Fiscal period not found.
    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Cash position metrics.
#[derive(Debug, Clone)]
pub struct CashPosition {
    /// Current cash balance.
    pub balance: Decimal,
    /// Currency code.
    pub currency: String,
    /// Change from last period.
    pub change_from_last_period: Decimal,
    /// Change percentage.
    pub change_percent: Decimal,
}

/// Burn rate metrics.
#[derive(Debug, Clone)]
pub struct BurnRate {
    /// Daily burn rate.
    pub daily: Decimal,
    /// Monthly burn rate.
    pub monthly: Decimal,
}

/// Pending approvals metrics.
#[derive(Debug, Clone)]
pub struct PendingApprovals {
    /// Number of pending transactions.
    pub count: i32,
    /// Total amount pending.
    pub total_amount: Decimal,
}

/// Budget status metrics.
#[derive(Debug, Clone)]
pub struct BudgetStatus {
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
    /// Total spent amount.
    pub total_spent: Decimal,
    /// Utilization percentage.
    pub utilization_percent: Decimal,
}

/// Department expense breakdown.
#[derive(Debug, Clone)]
pub struct DepartmentExpense {
    /// Department name.
    pub department: String,
    /// Total expense amount.
    pub amount: Decimal,
    /// Percentage of total.
    pub percent: Decimal,
}

/// Currency exposure breakdown.
#[derive(Debug, Clone)]
pub struct CurrencyExposure {
    /// Currency code.
    pub currency: String,
    /// Balance in source currency.
    pub balance: Decimal,
    /// Balance in functional currency.
    pub functional_value: Decimal,
    /// Percentage of total.
    pub percent: Decimal,
}

/// Activity event for recent activity feed.
#[derive(Debug, Clone)]
pub struct ActivityEvent {
    /// Event ID.
    pub id: Uuid,
    /// Event type (transaction, budget).
    pub event_type: String,
    /// Action (created, submitted, approved, etc.).
    pub action: String,
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: Uuid,
    /// Description.
    pub description: String,
    /// Amount (if applicable).
    pub amount: Option<Decimal>,
    /// Currency (if applicable).
    pub currency: Option<String>,
    /// User who performed the action.
    pub user_id: Uuid,
    /// User full name.
    pub user_full_name: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// Activity pagination info.
#[derive(Debug, Clone)]
pub struct ActivityPagination {
    /// Limit.
    pub limit: u64,
    /// Has more results.
    pub has_more: bool,
    /// Next cursor for pagination.
    pub next_cursor: Option<String>,
}

/// Dashboard repository for metrics queries.
#[derive(Debug, Clone)]
pub struct DashboardRepository {
    db: DatabaseConnection,
}

impl DashboardRepository {
    /// Creates a new dashboard repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ========================================================================
    // Dashboard Metrics (Requirements 16.2-16.8)
    // ========================================================================

    /// Queries cash position (sum of cash/bank accounts).
    ///
    /// Requirements: 16.2
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_cash_position(
        &self,
        organization_id: Uuid,
        as_of: NaiveDate,
    ) -> Result<CashPosition, DashboardError> {
        // Get organization for currency
        let org = organizations::Entity::find_by_id(organization_id)
            .one(&self.db)
            .await?
            .ok_or(DashboardError::OrganizationNotFound(organization_id))?;

        // Get cash and bank accounts (account_subtype = 'cash' or 'bank')
        let cash_accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .filter(
                chart_of_accounts::Column::AccountSubtype
                    .is_in([AccountSubtype::Cash, AccountSubtype::Bank]),
            )
            .all(&self.db)
            .await?;

        if cash_accounts.is_empty() {
            return Ok(CashPosition {
                balance: Decimal::ZERO,
                currency: org.base_currency,
                change_from_last_period: Decimal::ZERO,
                change_percent: Decimal::ZERO,
            });
        }

        let account_ids: Vec<Uuid> = cash_accounts.iter().map(|a| a.id).collect();

        // Get posted transaction IDs up to as_of date
        let posted_tx_ids: Vec<Uuid> = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .filter(transactions::Column::TransactionDate.lte(as_of))
            .select_only()
            .column(transactions::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok(CashPosition {
                balance: Decimal::ZERO,
                currency: org.base_currency,
                change_from_last_period: Decimal::ZERO,
                change_percent: Decimal::ZERO,
            });
        }

        // Sum balances for cash accounts (Asset accounts: debit - credit)
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.is_in(account_ids))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids))
            .all(&self.db)
            .await?;

        let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = entries.iter().map(|e| e.credit).sum();
        let balance = total_debit - total_credit;

        Ok(CashPosition {
            balance,
            currency: org.base_currency,
            change_from_last_period: Decimal::ZERO, // TODO: Calculate from previous period
            change_percent: Decimal::ZERO,
        })
    }

    /// Queries pending approvals count and total.
    ///
    /// Requirements: 16.5
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_pending_approvals(
        &self,
        organization_id: Uuid,
    ) -> Result<PendingApprovals, DashboardError> {
        // Count pending transactions
        let pending_txs = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Pending))
            .all(&self.db)
            .await?;

        let count = i32::try_from(pending_txs.len()).unwrap_or(i32::MAX);

        // Sum total amount from ledger entries (debit side)
        let mut total_amount = Decimal::ZERO;
        for tx in &pending_txs {
            let entries = ledger_entries::Entity::find()
                .filter(ledger_entries::Column::TransactionId.eq(tx.id))
                .all(&self.db)
                .await?;

            let tx_total: Decimal = entries.iter().map(|e| e.debit).sum();
            total_amount += tx_total;
        }

        Ok(PendingApprovals {
            count,
            total_amount,
        })
    }

    /// Queries budget status for a period.
    ///
    /// Requirements: 16.6
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_budget_status(
        &self,
        organization_id: Uuid,
        fiscal_period_id: Uuid,
    ) -> Result<BudgetStatus, DashboardError> {
        // Get active budgets for this organization
        let active_budgets = budgets::Entity::find()
            .filter(budgets::Column::OrganizationId.eq(organization_id))
            .filter(budgets::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        if active_budgets.is_empty() {
            return Ok(BudgetStatus {
                total_budgeted: Decimal::ZERO,
                total_spent: Decimal::ZERO,
                utilization_percent: Decimal::ZERO,
            });
        }

        let budget_ids: Vec<Uuid> = active_budgets.iter().map(|b| b.id).collect();

        // Get budget lines for this period
        let budget_lines_list = budget_lines::Entity::find()
            .filter(budget_lines::Column::BudgetId.is_in(budget_ids))
            .filter(budget_lines::Column::FiscalPeriodId.eq(fiscal_period_id))
            .all(&self.db)
            .await?;

        let total_budgeted: Decimal = budget_lines_list.iter().map(|l| l.amount).sum();

        // TODO: Calculate actual spent from ledger entries
        // For now, return zero spent
        let total_spent = Decimal::ZERO;

        let utilization_percent = if total_budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (total_spent / total_budgeted * Decimal::from(100)).round_dp(2)
        };

        Ok(BudgetStatus {
            total_budgeted,
            total_spent,
            utilization_percent,
        })
    }

    /// Queries top expenses by department.
    ///
    /// Requirements: 16.7
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_top_expenses_by_department(
        &self,
        organization_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        limit: usize,
    ) -> Result<Vec<DepartmentExpense>, DashboardError> {
        // Get department dimension type
        let dept_type = dimension_types::Entity::find()
            .filter(dimension_types::Column::OrganizationId.eq(organization_id))
            .filter(dimension_types::Column::Name.eq("DEPARTMENT"))
            .one(&self.db)
            .await?;

        let Some(dept_type) = dept_type else {
            return Ok(vec![]); // No department dimension configured
        };

        // Get expense accounts
        let expense_accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::AccountType.eq(AccountType::Expense))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        if expense_accounts.is_empty() {
            return Ok(vec![]);
        }

        let account_ids: Vec<Uuid> = expense_accounts.iter().map(|a| a.id).collect();

        // Get posted transaction IDs within date range
        let posted_tx_ids: Vec<Uuid> = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .filter(transactions::Column::TransactionDate.gte(from))
            .filter(transactions::Column::TransactionDate.lte(to))
            .select_only()
            .column(transactions::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok(vec![]);
        }

        // Get expense entries
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.is_in(account_ids))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids))
            .all(&self.db)
            .await?;

        // Group by department
        let mut dept_totals: std::collections::HashMap<String, Decimal> =
            std::collections::HashMap::new();
        let mut grand_total = Decimal::ZERO;

        for entry in entries {
            let expense_amount = entry.debit - entry.credit;
            grand_total += expense_amount;

            // Get department dimension for this entry
            let entry_dims = entry_dimensions::Entity::find()
                .filter(entry_dimensions::Column::LedgerEntryId.eq(entry.id))
                .all(&self.db)
                .await?;

            let mut dept_name = "Unassigned".to_string();
            for ed in entry_dims {
                let dv = dimension_values::Entity::find_by_id(ed.dimension_value_id)
                    .one(&self.db)
                    .await?;

                if let Some(dv) = dv
                    && dv.dimension_type_id == dept_type.id
                {
                    dept_name = dv.name;
                    break;
                }
            }

            *dept_totals.entry(dept_name).or_insert(Decimal::ZERO) += expense_amount;
        }

        // Sort by amount descending and take top N
        let mut dept_list: Vec<(String, Decimal)> = dept_totals.into_iter().collect();
        dept_list.sort_by(|a, b| b.1.cmp(&a.1));
        dept_list.truncate(limit);

        // Calculate percentages
        let result: Vec<DepartmentExpense> = dept_list
            .into_iter()
            .map(|(department, amount)| {
                let percent = if grand_total.is_zero() {
                    Decimal::ZERO
                } else {
                    (amount / grand_total * Decimal::from(100)).round_dp(2)
                };
                DepartmentExpense {
                    department,
                    amount,
                    percent,
                }
            })
            .collect();

        Ok(result)
    }

    /// Queries currency exposure.
    ///
    /// Requirements: 16.8
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_currency_exposure(
        &self,
        organization_id: Uuid,
        as_of: NaiveDate,
    ) -> Result<Vec<CurrencyExposure>, DashboardError> {
        // Get all accounts
        let accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        if accounts.is_empty() {
            return Ok(vec![]);
        }

        let account_ids: Vec<Uuid> = accounts.iter().map(|a| a.id).collect();

        // Get posted transaction IDs up to as_of date
        let posted_tx_ids: Vec<Uuid> = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .filter(transactions::Column::TransactionDate.lte(as_of))
            .select_only()
            .column(transactions::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok(vec![]);
        }

        // Get entries and group by source currency
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.is_in(account_ids))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids))
            .all(&self.db)
            .await?;

        let mut currency_totals: std::collections::HashMap<String, (Decimal, Decimal)> =
            std::collections::HashMap::new();
        let mut grand_total_functional = Decimal::ZERO;

        for entry in entries {
            let source_balance = entry.source_amount;
            let functional_balance = entry.functional_amount;

            let totals = currency_totals
                .entry(entry.source_currency.clone())
                .or_insert((Decimal::ZERO, Decimal::ZERO));
            totals.0 += source_balance;
            totals.1 += functional_balance;
            grand_total_functional += functional_balance.abs();
        }

        // Calculate percentages
        let result: Vec<CurrencyExposure> = currency_totals
            .into_iter()
            .map(|(currency, (balance, functional_value))| {
                let percent = if grand_total_functional.is_zero() {
                    Decimal::ZERO
                } else {
                    (functional_value.abs() / grand_total_functional * Decimal::from(100))
                        .round_dp(2)
                };
                CurrencyExposure {
                    currency,
                    balance,
                    functional_value,
                    percent,
                }
            })
            .collect();

        Ok(result)
    }

    // ========================================================================
    // Recent Activity (Requirements 17.1-17.6)
    // ========================================================================

    /// Queries recent activity events.
    ///
    /// Requirements: 17.1, 17.2, 17.3, 17.4, 17.5, 17.6
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn query_recent_activity(
        &self,
        organization_id: Uuid,
        limit: u64,
        cursor: Option<String>,
    ) -> Result<(Vec<ActivityEvent>, ActivityPagination), DashboardError> {
        // Parse cursor if provided (cursor is a timestamp)
        let cursor_time: Option<DateTime<Utc>> = cursor.and_then(|c| {
            DateTime::parse_from_rfc3339(&c)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        // Query recent transactions (Requirement 17.2)
        let mut tx_query = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id));

        if let Some(ct) = cursor_time {
            tx_query = tx_query.filter(transactions::Column::UpdatedAt.lt(ct));
        }

        let recent_txs = tx_query
            .order_by_desc(transactions::Column::UpdatedAt)
            .limit(limit + 1) // +1 to check has_more
            .all(&self.db)
            .await?;

        let limit_usize = usize::try_from(limit).unwrap_or(usize::MAX);
        let has_more = recent_txs.len() > limit_usize;
        let txs_to_process: Vec<_> = recent_txs.into_iter().take(limit_usize).collect();

        let mut events = Vec::with_capacity(txs_to_process.len());

        for tx in txs_to_process {
            // Get user info (Requirement 17.4)
            let user = users::Entity::find_by_id(tx.created_by)
                .one(&self.db)
                .await?;

            let user_full_name = user.map_or_else(|| "Unknown User".to_string(), |u| u.full_name);

            // Determine action based on status
            let action = match tx.status {
                TransactionStatus::Draft => "created",
                TransactionStatus::Pending => "submitted",
                TransactionStatus::Approved => "approved",
                TransactionStatus::Posted => "posted",
                TransactionStatus::Voided => "voided",
            };

            // Get total amount from entries
            let entries = ledger_entries::Entity::find()
                .filter(ledger_entries::Column::TransactionId.eq(tx.id))
                .all(&self.db)
                .await?;

            let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();

            events.push(ActivityEvent {
                id: tx.id,
                event_type: "transaction".to_string(),
                action: action.to_string(),
                entity_type: "transaction".to_string(),
                entity_id: tx.id,
                description: tx.description,
                amount: Some(total_debit),
                currency: entries.first().map(|e| e.functional_currency.clone()),
                user_id: tx.created_by,
                user_full_name,
                timestamp: tx.updated_at.into(),
            });
        }

        // Sort by timestamp descending (Requirement 17.5)
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Calculate next cursor (Requirement 17.6)
        let next_cursor = if has_more {
            events.last().map(|e| e.timestamp.to_rfc3339())
        } else {
            None
        };

        let pagination = ActivityPagination {
            limit,
            has_more,
            next_cursor,
        };

        Ok((events, pagination))
    }
}
