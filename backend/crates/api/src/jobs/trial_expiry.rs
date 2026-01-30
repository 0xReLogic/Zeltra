//! Background job to check and update expired trial subscriptions.

use sea_orm::ActiveModelTrait;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info};

/// Start the trial expiry check background job.
///
/// This job runs periodically to check for expired trials and update their status.
/// Default interval: 1 hour (configurable via TRIAL_CHECK_INTERVAL_HOURS env var)
pub fn start_trial_expiry_job(db: Arc<sea_orm::DatabaseConnection>) {
    tokio::spawn(async move {
        // Get interval from env or default to 1 hour
        let interval_hours = std::env::var("TRIAL_CHECK_INTERVAL_HOURS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1);

        let mut ticker = interval(Duration::from_secs(interval_hours * 3600));

        info!(
            interval_hours = interval_hours,
            "🔄 Trial expiry check job started"
        );

        loop {
            ticker.tick().await;

            match check_and_update_expired_trials(&db).await {
                Ok(count) => {
                    if count > 0 {
                        info!(
                            expired_count = count,
                            "✅ Updated {} expired trial subscriptions", count
                        );
                    }
                }
                Err(e) => {
                    error!(error = %e, "❌ Failed to check expired trials");
                }
            }
        }
    });
}

/// Check all users with trialing status and update expired ones.
async fn check_and_update_expired_trials(
    db: &sea_orm::DatabaseConnection,
) -> Result<usize, sea_orm::DbErr> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
    use zeltra_db::entities::{sea_orm_active_enums::SubscriptionStatus, users};

    // Find all users with trialing status and expired trial_ends_at
    let now = chrono::Utc::now();
    let trialing_users = users::Entity::find()
        .filter(users::Column::SubscriptionStatus.eq(SubscriptionStatus::Trialing))
        .filter(users::Column::TrialEndsAt.is_not_null())
        .filter(users::Column::TrialEndsAt.lt(now))
        .all(db)
        .await?;

    let mut expired_count = 0;

    for user in trialing_users {
        info!(
            user_id = %user.id,
            email = %user.email,
            "⏰ Trial expired for user"
        );

        // Update status to expired
        let mut user_active: users::ActiveModel = user.into();
        user_active.subscription_status = Set(SubscriptionStatus::Expired);
        user_active.update(db).await?;

        expired_count += 1;
    }

    Ok(expired_count)
}
