//! Supabase 后端测试
//! 
//! NOTE: These tests require Supabase credentials to run.
//! They are ignored by default. To run them, use:
//! `cargo test --lib -p agent-mem-storage -- --ignored`

#[cfg(test)]
mod tests {
    use super::super::supabase::{SimilarityFunction, SupabaseConfig, SupabaseStore};
    use agent_mem_traits::{VectorData, VectorStore};
    use std::collections::HashMap;

    async fn create_test_store() -> SupabaseStore {
        let config = SupabaseConfig {
            project_url: "https://test-project.supabase.co".to_string(),
            api_key: "test-anon-key".to_string(),
            table_name: "test_vectors".to_string(),
            vector_dimension: 4,
            similarity_function: SimilarityFunction::Cosine,
            ..Default::default()
        };
        SupabaseStore::new(config).await.unwrap()
    }

    fn create_test_vector(id: &str, vector: Vec<f32>) -> VectorData {
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        metadata.insert("category".to_string(), "test".to_string());
        metadata.insert("content".to_string(), format!("Test content for {}", id));
        metadata.insert("source".to_string(), "supabase_test".to_string());

        VectorData {
            id: id.to_string(),
            vector,
            metadata,
        }
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_supabase_store_creation() {
        let store = create_test_store().await;
        let count = store.count_vectors().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_add_and_get_vector() {
        let store = create_test_store().await;

        let vector_data = create_test_vector("test1", vec![1.0, 2.0, 3.0, 4.0]);
        let ids = store.add_vectors(vec![vector_data.clone()]).await.unwrap();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "test1");

        let retrieved = store.get_vector("test1").await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_data = retrieved.unwrap();
        assert_eq!(retrieved_data.id, "test1");
        assert_eq!(retrieved_data.vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            retrieved_data.metadata.get("test_key").unwrap(),
            "test_value"
        );
        assert_eq!(retrieved_data.metadata.get("category").unwrap(), "test");
        assert_eq!(
            retrieved_data.metadata.get("source").unwrap(),
            "supabase_test"
        );
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_search_vectors() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]),
            create_test_vector("vec3", vec![0.0, 0.0, 1.0, 0.0]),
        ];

        store.add_vectors(vectors).await.unwrap();

        // 搜索与第一个向量相似的向量
        let query_vector = vec![1.0, 0.0, 0.0, 0.0];
        let results = store.search_vectors(query_vector, 2, None).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "vec1"); // 最相似的应该是自己
        assert!(results[0].similarity > 0.99); // 余弦相似度应该接近1
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_search_with_threshold() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]), // 与查询向量垂直，相似度为0
        ];

        store.add_vectors(vectors).await.unwrap();

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
    #[ignore] // Requires Supabase credentials
    async fn test_update_vectors() {
        let store = create_test_store().await;

        // 添加初始向量
        let vector_data = create_test_vector("test1", vec![1.0, 2.0, 3.0, 4.0]);
        store.add_vectors(vec![vector_data]).await.unwrap();

        // 更新向量
        let updated_vector = create_test_vector("test1", vec![5.0, 6.0, 7.0, 8.0]);
        store.update_vectors(vec![updated_vector]).await.unwrap();

        // 验证更新
        let retrieved = store.get_vector("test1").await.unwrap().unwrap();
        assert_eq!(retrieved.vector, vec![5.0, 6.0, 7.0, 8.0]);
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_delete_vectors() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await.unwrap();
        assert_eq!(store.count_vectors().await.unwrap(), 2);

        // 删除一个向量
        store
            .delete_vectors(vec!["vec1".to_string()])
            .await
            .unwrap();
        assert_eq!(store.count_vectors().await.unwrap(), 1);

        // 验证删除
        let retrieved = store.get_vector("vec1").await.unwrap();
        assert!(retrieved.is_none());

        let retrieved = store.get_vector("vec2").await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_clear_store() {
        let store = create_test_store().await;

        // 添加测试向量
        let vectors = vec![
            create_test_vector("vec1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("vec2", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await.unwrap();
        assert_eq!(store.count_vectors().await.unwrap(), 2);

        // 清空存储
        store.clear().await.unwrap();
        assert_eq!(store.count_vectors().await.unwrap(), 0);
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
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
    #[ignore] // Requires Supabase credentials
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

        let ids = store.add_vectors(vec![vector_data]).await.unwrap();

        assert_eq!(ids.len(), 1);
        assert!(!ids[0].is_empty()); // 应该生成一个非空ID
        assert!(ids[0].starts_with("supabase_")); // 应该以supabase_开头
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_batch_operations() {
        let store = create_test_store().await;

        // 批量添加向量
        let vectors = vec![
            create_test_vector("batch1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("batch2", vec![0.0, 1.0, 0.0, 0.0]),
            create_test_vector("batch3", vec![0.0, 0.0, 1.0, 0.0]),
        ];

        let ids = store.add_vectors(vectors).await.unwrap();
        assert_eq!(ids.len(), 3);
        assert_eq!(store.count_vectors().await.unwrap(), 3);

        // 批量删除向量
        store
            .delete_vectors(vec!["batch1".to_string(), "batch3".to_string()])
            .await
            .unwrap();
        assert_eq!(store.count_vectors().await.unwrap(), 1);

        // 验证剩余向量
        let remaining = store.get_vector("batch2").await.unwrap();
        assert!(remaining.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_similarity_calculation() {
        let store = create_test_store().await;

        // 添加已知向量
        let vectors = vec![
            create_test_vector("identical", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_vector("opposite", vec![-1.0, 0.0, 0.0, 0.0]),
            create_test_vector("orthogonal", vec![0.0, 1.0, 0.0, 0.0]),
        ];

        store.add_vectors(vectors).await.unwrap();

        // 搜索
        let query_vector = vec![1.0, 0.0, 0.0, 0.0];
        let results = store.search_vectors(query_vector, 3, None).await.unwrap();

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
    #[ignore] // Requires Supabase credentials
    async fn test_postgresql_features() {
        let store = create_test_store().await;

        // 添加包含丰富元数据的向量，测试 PostgreSQL JSONB 功能
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "PostgreSQL Document".to_string());
        metadata.insert("author".to_string(), "Jane Doe".to_string());
        metadata.insert("tags".to_string(), "postgresql,vector,search".to_string());
        metadata.insert("category".to_string(), "database".to_string());
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());

        let vector_data = VectorData {
            id: "pg_doc_1".to_string(),
            vector: vec![0.7, 0.5, 0.3, 0.1],
            metadata,
        };

        let ids = store.add_vectors(vec![vector_data]).await.unwrap();
        assert_eq!(ids[0], "pg_doc_1");

        // 验证 JSONB 元数据保持完整
        let retrieved = store.get_vector("pg_doc_1").await.unwrap().unwrap();
        assert_eq!(
            retrieved.metadata.get("title").unwrap(),
            "PostgreSQL Document"
        );
        assert_eq!(retrieved.metadata.get("author").unwrap(), "Jane Doe");
        assert_eq!(
            retrieved.metadata.get("tags").unwrap(),
            "postgresql,vector,search"
        );
        assert_eq!(retrieved.metadata.get("category").unwrap(), "database");
        assert_eq!(retrieved.metadata.get("priority").unwrap(), "high");
        assert_eq!(retrieved.metadata.get("version").unwrap(), "1.0");
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_realtime_capabilities() {
        let store = create_test_store().await;

        // 测试实时功能的配置（在实际实现中会启用实时订阅）
        // 这里我们测试基本的 CRUD 操作，验证实时更新场景

        // 添加初始数据
        let vector_data = create_test_vector("realtime_test", vec![0.5, 0.5, 0.5, 0.5]);
        store.add_vectors(vec![vector_data]).await.unwrap();

        // 测试实时更新
        let updated_vector = create_test_vector("realtime_test", vec![0.8, 0.6, 0.4, 0.2]);
        store.update_vectors(vec![updated_vector]).await.unwrap();

        // 验证更新
        let retrieved = store.get_vector("realtime_test").await.unwrap().unwrap();
        assert_eq!(retrieved.vector, vec![0.8, 0.6, 0.4, 0.2]);

        // 测试实时删除
        store
            .delete_vectors(vec!["realtime_test".to_string()])
            .await
            .unwrap();

        // 验证删除
        let retrieved = store.get_vector("realtime_test").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_edge_computing_simulation() {
        let store = create_test_store().await;

        // 测试边缘计算场景：快速的本地操作
        let start_time = std::time::Instant::now();

        // 添加多个向量
        let vectors = vec![
            create_test_vector("edge1", vec![0.1, 0.2, 0.3, 0.4]),
            create_test_vector("edge2", vec![0.5, 0.6, 0.7, 0.8]),
            create_test_vector("edge3", vec![0.9, 0.8, 0.7, 0.6]),
        ];

        store.add_vectors(vectors).await.unwrap();

        // 执行搜索
        let query_vector = vec![0.5, 0.5, 0.5, 0.5];
        let results = store.search_vectors(query_vector, 3, None).await.unwrap();

        let elapsed = start_time.elapsed();

        // 验证结果
        assert_eq!(results.len(), 3);
        assert!(elapsed.as_millis() < 100); // 应该很快完成

        // 验证边缘计算的低延迟特性
        for result in results {
            assert!(result.similarity >= 0.0);
            assert!(!result.id.is_empty());
        }
    }

    #[tokio::test]
    #[ignore] // Requires Supabase credentials
    async fn test_open_source_compatibility() {
        let store = create_test_store().await;

        // 测试开源友好的特性：标准的 PostgreSQL 兼容性
        let vector_data = create_test_vector("opensource_test", vec![1.0, 1.0, 1.0, 1.0]);

        // 添加向量
        let ids = store.add_vectors(vec![vector_data]).await.unwrap();
        assert_eq!(ids.len(), 1);

        // 验证标准操作
        let count = store.count_vectors().await.unwrap();
        assert_eq!(count, 1);

        // 验证搜索功能
        let query_vector = vec![1.0, 1.0, 1.0, 1.0];
        let results = store.search_vectors(query_vector, 1, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "opensource_test");

        // 验证清理功能
        store.clear().await.unwrap();
        let final_count = store.count_vectors().await.unwrap();
        assert_eq!(final_count, 0);
    }
}
