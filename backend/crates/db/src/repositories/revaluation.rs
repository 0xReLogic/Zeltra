//! Revaluation repository for currency revaluation database operations.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    QuerySelect, QueryTrait, Set, prelude::Expr,
};
use uuid::Uuid;

use crate::entities::{
    chart_of_accounts, ledger_entries, organizations, revaluation_logs,
    sea_orm_active_enums::TransactionStatus,
};

/// Error types for revaluation operations.
#[derive(Debug, thiserror::Error)]
pub enum RevaluationError {
    /// Organization not found.
    #[error("Organization not found")]
    OrganizationNotFound,

    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Input for creating a revaluation log.
#[derive(Debug, Clone)]
pub struct CreateRevaluationLogInput {
    pub organization_id: Uuid,
    pub entity_id: Uuid,
    pub account_id: Uuid,
    pub revaluation_date: NaiveDate,
    pub currency_id: String,
    pub balance_in_currency: Decimal,
    pub old_exchange_rate: Decimal,
    pub new_exchange_rate: Decimal,
    pub unrealized_gain_loss: Decimal,
    pub transaction_id: Option<Uuid>,
}

/// Repository for revaluation operations.
#[derive(Debug, Clone)]
pub struct RevaluationRepository {
    db: DatabaseConnection,
}

impl RevaluationRepository {
    /// Creates a new revaluation repository.
    #[must_use]
    pub const fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Finds accounts that are candidates for revaluation.
    pub async fn find_revaluation_candidates(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<chart_of_accounts::Model>, RevaluationError> {
        let org = organizations::Entity::find_by_id(organization_id)
            .one(&self.db)
            .await?
            .ok_or(RevaluationError::OrganizationNotFound)?;

        let accounts = chart_of_accounts::Entity::find()
            .filter(chart_of_accounts::Column::OrganizationId.eq(organization_id))
            .filter(chart_of_accounts::Column::Currency.ne(org.base_currency))
            .filter(chart_of_accounts::Column::IsActive.eq(true))
            .all(&self.db)
            .await?;

        Ok(accounts)
    }

    /// Generates a deterministic idempotency key for a revaluation event.
    fn generate_idempotency_key(&self, org_id: Uuid, account_id: Uuid, date: NaiveDate) -> Uuid {
        use sha2::{Digest, Sha256};
        let data = format!("reval-{org_id}-{account_id}-{date}");
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hash = hasher.finalize();
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&hash[..16]);
        Uuid::from_bytes(bytes)
    }

    /// Returns the carrying balances (functional and source) for an account as of a date.
    pub async fn get_carrying_balances(
        &self,
        account_id: Uuid,
        as_of: NaiveDate,
    ) -> Result<(Decimal, Decimal), RevaluationError> {
        use sea_orm::FromQueryResult;

        #[derive(Debug, FromQueryResult)]
        struct BalanceRow {
            total_functional: Option<Decimal>,
            total_source: Option<Decimal>,
        }

        let row = ledger_entries::Entity::find()
            .select_only()
            .column_as(Expr::cust("SUM(debit) - SUM(credit)"), "total_functional")
            .column_as(
                Expr::cust("SUM(CASE WHEN debit > 0 THEN source_amount ELSE -source_amount END)"),
                "total_source",
            )
            .filter(ledger_entries::Column::AccountId.eq(account_id))
            .filter(
                ledger_entries::Column::TransactionId.in_subquery(
                    crate::entities::transactions::Entity::find()
                        .select_only()
                        .column(crate::entities::transactions::Column::Id)
                        .filter(crate::entities::transactions::Column::TransactionDate.lte(as_of))
                        .filter(
                            crate::entities::transactions::Column::Status
                                .eq(TransactionStatus::Posted),
                        )
                        .into_query(),
                ),
            )
            .into_model::<BalanceRow>()
            .one(&self.db)
            .await?;

        if let Some(res) = row {
            Ok((
                res.total_functional.unwrap_or(Decimal::ZERO),
                res.total_source.unwrap_or(Decimal::ZERO),
            ))
        } else {
            Ok((Decimal::ZERO, Decimal::ZERO))
        }
    }

    /// Logs a revaluation event.
    pub async fn log_revaluation(
        &self,
        input: CreateRevaluationLogInput,
    ) -> Result<revaluation_logs::Model, RevaluationError> {
        let active = revaluation_logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(input.organization_id),
            entity_id: Set(input.entity_id),
            account_id: Set(input.account_id),
            revaluation_date: Set(input.revaluation_date),
            currency_id: Set(input.currency_id),
            balance_in_currency: Set(input.balance_in_currency),
            old_exchange_rate: Set(input.old_exchange_rate),
            new_exchange_rate: Set(input.new_exchange_rate),
            unrealized_gain_loss: Set(input.unrealized_gain_loss),
            transaction_id: Set(input.transaction_id),
            created_at: Set(chrono::Utc::now().into()),
        };

        let result = active.insert(&self.db).await?;
        Ok(result)
    }

    /// Orchestrates revaluation for all eligible accounts in an organization.
    ///
    /// # Arguments
    /// * `organization_id` - The organization to process.
    /// * `as_of` - The date for balance calculation and rate lookup.
    /// * `gain_loss_account_id` - The account to post unrealized gains/losses.
    /// * `rate_repo` - The exchange rate repository for lookups.
    /// * `tx_repo` - The transaction repository for posting adjustments.
    pub async fn process_revaluations(
        &self,
        organization_id: Uuid,
        as_of: NaiveDate,
        gain_loss_account_id: Uuid,
        rate_repo: &super::exchange_rate::ExchangeRateRepository,
        tx_repo: &super::transaction::TransactionRepository,
    ) -> Result<usize, RevaluationError> {
        use zeltra_core::ledger::revaluation::RevaluationEngine;

        // Get organization info for base currency
        let org = organizations::Entity::find_by_id(organization_id)
            .one(&self.db)
            .await?
            .ok_or(RevaluationError::OrganizationNotFound)?;

        let candidates = self.find_revaluation_candidates(organization_id).await?;
        let mut processed_count = 0;

        for account in candidates {
            // Check if already revalued to prevent double-posting
            let existing = revaluation_logs::Entity::find()
                .filter(revaluation_logs::Column::AccountId.eq(account.id))
                .filter(revaluation_logs::Column::RevaluationDate.eq(as_of))
                .one(&self.db)
                .await?;

            if existing.is_some() {
                continue;
            }

            // Get balances
            let (carrying_functional, source_balance) =
                self.get_carrying_balances(account.id, as_of).await?;

            if source_balance.is_zero() && carrying_functional.is_zero() {
                continue;
            }

            // Lookup current rate
            let rate_lookup = match rate_repo
                .find_rate(
                    organization_id,
                    &account.currency,
                    &org.base_currency,
                    as_of,
                )
                .await
            {
                Ok(lookup) => lookup,
                Err(_) => continue, // Skip if no rate found for this date
            };

            // Calculate adjustment
            let adjustment = RevaluationEngine::calculate_adjustment(
                carrying_functional,
                source_balance,
                rate_lookup.rate,
            );

            if adjustment.is_zero() {
                continue;
            }

            use zeltra_core::ledger::revaluation::RevaluationTransactionInput;
            let reval_input = RevaluationTransactionInput {
                organization_id,
                entity_id: Some(account.entity_id),
                account_id: account.id,
                gain_loss_account_id,
                account_currency: account.currency.clone(),
                base_currency: org.base_currency.clone(),
                adjustment_amount: adjustment,
                current_rate: rate_lookup.rate,
                transaction_date: as_of,
                created_by: Uuid::nil(),
            };

            let tx_input = RevaluationEngine::create_revaluation_transaction(&reval_input);

            // Map core input to repo input
            let repo_tx_input = crate::repositories::transaction::CreateTransactionInput {
                organization_id: tx_input.organization_id,
                entity_id: tx_input.entity_id.unwrap_or(account.entity_id),
                transaction_type:
                    crate::entities::sea_orm_active_enums::TransactionType::Revaluation,
                transaction_date: tx_input.transaction_date,
                description: tx_input.description,
                reference_number: tx_input.reference_number,
                memo: tx_input.memo,
                entries: tx_input
                    .entries
                    .into_iter()
                    .map(|e| {
                        let fa = e.functional_amount.unwrap_or(Decimal::ZERO);
                        let (debit, credit) = match e.entry_type {
                            zeltra_core::ledger::types::EntryType::Debit => (fa, Decimal::ZERO),
                            zeltra_core::ledger::types::EntryType::Credit => (Decimal::ZERO, fa),
                        };
                        crate::repositories::transaction::CreateLedgerEntryInput {
                            account_id: e.account_id,
                            source_currency: e.source_currency,
                            source_amount: e.source_amount,
                            exchange_rate: if !e.source_amount.is_zero() {
                                fa / e.source_amount
                            } else {
                                rate_lookup.rate
                            },
                            functional_currency: org.base_currency.clone(),
                            functional_amount: fa,
                            debit,
                            credit,
                            memo: e.memo,
                            compliance_metadata: e.compliance_metadata,
                            dimensions: e.dimensions,
                        }
                    })
                    .collect(),
                created_by: tx_input.created_by,
                timezone: "UTC".to_string(),
                idempotency_key: Some(self.generate_idempotency_key(
                    organization_id,
                    account.id,
                    as_of,
                )),
                iso_metadata: None,
            };

            // Execute transaction
            let tx_result = tx_repo
                .create_transaction(repo_tx_input)
                .await
                .map_err(|e| RevaluationError::Database(DbErr::Custom(e.to_string())))?;

            // Log revaluation
            let old_rate = if !source_balance.is_zero() {
                carrying_functional / source_balance
            } else {
                rate_lookup.rate
            };

            self.log_revaluation(CreateRevaluationLogInput {
                organization_id,
                entity_id: account.entity_id,
                account_id: account.id,
                revaluation_date: as_of,
                currency_id: account.currency,
                balance_in_currency: source_balance,
                old_exchange_rate: old_rate,
                new_exchange_rate: rate_lookup.rate,
                unrealized_gain_loss: adjustment,
                transaction_id: Some(tx_result.transaction.id),
            })
            .await?;

            processed_count += 1;
        }

        Ok(processed_count)
    }
}
