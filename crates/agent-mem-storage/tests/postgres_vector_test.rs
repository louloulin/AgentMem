//! PostgreSQL + pgvector 向量存储测试
//!
//! 这些测试需要运行 PostgreSQL 数据库并安装 pgvector 扩展
//! 
//! 设置测试环境:
//! ```bash
//! docker run -d --name postgres-test \
//!   -e POSTGRES_PASSWORD=test \
//!   -e POSTGRES_DB=agentmem_test \
//!   -p 5432:5432 \
//!   pgvector/pgvector:pg16
//! ```
//!
//! 运行测试:
//! ```bash
//! export DATABASE_URL="postgresql://postgres:test@localhost:5432/agentmem_test"
//! cargo test --package agent-mem-storage --test postgres_vector_test --features postgres -- --nocapture
//! ```

#[cfg(feature = "postgres")]
mod tests {
    use agent_mem_storage::backends::{
        PostgresVectorConfig, PostgresVectorStore, VectorDistanceOperator,
    };
    use agent_mem_traits::{VectorData, VectorStore};
    use sqlx::postgres::PgPoolOptions;
    use std::collections::HashMap;
    use std::env;
    use std::sync::Arc;

    /// 创建测试数据库连接池
    async fn create_test_pool() -> Arc<sqlx::PgPool> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:test@localhost:5432/agentmem_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Arc::new(pool)
    }

    /// 清理测试表
    async fn cleanup_test_table(pool: &sqlx::PgPool, table_name: &str) {
        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", table_name))
            .execute(pool)
            .await;
    }

    #[tokio::test]
    async fn test_postgres_vector_basic_operations() {
        let pool = create_test_pool().await;
        cleanup_test_table(&pool, "test_vectors").await;

        let config = PostgresVectorConfig {
            table_name: "test_vectors".to_string(),
            vector_column: "embedding".to_string(),
            dimension: 128,
            distance_operator: VectorDistanceOperator::Cosine,
            auto_create_table: true,
            index_type: Some("ivfflat".to_string()),
        };

        let store = PostgresVectorStore::new(pool.clone(), config);

        // 测试插入向量
        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![0.1; 128],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("type".to_string(), "test".to_string());
                    map
                },
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.2; 128],
                metadata: HashMap::new(),
            },
            VectorData {
                id: "vec3".to_string(),
                vector: vec![0.3; 128],
                metadata: HashMap::new(),
            },
        ];

        let ids = store.add_vectors(vectors).await.unwrap();
        assert_eq!(ids.len(), 3);
        println!("✅ 插入 3 个向量");

        // 测试向量计数
        let count = store.count_vectors().await.unwrap();
        assert_eq!(count, 3);
        println!("✅ 向量计数: {}", count);

        // 测试获取单个向量
        let vector = store.get_vector("vec1").await.unwrap();
        assert!(vector.is_some());
        let vector = vector.unwrap();
        assert_eq!(vector.id, "vec1");
        assert_eq!(vector.vector.len(), 128);
        println!("✅ 获取单个向量");

        // 测试向量搜索
        let query = vec![0.15; 128];
        let results = store.search_vectors(query, 2, None).await.unwrap();
        assert_eq!(results.len(), 2);
        println!("✅ 向量搜索返回 {} 个结果", results.len());

        // 验证搜索结果按相似度排序
        assert!(results[0].similarity >= results[1].similarity);
        println!(
            "   结果 1: id={}, similarity={:.4}",
            results[0].id, results[0].similarity
        );
        println!(
            "   结果 2: id={}, similarity={:.4}",
            results[1].id, results[1].similarity
        );

        // 测试带阈值的搜索
        let results_with_threshold = store
            .search_vectors(vec![0.1; 128], 10, Some(0.9))
            .await
            .unwrap();
        println!(
            "✅ 带阈值搜索返回 {} 个结果",
            results_with_threshold.len()
        );

        // 测试删除向量
        store.delete_vectors(vec!["vec2".to_string()]).await.unwrap();
        let count_after_delete = store.count_vectors().await.unwrap();
        assert_eq!(count_after_delete, 2);
        println!("✅ 删除向量后计数: {}", count_after_delete);

        // 测试更新向量
        let updated_vector = VectorData {
            id: "vec1".to_string(),
            vector: vec![0.5; 128],
            metadata: {
                let mut map = HashMap::new();
                map.insert("type".to_string(), "updated".to_string());
                map
            },
        };
        store.update_vectors(vec![updated_vector]).await.unwrap();
        let updated = store.get_vector("vec1").await.unwrap().unwrap();
        assert_eq!(updated.vector[0], 0.5);
        println!("✅ 更新向量");

        // 测试清空
        store.clear().await.unwrap();
        let count_after_clear = store.count_vectors().await.unwrap();
        assert_eq!(count_after_clear, 0);
        println!("✅ 清空所有向量");

        // 清理
        cleanup_test_table(&pool, "test_vectors").await;
    }

    #[tokio::test]
    async fn test_postgres_vector_distance_operators() {
        let pool = create_test_pool().await;

        // 测试余弦距离
        {
            cleanup_test_table(&pool, "test_cosine").await;
            let config = PostgresVectorConfig {
                table_name: "test_cosine".to_string(),
                distance_operator: VectorDistanceOperator::Cosine,
                dimension: 64,
                ..Default::default()
            };
            let store = PostgresVectorStore::new(pool.clone(), config);

            let vectors = vec![VectorData {
                id: "v1".to_string(),
                vector: vec![1.0; 64],
                metadata: HashMap::new(),
            }];
            store.add_vectors(vectors).await.unwrap();

            let results = store.search_vectors(vec![1.0; 64], 1, None).await.unwrap();
            assert_eq!(results.len(), 1);
            assert!(results[0].similarity > 0.99); // 完全相同的向量
            println!("✅ 余弦距离测试: similarity={:.4}", results[0].similarity);

            cleanup_test_table(&pool, "test_cosine").await;
        }

        // 测试 L2 距离
        {
            cleanup_test_table(&pool, "test_l2").await;
            let config = PostgresVectorConfig {
                table_name: "test_l2".to_string(),
                distance_operator: VectorDistanceOperator::L2,
                dimension: 64,
                ..Default::default()
            };
            let store = PostgresVectorStore::new(pool.clone(), config);

            let vectors = vec![VectorData {
                id: "v1".to_string(),
                vector: vec![1.0; 64],
                metadata: HashMap::new(),
            }];
            store.add_vectors(vectors).await.unwrap();

            let results = store.search_vectors(vec![1.0; 64], 1, None).await.unwrap();
            assert_eq!(results.len(), 1);
            assert!(results[0].distance < 0.01); // 距离接近 0
            println!("✅ L2 距离测试: distance={:.4}", results[0].distance);

            cleanup_test_table(&pool, "test_l2").await;
        }

        // 测试内积
        {
            cleanup_test_table(&pool, "test_ip").await;
            let config = PostgresVectorConfig {
                table_name: "test_ip".to_string(),
                distance_operator: VectorDistanceOperator::InnerProduct,
                dimension: 64,
                ..Default::default()
            };
            let store = PostgresVectorStore::new(pool.clone(), config);

            let vectors = vec![VectorData {
                id: "v1".to_string(),
                vector: vec![1.0; 64],
                metadata: HashMap::new(),
            }];
            store.add_vectors(vectors).await.unwrap();

            let results = store.search_vectors(vec![1.0; 64], 1, None).await.unwrap();
            assert_eq!(results.len(), 1);
            println!("✅ 内积测试: distance={:.4}", results[0].distance);

            cleanup_test_table(&pool, "test_ip").await;
        }
    }

    #[tokio::test]
    async fn test_postgres_vector_batch_operations() {
        let pool = create_test_pool().await;
        cleanup_test_table(&pool, "test_batch").await;

        let config = PostgresVectorConfig {
            table_name: "test_batch".to_string(),
            dimension: 128,
            ..Default::default()
        };
        let store = PostgresVectorStore::new(pool.clone(), config);

        // 测试批量插入
        let batch1 = vec![
            VectorData {
                id: "b1_v1".to_string(),
                vector: vec![0.1; 128],
                metadata: HashMap::new(),
            },
            VectorData {
                id: "b1_v2".to_string(),
                vector: vec![0.2; 128],
                metadata: HashMap::new(),
            },
        ];

        let batch2 = vec![
            VectorData {
                id: "b2_v1".to_string(),
                vector: vec![0.3; 128],
                metadata: HashMap::new(),
            },
            VectorData {
                id: "b2_v2".to_string(),
                vector: vec![0.4; 128],
                metadata: HashMap::new(),
            },
        ];

        let results = store
            .add_vectors_batch(vec![batch1, batch2])
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 2);
        assert_eq!(results[1].len(), 2);
        println!("✅ 批量插入: {} 批，共 {} 个向量", results.len(), 4);

        // 测试批量删除
        let delete_results = store
            .delete_vectors_batch(vec![
                vec!["b1_v1".to_string(), "b1_v2".to_string()],
                vec!["b2_v1".to_string()],
            ])
            .await
            .unwrap();
        assert_eq!(delete_results.len(), 2);
        assert!(delete_results[0]);
        assert!(delete_results[1]);
        println!("✅ 批量删除: {} 批", delete_results.len());

        let count = store.count_vectors().await.unwrap();
        assert_eq!(count, 1); // 只剩 b2_v2
        println!("   删除后剩余: {} 个向量", count);

        cleanup_test_table(&pool, "test_batch").await;
    }

    #[tokio::test]
    async fn test_postgres_vector_health_and_stats() {
        let pool = create_test_pool().await;
        cleanup_test_table(&pool, "test_health").await;

        let config = PostgresVectorConfig {
            table_name: "test_health".to_string(),
            dimension: 256,
            ..Default::default()
        };
        let store = PostgresVectorStore::new(pool.clone(), config);

        // 测试健康检查
        let health = store.health_check().await.unwrap();
        println!("✅ 健康检查: {:?}", health);

        // 添加一些向量
        let vectors = vec![
            VectorData {
                id: "v1".to_string(),
                vector: vec![0.1; 256],
                metadata: HashMap::new(),
            },
            VectorData {
                id: "v2".to_string(),
                vector: vec![0.2; 256],
                metadata: HashMap::new(),
            },
        ];
        store.add_vectors(vectors).await.unwrap();

        // 测试统计信息
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 2);
        assert_eq!(stats.dimension, 256);
        println!("✅ 统计信息:");
        println!("   总向量数: {}", stats.total_vectors);
        println!("   向量维度: {}", stats.dimension);
        println!("   索引类型: {:?}", stats.index_type);

        cleanup_test_table(&pool, "test_health").await;
    }
}

