//! Accrual repository for database operations.

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::entities::accrual_schedules;
use zeltra_core::ledger::types::{AccrualFrequency, AccrualStatus};

/// Error types for accrual operations.
#[derive(Debug, thiserror::Error)]
pub enum AccrualError {
    /// Schedule not found.
    #[error("Accrual schedule not found")]
    NotFound,

    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

/// Input for creating a new accrual schedule.
#[derive(Debug, Clone)]
pub struct CreateAccrualScheduleInput {
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub total_amount: Decimal,
    pub currency_id: String,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub frequency: AccrualFrequency,
    pub total_periods: i32,
    pub next_run_date: Option<NaiveDate>,
}

/// Accrual repository for CRUD operations.
#[derive(Debug, Clone)]
pub struct AccrualRepository {
    db: DatabaseConnection,
}

impl AccrualRepository {
    /// Creates a new accrual repository.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates a new accrual schedule.
    pub async fn create_schedule(
        &self,
        input: CreateAccrualScheduleInput,
    ) -> Result<accrual_schedules::Model, AccrualError> {
        let now = Utc::now().into();
        let schedule = accrual_schedules::ActiveModel {
            id: Set(Uuid::new_v4()),
            organization_id: Set(input.organization_id),
            name: Set(input.name),
            description: Set(input.description),
            total_amount: Set(input.total_amount),
            currency_id: Set(input.currency_id),
            debit_account_id: Set(input.debit_account_id),
            credit_account_id: Set(input.credit_account_id),
            start_date: Set(input.start_date),
            end_date: Set(input.end_date),
            frequency: Set(format!("{:?}", input.frequency).to_lowercase()),
            total_periods: Set(input.total_periods),
            periods_processed: Set(0),
            next_run_date: Set(input.next_run_date),
            status: Set("active".to_string()),
            last_transaction_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = schedule.insert(&self.db).await?;
        Ok(result)
    }

    /// Finds all active schedules that are due for processing.
    pub async fn find_due_schedules(
        &self,
        target_date: NaiveDate,
    ) -> Result<Vec<accrual_schedules::Model>, AccrualError> {
        let schedules = accrual_schedules::Entity::find()
            .filter(accrual_schedules::Column::Status.eq("active"))
            .filter(accrual_schedules::Column::NextRunDate.lte(target_date))
            .all(&self.db)
            .await?;

        Ok(schedules)
    }

    /// Updates a schedule after a successful run.
    pub async fn update_after_run(
        &self,
        id: Uuid,
        periods_processed: i32,
        next_run_date: Option<NaiveDate>,
        last_transaction_id: Uuid,
        status: AccrualStatus,
    ) -> Result<accrual_schedules::Model, AccrualError> {
        let schedule = accrual_schedules::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(AccrualError::NotFound)?;

        let mut active: accrual_schedules::ActiveModel = schedule.into();
        active.periods_processed = Set(periods_processed);
        active.next_run_date = Set(next_run_date);
        active.last_transaction_id = Set(Some(last_transaction_id));
        active.status = Set(format!("{:?}", status).to_lowercase());
        active.updated_at = Set(Utc::now().into());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    /// Processes all due accrual schedules.
    pub async fn process_due_accruals(
        &self,
        transaction_repo: &super::transaction::TransactionRepository,
        target_date: NaiveDate,
    ) -> Result<usize, AccrualError> {
        let schedules = self.find_due_schedules(target_date).await?;
        let mut processed = 0;

        for schedule in schedules {
            let frequency = AccrualFrequency::from_str(&schedule.frequency)
                .unwrap_or(AccrualFrequency::Monthly);

            let period_amount =
                zeltra_core::ledger::accrual::AccrualEngine::calculate_period_amount(
                    schedule.total_amount,
                    schedule.total_periods,
                    schedule.periods_processed,
                );

            let new_periods_processed = schedule.periods_processed + 1;
            let next_run = if new_periods_processed < schedule.total_periods {
                Some(
                    zeltra_core::ledger::accrual::AccrualEngine::calculate_next_run_date(
                        schedule.next_run_date.unwrap_or(schedule.start_date),
                        frequency,
                    ),
                )
            } else {
                None
            };

            let status = if next_run.is_none() {
                AccrualStatus::Completed
            } else {
                AccrualStatus::Active
            };

            // Create the transaction
            let tx_input = crate::repositories::transaction::CreateTransactionInput {
                organization_id: schedule.organization_id,
                transaction_type: crate::entities::sea_orm_active_enums::TransactionType::Accrual,
                transaction_date: target_date,
                description: format!("Automated Accrual: {}", schedule.name),
                reference_number: Some(format!("ACCR-{}-{}", schedule.id, new_periods_processed)),
                memo: Some(format!(
                    "Period {} of {}",
                    new_periods_processed, schedule.total_periods
                )),
                entries: vec![
                    crate::repositories::transaction::CreateLedgerEntryInput {
                        account_id: schedule.debit_account_id,
                        source_currency: schedule.currency_id.clone(),
                        source_amount: period_amount,
                        exchange_rate: Decimal::from(1),
                        functional_currency: schedule.currency_id.clone(),
                        functional_amount: period_amount,
                        debit: period_amount,
                        credit: Decimal::ZERO,
                        memo: Some(format!("Accrual for {}", schedule.name)),
                        compliance_metadata: None,
                        dimensions: vec![],
                    },
                    crate::repositories::transaction::CreateLedgerEntryInput {
                        account_id: schedule.credit_account_id,
                        source_currency: schedule.currency_id.clone(),
                        source_amount: period_amount,
                        exchange_rate: Decimal::from(1),
                        functional_currency: schedule.currency_id.clone(),
                        functional_amount: period_amount,
                        debit: Decimal::ZERO,
                        credit: period_amount,
                        memo: Some(format!("Accrual for {}", schedule.name)),
                        compliance_metadata: None,
                        dimensions: vec![],
                    },
                ],
                created_by: Uuid::nil(), // System generated
                timezone: "UTC".to_string(),
                idempotency_key: Some(Uuid::new_v4()),
                iso_metadata: None,
            };

            // Use the transaction repo to create the ledger entries
            let result = transaction_repo
                .create_transaction(tx_input)
                .await
                .map_err(|e| AccrualError::Database(DbErr::Custom(e.to_string())))?;

            // Update the schedule
            self.update_after_run(
                schedule.id,
                new_periods_processed,
                next_run,
                result.transaction.id,
                status,
            )
            .await?;

            processed += 1;
        }

        Ok(processed)
    }
}
