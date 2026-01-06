// Hybrid storage-enabled hierarchy manager

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::{postgres::PostgresStorage, redis::RedisCache, HybridStorageManager, StorageConfig};
use crate::hierarchy::{
    HierarchicalMemory, HierarchyConfig, HierarchyManager, HierarchyStatistics, MemoryLevel,
    MemoryScope,
};
use crate::{types::*, CoreError, CoreResult};

/// Storage-enabled hierarchy manager that combines in-memory, PostgreSQL, and Redis
pub struct HybridHierarchyManager {
    /// Configuration
    config: HierarchyConfig,
    /// Storage configuration
    storage_config: StorageConfig,
    /// Hybrid storage manager
    storage: Option<HybridStorageManager>,
    /// In-memory cache for frequently accessed memories
    memory_cache: Arc<RwLock<HashMap<String, HierarchicalMemory>>>,
    /// Index by scope for faster lookups
    scope_index: Arc<RwLock<HashMap<MemoryScope, Vec<String>>>>,
    /// Index by level for faster lookups
    level_index: Arc<RwLock<HashMap<MemoryLevel, Vec<String>>>>,
    /// Whether storage is initialized
    storage_initialized: Arc<RwLock<bool>>,
}

impl HybridHierarchyManager {
    /// Create new hybrid hierarchy manager
    pub fn new(config: HierarchyConfig, storage_config: StorageConfig) -> Self {
        Self {
            config,
            storage_config,
            storage: None,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            scope_index: Arc::new(RwLock::new(HashMap::new())),
            level_index: Arc::new(RwLock::new(HashMap::new())),
            storage_initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize storage backends
    pub async fn initialize_storage(&mut self) -> CoreResult<()> {
        info!("Initializing hybrid storage backends");

        // Create PostgreSQL backend
        let postgres = PostgresStorage::new(self.storage_config.postgres.clone()).await?;

        // Create Redis cache backend
        let redis = RedisCache::new(self.storage_config.redis.clone()).await?;

        // Create hybrid storage manager
        let storage = HybridStorageManager::new(
            Box::new(postgres),
            Box::new(redis),
            self.storage_config.clone(),
        );

        // Initialize storage
        storage.initialize().await?;

        self.storage = Some(storage);

        // Mark as initialized
        {
            let mut initialized = self.storage_initialized.write().await;
            *initialized = true;
        }

        info!("Hybrid storage backends initialized successfully");
        Ok(())
    }

    /// Check if storage is initialized
    async fn ensure_storage_initialized(&self) -> CoreResult<()> {
        let initialized = self.storage_initialized.read().await;
        if !*initialized {
            return Err(CoreError::Storage(
                "Storage not initialized. Call initialize_storage() first.".to_string(),
            ));
        }
        Ok(())
    }

    /// Get storage reference
    fn get_storage(&self) -> CoreResult<&HybridStorageManager> {
        self.storage
            .as_ref()
            .ok_or_else(|| CoreError::Storage("Storage not initialized".to_string()))
    }

    /// Update in-memory indexes
    async fn update_indexes(&self, memory: &HierarchicalMemory) -> CoreResult<()> {
        // Update scope index
        {
            let mut scope_index = self.scope_index.write().await;
            scope_index
                .entry(memory.scope.clone())
                .or_insert_with(Vec::new)
                .push(memory.memory.id.clone());
        }

        // Update level index
        {
            let mut level_index = self.level_index.write().await;
            level_index
                .entry(memory.level.clone())
                .or_insert_with(Vec::new)
                .push(memory.memory.id.clone());
        }

        Ok(())
    }

    /// Remove from in-memory indexes
    async fn remove_from_indexes(
        &self,
        memory_id: &str,
        scope: &MemoryScope,
        level: MemoryLevel,
    ) -> CoreResult<()> {
        // Remove from scope index
        {
            let mut scope_index = self.scope_index.write().await;
            if let Some(ids) = scope_index.get_mut(scope) {
                ids.retain(|id| id != memory_id);
            }
        }

        // Remove from level index
        {
            let mut level_index = self.level_index.write().await;
            if let Some(ids) = level_index.get_mut(&level) {
                ids.retain(|id| id != memory_id);
            }
        }

        Ok(())
    }

    /// Get memory from cache or storage
    async fn get_memory_internal(&self, id: &str) -> CoreResult<Option<HierarchicalMemory>> {
        // Try memory cache first
        {
            let cache = self.memory_cache.read().await;
            if let Some(memory) = cache.get(id) {
                return Ok(Some(memory.clone()));
            }
        }

        // Try storage if initialized
        if let Ok(storage) = self.get_storage() {
            if let Some(memory) = storage.get_memory(id).await? {
                // Cache the result
                {
                    let mut cache = self.memory_cache.write().await;
                    cache.insert(id.to_string(), memory.clone());
                }
                return Ok(Some(memory));
            }
        }

        Ok(None)
    }

    /// Store memory in both cache and storage
    async fn store_memory_internal(&self, memory: &HierarchicalMemory) -> CoreResult<()> {
        // Store in memory cache
        {
            let mut cache = self.memory_cache.write().await;
            cache.insert(memory.memory.id.clone(), memory.clone());
        }

        // Store in persistent storage if initialized
        if let Ok(storage) = self.get_storage() {
            storage.store_memory(memory).await?;
        }

        // Update indexes
        self.update_indexes(memory).await?;

        Ok(())
    }

    /// Remove memory from both cache and storage
    async fn remove_memory_internal(&self, id: &str) -> CoreResult<bool> {
        // Get memory info for index cleanup
        let memory_info = self.get_memory_internal(id).await?;

        // Remove from memory cache
        {
            let mut cache = self.memory_cache.write().await;
            cache.remove(id);
        }

        // Remove from persistent storage if initialized
        let storage_removed = if let Ok(storage) = self.get_storage() {
            storage.delete_memory(id).await?
        } else {
            false
        };

        // Remove from indexes if we had the memory
        let had_memory = memory_info.is_some();
        if let Some(memory) = memory_info {
            self.remove_from_indexes(id, &memory.scope, memory.level)
                .await?;
        }

        Ok(storage_removed || had_memory)
    }

    /// Get health status of storage backends
    pub async fn get_storage_health(
        &self,
    ) -> CoreResult<(super::HealthStatus, super::HealthStatus)> {
        self.ensure_storage_initialized().await?;
        let storage = self.get_storage()?;
        storage.health_check().await
    }

    /// Get storage statistics
    pub async fn get_storage_statistics(&self) -> CoreResult<super::StorageStatistics> {
        self.ensure_storage_initialized().await?;
        let storage = self.get_storage()?;
        storage.get_statistics().await
    }

    /// Get cache statistics
    pub async fn get_cache_statistics(&self) -> CoreResult<super::CacheStatistics> {
        self.ensure_storage_initialized().await?;
        let storage = self.get_storage()?;
        storage.get_cache_statistics().await
    }

    /// Warm up cache from storage
    pub async fn warm_cache(
        &self,
        limit: Option<usize>,
    ) -> CoreResult<super::migration::MigrationProgress> {
        self.ensure_storage_initialized().await?;
        let storage = self.get_storage()?;

        let migration_manager =
            super::migration::MigrationManager::new(super::migration::MigrationConfig::default());

        migration_manager
            .warm_cache(storage.postgres.as_ref(), storage.redis.as_ref(), limit)
            .await
    }
}

#[async_trait]
impl HierarchyManager for HybridHierarchyManager {
    async fn add_memory(&self, memory: crate::Memory) -> CoreResult<HierarchicalMemory> {
        // Determine appropriate level based on importance
        let level = if memory.importance > 0.8 {
            MemoryLevel::Strategic
        } else if memory.importance > 0.6 {
            MemoryLevel::Tactical
        } else if memory.importance > 0.4 {
            MemoryLevel::Operational
        } else {
            MemoryLevel::Contextual
        };

        // Determine scope based on memory metadata
        let scope = if memory.metadata.contains_key("global") {
            MemoryScope::Global
        } else if let Some(agent_id) = memory.metadata.get("agent_id").and_then(|v| v.as_str()) {
            if let Some(user_id) = memory.metadata.get("user_id").and_then(|v| v.as_str()) {
                if let Some(session_id) = memory.metadata.get("session_id").and_then(|v| v.as_str())
                {
                    MemoryScope::Session {
                        agent_id: agent_id.to_string(),
                        user_id: user_id.to_string(),
                        session_id: session_id.to_string(),
                    }
                } else {
                    MemoryScope::User {
                        agent_id: agent_id.to_string(),
                        user_id: user_id.to_string(),
                    }
                }
            } else {
                MemoryScope::Agent(agent_id.to_string())
            }
        } else {
            MemoryScope::Global
        };

        // Convert MemoryItem to types::Memory
        let types_memory: crate::types::Memory = memory
            .try_into()
            .map_err(|e| CoreError::Storage(format!("Failed to convert memory: {}", e)))?;

        use crate::hierarchy::HierarchyMetadata;

        let hierarchical_memory = HierarchicalMemory {
            memory: types_memory.into(),
            scope,
            level,
            hierarchy_metadata: HierarchyMetadata::default(),
        };

        // Store the memory
        self.store_memory_internal(&hierarchical_memory).await?;

        info!(
            "Added memory {} at level {:?} in scope {:?}",
            hierarchical_memory.memory.id, hierarchical_memory.level, hierarchical_memory.scope
        );

        Ok(hierarchical_memory)
    }

    async fn get_memory(&self, id: &str) -> CoreResult<Option<HierarchicalMemory>> {
        self.get_memory_internal(id).await
    }

    async fn update_memory(&self, memory: HierarchicalMemory) -> CoreResult<HierarchicalMemory> {
        let memory_id = memory.memory.id.clone();

        // Update in storage
        self.store_memory_internal(&memory).await?;

        info!("Updated memory {}", memory_id);
        Ok(memory)
    }

    async fn remove_memory(&self, id: &str) -> CoreResult<bool> {
        let removed = self.remove_memory_internal(id).await?;

        if removed {
            info!("Removed memory {}", id);
        } else {
            warn!("Memory {} not found for removal", id);
        }

        Ok(removed)
    }

    async fn get_memories_at_level(
        &self,
        level: MemoryLevel,
    ) -> CoreResult<Vec<HierarchicalMemory>> {
        // Get IDs from level index
        let memory_ids = {
            let level_index = self.level_index.read().await;
            level_index.get(&level).cloned().unwrap_or_default()
        };

        let mut result = Vec::new();
        for id in memory_ids {
            if let Some(memory) = self.get_memory_internal(&id).await? {
                result.push(memory);
            }
        }

        // If storage is available, also search there for any missing memories
        if let Ok(storage) = self.get_storage() {
            let storage_memories = storage.postgres.get_memories_by_level(level, None).await?;
            for memory in storage_memories {
                if !result.iter().any(|m| m.memory.id == memory.memory.id) {
                    result.push(memory);
                }
            }
        }

        Ok(result)
    }

    async fn get_hierarchy_stats(&self) -> CoreResult<HierarchyStatistics> {
        // Get statistics from storage if available
        if let Ok(storage) = self.get_storage() {
            let storage_stats = storage.get_statistics().await?;

            // Convert storage statistics to hierarchy statistics
            let mut memories_by_level = HashMap::new();
            let mut avg_importance_by_level = HashMap::new();
            let mut level_utilization = HashMap::new();

            for (level, count) in storage_stats.memories_by_level {
                memories_by_level.insert(level.clone(), count as usize);
                avg_importance_by_level.insert(level.clone(), 0.5); // Simplified

                let max_capacity = self
                    .config
                    .level_capacities
                    .get(&level)
                    .copied()
                    .unwrap_or(1000) as f64;
                let utilization = (count as f64 / max_capacity).min(1.0);
                level_utilization.insert(level, utilization);
            }

            return Ok(HierarchyStatistics {
                memories_by_level,
                avg_importance_by_level,
                inheritance_relationships: 0, // Simplified
                level_utilization,
            });
        }

        // Fallback to in-memory statistics
        let cache = self.memory_cache.read().await;
        let mut memories_by_level = HashMap::new();
        let mut avg_importance_by_level = HashMap::new();
        let mut level_utilization = HashMap::new();

        for memory in cache.values() {
            *memories_by_level.entry(memory.level.clone()).or_insert(0) += 1;
        }

        for (level, count) in &memories_by_level {
            let total_importance: f32 = cache
                .values()
                .filter(|m| m.level == *level)
                .map(|m| m.memory.importance)
                .sum();
            let avg_importance = if *count > 0 {
                total_importance / *count as f32
            } else {
                0.0
            };
            avg_importance_by_level.insert(level.clone(), avg_importance as f64);

            let max_capacity = self
                .config
                .level_capacities
                .get(level)
                .copied()
                .unwrap_or(1000) as f64;
            let utilization = (*count as f64 / max_capacity).min(1.0);
            level_utilization.insert(level.clone(), utilization);
        }

        Ok(HierarchyStatistics {
            memories_by_level,
            avg_importance_by_level,
            inheritance_relationships: 0,
            level_utilization,
        })
    }

    async fn search_memories(
        &self,
        query: &str,
        scope: Option<MemoryScope>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<HierarchicalMemory>> {
        // Search in storage if available
        if let Ok(storage) = self.get_storage() {
            return storage.search_memories(query, scope, None, limit).await;
        }

        // Fallback to in-memory search
        let cache = self.memory_cache.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<HierarchicalMemory> = cache
            .values()
            .filter(|memory| {
                // Filter by scope if specified
                if let Some(ref target_scope) = scope {
                    if &memory.scope != target_scope {
                        return false;
                    }
                }

                // Simple text search in content
                memory.memory.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        // Sort by importance (descending)
        results.sort_by(|a, b| {
            b.memory
                .importance
                .partial_cmp(&a.memory.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}
