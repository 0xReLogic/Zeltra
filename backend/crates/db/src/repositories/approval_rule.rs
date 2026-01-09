//! Approval Rule Repository
//!
//! Provides CRUD operations for approval rules.
//!
//! **Validates: Requirements 3.1, 6.8, 6.9**

use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
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

        let result = rule.insert(&self.db).await?;
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
        let existing = self.get_rule(organization_id, rule_id).await?;

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

        let result = rule.update(&self.db).await?;
        Ok(result)
    }

    /// Soft deletes an approval rule by setting is_active to false.
    pub async fn delete_rule(
        &self,
        organization_id: Uuid,
        rule_id: Uuid,
    ) -> Result<(), ApprovalRuleError> {
        let existing = self.get_rule(organization_id, rule_id).await?;

        let mut rule: ActiveModel = existing.into();
        rule.is_active = Set(false);
        rule.updated_at = Set(chrono::Utc::now().into());

        rule.update(&self.db).await?;
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
}
