//! Background job to check and update expired trial subscriptions.

use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info};
use zeltra_db::repositories::SubscriptionRepository;

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

/// Check all organizations with trialing status and update expired ones.
async fn check_and_update_expired_trials(
    db: &sea_orm::DatabaseConnection,
) -> Result<usize, sea_orm::DbErr> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use zeltra_db::entities::{organizations, sea_orm_active_enums::SubscriptionStatus};

    // Find all organizations with trialing status
    let trialing_orgs = organizations::Entity::find()
        .filter(organizations::Column::SubscriptionStatus.eq(SubscriptionStatus::Trialing))
        .all(db)
        .await?;

    let mut expired_count = 0;

    for org in trialing_orgs {
        // Check if trial has expired
        if SubscriptionRepository::is_trial_expired(db, org.id).await? {
            info!(
                org_id = %org.id,
                org_name = %org.name,
                "⏰ Trial expired for organization"
            );

            // Update status to expired
            SubscriptionRepository::update_subscription_status(
                db,
                org.id,
                SubscriptionStatus::Expired,
            )
            .await?;

            expired_count += 1;
        }
    }

    Ok(expired_count)
}
