//! Account repository for chart of accounts database operations.
//!
//! Implements Requirements 2.1-2.7 for chart of accounts management.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, JoinType,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};
use uuid::Uuid;

use crate::entities::{
    chart_of_accounts, currencies, ledger_entries,
    sea_orm_active_enums::{AccountSubtype, AccountType, TransactionStatus},
    transactions,
};

/// Error types for account operations.
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    /// Account code already exists in organization.
    #[error("Account code '{0}' already exists")]
    DuplicateCode(String),

    /// Currency not found.
    #[error("Currency '{0}' not found")]
    CurrencyNotFound(String),

    /// Parent account not found.
    #[error("Parent account not found: {0}")]
    ParentNotFound(Uuid),

    /// Parent account belongs to different organization.
    #[error("Parent account belongs to different organization")]
    ParentWrongOrganization,

    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Cannot change account type because account has ledger entries.
    #[error("Cannot change account type: account has {0} ledger entries")]
    HasLedgerEntries(u64),

    /// Cannot delete account because it has ledger entries.
    #[error("Cannot delete account: account has {0} ledger entries")]
    CannotDeleteWithEntries(u64),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Account with computed balance.
#[derive(Debug, Clone)]
pub struct AccountWithBalance {
    /// The account record.
    pub account: chart_of_accounts::Model,
    /// Current balance (from latest ledger entry, or zero if no entries).
    pub balance: Decimal,
}

/// Ledger entry with transaction details for ledger listing.
#[derive(Debug, Clone)]
pub struct LedgerEntryWithTransaction {
    /// The ledger entry.
    pub entry: ledger_entries::Model,
    /// Transaction date.
    pub transaction_date: NaiveDate,
    /// Transaction reference number.
    pub reference_number: Option<String>,
    /// Transaction description.
    pub description: String,
    /// Transaction status.
    pub status: TransactionStatus,
}

/// Paginated result for ledger entries.
#[derive(Debug, Clone)]
pub struct PaginatedLedgerEntries {
    /// The ledger entries.
    pub entries: Vec<LedgerEntryWithTransaction>,
    /// Total count of entries.
    pub total: u64,
    /// Current page (1-indexed).
    pub page: u64,
    /// Page size.
    pub limit: u64,
    /// Total pages.
    pub total_pages: u64,
}

/// Input for creating an account.
#[derive(Debug, Clone)]
pub struct CreateAccountInput {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Account code (must be unique within organization).
    pub code: String,
    /// Account name.
    pub name: String,
    /// Account description.
    pub description: Option<String>,
    /// Account type (asset, liability, equity, revenue, expense).
    pub account_type: AccountType,
    /// Account subtype for more specific categorization.
    pub account_subtype: Option<AccountSubtype>,
    /// Parent account ID for hierarchical structure.
    pub parent_id: Option<Uuid>,
    /// Currency code.
    pub currency: String,
    /// Whether the account is active.
    pub is_active: bool,
    /// Whether direct posting is allowed.
    pub allow_direct_posting: bool,
}

/// Input for updating an account.
#[derive(Debug, Clone, Default)]
pub struct UpdateAccountInput {
    /// Account code.
    pub code: Option<String>,
    /// Account name.
    pub name: Option<String>,
    /// Account description.
    pub description: Option<Option<String>>,
    /// Account type (only if no ledger entries).
    pub account_type: Option<AccountType>,
    /// Account subtype.
    pub account_subtype: Option<Option<AccountSubtype>>,
    /// Parent account ID.
    pub parent_id: Option<Option<Uuid>>,
    /// Whether the account is active.
    pub is_active: Option<bool>,
    /// Whether direct posting is allowed.
    pub allow_direct_posting: Option<bool>,
}

/// Filter options for listing accounts.
#[derive(Debug, Clone, Default)]
pub struct AccountFilter {
    /// Filter by account type.
    pub account_type: Option<AccountType>,
    /// Filter by active status.
    pub is_active: Option<bool>,
    /// Filter by parent ID (None = root accounts only).
    pub parent_id: Option<Option<Uuid>>,
}

/// Account repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct AccountRepository {
    db: DatabaseConnection,
}

impl AccountRepository {
    /// Creates a new account repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new account with validation.
    ///
    /// Requirements: 2.1, 2.2, 2.3, 2.4
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Account code already exists in organization
    /// - Currency does not exist
    /// - Parent account does not exist or belongs to different organization
    pub async fn create_account(
        &self,
        input: CreateAccountInput,
    ) -> Result<chart_of_accounts::Model, AccountError> {
        // Validate unique code within organization (Requirement 2.2)
        let existing = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(input.organization_id))
            .filter(chart_of_accounts::Column::Code.eq(&input.code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AccountError::DuplicateCode(input.code));
        }

        // Validate currency exists (Requirement 2.3)
        let currency = currencies::Entity::find_by_id(&input.currency)
            .one(&self.db)
            .await?;

        if currency.is_none() {
            return Err(AccountError::CurrencyNotFound(input.currency));
        }

        // Validate parent account if provided (Requirement 2.4)
        if let Some(parent_id) = input.parent_id {
            let parent = chart_of_accounts::Entity::find_by_id(parent_id)
                .one(&self.db)
                .await?;

            match parent {
                None => return Err(AccountError::ParentNotFound(parent_id)),
                Some(p) if p.organization_id != input.organization_id => {
                    return Err(AccountError::ParentWrongOrganization);
                }
                _ => {}
            }
        }

        let now = chrono::Utc::now().into();
        let account = chart_of_accounts::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(input.organization_id),
            code: Set(input.code),
            name: Set(input.name),
            description: Set(input.description),
            account_type: Set(input.account_type),
            account_subtype: Set(input.account_subtype),
            parent_id: Set(input.parent_id),
            currency: Set(input.currency),
            is_active: Set(input.is_active),
            is_system_account: Set(false),
            allow_direct_posting: Set(input.allow_direct_posting),
            is_bank_account: Set(false),
            bank_account_number: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let account = account.insert(&self.db).await?;
        Ok(account)
    }

    /// Lists accounts for an organization with computed balances.
    ///
    /// Requirements: 2.5
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_accounts(
        &self,
        organization_id: Uuid,
        filter: AccountFilter,
    ) -> Result<Vec<AccountWithBalance>, AccountError> {
        let mut query = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .order_by_asc(chart_of_accounts::Column::Code);

        if let Some(account_type) = filter.account_type {
            query = query.filter(chart_of_accounts::Column::AccountType.eq(account_type));
        }

        if let Some(is_active) = filter.is_active {
            query = query.filter(chart_of_accounts::Column::IsActive.eq(is_active));
        }

        if let Some(parent_id) = filter.parent_id {
            match parent_id {
                Some(pid) => {
                    query = query.filter(chart_of_accounts::Column::ParentId.eq(pid));
                }
                None => {
                    query = query.filter(chart_of_accounts::Column::ParentId.is_null());
                }
            }
        }

        let accounts = query.all(&self.db).await?;

        // Get balances for all accounts
        let mut results = Vec::with_capacity(accounts.len());
        for account in accounts {
            let balance = self.get_account_balance(account.id).await?;
            results.push(AccountWithBalance { account, balance });
        }

        Ok(results)
    }

    /// Finds an account by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn find_account_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<AccountWithBalance>, AccountError> {
        let account = chart_of_accounts::Entity::find_by_id(id)
            .one(&self.db)
            .await?;

        match account {
            Some(acc) => {
                let balance = self.get_account_balance(acc.id).await?;
                Ok(Some(AccountWithBalance {
                    account: acc,
                    balance,
                }))
            }
            None => Ok(None),
        }
    }

    /// Updates an account with validation.
    ///
    /// Requirements: 2.6
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Account not found
    /// - Trying to change account_type when account has ledger entries
    /// - New code already exists in organization
    /// - Parent account validation fails
    pub async fn update_account(
        &self,
        id: Uuid,
        input: UpdateAccountInput,
    ) -> Result<chart_of_accounts::Model, AccountError> {
        let account = chart_of_accounts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(AccountError::AccountNotFound(id))?;

        // If changing account_type, check for ledger entries (Requirement 2.6)
        if let Some(new_type) = &input.account_type
            && *new_type != account.account_type
        {
            let entry_count = self.count_ledger_entries(id).await?;
            if entry_count > 0 {
                return Err(AccountError::HasLedgerEntries(entry_count));
            }
        }

        // If changing code, validate uniqueness
        if let Some(new_code) = &input.code
            && *new_code != account.code
        {
            let existing = chart_of_accounts::Entity::find()
                .filter(chart_of_accounts::Column::OrganizationId.eq(account.organization_id))
                .filter(chart_of_accounts::Column::Code.eq(new_code))
                .filter(chart_of_accounts::Column::Id.ne(id))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                return Err(AccountError::DuplicateCode(new_code.clone()));
            }
        }

        // If changing parent, validate
        if let Some(new_parent) = &input.parent_id
            && let Some(parent_id) = new_parent
        {
            let parent = chart_of_accounts::Entity::find_by_id(*parent_id)
                .one(&self.db)
                .await?;

            match parent {
                None => return Err(AccountError::ParentNotFound(*parent_id)),
                Some(p) if p.organization_id != account.organization_id => {
                    return Err(AccountError::ParentWrongOrganization);
                }
                _ => {}
            }
        }

        let now = chrono::Utc::now().into();
        let mut active: chart_of_accounts::ActiveModel = account.into();

        if let Some(code) = input.code {
            active.code = Set(code);
        }
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(description);
        }
        if let Some(account_type) = input.account_type {
            active.account_type = Set(account_type);
        }
        if let Some(account_subtype) = input.account_subtype {
            active.account_subtype = Set(account_subtype);
        }
        if let Some(parent_id) = input.parent_id {
            active.parent_id = Set(parent_id);
        }
        if let Some(is_active) = input.is_active {
            active.is_active = Set(is_active);
        }
        if let Some(allow_direct_posting) = input.allow_direct_posting {
            active.allow_direct_posting = Set(allow_direct_posting);
        }
        active.updated_at = Set(now);

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Deletes (deactivates) an account.
    ///
    /// Requirements: 2.7
    ///
    /// Note: This performs a soft delete by setting is_active = false.
    /// Accounts with ledger entries cannot be deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Account not found
    /// - Account has ledger entries
    pub async fn delete_account(&self, id: Uuid) -> Result<(), AccountError> {
        let account = chart_of_accounts::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(AccountError::AccountNotFound(id))?;

        // Check for ledger entries
        let entry_count = self.count_ledger_entries(id).await?;
        if entry_count > 0 {
            return Err(AccountError::CannotDeleteWithEntries(entry_count));
        }

        // Soft delete by setting is_active = false
        let now = chrono::Utc::now().into();
        let mut active: chart_of_accounts::ActiveModel = account.into();
        active.is_active = Set(false);
        active.updated_at = Set(now);
        active.update(&self.db).await?;

        Ok(())
    }

    /// Checks if an account code exists in an organization.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn code_exists(
        &self,
        organization_id: Uuid,
        code: &str,
    ) -> Result<bool, AccountError> {
        let count = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::Code.eq(code))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// Gets the current balance for an account.
    ///
    /// Returns the `account_current_balance` from the most recent ledger entry,
    /// or zero if no entries exist.
    async fn get_account_balance(&self, account_id: Uuid) -> Result<Decimal, AccountError> {
        let latest_entry = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .order_by_desc(ledger_entries::Column::AccountVersion)
            .one(&self.db)
            .await?;

        Ok(latest_entry.map_or(Decimal::ZERO, |e| e.account_current_balance))
    }

    /// Counts ledger entries for an account.
    async fn count_ledger_entries(&self, account_id: Uuid) -> Result<u64, AccountError> {
        let count = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    /// Gets the balance for an account at a specific date.
    ///
    /// Returns the `account_current_balance` from the most recent ledger entry
    /// on or before the given date, or zero if no entries exist before that date.
    ///
    /// # Arguments
    /// * `account_id` - The account ID
    /// * `as_of` - The date to get the balance as of (inclusive)
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_balance_at_date(
        &self,
        account_id: Uuid,
        as_of: NaiveDate,
    ) -> Result<Decimal, AccountError> {
        // Join with transactions to filter by transaction_date
        let latest_entry = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .join(
                JoinType::InnerJoin,
                ledger_entries::Relation::Transactions.def(),
            )
            .filter(transactions::Column::TransactionDate.lte(as_of))
            .filter(transactions::Column::Status.eq(TransactionStatus::Posted))
            .order_by_desc(transactions::Column::TransactionDate)
            .order_by_desc(ledger_entries::Column::AccountVersion)
            .one(&self.db)
            .await?;

        Ok(latest_entry.map_or(Decimal::ZERO, |e| e.account_current_balance))
    }

    /// Gets ledger entries for an account with pagination.
    ///
    /// Returns ledger entries with transaction details, filtered by date range.
    ///
    /// # Arguments
    /// * `account_id` - The account ID
    /// * `from` - Optional start date (inclusive)
    /// * `to` - Optional end date (inclusive)
    /// * `page` - Page number (1-indexed)
    /// * `limit` - Number of entries per page
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_ledger_entries(
        &self,
        account_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        page: u64,
        limit: u64,
    ) -> Result<PaginatedLedgerEntries, AccountError> {
        use sea_orm::FromQueryResult;

        // Helper struct for query result - defined before any statements
        #[derive(Debug, FromQueryResult)]
        struct LedgerEntryRow {
            // Ledger entry fields
            id: Uuid,
            transaction_id: Uuid,
            account_id: Uuid,
            source_currency: String,
            source_amount: Decimal,
            exchange_rate: Decimal,
            functional_currency: String,
            functional_amount: Decimal,
            debit: Decimal,
            credit: Decimal,
            account_version: i64,
            account_previous_balance: Decimal,
            account_current_balance: Decimal,
            memo: Option<String>,
            event_at: chrono::DateTime<chrono::FixedOffset>,
            created_at: chrono::DateTime<chrono::FixedOffset>,
            // Transaction fields (aliased)
            txn_date: NaiveDate,
            txn_ref: Option<String>,
            txn_desc: String,
            txn_status: TransactionStatus,
        }

        // Build base query for counting
        let mut count_query = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .join(
                JoinType::InnerJoin,
                ledger_entries::Relation::Transactions.def(),
            );

        // Apply date filters to count query
        if let Some(from_date) = from {
            count_query = count_query.filter(transactions::Column::TransactionDate.gte(from_date));
        }
        if let Some(to_date) = to {
            count_query = count_query.filter(transactions::Column::TransactionDate.lte(to_date));
        }

        // Get total count first
        let total = count_query.count(&self.db).await?;

        // Calculate pagination
        let total_pages = if total == 0 { 1 } else { total.div_ceil(limit) };
        let offset = (page.saturating_sub(1)) * limit;

        // Build data query with column aliases
        let mut query = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .join(
                JoinType::InnerJoin,
                ledger_entries::Relation::Transactions.def(),
            )
            .column_as(transactions::Column::TransactionDate, "txn_date")
            .column_as(transactions::Column::ReferenceNumber, "txn_ref")
            .column_as(transactions::Column::Description, "txn_desc")
            .column_as(transactions::Column::Status, "txn_status");

        if let Some(from_date) = from {
            query = query.filter(transactions::Column::TransactionDate.gte(from_date));
        }
        if let Some(to_date) = to {
            query = query.filter(transactions::Column::TransactionDate.lte(to_date));
        }

        let rows: Vec<LedgerEntryRow> = query
            .order_by_desc(transactions::Column::TransactionDate)
            .order_by_desc(ledger_entries::Column::AccountVersion)
            .offset(offset)
            .limit(limit)
            .into_model::<LedgerEntryRow>()
            .all(&self.db)
            .await?;

        // Convert to result type
        let entries = rows
            .into_iter()
            .map(|row| LedgerEntryWithTransaction {
                entry: ledger_entries::Model {
                    id: row.id,
                    transaction_id: row.transaction_id,
                    account_id: row.account_id,
                    source_currency: row.source_currency,
                    source_amount: row.source_amount,
                    exchange_rate: row.exchange_rate,
                    functional_currency: row.functional_currency,
                    functional_amount: row.functional_amount,
                    debit: row.debit,
                    credit: row.credit,
                    account_version: row.account_version,
                    account_previous_balance: row.account_previous_balance,
                    account_current_balance: row.account_current_balance,
                    memo: row.memo,
                    event_at: row.event_at,
                    created_at: row.created_at,
                },
                transaction_date: row.txn_date,
                reference_number: row.txn_ref,
                description: row.txn_desc,
                status: row.txn_status,
            })
            .collect();

        Ok(PaginatedLedgerEntries {
            entries,
            total,
            page,
            limit,
            total_pages,
        })
    }
}

// ============================================================================
// Pure validation functions for property testing
// ============================================================================

/// Represents an account code entry for uniqueness checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountCodeEntry {
    /// Organization ID.
    pub organization_id: Uuid,
    /// Account code.
    pub code: String,
}

/// Checks if a code would be unique within an organization given existing codes.
///
/// This is a pure function that can be tested without database access.
///
/// # Arguments
/// * `existing_codes` - Set of existing (org_id, code) pairs
/// * `new_org_id` - Organization ID for the new code
/// * `new_code` - The code to check
///
/// # Returns
/// * `true` if the code is unique within the organization
/// * `false` if the code already exists in the organization
#[must_use]
pub fn is_code_unique<S: std::hash::BuildHasher>(
    existing_codes: &std::collections::HashSet<AccountCodeEntry, S>,
    new_org_id: Uuid,
    new_code: &str,
) -> bool {
    let entry = AccountCodeEntry {
        organization_id: new_org_id,
        code: new_code.to_string(),
    };
    !existing_codes.contains(&entry)
}

/// Checks if updating an account code would cause a conflict.
///
/// # Arguments
/// * `existing_codes` - Set of existing (org_id, code) pairs
/// * `current_account_id` - ID of the account being updated
/// * `current_org_id` - Organization ID of the account
/// * `current_code` - Current code of the account
/// * `new_code` - The new code to set
///
/// # Returns
/// * `true` if the update is valid (no conflict)
/// * `false` if the new code conflicts with another account
#[must_use]
pub fn is_code_update_valid<S: std::hash::BuildHasher>(
    existing_codes: &std::collections::HashSet<AccountCodeEntry, S>,
    current_org_id: Uuid,
    current_code: &str,
    new_code: &str,
) -> bool {
    // Same code = no change, always valid
    if current_code == new_code {
        return true;
    }

    // Check if new code exists in same org (excluding current account)
    let new_entry = AccountCodeEntry {
        organization_id: current_org_id,
        code: new_code.to_string(),
    };

    !existing_codes.contains(&new_entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    // ========================================================================
    // Property 11: Uniqueness Constraints
    // **Validates: Requirements 2.2, 3.2, 3.4**
    // ========================================================================

    /// Strategy for generating valid account codes (alphanumeric, 1-20 chars)
    fn account_code_strategy() -> impl Strategy<Value = String> {
        "[A-Z0-9]{1,10}"
    }

    /// Strategy for generating a set of existing account codes
    fn existing_codes_strategy() -> impl Strategy<Value = HashSet<AccountCodeEntry>> {
        prop::collection::hash_set(
            (any::<u128>(), account_code_strategy()).prop_map(|(org_bits, code)| {
                AccountCodeEntry {
                    organization_id: Uuid::from_u128(org_bits),
                    code,
                }
            }),
            0..20,
        )
    }

    // ------------------------------------------------------------------------
    // Property 11.1: Duplicate account codes in same organization are rejected
    // ------------------------------------------------------------------------

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Property 11.1: Duplicate codes in same org rejected**
        ///
        /// *For any* existing account code in an organization,
        /// attempting to create another account with the same code
        /// in the same organization SHALL be rejected.
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn prop_duplicate_code_same_org_rejected(
            org_bits in any::<u128>(),
            code in account_code_strategy(),
        ) {
            let org_id = Uuid::from_u128(org_bits);

            // Create existing codes with the target code
            let mut existing = HashSet::new();
            existing.insert(AccountCodeEntry {
                organization_id: org_id,
                code: code.clone(),
            });

            // Attempting to add same code in same org should fail
            let is_unique = is_code_unique(&existing, org_id, &code);
            prop_assert!(!is_unique, "Duplicate code in same org should be rejected");
        }

        /// **Property 11.2: Same code in different organizations allowed**
        ///
        /// *For any* account code, the same code CAN exist in different
        /// organizations (uniqueness is per-organization, not global).
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn prop_same_code_different_org_allowed(
            org1_bits in any::<u128>(),
            org2_bits in any::<u128>(),
            code in account_code_strategy(),
        ) {
            // Ensure different orgs
            prop_assume!(org1_bits != org2_bits);

            let org1_id = Uuid::from_u128(org1_bits);
            let org2_id = Uuid::from_u128(org2_bits);

            // Create existing codes with code in org1
            let mut existing = HashSet::new();
            existing.insert(AccountCodeEntry {
                organization_id: org1_id,
                code: code.clone(),
            });

            // Same code in different org should be allowed
            let is_unique = is_code_unique(&existing, org2_id, &code);
            prop_assert!(is_unique, "Same code in different org should be allowed");
        }

        /// **Property 11.3: Unique codes are accepted**
        ///
        /// *For any* new code that doesn't exist in the organization,
        /// the code SHALL be accepted.
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn prop_unique_code_accepted(
            existing in existing_codes_strategy(),
            org_bits in any::<u128>(),
            new_code in account_code_strategy(),
        ) {
            let org_id = Uuid::from_u128(org_bits);

            // Check if code already exists in this org
            let entry = AccountCodeEntry {
                organization_id: org_id,
                code: new_code.clone(),
            };
            let already_exists = existing.contains(&entry);

            let is_unique = is_code_unique(&existing, org_id, &new_code);

            // Result should match: unique if not already exists
            prop_assert_eq!(is_unique, !already_exists);
        }

        /// **Property 11.4: Code update to same value always valid**
        ///
        /// *For any* account, updating the code to its current value
        /// SHALL always be valid (no-op).
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn prop_code_update_same_value_valid(
            existing in existing_codes_strategy(),
            org_bits in any::<u128>(),
            code in account_code_strategy(),
        ) {
            let org_id = Uuid::from_u128(org_bits);

            // Updating to same code should always be valid
            let is_valid = is_code_update_valid(&existing, org_id, &code, &code);
            prop_assert!(is_valid, "Updating code to same value should always be valid");
        }

        /// **Property 11.5: Code update to existing code rejected**
        ///
        /// *For any* account, updating the code to a code that already
        /// exists in the same organization SHALL be rejected.
        ///
        /// **Validates: Requirements 2.2**
        #[test]
        fn prop_code_update_to_existing_rejected(
            org_bits in any::<u128>(),
            current_code in account_code_strategy(),
            other_code in account_code_strategy(),
        ) {
            // Ensure different codes
            prop_assume!(current_code != other_code);

            let org_id = Uuid::from_u128(org_bits);

            // Create existing codes with both codes
            let mut existing = HashSet::new();
            existing.insert(AccountCodeEntry {
                organization_id: org_id,
                code: current_code.clone(),
            });
            existing.insert(AccountCodeEntry {
                organization_id: org_id,
                code: other_code.clone(),
            });

            // Updating to other_code should fail (it exists)
            let is_valid = is_code_update_valid(&existing, org_id, &current_code, &other_code);
            prop_assert!(!is_valid, "Updating to existing code should be rejected");
        }
    }

    // ========================================================================
    // Unit tests for edge cases
    // ========================================================================

    #[test]
    fn test_empty_existing_codes_allows_any() {
        let existing = HashSet::new();
        let org_id = Uuid::new_v4();

        assert!(is_code_unique(&existing, org_id, "1000"));
        assert!(is_code_unique(&existing, org_id, "CASH"));
        assert!(is_code_unique(&existing, org_id, "A"));
    }

    #[test]
    fn test_case_sensitive_codes() {
        let org_id = Uuid::new_v4();
        let mut existing = HashSet::new();
        existing.insert(AccountCodeEntry {
            organization_id: org_id,
            code: "CASH".to_string(),
        });

        // Same case = duplicate
        assert!(!is_code_unique(&existing, org_id, "CASH"));

        // Different case = unique (codes are case-sensitive)
        assert!(is_code_unique(&existing, org_id, "cash"));
        assert!(is_code_unique(&existing, org_id, "Cash"));
    }

    #[test]
    fn test_multiple_orgs_isolation() {
        let org1 = Uuid::new_v4();
        let org2 = Uuid::new_v4();
        let org3 = Uuid::new_v4();

        let mut existing = HashSet::new();
        existing.insert(AccountCodeEntry {
            organization_id: org1,
            code: "1000".to_string(),
        });
        existing.insert(AccountCodeEntry {
            organization_id: org2,
            code: "1000".to_string(),
        });

        // Code exists in org1 and org2
        assert!(!is_code_unique(&existing, org1, "1000"));
        assert!(!is_code_unique(&existing, org2, "1000"));

        // Code doesn't exist in org3
        assert!(is_code_unique(&existing, org3, "1000"));
    }

    #[test]
    fn test_update_to_new_unique_code() {
        let org_id = Uuid::new_v4();
        let mut existing = HashSet::new();
        existing.insert(AccountCodeEntry {
            organization_id: org_id,
            code: "1000".to_string(),
        });

        // Update from 1000 to 2000 (2000 doesn't exist) = valid
        assert!(is_code_update_valid(&existing, org_id, "1000", "2000"));
    }
}
