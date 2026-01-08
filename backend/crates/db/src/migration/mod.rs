//! Database migrations.
//!
//! Migrations are managed using sea-orm-migration.

pub use sea_orm_migration::prelude::*;

// Migration modules will be added here
// mod m20260107_000001_initial;

/// Migrator for running database migrations.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Add migrations here as they are created
            // Box::new(m20260107_000001_initial::Migration),
        ]
    }
}
