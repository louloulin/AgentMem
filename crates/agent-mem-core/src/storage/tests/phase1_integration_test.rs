//! 🆕 Phase 1 性能优化集成测试
//!
//! 测试所有P0任务的集成效果

#[cfg(test)]
pub mod tests {
    use super::super::coordinator::{CacheConfig, UnifiedStorageCoordinator};
    use super::super::libsql::memory_repository::LibSqlMemoryRepository;
    use agent_mem_traits::{MemoryV4 as Memory, VectorStore, VectorData};
    use std::sync::Arc;
    use std::collections::HashMap;
    use tokio::time::Instant;

    // Mock implementations for testing
    struct MockMemoryRepository {
        memories: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Memory>>>,
    }

    #[async_trait::async_trait]
    impl super::super::traits::MemoryRepositoryTrait for MockMemoryRepository {
        async fn create(&self, memory: &Memory) -> agent_mem_traits::Result<Memory> {
            // Simulate database delay
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            self.memories.write().await.insert(memory.id.as_str().to_string(), memory.clone());
            Ok(memory.clone())
        }

        async fn find_by_id(&self, id: &str) -> agent_mem_traits::Result<Option<Memory>> {
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            Ok(self.memories.read().await.get(id).cloned())
        }

        async fn batch_find_by_ids(&self, ids: &[String]) -> agent_mem_traits::Result<Vec<Memory>> {
            // Simulate batch query (faster than individual queries)
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let memories = self.memories.read().await;
            Ok(ids.iter()
                .filter_map(|id| memories.get(id).cloned())
                .collect())
        }

        async fn find_by_agent_id(&self, _agent_id: &str, _limit: i64) -> agent_mem_traits::Result<Vec<Memory>> {
            Ok(Vec::new())
        }

        async fn find_by_user_id(&self, _user_id: &str, _limit: i64) -> agent_mem_traits::Result<Vec<Memory>> {
            Ok(Vec::new())
        }

        async fn search(&self, _query: &str, _limit: i64) -> agent_mem_traits::Result<Vec<Memory>> {
            Ok(Vec::new())
        }

        async fn update(&self, memory: &Memory) -> agent_mem_traits::Result<Memory> {
            self.memories.write().await.insert(memory.id.as_str().to_string(), memory.clone());
            Ok(memory.clone())
        }

        async fn delete(&self, id: &str) -> agent_mem_traits::Result<()> {
            self.memories.write().await.remove(id);
            Ok(())
        }

        async fn delete_by_agent_id(&self, _agent_id: &str) -> agent_mem_traits::Result<u64> {
            Ok(0)
        }

        async fn list(&self, _limit: i64, _offset: i64) -> agent_mem_traits::Result<Vec<Memory>> {
            Ok(self.memories.read().await.values().cloned().collect())
        }
    }

    struct MockVectorStore {
        vectors: Arc<tokio::sync::RwLock<std::collections::HashMap<String, VectorData>>>,
        add_delay_ms: u64,
    }

    #[async_trait::async_trait]
    impl VectorStore for MockVectorStore {
        async fn add_vectors(&self, vectors: Vec<VectorData>) -> agent_mem_traits::Result<Vec<String>> {
            // Simulate vector store delay
            tokio::time::sleep(tokio::time::Duration::from_millis(self.add_delay_ms)).await;
            let mut vecs = self.vectors.write().await;
            for v in vectors {
                vecs.insert(v.id.clone(), v);
            }
            Ok(vecs.keys().cloned().collect())
        }

        async fn search_vectors(
            &self,
            _query_vector: Vec<f32>,
            _limit: usize,
            _threshold: Option<f32>,
        ) -> agent_mem_traits::Result<Vec<agent_mem_traits::VectorSearchResult>> {
            Ok(Vec::new())
        }

        async fn delete_vectors(&self, ids: Vec<String>) -> agent_mem_traits::Result<()> {
            let mut vecs = self.vectors.write().await;
            for id in ids {
                vecs.remove(&id);
            }
            Ok(())
        }

        async fn update_vectors(&self, vectors: Vec<VectorData>) -> agent_mem_traits::Result<()> {
            let mut vecs = self.vectors.write().await;
            for v in vectors {
                vecs.insert(v.id.clone(), v);
            }
            Ok(())
        }

        async fn clear(&self) -> agent_mem_traits::Result<()> {
            self.vectors.write().await.clear();
            Ok(())
        }

        async fn search_with_filters(
            &self,
            _query_vector: Vec<f32>,
            _limit: usize,
            _filters: &std::collections::HashMap<String, serde_json::Value>,
            _threshold: Option<f32>,
        ) -> agent_mem_traits::Result<Vec<agent_mem_traits::VectorSearchResult>> {
            Ok(Vec::new())
        }

        async fn get_vector(&self, id: &str) -> agent_mem_traits::Result<Option<VectorData>> {
            let vecs = self.vectors.read().await;
            Ok(vecs.get(id).cloned())
        }

        async fn count_vectors(&self) -> agent_mem_traits::Result<usize> {
            Ok(self.vectors.read().await.len())
        }

        async fn health_check(&self) -> agent_mem_traits::Result<agent_mem_traits::HealthStatus> {
            Ok(agent_mem_traits::HealthStatus {
                status: "healthy".to_string(),
                message: "OK".to_string(),
                timestamp: chrono::Utc::now(),
                details: std::collections::HashMap::new(),
            })
        }

        async fn get_stats(&self) -> agent_mem_traits::Result<agent_mem_traits::VectorStoreStats> {
            Ok(agent_mem_traits::VectorStoreStats {
                total_vectors: self.vectors.read().await.len(),
                dimension: 384,
                index_size: 0,
            })
        }

        async fn add_vectors_batch(
            &self,
            batches: Vec<Vec<VectorData>>,
        ) -> agent_mem_traits::Result<Vec<Vec<String>>> {
            let mut results = Vec::new();
            for batch in batches {
                let result = self.add_vectors(batch).await?;
                results.push(result);
            }
            Ok(results)
        }

        async fn delete_vectors_batch(
            &self,
            batches: Vec<Vec<String>>,
        ) -> agent_mem_traits::Result<Vec<bool>> {
            let mut results = Vec::new();
            for batch in batches {
                let result = self.delete_vectors(batch).await;
                results.push(result.is_ok());
            }
            Ok(results)
        }
    }

    fn create_test_memory(id: &str) -> Memory {
        // 复用coordinator.rs中的create_test_memory实现
        let mut memory = Memory::new(
            "agent-1",
            Some("user-1".to_string()),
            "episodic",
            format!("Test memory content: {}", id),
            0.5,
        );
        
        // Override ID
        use agent_mem_traits::abstractions::MemoryId;
        memory.id = MemoryId::from_string(id.to_string());
        memory
    }

    /// 测试1.1: 并行存储优化
    /// 验证并行执行比串行执行快
    #[tokio::test]
    async fn test_parallel_storage_performance() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            add_delay_ms: 50, // 50ms delay
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        let memory = create_test_memory("test-parallel");
        let embedding = Some(vec![0.5; 384]);

        // 测试并行存储
        let start = Instant::now();
        let result = coordinator.add_memory(&memory, embedding.clone()).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 并行执行应该接近max(20ms, 50ms) = 50ms，而不是sum(70ms)
        // 允许一些 overhead
        assert!(elapsed.as_millis() < 70, "Parallel storage should be faster than sequential");
        println!("✅ Parallel storage completed in {}ms", elapsed.as_millis());
    }

    /// 测试1.6: 批量查询优化
    /// 验证批量查询比循环查询快
    #[tokio::test]
    async fn test_batch_query_performance() {
        let sql_repo = Arc::new(MockMemoryRepository {
            memories: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        });
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            add_delay_ms: 0,
        });

        let coordinator = UnifiedStorageCoordinator::new(
            sql_repo.clone(),
            vector_store.clone(),
            Some(CacheConfig::default()),
        );

        // 创建测试记忆
        let ids: Vec<String> = (0..100).map(|i| format!("mem_{}", i)).collect();
        for id in &ids {
            let memory = create_test_memory(id);
            sql_repo.memories.write().await.insert(id.clone(), memory);
        }

        // 测试批量查询
        let start = Instant::now();
        let results = coordinator.batch_get_memories(&ids).await;
        let batch_elapsed = start.elapsed();

        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 100);
        
        // 批量查询应该比循环查询快（100次 * 5ms = 500ms vs 10ms）
        assert!(batch_elapsed.as_millis() < 100, "Batch query should be much faster");
        println!("✅ Batch query completed in {}ms", batch_elapsed.as_millis());
    }

    /// 测试1.2: 批量向量存储队列
    /// 验证批量队列能够批量处理向量存储
    #[tokio::test]
    async fn test_batch_vector_queue() {
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            add_delay_ms: 10, // 10ms delay per vector
        });

        // 创建批量队列
        use super::super::batch_vector_queue::{BatchVectorStorageQueue, BatchVectorQueueConfig};
        let mut config = BatchVectorQueueConfig::default();
        config.batch_size = 10; // 小批量用于测试
        config.batch_interval_ms = 50; // 50ms间隔
        let queue = BatchVectorStorageQueue::new(vector_store, config);

        // 添加多个向量
        let start = Instant::now();
        for i in 0..30 {
            let vector_data = VectorData {
                id: format!("vec_{}", i),
                vector: vec![0.5; 384],
                metadata: std::collections::HashMap::new(),
            };
            queue.add_vector(vector_data).await.unwrap();
        }

        // 等待队列处理完成
        queue.flush().await.unwrap();
        let elapsed = start.elapsed();

        // 验证: 批量处理应该比单个处理快
        // 30个向量，每个10ms = 300ms串行
        // 批量处理（batch_size=10）应该接近30-50ms（3批，每批10ms）
        assert!(elapsed.as_millis() < 200, "Batch processing should be faster than sequential");
        println!("✅ Batch vector queue completed in {}ms", elapsed.as_millis());
        
        // 检查统计
        let stats = queue.stats().await;
        assert_eq!(stats.total_tasks, 30);
        assert_eq!(stats.processed_tasks, 30);
        assert!(stats.total_batches >= 3); // 至少3批
        println!("✅ Queue stats: {} tasks, {} batches", stats.total_tasks, stats.total_batches);
    }
}
