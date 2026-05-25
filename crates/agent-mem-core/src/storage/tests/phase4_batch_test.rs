//! 🆕 Phase 4 批量操作优化测试
//!
//! 测试批量处理队列和批量向量搜索功能

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::super::batch_vector_queue::{BatchVectorQueueConfig, BatchVectorStorageQueue};
    use agent_mem_traits::{VectorData, VectorStore};
    use std::sync::Arc;
    use tokio::time::Instant;

    // Mock VectorStore for testing
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

        async fn delete_vectors(&self, _ids: Vec<String>) -> agent_mem_traits::Result<()> {
            Ok(())
        }

        async fn update_vectors(&self, _vectors: Vec<VectorData>) -> agent_mem_traits::Result<()> {
            Ok(())
        }

        async fn clear(&self) -> agent_mem_traits::Result<()> {
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

        async fn get_vector(&self, _id: &str) -> agent_mem_traits::Result<Option<VectorData>> {
            Ok(None)
        }

        async fn count_vectors(&self) -> agent_mem_traits::Result<usize> {
            Ok(0)
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
                match self.delete_vectors(batch).await {
                    Ok(_) => results.push(true),
                    Err(_) => results.push(false),
                }
            }
            Ok(results)
        }
    }

    /// 测试4.1: 自动批量处理队列
    /// 验证批量队列能够自动批量处理向量存储
    #[tokio::test]
    async fn test_auto_batch_processing_queue() -> anyhow::Result<()> {
        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            add_delay_ms: 10, // 10ms delay per vector
        });

        // 创建批量队列
        let mut config = BatchVectorQueueConfig::default();
        config.batch_size = 10; // 小批量用于测试
        config.batch_interval_ms = 50; // 50ms间隔
        let queue = BatchVectorStorageQueue::new(vector_store.clone(), config);

        // 添加多个向量（应该自动批量处理）
        let start = Instant::now();
        for i in 0..25 {
            let vector_data = VectorData {
                id: format!("vec_{}", i),
                vector: vec![0.5; 384],
                metadata: std::collections::HashMap::new(),
            };
            queue.add_vector(vector_data).await?;
        }

        // 等待队列处理完成
        queue.flush().await?;
        let elapsed = start.elapsed();

        // 验证: 批量处理应该比单个处理快
        // 25个向量，每个10ms = 250ms串行
        // 批量处理（batch_size=10）应该接近30-50ms（3批，每批10ms）
        assert!(elapsed.as_millis() < 200, "Batch processing should be faster than sequential");
        println!("✅ Auto batch processing completed in {}ms", elapsed.as_millis());
        
        // 检查统计
        let stats = queue.stats().await;
        assert_eq!(stats.total_tasks, 25);
        assert_eq!(stats.processed_tasks, 25);
        assert!(stats.total_batches >= 3); // 至少3批
        println!("✅ Queue stats: {} tasks, {} batches", stats.total_tasks, stats.total_batches);
    }

    /// 测试4.2: 批量向量搜索性能
    /// 验证批量搜索比单个搜索快
    #[tokio::test]
    async fn test_batch_vector_search_performance() {
        use super::super::super::search::vector_search::{VectorSearchConfig, VectorSearchEngine};
        use super::super::super::search::SearchQuery;

        let vector_store = Arc::new(MockVectorStore {
            vectors: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            add_delay_ms: 0, // 无延迟用于搜索测试
        });

        let mut config = VectorSearchConfig::default();
        config.enable_batch_optimization = true;

        let engine = VectorSearchEngine::with_config(vector_store, 384, config);

        // 创建多个查询向量
        let query_vectors: Vec<Vec<f32>> = (0..10)
            .map(|_| vec![0.5; 384])
            .collect();

        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            threshold: Some(0.5),
            vector_weight: 1.0,
            fulltext_weight: 0.0,
            filters: None,
            metadata_filters: None,
        };

        // 测试批量搜索
        let start = Instant::now();
        let results = engine.batch_search(query_vectors.clone(), &query).await;
        let batch_elapsed = start.elapsed();

        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 10);

        // 批量搜索应该比单个搜索快（并发执行）
        assert!(batch_elapsed.as_millis() < 100, "Batch search should be fast");
        println!("✅ Batch vector search completed in {}ms", batch_elapsed.as_millis());
    }
}
