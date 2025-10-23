//! AgentMem 嵌入式模式基础使用示例
//!
//! 演示如何使用嵌入式模式（LibSQL + LanceDB）进行基本的数据操作

use agent_mem_config::storage::DeploymentMode;
use agent_mem_core::storage::factory::StorageFactory;
use agent_mem_core::storage::models::{User, Organization, Agent};
use anyhow::Result;
use chrono::Utc;
use tracing::{info, Level};
use tracing_subscriber;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 AgentMem 嵌入式模式基础使用示例");

    // 1. 创建嵌入式模式配置
    let mode = DeploymentMode::embedded("./data");

    info!("📝 配置: {:?}", mode);

    // 2. 创建 Storage Factory
    info!("🔧 创建 Storage Factory...");
    let repositories = StorageFactory::create(mode).await?;
    info!("✅ Storage Factory 创建成功");

    // 3. 创建组织
    info!("\n📦 创建组织...");
    let org = Organization {
        id: Uuid::new_v4().to_string(),
        name: "示例组织".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_deleted: false,
    };
    
    let created_org = repositories.organizations.create(&org).await?;
    info!("✅ 组织创建成功: {} (ID: {})", created_org.name, created_org.id);

    // 4. 创建用户
    info!("\n👤 创建用户...");
    let user = User {
        id: Uuid::new_v4().to_string(),
        organization_id: created_org.id.clone(),
        name: "张三".to_string(),
        email: "zhangsan@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        roles: Some(vec!["user".to_string()]),
        status: "active".to_string(),
        timezone: "Asia/Shanghai".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_deleted: false,
        created_by_id: None,
        last_updated_by_id: None,
    };
    
    let created_user = repositories.users.create(&user).await?;
    info!("✅ 用户创建成功: {} (ID: {})", created_user.name, created_user.id);

    // 5. 创建 Agent
    info!("\n🤖 创建 Agent...");
    let agent = Agent {
        id: Uuid::new_v4().to_string(),
        organization_id: created_org.id.clone(),
        name: Some("智能助手".to_string()),
        agent_type: Some("assistant".to_string()),
        description: None,
        system: None,
        topic: None,
        message_ids: None,
        metadata_: None,
        llm_config: Some(serde_json::json!({
            "model": "gpt-4",
            "temperature": 0.7,
            "max_tokens": 2000
        })),
        embedding_config: None,
        tool_rules: None,
        mcp_tools: None,
        state: Some("idle".to_string()),
        last_active_at: None,
        error_message: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_deleted: false,
        created_by_id: Some(created_user.id.clone()),
        last_updated_by_id: Some(created_user.id.clone()),
    };
    
    let created_agent = repositories.agents.create(&agent).await?;
    info!("✅ Agent 创建成功: {} (ID: {})", created_agent.name.as_deref().unwrap_or("未命名"), created_agent.id);

    // 6. 查询数据
    info!("\n🔍 查询数据...");
    
    // 查询用户
    if let Some(found_user) = repositories.users.find_by_id(&created_user.id).await? {
        info!("✅ 找到用户: {}", found_user.name);
    }
    
    // 查询组织的所有 Agent
    let agents = repositories.agents.find_by_organization_id(&created_org.id).await?;
    info!("✅ 组织 {} 有 {} 个 Agent", created_org.name, agents.len());

    // 7. 更新数据
    info!("\n📝 更新 Agent 配置...");
    let mut updated_agent = created_agent.clone();
    updated_agent.llm_config = Some(serde_json::json!({
        "model": "gpt-4-turbo",
        "temperature": 0.8,
        "max_tokens": 4000
    }));
    updated_agent.updated_at = Utc::now();
    
    let updated_agent = repositories.agents.update(&updated_agent).await?;
    info!("✅ Agent 配置更新成功");

    // 8. 列出所有用户
    info!("\n📋 列出所有用户...");
    let all_users = repositories.users.list(10, 0).await?;
    info!("✅ 共有 {} 个用户", all_users.len());
    for user in all_users {
        info!("  - {} ({})", user.name, user.id);
    }

    // 9. 软删除
    info!("\n🗑️  软删除 Agent...");
    repositories.agents.delete(&created_agent.id).await?;
    info!("✅ Agent 已软删除");

    // 验证软删除
    if let Some(deleted_agent) = repositories.agents.find_by_id(&created_agent.id).await? {
        info!("⚠️  注意: 软删除的 Agent 仍然可以通过 ID 查询到（is_deleted={}）", deleted_agent.is_deleted);
    }

    info!("\n🎉 基础使用示例完成！");
    info!("💾 数据已保存到: ./data/agentmem.db");

    Ok(())
}

