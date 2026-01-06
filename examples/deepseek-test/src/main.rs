use agent_mem_llm::providers::deepseek::DeepSeekProvider;
use anyhow::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🚀 开始 DeepSeek API 测试");

    // 使用提供的 API 密钥
    let api_key = "sk-498fd5f3041f4466a43fa2b9bbbec250";

    match DeepSeekProvider::with_api_key(api_key.to_string()) {
        Ok(provider) => {
            info!("✅ DeepSeek 提供商创建成功");

            // 测试简单文本生成
            info!("📝 测试简单文本生成...");
            match provider.generate_text("你好，请简单介绍一下你自己").await {
                Ok(response) => {
                    info!("✅ 文本生成成功:");
                    println!("Response: {response}");
                }
                Err(e) => {
                    error!("❌ 文本生成失败: {}", e);
                }
            }

            // 测试系统提示
            info!("🎯 测试系统提示...");
            match provider
                .generate_with_system(
                    "你是一个专业的 AI 助手，专门帮助用户理解和使用 AgentMem 记忆系统。",
                    "请解释什么是向量数据库，以及它在 AI 记忆系统中的作用。",
                )
                .await
            {
                Ok(response) => {
                    info!("✅ 系统提示测试成功:");
                    println!("Response: {response}");
                }
                Err(e) => {
                    error!("❌ 系统提示测试失败: {}", e);
                }
            }

            // 测试 JSON 生成
            info!("📊 测试 JSON 生成...");
            #[derive(serde::Deserialize, Debug)]
            struct MemoryAnalysis {
                importance_score: f32,
                memory_type: String,
                keywords: Vec<String>,
                summary: String,
            }

            let json_prompt = r#"
分析以下记忆内容，并返回 JSON 格式的分析结果：
"用户今天学习了 Rust 编程语言的所有权概念，感觉很有挑战性但很有趣。"

请返回包含以下字段的 JSON：
- importance_score: 重要性评分 (0.0-1.0)
- memory_type: 记忆类型 ("episodic", "semantic", "procedural")
- keywords: 关键词数组
- summary: 简短总结
"#;

            match provider.generate_json::<MemoryAnalysis>(json_prompt).await {
                Ok(analysis) => {
                    info!("✅ JSON 生成成功:");
                    println!("Analysis: {analysis:#?}");
                }
                Err(e) => {
                    error!("❌ JSON 生成失败: {}", e);
                }
            }

            info!("🎉 DeepSeek API 测试完成");
        }
        Err(e) => {
            error!("❌ DeepSeek 提供商创建失败: {}", e);
        }
    }

    Ok(())
}
