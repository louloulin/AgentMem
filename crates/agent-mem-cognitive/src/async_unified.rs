//! Async Unified Memory Manager
//! 
//! Async version of UnifiedMemoryManager for non-blocking operations

use crate::error::{MemoryError, Result};
use crate::hierarchy::{MemoryTier, TieredMemoryItem, MemoryHierarchy};
use crate::tiering::{SmartTiering, TieringConfig};
use crate::archive::{ArchiveMemoryManager, ArchiveConfig};
use crate::review::{ReviewTriggerManager, ReviewConfig, ReviewTrigger};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Async unified configuration
#[derive(Debug, Clone)]
pub struct AsyncUnifiedConfig {
    pub hierarchy: AsyncHierarchyConfig,
    pub tiering: TieringConfig,
    pub archive: ArchiveConfig,
    pub review: ReviewConfig,
}

#[derive(Debug, Clone)]
pub struct AsyncHierarchyConfig {
    pub working_capacity: usize,
    pub core_capacity: usize,
}

impl Default for AsyncUnifiedConfig {
    fn default() -> Self {
        Self {
            hierarchy: AsyncHierarchyConfig {
                working_capacity: 100,
                core_capacity: 1000,
            },
            tiering: TieringConfig::default(),
            archive: ArchiveConfig::default(),
            review: ReviewConfig::default(),
        }
    }
}

/// Async unified memory manager
pub struct AsyncUnifiedMemoryManager {
    hierarchy: Arc<RwLock<MemoryHierarchy>>,
    tiering: Arc<SmartTiering>,
    archive: Arc<RwLock<ArchiveMemoryManager>>,
    review: Arc<ReviewTriggerManager>,
}

impl AsyncUnifiedMemoryManager {
    /// Create new async manager
    pub fn new(config: AsyncUnifiedConfig) -> Self {
        Self {
            hierarchy: Arc::new(RwLock::new(MemoryHierarchy::new(
                config.hierarchy.working_capacity,
                config.hierarchy.core_capacity,
            ))),
            tiering: Arc::new(SmartTiering::new(config.tiering)),
            archive: Arc::new(RwLock::new(ArchiveMemoryManager::new(config.archive))),
            review: Arc::new(ReviewTriggerManager::new(config.review)),
        }
    }
    
    /// Create with defaults
    pub fn with_defaults() -> Self {
        Self::new(AsyncUnifiedConfig::default())
    }
    
    /// Add memory (async)
    pub async fn add(&self, id: String, content: String, importance: f32) -> Result<()> {
        let mut item = TieredMemoryItem::new(id.clone(), content, MemoryTier::Working);
        item.importance = importance;
        
        // Check capacity first
        {
            let hierarchy = self.hierarchy.read().await;
            let stats = hierarchy.stats();
            if stats.working_count >= stats.working_capacity {
                return Err(MemoryError::capacity_exceeded(
                    "Working",
                    stats.working_count,
                    stats.working_capacity,
                ));
            }
        }
        
        // Add item
        {
            let mut hierarchy = self.hierarchy.write().await;
            hierarchy.add(item);
        }
        
        // Register for review
        self.review.register(id, String::new(), importance);
        
        Ok(())
    }
    
    /// Access memory (async)
    pub async fn access(&self, id: &str) -> Result<Option<String>> {
        // Clone the content before releasing the lock
        let content = {
            let mut hierarchy = self.hierarchy.write().await;
            let item = hierarchy.access(id);
            
            if let Some(item) = item {
                item.access_count += 1;
                item.last_accessed = Utc::now().timestamp();
                Some(item.content.clone())
            } else {
                None
            }
        };
        
        if content.is_some() {
            self.review.record_access(id);
        }
        
        Ok(content)
    }
    
    /// Search memory (async)
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<AsyncSearchResult>> {
        let query_lower = query.to_lowercase();
        
        // Search hierarchy
        let mut results = {
            let hierarchy = self.hierarchy.read().await;
            
            // Search working tier
            let mut found = Vec::new();
            for item in hierarchy.get_tier(MemoryTier::Working) {
                if item.content.to_lowercase().contains(&query_lower) {
                    found.push(AsyncSearchResult {
                        id: item.id.clone(),
                        content: item.content.clone(),
                        tier: MemoryTier::Working,
                        relevance: self.calculate_relevance(item, query),
                    });
                }
            }
            
            // Search core tier
            for item in hierarchy.get_tier(MemoryTier::Core) {
                if item.content.to_lowercase().contains(&query_lower) {
                    found.push(AsyncSearchResult {
                        id: item.id.clone(),
                        content: item.content.clone(),
                        tier: MemoryTier::Core,
                        relevance: self.calculate_relevance(item, query),
                    });
                }
            }
            
            found
        };
        
        // Search archive
        {
            let archive = self.archive.read().await;
            for item in archive.search(query, limit, 0.3) {
                results.push(AsyncSearchResult {
                    id: item.id.clone(),
                    content: item.content.clone(),
                    tier: MemoryTier::Archive,
                    relevance: 0.5,
                });
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        results.truncate(limit);
        
        Ok(results)
    }
    
    fn calculate_relevance(&self, item: &TieredMemoryItem, query: &str) -> f32 {
        let mut score = item.importance;
        if item.content.to_lowercase().contains(&query.to_lowercase()) {
            score += 0.3;
        }
        score += (item.access_count as f32 / 100.0).min(0.2);
        score
    }
    
    /// Review memory (async)
    pub async fn review(&self, id: &str) -> Result<bool> {
        Ok(self.review.review(id))
    }
    
    /// Get pending reviews (async)
    pub async fn get_pending_reviews(&self, limit: usize) -> Vec<ReviewTrigger> {
        self.review.get_pending_reviews(limit)
    }
    
    /// Rebalance hierarchy (async)
    pub async fn rebalance(&self) {
        let mut hierarchy = self.hierarchy.write().await;
        self.tiering.rebalance(&mut hierarchy);
    }
    
    /// Get stats (async)
    pub async fn stats(&self) -> AsyncUnifiedStats {
        let hierarchy_stats = {
            let hierarchy = self.hierarchy.read().await;
            hierarchy.stats()
        };
        
        let archive_stats = {
            let archive = self.archive.read().await;
            archive.stats()
        };
        
        let review_stats = self.review.stats();
        
        AsyncUnifiedStats {
            working_count: hierarchy_stats.working_count,
            working_capacity: hierarchy_stats.working_capacity,
            core_count: hierarchy_stats.core_count,
            core_capacity: hierarchy_stats.core_capacity,
            archive_count: archive_stats.total_items,
            pending_reviews: review_stats.pending_reviews,
            avg_retention: review_stats.avg_retention,
        }
    }
    
    /// Delete memory (async)
    pub async fn delete(&self, id: &str) -> Result<bool> {
        // Search in hierarchy
        let found = {
            let hierarchy = self.hierarchy.read().await;
            
            // Try working tier
            let working_items: Vec<_> = hierarchy.get_tier(MemoryTier::Working).to_vec();
            for item in working_items {
                if item.id == id {
                    return Ok(true);
                }
            }
            
            // Try core tier
            let core_items: Vec<_> = hierarchy.get_tier(MemoryTier::Core).to_vec();
            for item in core_items {
                if item.id == id {
                    return Ok(true);
                }
            }
            
            false
        };
        
        Ok(found)
    }
}

/// Async search result
#[derive(Debug, Clone)]
pub struct AsyncSearchResult {
    pub id: String,
    pub content: String,
    pub tier: MemoryTier,
    pub relevance: f32,
}

/// Async unified stats
#[derive(Debug, Clone)]
pub struct AsyncUnifiedStats {
    pub working_count: usize,
    pub working_capacity: usize,
    pub core_count: usize,
    pub core_capacity: usize,
    pub archive_count: usize,
    pub pending_reviews: usize,
    pub avg_retention: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_access() {
        let manager = AsyncUnifiedMemoryManager::with_defaults();
        
        manager.add("test1".into(), "Hello world".into(), 0.8).await.unwrap();
        
        let result = manager.access("test1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Hello world");
    }
    
    #[tokio::test]
    async fn test_search() {
        let manager = AsyncUnifiedMemoryManager::with_defaults();
        
        manager.add("test1".into(), "Rust programming".into(), 0.8).await.unwrap();
        manager.add("test2".into(), "Python programming".into(), 0.7).await.unwrap();
        
        let results = manager.search("Rust", 10).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("Rust"));
    }
    
    #[tokio::test]
    async fn test_stats() {
        let manager = AsyncUnifiedMemoryManager::with_defaults();
        
        manager.add("test1".into(), "Hello".into(), 0.8).await.unwrap();
        
        let stats = manager.stats().await;
        assert_eq!(stats.working_count, 1);
        assert_eq!(stats.working_capacity, 100);
    }
}
