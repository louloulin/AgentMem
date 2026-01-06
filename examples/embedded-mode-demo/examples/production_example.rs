//! 生产环境嵌入式模式示例
//!
//! 展示如何在生产环境中使用 AgentMem 嵌入式模式：
//! - 完整的错误处理
//! - 数据持久化
//! - 批量操作
//! - 性能监控

use agent_mem_storage::backends::LanceDBVectorStore;
use agent_mem_traits::{VectorData, VectorStore};
use std::collections::HashMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🚀 AgentMem 生产环境嵌入式模式示例\n");

    // 2. 创建向量存储
    println!("💾 创建向量存储...");
    let store = LanceDBVectorStore::new("./production-data/vectors.lance", "vectors").await?;
    println!("✅ 向量存储创建成功\n");

    // 3. 批量插入向量（模拟生产数据）
    println!("📥 批量插入向量...");
    let start = Instant::now();

    let mut vectors = Vec::new();
    for i in 0..1000 {
        vectors.push(VectorData {
            id: format!("doc_{}", i),
            vector: generate_random_vector(1536),
            metadata: HashMap::from([
                ("type".to_string(), "document".to_string()),
                ("index".to_string(), i.to_string()),
                ("timestamp".to_string(), chrono::Utc::now().to_rfc3339()),
            ]),
        });
    }

    store.add_vectors(vectors).await?;
    let duration = start.elapsed();
    println!("✅ 插入 1000 个向量完成");
    println!("   耗时: {:?}", duration);
    println!("   吞吐量: {:.2} ops/s\n", 1000.0 / duration.as_secs_f64());

    // 4. 向量搜索性能测试
    println!("🔍 向量搜索性能测试...");
    let query_vector = generate_random_vector(1536);
    let start = Instant::now();

    let results = store.search_vectors(query_vector, 10, None).await?;
    let duration = start.elapsed();

    println!("✅ 搜索完成");
    println!("   找到结果: {} 个", results.len());
    println!("   搜索耗时: {:?}", duration);
    println!("   前 3 个结果:");
    for (i, result) in results.iter().take(3).enumerate() {
        println!(
            "     {}. ID: {}, 相似度: {:.4}",
            i + 1,
            result.id,
            result.similarity
        );
    }
    println!();

    // 5. 批量更新操作
    println!("📝 批量更新操作...");
    let start = Instant::now();

    let update_vectors: Vec<VectorData> = (0..100)
        .map(|i| VectorData {
            id: format!("doc_{}", i),
            vector: generate_random_vector(1536),
            metadata: HashMap::from([
                ("type".to_string(), "document".to_string()),
                ("index".to_string(), i.to_string()),
                ("updated".to_string(), "true".to_string()),
                ("timestamp".to_string(), chrono::Utc::now().to_rfc3339()),
            ]),
        })
        .collect();

    store.update_vectors(update_vectors).await?;
    let duration = start.elapsed();
    println!("✅ 更新 100 个向量完成");
    println!("   耗时: {:?}\n", duration);

    // 6. 验证更新
    println!("🔍 验证更新...");
    if let Some(vector) = store.get_vector("doc_0").await? {
        let updated = vector
            .metadata
            .get("updated")
            .map(|v| v == "true")
            .unwrap_or(false);
        println!("✅ 验证成功: doc_0 已更新 = {}\n", updated);
    }

    // 7. 批量删除操作
    println!("🗑️  批量删除操作...");
    let start = Instant::now();

    let delete_ids: Vec<String> = (900..1000).map(|i| format!("doc_{}", i)).collect();

    store.delete_vectors(delete_ids).await?;
    let duration = start.elapsed();
    println!("✅ 删除 100 个向量完成");
    println!("   耗时: {:?}\n", duration);

    // 8. 统计信息
    println!("📊 最终统计信息:");
    let stats = store.get_stats().await?;
    println!("   总向量数: {}", stats.total_vectors);
    println!("   向量维度: {}", stats.dimension);
    println!("   索引大小: {} bytes\n", stats.index_size);

    // 9. 健康检查
    println!("🏥 健康检查...");
    let health = store.health_check().await?;
    println!("✅ 健康状态: {:?}\n", health);

    println!("🎉 生产环境示例完成！");
    println!("💾 数据已持久化到: ./production-data/");
    println!("📝 下次启动时数据将自动加载");

    Ok(())
}

/// 生成随机向量（用于测试）
fn generate_random_vector(dimension: usize) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..dimension).map(|_| rng.gen::<f32>()).collect()
}
