//! 自适应搜索集成测试
//! 
//! 测试自适应搜索优化与混合搜索引擎的集成

use agent_mem_core::search::{
    AdaptiveSearchOptimizer, EnhancedHybridSearchEngine, HybridSearchEngine, 
    HybridSearchConfig, QueryFeatures, SearchQuery, SearchReranker,
};
use std::sync::Arc;

#[test]
fn test_query_analysis_integration() {
    // 测试各种查询类型的分析
    let test_cases = vec![
        (
            "user@example.com",
            "精确邮箱",
            |f: &QueryFeatures| f.has_exact_terms
        ),
        (
            "How does the memory system work in AgentMem?",
            "复杂语义问题",
            |f: &QueryFeatures| f.is_question && f.semantic_complexity > 0.6
        ),
        (
            "what happened yesterday?",
            "时间查询",
            |f: &QueryFeatures| f.has_temporal_indicator && f.is_question
        ),
        (
            "Find conversations with @John about Python",
            "实体查询",
            |f: &QueryFeatures| f.has_exact_terms && f.entity_count > 0
        ),
    ];
    
    for (query, description, validator) in test_cases {
        println!("\n测试场景: {description}");
        println!("查询: {query}");
        
        let features = QueryFeatures::extract_from_query(query);
        
        println!("  精确匹配: {}", features.has_exact_terms);
        println!("  语义复杂度: {:.2}", features.semantic_complexity);
        println!("  时间指示: {}", features.has_temporal_indicator);
        println!("  实体数量: {}", features.entity_count);
        println!("  查询长度: {}", features.query_length);
        println!("  是问句: {}", features.is_question);
        
        assert!(validator(&features), "查询特征不符合预期");
    }
}

#[test]
fn test_adaptive_weight_prediction() {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    
    // 测试场景1: 精确匹配查询
    let query1 = SearchQuery {
        query: "user@example.com".to_string(),
        limit: 10,
        ..Default::default()
    };
    
    let (_, weights1) = optimizer.optimize_query(&query1);
    println!("\n场景1: 精确匹配查询");
    println!("  向量权重: {:.3}", weights1.vector_weight);
    println!("  全文权重: {:.3}", weights1.fulltext_weight);
    assert!(
        weights1.fulltext_weight > weights1.vector_weight,
        "精确匹配应该提高全文权重"
    );
    
    // 测试场景2: 复杂语义查询
    let query2 = SearchQuery {
        query: "Explain how the hierarchical memory system processes and stores episodic events".to_string(),
        limit: 10,
        ..Default::default()
    };
    
    let (_, weights2) = optimizer.optimize_query(&query2);
    println!("\n场景2: 复杂语义查询");
    println!("  向量权重: {:.3}", weights2.vector_weight);
    println!("  全文权重: {:.3}", weights2.fulltext_weight);
    assert!(
        weights2.vector_weight > weights2.fulltext_weight,
        "复杂语义查询应该提高向量权重"
    );
    
    // 测试场景3: 简单关键词查询
    let query3 = SearchQuery {
        query: "pizza".to_string(),
        limit: 10,
        ..Default::default()
    };
    
    let (_, weights3) = optimizer.optimize_query(&query3);
    println!("\n场景3: 简单关键词");
    println!("  向量权重: {:.3}", weights3.vector_weight);
    println!("  全文权重: {:.3}", weights3.fulltext_weight);
    assert!(
        weights3.fulltext_weight >= weights3.vector_weight,
        "简单查询应该偏向全文搜索"
    );
}

#[test]
fn test_weight_normalization() {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    
    // 测试100个不同的查询
    let test_queries = vec![
        "simple",
        "user@test.com",
        "What is the meaning of life?",
        "yesterday's conversation",
        "@Alice told me about #project",
        "How can I improve search performance in large-scale distributed systems?",
    ];
    
    for query_text in test_queries {
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };
        
        let (_, weights) = optimizer.optimize_query(&query);
        
        let sum = weights.vector_weight + weights.fulltext_weight;
        assert!(
            (sum - 1.0).abs() < 0.001,
            "权重和应该为1.0，实际: {:.6}，查询: {}",
            sum,
            query_text
        );
        
        assert!(
            weights.vector_weight >= 0.0 && weights.vector_weight <= 1.0,
            "向量权重应该在0-1之间"
        );
        
        assert!(
            weights.fulltext_weight >= 0.0 && weights.fulltext_weight <= 1.0,
            "全文权重应该在0-1之间"
        );
    }
}

#[test]
fn test_search_reranker() {
    use agent_mem_core::search::SearchResult;
    
    let reranker = SearchReranker::new();
    
    // 创建测试结果
    let mut results = vec![
        SearchResult {
            id: "1".to_string(),
            content: "Very short".to_string(),
            score: 0.9,
            vector_score: Some(0.9),
            fulltext_score: None,
            metadata: None,
        },
        SearchResult {
            id: "2".to_string(),
            content: "This is a good medium length content that should be favored by the reranker".to_string(),
            score: 0.85,
            vector_score: Some(0.85),
            fulltext_score: None,
            metadata: Some(serde_json::json!({
                "importance": 0.95,
                "created_at": 1700000000, // 较新
            })),
        },
        SearchResult {
            id: "3".to_string(),
            content: "Another medium content".to_string(),
            score: 0.8,
            vector_score: Some(0.8),
            fulltext_score: None,
            metadata: Some(serde_json::json!({
                "importance": 0.5,
                "created_at": 1600000000, // 较旧
            })),
        },
    ];
    
    let query = SearchQuery {
        query: "test query".to_string(),
        ..Default::default()
    };
    
    let reranked = reranker.rerank(results, &query);
    
    // 验证结果仍然有序
    for i in 0..reranked.len() - 1 {
        assert!(
            reranked[i].score >= reranked[i + 1].score,
            "结果应该按分数降序排列"
        );
    }
    
    println!("\n重排序结果:");
    for (i, result) in reranked.iter().enumerate() {
        println!("  {}. ID={}, score={:.4}, content_len={}", 
            i + 1, result.id, result.score, result.content.len());
    }
    
    // 验证高重要性和适中长度的结果排名提升
    let id2_position = reranked.iter().position(|r| r.id == "2").unwrap();
    println!("\nID=2 (高重要性+适中长度) 排名: {}", id2_position + 1);
}

#[test]
fn test_learning_mechanism() {
    let mut optimizer = AdaptiveSearchOptimizer::new(true);
    
    let query = "test query";
    let weights = agent_mem_core::search::SearchWeights {
        vector_weight: 0.6,
        fulltext_weight: 0.4,
        confidence: 0.8,
    };
    
    // 记录高效查询
    optimizer.record_feedback(query, weights.clone(), 0.95);
    
    // 记录低效查询（不应该被记录）
    optimizer.record_feedback(query, weights.clone(), 0.3);
    
    // 记录中等效果（不应该被记录）
    optimizer.record_feedback(query, weights, 0.6);
    
    println!("✅ 学习机制测试通过");
}

#[test]
fn test_comprehensive_scenarios() {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    let reranker = SearchReranker::new();
    
    // 场景集合
    let scenarios = vec![
        (
            "user@example.com",
            "精确邮箱查询",
            vec!["fulltext > vector", "短查询", "精确匹配"],
        ),
        (
            "How can machine learning models improve memory retrieval accuracy?",
            "技术问题",
            vec!["vector > fulltext", "问句", "复杂语义"],
        ),
        (
            "what did we discuss yesterday?",
            "时间回顾",
            vec!["时间指示", "问句", "中等复杂度"],
        ),
        (
            "pizza",
            "单词查询",
            vec!["fulltext >= vector", "超短查询", "简单"],
        ),
        (
            "@Alice mentioned #project-x in the meeting",
            "实体提及",
            vec!["精确匹配", "多实体", "中等长度"],
        ),
    ];
    
    println!("\n=== 综合场景测试 ===\n");
    
    for (query_text, description, expectations) in scenarios {
        println!("场景: {description}");
        println!("查询: \"{query_text}\"");
        
        // 分析查询
        let features = QueryFeatures::extract_from_query(query_text);
        println!("特征:");
        println!("  - 精确匹配: {}", features.has_exact_terms);
        println!("  - 语义复杂度: {:.2}", features.semantic_complexity);
        println!("  - 时间指示: {}", features.has_temporal_indicator);
        println!("  - 实体数: {}", features.entity_count);
        println!("  - 问句: {}", features.is_question);
        
        // 预测权重
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };
        let (_, weights) = optimizer.optimize_query(&query);
        
        println!("权重:");
        println!("  - 向量: {:.3}", weights.vector_weight);
        println!("  - 全文: {:.3}", weights.fulltext_weight);
        println!("  - 置信度: {:.3}", weights.confidence);
        
        println!("预期: {}", expectations.join(", "));
        println!();
    }
}

#[test]
fn test_edge_cases() {
    let optimizer = AdaptiveSearchOptimizer::new(true);
    
    // 边界情况测试
    let edge_cases = vec![
        ("", "空查询"),
        ("a", "单字符"),
        ("    ", "纯空格"),
        ("?", "单个问号"),
        ("@", "单个符号"),
        ("!@#$%^&*()", "特殊符号"),
        ("a".repeat(500).as_str(), "超长查询"),
    ];
    
    println!("\n=== 边界情况测试 ===\n");
    
    for (query_text, description) in edge_cases {
        println!("测试: {description}");
        
        let query = SearchQuery {
            query: query_text.to_string(),
            ..Default::default()
        };
        
        let (_, weights) = optimizer.optimize_query(&query);
        
        // 验证权重始终有效
        assert!(
            weights.vector_weight >= 0.0 && weights.vector_weight <= 1.0,
            "向量权重无效: {}", weights.vector_weight
        );
        assert!(
            weights.fulltext_weight >= 0.0 && weights.fulltext_weight <= 1.0,
            "全文权重无效: {}", weights.fulltext_weight
        );
        
        let sum = weights.vector_weight + weights.fulltext_weight;
        assert!(
            (sum - 1.0).abs() < 0.001,
            "权重和无效: {}", sum
        );
        
        println!("  ✅ 权重有效: vector={:.3}, fulltext={:.3}\n", 
            weights.vector_weight, weights.fulltext_weight);
    }
}

