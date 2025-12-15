//! LOCOMO基准测试主程序
//!
//! 使用LOCOMO标准评估AgentMem的性能和效果

mod framework;
mod datasets;
mod metrics;
mod test_cases;
mod report;
mod llm_integration;

use anyhow::Result;
use tracing_subscriber;

use framework::LocomoTestFramework;
use report::ReportGenerator;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("locomo_benchmark=debug,agent_mem=info")
        .init();

    println!("🚀 AgentMem LOCOMO基准测试");
    println!("================================");

    // 创建测试框架
    let framework = LocomoTestFramework::new()?;

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
