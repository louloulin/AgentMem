//! 真实压测配置
//!
//! 管理真实数据库连接和 SDK 配置

use agent_mem::Memory;
use agent_mem_embeddings::{config::EmbeddingConfig, providers::LocalEmbedder};
use agent_mem_storage::backends::lancedb_store::LanceDBStore;
use agent_mem_traits::{Embedder, Result, VectorStore};
use std::sync::Arc;
use tracing::{info, warn};

/// 真实压测配置
#[derive(Debug, Clone)]
pub struct RealStressTestConfig {
    /// LibSQL 数据库路径
    pub libsql_path: String,
    /// LanceDB 数据路径
    pub lancedb_path: String,
    /// 是否启用嵌入生成
    pub enable_embeddings: bool,
}

impl Default for RealStressTestConfig {
    fn default() -> Self {
        Self {
            libsql_path: "./data/stress-test.db".to_string(),
            lancedb_path: "./data/stress-test-vectors.lance".to_string(),
            enable_embeddings: true,
        }
    }
}

/// 真实压测环境
///
/// 包含所有真实的数据库连接和 SDK 实例
pub struct RealStressTestEnv {
    /// AgentMem SDK 实例
    pub memory: Arc<Memory>,
    /// LanceDB 向量存储
    pub vector_store: Arc<LanceDBStore>,
    /// 嵌入生成器
    pub embedder: Option<Arc<LocalEmbedder>>,
    /// 配置
    pub config: RealStressTestConfig,
}

impl RealStressTestEnv {
    /// 初始化真实压测环境
    pub async fn new(config: RealStressTestConfig) -> Result<Self> {
        info!("🚀 初始化真实压测环境...");

        // 1. 初始化 LanceDB 向量存储
        info!("🔍 初始化 LanceDB: {}", config.lancedb_path);
        let vector_store = LanceDBStore::new(&config.lancedb_path, "stress_test_embeddings")
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to initialize LanceDB: {}",
                    e
                ))
            })?;

        info!("✅ LanceDB 初始化成功");

        // 2. 初始化嵌入生成器（如果启用）
        let embedder = if config.enable_embeddings {
            info!("🧠 初始化 FastEmbed 嵌入模型...");
            match LocalEmbedder::new(EmbeddingConfig::default()).await {
                Ok(emb) => {
                    info!("✅ FastEmbed 初始化成功");
                    Some(Arc::new(emb))
                }
                Err(e) => {
                    warn!("⚠️  FastEmbed 初始化失败: {}, 将使用确定性嵌入", e);
                    None
                }
            }
        } else {
            info!("⏭️  跳过嵌入生成器初始化");
            None
        };

        // 3. 初始化 AgentMem SDK (使用 LibSQL)
        info!("🎯 初始化 AgentMem SDK (LibSQL)...");
        let memory = Memory::new().await.map_err(|e| {
            agent_mem_traits::AgentMemError::storage_error(&format!(
                "Failed to initialize AgentMem: {}",
                e
            ))
        })?;

        info!("✅ AgentMem SDK 初始化成功");

        info!("🎉 真实压测环境初始化完成！");

        Ok(Self {
            memory: Arc::new(memory),
            vector_store: Arc::new(vector_store),
            embedder,
            config,
        })
    }

    /// 清理测试数据
    pub async fn cleanup(&self) -> Result<()> {
        info!("🧹 清理测试数据...");

        // 清理 LanceDB 测试数据
        if let Err(e) = self.vector_store.clear().await {
            warn!("⚠️  清理 LanceDB 失败: {}", e);
        }

        info!("✅ 测试数据清理完成");
        Ok(())
    }

    /// 获取数据库统计信息
    pub async fn get_db_stats(&self) -> Result<DbStats> {
        // LibSQL 不支持直接查询，使用 Memory SDK 的统计功能
        let memory_count = 0; // TODO: 实现统计功能

        let vector_count = self.vector_store.count_vectors().await.unwrap_or(0);

        Ok(DbStats {
            memory_count,
            vector_count,
        })
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DbStats {
    pub memory_count: usize,
    pub vector_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RealStressTestConfig::default();
        assert_eq!(config.libsql_path, "./data/stress-test.db");
        assert_eq!(config.lancedb_path, "./data/stress-test-vectors.lance");
    }
}
