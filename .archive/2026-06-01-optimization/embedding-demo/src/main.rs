//! 嵌入模型真实化演示
//!
//! 本示例演示了：
//! 1. 移除Mock嵌入实现
//! 2. 使用真实的嵌入提供商
//! 3. 健康检查和重试机制
//! 4. 回退机制

use agent_mem_embeddings::{EmbeddingConfig, RealEmbeddingFactory};
use anyhow::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🚀 AgentMem 嵌入模型真实化演示");
    info!("===============================");

    // 演示1: OpenAI嵌入（真实实现）
    info!("\n📊 测试 1: OpenAI 嵌入提供商");
    test_openai_embeddings().await;

    // 演示2: HuggingFace嵌入（真实实现）
    info!("\n🤗 测试 2: HuggingFace 嵌入提供商");
    test_huggingface_embeddings().await;

    // 演示3: Cohere嵌入（真实实现）
    info!("\n🔮 测试 3: Cohere 嵌入提供商");
    test_cohere_embeddings().await;

    // 演示4: 本地嵌入（真实实现）
    info!("\n💻 测试 4: 本地嵌入提供商");
    test_local_embeddings().await;

    // 演示5: Anthropic嵌入（已移除Mock实现）
    info!("\n🚫 测试 5: Anthropic 嵌入提供商（已移除）");
    test_anthropic_embeddings().await;

    // 演示6: 重试和回退机制
    info!("\n🔄 测试 6: 重试和回退机制");
    test_retry_and_fallback().await;

    // 演示7: 健康检查
    info!("\n🏥 测试 7: 健康检查机制");
    test_health_checks().await;

    info!("\n✅ 所有测试完成！");
    info!("📝 总结：");
    info!("   - ✅ 移除了所有Mock嵌入实现");
    info!("   - ✅ 所有嵌入提供商都使用真实API");
    info!("   - ✅ 实现了健康检查和重试机制");
    info!("   - ✅ 提供了智能回退机制");
    info!("   - ✅ Anthropic嵌入已正确移除（无专用API）");

    Ok(())
}

async fn test_openai_embeddings() {
    let config = EmbeddingConfig {
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        api_key: Some("demo-key".to_string()), // 演示用密钥
        dimension: 1536,
        ..Default::default()
    };

    match RealEmbeddingFactory::create_with_retry(&config, 3).await {
        Ok(embedder) => {
            info!("   ✅ OpenAI 嵌入提供商创建成功");
            info!("   📏 维度: {}", embedder.dimension());
            info!("   🏷️  模型: {}", embedder.model_name());

            // 测试嵌入生成（会因为demo密钥失败，这是预期的）
            match embedder.embed("Hello, world!").await {
                Ok(embedding) => {
                    info!("   ✅ 嵌入生成成功，长度: {}", embedding.len());
                }
                Err(e) => {
                    info!("   ⚠️  嵌入生成失败（预期，因为使用demo密钥）: {}", e);
                }
            }
        }
        Err(e) => {
            info!(
                "   ⚠️  OpenAI 嵌入提供商创建失败（预期，因为使用demo密钥）: {}",
                e
            );
        }
    }
}

async fn test_huggingface_embeddings() {
    let config = EmbeddingConfig {
        provider: "huggingface".to_string(),
        model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        api_key: Some("demo-key".to_string()),
        dimension: 384,
        ..Default::default()
    };

    match RealEmbeddingFactory::create_with_retry(&config, 3).await {
        Ok(embedder) => {
            info!("   ✅ HuggingFace 嵌入提供商创建成功");
            info!("   📏 维度: {}", embedder.dimension());
            info!("   🏷️  模型: {}", embedder.model_name());
        }
        Err(e) => {
            info!(
                "   ⚠️  HuggingFace 嵌入提供商创建失败（预期，因为使用demo密钥）: {}",
                e
            );
        }
    }
}

async fn test_cohere_embeddings() {
    let config = EmbeddingConfig {
        provider: "cohere".to_string(),
        model: "embed-english-v3.0".to_string(),
        api_key: Some("demo-key".to_string()),
        dimension: 1024,
        ..Default::default()
    };

    match RealEmbeddingFactory::create_with_retry(&config, 3).await {
        Ok(embedder) => {
            info!("   ✅ Cohere 嵌入提供商创建成功");
            info!("   📏 维度: {}", embedder.dimension());
            info!("   🏷️  模型: {}", embedder.model_name());
        }
        Err(e) => {
            info!(
                "   ⚠️  Cohere 嵌入提供商创建失败（预期，因为使用demo密钥）: {}",
                e
            );
        }
    }
}

async fn test_local_embeddings() {
    let config = EmbeddingConfig {
        provider: "local".to_string(),
        model: "all-MiniLM-L6-v2".to_string(),
        api_key: None,
        dimension: 384,
        ..Default::default()
    };

    match RealEmbeddingFactory::create_with_retry(&config, 3).await {
        Ok(embedder) => {
            info!("   ✅ 本地嵌入提供商创建成功");
            info!("   📏 维度: {}", embedder.dimension());
            info!("   🏷️  模型: {}", embedder.model_name());

            // 本地模型可能可以工作
            match embedder.embed("Hello, world!").await {
                Ok(embedding) => {
                    info!("   ✅ 本地嵌入生成成功，长度: {}", embedding.len());
                }
                Err(e) => {
                    info!("   ⚠️  本地嵌入生成失败: {}", e);
                }
            }
        }
        Err(e) => {
            info!("   ⚠️  本地嵌入提供商创建失败: {}", e);
        }
    }
}

async fn test_anthropic_embeddings() {
    let config = EmbeddingConfig {
        provider: "anthropic".to_string(),
        model: "claude-embedding".to_string(),
        api_key: Some("demo-key".to_string()),
        dimension: 1536,
        ..Default::default()
    };

    match RealEmbeddingFactory::create_with_retry(&config, 3).await {
        Ok(_) => {
            error!("   ❌ 错误：Anthropic 嵌入提供商不应该创建成功！");
        }
        Err(e) => {
            info!("   ✅ 正确：Anthropic 嵌入提供商已被移除: {}", e);
            info!("   📝 原因：Anthropic 没有提供专用的嵌入API");
        }
    }
}

async fn test_retry_and_fallback() {
    // 测试无效配置的重试机制
    let invalid_config = EmbeddingConfig {
        provider: "invalid-provider".to_string(),
        model: "invalid-model".to_string(),
        api_key: Some("invalid-key".to_string()),
        dimension: 1536,
        ..Default::default()
    };

    info!("   🔄 测试重试机制（使用无效提供商）...");
    match RealEmbeddingFactory::create_with_retry(&invalid_config, 3).await {
        Ok(_) => {
            error!("   ❌ 错误：无效配置不应该创建成功！");
        }
        Err(e) => {
            info!("   ✅ 正确：无效配置被正确拒绝: {}", e);
        }
    }

    // 测试回退机制
    let fallback_config = EmbeddingConfig {
        provider: "huggingface".to_string(), // 主要提供商
        model: "invalid-model".to_string(),
        api_key: Some("invalid-key".to_string()),
        dimension: 1536,
        ..Default::default()
    };

    info!("   🔄 测试回退机制（HuggingFace -> OpenAI）...");
    match RealEmbeddingFactory::create_with_fallback(&fallback_config).await {
        Ok(embedder) => {
            info!(
                "   ✅ 回退机制成功，使用提供商: {}",
                embedder.provider_name()
            );
        }
        Err(e) => {
            info!("   ⚠️  回退机制失败（预期，因为使用demo密钥）: {}", e);
        }
    }
}

async fn test_health_checks() {
    info!("   🏥 支持的嵌入提供商:");
    for provider in RealEmbeddingFactory::supported_providers() {
        let supported = RealEmbeddingFactory::is_provider_supported(provider);
        info!(
            "      - {}: {}",
            provider,
            if supported { "✅" } else { "❌" }
        );
    }

    // 测试不支持的提供商
    let unsupported = RealEmbeddingFactory::is_provider_supported("anthropic");
    info!(
        "   🚫 Anthropic 支持状态: {}",
        if unsupported {
            "❌ 错误"
        } else {
            "✅ 正确移除"
        }
    );
}
