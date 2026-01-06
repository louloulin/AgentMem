//! 单二进制打包演示
//!
//! 演示如何使用 agent-mem-deployment 创建单二进制部署包

use agent_mem_deployment::config_embed::{ConfigTemplate, EmbeddedConfigManager};
use agent_mem_deployment::embedded::database::EmbeddedDatabaseConfig;
use agent_mem_deployment::embedded::vector_store::{DistanceMetric, EmbeddedVectorStoreConfig};
use agent_mem_deployment::embedded::{EmbeddedConfig, EmbeddedDatabase, EmbeddedVectorStore};
use agent_mem_deployment::packaging::config::{OptimizationLevel, PackageConfig, TargetPlatform};
use anyhow::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("=== AgentMem 单二进制打包演示 ===\n");

    // 1. 演示嵌入式数据库
    demo_embedded_database().await?;

    // 2. 演示嵌入式向量存储
    demo_embedded_vector_store().await?;

    // 3. 演示配置管理
    demo_config_management()?;

    // 4. 演示打包配置
    demo_packaging_config()?;

    info!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示嵌入式数据库
async fn demo_embedded_database() -> Result<()> {
    info!("--- 1. 嵌入式数据库演示 ---");

    // 创建内存数据库配置
    let config = EmbeddedDatabaseConfig::in_memory();
    info!("✓ 创建内存数据库配置");

    // 创建数据库实例
    let db = EmbeddedDatabase::new(config);
    info!("✓ 创建数据库实例");

    // 初始化数据库
    db.initialize().await?;
    info!("✓ 初始化数据库");

    // 获取连接并执行 SQL
    let conn = db.get_connection().await?;
    info!("✓ 获取数据库连接");

    // 创建表
    conn.execute(
        "CREATE TABLE memories (id INTEGER PRIMARY KEY, content TEXT, created_at INTEGER)",
        (),
    )
    .await?;
    info!("✓ 创建 memories 表");

    // 插入数据
    conn.execute(
        "INSERT INTO memories (content, created_at) VALUES ('Hello AgentMem!', 1234567890)",
        (),
    )
    .await?;
    info!("✓ 插入测试数据");

    // 关闭数据库
    db.shutdown().await?;
    info!("✓ 关闭数据库\n");

    Ok(())
}

/// 演示嵌入式向量存储
async fn demo_embedded_vector_store() -> Result<()> {
    info!("--- 2. 嵌入式向量存储演示 ---");

    // 创建向量存储配置
    let config = EmbeddedVectorStoreConfig {
        path: "./data/vectors".into(),
        dimension: 384,
        distance: DistanceMetric::Cosine,
        enable_persistence: false,
        auto_create_dir: true,
        in_memory: true,
        collection_name: "demo_vectors".to_string(),
    };
    info!("✓ 创建向量存储配置（384 维，余弦距离）");

    // 创建向量存储实例
    let store = EmbeddedVectorStore::new(config);
    info!("✓ 创建向量存储实例");

    // 初始化向量存储
    store.initialize().await?;
    info!("✓ 初始化向量存储（LanceDB）");

    // 关闭向量存储
    store.shutdown().await?;
    info!("✓ 关闭向量存储\n");

    Ok(())
}

/// 演示配置管理
fn demo_config_management() -> Result<()> {
    info!("--- 3. 配置管理演示 ---");

    // 列出所有可用的配置模板
    let templates = ConfigTemplate::all();
    info!(
        "✓ 可用配置模板: {:?}",
        templates.iter().map(|t| t.name()).collect::<Vec<_>>()
    );

    // 使用开发环境模板
    let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
    info!("✓ 创建配置管理器（开发环境模板）");

    // 获取配置内容
    let config_content = manager.get_config();
    info!("✓ 获取配置内容（{} 字节）", config_content.len());

    // 验证配置
    let validation_result = manager.validate();
    info!("✓ 配置验证结果: {:?}\n", validation_result.is_ok());

    Ok(())
}

/// 演示打包配置
fn demo_packaging_config() -> Result<()> {
    info!("--- 4. 打包配置演示 ---");

    // 创建打包配置
    let config = PackageConfig {
        target: TargetPlatform::LinuxX64,
        optimization_level: OptimizationLevel::Release,
        enable_lto: true,
        enable_strip: true,
        enable_compression: true,
        compression: agent_mem_deployment::packaging::config::CompressionAlgorithm::Gzip,
        binary_name: "agentmem".to_string(),
        output_dir: "./dist".into(),
        include_config: true,
        include_docs: false,
    };
    info!("✓ 创建打包配置");
    info!("  - 目标平台: {:?}", config.target);
    info!("  - 优化级别: {:?}", config.optimization_level);
    info!("  - 启用 LTO: {}", config.enable_lto);
    info!("  - 启用 Strip: {}", config.enable_strip);

    // 获取二进制文件名
    let binary_name = &config.binary_name;
    info!("✓ 二进制文件名: {}", binary_name);

    // 获取完整二进制文件名（带扩展名）
    let full_name = config.full_binary_name();
    info!("✓ 完整文件名: {}\n", full_name);

    Ok(())
}

/// 演示完整的嵌入式配置
#[allow(dead_code)]
async fn demo_full_embedded_config() -> Result<()> {
    info!("--- 完整嵌入式配置演示 ---");

    // 创建完整配置
    let config = EmbeddedConfig::default()
        .with_vector_dimension(768)
        .with_wal(true)
        .with_persistence(true);
    info!("✓ 创建完整嵌入式配置");

    // 初始化所有组件
    config.initialize_all().await?;
    info!("✓ 初始化所有嵌入式组件");

    Ok(())
}
