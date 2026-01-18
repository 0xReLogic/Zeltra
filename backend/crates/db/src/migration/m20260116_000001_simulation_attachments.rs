//! Migration to add simulation_id column to attachments table.
//!
//! This enables attaching files to simulation runs.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add simulation_id column to attachments table
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .add_column(ColumnDef::new(Attachments::SimulationId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add index for simulation_id lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_attachments_simulation_id")
                    .table(Attachments::Table)
                    .col(Attachments::SimulationId)
                    .to_owned(),
            )
            .await?;

        // Add check constraint: either transaction_id or simulation_id must be set (not both)
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            ALTER TABLE attachments
            ADD CONSTRAINT chk_attachment_parent
            CHECK (
                (transaction_id IS NOT NULL AND simulation_id IS NULL) OR
                (transaction_id IS NULL AND simulation_id IS NOT NULL)
            )
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop check constraint
        db.execute_unprepared(
            "ALTER TABLE attachments DROP CONSTRAINT IF EXISTS chk_attachment_parent",
        )
        .await?;

        // Drop index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_attachments_simulation_id")
                    .table(Attachments::Table)
                    .to_owned(),
            )
            .await?;

        // Drop column
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .drop_column(Attachments::SimulationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Attachments {
    Table,
    SimulationId,
}
