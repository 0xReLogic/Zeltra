//! Migration to enable FORCE ROW LEVEL SECURITY on all tenant tables.
//!
//! This ensures RLS policies apply even to table owners and superusers,
//! providing an additional layer of security for multi-tenant isolation.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Enable FORCE ROW LEVEL SECURITY on all tenant tables
        // This ensures RLS applies even to superusers and table owners
        db.execute_unprepared(FORCE_RLS_SQL).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Disable FORCE RLS (revert to normal RLS behavior)
        db.execute_unprepared(DISABLE_FORCE_RLS_SQL).await?;

        Ok(())
    }
}

const FORCE_RLS_SQL: &str = r"
-- ============================================================
-- FORCE ROW LEVEL SECURITY
-- Ensures RLS policies apply to ALL users including superusers
-- ============================================================

ALTER TABLE organizations FORCE ROW LEVEL SECURITY;
ALTER TABLE organization_users FORCE ROW LEVEL SECURITY;
ALTER TABLE fiscal_years FORCE ROW LEVEL SECURITY;
ALTER TABLE fiscal_periods FORCE ROW LEVEL SECURITY;
ALTER TABLE dimension_types FORCE ROW LEVEL SECURITY;
ALTER TABLE dimension_values FORCE ROW LEVEL SECURITY;
ALTER TABLE chart_of_accounts FORCE ROW LEVEL SECURITY;
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
ALTER TABLE ledger_entries FORCE ROW LEVEL SECURITY;
ALTER TABLE budgets FORCE ROW LEVEL SECURITY;
ALTER TABLE budget_lines FORCE ROW LEVEL SECURITY;
ALTER TABLE budget_line_dimensions FORCE ROW LEVEL SECURITY;
ALTER TABLE attachments FORCE ROW LEVEL SECURITY;
ALTER TABLE exchange_rates FORCE ROW LEVEL SECURITY;
ALTER TABLE approval_rules FORCE ROW LEVEL SECURITY;
ALTER TABLE organization_usage FORCE ROW LEVEL SECURITY;
ALTER TABLE entry_dimensions FORCE ROW LEVEL SECURITY;
";

const DISABLE_FORCE_RLS_SQL: &str = r"
-- ============================================================
-- DISABLE FORCE ROW LEVEL SECURITY (Rollback)
-- ============================================================

ALTER TABLE organizations NO FORCE ROW LEVEL SECURITY;
ALTER TABLE organization_users NO FORCE ROW LEVEL SECURITY;
ALTER TABLE fiscal_years NO FORCE ROW LEVEL SECURITY;
ALTER TABLE fiscal_periods NO FORCE ROW LEVEL SECURITY;
ALTER TABLE dimension_types NO FORCE ROW LEVEL SECURITY;
ALTER TABLE dimension_values NO FORCE ROW LEVEL SECURITY;
ALTER TABLE chart_of_accounts NO FORCE ROW LEVEL SECURITY;
ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
ALTER TABLE ledger_entries NO FORCE ROW LEVEL SECURITY;
ALTER TABLE budgets NO FORCE ROW LEVEL SECURITY;
ALTER TABLE budget_lines NO FORCE ROW LEVEL SECURITY;
ALTER TABLE budget_line_dimensions NO FORCE ROW LEVEL SECURITY;
ALTER TABLE attachments NO FORCE ROW LEVEL SECURITY;
ALTER TABLE exchange_rates NO FORCE ROW LEVEL SECURITY;
ALTER TABLE approval_rules NO FORCE ROW LEVEL SECURITY;
ALTER TABLE organization_usage NO FORCE ROW LEVEL SECURITY;
ALTER TABLE entry_dimensions NO FORCE ROW LEVEL SECURITY;
";
