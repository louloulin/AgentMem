//! 多租户隔离系统演示
//!
//! 展示 AgentMem 6.0 的企业级多租户隔离功能，包括：
//! - 租户创建和管理
//! - 资源限制和隔离
//! - 计费追踪
//! - 数据分区

use agent_mem_core::tenant::{
    MultiTenantManager, ResourceLimits, ResourceOperation, SecurityPolicy, TenantId,
};
use anyhow::Result;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🏢 启动 AgentMem 6.0 多租户隔离系统演示");

    // 创建多租户管理器
    let manager = MultiTenantManager::new();

    // 演示1: 创建多个租户
    info!("📋 演示1: 创建多个租户");
    let tenant_ids = create_demo_tenants(&manager).await?;

    // 演示2: 测试资源限制
    info!("🔒 演示2: 测试资源限制");
    test_resource_limits(&manager, &tenant_ids).await?;

    // 演示3: 测试数据分区
    info!("🗂️  演示3: 测试数据分区");
    test_data_partitioning(&manager, &tenant_ids).await?;

    // 演示4: 测试计费追踪
    info!("💰 演示4: 测试计费追踪");
    test_billing_tracking(&manager, &tenant_ids).await?;

    // 演示5: 获取租户统计信息
    info!("📊 演示5: 获取租户统计信息");
    show_tenant_statistics(&manager, &tenant_ids).await?;

    info!("🎉 多租户隔离系统演示完成！");
    Ok(())
}

/// 创建演示租户
async fn create_demo_tenants(manager: &MultiTenantManager) -> Result<Vec<TenantId>> {
    let mut tenant_ids = Vec::new();

    // 创建企业租户 - 高资源限制
    let enterprise_limits = ResourceLimits {
        max_memories: 100_000,
        max_storage_bytes: 10_000_000_000, // 10GB
        max_concurrent_requests: 1000,
        max_requests_per_second: 10000,
        max_embedding_dimensions: 3072,
        max_batch_size: 1000,
    };

    let enterprise_policy = SecurityPolicy {
        encryption_enabled: true,
        audit_logging_enabled: true,
        access_control_enabled: true,
        data_retention_days: 2555, // 7年
        cross_tenant_access_allowed: false,
        allowed_ip_ranges: vec!["10.0.0.0/8".to_string()],
    };

    let enterprise_id = manager
        .create_tenant(
            "Enterprise Corp".to_string(),
            Some(enterprise_limits),
            Some(enterprise_policy),
        )
        .await?;

    info!("✅ 创建企业租户: {}", enterprise_id.as_str());
    tenant_ids.push(enterprise_id);

    // 创建初创公司租户 - 中等资源限制
    let startup_limits = ResourceLimits {
        max_memories: 10_000,
        max_storage_bytes: 1_000_000_000, // 1GB
        max_concurrent_requests: 100,
        max_requests_per_second: 1000,
        max_embedding_dimensions: 1536,
        max_batch_size: 100,
    };

    let startup_id = manager
        .create_tenant("Startup Inc".to_string(), Some(startup_limits), None)
        .await?;

    info!("✅ 创建初创公司租户: {}", startup_id.as_str());
    tenant_ids.push(startup_id);

    // 创建个人开发者租户 - 低资源限制
    let developer_limits = ResourceLimits {
        max_memories: 1_000,
        max_storage_bytes: 100_000_000, // 100MB
        max_concurrent_requests: 10,
        max_requests_per_second: 100,
        max_embedding_dimensions: 768,
        max_batch_size: 10,
    };

    let developer_id = manager
        .create_tenant("Developer John".to_string(), Some(developer_limits), None)
        .await?;

    info!("✅ 创建个人开发者租户: {}", developer_id.as_str());
    tenant_ids.push(developer_id);

    Ok(tenant_ids)
}

/// 测试资源限制
async fn test_resource_limits(manager: &MultiTenantManager, tenant_ids: &[TenantId]) -> Result<()> {
    for tenant_id in tenant_ids {
        info!("🔍 测试租户 {} 的资源限制", tenant_id.as_str());

        // 测试正常操作
        let normal_operation = ResourceOperation::AddMemory { size: 1024 }; // 1KB
        match manager
            .validate_operation(tenant_id, normal_operation)
            .await
        {
            Ok(()) => info!("  ✅ 正常操作通过"),
            Err(e) => error!("  ❌ 正常操作失败: {}", e),
        }

        // 测试大内存操作 (可能触发限制)
        let large_operation = ResourceOperation::AddMemory { size: 100_000_000 }; // 100MB
        match manager.validate_operation(tenant_id, large_operation).await {
            Ok(()) => info!("  ✅ 大内存操作通过"),
            Err(e) => info!("  ⚠️  大内存操作被限制: {}", e),
        }

        // 模拟多个并发请求
        for i in 0..5 {
            let request_op = ResourceOperation::StartRequest;
            match manager.validate_operation(tenant_id, request_op).await {
                Ok(()) => info!("  ✅ 请求 {} 通过", i + 1),
                Err(e) => info!("  ⚠️  请求 {} 被限制: {}", i + 1, e),
            }
        }
    }

    Ok(())
}

/// 测试数据分区
async fn test_data_partitioning(
    manager: &MultiTenantManager,
    tenant_ids: &[TenantId],
) -> Result<()> {
    for tenant_id in tenant_ids {
        let partition_key = manager.get_partition_key(tenant_id);
        info!(
            "🗂️  租户 {} 的数据分区键: {}",
            tenant_id.as_str(),
            partition_key
        );
    }

    // 验证不同租户有不同的分区键
    let keys: Vec<String> = tenant_ids
        .iter()
        .map(|id| manager.get_partition_key(id))
        .collect();

    let unique_keys: std::collections::HashSet<_> = keys.iter().collect();

    if unique_keys.len() == keys.len() {
        info!("✅ 所有租户都有唯一的数据分区键");
    } else {
        error!("❌ 发现重复的数据分区键");
    }

    Ok(())
}

/// 测试计费追踪
async fn test_billing_tracking(
    manager: &MultiTenantManager,
    tenant_ids: &[TenantId],
) -> Result<()> {
    for tenant_id in tenant_ids {
        info!("💰 为租户 {} 记录计费事件", tenant_id.as_str());

        // 记录不同类型的计费事件
        manager
            .record_billing(tenant_id, "memory_storage", 1000)
            .await?; // 1000MB存储
        manager
            .record_billing(tenant_id, "api_request", 5000)
            .await?; // 5000次API请求
        manager
            .record_billing(tenant_id, "embedding_generation", 2000)
            .await?; // 2000次嵌入生成
        manager
            .record_billing(tenant_id, "search_operation", 1500)
            .await?; // 1500次搜索

        info!("  ✅ 计费事件记录完成");
    }

    Ok(())
}

/// 显示租户统计信息
async fn show_tenant_statistics(
    manager: &MultiTenantManager,
    tenant_ids: &[TenantId],
) -> Result<()> {
    for tenant_id in tenant_ids {
        match manager.get_tenant_stats(tenant_id).await {
            Ok(stats) => {
                info!("📊 租户统计信息: {}", stats.name);
                info!("  - 租户ID: {}", stats.tenant_id.as_str());
                info!(
                    "  - 状态: {}",
                    if stats.is_active { "激活" } else { "停用" }
                );
                info!(
                    "  - 内存数量: {} / {}",
                    stats.resource_usage.memory_count, stats.resource_limits.max_memories
                );
                info!(
                    "  - 存储使用: {} / {} 字节",
                    stats.resource_usage.storage_bytes, stats.resource_limits.max_storage_bytes
                );
                info!(
                    "  - 并发请求: {} / {}",
                    stats.resource_usage.concurrent_requests,
                    stats.resource_limits.max_concurrent_requests
                );
                info!(
                    "  - 总费用: {} 分 (${:.2})",
                    stats.total_cost_cents,
                    stats.total_cost_cents as f64 / 100.0
                );
                info!(
                    "  - 创建时间: {}",
                    chrono::DateTime::from_timestamp(stats.created_at, 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d %H:%M:%S")
                );
            }
            Err(e) => error!("❌ 获取租户 {} 统计信息失败: {}", tenant_id.as_str(), e),
        }
        println!();
    }

    Ok(())
}
