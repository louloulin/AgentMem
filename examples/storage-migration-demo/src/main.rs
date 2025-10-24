//! 存储迁移演示程序
//!
//! 展示真实的数据迁移功能，包括：
//! - 不同存储后端之间的数据迁移
//! - 批量处理和进度跟踪
//! - 错误恢复和重试机制
//! - 性能监控和统计

use agent_mem_storage::backends::memory::MemoryVectorStore;
use agent_mem_traits::{Result, VectorData, VectorStore, VectorStoreConfig};
use agent_mem_utils::migration::{DataMigrator, MigrationConfig, MigrationStatus, MigrationTools};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("🚀 开始存储迁移演示");

    // 演示 1: 基本迁移功能
    demo_basic_migration().await?;

    // 演示 2: 大规模数据迁移
    demo_large_scale_migration().await?;

    // 演示 3: 迁移工具功能
    demo_migration_tools().await?;

    // 演示 4: 错误处理和恢复
    demo_error_handling().await?;

    info!("✅ 存储迁移演示完成");
    Ok(())
}

/// 演示基本迁移功能
async fn demo_basic_migration() -> Result<()> {
    info!("📦 演示 1: 基本迁移功能");

    // 创建源存储（包含数据）
    let source = create_populated_store("source", 100).await?;

    // 创建目标存储（空）
    let target = create_empty_store("target").await?;

    // 配置迁移参数
    let config = MigrationConfig {
        batch_size: 20,
        clear_target: true,
        validate_data: true,
        ..Default::default()
    };

    // 执行迁移
    let migrator = DataMigrator::new(config);
    info!("开始迁移 100 条记录...");

    let result = migrator.migrate(source.clone(), target.clone()).await?;

    info!("✅ 迁移完成:");
    info!("  - 总记录数: {}", result.total_records);
    info!("  - 成功记录: {}", result.successful_records);
    info!("  - 失败记录: {}", result.failed_records);
    info!("  - 耗时: {:.2}秒", result.duration_seconds);
    info!("  - 平均速度: {:.2} 记录/秒", result.average_speed);

    // 验证迁移结果
    let source_count = source.count_vectors().await?;
    let target_count = target.count_vectors().await?;

    info!("📊 迁移验证:");
    info!("  - 源存储记录数: {}", source_count);
    info!("  - 目标存储记录数: {}", target_count);

    assert_eq!(source_count, target_count);
    info!("✅ 数据完整性验证通过");

    Ok(())
}

/// 演示大规模数据迁移
async fn demo_large_scale_migration() -> Result<()> {
    info!("📦 演示 2: 大规模数据迁移");

    // 创建大规模数据集
    let source = create_populated_store("large_source", 5000).await?;
    let target = create_empty_store("large_target").await?;

    // 配置大规模迁移
    let config = MigrationConfig {
        batch_size: 500,
        clear_target: true,
        validate_data: false, // 大规模迁移时可以关闭验证以提高性能
        retry_count: 3,
        retry_delay_ms: 1000,
        ..Default::default()
    };

    let migrator = DataMigrator::new(config);

    // 预估迁移时间
    let config = MigrationConfig {
        batch_size: 500,
        clear_target: true,
        validate_data: false,
        retry_count: 3,
        retry_delay_ms: 1000,
        ..Default::default()
    };
    let estimated_time = MigrationTools::estimate_migration_time(source.clone(), &config).await?;
    info!("📊 预估迁移时间: {:.2}秒", estimated_time.as_secs_f64());

    // 执行迁移
    info!("开始大规模迁移 5000 条记录...");
    let result = migrator.migrate(source, target).await?;

    info!("✅ 大规模迁移完成:");
    info!("  - 总记录数: {}", result.total_records);
    info!("  - 成功记录: {}", result.successful_records);
    info!("  - 失败记录: {}", result.failed_records);
    info!("  - 实际耗时: {:.2}秒", result.duration_seconds);
    info!("  - 平均速度: {:.2} 记录/秒", result.average_speed);
    info!(
        "  - 预估准确度: {:.1}%",
        (estimated_time.as_secs_f64() / result.duration_seconds) * 100.0
    );

    Ok(())
}

/// 演示迁移工具功能
async fn demo_migration_tools() -> Result<()> {
    info!("📦 演示 3: 迁移工具功能");

    let source = create_populated_store("tools_source", 200).await?;
    let target = create_empty_store("tools_target").await?;

    // 兼容性检查
    info!("🔍 检查存储兼容性...");
    let is_compatible =
        MigrationTools::validate_compatibility(source.clone(), target.clone()).await?;
    info!(
        "兼容性检查结果: {}",
        if is_compatible {
            "✅ 兼容"
        } else {
            "❌ 不兼容"
        }
    );

    // 创建迁移器并监控进度
    let migrator = MigrationTools::create_migrator();

    // 模拟进度监控
    info!("📊 监控迁移进度...");
    let progress = migrator.get_progress().await;
    info!("当前状态: {:?}", progress.status);
    info!(
        "处理进度: {}/{}",
        progress.processed_records, progress.total_records
    );

    // 暂停和恢复功能
    info!("⏸️  测试暂停功能...");
    migrator.pause().await;
    let progress = migrator.get_progress().await;
    assert_eq!(progress.status, MigrationStatus::Preparing);

    info!("▶️  测试恢复功能...");
    migrator.resume().await;
    let progress = migrator.get_progress().await;
    info!("恢复后状态: {:?}", progress.status);

    Ok(())
}

/// 演示错误处理和恢复
async fn demo_error_handling() -> Result<()> {
    info!("📦 演示 4: 错误处理和恢复");

    let source = create_populated_store("error_source", 50).await?;
    let target = create_empty_store("error_target").await?;

    // 配置带重试的迁移
    let config = MigrationConfig {
        batch_size: 10,
        retry_count: 3,
        retry_delay_ms: 500,
        clear_target: true,
        validate_data: true,
        ..Default::default()
    };

    let migrator = DataMigrator::new(config);

    info!("🔄 执行带错误恢复的迁移...");
    let result = migrator.migrate(source, target).await?;

    info!("✅ 错误恢复迁移完成:");
    info!("  - 总记录数: {}", result.total_records);
    info!("  - 成功记录: {}", result.successful_records);
    info!("  - 失败记录: {}", result.failed_records);
    info!("  - 耗时: {:.2}秒", result.duration_seconds);

    if result.failed_records > 0 {
        warn!("⚠️  有 {} 条记录迁移失败", result.failed_records);
    }

    Ok(())
}

/// 创建包含数据的存储
async fn create_populated_store(name: &str, count: usize) -> Result<Arc<MemoryVectorStore>> {
    let mut config = VectorStoreConfig::default();
    config.dimension = Some(128); // 设置为我们测试向量的维度
    let store = Arc::new(MemoryVectorStore::new(config).await?);

    // 生成测试数据
    let mut vectors = Vec::new();
    for i in 0..count {
        let id = format!("{name}_{i}");
        let vector = generate_test_vector(i, 128);
        let mut metadata = HashMap::new();
        metadata.insert("index".to_string(), i.to_string());
        metadata.insert("source".to_string(), name.to_string());
        metadata.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        vectors.push(VectorData {
            id,
            vector,
            metadata,
        });
    }

    // 批量添加数据
    store.add_vectors(vectors).await?;
    info!("📊 创建存储 '{}' 包含 {} 条记录", name, count);

    Ok(store)
}

/// 创建空存储
async fn create_empty_store(name: &str) -> Result<Arc<MemoryVectorStore>> {
    let mut config = VectorStoreConfig::default();
    config.dimension = Some(128); // 设置为我们测试向量的维度
    let store = Arc::new(MemoryVectorStore::new(config).await?);
    info!("📊 创建空存储 '{}'", name);
    Ok(store)
}

/// 生成测试向量
fn generate_test_vector(seed: usize, dim: usize) -> Vec<f32> {
    let mut vector = Vec::with_capacity(dim);
    for i in 0..dim {
        let value =
            ((seed * 31 + i * 17) as f32).sin() * 0.5 + ((seed * 13 + i * 7) as f32).cos() * 0.3;
        vector.push(value);
    }

    // 归一化
    let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vector {
            *v /= norm;
        }
    }

    vector
}
