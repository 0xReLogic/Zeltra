//! Budget repository for budget database operations.
//!
//! Implements Requirements 1.1-1.7, 2.1-2.7, 3.1-3.4, 4.1-4.9 for budget management.

use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::entities::{
    budget_line_dimensions, budget_lines, budgets, chart_of_accounts, dimension_values,
    fiscal_periods, fiscal_years, ledger_entries,
    sea_orm_active_enums::{AccountType, BudgetType as DbBudgetType, TransactionStatus},
    transactions,
};

/// Error types for budget operations.
#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    /// Budget not found.
    #[error("Budget not found: {0}")]
    NotFound(Uuid),

    /// Budget is locked and cannot be modified.
    #[error("Budget is locked and cannot be modified")]
    BudgetLocked,

    /// Budget name already exists for this fiscal year.
    #[error("Budget name already exists for this fiscal year")]
    DuplicateName,

    /// Fiscal year not found.
    #[error("Fiscal year not found: {0}")]
    FiscalYearNotFound(Uuid),

    /// Fiscal period not found.
    #[error("Fiscal period not found: {0}")]
    FiscalPeriodNotFound(Uuid),

    /// Fiscal period does not belong to budget's fiscal year.
    #[error("Fiscal period does not belong to budget's fiscal year")]
    PeriodNotInFiscalYear,

    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Budget line already exists for this account and period.
    #[error("Budget line already exists for this account and period")]
    DuplicateBudgetLine,

    /// Amount cannot be negative.
    #[error("Amount cannot be negative")]
    NegativeAmount,

    /// Invalid dimension value.
    #[error("Invalid dimension value: {0}")]
    InvalidDimension(Uuid),

    /// Budget line not found.
    #[error("Budget line not found: {0}")]
    BudgetLineNotFound(Uuid),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Input for creating a budget.
#[derive(Debug, Clone)]
pub struct CreateBudgetInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Fiscal year ID.
    pub fiscal_year_id: Uuid,
    /// Budget name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Budget type.
    pub budget_type: DbBudgetType,
    /// Currency code.
    pub currency: String,
    /// User creating the budget.
    pub created_by: Uuid,
}

/// Input for updating a budget.
#[derive(Debug, Clone, Default)]
pub struct UpdateBudgetInput {
    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<Option<String>>,
    /// New active status.
    pub is_active: Option<bool>,
}

/// Input for creating a budget line.
#[derive(Debug, Clone)]
pub struct CreateBudgetLineInput {
    /// Account ID.
    pub account_id: Uuid,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Budgeted amount.
    pub amount: Decimal,
    /// Optional notes.
    pub notes: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Input for updating a budget line.
#[derive(Debug, Clone, Default)]
pub struct UpdateBudgetLineInput {
    /// New amount.
    pub amount: Option<Decimal>,
    /// New notes.
    pub notes: Option<Option<String>>,
}

/// Budget with summary totals.
#[derive(Debug, Clone)]
pub struct BudgetWithSummary {
    /// Budget record.
    pub budget: budgets::Model,
    /// Fiscal year name.
    pub fiscal_year_name: String,
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
}

/// Budget line with dimensions.
#[derive(Debug, Clone)]
pub struct BudgetLineWithDimensions {
    /// Budget line record.
    pub line: budget_lines::Model,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Budget line with actual amount and variance.
#[derive(Debug, Clone)]
pub struct BudgetLineWithActual {
    /// Budget line record.
    pub line: budget_lines::Model,
    /// Account code.
    pub account_code: String,
    /// Account name.
    pub account_name: String,
    /// Period name.
    pub period_name: String,
    /// Actual amount.
    pub actual: Decimal,
    /// Variance (budgeted - actual for expenses, actual - budgeted for revenue).
    pub variance: Decimal,
    /// Utilization percentage.
    pub utilization_percent: Decimal,
    /// Variance status: favorable, unfavorable, on_budget.
    pub status: String,
    /// Dimension values.
    pub dimensions: Vec<DimensionValueInfo>,
}

/// Budget vs actual summary.
#[derive(Debug, Clone)]
pub struct BudgetVsActualSummary {
    /// Total budgeted amount.
    pub total_budgeted: Decimal,
    /// Total actual amount.
    pub total_actual: Decimal,
    /// Total variance.
    pub total_variance: Decimal,
    /// Overall utilization percentage.
    pub overall_utilization: Decimal,
}

/// Budget repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct BudgetRepository {
    db: DatabaseConnection,
}

impl BudgetRepository {
    /// Creates a new budget repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ========================================================================
    // Budget CRUD Operations (Requirements 1.1-1.6)
    // ========================================================================

    /// Creates a new budget.
    ///
    /// Requirements: 1.1, 1.2, 1.3, 1.4
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Fiscal year does not exist
    /// - Budget name already exists for this fiscal year
    /// - Database operation fails
    pub async fn create_budget(
        &self,
        input: CreateBudgetInput,
    ) -> Result<budgets::Model, BudgetError> {
        // Validate fiscal year exists (Requirement 1.2)
        let _fiscal_year = fiscal_years::Entity::find_by_id(input.fiscal_year_id)
            .filter(fiscal_years::Column::OrganizationId.eq(input.organization_id))
            .one(&self.db)
            .await?
            .ok_or(BudgetError::FiscalYearNotFound(input.fiscal_year_id))?;

        // Check for duplicate name (Requirement 1.4)
        let existing = budgets::Entity::find()
            .filter(budgets::Column::OrganizationId.eq(input.organization_id))
            .filter(budgets::Column::FiscalYearId.eq(input.fiscal_year_id))
            .filter(budgets::Column::Name.eq(&input.name))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(BudgetError::DuplicateName);
        }

        let now = Utc::now().into();
        let budget_id = Uuid::new_v4();

        // Currency must be provided in input
        let currency = input.currency;

        let budget = budgets::ActiveModel {
            id: Set(budget_id),
            organization_id: Set(input.organization_id),
            fiscal_year_id: Set(input.fiscal_year_id),
            name: Set(input.name),
            description: Set(input.description),
            budget_type: Set(input.budget_type),
            currency: Set(currency),
            is_active: Set(true),
            is_locked: Set(false),
            created_by: Set(input.created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = budget.insert(&self.db).await?;
        Ok(result)
    }

    /// Gets a budget by ID.
    ///
    /// Requirements: 1.1
    ///
    /// # Errors
    ///
    /// Returns an error if the budget is not found or database query fails.
    pub async fn get_budget(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
    ) -> Result<budgets::Model, BudgetError> {
        budgets::Entity::find_by_id(budget_id)
            .filter(budgets::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(BudgetError::NotFound(budget_id))
    }

    /// Lists budgets for an organization with summary totals.
    ///
    /// Requirements: 1.5
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_budgets(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<BudgetWithSummary>, BudgetError> {
        let budgets_list = budgets::Entity::find()
            .filter(budgets::Column::OrganizationId.eq(organization_id))
            .order_by_desc(budgets::Column::CreatedAt)
            .all(&self.db)
            .await?;

        let mut result = Vec::with_capacity(budgets_list.len());

        for budget in budgets_list {
            // Get fiscal year name
            let fiscal_year = fiscal_years::Entity::find_by_id(budget.fiscal_year_id)
                .one(&self.db)
                .await?
                .map(|fy| fy.name)
                .unwrap_or_default();

            // Calculate total budgeted
            let total_budgeted = self.calculate_total_budgeted(budget.id).await?;

            result.push(BudgetWithSummary {
                budget,
                fiscal_year_name: fiscal_year,
                total_budgeted,
            });
        }

        Ok(result)
    }

    /// Updates a budget.
    ///
    /// Requirements: 1.1
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Budget is not found
    /// - Budget is locked
    /// - Database operation fails
    pub async fn update_budget(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
        input: UpdateBudgetInput,
    ) -> Result<budgets::Model, BudgetError> {
        let budget = self.get_budget(organization_id, budget_id).await?;

        // Check if locked (Requirement 1.7)
        if budget.is_locked {
            return Err(BudgetError::BudgetLocked);
        }

        let mut active: budgets::ActiveModel = budget.into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(description);
        }
        if let Some(is_active) = input.is_active {
            active.is_active = Set(is_active);
        }
        active.updated_at = Set(Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Locks a budget to prevent further modifications.
    ///
    /// Requirements: 1.6
    ///
    /// # Errors
    ///
    /// Returns an error if the budget is not found or database operation fails.
    pub async fn lock_budget(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
    ) -> Result<budgets::Model, BudgetError> {
        let budget = self.get_budget(organization_id, budget_id).await?;

        let mut active: budgets::ActiveModel = budget.into();
        active.is_locked = Set(true);
        active.updated_at = Set(Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    // ========================================================================
    // Budget Line Operations (Requirements 2.1-2.7)
    // ========================================================================

    /// Creates budget lines in bulk.
    ///
    /// Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Budget is locked
    /// - Account does not exist
    /// - Fiscal period does not belong to budget's fiscal year
    /// - Amount is negative
    /// - Duplicate budget line exists
    /// - Database operation fails
    pub async fn create_budget_lines(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
        lines: Vec<CreateBudgetLineInput>,
    ) -> Result<Vec<BudgetLineWithDimensions>, BudgetError> {
        let budget = self.get_budget(organization_id, budget_id).await?;

        // Check if locked (Requirement 1.7)
        if budget.is_locked {
            return Err(BudgetError::BudgetLocked);
        }

        // Start transaction (Requirement 2.6)
        let txn = self.db.begin().await?;

        let mut result = Vec::with_capacity(lines.len());

        for line_input in lines {
            let line = self
                .create_budget_line_internal(&txn, &budget, &line_input)
                .await?;
            result.push(line);
        }

        txn.commit().await?;
        Ok(result)
    }

    /// Internal helper to create a single budget line within a transaction.
    async fn create_budget_line_internal(
        &self,
        txn: &DatabaseTransaction,
        budget: &budgets::Model,
        input: &CreateBudgetLineInput,
    ) -> Result<BudgetLineWithDimensions, BudgetError> {
        // Validate amount (Requirement 2.4)
        if input.amount < Decimal::ZERO {
            return Err(BudgetError::NegativeAmount);
        }

        // Validate account exists (Requirement 2.2)
        let _account = chart_of_accounts::Entity::find_by_id(input.account_id)
            .filter(chart_of_accounts::Column::OrganizationId.eq(budget.organization_id))
            .one(txn)
            .await?
            .ok_or(BudgetError::AccountNotFound(input.account_id))?;

        // Validate fiscal period belongs to budget's fiscal year (Requirement 2.3)
        let period = fiscal_periods::Entity::find_by_id(input.fiscal_period_id)
            .one(txn)
            .await?
            .ok_or(BudgetError::FiscalPeriodNotFound(input.fiscal_period_id))?;

        if period.fiscal_year_id != budget.fiscal_year_id {
            return Err(BudgetError::PeriodNotInFiscalYear);
        }

        // Check for duplicate (Requirement 2.5)
        let existing = budget_lines::Entity::find()
            .filter(budget_lines::Column::BudgetId.eq(budget.id))
            .filter(budget_lines::Column::AccountId.eq(input.account_id))
            .filter(budget_lines::Column::FiscalPeriodId.eq(input.fiscal_period_id))
            .one(txn)
            .await?;

        if existing.is_some() {
            return Err(BudgetError::DuplicateBudgetLine);
        }

        // Validate dimensions (Requirement 3.1)
        for dim_id in &input.dimensions {
            let dim = dimension_values::Entity::find_by_id(*dim_id)
                .one(txn)
                .await?
                .ok_or(BudgetError::InvalidDimension(*dim_id))?;

            if !dim.is_active {
                return Err(BudgetError::InvalidDimension(*dim_id));
            }
        }

        let now = Utc::now().into();
        let line_id = Uuid::new_v4();

        // Create budget line
        let line = budget_lines::ActiveModel {
            id: Set(line_id),
            budget_id: Set(budget.id),
            account_id: Set(input.account_id),
            fiscal_period_id: Set(input.fiscal_period_id),
            amount: Set(input.amount),
            notes: Set(input.notes.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let inserted_line = line.insert(txn).await?;

        // Create dimension associations (Requirement 3.2)
        for dim_id in &input.dimensions {
            let dim_assoc = budget_line_dimensions::ActiveModel {
                id: Set(Uuid::new_v4()),
                budget_line_id: Set(line_id),
                dimension_value_id: Set(*dim_id),
                created_at: Set(now),
            };
            dim_assoc.insert(txn).await?;
        }

        Ok(BudgetLineWithDimensions {
            line: inserted_line,
            dimensions: input.dimensions.clone(),
        })
    }

    /// Gets budget lines for a budget.
    ///
    /// Requirements: 2.7
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_budget_lines(
        &self,
        budget_id: Uuid,
    ) -> Result<Vec<BudgetLineWithDimensions>, BudgetError> {
        let lines = budget_lines::Entity::find()
            .filter(budget_lines::Column::BudgetId.eq(budget_id))
            .order_by_asc(budget_lines::Column::CreatedAt)
            .all(&self.db)
            .await?;

        let mut result = Vec::with_capacity(lines.len());

        for line in lines {
            let dimensions = budget_line_dimensions::Entity::find()
                .filter(budget_line_dimensions::Column::BudgetLineId.eq(line.id))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|d| d.dimension_value_id)
                .collect();

            result.push(BudgetLineWithDimensions { line, dimensions });
        }

        Ok(result)
    }

    /// Updates a budget line.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Budget is locked
    /// - Budget line is not found
    /// - Amount is negative
    /// - Database operation fails
    pub async fn update_budget_line(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
        line_id: Uuid,
        input: UpdateBudgetLineInput,
    ) -> Result<budget_lines::Model, BudgetError> {
        let budget = self.get_budget(organization_id, budget_id).await?;

        if budget.is_locked {
            return Err(BudgetError::BudgetLocked);
        }

        let line = budget_lines::Entity::find_by_id(line_id)
            .filter(budget_lines::Column::BudgetId.eq(budget_id))
            .one(&self.db)
            .await?
            .ok_or(BudgetError::BudgetLineNotFound(line_id))?;

        let mut active: budget_lines::ActiveModel = line.into();

        if let Some(amount) = input.amount {
            if amount < Decimal::ZERO {
                return Err(BudgetError::NegativeAmount);
            }
            active.amount = Set(amount);
        }
        if let Some(notes) = input.notes {
            active.notes = Set(notes);
        }
        active.updated_at = Set(Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Deletes a budget line.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Budget is locked
    /// - Budget line is not found
    /// - Database operation fails
    pub async fn delete_budget_line(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
        line_id: Uuid,
    ) -> Result<(), BudgetError> {
        let budget = self.get_budget(organization_id, budget_id).await?;

        if budget.is_locked {
            return Err(BudgetError::BudgetLocked);
        }

        let result = budget_lines::Entity::delete_by_id(line_id)
            .filter(budget_lines::Column::BudgetId.eq(budget_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(BudgetError::BudgetLineNotFound(line_id));
        }

        Ok(())
    }

    // ========================================================================
    // Dimension Operations (Requirements 3.1-3.4)
    // ========================================================================

    /// Gets dimension values for a budget line.
    ///
    /// Requirements: 3.4
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_budget_line_dimensions(
        &self,
        line_id: Uuid,
    ) -> Result<Vec<DimensionValueInfo>, BudgetError> {
        use crate::entities::dimension_types;

        let dims = budget_line_dimensions::Entity::find()
            .filter(budget_line_dimensions::Column::BudgetLineId.eq(line_id))
            .all(&self.db)
            .await?;

        let mut result = Vec::with_capacity(dims.len());

        for dim in dims {
            let value = dimension_values::Entity::find_by_id(dim.dimension_value_id)
                .one(&self.db)
                .await?;

            if let Some(value) = value {
                let dim_type = dimension_types::Entity::find_by_id(value.dimension_type_id)
                    .one(&self.db)
                    .await?;

                result.push(DimensionValueInfo {
                    dimension_value_id: value.id,
                    dimension_type: dim_type.map(|t| t.name).unwrap_or_default(),
                    code: value.code,
                    name: value.name,
                });
            }
        }

        Ok(result)
    }

    // ========================================================================
    // Actual Amount Calculation (Requirements 4.1-4.3, 4.9)
    // ========================================================================

    /// Calculates actual amount for a budget line from posted ledger entries.
    ///
    /// Requirements: 4.1, 4.2, 4.3, 4.9
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn calculate_actual_amount(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        fiscal_period_id: Uuid,
        dimension_filters: &[Uuid],
    ) -> Result<ActualAmountResult, BudgetError> {
        // Get fiscal period date range
        let period = fiscal_periods::Entity::find_by_id(fiscal_period_id)
            .one(&self.db)
            .await?
            .ok_or(BudgetError::FiscalPeriodNotFound(fiscal_period_id))?;

        // Get account type for balance calculation
        let account = chart_of_accounts::Entity::find_by_id(account_id)
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(BudgetError::AccountNotFound(account_id))?;

        // Query posted ledger entries for this account and period
        let entries = self
            .query_posted_entries(
                organization_id,
                account_id,
                period.start_date,
                period.end_date,
                dimension_filters,
            )
            .await?;

        // Calculate actual based on account type (Requirements 4.2, 4.3)
        let actual = calculate_actual_by_account_type(&account.account_type, &entries);

        Ok(ActualAmountResult {
            account_id,
            account_type: account.account_type,
            fiscal_period_id,
            total_debit: entries.iter().map(|e| e.debit).sum(),
            total_credit: entries.iter().map(|e| e.credit).sum(),
            actual,
        })
    }

    /// Queries posted ledger entries for an account within a date range.
    async fn query_posted_entries(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        dimension_filters: &[Uuid],
    ) -> Result<Vec<ledger_entries::Model>, BudgetError> {
        use crate::entities::entry_dimensions;

        // Get posted transaction IDs for this organization and date range
        let posted_tx_ids: Vec<Uuid> = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .filter(transactions::Column::TransactionDate.gte(start_date))
            .filter(transactions::Column::TransactionDate.lte(end_date))
            .select_only()
            .column(transactions::Column::Id)
            .into_tuple()
            .all(&self.db)
            .await?;

        if posted_tx_ids.is_empty() {
            return Ok(vec![]);
        }

        // Query ledger entries for this account
        let mut entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .filter(ledger_entries::Column::TransactionId.is_in(posted_tx_ids))
            .all(&self.db)
            .await?;

        // Filter by dimensions if specified (Requirement 4.9)
        if !dimension_filters.is_empty() {
            let mut filtered_entries = Vec::new();

            for entry in entries {
                let entry_dims: Vec<Uuid> = entry_dimensions::Entity::find()
                    .filter(entry_dimensions::Column::LedgerEntryId.eq(entry.id))
                    .all(&self.db)
                    .await?
                    .into_iter()
                    .map(|d| d.dimension_value_id)
                    .collect();

                // Check if entry has all required dimensions
                let has_all_dims = dimension_filters.iter().all(|dim| entry_dims.contains(dim));

                if has_all_dims {
                    filtered_entries.push(entry);
                }
            }

            entries = filtered_entries;
        }

        Ok(entries)
    }

    // ========================================================================
    // Budget vs Actual (Requirements 4.1-4.9)
    // ========================================================================

    /// Gets budget vs actual comparison with variance analysis.
    ///
    /// Requirements: 4.1, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_budget_vs_actual(
        &self,
        organization_id: Uuid,
        budget_id: Uuid,
        fiscal_period_id: Option<Uuid>,
        dimension_filters: &[Uuid],
    ) -> Result<(Vec<BudgetLineWithActual>, BudgetVsActualSummary), BudgetError> {
        let budget = self.get_budget(organization_id, budget_id).await?;

        // Get budget lines, optionally filtered by period
        let mut query =
            budget_lines::Entity::find().filter(budget_lines::Column::BudgetId.eq(budget_id));

        if let Some(period_id) = fiscal_period_id {
            query = query.filter(budget_lines::Column::FiscalPeriodId.eq(period_id));
        }

        let lines = query
            .order_by_asc(budget_lines::Column::CreatedAt)
            .all(&self.db)
            .await?;

        let mut result = Vec::with_capacity(lines.len());
        let mut total_budgeted = Decimal::ZERO;
        let mut total_actual = Decimal::ZERO;

        for line in lines {
            // Get account info
            let account = chart_of_accounts::Entity::find_by_id(line.account_id)
                .one(&self.db)
                .await?
                .ok_or(BudgetError::AccountNotFound(line.account_id))?;

            // Get period info
            let period = fiscal_periods::Entity::find_by_id(line.fiscal_period_id)
                .one(&self.db)
                .await?
                .ok_or(BudgetError::FiscalPeriodNotFound(line.fiscal_period_id))?;

            // Get dimension values for this line
            let line_dims = budget_line_dimensions::Entity::find()
                .filter(budget_line_dimensions::Column::BudgetLineId.eq(line.id))
                .all(&self.db)
                .await?;

            // Filter by dimensions if specified
            if !dimension_filters.is_empty() {
                let line_dim_ids: Vec<Uuid> =
                    line_dims.iter().map(|d| d.dimension_value_id).collect();
                let has_all = dimension_filters.iter().all(|f| line_dim_ids.contains(f));
                if !has_all {
                    continue;
                }
            }

            // Calculate actual amount
            let actual_result = self
                .calculate_actual_amount(
                    budget.organization_id,
                    line.account_id,
                    line.fiscal_period_id,
                    dimension_filters,
                )
                .await?;

            let actual = actual_result.actual;
            let budgeted = line.amount;

            // Calculate variance based on account type (Requirements 4.4, 4.5)
            let variance = match account.account_type {
                AccountType::Expense | AccountType::Asset => budgeted - actual,
                AccountType::Revenue | AccountType::Liability | AccountType::Equity => {
                    actual - budgeted
                }
            };

            // Determine status (Requirements 4.6, 4.7)
            let status = match variance.cmp(&Decimal::ZERO) {
                std::cmp::Ordering::Greater => "favorable".to_string(),
                std::cmp::Ordering::Less => "unfavorable".to_string(),
                std::cmp::Ordering::Equal => "on_budget".to_string(),
            };

            // Calculate utilization (Requirement 4.8)
            let utilization_percent = if budgeted.is_zero() {
                Decimal::ZERO
            } else {
                (actual / budgeted * Decimal::from(100)).round_dp(2)
            };

            // Get dimension info
            let dimensions = self.get_dimension_info(&line_dims).await?;

            total_budgeted += budgeted;
            total_actual += actual;

            result.push(BudgetLineWithActual {
                line,
                account_code: account.code,
                account_name: account.name,
                period_name: period.name,
                actual,
                variance,
                utilization_percent,
                status,
                dimensions,
            });
        }

        let total_variance = total_budgeted - total_actual;
        let overall_utilization = if total_budgeted.is_zero() {
            Decimal::ZERO
        } else {
            (total_actual / total_budgeted * Decimal::from(100)).round_dp(2)
        };

        let summary = BudgetVsActualSummary {
            total_budgeted,
            total_actual,
            total_variance,
            overall_utilization,
        };

        Ok((result, summary))
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Gets dimension info for budget line dimensions.
    async fn get_dimension_info(
        &self,
        line_dims: &[budget_line_dimensions::Model],
    ) -> Result<Vec<DimensionValueInfo>, BudgetError> {
        let mut dimensions = Vec::new();
        for dim in line_dims {
            let dim_value = dimension_values::Entity::find_by_id(dim.dimension_value_id)
                .one(&self.db)
                .await?;

            if let Some(dv) = dim_value {
                let dim_type =
                    crate::entities::dimension_types::Entity::find_by_id(dv.dimension_type_id)
                        .one(&self.db)
                        .await?;

                dimensions.push(DimensionValueInfo {
                    dimension_value_id: dv.id,
                    dimension_type: dim_type.map(|t| t.name).unwrap_or_default(),
                    code: dv.code,
                    name: dv.name,
                });
            }
        }
        Ok(dimensions)
    }

    /// Calculates total budgeted amount for a budget.
    async fn calculate_total_budgeted(&self, budget_id: Uuid) -> Result<Decimal, BudgetError> {
        let lines = budget_lines::Entity::find()
            .filter(budget_lines::Column::BudgetId.eq(budget_id))
            .all(&self.db)
            .await?;

        Ok(lines.iter().map(|l| l.amount).sum())
    }
}

// ============================================================================
// Helper Types
// ============================================================================

/// Dimension value information.
#[derive(Debug, Clone)]
pub struct DimensionValueInfo {
    /// Dimension value ID.
    pub dimension_value_id: Uuid,
    /// Dimension type name.
    pub dimension_type: String,
    /// Dimension value code.
    pub code: String,
    /// Dimension value name.
    pub name: String,
}

/// Result of actual amount calculation.
#[derive(Debug, Clone)]
pub struct ActualAmountResult {
    /// Account ID.
    pub account_id: Uuid,
    /// Account type.
    pub account_type: AccountType,
    /// Fiscal period ID.
    pub fiscal_period_id: Uuid,
    /// Total debit amount.
    pub total_debit: Decimal,
    /// Total credit amount.
    pub total_credit: Decimal,
    /// Calculated actual amount.
    pub actual: Decimal,
}

// ============================================================================
// Actual Amount Calculation Helper (Requirements 4.2, 4.3)
// ============================================================================

/// Calculates actual amount based on account type.
///
/// Property 5: Actual Amount Calculation by Account Type
/// - For expense/asset accounts: actual = sum(debit) - sum(credit)
/// - For revenue/liability/equity accounts: actual = sum(credit) - sum(debit)
///
/// Requirements: 4.2, 4.3
#[must_use]
pub fn calculate_actual_by_account_type(
    account_type: &AccountType,
    entries: &[ledger_entries::Model],
) -> Decimal {
    let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();
    let total_credit: Decimal = entries.iter().map(|e| e.credit).sum();

    match account_type {
        // Debit-normal accounts (Requirement 4.2)
        AccountType::Asset | AccountType::Expense => total_debit - total_credit,
        // Credit-normal accounts (Requirement 4.3)
        AccountType::Liability | AccountType::Equity | AccountType::Revenue => {
            total_credit - total_debit
        }
    }
}

/// Determines if an account type is debit-normal.
#[must_use]
pub fn is_debit_normal_account(account_type: &AccountType) -> bool {
    matches!(account_type, AccountType::Asset | AccountType::Expense)
}

#[cfg(test)]
#[path = "budget_tests.rs"]
mod tests;
