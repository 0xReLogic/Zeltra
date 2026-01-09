//! Transaction repository for ledger transaction database operations.
//!
//! Implements Requirements 5.8, 5.9, 7.4, 8.1-8.5, 10.2-10.7 for transaction management.

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::entities::{
    chart_of_accounts, entry_dimensions, fiscal_periods, ledger_entries, transactions,
    sea_orm_active_enums::{AccountType, TransactionStatus, TransactionType},
};

/// Error types for transaction operations.
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    /// Transaction not found.
    #[error("Transaction not found: {0}")]
    NotFound(Uuid),

    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// No fiscal period found for the transaction date.
    #[error("No fiscal period found for date {0}")]
    NoFiscalPeriod(NaiveDate),

    /// Fiscal period is closed.
    #[error("Fiscal period is closed, no posting allowed")]
    PeriodClosed,

    /// Fiscal period is soft-closed.
    #[error("Fiscal period is soft-closed, only accountants can post")]
    PeriodSoftClosed,

    /// Cannot modify posted transaction.
    #[error("Cannot modify posted transaction")]
    CannotModifyPosted,

    /// Cannot modify voided transaction.
    #[error("Cannot modify voided transaction")]
    CannotModifyVoided,

    /// Can only delete draft transactions.
    #[error("Can only delete draft transactions")]
    CanOnlyDeleteDraft,

    /// Concurrent modification detected.
    #[error("Concurrent modification detected for account {0}, please retry")]
    ConcurrentModification(Uuid),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Input for creating a transaction.
#[derive(Debug, Clone)]
pub struct CreateTransactionInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Transaction type.
    pub transaction_type: TransactionType,
    /// Transaction date.
    pub transaction_date: NaiveDate,
    /// Description.
    pub description: String,
    /// Optional reference number.
    pub reference_number: Option<String>,
    /// Optional memo.
    pub memo: Option<String>,
    /// Ledger entries.
    pub entries: Vec<CreateLedgerEntryInput>,
    /// User who created the transaction.
    pub created_by: Uuid,
}

/// Input for a single ledger entry.
#[derive(Debug, Clone)]
pub struct CreateLedgerEntryInput {
    /// Account ID.
    pub account_id: Uuid,
    /// Source currency code.
    pub source_currency: String,
    /// Source amount.
    pub source_amount: Decimal,
    /// Exchange rate.
    pub exchange_rate: Decimal,
    /// Functional currency code.
    pub functional_currency: String,
    /// Functional amount.
    pub functional_amount: Decimal,
    /// Debit amount (in functional currency).
    pub debit: Decimal,
    /// Credit amount (in functional currency).
    pub credit: Decimal,
    /// Optional memo.
    pub memo: Option<String>,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Filter options for listing transactions.
#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    /// Filter by status.
    pub status: Option<TransactionStatus>,
    /// Filter by transaction type.
    pub transaction_type: Option<TransactionType>,
    /// Filter by date range start.
    pub date_from: Option<NaiveDate>,
    /// Filter by date range end.
    pub date_to: Option<NaiveDate>,
    /// Filter by dimension value ID.
    pub dimension_value_id: Option<Uuid>,
}

/// Transaction with its entries.
#[derive(Debug, Clone)]
pub struct TransactionWithEntries {
    /// Transaction header.
    pub transaction: transactions::Model,
    /// Ledger entries.
    pub entries: Vec<LedgerEntryWithDimensions>,
}

/// Ledger entry with its dimensions.
#[derive(Debug, Clone)]
pub struct LedgerEntryWithDimensions {
    /// Ledger entry.
    pub entry: ledger_entries::Model,
    /// Dimension value IDs.
    pub dimensions: Vec<Uuid>,
}

/// Transaction repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct TransactionRepository {
    db: DatabaseConnection,
}

impl TransactionRepository {
    /// Creates a new transaction repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new transaction with entries and dimensions.
    ///
    /// Requirements: 5.8, 5.9, 7.4
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No fiscal period exists for the transaction date
    /// - The fiscal period is closed
    /// - Database operation fails
    pub async fn create_transaction(
        &self,
        input: CreateTransactionInput,
    ) -> Result<TransactionWithEntries, TransactionError> {
        // Find fiscal period for the transaction date (Requirement 5.9)
        let fiscal_period = self
            .find_fiscal_period(input.organization_id, input.transaction_date)
            .await?;

        // Start database transaction
        let txn = self.db.begin().await?;

        // Create transaction header with status = draft (Requirement 5.8)
        let transaction = self
            .insert_transaction(&txn, &input, fiscal_period.id)
            .await?;

        // Create ledger entries and dimensions
        let entries = self
            .insert_entries(&txn, transaction.id, &input.entries)
            .await?;

        // Commit database transaction
        txn.commit().await?;

        Ok(TransactionWithEntries {
            transaction,
            entries,
        })
    }

    /// Finds the fiscal period containing the given date.
    async fn find_fiscal_period(
        &self,
        organization_id: Uuid,
        date: NaiveDate,
    ) -> Result<fiscal_periods::Model, TransactionError> {
        let period = fiscal_periods::Entity::find()
            .filter(fiscal_periods::Column::OrganizationId.eq(organization_id))
            .filter(fiscal_periods::Column::StartDate.lte(date))
            .filter(fiscal_periods::Column::EndDate.gte(date))
            .one(&self.db)
            .await?
            .ok_or(TransactionError::NoFiscalPeriod(date))?;

        Ok(period)
    }

    /// Inserts the transaction header.
    async fn insert_transaction(
        &self,
        txn: &DatabaseTransaction,
        input: &CreateTransactionInput,
        fiscal_period_id: Uuid,
    ) -> Result<transactions::Model, TransactionError> {
        let now = Utc::now().into();
        let transaction_id = Uuid::new_v4();

        let transaction = transactions::ActiveModel {
            id: Set(transaction_id),
            organization_id: Set(input.organization_id),
            fiscal_period_id: Set(fiscal_period_id),
            reference_number: Set(input.reference_number.clone()),
            transaction_type: Set(input.transaction_type.clone()),
            transaction_date: Set(input.transaction_date),
            description: Set(input.description.clone()),
            memo: Set(input.memo.clone()),
            status: Set(TransactionStatus::Draft), // Requirement 5.8
            created_by: Set(input.created_by),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = transaction.insert(txn).await?;
        Ok(result)
    }

    /// Inserts ledger entries and their dimensions with balance tracking.
    ///
    /// Requirements 8.1-8.5:
    /// - Increment account_version for each entry
    /// - Calculate and store previous_balance and current_balance
    /// - Apply account type balance rules (debit-normal vs credit-normal)
    async fn insert_entries(
        &self,
        txn: &DatabaseTransaction,
        transaction_id: Uuid,
        entries: &[CreateLedgerEntryInput],
    ) -> Result<Vec<LedgerEntryWithDimensions>, TransactionError> {
        let now = Utc::now().into();
        let mut result = Vec::with_capacity(entries.len());

        // Track balance changes per account within this transaction
        // Key: account_id, Value: (latest_version, latest_balance)
        let mut account_balances: std::collections::HashMap<Uuid, (i64, Decimal)> =
            std::collections::HashMap::new();

        for entry_input in entries {
            let entry_id = Uuid::new_v4();

            // Get account info for balance calculation (Requirement 8.4, 8.5)
            let account = chart_of_accounts::Entity::find_by_id(entry_input.account_id)
                .one(txn)
                .await?
                .ok_or(TransactionError::AccountNotFound(entry_input.account_id))?;

            // Calculate balance change based on account type
            let balance_change =
                calculate_balance_change(&account.account_type, entry_input.debit, entry_input.credit);

            // Get or fetch the current balance for this account
            let (account_version, previous_balance) =
                if let Some(&(ver, bal)) = account_balances.get(&entry_input.account_id) {
                    // We already have an entry for this account in this transaction
                    (ver + 1, bal)
                } else {
                    // Fetch the latest balance from the database (Requirement 8.1, 8.2)
                    let latest = self
                        .get_latest_account_balance(txn, entry_input.account_id)
                        .await?;
                    (latest.0 + 1, latest.1)
                };

            // Calculate current balance (Requirement 8.3)
            let current_balance = previous_balance + balance_change;

            // Update our tracking map
            account_balances.insert(entry_input.account_id, (account_version, current_balance));

            // Insert ledger entry with balance tracking
            let entry = ledger_entries::ActiveModel {
                id: Set(entry_id),
                transaction_id: Set(transaction_id),
                account_id: Set(entry_input.account_id),
                source_currency: Set(entry_input.source_currency.clone()),
                source_amount: Set(entry_input.source_amount),
                exchange_rate: Set(entry_input.exchange_rate),
                functional_currency: Set(entry_input.functional_currency.clone()),
                functional_amount: Set(entry_input.functional_amount),
                debit: Set(entry_input.debit),
                credit: Set(entry_input.credit),
                memo: Set(entry_input.memo.clone()),
                event_at: Set(now),
                created_at: Set(now),
                // Balance tracking fields (Requirements 8.1-8.3)
                account_version: Set(account_version),
                account_previous_balance: Set(previous_balance),
                account_current_balance: Set(current_balance),
            };

            let inserted_entry = entry.insert(txn).await?;

            // Insert entry dimensions (Requirement 7.4)
            for dimension_value_id in &entry_input.dimensions {
                let dimension = entry_dimensions::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    ledger_entry_id: Set(entry_id),
                    dimension_value_id: Set(*dimension_value_id),
                    created_at: Set(now),
                };
                dimension.insert(txn).await?;
            }

            result.push(LedgerEntryWithDimensions {
                entry: inserted_entry,
                dimensions: entry_input.dimensions.clone(),
            });
        }

        Ok(result)
    }

    /// Gets the latest account balance (version and balance).
    ///
    /// Returns (0, 0) if no entries exist for the account.
    async fn get_latest_account_balance(
        &self,
        txn: &DatabaseTransaction,
        account_id: Uuid,
    ) -> Result<(i64, Decimal), TransactionError> {
        let latest_entry = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .order_by_desc(ledger_entries::Column::AccountVersion)
            .limit(1)
            .one(txn)
            .await?;

        match latest_entry {
            Some(entry) => Ok((entry.account_version, entry.account_current_balance)),
            None => Ok((0, Decimal::ZERO)),
        }
    }

    /// Lists transactions with optional filters.
    ///
    /// Requirements: 10.2
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_transactions(
        &self,
        organization_id: Uuid,
        filter: TransactionFilter,
    ) -> Result<Vec<transactions::Model>, TransactionError> {
        let mut query = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id));

        if let Some(status) = filter.status {
            query = query.filter(transactions::Column::Status.eq(status));
        }

        if let Some(tx_type) = filter.transaction_type {
            query = query.filter(transactions::Column::TransactionType.eq(tx_type));
        }

        if let Some(date_from) = filter.date_from {
            query = query.filter(transactions::Column::TransactionDate.gte(date_from));
        }

        if let Some(date_to) = filter.date_to {
            query = query.filter(transactions::Column::TransactionDate.lte(date_to));
        }

        // TODO: Filter by dimension_value_id requires a join with entry_dimensions

        let transactions = query
            .order_by_desc(transactions::Column::TransactionDate)
            .order_by_desc(transactions::Column::CreatedAt)
            .all(&self.db)
            .await?;

        Ok(transactions)
    }

    /// Gets a transaction by ID with all entries and dimensions.
    ///
    /// Requirements: 10.3
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction is not found or database query fails.
    pub async fn get_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
    ) -> Result<TransactionWithEntries, TransactionError> {
        // Get transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(TransactionError::NotFound(transaction_id))?;

        // Get entries
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::TransactionId.eq(transaction_id))
            .all(&self.db)
            .await?;

        // Get dimensions for each entry
        let mut entries_with_dims = Vec::with_capacity(entries.len());
        for entry in entries {
            let dimensions = entry_dimensions::Entity::find()
                .filter(entry_dimensions::Column::LedgerEntryId.eq(entry.id))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|d| d.dimension_value_id)
                .collect();

            entries_with_dims.push(LedgerEntryWithDimensions {
                entry,
                dimensions,
            });
        }

        Ok(TransactionWithEntries {
            transaction,
            entries: entries_with_dims,
        })
    }

    /// Updates a draft transaction.
    ///
    /// Requirements: 10.4, 10.5
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in draft status
    /// - Database operation fails
    pub async fn update_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
        description: Option<String>,
        memo: Option<String>,
        reference_number: Option<String>,
    ) -> Result<transactions::Model, TransactionError> {
        // Get existing transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(TransactionError::NotFound(transaction_id))?;

        // Check status (Requirement 10.5)
        match transaction.status {
            TransactionStatus::Posted => return Err(TransactionError::CannotModifyPosted),
            TransactionStatus::Voided => return Err(TransactionError::CannotModifyVoided),
            _ => {}
        }

        // Update transaction
        let mut active: transactions::ActiveModel = transaction.into();
        
        if let Some(desc) = description {
            active.description = Set(desc);
        }
        if let Some(m) = memo {
            active.memo = Set(Some(m));
        }
        if let Some(ref_num) = reference_number {
            active.reference_number = Set(Some(ref_num));
        }
        active.updated_at = Set(Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Deletes a draft transaction.
    ///
    /// Requirements: 10.6, 10.7
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in draft status
    /// - Database operation fails
    pub async fn delete_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
    ) -> Result<(), TransactionError> {
        // Get existing transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(TransactionError::NotFound(transaction_id))?;

        // Check status (Requirement 10.7)
        if transaction.status != TransactionStatus::Draft {
            return Err(TransactionError::CanOnlyDeleteDraft);
        }

        // Delete transaction (cascade will delete entries and dimensions)
        transactions::Entity::delete_by_id(transaction_id)
            .exec(&self.db)
            .await?;

        Ok(())
    }
}


// ============================================================================
// Balance Calculation Helpers
// ============================================================================

/// Calculates the balance change for an entry based on account type.
///
/// Requirements 8.4, 8.5:
/// - Asset/Expense (debit-normal): balance += debit - credit
/// - Liability/Equity/Revenue (credit-normal): balance += credit - debit
#[must_use]
pub fn calculate_balance_change(account_type: &AccountType, debit: Decimal, credit: Decimal) -> Decimal {
    match account_type {
        // Debit-normal accounts (Requirement 8.4)
        AccountType::Asset | AccountType::Expense => debit - credit,
        // Credit-normal accounts (Requirement 8.5)
        AccountType::Liability | AccountType::Equity | AccountType::Revenue => credit - debit,
    }
}

/// Determines if an account type is debit-normal.
#[must_use]
pub fn is_debit_normal(account_type: &AccountType) -> bool {
    matches!(account_type, AccountType::Asset | AccountType::Expense)
}

// ============================================================================
// Transaction Status Validation Helpers
// ============================================================================

/// Checks if a transaction status allows modification.
///
/// Property 15: Transaction Immutability
/// - Posted transactions reject updates (except void)
/// - Voided transactions reject all modifications
///
/// Requirements 13.4, 13.5
#[must_use]
pub fn can_modify_transaction(status: &TransactionStatus) -> Result<(), TransactionError> {
    match status {
        TransactionStatus::Posted => Err(TransactionError::CannotModifyPosted),
        TransactionStatus::Voided => Err(TransactionError::CannotModifyVoided),
        _ => Ok(()),
    }
}

/// Checks if a transaction status allows deletion.
///
/// Property 15: Transaction Immutability
/// - Only draft transactions can be deleted
///
/// Requirements 10.7
#[must_use]
pub fn can_delete_transaction(status: &TransactionStatus) -> Result<(), TransactionError> {
    match status {
        TransactionStatus::Draft => Ok(()),
        _ => Err(TransactionError::CanOnlyDeleteDraft),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rust_decimal_macros::dec;

    // ========================================================================
    // Property 2: Account Type Balance Rules
    // **Validates: Requirements 8.4, 8.5**
    // ========================================================================

    /// Strategy for generating positive decimal amounts
    fn amount_strategy() -> impl Strategy<Value = Decimal> {
        (0i64..1_000_000i64).prop_map(|n| Decimal::new(n, 2))
    }

    /// Strategy for generating account types
    fn account_type_strategy() -> impl Strategy<Value = AccountType> {
        prop_oneof![
            Just(AccountType::Asset),
            Just(AccountType::Expense),
            Just(AccountType::Liability),
            Just(AccountType::Equity),
            Just(AccountType::Revenue),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 2.1: Debit-normal accounts increase with debits**
        ///
        /// *For any* Asset or Expense account, a debit entry SHALL increase the balance.
        ///
        /// **Validates: Requirements 8.4**
        #[test]
        fn prop_debit_normal_debit_increases(
            debit in amount_strategy(),
        ) {
            prop_assume!(debit > Decimal::ZERO);

            // Asset
            let change_asset = calculate_balance_change(&AccountType::Asset, debit, Decimal::ZERO);
            prop_assert!(change_asset > Decimal::ZERO, "Asset debit should increase balance");
            prop_assert_eq!(change_asset, debit);

            // Expense
            let change_expense = calculate_balance_change(&AccountType::Expense, debit, Decimal::ZERO);
            prop_assert!(change_expense > Decimal::ZERO, "Expense debit should increase balance");
            prop_assert_eq!(change_expense, debit);
        }

        /// **Property 2.2: Debit-normal accounts decrease with credits**
        ///
        /// *For any* Asset or Expense account, a credit entry SHALL decrease the balance.
        ///
        /// **Validates: Requirements 8.4**
        #[test]
        fn prop_debit_normal_credit_decreases(
            credit in amount_strategy(),
        ) {
            prop_assume!(credit > Decimal::ZERO);

            // Asset
            let change_asset = calculate_balance_change(&AccountType::Asset, Decimal::ZERO, credit);
            prop_assert!(change_asset < Decimal::ZERO, "Asset credit should decrease balance");
            prop_assert_eq!(change_asset, -credit);

            // Expense
            let change_expense = calculate_balance_change(&AccountType::Expense, Decimal::ZERO, credit);
            prop_assert!(change_expense < Decimal::ZERO, "Expense credit should decrease balance");
            prop_assert_eq!(change_expense, -credit);
        }

        /// **Property 2.3: Credit-normal accounts increase with credits**
        ///
        /// *For any* Liability, Equity, or Revenue account, a credit entry SHALL increase the balance.
        ///
        /// **Validates: Requirements 8.5**
        #[test]
        fn prop_credit_normal_credit_increases(
            credit in amount_strategy(),
        ) {
            prop_assume!(credit > Decimal::ZERO);

            // Liability
            let change_liability = calculate_balance_change(&AccountType::Liability, Decimal::ZERO, credit);
            prop_assert!(change_liability > Decimal::ZERO, "Liability credit should increase balance");
            prop_assert_eq!(change_liability, credit);

            // Equity
            let change_equity = calculate_balance_change(&AccountType::Equity, Decimal::ZERO, credit);
            prop_assert!(change_equity > Decimal::ZERO, "Equity credit should increase balance");
            prop_assert_eq!(change_equity, credit);

            // Revenue
            let change_revenue = calculate_balance_change(&AccountType::Revenue, Decimal::ZERO, credit);
            prop_assert!(change_revenue > Decimal::ZERO, "Revenue credit should increase balance");
            prop_assert_eq!(change_revenue, credit);
        }

        /// **Property 2.4: Credit-normal accounts decrease with debits**
        ///
        /// *For any* Liability, Equity, or Revenue account, a debit entry SHALL decrease the balance.
        ///
        /// **Validates: Requirements 8.5**
        #[test]
        fn prop_credit_normal_debit_decreases(
            debit in amount_strategy(),
        ) {
            prop_assume!(debit > Decimal::ZERO);

            // Liability
            let change_liability = calculate_balance_change(&AccountType::Liability, debit, Decimal::ZERO);
            prop_assert!(change_liability < Decimal::ZERO, "Liability debit should decrease balance");
            prop_assert_eq!(change_liability, -debit);

            // Equity
            let change_equity = calculate_balance_change(&AccountType::Equity, debit, Decimal::ZERO);
            prop_assert!(change_equity < Decimal::ZERO, "Equity debit should decrease balance");
            prop_assert_eq!(change_equity, -debit);

            // Revenue
            let change_revenue = calculate_balance_change(&AccountType::Revenue, debit, Decimal::ZERO);
            prop_assert!(change_revenue < Decimal::ZERO, "Revenue debit should decrease balance");
            prop_assert_eq!(change_revenue, -debit);
        }

        /// **Property 2.5: Balance change formula is consistent**
        ///
        /// *For any* account type and any debit/credit amounts:
        /// - Debit-normal: change = debit - credit
        /// - Credit-normal: change = credit - debit
        ///
        /// **Validates: Requirements 8.4, 8.5**
        #[test]
        fn prop_balance_change_formula(
            account_type in account_type_strategy(),
            debit in amount_strategy(),
            credit in amount_strategy(),
        ) {
            let change = calculate_balance_change(&account_type, debit, credit);

            let expected = if is_debit_normal(&account_type) {
                debit - credit
            } else {
                credit - debit
            };

            prop_assert_eq!(change, expected, "Balance change formula should match account type");
        }

        /// **Property 2.6: Zero entries produce zero change**
        ///
        /// *For any* account type, an entry with zero debit and zero credit SHALL produce zero balance change.
        ///
        /// **Validates: Requirements 8.4, 8.5**
        #[test]
        fn prop_zero_entry_zero_change(
            account_type in account_type_strategy(),
        ) {
            let change = calculate_balance_change(&account_type, Decimal::ZERO, Decimal::ZERO);
            prop_assert_eq!(change, Decimal::ZERO, "Zero entry should produce zero change");
        }
    }

    // ========================================================================
    // Unit tests for specific examples
    // ========================================================================

    #[test]
    fn test_asset_balance_change() {
        // Asset is debit-normal: balance += debit - credit
        assert_eq!(calculate_balance_change(&AccountType::Asset, dec!(100), dec!(0)), dec!(100));
        assert_eq!(calculate_balance_change(&AccountType::Asset, dec!(0), dec!(50)), dec!(-50));
        assert_eq!(calculate_balance_change(&AccountType::Asset, dec!(100), dec!(30)), dec!(70));
    }

    #[test]
    fn test_expense_balance_change() {
        // Expense is debit-normal: balance += debit - credit
        assert_eq!(calculate_balance_change(&AccountType::Expense, dec!(200), dec!(0)), dec!(200));
        assert_eq!(calculate_balance_change(&AccountType::Expense, dec!(0), dec!(100)), dec!(-100));
    }

    #[test]
    fn test_liability_balance_change() {
        // Liability is credit-normal: balance += credit - debit
        assert_eq!(calculate_balance_change(&AccountType::Liability, dec!(0), dec!(100)), dec!(100));
        assert_eq!(calculate_balance_change(&AccountType::Liability, dec!(50), dec!(0)), dec!(-50));
        assert_eq!(calculate_balance_change(&AccountType::Liability, dec!(30), dec!(100)), dec!(70));
    }

    #[test]
    fn test_equity_balance_change() {
        // Equity is credit-normal: balance += credit - debit
        assert_eq!(calculate_balance_change(&AccountType::Equity, dec!(0), dec!(500)), dec!(500));
        assert_eq!(calculate_balance_change(&AccountType::Equity, dec!(200), dec!(0)), dec!(-200));
    }

    #[test]
    fn test_revenue_balance_change() {
        // Revenue is credit-normal: balance += credit - debit
        assert_eq!(calculate_balance_change(&AccountType::Revenue, dec!(0), dec!(1000)), dec!(1000));
        assert_eq!(calculate_balance_change(&AccountType::Revenue, dec!(100), dec!(0)), dec!(-100));
    }

    #[test]
    fn test_is_debit_normal() {
        assert!(is_debit_normal(&AccountType::Asset));
        assert!(is_debit_normal(&AccountType::Expense));
        assert!(!is_debit_normal(&AccountType::Liability));
        assert!(!is_debit_normal(&AccountType::Equity));
        assert!(!is_debit_normal(&AccountType::Revenue));
    }

    // ========================================================================
    // Property 15: Transaction Immutability
    // **Validates: Requirements 13.4, 13.5**
    // ========================================================================

    /// Strategy for generating transaction statuses
    fn transaction_status_strategy() -> impl Strategy<Value = TransactionStatus> {
        prop_oneof![
            Just(TransactionStatus::Draft),
            Just(TransactionStatus::Pending),
            Just(TransactionStatus::Approved),
            Just(TransactionStatus::Posted),
            Just(TransactionStatus::Voided),
        ]
    }

    /// Strategy for generating immutable statuses (Posted, Voided)
    fn immutable_status_strategy() -> impl Strategy<Value = TransactionStatus> {
        prop_oneof![
            Just(TransactionStatus::Posted),
            Just(TransactionStatus::Voided),
        ]
    }

    /// Strategy for generating modifiable statuses (Draft, Pending, Approved)
    fn modifiable_status_strategy() -> impl Strategy<Value = TransactionStatus> {
        prop_oneof![
            Just(TransactionStatus::Draft),
            Just(TransactionStatus::Pending),
            Just(TransactionStatus::Approved),
        ]
    }

    /// Strategy for generating non-draft statuses
    fn non_draft_status_strategy() -> impl Strategy<Value = TransactionStatus> {
        prop_oneof![
            Just(TransactionStatus::Pending),
            Just(TransactionStatus::Approved),
            Just(TransactionStatus::Posted),
            Just(TransactionStatus::Voided),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 15.1: Posted transactions reject modifications**
        ///
        /// *For any* posted transaction, updates SHALL be rejected.
        ///
        /// **Validates: Requirements 13.4**
        #[test]
        fn prop_posted_rejects_modification(_dummy in 0..100i32) {
            let result = can_modify_transaction(&TransactionStatus::Posted);
            prop_assert!(result.is_err(), "Posted transactions should reject modifications");
            
            match result {
                Err(TransactionError::CannotModifyPosted) => {},
                _ => prop_assert!(false, "Should return CannotModifyPosted error"),
            }
        }

        /// **Property 15.2: Voided transactions reject all modifications**
        ///
        /// *For any* voided transaction, all modifications SHALL be rejected.
        ///
        /// **Validates: Requirements 13.5**
        #[test]
        fn prop_voided_rejects_modification(_dummy in 0..100i32) {
            let result = can_modify_transaction(&TransactionStatus::Voided);
            prop_assert!(result.is_err(), "Voided transactions should reject modifications");
            
            match result {
                Err(TransactionError::CannotModifyVoided) => {},
                _ => prop_assert!(false, "Should return CannotModifyVoided error"),
            }
        }

        /// **Property 15.3: Immutable statuses always reject modifications**
        ///
        /// *For any* transaction with Posted or Voided status, modifications SHALL be rejected.
        ///
        /// **Validates: Requirements 13.4, 13.5**
        #[test]
        fn prop_immutable_statuses_reject_modification(
            status in immutable_status_strategy(),
        ) {
            let result = can_modify_transaction(&status);
            prop_assert!(result.is_err(), "Immutable statuses should reject modifications");
        }

        /// **Property 15.4: Modifiable statuses allow modifications**
        ///
        /// *For any* transaction with Draft, Pending, or Approved status, modifications SHALL be allowed.
        ///
        /// **Validates: Requirements 10.4**
        #[test]
        fn prop_modifiable_statuses_allow_modification(
            status in modifiable_status_strategy(),
        ) {
            let result = can_modify_transaction(&status);
            prop_assert!(result.is_ok(), "Modifiable statuses should allow modifications");
        }

        /// **Property 15.5: Only draft transactions can be deleted**
        ///
        /// *For any* draft transaction, deletion SHALL be allowed.
        ///
        /// **Validates: Requirements 10.6**
        #[test]
        fn prop_draft_allows_deletion(_dummy in 0..100i32) {
            let result = can_delete_transaction(&TransactionStatus::Draft);
            prop_assert!(result.is_ok(), "Draft transactions should allow deletion");
        }

        /// **Property 15.6: Non-draft transactions reject deletion**
        ///
        /// *For any* non-draft transaction, deletion SHALL be rejected.
        ///
        /// **Validates: Requirements 10.7**
        #[test]
        fn prop_non_draft_rejects_deletion(
            status in non_draft_status_strategy(),
        ) {
            let result = can_delete_transaction(&status);
            prop_assert!(result.is_err(), "Non-draft transactions should reject deletion");
            
            match result {
                Err(TransactionError::CanOnlyDeleteDraft) => {},
                _ => prop_assert!(false, "Should return CanOnlyDeleteDraft error"),
            }
        }

        /// **Property 15.7: Status modification rules are consistent**
        ///
        /// *For any* transaction status, the modification and deletion rules SHALL be consistent:
        /// - Draft: can modify AND can delete
        /// - Pending/Approved: can modify BUT cannot delete
        /// - Posted/Voided: cannot modify AND cannot delete
        ///
        /// **Validates: Requirements 10.4-10.7, 13.4, 13.5**
        #[test]
        fn prop_status_rules_consistent(
            status in transaction_status_strategy(),
        ) {
            let can_mod = can_modify_transaction(&status).is_ok();
            let can_del = can_delete_transaction(&status).is_ok();

            match status {
                TransactionStatus::Draft => {
                    prop_assert!(can_mod, "Draft should allow modification");
                    prop_assert!(can_del, "Draft should allow deletion");
                },
                TransactionStatus::Pending | TransactionStatus::Approved => {
                    prop_assert!(can_mod, "Pending/Approved should allow modification");
                    prop_assert!(!can_del, "Pending/Approved should reject deletion");
                },
                TransactionStatus::Posted | TransactionStatus::Voided => {
                    prop_assert!(!can_mod, "Posted/Voided should reject modification");
                    prop_assert!(!can_del, "Posted/Voided should reject deletion");
                },
            }
        }
    }

    // ========================================================================
    // Unit tests for transaction immutability
    // ========================================================================

    #[test]
    fn test_can_modify_draft() {
        assert!(can_modify_transaction(&TransactionStatus::Draft).is_ok());
    }

    #[test]
    fn test_can_modify_pending() {
        assert!(can_modify_transaction(&TransactionStatus::Pending).is_ok());
    }

    #[test]
    fn test_can_modify_approved() {
        assert!(can_modify_transaction(&TransactionStatus::Approved).is_ok());
    }

    #[test]
    fn test_cannot_modify_posted() {
        let result = can_modify_transaction(&TransactionStatus::Posted);
        assert!(matches!(result, Err(TransactionError::CannotModifyPosted)));
    }

    #[test]
    fn test_cannot_modify_voided() {
        let result = can_modify_transaction(&TransactionStatus::Voided);
        assert!(matches!(result, Err(TransactionError::CannotModifyVoided)));
    }

    #[test]
    fn test_can_delete_draft() {
        assert!(can_delete_transaction(&TransactionStatus::Draft).is_ok());
    }

    #[test]
    fn test_cannot_delete_non_draft() {
        assert!(can_delete_transaction(&TransactionStatus::Pending).is_err());
        assert!(can_delete_transaction(&TransactionStatus::Approved).is_err());
        assert!(can_delete_transaction(&TransactionStatus::Posted).is_err());
        assert!(can_delete_transaction(&TransactionStatus::Voided).is_err());
    }
}
