use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Update ledger_entries for Hash Chaining
        manager
            .alter_table(
                Table::alter()
                    .table(LedgerEntries::Table)
                    .add_column(
                        ColumnDef::new(LedgerEntries::EntryHash)
                            .string_len(64)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(LedgerEntries::PreviousEntryHash)
                            .string_len(64)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Update transactions for ISO 20022 and Idempotency
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(
                        ColumnDef::new(Transactions::IdempotencyKey)
                            .uuid()
                            .unique_key()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Transactions::IsoMetadata)
                            .json_binary()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LedgerEntries::Table)
                    .drop_column(LedgerEntries::EntryHash)
                    .drop_column(LedgerEntries::PreviousEntryHash)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::IdempotencyKey)
                    .drop_column(Transactions::IsoMetadata)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum LedgerEntries {
    Table,
    EntryHash,
    PreviousEntryHash,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    IdempotencyKey,
    IsoMetadata,
}
