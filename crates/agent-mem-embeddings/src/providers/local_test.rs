//! 本地嵌入提供商测试
//!
//! 验证本地嵌入模型的真实实现功能

use super::local::LocalEmbedder;
use crate::config::EmbeddingConfig;
use agent_mem_traits::{Embedder, Result};

#[tokio::test]
async fn test_local_embedder_creation() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;
    assert_eq!(embedder.provider_name(), "local");
    assert_eq!(embedder.dimension(), 384);

    Ok(())
}

#[tokio::test]
async fn test_deterministic_embedding() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 测试确定性嵌入
    let text = "This is a test sentence for embedding.";
    let embedding1 = embedder.embed(text).await?;
    let embedding2 = embedder.embed(text).await?;

    // 相同输入应该产生相同的嵌入
    assert_eq!(embedding1.len(), 384);
    assert_eq!(embedding2.len(), 384);
    assert_eq!(embedding1, embedding2);

    // 不同输入应该产生不同的嵌入
    let different_text = "This is a different sentence.";
    let embedding3 = embedder.embed(different_text).await?;
    assert_ne!(embedding1, embedding3);

    Ok(())
}

#[tokio::test]
async fn test_batch_embedding() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 2,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    let texts = vec![
        "First test sentence.".to_string(),
        "Second test sentence.".to_string(),
        "Third test sentence.".to_string(),
    ];

    let embeddings = embedder.embed_batch(&texts).await?;

    assert_eq!(embeddings.len(), 3);
    for embedding in &embeddings {
        assert_eq!(embedding.len(), 384);
    }

    // 验证批量处理和单独处理的结果一致
    for (i, text) in texts.iter().enumerate() {
        let single_embedding = embedder.embed(text).await?;
        assert_eq!(embeddings[i], single_embedding);
    }

    Ok(())
}

#[tokio::test]
async fn test_health_check() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 初始状态下健康检查应该返回 false（模型未加载）
    let health = embedder.health_check().await?;
    assert!(!health);

    Ok(())
}

#[tokio::test]
async fn test_embedding_normalization() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    let text = "Test sentence for normalization check.";
    let embedding = embedder.embed(text).await?;

    // 计算 L2 范数
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

    // 嵌入向量应该是归一化的（范数接近1）
    assert!((norm - 1.0).abs() < 0.01, "Embedding norm: {}", norm);

    Ok(())
}

#[cfg(feature = "onnx")]
#[tokio::test]
async fn test_onnx_model_type_detection() -> Result<()> {
    let config = EmbeddingConfig::local("/path/to/model.onnx", 384);

    let embedder = LocalEmbedder::new(config).await?;
    assert_eq!(embedder.model_name(), "model.onnx");

    Ok(())
}

#[cfg(feature = "local")]
#[tokio::test]
async fn test_candle_model_type_detection() -> Result<()> {
    let config = EmbeddingConfig::local("./models/bert-base-uncased", 384);

    let embedder = LocalEmbedder::new(config).await?;
    assert_eq!(embedder.model_name(), "bert-base-uncased");

    Ok(())
}

#[tokio::test]
async fn test_huggingface_model_type_detection() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;
    assert_eq!(
        embedder.model_name(),
        "sentence-transformers/all-MiniLM-L6-v2"
    );

    Ok(())
}

#[tokio::test]
async fn test_empty_text_embedding() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 测试空文本
    let embedding = embedder.embed("").await?;
    assert_eq!(embedding.len(), 384);

    // 测试空白文本
    let embedding = embedder.embed("   ").await?;
    assert_eq!(embedding.len(), 384);

    Ok(())
}

#[tokio::test]
async fn test_long_text_embedding() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 测试长文本（超过典型的512 token限制）
    let long_text = "This is a very long text. ".repeat(100);
    let embedding = embedder.embed(&long_text).await?;
    assert_eq!(embedding.len(), 384);

    Ok(())
}

#[tokio::test]
async fn test_unicode_text_embedding() -> Result<()> {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "test-model".to_string(),
        dimension: 384,
        batch_size: 32,
        ..Default::default()
    };

    let embedder = LocalEmbedder::new(config).await?;

    // 测试包含Unicode字符的文本
    let unicode_text = "这是一个测试句子。🚀 This includes emojis and 中文字符.";
    let embedding = embedder.embed(unicode_text).await?;
    assert_eq!(embedding.len(), 384);

    Ok(())
}
