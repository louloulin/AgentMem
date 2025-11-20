//! LibSQL 真实压测配置
//!
//! 使用 LibSQL 嵌入式数据库进行真实压测，无需外部数据库服务

use agent_mem::Memory;
use agent_mem_traits::Result;
use std::path::PathBuf;
use tracing::info;

/// LibSQL 压测配置
#[derive(Debug, Clone)]
pub struct LibSQLStressTestConfig {
    /// LibSQL 数据库文件路径
    pub db_path: String,

    /// 是否启用嵌入生成
    pub enable_embeddings: bool,

    /// 嵌入模型名称
    pub embedding_model: String,
}

impl Default for LibSQLStressTestConfig {
    fn default() -> Self {
        Self {
            db_path: "./data/stress-test.db".to_string(),
            enable_embeddings: false, // 默认禁用以加快测试速度
            embedding_model: "BAAI/bge-small-en-v1.5".to_string(),
        }
    }
}

/// LibSQL 压测环境
pub struct LibSQLStressTestEnv {
    /// AgentMem SDK 实例
    pub memory: Memory,

    /// 配置
    pub config: LibSQLStressTestConfig,
}

impl LibSQLStressTestEnv {
    /// 初始化 LibSQL 压测环境
    pub async fn new(config: LibSQLStressTestConfig) -> Result<Self> {
        info!("🚀 初始化 LibSQL 压测环境...");
        info!("   数据库路径: {}", config.db_path);
        info!(
            "   嵌入生成: {}",
            if config.enable_embeddings {
                "启用"
            } else {
                "禁用"
            }
        );

        // 确保数据目录存在
        if let Some(parent) = PathBuf::from(&config.db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 初始化 AgentMem SDK (使用 LibSQL 后端)
        // 格式: libsql://path/to/db
        let storage_url = format!("libsql://{}", config.db_path);

        let mut builder = Memory::builder().with_storage(&storage_url);

        // 如果启用嵌入，配置嵌入模型
        if config.enable_embeddings {
            builder = builder.with_embedder("local", &config.embedding_model);
        }

        let memory = builder.build().await?;

        info!("✅ LibSQL 压测环境初始化完成");

        Ok(Self { memory, config })
    }

    /// 清理测试数据
    pub async fn cleanup(&self) -> Result<()> {
        info!("🧹 清理 LibSQL 测试数据...");

        // 删除所有测试记忆
        // TODO: 实现批量删除 API

        info!("✅ 清理完成");
        Ok(())
    }

    /// 获取数据库统计信息
    pub async fn get_stats(&self) -> Result<DbStats> {
        info!("📊 获取数据库统计信息...");

        // TODO: 实现统计 API
        let stats = DbStats {
            total_memories: 0,
            total_vectors: 0,
            db_size_bytes: 0,
        };

        Ok(stats)
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DbStats {
    pub total_memories: usize,
    pub total_vectors: usize,
    pub db_size_bytes: u64,
}
