use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Issue 1: Add UNIQUE constraint to ledger_entries(account_id, account_version)
        // We drop the existing index and create a unique one.
        db.execute_unprepared("DROP INDEX IF EXISTS idx_le_account_version")
            .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX idx_le_account_version_unique ON ledger_entries (account_id, account_version)"
        ).await?;

        // 2. Issue 13: Add ON DELETE CASCADE to entry_dimensions(dimension_value_id)
        db.execute_unprepared(
            "ALTER TABLE entry_dimensions DROP CONSTRAINT IF EXISTS entry_dimensions_dimension_value_id_fkey"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE entry_dimensions ADD CONSTRAINT entry_dimensions_dimension_value_id_fkey 
             FOREIGN KEY (dimension_value_id) REFERENCES dimension_values(id) ON DELETE CASCADE",
        )
        .await?;

        // 3. ISO 20022 Alignment: Add timezone to transactions
        db.execute_unprepared(
            "ALTER TABLE transactions ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) NOT NULL DEFAULT 'UTC'"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Rollback timezone
        db.execute_unprepared("ALTER TABLE transactions DROP COLUMN IF EXISTS timezone")
            .await?;

        // Rollback Issue 13
        db.execute_unprepared(
            "ALTER TABLE entry_dimensions DROP CONSTRAINT IF EXISTS entry_dimensions_dimension_value_id_fkey"
        ).await?;
        db.execute_unprepared(
            "ALTER TABLE entry_dimensions ADD CONSTRAINT entry_dimensions_dimension_value_id_fkey 
             FOREIGN KEY (dimension_value_id) REFERENCES dimension_values(id)",
        )
        .await?;

        // Rollback Issue 1
        db.execute_unprepared("DROP INDEX IF EXISTS idx_le_account_version_unique")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX idx_le_account_version ON ledger_entries (account_id, account_version)",
        )
        .await?;

        Ok(())
    }
}
