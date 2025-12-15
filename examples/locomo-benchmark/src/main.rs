//! LOCOMO基准测试主程序
//!
//! 使用LOCOMO标准评估AgentMem的性能和效果

mod datasets;
mod framework;
mod llm_integration;
mod metrics;
mod report;
mod test_cases;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber;

use framework::{LocomoTestFramework, TestConfig};
use llm_integration::LlmConfig;
use report::ReportGenerator;

#[derive(Parser, Debug)]
#[command(name = "locomo-benchmark")]
#[command(about = "Run AgentMem LOCOMO benchmark", long_about = None)]
struct Cli {
    /// 数据集路径，需包含 single_hop/multi_hop/... 等子目录
    #[arg(long, default_value = "data")]
    dataset_path: String,

    /// 是否展示详细日志
    #[arg(long, default_value_t = true)]
    verbose: bool,

    /// LLM提供商（openai/openai_compatible 等），可用环境变量 LOCOMO_LLM_PROVIDER
    #[arg(long)]
    llm_provider: Option<String>,

    /// LLM模型名称，可用环境变量 LOCOMO_LLM_MODEL
    #[arg(long)]
    llm_model: Option<String>,

    /// LLM API Key，默认读取环境变量 OPENAI_API_KEY
    #[arg(long)]
    llm_api_key: Option<String>,

    /// 自定义OpenAI兼容基址，可用环境变量 LOCOMO_LLM_BASE_URL
    #[arg(long)]
    llm_base_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("locomo_benchmark=debug,agent_mem=info")
        .init();

    println!("🚀 AgentMem LOCOMO基准测试");
    println!("================================");

    let args = Cli::parse();

    // CLI 优先，未提供时尝试读取环境变量
    let env_provider = std::env::var("LOCOMO_LLM_PROVIDER").ok();
    let env_model = std::env::var("LOCOMO_LLM_MODEL").ok();
    let env_api_key = std::env::var("OPENAI_API_KEY").ok();
    let env_base_url = std::env::var("LOCOMO_LLM_BASE_URL").ok();

    let llm_provider = args.llm_provider.or(env_provider);
    let llm_model = args.llm_model.or(env_model);
    let llm_api_key = args.llm_api_key.or(env_api_key);
    let llm_base_url = args.llm_base_url.or(env_base_url);

    let llm_config = if llm_provider.is_some()
        || llm_model.is_some()
        || llm_api_key.is_some()
        || llm_base_url.is_some()
    {
        Some(LlmConfig {
            provider: llm_provider.unwrap_or_else(|| "openai".to_string()),
            api_key: llm_api_key,
            model: llm_model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            base_url: llm_base_url,
        })
    } else {
        None
    };

    // 创建测试框架（使用异步版本）
    let framework = LocomoTestFramework::with_config_async(TestConfig {
        dataset_path: args.dataset_path,
        verbose: args.verbose,
        llm_config,
        ..Default::default()
    })
    .await?;

    // 运行所有测试
    println!("\n📊 开始运行LOCOMO测试...\n");
    let results = framework.run_all_tests().await?;

    // 生成报告
    println!("\n📝 生成测试报告...\n");
    let report_generator = ReportGenerator::new();
    report_generator.generate_report(&results).await?;

    println!("\n✅ 测试完成！");
    println!("📄 报告已保存到: results/reports/");

    Ok(())
}
