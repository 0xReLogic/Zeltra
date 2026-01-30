//! Exchange rate background sync task.
//!
//! Provides a background task that periodically syncs exchange rates
//! from Frankfurter API to the database.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sea_orm::DatabaseConnection;
use tokio::time::interval;
use tracing::{error, info, warn};

use zeltra_core::currency::{ExchangeRateService, RateSource};
use zeltra_db::OrganizationRepository;
use zeltra_db::entities::sea_orm_active_enums::{RateSource as DbRateSource, SubscriptionTier};
use zeltra_db::repositories::exchange_rate::{CreateExchangeRateInput, ExchangeRateRepository};

/// Configuration for the exchange rate sync task.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Interval between sync runs (in seconds).
    pub interval_secs: u64,
    /// Base currency to fetch rates for (usually USD).
    pub base_currency: String,
    /// Target currencies to sync.
    pub target_currencies: Vec<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_secs: 1800, // 30 minutes
            base_currency: "USD".to_string(),
            target_currencies: vec![
                "EUR".to_string(),
                "GBP".to_string(),
                "JPY".to_string(),
                "IDR".to_string(),
                "SGD".to_string(),
                "CNY".to_string(),
                "AUD".to_string(),
                "CAD".to_string(),
                "CHF".to_string(),
            ],
        }
    }
}

/// Run the exchange rate sync loop as a background task.
///
/// This function runs indefinitely, syncing exchange rates at the configured interval.
/// It should be spawned with `tokio::spawn`.
pub async fn run_sync_loop(db: Arc<DatabaseConnection>, config: SyncConfig) {
    info!(
        interval_secs = config.interval_secs,
        base_currency = %config.base_currency,
        targets = ?config.target_currencies,
        "Starting exchange rate sync task"
    );

    // Create service
    let service = match ExchangeRateService::new(RateSource::from_env()) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to create exchange rate service, sync disabled");
            return;
        }
    };

    // Run initial sync immediately
    sync_rates(&db, &service, &config).await;

    // Then run on interval
    let mut ticker = interval(Duration::from_secs(config.interval_secs));

    loop {
        ticker.tick().await;
        sync_rates(&db, &service, &config).await;
    }
}

/// Perform a single sync of exchange rates.
async fn sync_rates(db: &DatabaseConnection, service: &ExchangeRateService, config: &SyncConfig) {
    info!("Starting exchange rate sync...");

    // Fetch rates from external API
    let rates = match service
        .fetch_latest(&config.base_currency, &config.target_currencies)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!(error = %e, "Failed to fetch exchange rates from API");
            return;
        }
    };

    if rates.is_empty() {
        warn!("No rates fetched from API");
        return;
    }

    info!(
        count = rates.len(),
        "Fetched {} rates from API",
        rates.len()
    );

    // Get all organizations to sync rates for
    let org_repo = OrganizationRepository::new(db.clone());
    let orgs = match org_repo.list_all().await {
        Ok(o) => o,
        Err(e) => {
            error!(error = %e, "Failed to list organizations");
            return;
        }
    };

    if orgs.is_empty() {
        warn!("No organizations found, skipping rate sync");
        return;
    }

    // Store rates for each organization (skip STARTER tier - manual input only per BUSINESS_MODEL.md)
    // NOTE: Subscription tier is now per-user, not per-organization.
    // We check the organization owner's subscription tier to determine if auto-sync is enabled.

    let rate_repo = ExchangeRateRepository::new(db.clone());
    let today = Utc::now().date_naive();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut skipped_starter = 0;

    for org in &orgs {
        // Get organization owner's subscription tier
        // TODO: Implement get_owner_subscription_tier method in OrganizationRepository
        // For now, skip STARTER tier check (all orgs get auto-sync)
        // if owner_tier == SubscriptionTier::Starter {
        //     skipped_starter += 1;
        //     continue;
        // }

        for rate in &rates {
            let input = CreateExchangeRateInput {
                organization_id: org.id,
                from_currency: rate.from_currency.clone(),
                to_currency: rate.to_currency.clone(),
                rate: rate.rate,
                effective_date: today,
                source: DbRateSource::Api,
                source_reference: Some("frankfurter".to_string()),
                created_by: None,
            };

            match rate_repo.create_or_update_rate(input).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!(
                        org_id = %org.id,
                        pair = %format!("{}/{}", rate.from_currency, rate.to_currency),
                        error = %e,
                        "Failed to store exchange rate"
                    );
                    error_count += 1;
                }
            }
        }
    }

    info!(
        success = success_count,
        errors = error_count,
        orgs = orgs.len(),
        skipped_starter = skipped_starter,
        "Exchange rate sync completed"
    );
}
