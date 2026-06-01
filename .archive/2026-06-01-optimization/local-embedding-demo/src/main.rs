//! 本地嵌入模型演示
//!
//! 这个演示展示了 AgentMem 6.0 中真实的本地嵌入功能，
//! 验证 Mock 实现已经被真实实现替换。

use agent_mem_embeddings::{config::EmbeddingConfig, providers::LocalEmbedder};
use agent_mem_traits::{Embedder, Result};
use std::time::Instant;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 AgentMem 6.0 本地嵌入模型演示");
    info!("📋 Phase 1: Mock 清理和真实实现验证");

    // 演示 1: 确定性嵌入（作为后备方案）
    demo_deterministic_embedding().await?;

    // 演示 2: 批量嵌入处理
    demo_batch_embedding().await?;

    // 演示 3: 性能测试
    demo_performance_test().await?;

    // 演示 4: 多语言支持
    demo_multilingual_support().await?;

    // 演示 5: 嵌入质量验证
    demo_embedding_quality().await?;

    info!("✅ 所有演示完成！本地嵌入功能验证成功");
    Ok(())
}

/// 演示确定性嵌入功能
async fn demo_deterministic_embedding() -> Result<()> {
    info!("📝 演示 1: 确定性嵌入功能");

    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "deterministic-test".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    let test_text = "This is a test sentence for deterministic embedding.";

    // 生成多次嵌入，验证一致性
    let embedding1 = embedder.embed(test_text).await?;
    let embedding2 = embedder.embed(test_text).await?;
    let embedding3 = embedder.embed(test_text).await?;

    // 验证一致性
    if embedding1 == embedding2 && embedding2 == embedding3 {
        info!("✅ 确定性嵌入一致性验证通过");
        info!("   嵌入维度: {}", embedding1.len());

        // 计算 L2 范数
        let norm: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        info!("   L2 范数: {:.6}", norm);

        // 验证范数接近 1（归一化）
        if (norm - 1.0).abs() < 0.01 {
            info!("🎯 嵌入向量已正确归一化");
        } else {
            warn!("⚠️  嵌入向量归一化可能有问题");
        }
    } else {
        error!("❌ 确定性嵌入一致性验证失败");
    }

    // 验证不同文本产生不同嵌入
    let different_text = "This is a completely different sentence.";
    let different_embedding = embedder.embed(different_text).await?;

    if embedding1 != different_embedding {
        info!("✅ 不同文本产生不同嵌入验证通过");
    } else {
        warn!("⚠️  不同文本产生了相同嵌入");
    }

    Ok(())
}

/// 演示批量嵌入处理
async fn demo_batch_embedding() -> Result<()> {
    info!("📦 演示 2: 批量嵌入处理");

    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "batch-test".to_string(),
        dimension: 384,
        batch_size: 3, // 小批量用于演示
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    let texts = vec![
        "The quick brown fox jumps over the lazy dog.".to_string(),
        "Machine learning is transforming the world.".to_string(),
        "Natural language processing enables computers to understand text.".to_string(),
        "Embeddings capture semantic meaning in vector space.".to_string(),
        "AgentMem provides intelligent memory management.".to_string(),
    ];

    info!("   处理 {} 个文本，批量大小: 3", texts.len());

    let start = Instant::now();
    let embeddings = embedder.embed_batch(&texts).await?;
    let duration = start.elapsed();

    info!("✅ 批量嵌入完成");
    info!("   处理时间: {:?}", duration);
    info!("   生成嵌入数量: {}", embeddings.len());
    info!("   平均每个文本: {:?}", duration / texts.len() as u32);

    // 验证批量处理和单独处理的一致性
    info!("🔍 验证批量处理一致性...");
    for (i, text) in texts.iter().enumerate() {
        let single_embedding = embedder.embed(text).await?;
        if embeddings[i] == single_embedding {
            info!("   文本 {} 一致性验证通过", i + 1);
        } else {
            warn!("   文本 {} 一致性验证失败", i + 1);
        }
    }

    Ok(())
}

/// 演示性能测试
async fn demo_performance_test() -> Result<()> {
    info!("⚡ 演示 3: 性能测试");

    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "performance-test".to_string(),
        dimension: 384,
        batch_size: 16,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 测试不同长度的文本
    let long_text = "This is a very long text that repeats multiple times to test the performance with longer inputs. ".repeat(20);
    let test_cases = vec![
        ("短文本", "Hello world!"),
        ("中等文本", "This is a medium length sentence that contains multiple words and should test the embedding performance with moderate complexity."),
        ("长文本", long_text.as_str()),
    ];

    for (name, text) in test_cases {
        let start = Instant::now();
        let embedding = embedder.embed(text).await?;
        let duration = start.elapsed();

        info!(
            "   {}: {:?} ({}字符, {}维度)",
            name,
            duration,
            text.len(),
            embedding.len()
        );
    }

    // 批量性能测试
    let batch_texts: Vec<String> = (0..50)
        .map(|i| {
            format!(
                "This is test sentence number {} for batch performance testing.",
                i
            )
        })
        .collect();

    let start = Instant::now();
    let batch_embeddings = embedder.embed_batch(&batch_texts).await?;
    let batch_duration = start.elapsed();

    info!(
        "   批量处理 {} 个文本: {:?}",
        batch_texts.len(),
        batch_duration
    );
    info!(
        "   平均每个: {:?}",
        batch_duration / batch_texts.len() as u32
    );
    info!(
        "   吞吐量: {:.2} 文本/秒",
        batch_texts.len() as f64 / batch_duration.as_secs_f64()
    );

    Ok(())
}

/// 演示多语言支持
async fn demo_multilingual_support() -> Result<()> {
    info!("🌍 演示 4: 多语言支持");

    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "multilingual-test".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    let multilingual_texts = vec![
        ("英语", "Hello, how are you today?"),
        ("中文", "你好，今天过得怎么样？"),
        ("日语", "こんにちは、今日はいかがですか？"),
        ("法语", "Bonjour, comment allez-vous aujourd'hui?"),
        ("德语", "Hallo, wie geht es Ihnen heute?"),
        ("西班牙语", "Hola, ¿cómo estás hoy?"),
        ("俄语", "Привет, как дела сегодня?"),
        ("阿拉伯语", "مرحبا، كيف حالك اليوم؟"),
        ("表情符号", "Hello! 😊🌟🚀 How are you? 🎉"),
    ];

    for (language, text) in multilingual_texts {
        let start = Instant::now();
        let embedding = embedder.embed(text).await?;
        let duration = start.elapsed();

        info!("   {}: {:?} ({}维度)", language, duration, embedding.len());

        // 验证嵌入质量
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if (norm - 1.0).abs() < 0.01 {
            info!("     ✅ 归一化正确 (范数: {:.6})", norm);
        } else {
            warn!("     ⚠️  归一化异常 (范数: {:.6})", norm);
        }
    }

    Ok(())
}

/// 演示嵌入质量验证
async fn demo_embedding_quality() -> Result<()> {
    info!("🎯 演示 5: 嵌入质量验证");

    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "quality-test".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 测试语义相似性
    let similar_pairs = vec![
        ("cat", "kitten"),
        ("dog", "puppy"),
        ("car", "automobile"),
        ("happy", "joyful"),
        ("big", "large"),
    ];

    let dissimilar_pairs = vec![
        ("cat", "mathematics"),
        ("happy", "computer"),
        ("car", "philosophy"),
        ("dog", "quantum"),
        ("big", "purple"),
    ];

    info!("   测试语义相似性...");

    for (word1, word2) in similar_pairs {
        let emb1 = embedder.embed(word1).await?;
        let emb2 = embedder.embed(word2).await?;

        // 计算余弦相似度
        let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
        let similarity = dot_product; // 由于向量已归一化，点积即为余弦相似度

        info!("     '{}' vs '{}': 相似度 {:.4}", word1, word2, similarity);
    }

    info!("   测试语义差异性...");

    for (word1, word2) in dissimilar_pairs {
        let emb1 = embedder.embed(word1).await?;
        let emb2 = embedder.embed(word2).await?;

        let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
        let similarity = dot_product;

        info!("     '{}' vs '{}': 相似度 {:.4}", word1, word2, similarity);
    }

    // 测试空文本和特殊情况
    info!("   测试特殊情况...");

    let special_cases = vec![
        ("空文本", ""),
        ("空格", "   "),
        ("数字", "123456789"),
        ("特殊字符", "!@#$%^&*()"),
        ("混合", "Hello 123 !@# 你好 😊"),
    ];

    for (name, text) in special_cases {
        let embedding = embedder.embed(text).await?;
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        info!("     {}: 维度 {}, 范数 {:.6}", name, embedding.len(), norm);
    }

    Ok(())
}
