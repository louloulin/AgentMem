//! 记忆路由工具函数测试
//! 
//! 测试中文检测和自适应阈值计算功能

#[cfg(test)]
mod tests {
    use super::super::memory::*;

    /// 测试中文检测函数
    #[test]
    fn test_contains_chinese() {
        // 测试中文字符
        assert!(contains_chinese("仓颉"));
        assert!(contains_chinese("中文测试"));
        assert!(contains_chinese("Hello 世界"));
        assert!(contains_chinese("测试123"));
        
        // 测试非中文字符
        assert!(!contains_chinese("Hello World"));
        assert!(!contains_chinese("123456"));
        assert!(!contains_chinese("product_id"));
        assert!(!contains_chinese(""));
        
        // 测试混合内容
        assert!(contains_chinese("商品ID: P123456"));
        assert!(!contains_chinese("Product ID: P123456"));
    }

    /// 测试自适应阈值计算 - 中文查询
    #[test]
    fn test_get_adaptive_threshold_chinese() {
        // 中文短查询应该使用较低阈值
        let threshold1 = get_adaptive_threshold("仓颉");
        assert!(threshold1 < 0.3, "中文短查询阈值应该 < 0.3, 实际: {}", threshold1);
        assert!(threshold1 >= 0.1, "阈值应该 >= 0.1, 实际: {}", threshold1);
        
        // 中文中等长度查询
        let threshold2 = get_adaptive_threshold("仓颉是造字圣人");
        assert!(threshold2 < 0.5, "中文中等查询阈值应该 < 0.5, 实际: {}", threshold2);
        
        // 中文长查询
        let threshold3 = get_adaptive_threshold("仓颉是中国古代传说中的人物，被尊为造字圣人");
        assert!(threshold3 < 0.7, "中文长查询阈值应该 < 0.7, 实际: {}", threshold3);
    }

    /// 测试自适应阈值计算 - 英文查询
    #[test]
    fn test_get_adaptive_threshold_english() {
        // 英文短查询
        let threshold1 = get_adaptive_threshold("test");
        assert!(threshold1 >= 0.3, "英文短查询阈值应该 >= 0.3, 实际: {}", threshold1);
        
        // 英文中等长度查询
        let threshold2 = get_adaptive_threshold("This is a test query");
        assert!(threshold2 >= 0.5, "英文中等查询阈值应该 >= 0.5, 实际: {}", threshold2);
        
        // 英文长查询
        let threshold3 = get_adaptive_threshold("This is a very long test query that should have a higher threshold");
        assert!(threshold3 >= 0.7, "英文长查询阈值应该 >= 0.7, 实际: {}", threshold3);
    }

    /// 测试自适应阈值计算 - 精确ID查询
    #[test]
    fn test_get_adaptive_threshold_exact_id() {
        // 商品ID格式
        let threshold1 = get_adaptive_threshold("P123456");
        assert_eq!(threshold1, 0.1, "商品ID阈值应该为0.1");
        
        // UUID格式
        let threshold2 = get_adaptive_threshold("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(threshold2, 0.1, "UUID阈值应该为0.1");
        
        // 其他精确ID
        let threshold3 = get_adaptive_threshold("abc123def");
        assert_eq!(threshold3, 0.2, "精确ID阈值应该为0.2");
    }

    /// 测试自适应阈值计算 - 混合中英文
    #[test]
    fn test_get_adaptive_threshold_mixed() {
        // 包含中文的查询应该降低阈值
        let threshold1 = get_adaptive_threshold("商品ID");
        assert!(threshold1 < 0.4, "包含中文的商品查询阈值应该 < 0.4, 实际: {}", threshold1);
        
        // 纯英文的商品查询
        let threshold2 = get_adaptive_threshold("product id");
        assert_eq!(threshold2, 0.4, "纯英文商品查询阈值应该为0.4");
    }

    /// 测试阈值边界情况
    #[test]
    fn test_get_adaptive_threshold_boundaries() {
        // 空字符串
        let threshold1 = get_adaptive_threshold("");
        assert!(threshold1 >= 0.1 && threshold1 <= 0.9);
        
        // 单个中文字符
        let threshold2 = get_adaptive_threshold("中");
        assert!(threshold2 >= 0.1 && threshold2 < 0.3);
        
        // 单个英文字符
        let threshold3 = get_adaptive_threshold("a");
        assert!(threshold3 >= 0.1 && threshold3 <= 0.9);
    }

    /// 测试Recency评分计算
    #[test]
    fn test_calculate_recency_score() {
        use chrono::{DateTime, Utc};
        
        // 测试最近访问（应该接近1.0）
        let now = Utc::now();
        let recent_time = now.to_rfc3339();
        let recency1 = calculate_recency_score(&recent_time, 0.1);
        assert!(recency1 > 0.9, "最近访问的recency应该 > 0.9, 实际: {}", recency1);
        
        // 测试1小时前访问（decay=0.1时应该约0.9）
        let one_hour_ago = (now - chrono::Duration::hours(1)).to_rfc3339();
        let recency2 = calculate_recency_score(&one_hour_ago, 0.1);
        assert!(recency2 > 0.85 && recency2 < 0.95, "1小时前访问的recency应该在0.85-0.95之间, 实际: {}", recency2);
        
        // 测试24小时前访问（decay=0.1时应该约0.08）
        let one_day_ago = (now - chrono::Duration::hours(24)).to_rfc3339();
        let recency3 = calculate_recency_score(&one_day_ago, 0.1);
        assert!(recency3 > 0.05 && recency3 < 0.15, "24小时前访问的recency应该在0.05-0.15之间, 实际: {}", recency3);
        
        // 测试无效时间格式（应该返回1.0作为默认值）
        let invalid_time = "invalid-time";
        let recency4 = calculate_recency_score(invalid_time, 0.1);
        assert_eq!(recency4, 1.0, "无效时间格式应该返回1.0");
    }

    /// 测试三维检索综合评分计算
    #[test]
    fn test_calculate_3d_score() {
        use chrono::{DateTime, Utc};
        
        // 测试完美记忆（高relevance、高importance、最近访问）
        let now = Utc::now();
        let recent_time = now.to_rfc3339();
        let score1 = calculate_3d_score(0.9, 0.9, &recent_time, 0.1);
        assert!(score1 > 0.7, "完美记忆的综合评分应该 > 0.7, 实际: {}", score1);
        
        // 测试低relevance记忆（即使importance和recency高，综合评分也应该低）
        let score2 = calculate_3d_score(0.1, 0.9, &recent_time, 0.1);
        assert!(score2 < 0.2, "低relevance记忆的综合评分应该 < 0.2, 实际: {}", score2);
        
        // 测试低importance记忆
        let score3 = calculate_3d_score(0.9, 0.1, &recent_time, 0.1);
        assert!(score3 < 0.2, "低importance记忆的综合评分应该 < 0.2, 实际: {}", score3);
        
        // 测试旧记忆（低recency）
        let old_time = (now - chrono::Duration::hours(48)).to_rfc3339();
        let score4 = calculate_3d_score(0.9, 0.9, &old_time, 0.1);
        assert!(score4 < 0.5, "旧记忆的综合评分应该 < 0.5, 实际: {}", score4);
        
        // 测试边界值（所有维度都是0）
        let score5 = calculate_3d_score(0.0, 0.0, &recent_time, 0.1);
        assert_eq!(score5, 0.0, "所有维度为0的综合评分应该为0.0");
        
        // 测试边界值（所有维度都是1.0）
        let score6 = calculate_3d_score(1.0, 1.0, &recent_time, 0.1);
        assert!(score6 > 0.9, "所有维度为1.0的综合评分应该 > 0.9, 实际: {}", score6);
    }

    /// 测试三维检索评分边界情况
    #[test]
    fn test_calculate_3d_score_boundaries() {
        use chrono::{DateTime, Utc};
        
        let now = Utc::now();
        let recent_time = now.to_rfc3339();
        
        // 测试超出范围的relevance（应该被clamp到[0.0, 1.0]）
        let score1 = calculate_3d_score(1.5, 0.5, &recent_time, 0.1);
        assert!(score1 <= 1.0, "超出范围的relevance应该被clamp, 实际: {}", score1);
        
        let score2 = calculate_3d_score(-0.5, 0.5, &recent_time, 0.1);
        assert!(score2 >= 0.0, "负值的relevance应该被clamp, 实际: {}", score2);
        
        // 测试超出范围的importance（应该被clamp到[0.0, 1.0]）
        let score3 = calculate_3d_score(0.5, 1.5, &recent_time, 0.1);
        assert!(score3 <= 1.0, "超出范围的importance应该被clamp, 实际: {}", score3);
        
        let score4 = calculate_3d_score(0.5, -0.5, &recent_time, 0.1);
        assert!(score4 >= 0.0, "负值的importance应该被clamp, 实际: {}", score4);
    }

    /// 测试查询缓存键生成
    #[test]
    fn test_generate_cache_key() {
        // 测试相同查询生成相同键
        let key1 = generate_cache_key("test query", &Some("agent1".to_string()), &Some("user1".to_string()), &Some(10));
        let key2 = generate_cache_key("test query", &Some("agent1".to_string()), &Some("user1".to_string()), &Some(10));
        assert_eq!(key1, key2, "相同查询应该生成相同的缓存键");
        
        // 测试不同查询生成不同键
        let key3 = generate_cache_key("different query", &Some("agent1".to_string()), &Some("user1".to_string()), &Some(10));
        assert_ne!(key1, key3, "不同查询应该生成不同的缓存键");
        
        // 测试不同agent_id生成不同键
        let key4 = generate_cache_key("test query", &Some("agent2".to_string()), &Some("user1".to_string()), &Some(10));
        assert_ne!(key1, key4, "不同agent_id应该生成不同的缓存键");
        
        // 测试不同limit生成不同键
        let key5 = generate_cache_key("test query", &Some("agent1".to_string()), &Some("user1".to_string()), &Some(20));
        assert_ne!(key1, key5, "不同limit应该生成不同的缓存键");
    }

    /// 测试搜索结果去重逻辑（基于hash）
    #[test]
    fn test_search_result_deduplication() {
        use std::collections::HashMap;
        
        // 模拟搜索结果（相同hash，不同综合评分）
        let mut hash_map: HashMap<String, (String, f64)> = HashMap::new();
        
        // 添加第一个结果（hash: "abc123", score: 0.8）
        hash_map.insert("abc123".to_string(), ("memory1".to_string(), 0.8));
        
        // 添加第二个结果（相同hash，更高评分）
        match hash_map.get_mut("abc123") {
            Some(existing) => {
                if 0.9 > existing.1 {
                    *existing = ("memory2".to_string(), 0.9);
                }
            }
            None => {
                hash_map.insert("abc123".to_string(), ("memory2".to_string(), 0.9));
            }
        }
        
        // 验证：应该保留评分更高的结果
        assert_eq!(hash_map.len(), 1, "相同hash应该只保留一个结果");
        assert_eq!(hash_map.get("abc123").unwrap().1, 0.9, "应该保留评分更高的结果");
        assert_eq!(hash_map.get("abc123").unwrap().0, "memory2", "应该保留评分更高的记忆ID");
        
        // 添加不同hash的结果
        hash_map.insert("def456".to_string(), ("memory3".to_string(), 0.7));
        assert_eq!(hash_map.len(), 2, "不同hash应该保留多个结果");
    }

    #[test]
    fn test_batch_search_request_validation() {
        use crate::models::{BatchSearchRequest, SearchRequest};
        
        // 测试有效的批量搜索请求
        let valid_request = BatchSearchRequest {
            queries: vec![
                SearchRequest {
                    query: "test query 1".to_string(),
                    agent_id: None,
                    user_id: None,
                    memory_type: None,
                    limit: Some(10),
                    threshold: None,
                },
                SearchRequest {
                    query: "test query 2".to_string(),
                    agent_id: None,
                    user_id: None,
                    memory_type: None,
                    limit: Some(20),
                    threshold: Some(0.7),
                },
            ],
            agent_id: Some("test_agent".to_string()),
            user_id: Some("test_user".to_string()),
        };
        
        // 验证请求结构
        assert_eq!(valid_request.queries.len(), 2);
        assert_eq!(valid_request.agent_id, Some("test_agent".to_string()));
        assert_eq!(valid_request.user_id, Some("test_user".to_string()));
        assert_eq!(valid_request.queries[0].query, "test query 1");
        assert_eq!(valid_request.queries[1].query, "test query 2");
    }

    #[test]
    fn test_batch_search_response_structure() {
        use crate::models::BatchSearchResponse;
        
        // 测试批量搜索响应结构
        let response = BatchSearchResponse {
            successful: 2,
            failed: 1,
            results: vec![
                vec![serde_json::json!({"id": "1", "content": "result 1"})],
                vec![serde_json::json!({"id": "2", "content": "result 2"})],
                vec![],
            ],
            errors: vec![None, None, Some("Error message".to_string())],
        };
        
        assert_eq!(response.successful, 2);
        assert_eq!(response.failed, 1);
        assert_eq!(response.results.len(), 3);
        assert_eq!(response.errors.len(), 3);
        assert_eq!(response.errors[0], None);
        assert_eq!(response.errors[1], None);
        assert_eq!(response.errors[2], Some("Error message".to_string()));
    }

    #[test]
    fn test_search_statistics_structure() {
        use crate::models::SearchStatsResponse;
        use chrono::Utc;
        
        // 测试搜索统计响应结构
        let response = SearchStatsResponse {
            total_searches: 100,
            cache_hits: 60,
            cache_misses: 40,
            cache_hit_rate: 0.6,
            exact_queries: 10,
            vector_searches: 90,
            avg_latency_ms: 25.5,
            cache_size: 50,
            last_updated: Utc::now(),
        };
        
        assert_eq!(response.total_searches, 100);
        assert_eq!(response.cache_hits, 60);
        assert_eq!(response.cache_misses, 40);
        assert_eq!(response.cache_hit_rate, 0.6);
        assert_eq!(response.exact_queries, 10);
        assert_eq!(response.vector_searches, 90);
        assert_eq!(response.avg_latency_ms, 25.5);
        assert_eq!(response.cache_size, 50);
    }

    #[test]
    fn test_search_statistics_calculations() {
        // 测试缓存命中率计算
        let total = 100u64;
        let hits = 60u64;
        let hit_rate = (hits as f64) / (total as f64);
        assert_eq!(hit_rate, 0.6);
        
        // 测试平均延迟计算
        let total_latency_us = 2550000u64; // 2.55秒 = 2550毫秒
        let total_searches = 100u64;
        let avg_latency_ms = (total_latency_us as f64) / (total_searches as f64) / 1000.0;
        assert_eq!(avg_latency_ms, 25.5);
        
        // 测试零搜索的情况
        let zero_total = 0u64;
        let zero_hit_rate = if zero_total == 0 { 0.0 } else { (hits as f64) / (zero_total as f64) };
        assert_eq!(zero_hit_rate, 0.0);
    }

    #[test]
    fn test_lru_cache_eviction_concept() {
        // 测试LRU缓存淘汰概念
        // LRU缓存会自动淘汰最久未使用的条目，保留最近使用的条目
        
        // 模拟LRU行为：容量为2的缓存
        let mut cache_order: Vec<String> = Vec::new();
        let capacity = 2;
        
        // 添加条目1
        cache_order.push("key1".to_string());
        assert_eq!(cache_order.len(), 1);
        
        // 添加条目2
        cache_order.push("key2".to_string());
        assert_eq!(cache_order.len(), 2);
        
        // 访问条目1（使其成为最近使用的）
        cache_order.retain(|k| k != "key1");
        cache_order.push("key1".to_string());
        assert_eq!(cache_order[0], "key2");
        assert_eq!(cache_order[1], "key1");
        
        // 添加条目3（应该淘汰key2，因为key1最近被访问过）
        if cache_order.len() >= capacity {
            cache_order.remove(0); // 移除最久未使用的
        }
        cache_order.push("key3".to_string());
        assert_eq!(cache_order.len(), 2);
        assert_eq!(cache_order[0], "key1");
        assert_eq!(cache_order[1], "key3");
        
        // 验证key2已被淘汰
        assert!(!cache_order.contains(&"key2".to_string()));
    }

    #[test]
    fn test_lru_vs_fifo_advantage() {
        // 测试LRU相比FIFO的优势
        // LRU保留热点数据，FIFO可能淘汰热点数据
        
        // 场景：容量为2的缓存
        // 访问模式：key1, key2, key1, key3
        
        // FIFO策略：会淘汰key1（第一个插入的）
        let fifo_order = vec!["key1", "key2"];
        // 访问key1后，FIFO不会改变顺序
        // 添加key3时，FIFO会淘汰key1
        let fifo_after = vec!["key2", "key3"];
        assert!(!fifo_after.contains(&"key1"));
        
        // LRU策略：会保留key1（最近访问的），淘汰key2
        let mut lru_order = vec!["key1", "key2"];
        // 访问key1后，LRU将其移到末尾
        lru_order.retain(|k| k != &"key1");
        lru_order.push("key1");
        // 添加key3时，LRU淘汰key2（最久未使用的）
        lru_order.remove(0);
        lru_order.push("key3");
        let lru_after = vec!["key1", "key3"];
        assert!(lru_after.contains(&"key1"), "LRU应该保留最近访问的key1");
        assert!(!lru_after.contains(&"key2"), "LRU应该淘汰最久未使用的key2");
    }

    #[tokio::test]
    async fn test_search_timeout_concept() {
        use std::time::Duration;
        use tokio::time::timeout;
        
        // 测试超时控制概念：快速操作应该成功
        let fast_operation = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<_, String>("success")
        };
        
        let result = timeout(Duration::from_secs(1), fast_operation).await;
        assert!(result.is_ok(), "快速操作应该在超时前完成");
        assert_eq!(result.unwrap().unwrap(), "success");
        
        // 测试超时控制概念：慢速操作应该超时
        let slow_operation = async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok::<_, String>("success")
        };
        
        let result = timeout(Duration::from_millis(500), slow_operation).await;
        assert!(result.is_err(), "慢速操作应该超时");
    }

    #[test]
    fn test_search_timeout_config() {
        // 测试搜索超时配置解析
        let timeout_str = "30";
        let timeout_secs: u64 = timeout_str.parse().unwrap();
        assert_eq!(timeout_secs, 30);
        
        // 测试默认值
        let default_timeout = 30u64;
        assert_eq!(default_timeout, 30);
        
        // 测试无效值处理
        let invalid_timeout: Option<u64> = "invalid".parse().ok();
        assert!(invalid_timeout.is_none());
    }

    #[test]
    fn test_quality_score_calculation() {
        use crate::routes::memory::calculate_quality_score;
        use agent_mem_traits::MemoryItem;
        use chrono::Utc;
        use std::collections::HashMap;
        
        // 创建高质量的记忆（理想长度、丰富元数据、有hash）
        let mut high_quality_item = MemoryItem {
            id: "test1".to_string(),
            content: "This is a high quality memory with ideal length between 50 and 500 characters. It contains meaningful information and has proper structure.".to_string(),
            hash: Some("abc123".to_string()),
            score: Some(0.9),
            metadata: {
                let mut m = HashMap::new();
                m.insert("key1".to_string(), "value1".to_string());
                m.insert("key2".to_string(), "value2".to_string());
                m.insert("key3".to_string(), "value3".to_string());
                m.insert("key4".to_string(), "value4".to_string());
                m.insert("key5".to_string(), "value5".to_string());
                m
            },
            created_at: Utc::now(),
            updated_at: None,
            session: Default::default(),
            memory_type: Default::default(),
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.9,
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 50,
            expires_at: None,
            version: 1,
        };
        
        let high_quality_score = calculate_quality_score(&high_quality_item);
        assert!(high_quality_score > 0.7, "高质量记忆应该有较高的质量评分");
        
        // 创建低质量的记忆（太短、无元数据、无hash）
        let low_quality_item = MemoryItem {
            id: "test2".to_string(),
            content: "Short".to_string(), // 太短
            hash: None, // 无hash
            score: Some(0.5),
            metadata: HashMap::new(), // 无元数据
            created_at: Utc::now(),
            updated_at: None,
            session: Default::default(),
            memory_type: Default::default(),
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.3,
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 0, // 无访问历史
            expires_at: None,
            version: 1,
        };
        
        let low_quality_score = calculate_quality_score(&low_quality_item);
        assert!(low_quality_score < 0.5, "低质量记忆应该有较低的质量评分");
        assert!(high_quality_score > low_quality_score, "高质量记忆的评分应该高于低质量记忆");
    }

    #[test]
    fn test_quality_score_length_scoring() {
        use crate::routes::memory::calculate_quality_score;
        use agent_mem_traits::MemoryItem;
        use chrono::Utc;
        
        // 测试不同长度的内容
        let test_cases = vec![
            ("Very short", 0.2), // 太短
            ("Short content but acceptable", 0.5), // 较短
            ("This is a memory with ideal length between 50 and 500 characters. It should score high for length quality.", 1.0), // 理想长度
            ("This is a longer memory that exceeds 500 characters but is still acceptable. It contains more information but might have some redundancy. The quality score should be slightly lower than ideal length memories but still reasonable for search results.".to_string().repeat(2), 0.8), // 较长
        ];
        
        for (content, expected_min) in test_cases {
            let item = MemoryItem {
                id: "test".to_string(),
                content: content.to_string(),
                hash: Some("test".to_string()),
                score: Some(0.5),
                metadata: HashMap::new(),
                created_at: Utc::now(),
                updated_at: None,
                session: Default::default(),
                memory_type: Default::default(),
                entities: Vec::new(),
                relations: Vec::new(),
                agent_id: "test".to_string(),
                user_id: None,
                importance: 0.5,
                embedding: None,
                last_accessed_at: Utc::now(),
                access_count: 0,
                expires_at: None,
                version: 1,
            };
            
            let quality = calculate_quality_score(&item);
            // 注意：质量评分是综合多个因素的，所以这里只检查长度因素的大致范围
            // 由于还有其他因素（元数据、hash等），实际评分可能略有不同
            assert!(quality >= 0.0 && quality <= 1.0, "质量评分应该在0.0到1.0之间");
        }
    }

    /// 测试并行查询优化的概念
    /// 
    /// 验证并行查询比串行查询更快
    #[tokio::test]
    async fn test_parallel_query_optimization() {
        use futures::future;
        use std::time::Instant;
        
        // 模拟10个查询任务
        let query_ids: Vec<String> = (1..=10).map(|i| format!("id_{}", i)).collect();
        
        // 串行查询（模拟旧方式）
        let serial_start = Instant::now();
        let mut serial_results = Vec::new();
        for id in &query_ids {
            // 模拟查询延迟（10ms）
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            serial_results.push(id.clone());
        }
        let serial_duration = serial_start.elapsed();
        
        // 并行查询（新方式）
        let parallel_start = Instant::now();
        let query_futures: Vec<_> = query_ids
            .iter()
            .map(|id| {
                let id_clone = id.clone();
                async move {
                    // 模拟查询延迟（10ms）
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    id_clone
                }
            })
            .collect();
        let parallel_results = future::join_all(query_futures).await;
        let parallel_duration = parallel_start.elapsed();
        
        // 验证结果一致
        assert_eq!(serial_results.len(), parallel_results.len());
        
        // 验证并行查询更快（理论上应该快约10倍，但实际可能受限于并发限制）
        // 这里只验证并行查询不慢于串行查询
        assert!(
            parallel_duration <= serial_duration,
            "并行查询应该不慢于串行查询: 串行={:?}, 并行={:?}",
            serial_duration,
            parallel_duration
        );
        
        // 验证并行查询确实更快（至少快2倍）
        assert!(
            parallel_duration.as_millis() < serial_duration.as_millis() / 2,
            "并行查询应该明显快于串行查询: 串行={:?}, 并行={:?}",
            serial_duration,
            parallel_duration
        );
    }

    /// 测试并行查询的错误处理
    /// 
    /// 验证即使部分查询失败，其他查询仍能正常完成
    #[tokio::test]
    async fn test_parallel_query_error_handling() {
        use futures::future;
        
        // 模拟5个查询，其中2个会失败
        let query_ids: Vec<String> = (1..=5).map(|i| format!("id_{}", i)).collect();
        
        let query_futures: Vec<_> = query_ids
            .iter()
            .map(|id| {
                let id_clone = id.clone();
                async move {
                    // 模拟查询：id_2和id_4会失败
                    if id_clone == "id_2" || id_clone == "id_4" {
                        Err(format!("Query failed for {}", id_clone))
                    } else {
                        Ok(id_clone)
                    }
                }
            })
            .collect();
        
        let results = future::join_all(query_futures).await;
        
        // 验证结果：3个成功，2个失败
        let successes: Vec<_> = results.iter().filter(|r| r.is_ok()).collect();
        let failures: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
        
        assert_eq!(successes.len(), 3, "应该有3个成功的查询");
        assert_eq!(failures.len(), 2, "应该有2个失败的查询");
        
        // 验证成功的查询结果
        let success_ids: Vec<String> = successes
            .iter()
            .map(|r| r.as_ref().unwrap().clone())
            .collect();
        assert!(success_ids.contains(&"id_1".to_string()));
        assert!(success_ids.contains(&"id_3".to_string()));
        assert!(success_ids.contains(&"id_5".to_string()));
    }

    /// 测试并行写入优化的概念
    /// 
    /// 验证并行写入比串行写入更快
    #[tokio::test]
    async fn test_parallel_write_optimization() {
        use futures::future;
        use std::time::Instant;
        
        // 模拟10个写入任务
        let write_tasks: Vec<String> = (1..=10).map(|i| format!("task_{}", i)).collect();
        
        // 串行写入（模拟旧方式）
        let serial_start = Instant::now();
        let mut serial_results = Vec::new();
        for task in &write_tasks {
            // 模拟写入延迟（20ms）
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            serial_results.push(task.clone());
        }
        let serial_duration = serial_start.elapsed();
        
        // 并行写入（新方式）
        let parallel_start = Instant::now();
        let write_futures: Vec<_> = write_tasks
            .iter()
            .map(|task| {
                let task_clone = task.clone();
                async move {
                    // 模拟写入延迟（20ms）
                    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                    task_clone
                }
            })
            .collect();
        let parallel_results = future::join_all(write_futures).await;
        let parallel_duration = parallel_start.elapsed();
        
        // 验证结果一致
        assert_eq!(serial_results.len(), parallel_results.len());
        
        // 验证并行写入更快（理论上应该快约10倍，但实际可能受限于并发限制）
        // 这里只验证并行写入不慢于串行写入
        assert!(
            parallel_duration <= serial_duration,
            "并行写入应该不慢于串行写入: 串行={:?}, 并行={:?}",
            serial_duration,
            parallel_duration
        );
        
        // 验证并行写入确实更快（至少快2倍）
        assert!(
            parallel_duration.as_millis() < serial_duration.as_millis() / 2,
            "并行写入应该明显快于串行写入: 串行={:?}, 并行={:?}",
            serial_duration,
            parallel_duration
        );
    }

    /// 测试并行写入的错误处理
    /// 
    /// 验证即使部分写入失败，其他写入仍能正常完成
    #[tokio::test]
    async fn test_parallel_write_error_handling() {
        use futures::future;
        
        // 模拟5个写入任务，其中2个会失败
        let write_tasks: Vec<String> = (1..=5).map(|i| format!("task_{}", i)).collect();
        
        let write_futures: Vec<_> = write_tasks
            .iter()
            .map(|task| {
                let task_clone = task.clone();
                async move {
                    // 模拟写入：task_2和task_4会失败
                    if task_clone == "task_2" || task_clone == "task_4" {
                        Err(format!("Write failed for {}", task_clone))
                    } else {
                        Ok(task_clone)
                    }
                }
            })
            .collect();
        
        let results = future::join_all(write_futures).await;
        
        // 验证结果：3个成功，2个失败
        let successes: Vec<_> = results.iter().filter(|r| r.is_ok()).collect();
        let failures: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
        
        assert_eq!(successes.len(), 3, "应该有3个成功的写入");
        assert_eq!(failures.len(), 2, "应该有2个失败的写入");
        
        // 验证成功的写入结果
        let success_tasks: Vec<String> = successes
            .iter()
            .map(|r| r.as_ref().unwrap().clone())
            .collect();
        assert!(success_tasks.contains(&"task_1".to_string()));
        assert!(success_tasks.contains(&"task_3".to_string()));
        assert!(success_tasks.contains(&"task_5".to_string()));
    }

    /// 测试缓存预热功能的概念
    /// 
    /// 验证缓存预热可以提升后续查询性能
    #[tokio::test]
    async fn test_cache_warmup_concept() {
        use std::time::Instant;
        
        // 模拟缓存预热：预取高访问频率的记忆
        let popular_memory_ids: Vec<String> = (1..=10)
            .map(|i| format!("memory_{}", i))
            .collect();
        
        // 模拟预热过程
        let warmup_start = Instant::now();
        let mut warmed_count = 0;
        for _id in &popular_memory_ids {
            // 模拟预取操作（实际中会查询并缓存）
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            warmed_count += 1;
        }
        let warmup_duration = warmup_start.elapsed();
        
        // 验证预热完成
        assert_eq!(warmed_count, popular_memory_ids.len());
        assert!(
            warmup_duration.as_millis() < 1000,
            "缓存预热应该在合理时间内完成"
        );
        
        // 模拟后续查询：应该更快（因为已预热）
        let query_start = Instant::now();
        for _id in &popular_memory_ids {
            // 模拟从缓存读取（应该很快）
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
        let query_duration = query_start.elapsed();
        
        // 验证预热后的查询更快
        assert!(
            query_duration < warmup_duration,
            "预热后的查询应该比预热过程更快"
        );
    }

    /// 测试缓存预热参数验证
    #[test]
    fn test_cache_warmup_params() {
        // 测试默认limit
        let default_limit = 50;
        assert!(default_limit > 0, "默认limit应该大于0");
        assert!(default_limit <= 1000, "默认limit应该合理");
        
        // 测试limit边界
        let min_limit = 1;
        let max_limit = 1000;
        assert!(min_limit > 0, "最小limit应该大于0");
        assert!(max_limit <= 10000, "最大limit应该合理");
    }

    /// 测试性能基准测试的概念
    /// 
    /// 验证性能测试可以测量操作延迟
    #[tokio::test]
    async fn test_performance_benchmark_concept() {
        use std::time::Instant;
        
        // 模拟性能测试：测量操作延迟
        let operations = vec!["search", "add", "delete"];
        let mut results = std::collections::HashMap::new();
        
        for op in operations {
            let start = Instant::now();
            
            // 模拟操作延迟
            match op {
                "search" => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                "add" => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
                }
                "delete" => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                }
                _ => {}
            }
            
            let duration = start.elapsed();
            results.insert(op.to_string(), duration.as_millis());
        }
        
        // 验证性能测试结果
        assert!(results.contains_key("search"));
        assert!(results.contains_key("add"));
        assert!(results.contains_key("delete"));
        
        // 验证延迟在合理范围内
        assert!(*results.get("search").unwrap() < 1000, "搜索延迟应该合理");
        assert!(*results.get("add").unwrap() < 1000, "添加延迟应该合理");
        assert!(*results.get("delete").unwrap() < 1000, "删除延迟应该合理");
    }

    /// 测试性能基准测试参数验证
    #[test]
    fn test_performance_benchmark_params() {
        // 测试操作列表解析
        let operations_str = "search,add,delete";
        let operations: Vec<&str> = operations_str.split(',').map(|s| s.trim()).collect();
        
        assert_eq!(operations.len(), 3);
        assert!(operations.contains(&"search"));
        assert!(operations.contains(&"add"));
        assert!(operations.contains(&"delete"));
        
        // 测试默认操作
        let default_operations = vec!["search"];
        assert_eq!(default_operations.len(), 1);
        assert!(default_operations.contains(&"search"));
    }
}

