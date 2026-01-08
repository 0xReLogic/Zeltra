//! Database layer with `SeaORM` entities and repositories.
//!
//! This crate provides:
//! - `SeaORM` entity definitions
//! - Repository abstractions for data access
//! - Database migrations

pub mod entities;
pub mod migration;
pub mod repositories;

pub use repositories::{OrganizationRepository, SessionRepository, UserRepository};

use sea_orm::{Database, DatabaseConnection, DbErr};

/// Establishes a connection to the database.
///
/// # Errors
///
/// Returns an error if the connection cannot be established.
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(database_url).await
}
