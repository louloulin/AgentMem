//! Redis 缓存后端测试

#[cfg(test)]
mod tests {
    use super::super::redis::{RedisConfig, RedisStore};
    use agent_mem_traits::{VectorData, VectorStore};
    use std::collections::HashMap;

    async fn create_test_store() -> RedisStore {
        let config = RedisConfig {
            connection_url: "redis://localhost:6379".to_string(),
            key_prefix: "test_agentmem".to_string(),
            vector_dimension: 4,
            ttl: 3600, // 1小时过期
            enable_distributed_lock: true,
            ..Default::default()
        };
        RedisStore::new(config).await?
    }

    fn create_test_vector(id: &str, vector: Vec<f32>) -> VectorData {
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        metadata.insert("category".to_string(), "cache_test".to_string());
        metadata.insert("content".to_string(), format!("Cached content for {id}"));
        metadata.insert("priority".to_string(), "high".to_string());

        VectorData {
            id: id.to_string(),
            vector,
            metadata,
        }
    }

    #[tokio::test]
    async fn test_redis_store_creation() {
        let store = create_test_store().await;
        let count = store.count_vectors().await?;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_add_and_get_vector() {
        let store = create_test_store().await;

        let vector_data = create_test_vector("cache1", vec![1.0, 2.0, 3.0, 4.0]);
        let ids = store.add_vectors(vec![vector_data.clone()]).await?;

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "cache1");

        let retrieved = store.get_vector("cache1").await?;
        assert!(retrieved.is_some());

        let retrieved_data = retrieved.unwrap();
        assert_eq!(retrieved_data.id, "cache1");
        assert_eq!(retrieved_data.vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            retrieved_data.metadata.get("test_key").unwrap(),
            "test_value"
        );
        assert_eq!(
            retrieved_data.metadata.get("category").unwrap(),
            "cache_test"
        );
        assert_eq!(retrieved_data.metadata.get("priority").unwrap(), "high");
    }

    #[tokio::test]
    async fn test_search_vectors() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]),
            create_test_vector("vec3", vec![0.0, 0.0, 1.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;

        // 搜索与第一个向量相似的向量
        let query_vector = vec![1.0, 0.0, 0.0, 0.0];
        let results = store.search_vectors(query_vector, 2, None).await?;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "vec1"); // 最相似的应该是自己
        assert!(results[0].similarity > 0.99); // 余弦相似度应该接近1
    }

    #[tokio::test]
    async fn test_search_with_threshold() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]), // 与查询向量垂直，相似度为0
        ];

        store.add_vectors(vectors).await?;

        // 使用高阈值搜索
        let query_vector = vec![1.0, 0.0, 0.0, 0.0];
        let results = store
            .search_vectors(query_vector, 10, Some(0.5))
            .await
            .unwrap();

        // 只有vec1应该满足阈值要求
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "vec1");
    }

    #[tokio::test]
    async fn test_update_vectors() {
        let store = create_test_store().await;

        // 添加初始向量
        let vector_data = create_test_vector("cache1", vec![1.0, 2.0, 3.0, 4.0]);
        store.add_vectors(vec![vector_data]).await?;

        // 更新向量
        let updated_vector = create_test_vector("cache1", vec![5.0, 6.0, 7.0, 8.0]);
        store.update_vectors(vec![updated_vector]).await?;

        // 验证更新
        let retrieved = store.get_vector("cache1").await?.unwrap();
        assert_eq!(retrieved.vector, vec![5.0, 6.0, 7.0, 8.0]);
    }

    #[tokio::test]
    async fn test_delete_vectors() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;
        assert_eq!(store.count_vectors().await?, 2);

        // 删除一个向量
        store
            .delete_vectors(vec!["vec1".to_string()])
            .await
            .unwrap();
        assert_eq!(store.count_vectors().await?, 1);

        // 验证删除
        let retrieved = store.get_vector("vec1").await?;
        assert!(retrieved.is_none());

        let retrieved = store.get_vector("vec2").await?;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_clear_store() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;
        assert_eq!(store.count_vectors().await?, 2);

        // 清空存储
        store.clear().await?;
        assert_eq!(store.count_vectors().await?, 0);
    }

    #[tokio::test]
    async fn test_dimension_validation() {
        let store = create_test_store().await;

        // 尝试添加错误维度的向量
        let wrong_dimension_vector = create_test_vector("test1", vec![1.0, 2.0]); // 只有2维，期望4维

        let result = store.add_vectors(vec![wrong_dimension_vector]).await;
        assert!(result.is_err());

        // 验证错误消息包含维度信息
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("dimension"));
    }

    #[tokio::test]
    async fn test_empty_id_generation() {
        let store = create_test_store().await;

        // 创建一个空ID的向量
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());

        let vector_data = VectorData {
            id: "".to_string(), // 空ID
            vector: vec![1.0, 2.0, 3.0, 4.0],
            metadata,
        };

        let ids = store.add_vectors(vec![vector_data]).await?;

        assert_eq!(ids.len(), 1);
        assert!(!ids[0].is_empty()); // 应该生成一个非空ID
        assert!(ids[0].starts_with("redis_")); // 应该以redis_开头
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let store = create_test_store().await;

        // 批量添加向量
        let vectors = vec![
            create_test_vector("batch1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("batch2", vec![0.0, 1.0, 0.0, 0.0]),
            create_test_vector("batch3", vec![0.0, 0.0, 1.0, 0.0]),
        ];

        let ids = store.add_vectors(vectors).await?;
        assert_eq!(ids.len(), 3);
        assert_eq!(store.count_vectors().await?, 3);

        // 批量删除向量
        store
            .delete_vectors(vec!["batch1".to_string(), "batch3".to_string()])
            .await
            .unwrap();
        assert_eq!(store.count_vectors().await?, 1);

        // 验证剩余向量
        let remaining = store.get_vector("batch2").await?;
        assert!(remaining.is_some());
    }

    #[tokio::test]
    async fn test_similarity_calculation() {
        let store = create_test_store().await;

        // 添加已知向量
        let vectors = vec![
            create_test_vector("identical", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("opposite", vec![-1.0, 0.0, 0.0, 0.0]),
            create_test_vector("orthogonal", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;

        // 搜索
        let query_vector = vec![1.0, 0.0, 0.0, 0.0];
        let results = store.search_vectors(query_vector, 3, None).await?;

        assert_eq!(results.len(), 3);

        // 验证相似度排序
        assert_eq!(results[0].id, "identical");
        assert!(results[0].similarity > 0.99); // 应该接近1

        assert_eq!(results[1].id, "orthogonal");
        assert!(results[1].similarity < 0.01 && results[1].similarity > -0.01); // 应该接近0

        assert_eq!(results[2].id, "opposite");
        assert!(results[2].similarity < -0.99); // 应该接近-1
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let store = create_test_store().await;

        // 添加一些向量
        let vectors = vec![
            create_test_vector("stats1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("stats2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;

        // 访问向量以生成统计数据
        let _ = store.get_vector("stats1").await?;
        let _ = store.get_vector("stats1").await?; // 再次访问
        let _ = store.get_vector("nonexistent").await?; // 缓存未命中

        // 获取缓存统计
        let stats = store.get_cache_stats();
        assert_eq!(stats.total_vectors, 2);
        assert!(stats.cache_hits > 0);
        assert!(stats.cache_misses > 0);
        assert!(stats.hit_rate > 0.0 && stats.hit_rate < 1.0);
    }

    #[tokio::test]
    async fn test_distributed_lock() {
        let store = create_test_store().await;

        // 获取分布式锁
        let lock = store.acquire_lock("test_resource", 60).await?;
        assert!(lock.is_some());

        let lock = lock.unwrap();
        assert_eq!(lock.key, "test_resource");
        assert_eq!(lock.timeout, 60);
        assert!(!lock.value.is_empty());

        // 释放锁
        let released = store.release_lock(&lock).await?;
        assert!(released);
    }

    #[tokio::test]
    async fn test_cache_warm_and_cleanup() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("warm1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("warm2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;

        // 缓存预热
        let warmed = store
            .warm_cache(vec![
                "warm1".to_string(),
                "warm2".to_string(),
                "nonexistent".to_string(),
            ])
            .await
            .unwrap();
        assert_eq!(warmed, 2); // 只有2个存在的向量被预热

        // 缓存清理
        let cleaned = store.cleanup_cache().await?;
        assert_eq!(cleaned, 0); // 在测试环境中没有过期项目
    }

    #[tokio::test]
    async fn test_ttl_operations() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("ttl1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("ttl2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await?;

        // 批量设置 TTL
        let result = store
            .set_batch_ttl(vec!["ttl1".to_string(), "ttl2".to_string()], 3600)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_high_performance_operations() {
        let store = create_test_store().await;

        // 测试高性能操作：快速的缓存访问
        let start_time = std::time::Instant::now();

        // 添加多个向量
        let mut vectors = Vec::new();
        for i in 0..50 {
            let vector = vec![
                (i as f32) / 50.0,
                ((i + 1) as f32) / 50.0,
                ((i + 2) as f32) / 50.0,
                ((i + 3) as f32) / 50.0,
            ];
            vectors.push(create_test_vector(&format!("perf_test_{i}"), vector));
        }

        store.add_vectors(vectors).await?;
        let add_duration = start_time.elapsed();

        // 验证添加性能（缓存应该很快）
        assert!(add_duration.as_millis() < 500); // 应该在500ms内完成
        assert_eq!(store.count_vectors().await?, 50);

        // 测试搜索性能
        let query_vector = vec![0.5, 0.5, 0.5, 0.5];
        let start = std::time::Instant::now();
        let results = store.search_vectors(query_vector, 10, None).await?;
        let search_duration = start.elapsed();

        // 验证搜索性能和结果
        assert!(search_duration.as_millis() < 50); // 缓存搜索应该很快
        assert_eq!(results.len(), 10);
        assert!(results[0].similarity > 0.0); // 应该有相似度分数
    }

    #[tokio::test]
    async fn test_session_management() {
        let store = create_test_store().await;

        // 测试会话管理场景
        let session_vectors = vec![
            create_test_vector("session_user_1", vec![0.8, 0.2, 0.1, 0.1]),
            create_test_vector("session_user_2", vec![0.1, 0.8, 0.2, 0.1]),
            create_test_vector("session_user_3", vec![0.1, 0.1, 0.8, 0.2]),
        ];

        store.add_vectors(session_vectors).await?;

        // 测试会话查询
        let user_query = vec![0.7, 0.3, 0.1, 0.1];
        let results = store
            .search_vectors(user_query, 3, Some(0.5))
            .await
            .unwrap();

        // 验证会话相关性
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "session_user_1"); // 最相关的用户

        // 验证访问统计更新
        let stats = store.get_cache_stats();
        assert!(stats.cache_hits > 0);
    }

    #[tokio::test]
    async fn test_real_time_processing() {
        let store = create_test_store().await;

        // 测试实时数据处理场景
        let real_time_data = vec![
            create_test_vector("realtime_1", vec![0.9, 0.1, 0.1, 0.1]),
            create_test_vector("realtime_2", vec![0.1, 0.9, 0.1, 0.1]),
        ];

        let start = std::time::Instant::now();

        // 快速添加
        store.add_vectors(real_time_data).await?;

        // 立即搜索
        let query = vec![0.8, 0.2, 0.1, 0.1];
        let results = store.search_vectors(query, 2, None).await?;

        let total_time = start.elapsed();

        // 验证实时性能
        assert!(total_time.as_millis() < 100); // 整个操作应该在100ms内完成
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "realtime_1"); // 最相关的结果

        // 验证数据一致性
        let count = store.count_vectors().await?;
        assert_eq!(count, 2);
    }
}
