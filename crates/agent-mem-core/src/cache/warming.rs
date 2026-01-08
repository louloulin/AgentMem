//! Cache warming strategies
//!
//! Provides mechanisms to preload frequently accessed data into cache:
//! - Eager warming: Load data at startup
//! - Lazy warming: Load data on first access
//! - Scheduled warming: Periodically refresh cache
//! - Predictive warming: Load data based on access patterns

use super::{Cache, CacheKey};
use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Cache warming strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarmingStrategy {
    /// Load all data at startup
    Eager,

    /// Load data on first access
    Lazy,

    /// Periodically refresh cache
    Scheduled { interval: Duration },

    /// Load based on access patterns
    Predictive {
        min_access_count: u64,
        lookback_duration: Duration,
    },
}

/// Cache warming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheWarmingConfig {
    /// Warming strategy
    pub strategy: WarmingStrategy,

    /// Maximum items to warm
    pub max_items: usize,

    /// Batch size for warming
    pub batch_size: usize,

    /// Enable warming statistics
    pub enable_stats: bool,
}

impl Default for CacheWarmingConfig {
    fn default() -> Self {
        Self {
            strategy: WarmingStrategy::Lazy,
            max_items: 1000,
            batch_size: 100,
            enable_stats: true,
        }
    }
}

/// Warming statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WarmingStats {
    /// Total warming operations
    pub total_warmings: u64,

    /// Total items warmed
    pub total_items_warmed: u64,

    /// Total warming time in milliseconds
    pub total_warming_time_ms: u64,

    /// Last warming timestamp
    pub last_warming_timestamp: u64,

    /// Failed warming attempts
    pub failed_warmings: u64,
}

impl WarmingStats {
    /// Calculate average warming time
    pub fn average_warming_time_ms(&self) -> f64 {
        if self.total_warmings == 0 {
            0.0
        } else {
            self.total_warming_time_ms as f64 / self.total_warmings as f64
        }
    }

    /// Calculate average items per warming
    pub fn average_items_per_warming(&self) -> f64 {
        if self.total_warmings == 0 {
            0.0
        } else {
            self.total_items_warmed as f64 / self.total_warmings as f64
        }
    }
}

/// Data loader trait for cache warming
#[async_trait::async_trait]
pub trait DataLoader: Send + Sync {
    /// Load data for warming
    async fn load_data(&self, keys: Vec<CacheKey>) -> Result<HashMap<CacheKey, Vec<u8>>>;

    /// Get frequently accessed keys
    async fn get_frequent_keys(&self, limit: usize) -> Result<Vec<CacheKey>>;

    /// Get all keys for eager loading
    async fn get_all_keys(&self, limit: usize) -> Result<Vec<CacheKey>>;
}

/// Cache warmer
pub struct CacheWarmer<C: Cache> {
    /// Cache to warm
    cache: Arc<C>,

    /// Data loader
    loader: Arc<dyn DataLoader>,

    /// Configuration
    config: CacheWarmingConfig,

    /// Statistics
    stats: Arc<RwLock<WarmingStats>>,

    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl<C: Cache + 'static> CacheWarmer<C> {
    /// Create a new cache warmer
    pub fn new(cache: Arc<C>, loader: Arc<dyn DataLoader>, config: CacheWarmingConfig) -> Self {
        info!("Creating cache warmer with strategy: {:?}", config.strategy);

        Self {
            cache,
            loader,
            config,
            stats: Arc::new(RwLock::new(WarmingStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start cache warming
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Cache warmer already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        match &self.config.strategy {
            WarmingStrategy::Eager => {
                self.warm_eager().await?;
            }
            WarmingStrategy::Scheduled { interval: duration } => {
                self.start_scheduled_warming(*duration).await;
            }
            WarmingStrategy::Lazy | WarmingStrategy::Predictive { .. } => {
                // These strategies don't require background tasks
                info!("Cache warmer started (passive mode)");
            }
        }

        Ok(())
    }

    /// Stop cache warming
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Cache warmer stopped");
        Ok(())
    }

    /// Warm cache eagerly (load all data)
    async fn warm_eager(&self) -> Result<()> {
        info!("Starting eager cache warming");
        let start_time = std::time::Instant::now();

        let keys = self.loader.get_all_keys(self.config.max_items).await?;
        let items_warmed = self.warm_keys(keys).await?;

        let elapsed = start_time.elapsed().as_millis() as u64;
        self.update_stats(items_warmed, elapsed).await;

        info!(
            "Eager warming completed: {} items in {}ms",
            items_warmed, elapsed
        );
        Ok(())
    }

    /// Start scheduled warming
    async fn start_scheduled_warming(&self, duration: Duration) {
        let cache = self.cache.clone();
        let loader = self.loader.clone();
        let config = self.config.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(duration);

            loop {
                ticker.tick().await;

                let is_running = *running.read().await;
                if !is_running {
                    break;
                }

                info!("Starting scheduled cache warming");
                let start_time = std::time::Instant::now();

                match loader.get_frequent_keys(config.max_items).await {
                    Ok(keys) => {
                        match Self::warm_keys_static(&cache, &loader, keys, config.batch_size).await
                        {
                            Ok(items_warmed) => {
                                let elapsed = start_time.elapsed().as_millis() as u64;
                                Self::update_stats_static(&stats, items_warmed, elapsed).await;
                                info!(
                                    "Scheduled warming completed: {} items in {}ms",
                                    items_warmed, elapsed
                                );
                            }
                            Err(e) => {
                                warn!("Scheduled warming failed: {}", e);
                                let mut stats_guard = stats.write().await;
                                stats_guard.failed_warmings += 1;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get frequent keys: {}", e);
                    }
                }
            }

            info!("Scheduled warming task stopped");
        });
    }

    /// Warm specific keys
    async fn warm_keys(&self, keys: Vec<CacheKey>) -> Result<u64> {
        Self::warm_keys_static(&self.cache, &self.loader, keys, self.config.batch_size).await
    }

    /// Static version of warm_keys for use in spawned tasks
    async fn warm_keys_static(
        cache: &Arc<C>,
        loader: &Arc<dyn DataLoader>,
        keys: Vec<CacheKey>,
        batch_size: usize,
    ) -> Result<u64> {
        let mut items_warmed = 0u64;

        for chunk in keys.chunks(batch_size) {
            let data = loader.load_data(chunk.to_vec()).await?;

            for (key, value) in data {
                if let Err(e) = cache.set(key.clone(), value, None).await {
                    warn!("Failed to warm key {}: {}", key, e);
                } else {
                    items_warmed += 1;
                    debug!("Warmed key: {}", key);
                }
            }
        }

        Ok(items_warmed)
    }

    /// Update warming statistics
    async fn update_stats(&self, items_warmed: u64, elapsed_ms: u64) {
        Self::update_stats_static(&self.stats, items_warmed, elapsed_ms).await;
    }

    /// Static version of update_stats
    async fn update_stats_static(
        stats: &Arc<RwLock<WarmingStats>>,
        items_warmed: u64,
        elapsed_ms: u64,
    ) {
        let mut stats_guard = stats.write().await;
        stats_guard.total_warmings += 1;
        stats_guard.total_items_warmed += items_warmed;
        stats_guard.total_warming_time_ms += elapsed_ms;
        stats_guard.last_warming_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                tracing::warn!("System time is before UNIX epoch: {e}, using 0 as timestamp");
                std::time::Duration::ZERO
            })
            .unwrap_or_default()
            .as_secs();
    }

    /// Get warming statistics
    pub async fn stats(&self) -> WarmingStats {
        self.stats.read().await.clone()
    }

    /// Check if warmer is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{MemoryCache, MemoryCacheConfig};

    struct MockDataLoader;

    #[async_trait::async_trait]
    impl DataLoader for MockDataLoader {
        async fn load_data(&self, keys: Vec<CacheKey>) -> Result<HashMap<CacheKey, Vec<u8>>> {
            let mut data = HashMap::new();
            for key in keys {
                data.insert(key.clone(), format!("value_{key}").into_bytes());
            }
            Ok(data)
        }

        async fn get_frequent_keys(&self, limit: usize) -> Result<Vec<CacheKey>> {
            Ok((0..limit).map(|i| format!("key_{i}")).collect())
        }

        async fn get_all_keys(&self, limit: usize) -> Result<Vec<CacheKey>> {
            Ok((0..limit).map(|i| format!("key_{i}")).collect())
        }
    }

    #[tokio::test]
    async fn test_cache_warmer_eager() -> anyhow::Result<()> {
        let cache = Arc::new(MemoryCache::new(MemoryCacheConfig::default()));
        let loader = Arc::new(MockDataLoader);
        let config = CacheWarmingConfig {
            strategy: WarmingStrategy::Eager,
            max_items: 10,
            batch_size: 5,
            enable_stats: true,
        Ok(())
        };

        let warmer = CacheWarmer::new(cache.clone(), loader, config);
        warmer.start().await?;

        // Check that keys were warmed
        let value = cache.get(&"key_0".to_string()).await?;
        assert!(value.is_some());

        let stats = warmer.stats().await;
        assert_eq!(stats.total_items_warmed, 10);
    }
}
