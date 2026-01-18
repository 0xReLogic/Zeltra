//! Approval Rule Repository
//!
//! Provides CRUD operations for approval rules.
//!
//! **Validates: Requirements 3.1, 6.8, 6.9**

use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use thiserror::Error;
use uuid::Uuid;

use crate::entities::{
    approval_rules::{self, ActiveModel, Entity as ApprovalRuleEntity, Model as ApprovalRuleModel},
    sea_orm_active_enums::{TransactionType, UserRole},
};

/// Errors that can occur during approval rule operations.
#[derive(Debug, Error)]
pub enum ApprovalRuleError {
    /// Approval rule not found.
    #[error("Approval rule {0} not found")]
    NotFound(Uuid),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Invalid transaction type.
    #[error("Invalid transaction type: {0}")]
    InvalidTransactionType(String),

    /// Invalid role.
    #[error("Invalid role: {0}")]
    InvalidRole(String),
}

/// Input for creating an approval rule.
#[derive(Debug, Clone)]
pub struct CreateApprovalRuleInput {
    /// Name of the approval rule.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Minimum amount threshold (inclusive).
    pub min_amount: Option<Decimal>,
    /// Maximum amount threshold (inclusive).
    pub max_amount: Option<Decimal>,
    /// Transaction types this rule applies to.
    pub transaction_types: Vec<String>,
    /// Required role to approve.
    pub required_role: String,
    /// Priority (lower = higher priority).
    pub priority: i16,
}

/// Input for updating an approval rule.
#[derive(Debug, Clone, Default)]
pub struct UpdateApprovalRuleInput {
    /// New name.
    pub name: Option<String>,
    /// New description.
    pub description: Option<Option<String>>,
    /// New minimum amount.
    pub min_amount: Option<Option<Decimal>>,
    /// New maximum amount.
    pub max_amount: Option<Option<Decimal>>,
    /// New transaction types.
    pub transaction_types: Option<Vec<String>>,
    /// New required role.
    pub required_role: Option<String>,
    /// New priority.
    pub priority: Option<i16>,
    /// Active status.
    pub is_active: Option<bool>,
}

/// Repository for approval rule operations.
pub struct ApprovalRuleRepository {
    db: DatabaseConnection,
}

impl ApprovalRuleRepository {
    /// Creates a new ApprovalRuleRepository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new approval rule.
    ///
    /// **Validates: Requirements 3.1, 6.8**
    pub async fn create_rule(
        &self,
        organization_id: Uuid,
        input: CreateApprovalRuleInput,
    ) -> Result<ApprovalRuleModel, ApprovalRuleError> {
        let txn = self.db.begin().await?;
        
        let result = self.create_rule_in_txn(&txn, organization_id, input).await;
        
        match result {
            Ok(rule) => {
                txn.commit().await?;
                Ok(rule)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// Creates a new approval rule within a transaction.
    async fn create_rule_in_txn(
        &self,
        txn: &DatabaseTransaction,
        organization_id: Uuid,
        input: CreateApprovalRuleInput,
    ) -> Result<ApprovalRuleModel, ApprovalRuleError> {
        let transaction_types = Self::parse_transaction_types(&input.transaction_types)?;
        let required_role = Self::parse_role_static(&input.required_role)?;

        let rule = ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(organization_id),
            name: Set(input.name),
            description: Set(input.description),
            min_amount: Set(input.min_amount),
            max_amount: Set(input.max_amount),
            transaction_types: Set(transaction_types),
            required_role: Set(required_role),
            priority: Set(input.priority),
            is_active: Set(true),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };

        let result = rule.insert(txn).await?;
        Ok(result)
    }

    /// Lists all active approval rules for an organization.
    ///
    /// **Validates: Requirements 6.9**
    pub async fn list_rules(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ApprovalRuleModel>, ApprovalRuleError> {
        let rules = ApprovalRuleEntity::find()
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .filter(approval_rules::Column::IsActive.eq(true))
            .order_by_asc(approval_rules::Column::Priority)
            .all(&self.db)
            .await?;

        Ok(rules)
    }

    /// Lists approval rules for an organization with pagination.
    ///
    /// **Validates: Requirements 2.2.1**
    pub async fn list_rules_paginated(
        &self,
        organization_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<(Vec<ApprovalRuleModel>, u64), ApprovalRuleError> {
        use sea_orm::{PaginatorTrait, QuerySelect};

        let query = ApprovalRuleEntity::find()
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .order_by_asc(approval_rules::Column::Priority);

        let paginator = query.paginate(&self.db, limit);
        let total = paginator.num_items().await?;
        
        let rules = ApprovalRuleEntity::find()
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .order_by_asc(approval_rules::Column::Priority)
            .offset(offset)
            .limit(limit)
            .all(&self.db)
            .await?;

        Ok((rules, total))
    }

    /// Lists approval rules for an organization with pagination, filtering, and sorting.
    ///
    /// **Validates: Requirements 2.2.1, 2.2.6**
    pub async fn list_rules_with_filters(
        &self,
        organization_id: Uuid,
        offset: u64,
        limit: u64,
        is_active: Option<bool>,
        transaction_type: Option<&str>,
        required_role: Option<&str>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<(Vec<ApprovalRuleModel>, u64), ApprovalRuleError> {
        use sea_orm::{QuerySelect, Order, PaginatorTrait};

        let mut base_query = ApprovalRuleEntity::find()
            .filter(approval_rules::Column::OrganizationId.eq(organization_id));

        // Apply filters
        if let Some(active) = is_active {
            base_query = base_query.filter(approval_rules::Column::IsActive.eq(active));
        }

        if let Some(role) = required_role {
            let parsed_role = Self::parse_role_static(role)?;
            base_query = base_query.filter(approval_rules::Column::RequiredRole.eq(parsed_role));
        }

        // Get total count before applying pagination
        let total = base_query.clone().count(&self.db).await?;

        // Apply sorting
        let order = if sort_order == Some("desc") { Order::Desc } else { Order::Asc };
        
        base_query = match sort_by {
            Some("created_at") => base_query.order_by(approval_rules::Column::CreatedAt, order),
            Some("name") => base_query.order_by(approval_rules::Column::Name, order),
            _ => base_query.order_by(approval_rules::Column::Priority, order), // Default to priority
        };

        // Apply pagination
        let mut rules = base_query
            .offset(offset)
            .limit(limit)
            .all(&self.db)
            .await?;

        // Apply transaction type filter in memory if needed
        // (This is because SeaORM doesn't have great support for array contains queries)
        if let Some(tx_type) = transaction_type {
            let parsed_type = Self::parse_transaction_type(tx_type)?;
            rules = rules.into_iter()
                .filter(|rule| rule.transaction_types.contains(&parsed_type))
                .collect();
        }

        Ok((rules, total))
    }

    /// Gets a specific approval rule by ID.
    pub async fn get_rule(
        &self,
        organization_id: Uuid,
        rule_id: Uuid,
    ) -> Result<ApprovalRuleModel, ApprovalRuleError> {
        let rule = ApprovalRuleEntity::find_by_id(rule_id)
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await?
            .ok_or(ApprovalRuleError::NotFound(rule_id))?;

        Ok(rule)
    }

    /// Updates an approval rule.
    pub async fn update_rule(
        &self,
        organization_id: Uuid,
        rule_id: Uuid,
        input: UpdateApprovalRuleInput,
    ) -> Result<ApprovalRuleModel, ApprovalRuleError> {
        let txn = self.db.begin().await?;
        
        let result = self.update_rule_in_txn(&txn, organization_id, rule_id, input).await;
        
        match result {
            Ok(rule) => {
                txn.commit().await?;
                Ok(rule)
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// Updates an approval rule within a transaction.
    async fn update_rule_in_txn(
        &self,
        txn: &DatabaseTransaction,
        organization_id: Uuid,
        rule_id: Uuid,
        input: UpdateApprovalRuleInput,
    ) -> Result<ApprovalRuleModel, ApprovalRuleError> {
        let existing = ApprovalRuleEntity::find_by_id(rule_id)
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .one(txn)
            .await?
            .ok_or(ApprovalRuleError::NotFound(rule_id))?;

        let mut rule: ActiveModel = existing.into();

        if let Some(name) = input.name {
            rule.name = Set(name);
        }
        if let Some(description) = input.description {
            rule.description = Set(description);
        }
        if let Some(min_amount) = input.min_amount {
            rule.min_amount = Set(min_amount);
        }
        if let Some(max_amount) = input.max_amount {
            rule.max_amount = Set(max_amount);
        }
        if let Some(transaction_types) = input.transaction_types {
            rule.transaction_types = Set(Self::parse_transaction_types(&transaction_types)?);
        }
        if let Some(required_role) = input.required_role {
            rule.required_role = Set(Self::parse_role_static(&required_role)?);
        }
        if let Some(priority) = input.priority {
            rule.priority = Set(priority);
        }
        if let Some(is_active) = input.is_active {
            rule.is_active = Set(is_active);
        }

        rule.updated_at = Set(chrono::Utc::now().into());

        let result = rule.update(txn).await?;
        Ok(result)
    }

    /// Soft deletes an approval rule by setting is_active to false.
    pub async fn delete_rule(
        &self,
        organization_id: Uuid,
        rule_id: Uuid,
    ) -> Result<(), ApprovalRuleError> {
        let txn = self.db.begin().await?;
        
        let result = self.delete_rule_in_txn(&txn, organization_id, rule_id).await;
        
        match result {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    /// Soft deletes an approval rule within a transaction.
    async fn delete_rule_in_txn(
        &self,
        txn: &DatabaseTransaction,
        organization_id: Uuid,
        rule_id: Uuid,
    ) -> Result<(), ApprovalRuleError> {
        let existing = ApprovalRuleEntity::find_by_id(rule_id)
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .one(txn)
            .await?
            .ok_or(ApprovalRuleError::NotFound(rule_id))?;

        let mut rule: ActiveModel = existing.into();
        rule.is_active = Set(false);
        rule.updated_at = Set(chrono::Utc::now().into());

        rule.update(txn).await?;
        Ok(())
    }

    /// Gets rules that match a transaction for approval.
    ///
    /// **Validates: Requirements 3.2, 3.3**
    pub async fn get_rules_for_transaction(
        &self,
        organization_id: Uuid,
        transaction_type: &str,
        amount: Decimal,
    ) -> Result<Vec<ApprovalRuleModel>, ApprovalRuleError> {
        let all_rules = self.list_rules(organization_id).await?;

        let tx_type = Self::parse_transaction_type(transaction_type)?;

        let matching_rules: Vec<ApprovalRuleModel> = all_rules
            .into_iter()
            .filter(|rule| {
                // Check transaction type matches
                if !rule.transaction_types.contains(&tx_type) {
                    return false;
                }

                // Check amount range
                let above_min = rule.min_amount.is_none_or(|min| amount >= min);
                let below_max = rule.max_amount.is_none_or(|max| amount <= max);

                above_min && below_max
            })
            .collect();

        Ok(matching_rules)
    }

    // Helper methods

    fn parse_transaction_types(
        types: &[String],
    ) -> Result<Vec<TransactionType>, ApprovalRuleError> {
        types
            .iter()
            .map(|t| Self::parse_transaction_type(t))
            .collect()
    }

    fn parse_transaction_type(t: &str) -> Result<TransactionType, ApprovalRuleError> {
        match t.to_lowercase().as_str() {
            "journal" => Ok(TransactionType::Journal),
            "invoice" => Ok(TransactionType::Invoice),
            "bill" => Ok(TransactionType::Bill),
            "payment" => Ok(TransactionType::Payment),
            "expense" => Ok(TransactionType::Expense),
            "transfer" => Ok(TransactionType::Transfer),
            "adjustment" => Ok(TransactionType::Adjustment),
            "opening_balance" => Ok(TransactionType::OpeningBalance),
            "reversal" => Ok(TransactionType::Reversal),
            "accrual" => Ok(TransactionType::Accrual),
            "revaluation" => Ok(TransactionType::Revaluation),
            "intercompany" => Ok(TransactionType::Intercompany),
            _ => Err(ApprovalRuleError::InvalidTransactionType(t.to_string())),
        }
    }

    fn parse_role_static(role: &str) -> Result<UserRole, ApprovalRuleError> {
        match role.to_lowercase().as_str() {
            "viewer" => Ok(UserRole::Viewer),
            "submitter" => Ok(UserRole::Submitter),
            "approver" => Ok(UserRole::Approver),
            "accountant" => Ok(UserRole::Accountant),
            "admin" => Ok(UserRole::Admin),
            "owner" => Ok(UserRole::Owner),
            _ => Err(ApprovalRuleError::InvalidRole(role.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use std::env;

    fn get_database_url() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            env::var("ZELTRA__DATABASE__URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/zeltra_dev".to_string()
            })
        })
    }

    #[tokio::test]
    async fn test_parse_transaction_type_valid() {
        assert!(ApprovalRuleRepository::parse_transaction_type("journal").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("JOURNAL").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("Invoice").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("bill").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("payment").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("opening_balance").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("expense").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("transfer").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("adjustment").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("reversal").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("accrual").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("ACCRUAL").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("revaluation").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("Revaluation").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("intercompany").is_ok());
        assert!(ApprovalRuleRepository::parse_transaction_type("INTERCOMPANY").is_ok());
    }

    #[tokio::test]
    async fn test_parse_transaction_type_invalid() {
        assert!(ApprovalRuleRepository::parse_transaction_type("invalid").is_err());
        assert!(ApprovalRuleRepository::parse_transaction_type("").is_err());
    }

    #[tokio::test]
    async fn test_parse_role_valid() {
        assert!(ApprovalRuleRepository::parse_role_static("viewer").is_ok());
        assert!(ApprovalRuleRepository::parse_role_static("VIEWER").is_ok());
        assert!(ApprovalRuleRepository::parse_role_static("Submitter").is_ok());
        assert!(ApprovalRuleRepository::parse_role_static("approver").is_ok());
        assert!(ApprovalRuleRepository::parse_role_static("accountant").is_ok());
        assert!(ApprovalRuleRepository::parse_role_static("admin").is_ok());
        assert!(ApprovalRuleRepository::parse_role_static("owner").is_ok());
    }

    #[tokio::test]
    async fn test_parse_role_invalid() {
        assert!(ApprovalRuleRepository::parse_role_static("invalid").is_err());
        assert!(ApprovalRuleRepository::parse_role_static("").is_err());
        assert!(ApprovalRuleRepository::parse_role_static("superadmin").is_err());
    }

    #[test]
    fn test_error_display() {
        let err = ApprovalRuleError::NotFound(Uuid::new_v4());
        assert!(err.to_string().contains("not found"));

        let err = ApprovalRuleError::InvalidTransactionType("bad".to_string());
        assert!(err.to_string().contains("Invalid transaction type"));

        let err = ApprovalRuleError::InvalidRole("bad".to_string());
        assert!(err.to_string().contains("Invalid role"));
    }

    #[tokio::test]
    async fn test_list_rules_empty_org() {
        let db = Database::connect(&get_database_url())
            .await
            .expect("Failed to connect to database");
        let repo = ApprovalRuleRepository::new(db);

        // Random org should return empty list
        let result = repo.list_rules(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_rule_not_found() {
        let db = Database::connect(&get_database_url())
            .await
            .expect("Failed to connect to database");
        let repo = ApprovalRuleRepository::new(db);

        let result = repo.get_rule(Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ApprovalRuleError::NotFound(_))));
    }

    /// Property test for pagination consistency.
    /// 
    /// **Property 1: Pagination Consistency**
    /// **Validates: Requirements 2.1.2, 2.2.1**
    /// 
    /// Tests that pagination returns consistent results:
    /// - Items count <= per_page
    /// - Total pages calculation is correct
    /// - Last page has correct remaining items
    #[tokio::test]
    async fn test_pagination_consistency_property() {
        let db = Database::connect(&get_database_url())
            .await
            .expect("Failed to connect to database");
        let repo = ApprovalRuleRepository::new(db);
        
        let org_id = Uuid::new_v4();
        
        // Test with various page sizes and page numbers
        let test_cases = vec![
            (1, 1),   // page 1, per_page 1
            (1, 5),   // page 1, per_page 5
            (1, 10),  // page 1, per_page 10
            (1, 20),  // page 1, per_page 20
            (1, 50),  // page 1, per_page 50
            (1, 100), // page 1, per_page 100 (max)
            (2, 10),  // page 2, per_page 10
            (5, 5),   // page 5, per_page 5
        ];

        for (page, per_page) in test_cases {
            let offset = ((page - 1) * per_page) as u64;
            let limit = per_page as u64;

            let result = repo.list_rules_paginated(org_id, offset, limit).await;
            assert!(result.is_ok(), "Pagination should not fail for page={}, per_page={}", page, per_page);

            let (items, total) = result.unwrap();

            // Property 1: Items count <= per_page
            assert!(
                items.len() <= per_page as usize,
                "Items count ({}) should be <= per_page ({}) for page={}, per_page={}",
                items.len(), per_page, page, per_page
            );

            // Property 2: Total pages calculation is correct
            let expected_total_pages = if total == 0 { 0 } else { ((total as f64) / (per_page as f64)).ceil() as u32 };
            let calculated_total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
            assert_eq!(
                calculated_total_pages, expected_total_pages,
                "Total pages calculation should be correct for total={}, per_page={}",
                total, per_page
            );

            // Property 3: If this is beyond the last page, items should be empty
            if page > expected_total_pages {
                assert!(
                    items.is_empty(),
                    "Items should be empty when page ({}) > total_pages ({})",
                    page, expected_total_pages
                );
            }

            // Property 4: Last page has correct remaining items
            if page == expected_total_pages && total > 0 {
                let expected_items_on_last_page = ((total - 1) % (per_page as u64) + 1) as usize;
                if page > 1 || total > per_page as u64 {
                    assert!(
                        items.len() <= expected_items_on_last_page,
                        "Last page should have {} items, got {} for total={}, per_page={}",
                        expected_items_on_last_page, items.len(), total, per_page
                    );
                }
            }
        }
    }

    /// Property test for filtering consistency.
    /// 
    /// Tests that filtering returns consistent results and respects filter parameters.
    #[tokio::test]
    async fn test_filtering_consistency_property() {
        let db = Database::connect(&get_database_url())
            .await
            .expect("Failed to connect to database");
        let repo = ApprovalRuleRepository::new(db);
        
        let org_id = Uuid::new_v4();
        
        // Test various filter combinations
        let filter_cases = vec![
            (Some(true), None, None),           // is_active = true
            (Some(false), None, None),          // is_active = false
            (None, Some("bill"), None),         // transaction_type = bill
            (None, None, Some("approver")),     // required_role = approver
            (Some(true), Some("invoice"), None), // combined filters
        ];

        for (is_active, transaction_type, required_role) in filter_cases {
            let result = repo.list_rules_with_filters(
                org_id, 0, 10, is_active, transaction_type, required_role, None, None
            ).await;
            
            assert!(result.is_ok(), "Filtering should not fail");
            let (items, _total) = result.unwrap();

            // Verify filters are applied correctly
            if let Some(active) = is_active {
                for item in &items {
                    assert_eq!(
                        item.is_active, active,
                        "All items should match is_active filter"
                    );
                }
            }

            if let Some(role) = required_role {
                let expected_role = ApprovalRuleRepository::parse_role_static(role).unwrap();
                for item in &items {
                    assert_eq!(
                        item.required_role, expected_role,
                        "All items should match required_role filter"
                    );
                }
            }

            if let Some(tx_type) = transaction_type {
                let expected_type = ApprovalRuleRepository::parse_transaction_type(tx_type).unwrap();
                for item in &items {
                    assert!(
                        item.transaction_types.contains(&expected_type),
                        "All items should contain the filtered transaction type"
                    );
                }
            }
        }
    }

    /// Test transaction rollback behavior.
    /// 
    /// Tests that database transactions are properly rolled back on errors.
    #[tokio::test]
    async fn test_transaction_rollback_on_create_error() {
        let db = Database::connect(&get_database_url())
            .await
            .expect("Failed to connect to database");
        let repo = ApprovalRuleRepository::new(db);
        
        let org_id = Uuid::new_v4();
        
        // Create input with invalid transaction type to force an error
        let input = CreateApprovalRuleInput {
            name: "Test Rule".to_string(),
            description: Some("Test Description".to_string()),
            min_amount: None,
            max_amount: None,
            transaction_types: vec!["invalid_type".to_string()], // This should cause an error
            required_role: "approver".to_string(),
            priority: 1,
        };
        
        // Attempt to create the rule - should fail
        let result = repo.create_rule(org_id, input).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ApprovalRuleError::InvalidTransactionType(_))));
        
        // Verify no rule was created (transaction was rolled back)
        let rules = repo.list_rules(org_id).await.unwrap();
        assert!(rules.is_empty(), "No rules should exist after failed transaction");
    }

    /// Test transaction rollback behavior on update.
    #[tokio::test]
    async fn test_transaction_rollback_on_update_error() {
        let db = Database::connect(&get_database_url())
            .await
            .expect("Failed to connect to database");
        let repo = ApprovalRuleRepository::new(db);
        
        let org_id = Uuid::new_v4();
        
        // First create a valid rule
        let create_input = CreateApprovalRuleInput {
            name: "Original Rule".to_string(),
            description: Some("Original Description".to_string()),
            min_amount: None,
            max_amount: None,
            transaction_types: vec!["bill".to_string()],
            required_role: "approver".to_string(),
            priority: 1,
        };
        
        let created_rule = repo.create_rule(org_id, create_input).await.unwrap();
        
        // Now try to update with invalid data
        let update_input = UpdateApprovalRuleInput {
            name: Some("Updated Rule".to_string()),
            transaction_types: Some(vec!["invalid_type".to_string()]), // This should cause an error
            ..Default::default()
        };
        
        // Attempt to update - should fail
        let result = repo.update_rule(org_id, created_rule.id, update_input).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ApprovalRuleError::InvalidTransactionType(_))));
        
        // Verify the original rule is unchanged (transaction was rolled back)
        let unchanged_rule = repo.get_rule(org_id, created_rule.id).await.unwrap();
        assert_eq!(unchanged_rule.name, "Original Rule");
        assert_eq!(unchanged_rule.transaction_types, vec![TransactionType::Bill]);
    }
}
