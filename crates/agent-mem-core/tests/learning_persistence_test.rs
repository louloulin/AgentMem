//! 学习机制持久化集成测试
//!
//! 验证学习反馈数据可以正确保存到数据库并在重启后恢复

#[cfg(feature = "libsql")]
mod persistence_tests {
    use agent_mem_core::search::adaptive::{QueryFeatures, SearchWeights};
    use agent_mem_core::search::learning::{LearningConfig, LearningEngine};
    use agent_mem_core::storage::libsql::{
        create_libsql_pool, run_migrations, LearningRepositoryTrait, LibSqlLearningRepository,
    };
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_learning_persistence_basic() {
        // 创建临时数据库
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_learning.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();

        // 运行迁移
        run_migrations(conn.clone()).await.unwrap();

        // 创建repository
        let repo = Arc::new(LibSqlLearningRepository::new(conn.clone()));

        // 创建学习引擎（带持久化）
        let config = LearningConfig {
            min_samples: 3,
            ..Default::default()
        };
        let engine = LearningEngine::with_persistence(config, repo.clone());

        // 记录一些反馈
        for i in 0..5 {
            let features = QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            };

            let weights = SearchWeights {
                vector_weight: 0.3,
                fulltext_weight: 0.7,
                confidence: 0.8,
            };

            engine
                .record_feedback(features, weights, 0.85 + (i as f32 * 0.01), None)
                .await;
        }

        // 验证数据已保存
        let all_feedback = repo.get_all_feedback().await.unwrap();
        assert_eq!(all_feedback.len(), 5);

        // 创建新的学习引擎并加载数据（使用相同的配置）
        let config2 = LearningConfig {
            min_samples: 3,
            ..Default::default()
        };
        let engine2 = LearningEngine::with_persistence(config2, repo.clone());
        engine2.load_from_storage().await.unwrap();

        // 验证统计数据已恢复
        let features = QueryFeatures {
            has_exact_terms: true,
            semantic_complexity: 0.3,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 20,
            is_question: false,
        };

        let recommended = engine2.get_recommended_weights(&features).await;
        // 因为样本数 >= min_samples (5 >= 3)，应该能获得推荐权重
        assert!(recommended.is_some());
    }

    #[tokio::test]
    async fn test_learning_persistence_across_restarts() {
        // 创建临时数据库
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_restart.db");

        // 第一次启动：创建和记录反馈
        {
            let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();
            run_migrations(conn.clone()).await.unwrap();

            let repo = Arc::new(LibSqlLearningRepository::new(conn));
            let engine = LearningEngine::with_persistence(LearningConfig::default(), repo);

            // 记录反馈
            for _ in 0..10 {
                let features = QueryFeatures {
                    has_exact_terms: false,
                    semantic_complexity: 0.8,
                    has_temporal_indicator: false,
                    entity_count: 2,
                    query_length: 50,
                    is_question: true,
                };

                let weights = SearchWeights {
                    vector_weight: 0.8,
                    fulltext_weight: 0.2,
                    confidence: 0.9,
                };

                engine.record_feedback(features, weights, 0.9, None).await;
            }

            let stats = engine.get_all_statistics().await;
            assert!(!stats.is_empty());
        } // 第一个engine被drop

        // 第二次启动：重新加载数据
        {
            let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();

            let repo = Arc::new(LibSqlLearningRepository::new(conn));
            let engine = LearningEngine::with_persistence(LearningConfig::default(), repo.clone());

            // 加载历史数据
            engine.load_from_storage().await.unwrap();

            // 验证数据已恢复
            let all_feedback = repo.get_all_feedback().await.unwrap();
            assert_eq!(all_feedback.len(), 10);

            let stats = engine.get_all_statistics().await;
            assert!(!stats.is_empty());

            // 验证可以获取推荐权重
            let features = QueryFeatures {
                has_exact_terms: false,
                semantic_complexity: 0.8,
                has_temporal_indicator: false,
                entity_count: 2,
                query_length: 50,
                is_question: true,
            };
            let recommended = engine.get_recommended_weights(&features).await;
            assert!(recommended.is_some());
        }
    }

    #[tokio::test]
    async fn test_learning_repository_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_repo.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();

        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlLearningRepository::new(conn);

        // 测试按模式获取反馈
        for i in 0..3 {
            let features = QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            };

            let record = agent_mem_core::search::learning::FeedbackRecord {
                id: format!("exact-{}", i),
                query_pattern: "exact_match".to_string(),
                features,
                weights: SearchWeights {
                    vector_weight: 0.3,
                    fulltext_weight: 0.7,
                    confidence: 0.8,
                },
                effectiveness: 0.9,
                timestamp: chrono::Utc::now(),
                user_id: None,
            };

            repo.create_feedback(&record).await.unwrap();
        }

        for i in 0..2 {
            let features = QueryFeatures {
                has_exact_terms: false,
                semantic_complexity: 0.8,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 50,
                is_question: true,
            };

            let record = agent_mem_core::search::learning::FeedbackRecord {
                id: format!("semantic-{}", i),
                query_pattern: "semantic_question".to_string(),
                features,
                weights: SearchWeights {
                    vector_weight: 0.8,
                    fulltext_weight: 0.2,
                    confidence: 0.9,
                },
                effectiveness: 0.85,
                timestamp: chrono::Utc::now(),
                user_id: None,
            };

            repo.create_feedback(&record).await.unwrap();
        }

        // 测试按模式获取
        let exact_feedback = repo.get_feedback_by_pattern("exact_match").await.unwrap();
        assert_eq!(exact_feedback.len(), 3);

        let semantic_feedback = repo
            .get_feedback_by_pattern("semantic_question")
            .await
            .unwrap();
        assert_eq!(semantic_feedback.len(), 2);

        // 测试计数
        let exact_count = repo
            .get_feedback_count_by_pattern("exact_match")
            .await
            .unwrap();
        assert_eq!(exact_count, 3);

        // 测试获取最近N条
        let recent = repo.get_recent_feedback(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn test_old_feedback_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cleanup.db");
        let conn = create_libsql_pool(db_path.to_str().unwrap()).await.unwrap();

        run_migrations(conn.clone()).await.unwrap();

        let repo = LibSqlLearningRepository::new(conn);

        // 创建旧记录
        let old_record = agent_mem_core::search::learning::FeedbackRecord {
            id: "old-record".to_string(),
            query_pattern: "exact_match".to_string(),
            features: QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            },
            weights: SearchWeights {
                vector_weight: 0.5,
                fulltext_weight: 0.5,
                confidence: 0.8,
            },
            effectiveness: 0.8,
            timestamp: chrono::Utc::now() - chrono::Duration::days(30),
            user_id: None,
        };

        // 创建新记录
        let new_record = agent_mem_core::search::learning::FeedbackRecord {
            id: "new-record".to_string(),
            query_pattern: "exact_match".to_string(),
            features: QueryFeatures {
                has_exact_terms: true,
                semantic_complexity: 0.3,
                has_temporal_indicator: false,
                entity_count: 0,
                query_length: 20,
                is_question: false,
            },
            weights: SearchWeights {
                vector_weight: 0.5,
                fulltext_weight: 0.5,
                confidence: 0.8,
            },
            effectiveness: 0.9,
            timestamp: chrono::Utc::now(),
            user_id: None,
        };

        repo.create_feedback(&old_record).await.unwrap();
        repo.create_feedback(&new_record).await.unwrap();

        // 删除超过7天的记录
        let deleted = repo
            .delete_old_feedback(chrono::Utc::now() - chrono::Duration::days(7))
            .await
            .unwrap();
        assert_eq!(deleted, 1);

        // 验证只剩新记录
        let remaining = repo.get_all_feedback().await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "new-record");
    }
}
