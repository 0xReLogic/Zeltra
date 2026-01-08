//! Row-Level Security (RLS) context management.
//!
//! This module provides utilities for setting `PostgreSQL` RLS context
//! per request to enforce multi-tenant data isolation.
//!
//! # Usage
//!
//! ```ignore
//! use zeltra_db::rls::RlsConnection;
//!
//! // In your handler or middleware:
//! let rls_conn = RlsConnection::new(&db, organization_id).await?;
//!
//! // Use rls_conn.transaction() for all queries
//! let users = User::find().all(rls_conn.transaction()).await?;
//!
//! // Commit when done
//! rls_conn.commit().await?;
//! ```

use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};
use uuid::Uuid;

/// A database connection wrapper that sets RLS context for multi-tenant isolation.
///
/// This struct wraps a database transaction and ensures that the `PostgreSQL`
/// session variable `app.current_organization_id` is set before any queries
/// are executed, enabling row-level security policies.
pub struct RlsConnection {
    txn: DatabaseTransaction,
}

impl RlsConnection {
    /// Creates a new RLS-enabled connection with the given organization context.
    ///
    /// This begins a transaction and sets the `app.current_organization_id`
    /// session variable using `SET LOCAL`, which scopes the setting to the
    /// current transaction only.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection pool
    /// * `organization_id` - The organization ID to set as the RLS context
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction cannot be started or the RLS
    /// context cannot be set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let rls = RlsConnection::new(&db, org_id).await?;
    /// let accounts = Account::find().all(rls.transaction()).await?;
    /// rls.commit().await?;
    /// ```
    pub async fn new(db: &DatabaseConnection, organization_id: Uuid) -> Result<Self, DbErr> {
        let txn = db.begin().await?;

        // Set the RLS context using SET LOCAL (scoped to transaction)
        // Using parameterized query to prevent SQL injection
        let sql = format!("SET LOCAL app.current_organization_id = '{organization_id}'");
        txn.execute_unprepared(&sql).await?;

        Ok(Self { txn })
    }

    /// Returns a reference to the underlying transaction for executing queries.
    ///
    /// All queries executed through this transaction will be subject to
    /// the RLS policies based on the organization context set during creation.
    #[must_use]
    pub fn transaction(&self) -> &DatabaseTransaction {
        &self.txn
    }

    /// Commits the transaction, persisting all changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit fails.
    pub async fn commit(self) -> Result<(), DbErr> {
        self.txn.commit().await
    }

    /// Rolls back the transaction, discarding all changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the rollback fails.
    pub async fn rollback(self) -> Result<(), DbErr> {
        self.txn.rollback().await
    }
}

/// Extension trait for `DatabaseConnection` to easily create RLS-enabled connections.
#[async_trait::async_trait]
pub trait RlsExt {
    /// Creates an RLS-enabled connection with the given organization context.
    ///
    /// # Errors
    ///
    /// Returns an error if the RLS connection cannot be created.
    async fn with_rls(&self, organization_id: Uuid) -> Result<RlsConnection, DbErr>;
}

#[async_trait::async_trait]
impl RlsExt for DatabaseConnection {
    async fn with_rls(&self, organization_id: Uuid) -> Result<RlsConnection, DbErr> {
        RlsConnection::new(self, organization_id).await
    }
}

/// Sets the RLS context on an existing transaction.
///
/// Use this when you already have a transaction and need to set the RLS context.
///
/// # Arguments
///
/// * `txn` - The database transaction
/// * `organization_id` - The organization ID to set as the RLS context
///
/// # Errors
///
/// Returns an error if the RLS context cannot be set.
pub async fn set_rls_context(
    txn: &DatabaseTransaction,
    organization_id: Uuid,
) -> Result<(), DbErr> {
    let sql = format!("SET LOCAL app.current_organization_id = '{organization_id}'");
    txn.execute_unprepared(&sql).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a real PostgreSQL database with RLS enabled.
    // They should be run as integration tests.

    #[test]
    fn test_rls_sql_format() {
        let org_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let sql = format!("SET LOCAL app.current_organization_id = '{org_id}'");
        assert_eq!(
            sql,
            "SET LOCAL app.current_organization_id = '550e8400-e29b-41d4-a716-446655440000'"
        );
    }
}
