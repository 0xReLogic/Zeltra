//! Migration: Add performance indexes for approval_rules table
//!
//! **Validates: Requirements 2.2.2, Property 9 (Database Index Usage)**
//!
//! This migration adds indexes to optimize approval rules queries:
//! - Index on (organization_id, priority) for sorting active rules
//! - GIN index on transaction_types for filtering by transaction type
//! - Index on (organization_id, required_role) for role-based filtering
//! - Index on (organization_id, min_amount, max_amount) for amount range queries

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Index for sorting by priority (most common query pattern)
        manager
            .create_index(
                Index::create()
                    .name("idx_approval_rules_org_priority")
                    .table(ApprovalRules::Table)
                    .col(ApprovalRules::OrganizationId)
                    .col(ApprovalRules::Priority)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Partial index for active rules only (most queries filter by is_active)
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE INDEX IF NOT EXISTS idx_approval_rules_org_priority_active
                ON approval_rules(organization_id, priority)
                WHERE is_active = true
                "#,
            )
            .await?;

        // GIN index for transaction_types array filtering
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE INDEX IF NOT EXISTS idx_approval_rules_tx_types
                ON approval_rules USING GIN(transaction_types)
                WHERE is_active = true
                "#,
            )
            .await?;

        // Index for role-based filtering
        manager
            .create_index(
                Index::create()
                    .name("idx_approval_rules_org_role")
                    .table(ApprovalRules::Table)
                    .col(ApprovalRules::OrganizationId)
                    .col(ApprovalRules::RequiredRole)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Index for amount range queries
        manager
            .create_index(
                Index::create()
                    .name("idx_approval_rules_org_amounts")
                    .table(ApprovalRules::Table)
                    .col(ApprovalRules::OrganizationId)
                    .col(ApprovalRules::MinAmount)
                    .col(ApprovalRules::MaxAmount)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes in reverse order
        manager
            .drop_index(
                Index::drop()
                    .name("idx_approval_rules_org_amounts")
                    .table(ApprovalRules::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_approval_rules_org_role")
                    .table(ApprovalRules::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_approval_rules_tx_types")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_approval_rules_org_priority_active")
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_approval_rules_org_priority")
                    .table(ApprovalRules::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ApprovalRules {
    Table,
    OrganizationId,
    Priority,
    RequiredRole,
    MinAmount,
    MaxAmount,
}
