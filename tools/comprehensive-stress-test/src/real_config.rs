//! 真实压测配置
//!
//! 管理真实数据库连接和 SDK 配置

use agent_mem::Memory;
use agent_mem_embeddings::{config::EmbeddingConfig, providers::LocalEmbedder};
use agent_mem_storage::backends::lancedb_store::LanceDBStore;
use agent_mem_traits::{Embedder, Result, VectorStore};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// 真实压测配置
#[derive(Debug, Clone)]
pub struct RealStressTestConfig {
    /// PostgreSQL 数据库 URL
    pub postgres_url: String,
    /// LanceDB 数据路径
    pub lancedb_path: String,
    /// 是否启用嵌入生成
    pub enable_embeddings: bool,
    /// 数据库连接池配置
    pub db_pool_config: DbPoolConfig,
}

/// 数据库连接池配置
#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

impl Default for DbPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 10,
            max_connections: 100,
            acquire_timeout_secs: 5,
            idle_timeout_secs: 600,
        }
    }
}

impl Default for RealStressTestConfig {
    fn default() -> Self {
        Self {
            postgres_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost:5432/agentmem_test".to_string()),
            lancedb_path: "./data/stress-test-vectors.lance".to_string(),
            enable_embeddings: true,
            db_pool_config: DbPoolConfig::default(),
        }
    }
}

/// 真实压测环境
///
/// 包含所有真实的数据库连接和 SDK 实例
pub struct RealStressTestEnv {
    /// AgentMem SDK 实例
    pub memory: Arc<Memory>,
    /// PostgreSQL 连接池
    pub pg_pool: Arc<PgPool>,
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

        // 1. 初始化 PostgreSQL 连接池
        info!("📊 连接 PostgreSQL: {}", mask_password(&config.postgres_url));
        let pg_pool = PgPoolOptions::new()
            .min_connections(config.db_pool_config.min_connections)
            .max_connections(config.db_pool_config.max_connections)
            .acquire_timeout(Duration::from_secs(
                config.db_pool_config.acquire_timeout_secs,
            ))
            .idle_timeout(Duration::from_secs(
                config.db_pool_config.idle_timeout_secs,
            ))
            .test_before_acquire(true)
            .connect(&config.postgres_url)
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to connect to PostgreSQL: {}",
                    e
                ))
            })?;

        info!("✅ PostgreSQL 连接成功");

        // 2. 初始化 LanceDB 向量存储
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

        // 3. 初始化嵌入生成器（如果启用）
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

        // 4. 初始化 AgentMem SDK
        info!("🎯 初始化 AgentMem SDK...");
        let memory = Memory::builder()
            .with_storage(&config.postgres_url)
            .build()
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to initialize AgentMem: {}",
                    e
                ))
            })?;

        info!("✅ AgentMem SDK 初始化成功");

        info!("🎉 真实压测环境初始化完成！");

        Ok(Self {
            memory: Arc::new(memory),
            pg_pool: Arc::new(pg_pool),
            vector_store: Arc::new(vector_store),
            embedder,
            config,
        })
    }

    /// 清理测试数据
    pub async fn cleanup(&self) -> Result<()> {
        info!("🧹 清理测试数据...");

        // 清理 PostgreSQL 测试数据
        sqlx::query("DELETE FROM memories WHERE content LIKE 'Test memory%' OR content LIKE 'Batch%'")
            .execute(self.pg_pool.as_ref())
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to cleanup PostgreSQL: {}",
                    e
                ))
            })?;

        // 清理 LanceDB 测试数据
        if let Err(e) = self.vector_store.clear().await {
            warn!("⚠️  清理 LanceDB 失败: {}", e);
        }

        info!("✅ 测试数据清理完成");
        Ok(())
    }

    /// 获取数据库统计信息
    pub async fn get_db_stats(&self) -> Result<DbStats> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM memories")
            .fetch_one(self.pg_pool.as_ref())
            .await
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to get memory count: {}",
                    e
                ))
            })?;

        let memory_count = row.0 as usize;

        let vector_count = self.vector_store.count_vectors().await.unwrap_or(0);

        Ok(DbStats {
            memory_count,
            vector_count,
            pool_size: self.pg_pool.size() as usize,
            pool_idle: self.pg_pool.num_idle() as usize,
        })
    }
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DbStats {
    pub memory_count: usize,
    pub vector_count: usize,
    pub pool_size: usize,
    pub pool_idle: usize,
}

/// 屏蔽密码显示
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgresql://user:password@localhost:5432/db";
        let masked = mask_password(url);
        assert!(masked.contains("****"));
        assert!(!masked.contains("password"));
    }

    #[test]
    fn test_default_config() {
        let config = RealStressTestConfig::default();
        assert_eq!(config.db_pool_config.min_connections, 10);
        assert_eq!(config.db_pool_config.max_connections, 100);
    }
}

