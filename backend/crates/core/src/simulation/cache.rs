//! Simulation result caching using Moka.
//!
//! Provides in-memory caching for simulation results to avoid
//! redundant computations when the same parameters are used.

use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

use super::engine::SimulationEngine;
use super::types::{HistoricalAccountData, SimulationParams, SimulationResult};

/// Default cache capacity (number of entries).
const DEFAULT_CACHE_CAPACITY: u64 = 100;

/// Default time-to-live for cache entries (5 minutes).
const DEFAULT_TTL_SECS: u64 = 300;

/// Cache for simulation results.
///
/// Uses parameter hash as the cache key and stores complete
/// simulation results. Thread-safe and suitable for concurrent access.
#[derive(Clone)]
pub struct SimulationCache {
    cache: Cache<String, Arc<SimulationResult>>,
}

impl SimulationCache {
    /// Creates a new simulation cache with default settings.
    ///
    /// Default: 100 entries max, 5 minute TTL.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(DEFAULT_CACHE_CAPACITY, DEFAULT_TTL_SECS)
    }

    /// Creates a new simulation cache with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of entries to cache
    /// * `ttl_secs` - Time-to-live in seconds for each entry
    #[must_use]
    pub fn with_config(max_capacity: u64, ttl_secs: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();

        Self { cache }
    }

    /// Runs a simulation, returning cached results if available.
    ///
    /// If a cached result exists for the given parameters, it is returned
    /// with `cached: true`. Otherwise, the simulation is run and the result
    /// is cached before being returned.
    ///
    /// # Arguments
    ///
    /// * `historical_data` - Historical account data for baseline calculation
    /// * `params` - Simulation parameters
    ///
    /// # Returns
    ///
    /// Simulation result, either from cache or freshly computed.
    #[must_use]
    pub fn run_cached(
        &self,
        historical_data: &[HistoricalAccountData],
        params: &SimulationParams,
    ) -> SimulationResult {
        let cache_key = SimulationEngine::hash_params(params);

        // Check cache first
        if let Some(cached_result) = self.cache.get(&cache_key) {
            // Return cached result with cached flag set to true
            let mut result = (*cached_result).clone();
            result.cached = true;
            return result;
        }

        // Run simulation
        let result = SimulationEngine::run(historical_data, params);

        // Store in cache
        self.cache.insert(cache_key, Arc::new(result.clone()));

        result
    }

    /// Invalidates all cached entries.
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    /// Invalidates a specific cache entry by parameters.
    pub fn invalidate(&self, params: &SimulationParams) {
        let cache_key = SimulationEngine::hash_params(params);
        self.cache.invalidate(&cache_key);
    }

    /// Returns the number of entries currently in the cache.
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Runs cache maintenance tasks.
    ///
    /// This should be called periodically to clean up expired entries.
    /// Moka handles this automatically in the background, but calling
    /// this explicitly can help reclaim memory sooner.
    pub fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks();
    }
}

impl Default for SimulationCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_params() -> SimulationParams {
        SimulationParams {
            base_period_start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            base_period_end: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            projection_months: 12,
            revenue_growth_rate: dec!(0.10),
            expense_growth_rate: dec!(0.05),
            account_adjustments: HashMap::new(),
            dimension_filters: vec![],
        }
    }

    fn create_test_data() -> Vec<HistoricalAccountData> {
        vec![HistoricalAccountData {
            account_id: Uuid::new_v4(),
            account_code: "4000".to_string(),
            account_name: "Revenue".to_string(),
            account_type: "revenue".to_string(),
            monthly_amounts: vec![dec!(1000)],
        }]
    }

    #[test]
    fn test_cache_miss_then_hit() {
        let cache = SimulationCache::new();
        let params = create_test_params();
        let data = create_test_data();

        // First call - cache miss
        let result1 = cache.run_cached(&data, &params);
        assert!(!result1.cached, "First call should not be cached");

        // Second call - cache hit
        let result2 = cache.run_cached(&data, &params);
        assert!(result2.cached, "Second call should be cached");

        // Results should be equivalent (except cached flag)
        assert_eq!(result1.parameters_hash, result2.parameters_hash);
        assert_eq!(result1.projections.len(), result2.projections.len());
    }

    #[test]
    fn test_different_params_not_cached() {
        let cache = SimulationCache::new();
        let data = create_test_data();

        let params1 = create_test_params();
        let mut params2 = create_test_params();
        params2.projection_months = 24;

        // First call with params1
        let result1 = cache.run_cached(&data, &params1);
        assert!(!result1.cached);

        // Call with different params - should be cache miss
        let result2 = cache.run_cached(&data, &params2);
        assert!(!result2.cached, "Different params should not hit cache");

        // Call with params1 again - should be cache hit
        let result3 = cache.run_cached(&data, &params1);
        assert!(result3.cached, "Same params should hit cache");
    }

    #[test]
    fn test_invalidate_all() {
        let cache = SimulationCache::new();
        let params = create_test_params();
        let data = create_test_data();

        // Populate cache
        let result1 = cache.run_cached(&data, &params);
        assert!(!result1.cached);

        // Verify it's cached
        let result2 = cache.run_cached(&data, &params);
        assert!(result2.cached, "Should be cached after first call");

        // Invalidate all
        cache.invalidate_all();
        cache.run_pending_tasks();

        // Next call should be cache miss
        let result = cache.run_cached(&data, &params);
        assert!(!result.cached, "Should be cache miss after invalidate_all");
    }

    #[test]
    fn test_invalidate_specific() {
        let cache = SimulationCache::new();
        let data = create_test_data();

        let params1 = create_test_params();
        let mut params2 = create_test_params();
        params2.projection_months = 24;

        // Populate cache with both
        let _ = cache.run_cached(&data, &params1);
        let _ = cache.run_cached(&data, &params2);

        // Invalidate only params1
        cache.invalidate(&params1);
        cache.run_pending_tasks();

        // params1 should be cache miss
        let result1 = cache.run_cached(&data, &params1);
        assert!(!result1.cached, "Invalidated params should be cache miss");

        // params2 should still be cache hit
        let result2 = cache.run_cached(&data, &params2);
        assert!(
            result2.cached,
            "Non-invalidated params should still hit cache"
        );
    }

    #[test]
    fn test_custom_config() {
        let cache = SimulationCache::with_config(10, 60);
        let params = create_test_params();
        let data = create_test_data();

        let result = cache.run_cached(&data, &params);
        assert!(!result.cached);

        let result = cache.run_cached(&data, &params);
        assert!(result.cached);
    }

    #[test]
    fn test_entry_count() {
        let cache = SimulationCache::new();
        let data = create_test_data();

        assert_eq!(cache.entry_count(), 0);

        let params1 = create_test_params();
        let _ = cache.run_cached(&data, &params1);

        // Entry count may not update immediately due to async nature
        // Run pending tasks to ensure count is updated
        cache.run_pending_tasks();
        assert!(cache.entry_count() >= 1);
    }

    #[test]
    fn test_default_impl() {
        let cache = SimulationCache::default();
        let params = create_test_params();
        let data = create_test_data();

        let result = cache.run_cached(&data, &params);
        assert!(!result.cached);
    }
}
