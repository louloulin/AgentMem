//! 多智能体协作记忆系统演示
//!
//! 展示智能体间的记忆共享、协作学习和知识传播功能

use agent_mem_core::collaboration::{
    AccessType, AgentPermissionLevel, CollaborationConfig, CollaborationOperation,
    CollaborativeMemorySystem, ConflictResolution, ConflictResolutionStrategy, ConflictingVersion,
    KnowledgeItem, KnowledgeType, ResolutionType,
};
use agent_mem_traits::{MemoryItem, MemoryType, Session};
use chrono::Utc;
use std::collections::HashMap;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🤖 启动多智能体协作记忆系统演示\n");

    // 演示 1: 记忆共享
    demo_memory_sharing().await?;

    // 演示 2: 权限管理
    demo_permission_management().await?;

    // 演示 3: 知识传播
    demo_knowledge_propagation().await?;

    // 演示 4: 冲突解决
    demo_conflict_resolution().await?;

    // 演示 5: 系统统计
    demo_system_statistics().await?;

    println!("✅ 所有多智能体协作演示完成！\n");
    println!("🎉 多智能体协作记忆系统特点:");
    println!("  - 🔄 智能体间记忆共享：支持细粒度权限控制");
    println!("  - 🛡️ 企业级权限管理：基于角色和策略的访问控制");
    println!("  - 🧠 智能知识传播：支持知识衰减和订阅机制");
    println!("  - ⚖️ 多策略冲突解决：支持投票、重要性、时间等多种策略");
    println!("  - 📊 全面协作统计：实时监控协作效果和系统性能");

    Ok(())
}

/// 演示 1: 记忆共享
async fn demo_memory_sharing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 演示 1: 智能体间记忆共享");
    info!("Testing memory sharing between agents");

    let config = CollaborationConfig::default();
    let system = CollaborativeMemorySystem::new(config);

    // 创建测试记忆
    let memory = create_test_memory(
        "shared_memory_1".to_string(),
        "这是一个重要的项目文档，包含了关键的技术规范和实施细节。".to_string(),
        0.8,
    );

    // 智能体 A 添加共享记忆
    let initial_permissions = HashMap::from([
        ("agent_b".to_string(), AgentPermissionLevel::ReadOnly),
        ("agent_c".to_string(), AgentPermissionLevel::ReadWrite),
    ]);

    system
        .shared_memory_pool()
        .add_shared_memory(memory.clone(), "agent_a".to_string(), initial_permissions)
        .await?;

    // 执行共享操作
    let share_operation = CollaborationOperation::ShareMemory {
        memory_id: "shared_memory_1".to_string(),
        target_agents: vec!["agent_b".to_string(), "agent_c".to_string()],
        permission_level: AgentPermissionLevel::ReadWrite,
    };

    let result = system.execute_operation(share_operation).await?;
    println!("  📤 记忆共享结果: {result:?}");

    // 智能体 B 请求访问
    let access_operation = CollaborationOperation::RequestAccess {
        memory_id: "shared_memory_1".to_string(),
        requesting_agent: "agent_b".to_string(),
        access_type: AccessType::Read,
    };

    let access_result = system.execute_operation(access_operation).await?;
    println!("  🔍 访问请求结果: {access_result:?}");

    println!("  ✅ 记忆共享演示完成\n");
    Ok(())
}

/// 演示 2: 权限管理
async fn demo_permission_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 演示 2: 企业级权限管理");
    info!("Testing permission management system");

    let config = CollaborationConfig::default();
    let system = CollaborativeMemorySystem::new(config);

    // 设置不同智能体的权限级别
    system
        .permission_manager()
        .set_agent_permission("admin_agent".to_string(), AgentPermissionLevel::SuperAdmin)
        .await?;

    system
        .permission_manager()
        .set_agent_permission("regular_agent".to_string(), AgentPermissionLevel::ReadOnly)
        .await?;

    // 测试不同权限级别的访问
    let test_cases = vec![
        ("admin_agent", AccessType::Delete, "应该允许"),
        ("regular_agent", AccessType::Delete, "应该拒绝"),
        ("regular_agent", AccessType::Read, "应该允许"),
    ];

    for (agent, access_type, expected) in test_cases {
        let operation = CollaborationOperation::RequestAccess {
            memory_id: "test_memory".to_string(),
            requesting_agent: agent.to_string(),
            access_type,
        };

        let result = system.execute_operation(operation).await?;
        println!("  🔐 权限测试 - {agent}: {result:?} ({expected})");
    }

    println!("  ✅ 权限管理演示完成\n");
    Ok(())
}

/// 演示 3: 知识传播
async fn demo_knowledge_propagation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 演示 3: 智能知识传播");
    info!("Testing knowledge propagation system");

    let config = CollaborationConfig::default();
    let system = CollaborativeMemorySystem::new(config);

    // 创建知识项
    let knowledge = KnowledgeItem {
        id: "knowledge_1".to_string(),
        content: "新的机器学习算法优化技巧：使用自适应学习率可以提高收敛速度。".to_string(),
        knowledge_type: KnowledgeType::Procedural,
        confidence_score: 0.9,
        source_agent: "expert_agent".to_string(),
        created_at: Utc::now(),
        tags: vec!["机器学习".to_string(), "优化".to_string()],
        metadata: HashMap::new(),
    };

    // 设置订阅关系
    let target_agents = vec![
        "learner_agent_1".to_string(),
        "learner_agent_2".to_string(),
        "learner_agent_3".to_string(),
    ];

    for agent in &target_agents {
        system
            .knowledge_propagator()
            .subscribe(agent.clone(), KnowledgeType::Procedural)
            .await?;
    }

    // 执行知识传播
    let propagation_operation = CollaborationOperation::PropagateKnowledge {
        knowledge,
        target_agents: target_agents.clone(),
    };

    let result = system.execute_operation(propagation_operation).await?;
    println!("  🌐 知识传播结果: {result:?}");

    // 获取传播统计
    let stats = system
        .knowledge_propagator()
        .get_propagation_statistics()
        .await?;
    println!("  📊 传播统计:");
    println!("    - 总传播次数: {}", stats.total_propagations);
    println!("    - 成功率: {:.2}%", stats.success_rate * 100.0);
    println!("    - 知识库大小: {}", stats.total_knowledge_items);

    println!("  ✅ 知识传播演示完成\n");
    Ok(())
}

/// 演示 4: 冲突解决
async fn demo_conflict_resolution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 演示 4: 多策略冲突解决");
    info!("Testing conflict resolution system");

    let config = CollaborationConfig {
        conflict_resolution_strategy: ConflictResolutionStrategy::ImportanceBased,
        ..Default::default()
    };
    let system = CollaborativeMemorySystem::new(config);

    // 创建冲突版本
    let conflicting_versions = vec![
        ConflictingVersion {
            version_id: "v1".to_string(),
            content: "项目截止日期是下周五。".to_string(),
            agent_id: "agent_a".to_string(),
            timestamp: Utc::now(),
            importance_score: 0.7,
            access_count: 10,
        },
        ConflictingVersion {
            version_id: "v2".to_string(),
            content: "项目截止日期延期到下个月。".to_string(),
            agent_id: "agent_b".to_string(),
            timestamp: Utc::now(),
            importance_score: 0.9,
            access_count: 5,
        },
    ];

    // 检测冲突
    let conflict_id = system
        .conflict_resolver()
        .detect_conflict("project_deadline".to_string(), conflicting_versions)
        .await?;

    println!("  ⚠️ 检测到冲突: {conflict_id}");

    // 解决冲突
    let resolution_operation = CollaborationOperation::ResolveConflict {
        conflict_id: conflict_id.clone(),
        resolution: ConflictResolution {
            conflict_id: conflict_id.clone(),
            resolution_type: ResolutionType::Accept,
            resolved_content: "".to_string(),
            resolver_agent: "system".to_string(),
            resolution_time: Utc::now(),
            confidence: 0.8,
        },
    };

    let result = system.execute_operation(resolution_operation).await?;
    println!("  ⚖️ 冲突解决结果: {result:?}");

    // 获取冲突统计
    let stats = system.conflict_resolver().get_conflict_statistics().await?;
    println!("  📊 冲突统计:");
    println!("    - 总冲突数: {}", stats.total_conflicts);
    println!("    - 已解决: {}", stats.resolved_conflicts);
    println!("    - 待处理: {}", stats.pending_conflicts);

    println!("  ✅ 冲突解决演示完成\n");
    Ok(())
}

/// 演示 5: 系统统计
async fn demo_system_statistics() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 演示 5: 协作系统统计分析");
    info!("Analyzing collaboration system statistics");

    let config = CollaborationConfig::default();
    let system = CollaborativeMemorySystem::new(config);

    // 获取系统统计
    let stats = system.get_system_statistics().await?;

    println!("  📊 协作系统统计信息:");
    println!("    🔍 访问统计:");
    println!(
        "      - 总访问次数: {}",
        stats.access_statistics.total_accesses
    );
    println!(
        "      - 成功访问: {}",
        stats.access_statistics.successful_accesses
    );
    println!(
        "      - 失败访问: {}",
        stats.access_statistics.failed_accesses
    );
    println!(
        "      - 活跃智能体: {}",
        stats.access_statistics.unique_agents
    );

    println!("    ⚖️ 冲突统计:");
    println!(
        "      - 总冲突数: {}",
        stats.conflict_statistics.total_conflicts
    );
    println!(
        "      - 已解决: {}",
        stats.conflict_statistics.resolved_conflicts
    );
    println!(
        "      - 平均解决时间: {:.2}秒",
        stats.conflict_statistics.average_resolution_time_seconds
    );

    println!("    🌐 传播统计:");
    println!(
        "      - 传播成功率: {:.2}%",
        stats.propagation_statistics.success_rate * 100.0
    );
    println!(
        "      - 知识库大小: {}",
        stats.propagation_statistics.total_knowledge_items
    );

    println!("  ✅ 系统统计演示完成\n");
    Ok(())
}

// 辅助函数：创建测试记忆
fn create_memory_item(id: String, content: String, importance: f32) -> MemoryItem {
    let now = Utc::now();

    MemoryItem {
        id,
        content,
        hash: None,
        metadata: HashMap::new(),
        score: Some(importance),
        created_at: now,
        updated_at: Some(now),
        session: Session::new(),
        memory_type: MemoryType::Episodic,
        entities: Vec::new(),
        relations: Vec::new(),
        agent_id: "test_agent".to_string(),
        user_id: Some("test_user".to_string()),
        importance,
        embedding: None,
        last_accessed_at: now,
        access_count: 0,
        expires_at: None,
        version: 1,
    }
}

fn create_test_memory(id: String, content: String, importance: f32) -> MemoryItem {
    create_memory_item(id, content, importance)
}
