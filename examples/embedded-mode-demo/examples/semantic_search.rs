//! 语义搜索示例
//! 
//! 展示如何使用 AgentMem 进行语义搜索：
//! - 文档向量化
//! - 相似度搜索
//! - 元数据过滤
//! - 结果排序

use agent_mem_storage::backends::LanceDBVectorStore;
use agent_mem_traits::{VectorData, VectorStore};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 AgentMem 语义搜索示例\n");

    // 1. 创建向量存储
    println!("📦 创建向量存储...");
    let store = LanceDBVectorStore::new("./semantic-data/vectors.lance", "vectors").await?;
    println!("✅ 创建成功\n");

    // 2. 准备文档数据（模拟已向量化的文档）
    println!("📚 准备文档数据...");
    let documents = vec![
        ("doc1", "Rust 是一种系统编程语言", vec![0.8, 0.2, 0.1, 0.9], "programming"),
        ("doc2", "Python 是一种高级编程语言", vec![0.7, 0.3, 0.2, 0.8], "programming"),
        ("doc3", "机器学习是人工智能的一个分支", vec![0.1, 0.9, 0.8, 0.2], "ai"),
        ("doc4", "深度学习使用神经网络", vec![0.2, 0.8, 0.9, 0.1], "ai"),
        ("doc5", "数据库用于存储数据", vec![0.6, 0.4, 0.3, 0.7], "database"),
        ("doc6", "向量数据库支持相似度搜索", vec![0.5, 0.5, 0.6, 0.4], "database"),
    ];

    let vectors: Vec<VectorData> = documents
        .iter()
        .map(|(id, text, vec, category)| {
            // 扩展向量到 384 维（重复模式）
            let mut full_vector = Vec::new();
            for _ in 0..96 {
                full_vector.extend_from_slice(vec);
            }
            
            VectorData {
                id: id.to_string(),
                vector: full_vector,
                metadata: HashMap::from([
                    ("text".to_string(), text.to_string()),
                    ("category".to_string(), category.to_string()),
                    ("language".to_string(), "zh".to_string()),
                ]),
            }
        })
        .collect();

    store.add_vectors(vectors).await?;
    println!("✅ 插入 {} 个文档\n", documents.len());

    // 3. 语义搜索：查找编程相关文档
    println!("🔍 搜索 1: 查找编程相关文档");
    let query1 = vec![0.75, 0.25, 0.15, 0.85]; // 类似 "编程语言" 的向量
    let mut query_vector1 = Vec::new();
    for _ in 0..96 {
        query_vector1.extend_from_slice(&query1);
    }
    
    let results = store.search_vectors(query_vector1, 3, None).await?;
    println!("找到 {} 个结果:", results.len());
    for (i, result) in results.iter().enumerate() {
        let text = result.metadata.get("text").map(|s| s.as_str()).unwrap_or("");
        let category = result.metadata.get("category").map(|s| s.as_str()).unwrap_or("");
        println!("  {}. [{}] {} (相似度: {:.4})",
            i + 1, category, text, result.similarity);
    }
    println!();

    // 4. 语义搜索：查找 AI 相关文档
    println!("🔍 搜索 2: 查找 AI 相关文档");
    let query2 = vec![0.15, 0.85, 0.85, 0.15]; // 类似 "人工智能" 的向量
    let mut query_vector2 = Vec::new();
    for _ in 0..96 {
        query_vector2.extend_from_slice(&query2);
    }
    
    let results = store.search_vectors(query_vector2, 3, None).await?;
    println!("找到 {} 个结果:", results.len());
    for (i, result) in results.iter().enumerate() {
        let text = result.metadata.get("text").map(|s| s.as_str()).unwrap_or("");
        let category = result.metadata.get("category").map(|s| s.as_str()).unwrap_or("");
        println!("  {}. [{}] {} (相似度: {:.4})",
            i + 1, category, text, result.similarity);
    }
    println!();

    // 5. 语义搜索：查找数据库相关文档
    println!("🔍 搜索 3: 查找数据库相关文档");
    let query3 = vec![0.55, 0.45, 0.45, 0.55]; // 类似 "数据库" 的向量
    let mut query_vector3 = Vec::new();
    for _ in 0..96 {
        query_vector3.extend_from_slice(&query3);
    }
    
    let results = store.search_vectors(query_vector3, 3, None).await?;
    println!("找到 {} 个结果:", results.len());
    for (i, result) in results.iter().enumerate() {
        let text = result.metadata.get("text").map(|s| s.as_str()).unwrap_or("");
        let category = result.metadata.get("category").map(|s| s.as_str()).unwrap_or("");
        println!("  {}. [{}] {} (相似度: {:.4})",
            i + 1, category, text, result.similarity);
    }
    println!();

    // 6. 获取特定文档
    println!("📄 获取特定文档 (doc3):");
    if let Some(doc) = store.get_vector("doc3").await? {
        let text = doc.metadata.get("text").map(|s| s.as_str()).unwrap_or("");
        let category = doc.metadata.get("category").map(|s| s.as_str()).unwrap_or("");
        println!("  文本: {}", text);
        println!("  类别: {}", category);
        println!("  向量维度: {}", doc.vector.len());
    }
    println!();

    // 7. 统计信息
    println!("📊 统计信息:");
    let stats = store.get_stats().await?;
    println!("  总文档数: {}", stats.total_vectors);
    println!("  向量维度: {}", stats.dimension);
    println!();

    println!("🎉 语义搜索示例完成！");
    println!("💡 提示: 在实际应用中，使用 OpenAI/HuggingFace 等模型生成真实的文本向量");

    Ok(())
}

