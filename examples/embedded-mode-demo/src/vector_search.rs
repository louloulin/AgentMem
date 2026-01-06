//! AgentMem 向量搜索示例
//!
//! 演示如何使用 LanceDB 进行向量存储和语义搜索

use agent_mem_storage::backends::lancedb_store::LanceDBStore;
use agent_mem_traits::{VectorData, VectorStore};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 AgentMem 向量搜索示例");

    // 1. 创建 LanceDB 向量存储
    info!("🔧 创建 LanceDB 向量存储...");
    let vector_store = LanceDBStore::new("./data/vectors.lance", "embeddings").await?;
    info!("✅ LanceDB 向量存储创建成功");

    // 2. 准备示例向量数据（模拟文本嵌入）
    info!("\n📝 准备示例向量数据...");

    let vectors = vec![
        VectorData {
            id: "doc1".to_string(),
            vector: generate_mock_embedding("Rust 是一门系统编程语言"),
            metadata: HashMap::from([
                ("text".to_string(), "Rust 是一门系统编程语言".to_string()),
                ("category".to_string(), "programming".to_string()),
                ("language".to_string(), "zh".to_string()),
            ]),
        },
        VectorData {
            id: "doc2".to_string(),
            vector: generate_mock_embedding("Python 是一门高级编程语言"),
            metadata: HashMap::from([
                ("text".to_string(), "Python 是一门高级编程语言".to_string()),
                ("category".to_string(), "programming".to_string()),
                ("language".to_string(), "zh".to_string()),
            ]),
        },
        VectorData {
            id: "doc3".to_string(),
            vector: generate_mock_embedding("机器学习是人工智能的一个分支"),
            metadata: HashMap::from([
                (
                    "text".to_string(),
                    "机器学习是人工智能的一个分支".to_string(),
                ),
                ("category".to_string(), "ai".to_string()),
                ("language".to_string(), "zh".to_string()),
            ]),
        },
        VectorData {
            id: "doc4".to_string(),
            vector: generate_mock_embedding("深度学习使用神经网络"),
            metadata: HashMap::from([
                ("text".to_string(), "深度学习使用神经网络".to_string()),
                ("category".to_string(), "ai".to_string()),
                ("language".to_string(), "zh".to_string()),
            ]),
        },
        VectorData {
            id: "doc5".to_string(),
            vector: generate_mock_embedding("数据库用于存储和管理数据"),
            metadata: HashMap::from([
                ("text".to_string(), "数据库用于存储和管理数据".to_string()),
                ("category".to_string(), "database".to_string()),
                ("language".to_string(), "zh".to_string()),
            ]),
        },
    ];

    info!("✅ 准备了 {} 个向量", vectors.len());

    // 3. 插入向量
    info!("\n💾 插入向量到 LanceDB...");
    let start = std::time::Instant::now();
    vector_store.add_vectors(vectors.clone()).await?;
    let duration = start.elapsed();
    info!("✅ 插入完成，耗时: {:?}", duration);

    // 4. 执行语义搜索
    info!("\n🔍 执行语义搜索...");

    // 搜索 1: 查找与 "编程语言" 相关的文档
    info!("\n查询 1: 查找与 '编程语言' 相关的文档");
    let query1 = generate_mock_embedding("编程语言");
    let results1 = vector_store.search_vectors(query1, 3, None).await?;

    info!("找到 {} 个结果:", results1.len());
    for (i, result) in results1.iter().enumerate() {
        let text = result
            .metadata
            .get("text")
            .map(|v| v.as_str())
            .unwrap_or("N/A");
        info!("  {}. [相似度: {:.4}] {}", i + 1, result.similarity, text);
    }

    // 搜索 2: 查找与 "人工智能" 相关的文档
    info!("\n查询 2: 查找与 '人工智能' 相关的文档");
    let query2 = generate_mock_embedding("人工智能");
    let results2 = vector_store.search_vectors(query2, 3, Some(0.5)).await?;

    info!("找到 {} 个结果 (相似度阈值 > 0.5):", results2.len());
    for (i, result) in results2.iter().enumerate() {
        let text = result
            .metadata
            .get("text")
            .map(|v| v.as_str())
            .unwrap_or("N/A");
        info!("  {}. [相似度: {:.4}] {}", i + 1, result.similarity, text);
    }

    // 5. 获取单个向量
    info!("\n📄 获取单个向量...");
    if let Some(vector) = vector_store.get_vector("doc1").await? {
        let text = vector
            .metadata
            .get("text")
            .map(|v| v.as_str())
            .unwrap_or("N/A");
        info!("✅ 找到向量 doc1: {}", text);
        info!("   向量维度: {}", vector.vector.len());
    }

    // 6. 更新向量
    info!("\n📝 更新向量...");
    let updated_vector = VectorData {
        id: "doc1".to_string(),
        vector: generate_mock_embedding("Rust 是一门安全高效的系统编程语言"),
        metadata: HashMap::from([
            (
                "text".to_string(),
                "Rust 是一门安全高效的系统编程语言".to_string(),
            ),
            ("category".to_string(), "programming".to_string()),
            ("language".to_string(), "zh".to_string()),
            ("updated".to_string(), "true".to_string()),
        ]),
    };

    vector_store.update_vectors(vec![updated_vector]).await?;
    info!("✅ 向量更新成功");

    // 验证更新
    if let Some(vector) = vector_store.get_vector("doc1").await? {
        let text = vector
            .metadata
            .get("text")
            .map(|v| v.as_str())
            .unwrap_or("N/A");
        let updated = vector
            .metadata
            .get("updated")
            .map(|v| v.as_str())
            .unwrap_or("false");
        info!("✅ 验证更新: {} (updated={})", text, updated);
    }

    // 7. 删除向量
    info!("\n🗑️  删除向量...");
    vector_store
        .delete_vectors(vec!["doc5".to_string()])
        .await?;
    info!("✅ 向量 doc5 已删除");

    // 验证删除
    if let Some(_) = vector_store.get_vector("doc5").await? {
        info!("❌ 错误: 向量 doc5 应该已被删除");
    } else {
        info!("✅ 验证删除成功: 向量 doc5 不存在");
    }

    // 8. 统计信息
    info!("\n📊 向量存储统计信息...");
    let stats = vector_store.get_stats().await?;
    info!("  总向量数: {}", stats.total_vectors);
    info!("  向量维度: {}", stats.dimension);
    info!("  索引大小: {} bytes", stats.index_size);

    info!("\n🎉 向量搜索示例完成！");
    info!("💾 向量数据已保存到: ./data/vectors.lance");

    Ok(())
}

/// 生成模拟的文本嵌入向量
///
/// 注意: 这只是一个简化的示例，实际应用中应该使用真实的嵌入模型
/// 如 OpenAI embeddings, sentence-transformers 等
fn generate_mock_embedding(text: &str) -> Vec<f32> {
    // 使用简单的哈希函数生成确定性的向量
    // 实际应用中应该使用真实的嵌入模型
    let mut vector = vec![0.0; 1536];

    // 基于文本内容生成向量
    for (i, byte) in text.bytes().enumerate() {
        let idx = (i * 7 + byte as usize) % 1536;
        vector[idx] += 0.1;
    }

    // 归一化
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vector {
            *v /= norm;
        }
    }

    vector
}
