//! Database migration runner for Zeltra.
//!
//! Usage:
//!   migrator up      - Run all pending migrations
//!   migrator down    - Rollback last migration
//!   migrator status  - Show migration status
//!   migrator fresh   - Drop all tables and re-run migrations

use sea_orm_migration::prelude::*;
use zeltra_db::migration::Migrator;

#[tokio::main]
async fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Run the migrator CLI (it sets up its own tracing)
    cli::run_cli(Migrator).await;
}
