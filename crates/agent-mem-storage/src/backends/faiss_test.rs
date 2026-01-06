//! FAISS 后端测试

#[cfg(test)]
mod tests {
    use super::super::faiss::{FaissConfig, FaissStore};
    use agent_mem_traits::{VectorData, VectorStore};
    use std::collections::HashMap;
    use tempfile::tempdir;

    async fn create_test_store() -> FaissStore {
        let temp_dir = tempdir().unwrap();
        let config = FaissConfig {
            dimension: 4,
            data_path: temp_dir.path().join("test_index"),
            metadata_path: temp_dir.path().join("test_metadata.json"),
            ..Default::default()
        };
        FaissStore::new(config).await?
    }

    fn create_test_vector(id: &str, vector: Vec<f32>) -> VectorData {
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());

        VectorData {
            id: id.to_string(),
            vector,
            metadata,
        }
    }

    #[tokio::test]
    async fn test_faiss_store_creation() {
        let store = create_test_store().await;
        let count = store.count_vectors().await?;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_add_and_get_vector() {
        let store = create_test_store().await;

        let vector_data = create_test_vector("test1", vec![1.0, 2.0, 3.0, 4.0]);
        let ids = store.add_vectors(vec![vector_data.clone()]).await?;

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "test1");

        let retrieved = store.get_vector("test1").await?;
        assert!(retrieved.is_some());

        let retrieved_data = retrieved.unwrap();
        assert_eq!(retrieved_data.id, "test1");
        assert_eq!(retrieved_data.vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            retrieved_data.metadata.get("test_key").unwrap(),
            "test_value"
        );
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
        let vector_data = create_test_vector("test1", vec![1.0, 2.0, 3.0, 4.0]);
        store.add_vectors(vec![vector_data]).await?;

        // 更新向量
        let updated_vector = create_test_vector("test1", vec![5.0, 6.0, 7.0, 8.0]);
        store.update_vectors(vec![updated_vector]).await?;

        // 验证更新
        let retrieved = store.get_vector("test1").await?.unwrap();
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
        assert!(ids[0].starts_with("faiss_")); // 应该以faiss_开头
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
}
