//! Database migrations.
//!
//! Migrations are managed using sea-orm-migration.

pub use sea_orm_migration::prelude::*;

mod m20260108_000001_initial;
mod m20260108_000002_sessions;
mod m20260108_000003_force_rls;
mod m20260108_000004_email_verification;
mod m20260113_000001_fix_constraints;
mod m20260113_000002_audit_trigger;
mod m20260113_000003_advanced_foundation;
mod m20260114_000001_sentinel_intelligence;
mod m20260114_000002_sentinel_hardening;
mod m20260114_000003_tier_enforcement;
mod m20260114_000004_tier_limit_updates;
mod m20260114_000005_fix_unlimited_dimensions;

/// Migrator for running database migrations.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260108_000001_initial::Migration),
            Box::new(m20260108_000002_sessions::Migration),
            Box::new(m20260108_000003_force_rls::Migration),
            Box::new(m20260108_000004_email_verification::Migration),
            Box::new(m20260113_000001_fix_constraints::Migration),
            Box::new(m20260113_000002_audit_trigger::Migration),
            Box::new(m20260113_000003_advanced_foundation::Migration),
            Box::new(m20260114_000001_sentinel_intelligence::Migration),
            Box::new(m20260114_000002_sentinel_hardening::Migration),
            Box::new(m20260114_000003_tier_enforcement::Migration),
            Box::new(m20260114_000004_tier_limit_updates::Migration),
            Box::new(m20260114_000005_fix_unlimited_dimensions::Migration),
        ]
    }
}
