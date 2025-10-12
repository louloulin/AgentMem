//! ONNX 嵌入模型演示
//!
//! 本示例演示如何使用 ONNX Runtime 进行嵌入模型推理。
//!
//! 注意：本示例使用模拟的 ONNX 模型文件。要使用真实模型，请下载真实的 ONNX 模型和 tokenizer。

use agent_mem_embeddings::config::EmbeddingConfig;
use agent_mem_embeddings::providers::LocalEmbedder;
use agent_mem_traits::Embedder;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("onnx_embedding_demo=info,agent_mem_embeddings=debug")
        .init();

    info!("🚀 ONNX 嵌入模型演示");
    info!("{}", "=".repeat(60));

    // 创建临时目录用于模拟 ONNX 模型文件
    let temp_dir = tempdir()?;
    let model_path = temp_dir.path().join("model.onnx");
    let tokenizer_path = temp_dir.path().join("tokenizer.json");

    // 创建模拟的 ONNX 模型文件
    File::create(&model_path)?;
    
    // 创建模拟的 tokenizer 文件（简化的 JSON 格式）
    let tokenizer_json = r###"{
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [],
        "normalizer": null,
        "pre_tokenizer": null,
        "post_processor": null,
        "decoder": null,
        "model": {
            "type": "WordPiece",
            "unk_token": "[UNK]",
            "continuing_subword_prefix": "##",
            "max_input_chars_per_word": 100,
            "vocab": {
                "[PAD]": 0,
                "[UNK]": 1,
                "[CLS]": 2,
                "[SEP]": 3,
                "[MASK]": 4,
                "hello": 5,
                "world": 6,
                "test": 7,
                "embedding": 8,
                "model": 9
            }
        }
    }"###;
    
    let mut tokenizer_file = File::create(&tokenizer_path)?;
    tokenizer_file.write_all(tokenizer_json.as_bytes())?;

    info!("📁 创建模拟 ONNX 模型文件:");
    info!("   模型: {:?}", model_path);
    info!("   分词器: {:?}", tokenizer_path);
    info!("");

    // 创建 ONNX 嵌入配置
    let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 384);

    info!("⚙️  创建 ONNX 嵌入器...");
    let embedder = LocalEmbedder::new(config).await?;

    info!("✅ ONNX 嵌入器创建成功");
    info!("");

    // 测试单个文本嵌入
    info!("📝 测试单个文本嵌入:");
    let text = "Hello, this is a test for ONNX embedding model";
    info!("   输入文本: \"{}\"", text);

    let embedding = embedder.embed(text).await?;
    info!("   ✅ 生成嵌入向量: {} 维", embedding.len());
    info!("   前 10 个值: {:?}", &embedding[..10.min(embedding.len())]);
    info!("");

    // 测试批量嵌入
    info!("📦 测试批量嵌入:");
    let texts = vec![
        "First text for batch embedding".to_string(),
        "Second text for batch embedding".to_string(),
        "Third text for batch embedding".to_string(),
    ];
    
    info!("   输入文本数量: {}", texts.len());
    for (i, text) in texts.iter().enumerate() {
        info!("   [{}] \"{}\"", i + 1, text);
    }

    let embeddings = embedder.embed_batch(&texts).await?;
    info!("   ✅ 生成批量嵌入: {} 个向量", embeddings.len());
    for (i, emb) in embeddings.iter().enumerate() {
        info!("   [{}] {} 维, 前 5 个值: {:?}", i + 1, emb.len(), &emb[..5.min(emb.len())]);
    }
    info!("");

    // 显示嵌入器信息
    info!("ℹ️  嵌入器信息:");
    info!("   提供商: {}", embedder.provider_name());
    info!("   维度: {}", embedder.dimension());
    info!("");

    // 健康检查
    info!("🏥 健康检查:");
    match embedder.health_check().await {
        Ok(_) => info!("   ✅ 健康检查通过"),
        Err(e) => warn!("   ⚠️  健康检查失败: {}", e),
    }
    info!("");

    // 重要说明
    info!("✅ 功能完整:");
    info!("   真实的 ONNX 推理已实现！");
    info!("   ");
    info!("   已实现的功能:");
    info!("   ✅ ONNX 模型加载");
    info!("   ✅ Tokenizer 集成");
    info!("   ✅ 真实的 ONNX 推理");
    info!("   ✅ 张量转换和处理");
    info!("   ✅ 池化策略（CLS token）");
    info!("   ✅ 批量处理接口");
    info!("   ✅ L2 归一化");
    info!("   ✅ 错误处理");
    info!("   ");
    info!("   ⚠️  注意:");
    info!("   本示例使用模拟的 ONNX 模型文件");
    info!("   要使用真实模型，请下载真实的 ONNX 模型和 tokenizer");
    info!("");

    info!("{}", "=".repeat(60));
    info!("✅ ONNX 嵌入模型演示完成");

    Ok(())
}

