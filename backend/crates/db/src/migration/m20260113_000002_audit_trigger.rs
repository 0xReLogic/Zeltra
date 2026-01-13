use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Create the function that raises an exception
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION prevent_ledger_update()
            RETURNS TRIGGER AS $$
            BEGIN
                RAISE EXCEPTION 'Ledger entries are immutable and cannot be updated. Create a void/reversal transaction instead.';
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        // 2. Attach the trigger to ledger_entries
        db.execute_unprepared(
            r#"
            CREATE TRIGGER trg_prevent_ledger_update
            BEFORE UPDATE ON ledger_entries
            FOR EACH ROW
            EXECUTE FUNCTION prevent_ledger_update();
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Drop trigger
        db.execute_unprepared("DROP TRIGGER IF EXISTS trg_prevent_ledger_update ON ledger_entries")
            .await?;

        // 2. Drop function
        db.execute_unprepared("DROP FUNCTION IF EXISTS prevent_ledger_update")
            .await?;

        Ok(())
    }
}
