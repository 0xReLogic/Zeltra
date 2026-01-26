//! Intercompany repository for cross-entity transaction database operations.

use crate::entities::{intercompany_mappings, ledger_entries, transactions};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QuerySelect, QueryTrait, Set, prelude::Uuid,
};

/// Error types for intercompany operations.
#[derive(Debug, thiserror::Error)]
pub enum IntercompanyError {
    /// Mapping not found.
    #[error("Intercompany mapping not found")]
    MappingNotFound,

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Repository for intercompany operations.
#[derive(Debug, Clone)]
pub struct IntercompanyRepository {
    db: DatabaseConnection,
}

impl IntercompanyRepository {
    /// Creates a new intercompany repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Finds all mappings for a source entity.
    pub async fn get_mappings(
        &self,
        source_entity_id: Uuid,
    ) -> Result<Vec<intercompany_mappings::Model>, IntercompanyError> {
        let mappings = intercompany_mappings::Entity::find()
            .filter(intercompany_mappings::Column::SourceEntityId.eq(source_entity_id))
            .all(&self.db)
            .await?;
        Ok(mappings)
    }

    /// Finds a specific mapping by source account.
    pub async fn find_mapping_by_account(
        &self,
        source_entity_id: Uuid,
        source_account_id: Uuid,
    ) -> Result<Option<intercompany_mappings::Model>, IntercompanyError> {
        let mapping = intercompany_mappings::Entity::find()
            .filter(intercompany_mappings::Column::SourceEntityId.eq(source_entity_id))
            .filter(intercompany_mappings::Column::SourceAccountId.eq(source_account_id))
            .one(&self.db)
            .await?;
        Ok(mapping)
    }

    /// Validates that two entities can have an intercompany mapping.
    /// Both entities must exist and belong to the same organization.
    pub async fn validate_mapping(
        &self,
        source_entity_id: Uuid,
        target_entity_id: Uuid,
    ) -> Result<(), IntercompanyError> {
        use crate::entities::entities;

        // Get both entities
        let source = entities::Entity::find_by_id(source_entity_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                IntercompanyError::Database(DbErr::Custom("Source entity not found".to_string()))
            })?;

        let target = entities::Entity::find_by_id(target_entity_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                IntercompanyError::Database(DbErr::Custom("Target entity not found".to_string()))
            })?;

        // Both entities must be in same organization
        if source.organization_id != target.organization_id {
            return Err(IntercompanyError::Database(DbErr::Custom(
                "Entities must belong to the same organization".to_string(),
            )));
        }

        Ok(())
    }

    /// Identifies ledger entries that hit intercompany accounts and might need mirroring or elimination.
    pub async fn get_pending_intercompany_entries(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<(ledger_entries::Model, intercompany_mappings::Model)>, IntercompanyError> {
        let mut result_tuples = Vec::new();

        // Get all intercompany mappings for this organization
        let mappings = self.get_mappings(organization_id).await?;

        for mapping in mappings {
            // Find entries for the source account that are in posted transactions
            let entries = ledger_entries::Entity::find()
                .filter(ledger_entries::Column::AccountId.eq(mapping.source_account_id))
                .filter(
                    ledger_entries::Column::TransactionId.in_subquery(
                        transactions::Entity::find()
                            .select_only()
                            .column(transactions::Column::Id)
                            .filter(transactions::Column::Status.eq(
                                crate::entities::sea_orm_active_enums::TransactionStatus::Posted,
                            ))
                            .into_query(),
                    ),
                )
                .all(&self.db)
                .await?;

            for entry in entries {
                result_tuples.push((entry, mapping.clone()));
            }
        }

        Ok(result_tuples)
    }

    /// Processes pending intercompany entries for mirroring and elimination.
    pub async fn process_intercompany_entries(
        &self,
        organization_id: Uuid,
        tx_repo: &super::transaction::TransactionRepository,
    ) -> Result<usize, IntercompanyError> {
        use zeltra_core::ledger::intercompany::IntercompanyEngine;
        use zeltra_core::ledger::types::EntryType as CoreEntryType;

        let pending = self
            .get_pending_intercompany_entries(organization_id)
            .await?;
        let mut processed = 0;

        for (entry, mapping) in pending {
            // Check if already processed (check compliance_metadata)
            if entry
                .compliance_metadata
                .as_ref()
                .and_then(|m| m.get("intercompany_processed"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }

            if mapping.auto_post {
                // Generate mirror transaction in target organization
                let core_entry_type = if entry.debit > rust_decimal::Decimal::ZERO {
                    CoreEntryType::Debit
                } else {
                    CoreEntryType::Credit
                };

                // For a mirror, we need a balancing account.
                // For now, we use the target account itself as the primary,
                // but we need a 'suspense' or 'offset' account in the target org.
                // In a real setup, this would be part of the mapping or a default.
                // We'll assume the target account is where the mirror goes,
                // but mirror_transaction needs a balancing account.
                // Let's use the same account for now as a placeholder or skip if not fully defined.

                // TODO: In a production scenario, balancing_account_id would be configurable.
                let balancing_account_id = mapping.target_account_id;

                use zeltra_core::ledger::intercompany::MirrorTransactionInput;
                let mirror_input = MirrorTransactionInput {
                    target_org_id: mapping.target_org_id,
                    target_account_id: mapping.target_account_id,
                    balancing_account_id,
                    source_currency: entry.source_currency.clone(),
                    amount: entry.source_amount,
                    date: entry.created_at.date_naive(),
                    source_entry_type: core_entry_type,
                    reference: format!("Ref: {id}", id = entry.transaction_id),
                };

                let tx_input = IntercompanyEngine::generate_mirror_transaction(&mirror_input);

                // Map and post (similar to revaluation)
                let repo_tx_input = crate::repositories::transaction::CreateTransactionInput {
                    organization_id: tx_input.organization_id,
                    transaction_type:
                        crate::entities::sea_orm_active_enums::TransactionType::Intercompany,
                    transaction_date: tx_input.transaction_date,
                    description: tx_input.description,
                    reference_number: tx_input.reference_number,
                    memo: tx_input.memo,
                    entries: tx_input
                        .entries
                        .into_iter()
                        .map(|e| {
                            let fa = e.functional_amount.unwrap_or(entry.functional_amount);
                            let (debit, credit) = match e.entry_type {
                                zeltra_core::ledger::types::EntryType::Debit => {
                                    (fa, rust_decimal::Decimal::ZERO)
                                }
                                zeltra_core::ledger::types::EntryType::Credit => {
                                    (rust_decimal::Decimal::ZERO, fa)
                                }
                            };
                            crate::repositories::transaction::CreateLedgerEntryInput {
                                account_id: e.account_id,
                                source_currency: e.source_currency,
                                source_amount: e.source_amount,
                                exchange_rate: entry.exchange_rate, // Simple mirror assumes same rate
                                functional_currency: "USD".to_string(), // TODO: Get from target org
                                functional_amount: fa,
                                debit,
                                credit,
                                memo: e.memo,
                                compliance_metadata: e.compliance_metadata,
                                dimensions: e.dimensions,
                            }
                        })
                        .collect(),
                    created_by: Uuid::nil(),
                    timezone: "UTC".to_string(),
                    idempotency_key: Some(Uuid::new_v4()),
                    iso_metadata: None,
                };

                match tx_repo.create_transaction(repo_tx_input).await {
                    Ok(_) => {
                        // Mark entry as processed
                        let mut active: ledger_entries::ActiveModel = entry.into();
                        let mut meta = active
                            .compliance_metadata
                            .take()
                            .flatten()
                            .unwrap_or_else(|| serde_json::json!({}));
                        if let Some(obj) = meta.as_object_mut() {
                            obj.insert(
                                "intercompany_processed".to_string(),
                                serde_json::Value::Bool(true),
                            );
                            obj.insert(
                                "mirror_transaction_posted".to_string(),
                                serde_json::Value::Bool(true),
                            );
                        }
                        active.compliance_metadata = Set(Some(meta));
                        active.update(&self.db).await?;
                        processed += 1;
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(processed)
    }
}
