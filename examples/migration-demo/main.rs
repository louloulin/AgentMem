//! 数据迁移工具演示
//!
//! 演示如何使用 AgentMem 的数据迁移工具在不同存储后端之间迁移数据

use agent_mem_storage::VectorStoreFactory;
use agent_mem_traits::{VectorData, VectorStoreConfig};
use agent_mem_utils::{DataMigrator, MigrationConfig, MigrationStatus, MigrationTools};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("AgentMem 数据迁移工具演示");

    // 创建源存储（内存存储，包含一些测试数据）
    let source_config = VectorStoreConfig {
        provider: "memory".to_string(),
        dimension: Some(128),
        ..Default::default()
    };
    let source = VectorStoreFactory::create_vector_store(&source_config).await?;

    // 添加一些测试数据到源存储
    info!("向源存储添加测试数据...");
    let mut test_vectors = Vec::new();
    for i in 0..1000 {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), format!("category_{}", i % 10));
        metadata.insert("index".to_string(), i.to_string());
        metadata.insert("description".to_string(), format!("Test vector {}", i));

        test_vectors.push(VectorData {
            id: format!("test_vector_{}", i),
            vector: (0..128).map(|j| (i + j) as f32 / 1000.0).collect(),
            metadata,
        });
    }

    // 批量添加数据
    let batch_size = 100;
    for chunk in test_vectors.chunks(batch_size) {
        source.add_vectors(chunk.to_vec()).await?;
    }

    let source_count = source.count_vectors().await?;
    info!("源存储包含 {} 条记录", source_count);

    // 创建目标存储（另一个内存存储）
    let target_config = VectorStoreConfig {
        provider: "memory".to_string(),
        dimension: Some(128),
        ..Default::default()
    };
    let target = VectorStoreFactory::create_vector_store(&target_config).await?;

    // 验证存储兼容性
    info!("验证存储兼容性...");
    let is_compatible =
        MigrationTools::validate_compatibility(source.clone(), target.clone()).await?;
    if !is_compatible {
        error!("存储后端不兼容，无法进行迁移");
        return Ok(());
    }
    info!("存储兼容性验证通过");

    // 估算迁移时间
    let migration_config = MigrationConfig {
        batch_size: 50,
        concurrency: 2,
        clear_target: true,
        validate_data: true,
        ..Default::default()
    };

    let estimated_time =
        MigrationTools::estimate_migration_time(source.clone(), &migration_config).await?;
    info!("预计迁移时间: {:.2} 秒", estimated_time.as_secs_f64());

    // 创建迁移器
    let migrator = DataMigrator::new(migration_config);

    // 启动迁移任务
    info!("开始数据迁移...");
    let migrator_clone = Arc::new(migrator);
    let source_clone = source.clone();
    let target_clone = target.clone();
    let migrator_for_task = migrator_clone.clone();

    // 在后台运行迁移
    let migration_task =
        tokio::spawn(async move { migrator_for_task.migrate(source_clone, target_clone).await });

    // 监控迁移进度
    let migrator_for_monitor = migrator_clone.clone();
    let monitor_task = tokio::spawn(async move {
        loop {
            let progress = migrator_for_monitor.get_progress().await;

            match progress.status {
                MigrationStatus::Running => {
                    let percentage = if progress.total_records > 0 {
                        (progress.processed_records as f64 / progress.total_records as f64) * 100.0
                    } else {
                        0.0
                    };

                    info!(
                        "迁移进度: {:.1}% ({}/{}) - 批次 {}/{}",
                        percentage,
                        progress.processed_records,
                        progress.total_records,
                        progress.current_batch,
                        progress.total_batches
                    );

                    if let Some(eta) = progress.estimated_completion {
                        let remaining = eta.signed_duration_since(chrono::Utc::now());
                        if remaining.num_seconds() > 0 {
                            info!("预计剩余时间: {} 秒", remaining.num_seconds());
                        }
                    }
                }
                MigrationStatus::Completed => {
                    info!("迁移已完成");
                    break;
                }
                MigrationStatus::Failed => {
                    error!("迁移失败");
                    for error in &progress.errors {
                        error!("错误: {}", error);
                    }
                    break;
                }
                MigrationStatus::Cancelled => {
                    warn!("迁移已取消");
                    break;
                }
                _ => {}
            }

            sleep(Duration::from_millis(500)).await;
        }
    });

    // 等待迁移完成
    let migration_result = migration_task.await??;
    monitor_task.abort();

    // 显示迁移结果
    info!("=== 迁移结果 ===");
    info!("成功: {}", migration_result.success);
    info!("总记录数: {}", migration_result.total_records);
    info!("成功迁移: {}", migration_result.successful_records);
    info!("失败记录: {}", migration_result.failed_records);
    info!("耗时: {:.2} 秒", migration_result.duration_seconds);
    info!("平均速度: {:.2} 记录/秒", migration_result.average_speed);

    if !migration_result.errors.is_empty() {
        warn!("迁移过程中的错误:");
        for error in &migration_result.errors {
            warn!("  - {}", error);
        }
    }

    // 验证迁移结果
    let target_count = target.count_vectors().await?;
    info!("目标存储现在包含 {} 条记录", target_count);

    if target_count == source_count {
        info!("✅ 迁移验证成功：记录数量匹配");
    } else {
        warn!(
            "⚠️  迁移验证警告：记录数量不匹配 (源: {}, 目标: {})",
            source_count, target_count
        );
    }

    // 随机验证几条记录的内容
    info!("验证数据完整性...");
    for i in [0, 100, 500, 999] {
        let source_id = format!("test_vector_{}", i);
        let source_vector = source.get_vector(&source_id).await?;
        let target_vector = target.get_vector(&source_id).await?;

        match (source_vector, target_vector) {
            (Some(src), Some(tgt)) => {
                if src.vector == tgt.vector && src.metadata == tgt.metadata {
                    info!("✅ 记录 {} 验证通过", source_id);
                } else {
                    warn!("⚠️  记录 {} 数据不匹配", source_id);
                }
            }
            (Some(_), None) => {
                warn!("⚠️  记录 {} 在目标存储中缺失", source_id);
            }
            (None, Some(_)) => {
                warn!("⚠️  记录 {} 在源存储中缺失", source_id);
            }
            (None, None) => {
                warn!("⚠️  记录 {} 在两个存储中都缺失", source_id);
            }
        }
    }

    info!("数据迁移演示完成");
    Ok(())
}
