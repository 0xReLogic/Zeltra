//! Migration to create `email_verification_tokens` table.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EmailVerificationTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EmailVerificationTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationTokens::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationTokens::TokenHash)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationTokens::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationTokens::UsedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(EmailVerificationTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_email_verification_tokens_user")
                            .from(
                                EmailVerificationTokens::Table,
                                EmailVerificationTokens::UserId,
                            )
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on token_hash for fast lookup
        manager
            .create_index(
                Index::create()
                    .name("idx_email_verification_tokens_hash")
                    .table(EmailVerificationTokens::Table)
                    .col(EmailVerificationTokens::TokenHash)
                    .to_owned(),
            )
            .await?;

        // Create index on user_id for finding user's tokens
        manager
            .create_index(
                Index::create()
                    .name("idx_email_verification_tokens_user")
                    .table(EmailVerificationTokens::Table)
                    .col(EmailVerificationTokens::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(EmailVerificationTokens::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum EmailVerificationTokens {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    UsedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
