//! Background job for cleaning up expired sessions.

use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info};
use zeltra_db::SessionRepository;

/// Starts the session cleanup background job.
///
/// This job runs every hour and deletes expired sessions from the database.
/// It runs in a separate tokio task and continues until the application shuts down.
///
/// # Arguments
///
/// * `db` - Database connection pool
/// * `cleanup_interval_hours` - How often to run cleanup (in hours)
///
/// # Returns
///
/// A `tokio::task::JoinHandle` that can be used to await the task or cancel it.
pub fn start_session_cleanup_job(
    db: Arc<DatabaseConnection>,
    cleanup_interval_hours: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval_timer = interval(Duration::from_secs(cleanup_interval_hours * 3600));

        info!(
            interval_hours = cleanup_interval_hours,
            "Session cleanup job started"
        );

        loop {
            // Wait for the next interval
            interval_timer.tick().await;

            // Run cleanup
            match cleanup_expired_sessions(&db).await {
                Ok(deleted_count) => {
                    if deleted_count > 0 {
                        info!(deleted_count = deleted_count, "Session cleanup completed");
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "Session cleanup failed"
                    );
                }
            }
        }
    })
}

/// Cleans up expired sessions from the database.
///
/// This function deletes all sessions where `expires_at` is in the past.
///
/// # Arguments
///
/// * `db` - Database connection
///
/// # Returns
///
/// The number of sessions deleted.
///
/// # Errors
///
/// Returns an error if the database operation fails.
async fn cleanup_expired_sessions(db: &DatabaseConnection) -> Result<u64, sea_orm::DbErr> {
    let session_repo = SessionRepository::new(db.clone());
    session_repo.cleanup_expired().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        // This test requires a database connection
        // In a real scenario, you'd use a test database
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/test".to_string());

        if let Ok(db) = Database::connect(&db_url).await {
            let result = cleanup_expired_sessions(&db).await;
            assert!(result.is_ok());
        }
    }
}
