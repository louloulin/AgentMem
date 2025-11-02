//! Multi-level cache implementation
//!
//! Provides a hierarchical caching system with:
//! - L1: Fast in-memory cache
//! - L2: Distributed Redis cache (optional)
//! - Automatic promotion/demotion between levels
//! - Performance monitoring
//! - Unified interface

use super::{Cache, CacheKey, CacheStats, CacheMonitor, MemoryCache, MemoryCacheConfig, MonitorConfig};
use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Cache level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheLevel {
    /// L1: In-memory cache (fastest)
    L1,
    
    /// L2: Redis cache (fast, distributed)
    L2,
}

/// Multi-level cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLevelCacheConfig {
    /// Enable L1 cache
    pub enable_l1: bool,
    
    /// Enable L2 cache
    pub enable_l2: bool,
    
    /// L1 configuration
    pub l1_config: MemoryCacheConfig,
    
    /// L2 Redis URL
    pub l2_redis_url: Option<String>,
    
    /// L2 default TTL
    pub l2_default_ttl: Duration,
    
    /// Promote L2 hits to L1
    pub promote_on_hit: bool,
    
    /// Write-through to L2
    pub write_through: bool,
    
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    
    /// Monitor configuration
    pub monitor_config: Option<MonitorConfig>,
}

impl Default for MultiLevelCacheConfig {
    fn default() -> Self {
        Self {
            enable_l1: true,
            enable_l2: false,
            l1_config: MemoryCacheConfig::default(),
            l2_redis_url: None,
            l2_default_ttl: Duration::from_secs(30 * 60), // 30 minutes
            promote_on_hit: true,
            write_through: true,
            enable_monitoring: false,
            monitor_config: None,
        }
    }
}

/// Multi-level cache
pub struct MultiLevelCache {
    /// L1 cache (always present)
    l1: Arc<MemoryCache>,
    
    /// L2 cache (optional)
    l2: Option<Arc<dyn Cache>>,
    
    /// Configuration
    config: MultiLevelCacheConfig,
    
    /// Performance monitor (optional)
    monitor: Option<Arc<CacheMonitor>>,
}

impl MultiLevelCache {
    /// Create a new multi-level cache
    pub fn new(config: MultiLevelCacheConfig) -> Self {
        info!("Creating multi-level cache (L1: {}, L2: {}, Monitoring: {})", 
              config.enable_l1, config.enable_l2, config.enable_monitoring);
        
        let l1 = Arc::new(MemoryCache::new(config.l1_config.clone()));
        
        // L2 cache would be initialized here if Redis is available
        // For now, we'll leave it as None
        let l2 = None;
        
        // Initialize monitor if enabled
        let monitor = if config.enable_monitoring {
            let monitor_config = config.monitor_config.clone()
                .unwrap_or_else(MonitorConfig::default);
            Some(Arc::new(CacheMonitor::new(monitor_config)))
        } else {
            None
        };
        
        Self { l1, l2, config, monitor }
    }
    
    /// Get from L1, fallback to L2
    async fn get_multi_level(&self, key: &CacheKey) -> Result<Option<Vec<u8>>> {
        let start = Instant::now();
        let mut hit = false;
        let mut cache_level = None;
        
        // Try L1 first
        if self.config.enable_l1 {
            if let Some(value) = self.l1.get(key).await? {
                debug!("Multi-level cache hit (L1): {}", key);
                hit = true;
                cache_level = Some(CacheLevel::L1);
                
                // Record performance
                if let Some(monitor) = &self.monitor {
                    monitor.record_operation(start.elapsed(), hit, cache_level).await;
                }
                
                return Ok(Some(value));
            }
        }
        
        // Try L2 if enabled
        if self.config.enable_l2 {
            if let Some(l2) = &self.l2 {
                if let Some(value) = l2.get(key).await? {
                    debug!("Multi-level cache hit (L2): {}", key);
                    hit = true;
                    cache_level = Some(CacheLevel::L2);
                    
                    // Promote to L1 if configured
                    if self.config.promote_on_hit && self.config.enable_l1 {
                        if let Err(e) = self.l1.set(key.clone(), value.clone(), None).await {
                            warn!("Failed to promote to L1: {}", e);
                        } else {
                            debug!("Promoted to L1: {}", key);
                        }
                    }
                    
                    // Record performance
                    if let Some(monitor) = &self.monitor {
                        monitor.record_operation(start.elapsed(), hit, cache_level).await;
                    }
                    
                    return Ok(Some(value));
                }
            }
        }
        
        debug!("Multi-level cache miss: {}", key);
        
        // Record miss
        if let Some(monitor) = &self.monitor {
            monitor.record_operation(start.elapsed(), false, None).await;
        }
        
        Ok(None)
    }
    
    /// Set to both levels
    async fn set_multi_level(&self, key: CacheKey, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        // Set to L1
        if self.config.enable_l1 {
            self.l1.set(key.clone(), value.clone(), ttl).await?;
        }
        
        // Set to L2 if write-through is enabled
        if self.config.enable_l2 && self.config.write_through {
            if let Some(l2) = &self.l2 {
                if let Err(e) = l2.set(key.clone(), value, ttl).await {
                    warn!("Failed to write to L2: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Delete from both levels
    async fn delete_multi_level(&self, key: &CacheKey) -> Result<bool> {
        let mut deleted = false;
        
        // Delete from L1
        if self.config.enable_l1 {
            deleted |= self.l1.delete(key).await?;
        }
        
        // Delete from L2
        if self.config.enable_l2 {
            if let Some(l2) = &self.l2 {
                if let Err(e) = l2.delete(key).await {
                    warn!("Failed to delete from L2: {}", e);
                } else {
                    deleted = true;
                }
            }
        }
        
        Ok(deleted)
    }
    
    /// Get combined statistics
    pub async fn combined_stats(&self) -> Result<CacheStats> {
        let mut combined = CacheStats::default();
        
        // Get L1 stats
        if self.config.enable_l1 {
            let l1_stats = self.l1.stats().await?;
            combined.merge(&l1_stats);
        }
        
        // Get L2 stats
        if self.config.enable_l2 {
            if let Some(l2) = &self.l2 {
                if let Ok(l2_stats) = l2.stats().await {
                    combined.merge(&l2_stats);
                }
            }
        }
        
        Ok(combined)
    }
    
    /// Get L1 cache reference
    pub fn l1_cache(&self) -> Arc<MemoryCache> {
        self.l1.clone()
    }
    
    /// Check if L2 is enabled and available
    pub fn has_l2(&self) -> bool {
        self.config.enable_l2 && self.l2.is_some()
    }
    
    /// Get monitor reference
    pub fn monitor(&self) -> Option<Arc<CacheMonitor>> {
        self.monitor.clone()
    }
    
    /// Generate performance snapshot
    pub async fn performance_snapshot(&self) -> Result<Option<super::PerformanceSnapshot>> {
        if let Some(monitor) = &self.monitor {
            let l1_stats = if self.config.enable_l1 {
                Some(self.l1.stats().await?)
            } else {
                None
            };
            
            let l2_stats = if self.config.enable_l2 {
                if let Some(l2) = &self.l2 {
                    l2.stats().await.ok()
                } else {
                    None
                }
            } else {
                None
            };
            
            let combined_stats = self.combined_stats().await?;
            
            let snapshot = monitor.create_snapshot(l1_stats, l2_stats, combined_stats).await;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }
    
    /// Generate performance report
    pub async fn performance_report(&self) -> Result<Option<super::PerformanceReport>> {
        if let Some(monitor) = &self.monitor {
            Ok(monitor.generate_report().await)
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Cache for MultiLevelCache {
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>> {
        self.get_multi_level(key).await
    }
    
    async fn set(&self, key: CacheKey, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        self.set_multi_level(key, value, ttl).await
    }
    
    async fn delete(&self, key: &CacheKey) -> Result<bool> {
        self.delete_multi_level(key).await
    }
    
    async fn exists(&self, key: &CacheKey) -> Result<bool> {
        // Check L1 first
        if self.config.enable_l1
            && self.l1.exists(key).await? {
                return Ok(true);
            }
        
        // Check L2
        if self.config.enable_l2 {
            if let Some(l2) = &self.l2 {
                return l2.exists(key).await;
            }
        }
        
        Ok(false)
    }
    
    async fn clear(&self) -> Result<()> {
        // Clear L1
        if self.config.enable_l1 {
            self.l1.clear().await?;
        }
        
        // Clear L2
        if self.config.enable_l2 {
            if let Some(l2) = &self.l2 {
                l2.clear().await?;
            }
        }
        
        info!("Multi-level cache cleared");
        Ok(())
    }
    
    async fn stats(&self) -> Result<CacheStats> {
        self.combined_stats().await
    }
    
    fn level(&self) -> CacheLevel {
        // Multi-level cache spans both levels
        CacheLevel::L1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_multi_level_cache_l1_only() {
        let config = MultiLevelCacheConfig {
            enable_l1: true,
            enable_l2: false,
            ..Default::default()
        };
        
        let cache = MultiLevelCache::new(config);
        
        cache.set("key1".to_string(), b"value1".to_vec(), None).await.unwrap();
        let value = cache.get(&"key1".to_string()).await.unwrap();
        
        assert_eq!(value, Some(b"value1".to_vec()));
    }
    
    #[tokio::test]
    async fn test_multi_level_cache_stats() {
        let config = MultiLevelCacheConfig::default();
        let cache = MultiLevelCache::new(config);
        
        cache.set("key1".to_string(), b"value1".to_vec(), None).await.unwrap();
        cache.get(&"key1".to_string()).await.unwrap();
        
        let stats = cache.stats().await.unwrap();
        assert!(stats.total_sets > 0);
        assert!(stats.hits > 0);
    }
    
    #[tokio::test]
    async fn test_multi_level_cache_delete() {
        let config = MultiLevelCacheConfig::default();
        let cache = MultiLevelCache::new(config);
        
        cache.set("key1".to_string(), b"value1".to_vec(), None).await.unwrap();
        let deleted = cache.delete(&"key1".to_string()).await.unwrap();
        assert!(deleted);
        
        let value = cache.get(&"key1".to_string()).await.unwrap();
        assert_eq!(value, None);
    }
}

