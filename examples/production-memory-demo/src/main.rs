//! Production Memory Demo
//!
//! 演示如何在生产环境中使用 AgentMem 的持久化存储功能。
//!
//! 本示例展示：
//! 1. 使用环境变量配置数据库
//! 2. 使用 Agent API 进行持久化存储
//! 3. 数据在重启后仍然存在
//! 4. 对比 SimpleMemory（内存存储）和 Agent（持久化存储）

use agent_mem_core::agents::{CoreAgent, EpisodicAgent, MemoryAgent, SemanticAgent};
use agent_mem_core::SimpleMemory;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("production_memory_demo=info,agent_mem_core=info")
        .init();

    info!("🚀 AgentMem 生产环境演示");
    info!("============================================================");

    // 演示 1: SimpleMemory（开发/测试用）
    demo_simple_memory().await;

    info!("");
    info!("============================================================");

    // 演示 2: Agent API（生产环境用）
    demo_agent_api().await;

    info!("");
    info!("============================================================");
    info!("✅ 演示完成！");
    info!("");
    info!("📝 总结：");
    info!("  - SimpleMemory: 适合开发和测试，数据存储在内存中");
    info!("  - Agent API: 适合生产环境，数据持久化到数据库");
    info!("  - 默认使用 LibSQL 嵌入式数据库（agentmem.db）");
    info!("  - 可通过环境变量配置 PostgreSQL 等其他数据库");
}

/// 演示 SimpleMemory（内存存储）
async fn demo_simple_memory() {
    info!("📦 演示 1: SimpleMemory（开发/测试模式）");
    info!("------------------------------------------------------------");

    match SimpleMemory::new().await {
        Ok(mem) => {
            info!("✅ SimpleMemory 创建成功");
            info!("⚠️  注意：数据存储在内存中，进程退出后会丢失");

            // 添加一些测试数据
            match mem.add("我喜欢吃披萨").await {
                Ok(id) => info!("✅ 添加记忆成功: {}", id),
                Err(e) => error!("❌ 添加记忆失败: {}", e),
            }

            match mem.add("我的生日是 1990-01-01").await {
                Ok(id) => info!("✅ 添加记忆成功: {}", id),
                Err(e) => error!("❌ 添加记忆失败: {}", e),
            }

            // 搜索记忆
            match mem.search("我喜欢什么？").await {
                Ok(results) => {
                    info!("✅ 搜索到 {} 条记忆", results.len());
                    for result in results {
                        info!("  - {}", result.content);
                    }
                }
                Err(e) => error!("❌ 搜索失败: {}", e),
            }

            info!("⚠️  这些数据在进程退出后会丢失！");
        }
        Err(e) => {
            error!("❌ SimpleMemory 创建失败: {}", e);
        }
    }
}

/// 演示 Agent API（持久化存储）
async fn demo_agent_api() {
    info!("💾 演示 2: Agent API（生产环境模式）");
    info!("------------------------------------------------------------");

    // 演示 CoreAgent
    info!("1️⃣  CoreAgent（核心记忆）");
    match CoreAgent::from_env("production-agent-1".to_string()).await {
        Ok(mut agent) => {
            info!("✅ CoreAgent 创建成功");

            // 初始化
            match agent.initialize().await {
                Ok(_) => info!("✅ Agent 初始化成功"),
                Err(e) => error!("❌ Agent 初始化失败: {}", e),
            }

            // 健康检查
            if agent.health_check().await {
                info!("✅ Agent 健康检查通过");
            } else {
                error!("❌ Agent 健康检查失败");
            }

            info!("💾 数据将持久化到数据库（默认: agentmem.db）");
            info!("🔄 重启后数据仍然存在");
        }
        Err(e) => {
            error!("❌ CoreAgent 创建失败: {}", e);
        }
    }

    info!("");

    // 演示 EpisodicAgent
    info!("2️⃣  EpisodicAgent（情节记忆）");
    match EpisodicAgent::from_env("production-agent-2".to_string()).await {
        Ok(mut agent) => {
            info!("✅ EpisodicAgent 创建成功");

            match agent.initialize().await {
                Ok(_) => info!("✅ Agent 初始化成功"),
                Err(e) => error!("❌ Agent 初始化失败: {}", e),
            }

            if agent.health_check().await {
                info!("✅ Agent 健康检查通过");
            }
        }
        Err(e) => {
            error!("❌ EpisodicAgent 创建失败: {}", e);
        }
    }

    info!("");

    // 演示 SemanticAgent
    info!("3️⃣  SemanticAgent（语义记忆）");
    match SemanticAgent::from_env("production-agent-3".to_string()).await {
        Ok(mut agent) => {
            info!("✅ SemanticAgent 创建成功");

            match agent.initialize().await {
                Ok(_) => info!("✅ Agent 初始化成功"),
                Err(e) => error!("❌ Agent 初始化失败: {}", e),
            }

            if agent.health_check().await {
                info!("✅ Agent 健康检查通过");
            }
        }
        Err(e) => {
            error!("❌ SemanticAgent 创建失败: {}", e);
        }
    }

    info!("");
    info!("📊 环境变量配置：");
    info!("  - DATABASE_URL: 完整的数据库连接字符串");
    info!("  - AGENTMEM_DB_PATH: LibSQL 数据库文件路径（默认: agentmem.db）");
    info!("  - AGENTMEM_DB_BACKEND: 后端类型（postgres 或 libsql）");
    info!("");
    info!("💡 示例：");
    info!("  # 使用默认 LibSQL");
    info!("  cargo run --example production-memory-demo");
    info!("");
    info!("  # 使用自定义 LibSQL 路径");
    info!("  AGENTMEM_DB_PATH=./data/memory.db cargo run --example production-memory-demo");
    info!("");
    info!("  # 使用 PostgreSQL");
    info!("  DATABASE_URL=postgresql://user:pass@localhost/agentmem cargo run --example production-memory-demo");
}
