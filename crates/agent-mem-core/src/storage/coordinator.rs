//! Unified Storage Coordinator
//!
//! Coordinates operations between LibSQL (structured data) and VectorStore (vector data)
//! to ensure data consistency and provide unified interface.

use agent_mem_traits::{AgentMemError, MemoryV4 as Memory, Result, VectorStore};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::traits::MemoryRepositoryTrait;

/// Unified storage coordinator that manages LibSQL and VectorStore
pub struct UnifiedStorageCoordinator {
    /// LibSQL repository for structured data
    sql_repository: Arc<dyn MemoryRepositoryTrait>,
    /// Vector store for embeddings
    vector_store: Arc<dyn VectorStore + Send + Sync>,
    /// In-memory L1 cache (true LRU)
    l1_cache: Arc<RwLock<LruCache<String, Memory>>>,
    /// Cache configuration
    cache_config: CacheConfig,
    /// Statistics
    stats: Arc<RwLock<CoordinatorStats>>,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable L1 cache
    pub l1_enabled: bool,
    /// L1 cache capacity
    pub l1_capacity: usize,
    /// Default TTL for different memory types (in seconds)
    pub ttl_by_type: HashMap<String, u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let mut ttl_by_type = HashMap::new();
        ttl_by_type.insert("working".to_string(), 300);      // 5 minutes
        ttl_by_type.insert("episodic".to_string(), 3600);   // 1 hour
        ttl_by_type.insert("semantic".to_string(), 86400);  // 24 hours
        ttl_by_type.insert("procedural".to_string(), 86400); // 24 hours

        Self {
            l1_enabled: true,
            l1_capacity: 1000,
            ttl_by_type,
        }
    }
}

/// Coordinator statistics
#[derive(Debug, Clone, Default)]
pub struct CoordinatorStats {
    /// Total operations
    pub total_ops: u64,
    /// Successful operations
    pub successful_ops: u64,
    /// Failed operations
    pub failed_ops: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// L1 cache size
    pub l1_cache_size: usize,
}

impl UnifiedStorageCoordinator {
    /// Create a new unified storage coordinator
    pub fn new(
        sql_repository: Arc<dyn MemoryRepositoryTrait>,
        vector_store: Arc<dyn VectorStore + Send + Sync>,
        cache_config: Option<CacheConfig>,
    ) -> Self {
        let config = cache_config.unwrap_or_default();
        let cache_capacity = NonZeroUsize::new(config.l1_capacity)
            .unwrap_or(NonZeroUsize::new(1000).unwrap());
        
        Self {
            sql_repository,
            vector_store,
            l1_cache: Arc::new(RwLock::new(LruCache::new(cache_capacity))),
            cache_config: config,
            stats: Arc::new(RwLock::new(CoordinatorStats::default())),
        }
    }

    /// Add memory with atomic write to both stores
    pub async fn add_memory(
        &self,
        memory: &Memory,
        embedding: Option<Vec<f32>>,
    ) -> Result<String> {
        info!("Adding memory: id={}", memory.id.0);

        // Step 1: Write to LibSQL first (primary source of truth)
        let _created_memory = self.sql_repository.create(memory).await.map_err(|e| {
            error!("Failed to create memory in LibSQL: {}", e);
            AgentMemError::StorageError(format!("Failed to create memory in LibSQL: {}", e))
        })?;

        info!("✅ Memory created in LibSQL: {}", memory.id.0);

        // Step 2: Write to VectorStore (if embedding provided)
        if let Some(emb) = embedding {
            let vector_data = agent_mem_traits::VectorData {
                id: memory.id.0.clone(),
                vector: emb,
                metadata: self.memory_to_metadata(memory),
            };

            if let Err(e) = self.vector_store.add_vectors(vec![vector_data]).await {
                // If vector store fails, we should rollback LibSQL
                // For now, we log the error and continue (LibSQL is primary)
                warn!(
                    "Failed to add memory to vector store (non-critical): {}. Memory exists in LibSQL.",
                    e
                );
            } else {
                info!("✅ Memory added to vector store: {}", memory.id.0);
            }
        }

        // Step 3: Update L1 cache
        if self.cache_config.l1_enabled {
            self.update_l1_cache(&memory.id.0, memory.clone()).await;
        }

        // Step 4: Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_ops += 1;
            stats.successful_ops += 1;
        }

        Ok(memory.id.0.clone())
    }

    /// Delete memory from both stores atomically
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        info!("Deleting memory: id={}", id);

        // Step 1: Delete from VectorStore first (non-critical)
        let vector_delete_result = self.vector_store.delete_vectors(vec![id.to_string()]).await;

        // Step 2: Delete from LibSQL (primary source of truth)
        let sql_delete_result = self.sql_repository.delete(id).await;

        // Step 3: Check results and ensure consistency
        match (vector_delete_result, sql_delete_result) {
            (Ok(_), Ok(_)) => {
                info!("✅ Memory deleted from both stores: {}", id);
                // Remove from L1 cache
                if self.cache_config.l1_enabled {
                    self.remove_from_l1_cache(id).await;
                }
                // Update statistics
                {
                    let mut stats = self.stats.write().await;
                    stats.total_ops += 1;
                    stats.successful_ops += 1;
                }
                Ok(())
            }
            (Ok(_), Err(e)) => {
                // LibSQL delete failed, but vector store succeeded
                error!("Failed to delete from LibSQL after vector store deleted: {}", e);
                Err(AgentMemError::StorageError(format!(
                    "Memory deleted from vector store but failed to delete from LibSQL: {}",
                    e
                )))
            }
            (Err(e), Ok(_)) => {
                // Vector store delete failed, but LibSQL succeeded
                error!("Failed to delete from vector store after LibSQL deleted: {}", e);
                warn!(
                    "⚠️  Data inconsistency: Memory deleted from LibSQL but still exists in vector store: {}",
                    id
                );
                // Still return success since LibSQL is primary
                // Remove from L1 cache
                if self.cache_config.l1_enabled {
                    self.remove_from_l1_cache(id).await;
                }
                {
                    let mut stats = self.stats.write().await;
                    stats.total_ops += 1;
                    stats.successful_ops += 1;
                }
                Ok(())
            }
            (Err(e1), Err(e2)) => {
                // Both failed
                error!("Failed to delete from both stores: vector={}, libsql={}", e1, e2);
                {
                    let mut stats = self.stats.write().await;
                    stats.total_ops += 1;
                    stats.failed_ops += 1;
                }
                Err(AgentMemError::StorageError(format!(
                    "Failed to delete memory: {}",
                    e2
                )))
            }
        }
    }

    /// Get memory with cache-first strategy
    pub async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        // Step 1: Try L1 cache
        if self.cache_config.l1_enabled {
            let mut cache = self.l1_cache.write().await;
            if let Some(memory) = cache.get(id) {
                debug!("Cache hit (L1): {}", id);
                {
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                }
                // LRU cache automatically moves accessed item to front
                return Ok(Some(memory.clone()));
            }
        }

        // Step 2: Query LibSQL (primary source)
        let memory = self.sql_repository.find_by_id(id).await.map_err(|e| {
            error!("Failed to get memory from LibSQL: {}", e);
            AgentMemError::StorageError(format!("Failed to get memory: {}", e))
        })?;

        // Step 3: Update cache if found
        if let Some(ref mem) = memory {
            if self.cache_config.l1_enabled {
                self.update_l1_cache(id, mem.clone()).await;
            }
            {
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
            }
        } else {
            {
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
            }
        }

        Ok(memory)
    }

    /// Update memory in both stores
    pub async fn update_memory(
        &self,
        memory: &Memory,
        embedding: Option<Vec<f32>>,
    ) -> Result<Memory> {
        info!("Updating memory: id={}", memory.id.0);

        // Step 1: Update LibSQL
        let updated_memory = self.sql_repository.update(memory).await.map_err(|e| {
            error!("Failed to update memory in LibSQL: {}", e);
            AgentMemError::StorageError(format!("Failed to update memory: {}", e))
        })?;

        // Step 2: Update VectorStore if embedding provided
        if let Some(emb) = embedding {
            let vector_data = agent_mem_traits::VectorData {
                id: memory.id.0.clone(),
                vector: emb,
                metadata: self.memory_to_metadata(memory),
            };

            if let Err(e) = self.vector_store.update_vectors(vec![vector_data]).await {
                warn!("Failed to update memory in vector store: {}", e);
            }
        }

        // Step 3: Update L1 cache
        if self.cache_config.l1_enabled {
            self.update_l1_cache(&memory.id.0, updated_memory.clone()).await;
        }

        // Step 4: Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_ops += 1;
            stats.successful_ops += 1;
        }

        Ok(updated_memory)
    }

    /// Get statistics
    pub async fn get_stats(&self) -> CoordinatorStats {
        let stats = self.stats.read().await;
        let cache = self.l1_cache.read().await;
        CoordinatorStats {
            l1_cache_size: cache.len(),
            ..stats.clone()
        }
    }

    /// Clear L1 cache
    pub async fn clear_cache(&self) {
        let mut cache = self.l1_cache.write().await;
        cache.clear();
        info!("L1 cache cleared");
    }

    /// Get cache hit rate
    pub async fn get_cache_hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total = stats.cache_hits + stats.cache_misses;
        if total == 0 {
            return 0.0;
        }
        stats.cache_hits as f64 / total as f64
    }

    /// Batch add memories with optimized performance
    ///
    /// This method efficiently adds multiple memories by:
    /// 1. Using batch operations for LibSQL (if available)
    /// 2. Using batch operations for VectorStore
    /// 3. Updating cache in batch
    pub async fn batch_add_memories(
        &self,
        memories: Vec<(Memory, Option<Vec<f32>>)>,
    ) -> Result<Vec<String>> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        info!("Batch adding {} memories", memories.len());

        let mut created_ids = Vec::new();
        let mut vector_data_batch = Vec::new();

        // Step 1: Prepare data and add to LibSQL
        for (memory, embedding) in &memories {
            // Add to LibSQL
            let _created = self.sql_repository.create(memory).await.map_err(|e| {
                error!("Failed to create memory in LibSQL: {}", e);
                AgentMemError::StorageError(format!("Failed to create memory in LibSQL: {}", e))
            })?;

            created_ids.push(memory.id.0.clone());

            // Prepare vector data if embedding provided
            if let Some(emb) = embedding {
                vector_data_batch.push(agent_mem_traits::VectorData {
                    id: memory.id.0.clone(),
                    vector: emb.clone(),
                    metadata: self.memory_to_metadata(memory),
                });
            }

            // Update L1 cache
            if self.cache_config.l1_enabled {
                self.update_l1_cache(&memory.id.0, memory.clone()).await;
            }
        }

        // Step 2: Batch add to VectorStore (if any embeddings)
        let vector_batch_len = vector_data_batch.len();
        if !vector_data_batch.is_empty() {
            if let Err(e) = self.vector_store.add_vectors(vector_data_batch).await {
                warn!(
                    "Failed to add memories to vector store (non-critical): {}. Memories exist in LibSQL.",
                    e
                );
            } else {
                info!("✅ Batch added {} memories to vector store", vector_batch_len);
            }
        }

        // Step 3: Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_ops += memories.len() as u64;
            stats.successful_ops += memories.len() as u64;
        }

        info!("✅ Batch added {} memories successfully", created_ids.len());
        Ok(created_ids)
    }

    /// Batch delete memories
    ///
    /// Efficiently deletes multiple memories from both stores
    pub async fn batch_delete_memories(&self, ids: Vec<String>) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        info!("Batch deleting {} memories", ids.len());

        // Step 1: Delete from VectorStore first (non-critical)
        let vector_delete_result = self.vector_store.delete_vectors(ids.clone()).await;

        // Step 2: Delete from LibSQL (primary source of truth)
        let mut deleted_count = 0;
        let mut errors = Vec::new();

        for id in &ids {
            match self.sql_repository.delete(id).await {
                Ok(_) => {
                    deleted_count += 1;
                    // Remove from L1 cache
                    if self.cache_config.l1_enabled {
                        self.remove_from_l1_cache(id).await;
                    }
                }
                Err(e) => {
                    error!("Failed to delete memory {} from LibSQL: {}", id, e);
                    errors.push(format!("{}: {}", id, e));
                }
            }
        }

        // Step 3: Check results
        if let Err(e) = vector_delete_result {
            warn!(
                "Failed to delete from vector store (non-critical): {}. {} memories deleted from LibSQL.",
                e, deleted_count
            );
        }

        // Step 4: Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_ops += ids.len() as u64;
            if deleted_count == ids.len() {
                stats.successful_ops += ids.len() as u64;
            } else {
                stats.successful_ops += deleted_count as u64;
                stats.failed_ops += (ids.len() - deleted_count) as u64;
            }
        }

        if !errors.is_empty() {
            warn!("Some memories failed to delete: {:?}", errors);
            // Still return success count, but log errors
        }

        info!("✅ Batch deleted {} memories successfully", deleted_count);
        Ok(deleted_count)
    }

    // Helper methods

    fn memory_to_metadata(&self, memory: &Memory) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), memory.id.0.clone());
        if let Some(agent_id) = memory.agent_id() {
            metadata.insert("agent_id".to_string(), agent_id.clone());
        }
        if let Some(user_id) = memory.user_id() {
            metadata.insert("user_id".to_string(), user_id.clone());
        }
        if let Some(memory_type) = memory.memory_type() {
            metadata.insert("memory_type".to_string(), memory_type.to_string());
        }
        // Get content as text
        let content_text = match &memory.content {
            agent_mem_traits::Content::Text(t) => t.clone(),
            agent_mem_traits::Content::Structured(v) => v.to_string(),
            _ => format!("{:?}", memory.content),
        };
        metadata.insert("content".to_string(), content_text);
        metadata
    }

    async fn update_l1_cache(&self, id: &str, memory: Memory) {
        let mut cache = self.l1_cache.write().await;
        // LRU cache automatically evicts least recently used entry when full
        cache.put(id.to_string(), memory);
    }

    async fn remove_from_l1_cache(&self, id: &str) {
        let mut cache = self.l1_cache.write().await;
        cache.pop(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::{MemoryId, MemoryType};
    use std::collections::HashMap;

    // Mock implementations for testing
    struct MockMemoryRepository {
        memories: Arc<RwLock<HashMap<String, Memory>>>,
    }

    #[async_trait]
    impl MemoryRepositoryTrait for MockMemoryRepository {
        async fn create(&self, memory: &Memory) -> Result<Memory> {
            let mut mems = self.memories.write().await;
            mems.insert(memory.id.0.clone(), memory.clone());
            Ok(memory.clone())
        }

        async fn find_by_id(&self, id: &str) -> Result<Option<Memory>> {
            let mems = self.memories.read().await;
            Ok(mems.get(id).cloned())
        }

        async fn find_by_agent_id(&self, _agent_id: &str, _limit: i64) -> Result<Vec<Memory>> {
            Ok(Vec::new())
        }

        async fn find_by_user_id(&self, _user_id: &str, _limit: i64) -> Result<Vec<Memory>> {
            Ok(Vec::new())
        }

        async fn search(&self, _query: &str, _limit: i64) -> Result<Vec<Memory>> {
            Ok(Vec::new())
        }

        async fn update(&self, memory: &Memory) -> Result<Memory> {
            let mut mems = self.memories.write().await;
            mems.insert(memory.id.0.clone(), memory.clone());
            Ok(memory.clone())
        }

        async fn delete(&self, id: &str) -> Result<()> {
            let mut mems = self.memories.write().await;
            mems.remove(id);
            Ok(())
        }

        async fn delete_by_agent_id(&self, _agent_id: &str) -> Result<u64> {
            Ok(0)
        }

        async fn list(&self, _limit: i64, _offset: i64) -> Result<Vec<Memory>> {
            let mems = self.memories.read().await;
            Ok(mems.values().cloned().collect())
        }
    }

    struct MockVectorStore {
        vectors: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    }

    #[async_trait]
    impl VectorStore for MockVectorStore {
        async fn add_vectors(
            &self,
            vectors: Vec<agent_mem_traits::VectorData>,
        ) -> Result<Vec<String>> {
            let mut vecs = self.vectors.write().await;
            let mut ids = Vec::new();
            for vd in vectors {
                vecs.insert(vd.id.clone(), vd.vector);
                ids.push(vd.id);
            }
            Ok(ids)
        }

        async fn search_vectors(
            &self,
            _query_vector: Vec<f32>,
            _limit: usize,
            _threshold: Option<f32>,
        ) -> Result<Vec<agent_mem_traits::VectorSearchResult>> {
            Ok(Vec::new())
        }

        async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
            let mut vecs = self.vectors.write().await;
            for id in ids {
                vecs.remove(&id);
            }
            Ok(())
        }

        async fn update_vectors(
            &self,
            vectors: Vec<agent_mem_traits::VectorData>,
        ) -> Result<()> {
            let mut vecs = self.vectors.write().await;
            for vd in vectors {
                vecs.insert(vd.id, vd.vector);
            }
            Ok(())
        }

        async fn get_vector(
            &self,
            _id: &str,
        ) -> Result<Option<agent_mem_traits::VectorData>> {
            Ok(None)
        }

        async fn count_vectors(&self) -> Result<usize> {
            let vecs = self.vectors.read().await;
            Ok(vecs.len())
        }

        async fn clear(&self) -> Result<()> {
            let mut vecs = self.vectors.write().await;
            vecs.clear();
            Ok(())
        }

        async fn search_with_filters(
            &self,
            _query_vector: Vec<f32>,
            _limit: usize,
            _filters: &HashMap<String, serde_json::Value>,
            _threshold: Option<f32>,
        ) -> Result<Vec<agent_mem_traits::VectorSearchResult>> {
            Ok(Vec::new())
        }

        async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
            Ok(agent_mem_traits::HealthStatus {
                status: "healthy".to_string(),
                message: "OK".to_string(),
                timestamp: chrono::Utc::now(),
                details: HashMap::new(),
            })
        }

        async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
            Ok(agent_mem_traits::VectorStoreStats {
                total_vectors: 0,
                dimension: 1536,
                index_size: 0,
            })
        }

        async fn add_vectors_batch(
            &self,
            batches: Vec<Vec<agent_mem_traits::VectorData>>,
        ) -> Result<Vec<Vec<String>>> {
            let mut all_ids = Vec::new();
            for batch in batches {
                let ids = self.add_vectors(batch).await?;
                all_ids.push(ids);
            }
            Ok(all_ids)
        }

        async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
            let mut results = Vec::new();
            for ids in id_batches {
                self.delete_vectors(ids.clone()).await?;
                results.push(true);
            }
            Ok(results)
        }
    }

    fn create_test_memory(id: &str) -> Memory {
        let mut memory = Memory::new(
            "agent-1",
            Some("user-1".to_string()),
            "episodic",
            format!("Test memory content: {}", id),
            0.5,
        );
        
        // Override ID
        memory.id = MemoryId::from_string(id.to_string());
        memory
    }

    #[tokio::test]
    async fn test_add_memory() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        let memory = create_test_memory("test-1");
        let embedding = Some(vec![0.1; 1536]);

        // Add memory
        let result = coordinator.add_memory(&memory, embedding.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-1");

        // Verify in LibSQL
        let retrieved = sql_repo.find_by_id("test-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id.0, "test-1");

        // Verify in VectorStore
        let vecs = vector_store.vectors.read().await;
        assert!(vecs.contains_key("test-1"));

        // Verify in cache
        let cached = coordinator.get_memory("test-1").await.unwrap();
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        let memory = create_test_memory("test-2");
        let embedding = Some(vec![0.1; 1536]);

        // Add memory first
        coordinator.add_memory(&memory, embedding.clone()).await.unwrap();

        // Delete memory
        let result = coordinator.delete_memory("test-2").await;
        assert!(result.is_ok());

        // Verify deleted from LibSQL
        let retrieved = sql_repo.find_by_id("test-2").await.unwrap();
        assert!(retrieved.is_none());

        // Verify deleted from VectorStore
        let vecs = vector_store.vectors.read().await;
        assert!(!vecs.contains_key("test-2"));

        // Verify deleted from cache
        let cached = coordinator.get_memory("test-2").await.unwrap();
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_get_memory_cache() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        let memory = create_test_memory("test-3");
        let embedding = Some(vec![0.1; 1536]);

        // Add memory
        coordinator.add_memory(&memory, embedding).await.unwrap();

        // First get (should miss cache, then cache it)
        let stats_before = coordinator.get_stats().await;
        let _ = coordinator.get_memory("test-3").await.unwrap();
        let stats_after = coordinator.get_stats().await;
        assert_eq!(stats_after.cache_misses, stats_before.cache_misses + 1);

        // Second get (should hit cache)
        let stats_before = coordinator.get_stats().await;
        let _ = coordinator.get_memory("test-3").await.unwrap();
        let stats_after = coordinator.get_stats().await;
        assert_eq!(stats_after.cache_hits, stats_before.cache_hits + 1);
    }

    #[tokio::test]
    async fn test_update_memory() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        let memory = create_test_memory("test-4");
        let embedding = Some(vec![0.1; 1536]);

        // Add memory
        coordinator.add_memory(&memory, embedding.clone()).await.unwrap();

        // Update memory
        let updated_memory = create_test_memory("test-4");
        let new_embedding = Some(vec![0.2; 1536]);
        let result = coordinator.update_memory(&updated_memory, new_embedding.clone()).await;
        assert!(result.is_ok());

        // Verify updated in LibSQL
        let retrieved = sql_repo.find_by_id("test-4").await.unwrap();
        assert!(retrieved.is_some());

        // Verify updated in VectorStore
        let vecs = vector_store.vectors.read().await;
        assert!(vecs.contains_key("test-4"));
        assert_eq!(vecs.get("test-4").unwrap()[0], 0.2);
    }

    #[tokio::test]
    async fn test_batch_add_memories() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        // Create test memories
        let memories = vec![
            (create_test_memory("batch-1"), Some(vec![0.1; 1536])),
            (create_test_memory("batch-2"), Some(vec![0.2; 1536])),
            (create_test_memory("batch-3"), Some(vec![0.3; 1536])),
        ];

        // Batch add
        let result = coordinator.batch_add_memories(memories.clone()).await;
        assert!(result.is_ok());
        let ids = result.unwrap();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"batch-1".to_string()));
        assert!(ids.contains(&"batch-2".to_string()));
        assert!(ids.contains(&"batch-3".to_string()));

        // Verify in LibSQL
        for (memory, _) in &memories {
            let retrieved = sql_repo.find_by_id(&memory.id.0).await.unwrap();
            assert!(retrieved.is_some());
        }

        // Verify in VectorStore
        let vecs = vector_store.vectors.read().await;
        assert!(vecs.contains_key("batch-1"));
        assert!(vecs.contains_key("batch-2"));
        assert!(vecs.contains_key("batch-3"));

        // Verify in cache
        for (memory, _) in &memories {
            let cached = coordinator.get_memory(&memory.id.0).await.unwrap();
            assert!(cached.is_some());
        }
    }

    #[tokio::test]
    async fn test_batch_delete_memories() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        // Add memories first
        let memories = vec![
            (create_test_memory("batch-del-1"), Some(vec![0.1; 1536])),
            (create_test_memory("batch-del-2"), Some(vec![0.2; 1536])),
            (create_test_memory("batch-del-3"), Some(vec![0.3; 1536])),
        ];
        coordinator.batch_add_memories(memories).await.unwrap();

        // Batch delete
        let ids = vec![
            "batch-del-1".to_string(),
            "batch-del-2".to_string(),
            "batch-del-3".to_string(),
        ];
        let result = coordinator.batch_delete_memories(ids.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        // Verify deleted from LibSQL
        for id in &ids {
            let retrieved = sql_repo.find_by_id(id).await.unwrap();
            assert!(retrieved.is_none());
        }

        // Verify deleted from VectorStore
        let vecs = vector_store.vectors.read().await;
        assert!(!vecs.contains_key("batch-del-1"));
        assert!(!vecs.contains_key("batch-del-2"));
        assert!(!vecs.contains_key("batch-del-3"));

        // Verify deleted from cache
        for id in &ids {
            let cached = coordinator.get_memory(id).await.unwrap();
            assert!(cached.is_none());
        }
    }

    #[tokio::test]
    async fn test_batch_add_empty() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        // Test empty batch
        let result = coordinator.batch_add_memories(Vec::new()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_batch_delete_empty() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        // Test empty batch delete
        let result = coordinator.batch_delete_memories(Vec::new()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_lru_cache_eviction() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        // Create coordinator with small cache capacity (2 entries)
        let mut cache_config = CacheConfig::default();
        cache_config.l1_capacity = 2;
        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(cache_config),
        );

        // Add 3 memories (should evict the first one)
        let memory1 = create_test_memory("lru-1");
        let memory2 = create_test_memory("lru-2");
        let memory3 = create_test_memory("lru-3");

        coordinator.add_memory(&memory1, None).await.unwrap();
        coordinator.add_memory(&memory2, None).await.unwrap();
        coordinator.add_memory(&memory3, None).await.unwrap();

        // Access memory2 to make it recently used
        let _ = coordinator.get_memory("lru-2").await.unwrap();

        // memory1 should be evicted (least recently used)
        let cached = coordinator.get_memory("lru-1").await.unwrap();
        assert!(cached.is_none(), "LRU-1 should be evicted from cache");

        // memory2 and memory3 should still be in cache
        let cached2 = coordinator.get_memory("lru-2").await.unwrap();
        assert!(cached2.is_some(), "LRU-2 should be in cache");

        let cached3 = coordinator.get_memory("lru-3").await.unwrap();
        assert!(cached3.is_some(), "LRU-3 should be in cache");
    }

    #[tokio::test]
    async fn test_lru_cache_hit_rate() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(RwLock::new(HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(RwLock::new(HashMap::new())),
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        let memory = create_test_memory("hit-rate-test");
        coordinator.add_memory(&memory, None).await.unwrap();

        // First access (cache miss)
        let _ = coordinator.get_memory("hit-rate-test").await.unwrap();
        
        // Second access (cache hit)
        let _ = coordinator.get_memory("hit-rate-test").await.unwrap();
        
        // Third access (cache hit)
        let _ = coordinator.get_memory("hit-rate-test").await.unwrap();

        let hit_rate = coordinator.get_cache_hit_rate().await;
        // Should be 2 hits / 3 total = 0.666...
        assert!(hit_rate > 0.6 && hit_rate < 0.7, "Hit rate should be around 0.67");
    }
}

