use agent_mem_embeddings::EmbeddingFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("🧪 测试 FastEmbed 初始化\n");

    println!("尝试创建 FastEmbed embedder...");
    match EmbeddingFactory::create_default().await {
        Ok(embedder) => {
            println!("✅ FastEmbed 创建成功！");
            println!("   - Provider: {}", embedder.provider_name());
            println!("   - Model: {}", embedder.model_name());
            println!("   - Dimension: {}", embedder.dimension());

            // 测试嵌入
            println!("\n测试嵌入生成...");
            match embedder.embed("Hello, world!").await {
                Ok(embedding) => {
                    println!("✅ 嵌入生成成功！维度: {}", embedding.len());
                }
                Err(e) => {
                    println!("❌ 嵌入生成失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ FastEmbed 创建失败: {}", e);
            println!("\n错误详情: {:?}", e);
        }
    }

    Ok(())
}

