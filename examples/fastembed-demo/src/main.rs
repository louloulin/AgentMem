//! FastEmbed 演示示例
//!
//! 展示如何使用 FastEmbed 进行本地嵌入生成

use agent_mem_embeddings::EmbeddingFactory;
use agent_mem_traits::Embedder; // For trait methods
use anyhow::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 FastEmbed 演示开始");

    // 方式 1: 使用默认配置（零配置）
    info!("\n📦 方式 1: 零配置创建");
    let embedder = EmbeddingFactory::create_default().await?;

    info!("  - 提供商: {}", embedder.provider_name());
    info!("  - 模型: {}", embedder.model_name());
    info!("  - 维度: {}", embedder.dimension());

    // 测试单个嵌入
    info!("\n📝 测试单个嵌入");
    let text = "你好，世界！这是一个测试。";
    info!("  - 输入文本: {}", text);

    let embedding = embedder.embed(text).await?;
    info!("  - 嵌入维度: {}", embedding.len());
    info!("  - 前5个值: {:?}", &embedding[..5.min(embedding.len())]);

    // 测试批量嵌入
    info!("\n📚 测试批量嵌入");
    let texts = vec![
        "人工智能正在改变世界".to_string(),
        "机器学习是AI的核心技术".to_string(),
        "深度学习推动了AI的发展".to_string(),
    ];

    info!("  - 文本数量: {}", texts.len());
    let embeddings = embedder.embed_batch(&texts).await?;
    info!("  - 嵌入数量: {}", embeddings.len());

    for (i, emb) in embeddings.iter().enumerate() {
        info!("  - 嵌入 {}: {} 维", i + 1, emb.len());
    }

    // 测试语义相似度
    info!("\n🔍 测试语义相似度");
    let text1 = "我喜欢吃披萨";
    let text2 = "披萨是我最喜欢的食物";
    let text3 = "今天天气很好";

    let emb1 = embedder.embed(text1).await?;
    let emb2 = embedder.embed(text2).await?;
    let emb3 = embedder.embed(text3).await?;

    let sim_12 = cosine_similarity(&emb1, &emb2);
    let sim_13 = cosine_similarity(&emb1, &emb3);

    info!("  - 文本1: {}", text1);
    info!("  - 文本2: {}", text2);
    info!("  - 文本3: {}", text3);
    info!("  - 相似度(1-2): {:.4}", sim_12);
    info!("  - 相似度(1-3): {:.4}", sim_13);
    info!(
        "  - 结论: 相似句子的相似度 ({:.4}) {} 不相似句子 ({:.4})",
        sim_12,
        if sim_12 > sim_13 { ">" } else { "<" },
        sim_13
    );

    // 方式 2: 指定模型
    info!("\n📦 方式 2: 指定模型创建");
    let embedder2 = EmbeddingFactory::create_fastembed("bge-small-en-v1.5").await?;
    info!("  - 模型: {}", embedder2.model_name());
    info!("  - 维度: {}", embedder2.dimension());

    let embedding2 = embedder2.embed("Hello, world!").await?;
    info!("  - 嵌入维度: {}", embedding2.len());

    // 健康检查
    info!("\n🏥 健康检查");
    let is_healthy = embedder.health_check().await?;
    info!(
        "  - 状态: {}",
        if is_healthy {
            "✅ 健康"
        } else {
            "❌ 不健康"
        }
    );

    info!("\n✅ FastEmbed 演示完成");
    Ok(())
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "向量维度必须相同");

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
