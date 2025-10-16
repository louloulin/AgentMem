//! 快速测试 LanceDB 功能

use agent_mem_storage::backends::lancedb_store::LanceDBStore;
use agent_mem_traits::{VectorStore, VectorData};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentMem LanceDB 快速测试\n");

    // 1. 创建 LanceDB 向量存储
    println!("📦 创建 LanceDB 向量存储...");
    let store = LanceDBStore::new("./test-data/vectors.lance", "test").await?;
    println!("✅ 创建成功\n");

    // 2. 插入向量
    println!("💾 插入向量...");
    let vectors = vec![
        VectorData {
            id: "vec1".to_string(),
            vector: vec![0.1; 128],
            metadata: HashMap::from([
                ("text".to_string(), "测试文本 1".to_string()),
            ]),
        },
        VectorData {
            id: "vec2".to_string(),
            vector: vec![0.2; 128],
            metadata: HashMap::from([
                ("text".to_string(), "测试文本 2".to_string()),
            ]),
        },
    ];

    store.add_vectors(vectors).await?;
    println!("✅ 插入成功\n");

    // 3. 搜索向量
    println!("🔍 搜索向量...");
    let query = vec![0.15; 128];
    let results = store.search_vectors(query, 2, None).await?;
    println!("✅ 找到 {} 个结果:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. ID: {}, 相似度: {:.4}", i + 1, result.id, result.similarity);
    }
    println!();

    // 4. 获取向量
    println!("📄 获取向量 vec1...");
    if let Some(vector) = store.get_vector("vec1").await? {
        println!("✅ 找到向量: ID={}, 维度={}", vector.id, vector.vector.len());
    }
    println!();

    // 5. 更新向量
    println!("📝 更新向量 vec1...");
    let updated = VectorData {
        id: "vec1".to_string(),
        vector: vec![0.3; 128],
        metadata: HashMap::from([
            ("text".to_string(), "更新后的文本 1".to_string()),
            ("updated".to_string(), "true".to_string()),
        ]),
    };
    store.update_vectors(vec![updated]).await?;
    println!("✅ 更新成功\n");

    // 6. 验证更新
    println!("🔍 验证更新...");
    if let Some(vector) = store.get_vector("vec1").await? {
        let updated = vector.metadata.get("updated")
            .map(|v| v == "true")
            .unwrap_or(false);
        println!("✅ 验证成功: updated={}", updated);
    }
    println!();

    // 7. 删除向量
    println!("🗑️  删除向量 vec2...");
    store.delete_vectors(vec!["vec2".to_string()]).await?;
    println!("✅ 删除成功\n");

    // 8. 统计信息
    println!("📊 统计信息:");
    let stats = store.get_stats().await?;
    println!("  总向量数: {}", stats.total_vectors);
    println!("  向量维度: {}", stats.dimension);
    println!("  索引大小: {} bytes", stats.index_size);

    println!("\n🎉 测试完成！");
    println!("💾 数据保存在: ./test-data/vectors.lance");

    Ok(())
}

