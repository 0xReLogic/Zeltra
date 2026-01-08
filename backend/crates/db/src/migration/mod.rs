//! Database migrations.
//!
//! Migrations are managed using sea-orm-migration.

pub use sea_orm_migration::prelude::*;

mod m20260108_000001_initial;

/// Migrator for running database migrations.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260108_000001_initial::Migration)]
    }
}
