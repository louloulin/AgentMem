//! AgentMem 综合压测工具
//!
//! 全面压测 AgentMem 系统，包括：
//! - 记忆构建压测
//! - 记忆检索压测
//! - 并发操作压测
//! - 图推理压测
//! - 智能处理压测
//! - 缓存性能压测
//! - 批量操作压测
//! - 长时间稳定性测试

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Emoji};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;
use tracing::{info, warn};

mod config;
mod monitor;
mod real_config;
mod report;
mod scenarios;
mod stats;

use config::StressTestConfig;
use monitor::SystemMonitor;
use real_config::{RealStressTestConfig, RealStressTestEnv};
use report::ReportGenerator;
use scenarios::*;
use stats::StressTestStats;

static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static CHART: Emoji<'_, '_> = Emoji("📊 ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅ ", "");
static WARN: Emoji<'_, '_> = Emoji("⚠️  ", "");
static FIRE: Emoji<'_, '_> = Emoji("🔥 ", "");

#[derive(Parser)]
#[command(name = "stress-test")]
#[command(about = "AgentMem 综合压测工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 配置文件路径
    #[arg(short, long, default_value = "stress-test-config.json")]
    config: String,

    /// 输出目录
    #[arg(short, long, default_value = "stress-test-results")]
    output: String,

    /// 详细输出
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行所有压测场景
    All,

    /// 场景 1: 记忆构建压测（真实实现）
    MemoryCreation {
        /// 并发数
        #[arg(short, long, default_value = "100")]
        concurrency: usize,

        /// 总记忆数
        #[arg(short, long, default_value = "10000")]
        total: usize,

        /// 使用真实 SDK（默认启用）
        #[arg(long, default_value = "true")]
        real: bool,
    },

    /// 场景 2: 记忆检索压测（真实实现）
    MemoryRetrieval {
        /// 数据集大小
        #[arg(short, long, default_value = "100000")]
        dataset_size: usize,

        /// 查询并发数
        #[arg(short, long, default_value = "100")]
        concurrency: usize,

        /// 使用真实 SDK（默认启用）
        #[arg(long, default_value = "true")]
        real: bool,
    },

    /// 场景 3: 并发操作压测
    ConcurrentOps {
        /// 并发用户数
        #[arg(short, long, default_value = "1000")]
        users: usize,

        /// 持续时间（秒）
        #[arg(short, long, default_value = "300")]
        duration: u64,
    },

    /// 场景 4: 图推理压测
    GraphReasoning {
        /// 图节点数
        #[arg(short, long, default_value = "10000")]
        nodes: usize,

        /// 图边数
        #[arg(short, long, default_value = "50000")]
        edges: usize,
    },

    /// 场景 5: 智能处理压测
    IntelligenceProcessing {
        /// 并发请求数
        #[arg(short, long, default_value = "10")]
        concurrency: usize,
    },

    /// 场景 6: 缓存性能压测
    CachePerformance {
        /// 缓存大小（MB）
        #[arg(short, long, default_value = "500")]
        cache_size_mb: usize,
    },

    /// 场景 7: 批量操作压测（真实实现）
    BatchOperations {
        /// 批量大小
        #[arg(short, long, default_value = "100")]
        batch_size: usize,

        /// 使用真实 SDK（默认启用）
        #[arg(long, default_value = "true")]
        real: bool,
    },

    /// 场景 8: 长时间稳定性测试
    StabilityTest {
        /// 运行时间（小时）
        #[arg(short, long, default_value = "24")]
        hours: u64,
    },

    /// 生成压测报告
    Report {
        /// 结果目录
        #[arg(short, long)]
        results_dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    // 加载配置
    let config = StressTestConfig::load(&cli.config)?;

    // 创建输出目录
    std::fs::create_dir_all(&cli.output)?;

    println!(
        "{} {}",
        ROCKET,
        style("AgentMem 综合压测工具").bold().cyan()
    );
    println!();

    // 初始化真实压测环境（如果需要）
    let real_env = match &cli.command {
        Commands::MemoryCreation { real, .. }
        | Commands::MemoryRetrieval { real, .. }
        | Commands::BatchOperations { real, .. }
            if *real =>
        {
            info!("🔧 初始化真实压测环境...");
            let real_config = RealStressTestConfig::default();
            Some(RealStressTestEnv::new(real_config).await?)
        }
        _ => None,
    };

    match cli.command {
        Commands::All => run_all_scenarios(&config, &cli.output).await?,
        Commands::MemoryCreation {
            concurrency,
            total,
            real,
        } => {
            if real {
                if let Some(env) = &real_env {
                    run_memory_creation_test_real(env, concurrency, total, &cli.output).await?
                }
            } else {
                run_memory_creation_test(concurrency, total, &cli.output).await?
            }
        }
        Commands::MemoryRetrieval {
            dataset_size,
            concurrency,
            real,
        } => {
            if real {
                if let Some(env) = &real_env {
                    run_memory_retrieval_test_real(env, dataset_size, concurrency, &cli.output)
                        .await?
                }
            } else {
                run_memory_retrieval_test(dataset_size, concurrency, &cli.output).await?
            }
        }
        Commands::ConcurrentOps { users, duration } => {
            run_concurrent_ops_test(users, duration, &cli.output).await?
        }
        Commands::GraphReasoning { nodes, edges } => {
            run_graph_reasoning_test(nodes, edges, &cli.output).await?
        }
        Commands::IntelligenceProcessing { concurrency } => {
            run_intelligence_processing_test(concurrency, &cli.output).await?
        }
        Commands::CachePerformance { cache_size_mb } => {
            run_cache_performance_test(cache_size_mb, &cli.output).await?
        }
        Commands::BatchOperations { batch_size, real } => {
            if real {
                if let Some(env) = &real_env {
                    run_batch_operations_test_real(env, batch_size, &cli.output).await?
                }
            } else {
                run_batch_operations_test(batch_size, &cli.output).await?
            }
        }
        Commands::StabilityTest { hours } => run_stability_test(hours, &cli.output).await?,
        Commands::Report { results_dir } => generate_report(&results_dir, &cli.output).await?,
    }

    // 清理真实环境
    if let Some(env) = real_env {
        info!("🧹 清理测试数据...");
        if let Err(e) = env.cleanup().await {
            warn!("清理失败: {}", e);
        }
    }

    println!();
    println!("{} {}", CHECK, style("压测完成！").bold().green());

    Ok(())
}

/// 运行所有压测场景
async fn run_all_scenarios(config: &StressTestConfig, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("运行所有压测场景").bold().yellow());
    println!();

    let multi_progress = MultiProgress::new();
    let mut all_stats = Vec::new();

    // 场景 1: 记忆构建
    println!("{} 场景 1: 记忆构建压测", CHART);
    let stats = memory_creation::run_test(
        config.memory_creation.concurrency,
        config.memory_creation.total_memories,
        &multi_progress,
    )
    .await?;
    all_stats.push(("memory_creation", stats));

    // 场景 2: 记忆检索
    println!("{} 场景 2: 记忆检索压测", CHART);
    let stats = memory_retrieval::run_test(
        config.memory_retrieval.dataset_size,
        config.memory_retrieval.concurrency,
        &multi_progress,
    )
    .await?;
    all_stats.push(("memory_retrieval", stats));

    // 场景 3: 并发操作
    println!("{} 场景 3: 并发操作压测", CHART);
    let stats = concurrent_ops::run_test(
        config.concurrent_ops.concurrent_users,
        config.concurrent_ops.duration_seconds,
        &multi_progress,
    )
    .await?;
    all_stats.push(("concurrent_ops", stats));

    // 场景 4: 图推理
    println!("{} 场景 4: 图推理压测", CHART);
    let stats = graph_reasoning::run_test(
        config.graph_reasoning.nodes,
        config.graph_reasoning.edges,
        &multi_progress,
    )
    .await?;
    all_stats.push(("graph_reasoning", stats));

    // 场景 5: 智能处理
    println!("{} 场景 5: 智能处理压测", CHART);
    let stats = intelligence_processing::run_test(
        config.intelligence_processing.concurrency,
        &multi_progress,
    )
    .await?;
    all_stats.push(("intelligence_processing", stats));

    // 场景 6: 缓存性能
    println!("{} 场景 6: 缓存性能压测", CHART);
    let stats =
        cache_performance::run_test(config.cache_performance.cache_size_mb, &multi_progress)
            .await?;
    all_stats.push(("cache_performance", stats));

    // 场景 7: 批量操作
    println!("{} 场景 7: 批量操作压测", CHART);
    let stats =
        batch_operations::run_test(config.batch_operations.batch_size, &multi_progress).await?;
    all_stats.push(("batch_operations", stats));

    // 生成综合报告
    let report_gen = ReportGenerator::new(output_dir);
    report_gen.generate_comprehensive_report(&all_stats).await?;

    println!();
    println!("{} 所有场景压测完成", CHECK);
    println!("报告已生成: {}/comprehensive-report.html", output_dir);

    Ok(())
}

/// 运行记忆构建压测（Mock 版本）
async fn run_memory_creation_test(
    concurrency: usize,
    total: usize,
    output_dir: &str,
) -> Result<()> {
    println!("{} {}", FIRE, style("记忆构建压测 (Mock)").bold().yellow());
    println!("  并发数: {}", concurrency);
    println!("  总记忆数: {}", total);
    println!();

    let multi_progress = MultiProgress::new();
    let stats = memory_creation::run_test(concurrency, total, &multi_progress).await?;

    // 保存结果
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("memory_creation_mock", &stats)
        .await?;

    // 打印摘要
    print_stats_summary(&stats);

    Ok(())
}

/// 运行记忆构建压测（真实版本）
async fn run_memory_creation_test_real(
    env: &RealStressTestEnv,
    concurrency: usize,
    total: usize,
    output_dir: &str,
) -> Result<()> {
    println!(
        "{} {}",
        FIRE,
        style("记忆构建压测 (真实 SDK)").bold().green()
    );
    println!("  并发数: {}", concurrency);
    println!("  总记忆数: {}", total);
    println!("  数据库: PostgreSQL");
    println!();

    let multi_progress = MultiProgress::new();
    let stats = memory_creation::run_test_real(env, concurrency, total, &multi_progress).await?;

    // 保存结果
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("memory_creation_real", &stats)
        .await?;

    // 打印摘要
    print_stats_summary(&stats);

    // 打印数据库统计
    if let Ok(db_stats) = env.get_db_stats().await {
        println!();
        println!("📊 数据库统计:");
        println!("  记忆总数: {}", db_stats.memory_count);
        println!("  向量总数: {}", db_stats.vector_count);
    }

    Ok(())
}

// 其他场景的运行函数
async fn run_memory_retrieval_test(
    dataset_size: usize,
    concurrency: usize,
    output_dir: &str,
) -> Result<()> {
    println!("{} {}", FIRE, style("记忆检索压测 (Mock)").bold().yellow());
    let multi_progress = MultiProgress::new();
    let stats = memory_retrieval::run_test(dataset_size, concurrency, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("memory_retrieval_mock", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_memory_retrieval_test_real(
    env: &RealStressTestEnv,
    dataset_size: usize,
    concurrency: usize,
    output_dir: &str,
) -> Result<()> {
    println!(
        "{} {}",
        FIRE,
        style("记忆检索压测 (真实 SDK)").bold().green()
    );
    let multi_progress = MultiProgress::new();
    let stats =
        memory_retrieval::run_test_real(env, dataset_size, concurrency, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("memory_retrieval_real", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_concurrent_ops_test(users: usize, duration: u64, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("并发操作压测").bold().yellow());
    let multi_progress = MultiProgress::new();
    let stats = concurrent_ops::run_test(users, duration, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("concurrent_ops", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_graph_reasoning_test(nodes: usize, edges: usize, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("图推理压测").bold().yellow());
    let multi_progress = MultiProgress::new();
    let stats = graph_reasoning::run_test(nodes, edges, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("graph_reasoning", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_intelligence_processing_test(concurrency: usize, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("智能处理压测").bold().yellow());
    let multi_progress = MultiProgress::new();
    let stats = intelligence_processing::run_test(concurrency, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("intelligence_processing", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_cache_performance_test(cache_size_mb: usize, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("缓存性能压测").bold().yellow());
    let multi_progress = MultiProgress::new();
    let stats = cache_performance::run_test(cache_size_mb, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("cache_performance", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_batch_operations_test(batch_size: usize, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("批量操作压测 (Mock)").bold().yellow());
    let multi_progress = MultiProgress::new();
    let stats = batch_operations::run_test(batch_size, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("batch_operations_mock", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_batch_operations_test_real(
    env: &RealStressTestEnv,
    batch_size: usize,
    output_dir: &str,
) -> Result<()> {
    println!(
        "{} {}",
        FIRE,
        style("批量操作压测 (真实 SDK)").bold().green()
    );
    let multi_progress = MultiProgress::new();
    let stats = batch_operations::run_test_real(env, batch_size, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen
        .save_scenario_stats("batch_operations_real", &stats)
        .await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn run_stability_test(hours: u64, output_dir: &str) -> Result<()> {
    println!("{} {}", FIRE, style("长时间稳定性测试").bold().yellow());
    println!("  运行时间: {} 小时", hours);
    println!();

    warn!("稳定性测试需要长时间运行，请确保系统资源充足");

    let multi_progress = MultiProgress::new();
    let stats = stability::run_test(hours, &multi_progress).await?;
    let report_gen = ReportGenerator::new(output_dir);
    report_gen.save_scenario_stats("stability", &stats).await?;
    print_stats_summary(&stats);
    Ok(())
}

async fn generate_report(results_dir: &str, output_dir: &str) -> Result<()> {
    println!("{} {}", CHART, style("生成压测报告").bold().yellow());

    let report_gen = ReportGenerator::new(output_dir);
    report_gen.generate_from_directory(results_dir).await?;

    println!("{} 报告已生成: {}/report.html", CHECK, output_dir);
    Ok(())
}

fn print_stats_summary(stats: &StressTestStats) {
    println!();
    println!("{}", style("=== 压测结果摘要 ===").bold().cyan());
    println!("总操作数: {}", stats.total_operations);
    println!("成功操作: {}", stats.successful_operations);
    println!("失败操作: {}", stats.failed_operations);
    println!("吞吐量: {:.2} ops/sec", stats.throughput);
    println!("P50 延迟: {:.2} ms", stats.latency_p50);
    println!("P95 延迟: {:.2} ms", stats.latency_p95);
    println!("P99 延迟: {:.2} ms", stats.latency_p99);
    println!("平均 CPU: {:.2}%", stats.avg_cpu_usage);
    println!("峰值内存: {:.2} MB", stats.peak_memory_mb);
    println!();
}
