use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Hardening Accrual: Add total_amount_recognized to track progress correctly
        db.execute_unprepared(
            "ALTER TABLE accrual_schedules ADD COLUMN total_amount_recognized NUMERIC(19, 4) NOT NULL DEFAULT 0"
        ).await?;

        // 2. Hardening Revaluation: Prevent double-posting on the same day for same account
        // This ensures idempotency and safe concurrency
        db.execute_unprepared(
            "ALTER TABLE revaluation_logs ADD CONSTRAINT uniq_reval_account_date UNIQUE (account_id, revaluation_date)"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "ALTER TABLE revaluation_logs DROP CONSTRAINT IF EXISTS uniq_reval_account_date",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE accrual_schedules DROP COLUMN IF EXISTS total_amount_recognized",
        )
        .await?;

        Ok(())
    }
}
