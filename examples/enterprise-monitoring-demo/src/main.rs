//! 企业级监控和运维系统演示程序
//!
//! 本程序演示 AgentMem 企业级监控和运维系统的完整功能，包括：
//! - 自动备份和恢复机制
//! - 集群部署和负载均衡
//! - 故障转移和自愈系统
//! - 性能调优和建议系统
//! - 容量规划和预测系统

use agent_mem_compat::enterprise_monitoring::{
    BackupConfig, CapacityPlanningConfig, ClusterConfig, ClusterNode, EnterpriseMonitoringConfig,
    EnterpriseMonitoringManager, FailoverConfig, LoadBalancingStrategy, NodeStatus,
    PerformanceTuningConfig,
};
use anyhow::Result;
use chrono::Utc;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动企业级监控和运维系统演示");

    // 创建企业监控配置
    let config = create_monitoring_config();

    // 创建企业监控管理器
    let monitoring_manager = EnterpriseMonitoringManager::new(config).await?;

    info!("✅ 企业监控管理器创建成功");

    // 演示各个功能模块
    demo_backup_management(&monitoring_manager).await?;
    demo_cluster_management(&monitoring_manager).await?;
    demo_performance_tuning(&monitoring_manager).await?;
    demo_capacity_planning(&monitoring_manager).await?;
    demo_system_health_monitoring(&monitoring_manager).await?;

    // 启动监控系统
    info!("🔄 启动企业监控系统");
    monitoring_manager.start_monitoring().await?;

    // 运行一段时间以展示监控功能
    info!("⏱️  运行监控系统 30 秒...");
    tokio::time::sleep(Duration::from_secs(30)).await;

    // 停止监控系统
    info!("⏹️  停止企业监控系统");
    monitoring_manager.stop_monitoring().await?;

    info!("🎉 企业级监控和运维系统演示完成！");
    Ok(())
}

/// 创建监控配置
fn create_monitoring_config() -> EnterpriseMonitoringConfig {
    EnterpriseMonitoringConfig {
        backup: BackupConfig {
            enabled: true,
            backup_directory: "./demo_backups".to_string(),
            backup_interval_hours: 1, // 演示用，设置为1小时
            retention_count: 5,
            enable_incremental: true,
            enable_compression: true,
            enable_encryption: false,
            encryption_key: None,
            remote_backup: None,
        },
        cluster: ClusterConfig {
            enabled: true,
            node_id: "demo-node-1".to_string(),
            nodes: vec![
                ClusterNode {
                    id: "node-1".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 8080,
                    weight: 1.0,
                    status: NodeStatus::Healthy,
                    last_health_check: Some(Utc::now()),
                },
                ClusterNode {
                    id: "node-2".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 8081,
                    weight: 1.0,
                    status: NodeStatus::Healthy,
                    last_health_check: Some(Utc::now()),
                },
            ],
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
            health_check_interval_seconds: 10,
            node_timeout_seconds: 30,
            enable_auto_scaling: true,
            min_nodes: 1,
            max_nodes: 5,
        },
        failover: FailoverConfig::default(),
        performance_tuning: PerformanceTuningConfig {
            enabled: true,
            analysis_interval_seconds: 60,
            tuning_threshold: 0.8,
            enable_cache_optimization: true,
            enable_query_optimization: true,
            enable_memory_optimization: true,
            target_response_time_ms: 10,
            target_throughput_qps: 10000,
        },
        capacity_planning: CapacityPlanningConfig {
            enabled: true,
            monitoring_interval_seconds: 300,
            prediction_window_days: 30,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            enable_auto_scaling_recommendations: true,
            target_resource_utilization: 0.7,
        },
        monitoring_interval_seconds: 30,
        enable_verbose_logging: true,
    }
}

/// 演示备份管理功能
async fn demo_backup_management(manager: &EnterpriseMonitoringManager) -> Result<()> {
    info!("📦 演示备份管理功能");

    // 创建手动备份
    info!("创建手动备份...");
    let backup_result = manager
        .create_manual_backup(Some("demo_backup".to_string()))
        .await?;
    info!("✅ 备份创建成功: {:?}", backup_result);

    // 列出所有备份
    info!("列出所有备份...");
    let backups = manager.list_backups().await?;
    info!("📋 当前备份列表 ({} 个):", backups.len());
    for backup in &backups {
        info!(
            "  - {}: {} ({})",
            backup.backup_id, backup.backup_name, backup.size_bytes
        );
    }

    // 如果有备份，演示恢复功能
    if let Some(backup) = backups.first() {
        info!("演示备份恢复...");
        let restore_result = manager.restore_backup(&backup.backup_id).await?;
        info!("✅ 备份恢复成功: {:?}", restore_result);
    }

    Ok(())
}

/// 演示集群管理功能
async fn demo_cluster_management(manager: &EnterpriseMonitoringManager) -> Result<()> {
    info!("🔗 演示集群管理功能");

    // 获取集群状态
    info!("获取集群状态...");
    let cluster_status = manager.get_cluster_status().await?;
    info!("📊 集群状态: {:?}", cluster_status.status);
    info!(
        "🖥️  活跃节点: {}/{}",
        cluster_status.active_nodes, cluster_status.total_nodes
    );

    // 添加新节点
    info!("添加新集群节点...");
    let new_node = ClusterNode {
        id: "node-3".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8082,
        weight: 1.0,
        status: NodeStatus::Healthy,
        last_health_check: Some(Utc::now()),
    };
    manager.add_cluster_node(new_node).await?;
    info!("✅ 新节点添加成功");

    // 再次获取集群状态
    let updated_status = manager.get_cluster_status().await?;
    info!(
        "📊 更新后集群状态: 活跃节点 {}/{}",
        updated_status.active_nodes, updated_status.total_nodes
    );

    Ok(())
}

/// 演示性能调优功能
async fn demo_performance_tuning(manager: &EnterpriseMonitoringManager) -> Result<()> {
    info!("⚡ 演示性能调优功能");

    // 获取性能建议
    info!("获取性能优化建议...");
    let recommendations = manager.get_performance_recommendations().await?;
    info!("💡 性能建议 ({} 个):", recommendations.len());
    for rec in &recommendations {
        info!("  - {}: {} (优先级: {:?})", rec.id, rec.title, rec.priority);
        info!(
            "    预期性能提升: {:.1}%",
            rec.expected_impact.performance_improvement_percent
        );
    }

    // 应用第一个优化建议
    if let Some(rec) = recommendations.first() {
        info!("应用优化建议: {}", rec.id);
        let result = manager.apply_performance_optimization(&rec.id).await?;
        info!(
            "✅ 优化应用成功: 实际性能提升 {:.1}%",
            result.actual_impact.performance_improvement_percent
        );
    }

    Ok(())
}

/// 演示容量规划功能
async fn demo_capacity_planning(manager: &EnterpriseMonitoringManager) -> Result<()> {
    info!("📈 演示容量规划功能");

    // 获取容量预测
    info!("获取 30 天容量预测...");
    let forecast = manager.get_capacity_forecast(30).await?;
    info!(
        "🔮 容量预测 (准确度: {:.1}%):",
        forecast.forecast_accuracy * 100.0
    );
    info!(
        "  - CPU: {:.1}% -> {:.1}% ({:?})",
        forecast.cpu_forecast.current_usage_percent,
        forecast.cpu_forecast.predicted_usage_percent,
        forecast.cpu_forecast.trend
    );
    info!(
        "  - 内存: {:.1}% -> {:.1}% ({:?})",
        forecast.memory_forecast.current_usage_percent,
        forecast.memory_forecast.predicted_usage_percent,
        forecast.memory_forecast.trend
    );

    // 获取扩容建议
    info!("获取扩容建议...");
    let scaling_recommendations = manager.get_scaling_recommendations().await?;
    info!("📊 扩容建议 ({} 个):", scaling_recommendations.len());
    for rec in &scaling_recommendations {
        info!(
            "  - {:?} {:?}: 扩容 {:.1}x (紧急程度: {:?})",
            rec.scaling_type, rec.resource_type, rec.recommended_scaling_amount, rec.urgency
        );
        info!(
            "    月度成本增加: ${:.2}",
            rec.cost_estimate.monthly_cost_increase
        );
    }

    Ok(())
}

/// 演示系统健康监控功能
async fn demo_system_health_monitoring(manager: &EnterpriseMonitoringManager) -> Result<()> {
    info!("🏥 演示系统健康监控功能");

    // 获取系统健康报告
    info!("生成系统健康报告...");
    let health_report = manager.get_system_health().await?;
    info!("🎯 系统整体状态: {:?}", health_report.overall_status);
    info!("⏰ 系统运行时间: {} 秒", health_report.uptime_seconds);

    // 显示各组件健康状态
    info!("📋 组件健康状态:");
    info!(
        "  - 备份管理: {:?} - {}",
        health_report.backup_health.status, health_report.backup_health.message
    );
    info!(
        "  - 集群管理: {:?} - {}",
        health_report.cluster_health.status, health_report.cluster_health.message
    );
    info!(
        "  - 故障转移: {:?} - {}",
        health_report.failover_health.status, health_report.failover_health.message
    );
    info!(
        "  - 性能调优: {:?} - {}",
        health_report.performance_health.status, health_report.performance_health.message
    );
    info!(
        "  - 容量规划: {:?} - {}",
        health_report.capacity_health.status, health_report.capacity_health.message
    );

    // 显示性能指标
    info!("📊 当前性能指标:");
    let metrics = &health_report.performance_metrics;
    info!("  - CPU 使用率: {:.1}%", metrics.cpu_usage_percent);
    info!("  - 内存使用率: {:.1}%", metrics.memory_usage_percent);
    info!("  - 响应时间: {:.1}ms", metrics.response_time_ms);
    info!("  - 吞吐量: {:.0} QPS", metrics.throughput_qps);
    info!("  - 错误率: {:.3}%", metrics.error_rate_percent);

    // 显示活跃告警
    if !health_report.active_alerts.is_empty() {
        warn!("⚠️  活跃告警 ({} 个):", health_report.active_alerts.len());
        for alert in &health_report.active_alerts {
            warn!(
                "  - {:?}: {} (级别: {:?})",
                alert.alert_type, alert.message, alert.level
            );
        }
    } else {
        info!("✅ 无活跃告警");
    }

    Ok(())
}
