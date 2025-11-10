//! 数据库 Schema 缺失字段演示
//!
//! 此示例演示新添加的数据库字段：
//! - embedding: 向量嵌入（用于语义搜索）
//! - expires_at: 过期时间（用于工作记忆）
//! - version: 版本号（用于乐观锁定）

use agent_mem_core::types::{Memory, MemoryType, AttributeKey, AttributeValue};
use agent_mem_traits::Vector;
use chrono::Utc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    env_logger::init();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║          AgentMem 数据库 Schema 新字段演示                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 1. 演示 embedding 字段（向量嵌入）
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1️⃣  演示 embedding 字段（向量嵌入）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut memory_with_embedding = Memory::new(
        "agent-001".to_string(),
        Some("user-001".to_string()),
        MemoryType::Semantic,
        "这是一段包含向量嵌入的语义记忆".to_string(),
        0.9,
    );

    // 添加向量嵌入（384维，模拟 sentence-transformers/all-MiniLM-L6-v2）
    let embedding_vector = Vector {
        id: "embedding-001".to_string(),
        values: vec![0.1, 0.2, 0.3, 0.4, 0.5], // 简化示例，实际应为 384 维
        metadata: HashMap::new(),
    };
    // Note: embedding now stored in attributes
    memory_with_embedding.attributes.set(
        agent_mem_core::types::AttributeKey::system("embedding"),
        agent_mem_core::types::AttributeValue::Array(
            embedding_vector.values.iter().map(|&v| agent_mem_core::types::AttributeValue::Number(v as f64)).collect()
        ),
    );

    println!("✅ 创建带有 embedding 的记忆:");
    println!("   - ID: {}", memory_with_embedding.id);
    println!("   - 内容: {}", memory_with_embedding.content);
    println!("   - Embedding 维度: {}", embedding_vector.values.len());
    println!(
        "   - Embedding 前5个值: {:?}",
        &embedding_vector.values[..5.min(embedding_vector.values.len())]
    );
    println!();

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 2. 演示 expires_at 字段（过期时间）
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2️⃣  演示 expires_at 字段（过期时间）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut working_memory = Memory::new(
        "agent-001".to_string(),
        Some("user-001".to_string()),
        MemoryType::Working,
        "这是一段临时工作记忆，将在1小时后过期".to_string(),
        0.7,
    );

    // 设置过期时间为1小时后
    let expires_at = Utc::now().timestamp() + 3600; // 1小时 = 3600秒
    working_memory.attributes.set(
        AttributeKey::system("expires_at"),
        AttributeValue::Number(expires_at as f64),
    );

    println!("✅ 创建带有 expires_at 的工作记忆:");
    println!("   - ID: {}", working_memory.id);
    println!("   - 内容: {}", working_memory.content.to_string());
    println!(
        "   - 创建时间: {}",
        working_memory.created_at()
    );
    println!(
        "   - 过期时间: {}",
        chrono::DateTime::from_timestamp(expires_at, 0).unwrap()
    );
    let is_expired = working_memory.attributes
        .get(&AttributeKey::system("expires_at"))
        .and_then(|v| v.as_number())
        .map(|ts| chrono::Utc::now().timestamp() > ts as i64)
        .unwrap_or(false);
    println!("   - 是否已过期: {}", is_expired);
    println!();

    // 测试过期检查
    let mut expired_memory = Memory::new(
        "agent-001".to_string(),
        Some("user-001".to_string()),
        MemoryType::Working,
        "这是一段已过期的记忆".to_string(),
        0.5,
    );
    let expired_timestamp = Utc::now().timestamp() - 3600; // 1小时前
    expired_memory.attributes.set(
        AttributeKey::system("expires_at"),
        AttributeValue::Number(expired_timestamp as f64),
    );

    println!("✅ 测试已过期的记忆:");
    println!("   - ID: {}", expired_memory.id);
    let expired_at = expired_memory.attributes
        .get(&AttributeKey::system("expires_at"))
        .and_then(|v| v.as_number())
        .map(|ts| chrono::DateTime::from_timestamp(ts as i64, 0).unwrap());
    if let Some(expired_at) = expired_at {
        println!("   - 过期时间: {}", expired_at);
    }
    let is_expired = expired_memory.attributes
        .get(&AttributeKey::system("expires_at"))
        .and_then(|v| v.as_number())
        .map(|ts| chrono::Utc::now().timestamp() > ts as i64)
        .unwrap_or(false);
    println!("   - 是否已过期: {} ⚠️", is_expired);
    println!();

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 3. 演示 version 字段（乐观锁定）
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3️⃣  演示 version 字段（乐观锁定）");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut versioned_memory = Memory::new(
        "agent-001".to_string(),
        Some("user-001".to_string()),
        MemoryType::Core,
        "这是一段支持版本控制的核心记忆".to_string(),
        0.95,
    );

    println!("✅ 创建带有 version 的记忆:");
    println!("   - ID: {}", versioned_memory.id);
    println!("   - 内容: {}", versioned_memory.content.to_string());
    println!("   - 初始版本: {}", versioned_memory.version());
    println!();

    // 模拟更新操作
    println!("📝 执行第一次更新...");
    versioned_memory.update_content("更新后的核心记忆内容 - 版本 2".to_string());
    println!("   - 新内容: {}", versioned_memory.content.to_string());
    println!("   - 新版本: {}", versioned_memory.version());
    println!();

    println!("📝 执行第二次更新...");
    versioned_memory.update_content("再次更新的核心记忆内容 - 版本 3".to_string());
    println!("   - 新内容: {}", versioned_memory.content.to_string());
    println!("   - 新版本: {}", versioned_memory.version());
    println!();

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 4. 综合演示：所有字段
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("4️⃣  综合演示：包含所有新字段的记忆");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut comprehensive_memory = Memory::new(
        "agent-002".to_string(),
        Some("user-002".to_string()),
        MemoryType::Episodic,
        "这是一段包含所有新字段的情景记忆".to_string(),
        0.85,
    );

    // 添加 embedding
    let embedding_values = vec![0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2];
    comprehensive_memory.attributes.set(
        AttributeKey::system("embedding"),
        AttributeValue::Array(embedding_values.clone()),
    );

    // 添加 expires_at
    let expires_timestamp = Utc::now().timestamp() + 7200; // 2小时后
    comprehensive_memory.attributes.set(
        AttributeKey::system("expires_at"),
        AttributeValue::Number(expires_timestamp as f64),
    );

    // 添加 metadata
    comprehensive_memory.add_metadata("source".to_string(), "user_conversation".to_string());
    comprehensive_memory.add_metadata("context".to_string(), "meeting_notes".to_string());

    println!("✅ 综合记忆详情:");
    println!("   - ID: {}", comprehensive_memory.id);
    println!("   - Agent ID: {}", comprehensive_memory.agent_id());
    println!("   - User ID: {:?}", comprehensive_memory.user_id());
    println!("   - 类型: {:?}", comprehensive_memory.memory_type());
    println!("   - 内容: {}", comprehensive_memory.content.to_string());
    println!("   - 重要性: {}", comprehensive_memory.importance());
    let embedding_info = comprehensive_memory.attributes
        .get(&AttributeKey::system("embedding"))
        .and_then(|v| v.as_array())
        .map(|arr| (arr.len(), arr.iter().take(3).map(|x| format!("{:.2}", x)).collect::<Vec<_>>().join(", ")));
    if let Some((len, preview)) = embedding_info {
        println!("   - Embedding: [{}...] ({}维)", preview, len);
    }
    let expires_at = comprehensive_memory.attributes
        .get(&AttributeKey::system("expires_at"))
        .and_then(|v| v.as_number())
        .map(|ts| chrono::DateTime::from_timestamp(ts as i64, 0).unwrap());
    if let Some(expires_at) = expires_at {
        println!("   - 过期时间: {}", expires_at);
    }
    println!("   - 版本: {}", comprehensive_memory.version());
    println!("   - 访问次数: {}", comprehensive_memory.metadata.accessed_count);
    println!("   - Metadata: {:?}", comprehensive_memory.metadata.to_hashmap());
    println!();

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // 5. 数据库 Schema 信息
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("5️⃣  数据库 Schema 信息");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("📊 PostgreSQL Schema:");
    println!("   CREATE TABLE memories (");
    println!("       id VARCHAR(255) PRIMARY KEY,");
    println!("       organization_id VARCHAR(255) NOT NULL,");
    println!("       user_id VARCHAR(255) NOT NULL,");
    println!("       agent_id VARCHAR(255) NOT NULL,");
    println!("       content TEXT NOT NULL,");
    println!("       ...");
    println!("       embedding JSONB,                    -- ✅ 新增字段");
    println!("       expires_at TIMESTAMPTZ,             -- ✅ 新增字段");
    println!("       version INTEGER NOT NULL DEFAULT 1, -- ✅ 新增字段");
    println!("       created_at TIMESTAMPTZ NOT NULL,");
    println!("       updated_at TIMESTAMPTZ NOT NULL");
    println!("   );");
    println!();

    println!("📊 LibSQL Schema:");
    println!("   CREATE TABLE memories (");
    println!("       id TEXT PRIMARY KEY,");
    println!("       organization_id TEXT NOT NULL,");
    println!("       user_id TEXT NOT NULL,");
    println!("       agent_id TEXT NOT NULL,");
    println!("       content TEXT NOT NULL,");
    println!("       ...");
    println!("       embedding TEXT,                     -- ✅ 新增字段 (JSON格式)");
    println!("       expires_at INTEGER,                 -- ✅ 新增字段 (Unix时间戳)");
    println!("       version INTEGER NOT NULL DEFAULT 1, -- ✅ 新增字段");
    println!("       created_at INTEGER NOT NULL,");
    println!("       updated_at INTEGER NOT NULL");
    println!("   );");
    println!();

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║          🎉 数据库 Schema 新字段演示完成！                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    println!("✅ 演示结果:");
    println!("   - embedding 字段: 支持向量嵌入存储 ✅");
    println!("   - expires_at 字段: 支持记忆过期管理 ✅");
    println!("   - version 字段: 支持乐观锁定和版本控制 ✅");
    println!();

    println!("📝 说明:");
    println!("   - 所有字段已添加到 PostgreSQL 和 LibSQL schema");
    println!("   - INSERT/UPDATE 语句已更新以包含新字段");
    println!("   - 读取逻辑已更新以正确解析新字段");
    println!("   - 迁移脚本已创建，可安全应用到现有数据库");
    println!();

    Ok(())
}
