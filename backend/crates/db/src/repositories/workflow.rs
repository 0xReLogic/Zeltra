//! Workflow repository for transaction state transitions.
//!
//! Implements Requirements 1.1-1.4, 2.1-2.7, 5.1-5.4 for transaction workflow management.

use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use uuid::Uuid;

use zeltra_core::workflow::{
    ApprovalEngine, ApprovalRule, OriginalEntry, ReversalInput, ReversalService, WorkflowError,
    WorkflowService,
};

use crate::entities::{
    approval_rules, chart_of_accounts, entry_dimensions, ledger_entries, organization_users,
    sea_orm_active_enums::{TransactionStatus, TransactionType},
    transactions,
};

use super::transaction::calculate_balance_change;

/// Result of a bulk approval operation.
#[derive(Debug, Clone)]
pub struct BulkApproveResult {
    /// Results for each transaction.
    pub results: Vec<BulkApproveItemResult>,
    /// Number of successful approvals.
    pub success_count: usize,
    /// Number of failed approvals.
    pub failure_count: usize,
}

/// Result for a single transaction in bulk approval.
#[derive(Debug, Clone)]
pub struct BulkApproveItemResult {
    /// Transaction ID.
    pub transaction_id: Uuid,
    /// Whether the approval succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Pending transaction with approval info.
#[derive(Debug, Clone)]
pub struct PendingTransaction {
    /// Transaction data.
    pub transaction: transactions::Model,
    /// Whether the current user can approve this transaction.
    pub can_approve: bool,
}

/// Void operation result.
#[derive(Debug, Clone)]
pub struct VoidResult {
    /// Original transaction (now voided).
    pub original_transaction: transactions::Model,
    /// Reversing transaction (posted).
    pub reversing_transaction: transactions::Model,
}

/// Workflow repository for transaction state transitions.
#[derive(Debug, Clone)]
pub struct WorkflowRepository {
    db: DatabaseConnection,
}

impl WorkflowRepository {
    /// Creates a new workflow repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Submits a draft transaction for approval.
    ///
    /// Requirements: 1.1, 7.2
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in draft status
    /// - Database operation fails
    pub async fn submit_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
        submitted_by: Uuid,
    ) -> Result<transactions::Model, WorkflowError> {
        // Fetch transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?
            .ok_or(WorkflowError::TransactionNotFound(transaction_id))?;

        // Convert DB status to core status
        let current_status = db_status_to_core(&transaction.status);

        // Validate transition using WorkflowService
        let _action = WorkflowService::submit(current_status, submitted_by)?;

        // Update transaction
        let now = Utc::now().into();
        let mut active: transactions::ActiveModel = transaction.into();
        active.status = Set(TransactionStatus::Pending);
        active.submitted_at = Set(Some(now));
        active.submitted_by = Set(Some(submitted_by));
        active.updated_at = Set(now);

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        Ok(updated)
    }

    /// Approves a pending transaction.
    ///
    /// Requirements: 1.2, 3.4, 3.5, 7.3
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in pending status
    /// - User is not authorized to approve
    /// - Database operation fails
    pub async fn approve_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
        approved_by: Uuid,
        approval_notes: Option<String>,
    ) -> Result<transactions::Model, WorkflowError> {
        // Fetch transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?
            .ok_or(WorkflowError::TransactionNotFound(transaction_id))?;

        // Convert DB status to core status
        let current_status = db_status_to_core(&transaction.status);

        // Validate transition using WorkflowService
        let _action =
            WorkflowService::approve(current_status, approved_by, approval_notes.clone())?;

        // Check user authorization
        self.check_approval_authorization(
            organization_id,
            approved_by,
            &transaction.transaction_type,
            self.calculate_transaction_total(transaction_id).await?,
        )
        .await?;

        // Update transaction
        let now = Utc::now().into();
        let mut active: transactions::ActiveModel = transaction.into();
        active.status = Set(TransactionStatus::Approved);
        active.approved_at = Set(Some(now));
        active.approved_by = Set(Some(approved_by));
        active.approval_notes = Set(approval_notes);
        active.updated_at = Set(now);

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        Ok(updated)
    }

    /// Rejects a pending transaction back to draft.
    ///
    /// Requirements: 1.3, 7.4
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in pending status
    /// - Rejection reason is empty
    /// - Database operation fails
    pub async fn reject_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
        rejection_reason: String,
    ) -> Result<transactions::Model, WorkflowError> {
        // Fetch transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?
            .ok_or(WorkflowError::TransactionNotFound(transaction_id))?;

        // Convert DB status to core status
        let current_status = db_status_to_core(&transaction.status);

        // Validate transition using WorkflowService
        let _action = WorkflowService::reject(current_status, rejection_reason.clone())?;

        // Update transaction
        let now = Utc::now().into();
        let mut active: transactions::ActiveModel = transaction.into();
        active.status = Set(TransactionStatus::Draft);
        active.approval_notes = Set(Some(rejection_reason));
        active.submitted_at = Set(None);
        active.submitted_by = Set(None);
        active.updated_at = Set(now);

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        Ok(updated)
    }

    /// Posts an approved transaction.
    ///
    /// Requirements: 1.4, 7.5
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in approved status
    /// - Database operation fails
    pub async fn post_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
        posted_by: Uuid,
    ) -> Result<transactions::Model, WorkflowError> {
        // Fetch transaction
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?
            .ok_or(WorkflowError::TransactionNotFound(transaction_id))?;

        // Convert DB status to core status
        let current_status = db_status_to_core(&transaction.status);

        // Validate transition using WorkflowService
        let _action = WorkflowService::post(current_status, posted_by)?;

        // Update transaction
        let now = Utc::now().into();
        let mut active: transactions::ActiveModel = transaction.into();
        active.status = Set(TransactionStatus::Posted);
        active.posted_at = Set(Some(now));
        active.posted_by = Set(Some(posted_by));
        active.updated_at = Set(now);

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        Ok(updated)
    }

    /// Voids a posted transaction by creating a reversing entry.
    ///
    /// Requirements: 2.1-2.7, 7.6
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Transaction is not found
    /// - Transaction is not in posted status
    /// - Void reason is empty
    /// - Database operation fails
    #[allow(clippy::too_many_lines)]
    pub async fn void_transaction(
        &self,
        organization_id: Uuid,
        transaction_id: Uuid,
        voided_by: Uuid,
        void_reason: String,
    ) -> Result<VoidResult, WorkflowError> {
        // Fetch transaction with entries
        let transaction = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?
            .ok_or(WorkflowError::TransactionNotFound(transaction_id))?;

        // Convert DB status to core status
        let current_status = db_status_to_core(&transaction.status);

        // Validate transition using WorkflowService
        let _action = WorkflowService::void(current_status, voided_by, void_reason.clone())?;

        // Fetch ledger entries
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::TransactionId.eq(transaction_id))
            .all(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        // Convert to OriginalEntry for ReversalService
        let original_entries: Vec<OriginalEntry> = entries
            .iter()
            .map(|e| OriginalEntry {
                account_id: e.account_id,
                source_currency: e.source_currency.clone(),
                source_amount: e.source_amount,
                exchange_rate: e.exchange_rate,
                functional_amount: e.functional_amount,
                debit: e.debit,
                credit: e.credit,
                memo: e.memo.clone(),
                dimensions: vec![], // Will fetch separately
            })
            .collect();

        // Create reversal input
        let reversal_input = ReversalInput {
            original_transaction_id: transaction_id,
            original_entries,
            fiscal_period_id: transaction.fiscal_period_id,
            voided_by,
            void_reason: void_reason.clone(),
        };

        // Generate reversing entries
        let reversal_output = ReversalService::create_reversing_entries(&reversal_input);

        // Begin database transaction
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        let now = Utc::now().into();

        // Create reversing transaction
        let reversing_tx_id = reversal_output.reversing_transaction_id;
        let reversing_transaction = transactions::ActiveModel {
            id: Set(reversing_tx_id),
            organization_id: Set(organization_id),
            fiscal_period_id: Set(transaction.fiscal_period_id),
            reference_number: Set(transaction.reference_number.clone()),
            transaction_type: Set(TransactionType::Reversal),
            transaction_date: Set(transaction.transaction_date),
            description: Set(reversal_output.description),
            memo: Set(Some(format!("Void reason: {void_reason}"))),
            status: Set(TransactionStatus::Posted),
            created_by: Set(voided_by),
            submitted_at: Set(Some(now)),
            submitted_by: Set(Some(voided_by)),
            approved_at: Set(Some(now)),
            approved_by: Set(Some(voided_by)),
            posted_at: Set(Some(now)),
            posted_by: Set(Some(voided_by)),
            reverses_transaction_id: Set(Some(transaction_id)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let reversing_tx = reversing_transaction
            .insert(&txn)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        // Insert reversing ledger entries with balance tracking
        for (idx, rev_entry) in reversal_output.reversing_entries.iter().enumerate() {
            let original_entry = &entries[idx];

            // Get account for balance calculation
            let account = chart_of_accounts::Entity::find_by_id(rev_entry.account_id)
                .one(&txn)
                .await
                .map_err(|e| WorkflowError::Database(e.to_string()))?
                .ok_or_else(|| {
                    WorkflowError::Database(format!("Account not found: {}", rev_entry.account_id))
                })?;

            // Get latest balance for this account
            let latest_entry = ledger_entries::Entity::find()
                .filter(ledger_entries::Column::AccountId.eq(rev_entry.account_id))
                .order_by_desc(ledger_entries::Column::AccountVersion)
                .one(&txn)
                .await
                .map_err(|e| WorkflowError::Database(e.to_string()))?;

            let (prev_version, prev_balance) = match latest_entry {
                Some(e) => (e.account_version, e.account_current_balance),
                None => (0, Decimal::ZERO),
            };

            // Calculate balance change (reversed)
            let (debit, credit) = match rev_entry.entry_type {
                zeltra_core::ledger::types::EntryType::Debit => {
                    (rev_entry.source_amount, Decimal::ZERO)
                }
                zeltra_core::ledger::types::EntryType::Credit => {
                    (Decimal::ZERO, rev_entry.source_amount)
                }
            };

            let balance_change = calculate_balance_change(&account.account_type, debit, credit);
            let current_balance = prev_balance + balance_change;

            let entry_id = Uuid::new_v4();
            let entry = ledger_entries::ActiveModel {
                id: Set(entry_id),
                transaction_id: Set(reversing_tx_id),
                account_id: Set(rev_entry.account_id),
                source_currency: Set(rev_entry.source_currency.clone()),
                source_amount: Set(rev_entry.source_amount),
                exchange_rate: Set(original_entry.exchange_rate),
                functional_currency: Set(original_entry.functional_currency.clone()),
                functional_amount: Set(rev_entry.source_amount),
                debit: Set(debit),
                credit: Set(credit),
                memo: Set(rev_entry.memo.clone()),
                event_at: Set(now),
                created_at: Set(now),
                account_version: Set(prev_version + 1),
                account_previous_balance: Set(prev_balance),
                account_current_balance: Set(current_balance),
            };

            entry
                .insert(&txn)
                .await
                .map_err(|e| WorkflowError::Database(e.to_string()))?;

            // Copy dimensions from original entry
            let dims = entry_dimensions::Entity::find()
                .filter(entry_dimensions::Column::LedgerEntryId.eq(original_entry.id))
                .all(&txn)
                .await
                .map_err(|e| WorkflowError::Database(e.to_string()))?;

            for dim in dims {
                let new_dim = entry_dimensions::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    ledger_entry_id: Set(entry_id),
                    dimension_value_id: Set(dim.dimension_value_id),
                    created_at: Set(now),
                };
                new_dim
                    .insert(&txn)
                    .await
                    .map_err(|e| WorkflowError::Database(e.to_string()))?;
            }
        }

        // Update original transaction to voided
        let mut original_active: transactions::ActiveModel = transaction.into();
        original_active.status = Set(TransactionStatus::Voided);
        original_active.voided_at = Set(Some(now));
        original_active.voided_by = Set(Some(voided_by));
        original_active.void_reason = Set(Some(void_reason));
        original_active.reversed_by_transaction_id = Set(Some(reversing_tx_id));
        original_active.updated_at = Set(now);

        let voided_tx = original_active
            .update(&txn)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        // Commit transaction
        txn.commit()
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        Ok(VoidResult {
            original_transaction: voided_tx,
            reversing_transaction: reversing_tx,
        })
    }

    /// Gets pending transactions that the user can approve.
    ///
    /// Requirements: 5.1
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_pending_transactions(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<PendingTransaction>, WorkflowError> {
        // Get user's role and approval limit
        let org_user = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(organization_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        let (user_role, approval_limit) = match org_user {
            Some(ou) => (db_role_to_string(&ou.role), ou.approval_limit),
            None => return Ok(vec![]), // User not in organization
        };

        // Fetch all pending transactions
        let pending = transactions::Entity::find()
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .filter(transactions::Column::Status.eq(TransactionStatus::Pending))
            .order_by_desc(transactions::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        // Fetch approval rules
        let rules = self.get_approval_rules(organization_id).await?;

        // Check each transaction
        let mut result = Vec::with_capacity(pending.len());
        for tx in pending {
            let total = self.calculate_transaction_total(tx.id).await?;
            let tx_type = db_tx_type_to_string(&tx.transaction_type);

            // Get required role for this transaction
            let required_role = ApprovalEngine::get_required_approval(&rules, &tx_type, total);

            let can_approve = match required_role {
                Some(role) => {
                    ApprovalEngine::can_approve(&user_role, approval_limit, &role, total).is_ok()
                }
                None => {
                    // No rule matches, default to Approver role
                    ApprovalEngine::can_approve(&user_role, approval_limit, "approver", total)
                        .is_ok()
                }
            };

            result.push(PendingTransaction {
                transaction: tx,
                can_approve,
            });
        }

        Ok(result)
    }

    /// Bulk approves multiple transactions.
    ///
    /// Requirements: 5.2, 5.3, 5.4
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn bulk_approve(
        &self,
        organization_id: Uuid,
        transaction_ids: Vec<Uuid>,
        approved_by: Uuid,
        approval_notes: Option<String>,
    ) -> Result<BulkApproveResult, WorkflowError> {
        let mut results = Vec::with_capacity(transaction_ids.len());
        let mut success_count = 0;
        let mut failure_count = 0;

        for tx_id in transaction_ids {
            match self
                .approve_transaction(organization_id, tx_id, approved_by, approval_notes.clone())
                .await
            {
                Ok(_) => {
                    success_count += 1;
                    results.push(BulkApproveItemResult {
                        transaction_id: tx_id,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    failure_count += 1;
                    results.push(BulkApproveItemResult {
                        transaction_id: tx_id,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(BulkApproveResult {
            results,
            success_count,
            failure_count,
        })
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    /// Checks if a user is authorized to approve a transaction.
    async fn check_approval_authorization(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        transaction_type: &TransactionType,
        amount: Decimal,
    ) -> Result<(), WorkflowError> {
        // Get user's role and approval limit
        let org_user = organization_users::Entity::find()
            .filter(organization_users::Column::OrganizationId.eq(organization_id))
            .filter(organization_users::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?
            .ok_or(WorkflowError::NotAuthorizedToApprove)?;

        let user_role = db_role_to_string(&org_user.role);
        let approval_limit = org_user.approval_limit;

        // Get approval rules
        let rules = self.get_approval_rules(organization_id).await?;
        let tx_type = db_tx_type_to_string(transaction_type);

        // Get required role
        let required_role = ApprovalEngine::get_required_approval(&rules, &tx_type, amount)
            .unwrap_or_else(|| "approver".to_string());

        // Check authorization
        ApprovalEngine::can_approve(&user_role, approval_limit, &required_role, amount)
    }

    /// Gets approval rules for an organization.
    async fn get_approval_rules(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<ApprovalRule>, WorkflowError> {
        let db_rules = approval_rules::Entity::find()
            .filter(approval_rules::Column::OrganizationId.eq(organization_id))
            .filter(approval_rules::Column::IsActive.eq(true))
            .order_by_asc(approval_rules::Column::Priority)
            .all(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        let rules = db_rules
            .into_iter()
            .map(|r| ApprovalRule {
                id: r.id,
                name: r.name,
                min_amount: r.min_amount,
                max_amount: r.max_amount,
                transaction_types: r
                    .transaction_types
                    .iter()
                    .map(db_tx_type_to_string)
                    .collect(),
                required_role: db_role_to_string(&r.required_role),
                priority: r.priority,
            })
            .collect();

        Ok(rules)
    }

    /// Calculates the total amount of a transaction.
    async fn calculate_transaction_total(
        &self,
        transaction_id: Uuid,
    ) -> Result<Decimal, WorkflowError> {
        let entries = ledger_entries::Entity::find()
            .filter(ledger_entries::Column::TransactionId.eq(transaction_id))
            .all(&self.db)
            .await
            .map_err(|e| WorkflowError::Database(e.to_string()))?;

        // Sum all debits (or credits, they should be equal for balanced transactions)
        let total: Decimal = entries.iter().map(|e| e.debit).sum();

        Ok(total)
    }
}

// ============================================================================
// Conversion helpers
// ============================================================================

/// Converts database TransactionStatus to core TransactionStatus.
fn db_status_to_core(
    status: &TransactionStatus,
) -> zeltra_core::workflow::types::TransactionStatus {
    match status {
        TransactionStatus::Draft => zeltra_core::workflow::types::TransactionStatus::Draft,
        TransactionStatus::Pending => zeltra_core::workflow::types::TransactionStatus::Pending,
        TransactionStatus::Approved => zeltra_core::workflow::types::TransactionStatus::Approved,
        TransactionStatus::Posted => zeltra_core::workflow::types::TransactionStatus::Posted,
        TransactionStatus::Voided => zeltra_core::workflow::types::TransactionStatus::Voided,
    }
}

/// Converts database UserRole to string.
fn db_role_to_string(role: &crate::entities::sea_orm_active_enums::UserRole) -> String {
    match role {
        crate::entities::sea_orm_active_enums::UserRole::Owner => "owner".to_string(),
        crate::entities::sea_orm_active_enums::UserRole::Admin => "admin".to_string(),
        crate::entities::sea_orm_active_enums::UserRole::Accountant => "accountant".to_string(),
        crate::entities::sea_orm_active_enums::UserRole::Approver => "approver".to_string(),
        crate::entities::sea_orm_active_enums::UserRole::Viewer => "viewer".to_string(),
        crate::entities::sea_orm_active_enums::UserRole::Submitter => "submitter".to_string(),
    }
}

/// Converts database TransactionType to string.
fn db_tx_type_to_string(tx_type: &TransactionType) -> String {
    match tx_type {
        TransactionType::Journal => "journal".to_string(),
        TransactionType::Expense => "expense".to_string(),
        TransactionType::Invoice => "invoice".to_string(),
        TransactionType::Bill => "bill".to_string(),
        TransactionType::Payment => "payment".to_string(),
        TransactionType::Transfer => "transfer".to_string(),
        TransactionType::Adjustment => "adjustment".to_string(),
        TransactionType::OpeningBalance => "opening_balance".to_string(),
        TransactionType::Reversal => "reversal".to_string(),
    }
}
