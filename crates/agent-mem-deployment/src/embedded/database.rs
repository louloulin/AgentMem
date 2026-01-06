//! 嵌入式数据库实现
//!
//! 基于 LibSQL 的嵌入式数据库，支持单二进制部署

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[cfg(feature = "embedded-db")]
use libsql::Database;

/// 嵌入式数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedDatabaseConfig {
    /// 数据库文件路径
    pub path: PathBuf,

    /// 是否启用 WAL 模式
    pub enable_wal: bool,

    /// 缓存大小（KB）
    pub cache_size_kb: usize,

    /// 页面大小（字节）
    pub page_size: usize,

    /// 是否自动创建目录
    pub auto_create_dir: bool,

    /// 是否在内存中运行（用于测试）
    pub in_memory: bool,
}

impl Default for EmbeddedDatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data/agentmem.db"),
            enable_wal: true,
            cache_size_kb: 10240, // 10 MB
            page_size: 4096,      // 4 KB
            auto_create_dir: true,
            in_memory: false,
        }
    }
}

impl EmbeddedDatabaseConfig {
    /// 创建内存数据库配置（用于测试）
    pub fn in_memory() -> Self {
        Self {
            in_memory: true,
            ..Default::default()
        }
    }

    /// 创建文件数据库配置
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            in_memory: false,
            ..Default::default()
        }
    }
}

/// 嵌入式数据库
pub struct EmbeddedDatabase {
    config: EmbeddedDatabaseConfig,
    #[cfg(feature = "embedded-db")]
    db: Arc<RwLock<Option<Database>>>,
    #[cfg(not(feature = "embedded-db"))]
    _phantom: std::marker::PhantomData<()>,
}

impl EmbeddedDatabase {
    /// 创建新的嵌入式数据库实例
    pub fn new(config: EmbeddedDatabaseConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "embedded-db")]
            db: Arc::new(RwLock::new(None)),
            #[cfg(not(feature = "embedded-db"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// 初始化数据库
    pub async fn initialize(&self) -> Result<()> {
        #[cfg(feature = "embedded-db")]
        {
            info!("初始化嵌入式数据库...");

            // 创建数据目录
            if self.config.auto_create_dir && !self.config.in_memory {
                if let Some(parent) = self.config.path.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| {
                        AgentMemError::internal_error(format!("创建数据目录失败: {e}"))
                    })?;
                }
            }

            // 打开数据库
            let db = if self.config.in_memory {
                debug!("使用内存数据库");
                Database::open(":memory:").map_err(|e| {
                    AgentMemError::internal_error(format!("打开内存数据库失败: {e}"))
                })?
            } else {
                debug!("使用文件数据库: {:?}", self.config.path);
                let path_str = self
                    .config
                    .path
                    .to_str()
                    .ok_or_else(|| AgentMemError::internal_error("无效的数据库路径"))?;
                Database::open(path_str)
                    .map_err(|e| AgentMemError::internal_error(format!("打开数据库失败: {e}")))?
            };

            // 配置数据库
            let conn = db
                .connect()
                .map_err(|e| AgentMemError::internal_error(format!("连接数据库失败: {e}")))?;

            // 启用 WAL 模式
            if self.config.enable_wal && !self.config.in_memory {
                conn.execute("PRAGMA journal_mode=WAL", ())
                    .await
                    .map_err(|e| {
                        AgentMemError::internal_error(format!("启用 WAL 模式失败: {e}"))
                    })?;
                debug!("已启用 WAL 模式");
            }

            // 设置缓存大小
            let cache_pages = self.config.cache_size_kb * 1024 / self.config.page_size;
            conn.execute(&format!("PRAGMA cache_size={cache_pages}"), ())
                .await
                .map_err(|e| AgentMemError::internal_error(format!("设置缓存大小失败: {e}")))?;
            debug!("缓存大小设置为 {} 页", cache_pages);

            // 保存数据库实例
            let mut db_lock = self.db.write().await;
            *db_lock = Some(db);

            info!("嵌入式数据库初始化完成");
            Ok(())
        }

        #[cfg(not(feature = "embedded-db"))]
        {
            warn!("嵌入式数据库功能未启用");
            Err(AgentMemError::internal_error(
                "嵌入式数据库功能未启用，请启用 'embedded-db' feature",
            ))
        }
    }

    /// 关闭数据库
    pub async fn shutdown(&self) -> Result<()> {
        #[cfg(feature = "embedded-db")]
        {
            info!("关闭嵌入式数据库...");
            let mut db_lock = self.db.write().await;
            *db_lock = None;
            info!("嵌入式数据库已关闭");
            Ok(())
        }

        #[cfg(not(feature = "embedded-db"))]
        {
            Ok(())
        }
    }

    /// 执行 SQL 查询
    #[cfg(feature = "embedded-db")]
    pub async fn execute(&self, sql: &str) -> Result<()> {
        let db_lock = self.db.read().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| AgentMemError::internal_error("数据库未初始化"))?;

        let conn = db
            .connect()
            .map_err(|e| AgentMemError::internal_error(format!("连接数据库失败: {e}")))?;

        // LibSQL 的 execute 方法返回 Future
        conn.execute(sql, ())
            .await
            .map_err(|e| AgentMemError::internal_error(format!("执行 SQL 失败: {e}")))?;

        Ok(())
    }

    /// 获取数据库连接
    #[cfg(feature = "embedded-db")]
    pub async fn get_connection(&self) -> Result<libsql::Connection> {
        let db_lock = self.db.read().await;
        let db = db_lock
            .as_ref()
            .ok_or_else(|| AgentMemError::internal_error("数据库未初始化"))?;

        db.connect()
            .map_err(|e| AgentMemError::internal_error(format!("连接数据库失败: {e}")))
    }

    /// 获取数据库大小（字节）
    pub async fn get_size(&self) -> Result<u64> {
        if self.config.in_memory {
            return Ok(0);
        }

        let metadata = tokio::fs::metadata(&self.config.path)
            .await
            .map_err(|e| AgentMemError::internal_error(format!("获取数据库大小失败: {e}")))?;

        Ok(metadata.len())
    }

    /// 优化数据库（VACUUM）
    #[cfg(feature = "embedded-db")]
    pub async fn optimize(&self) -> Result<()> {
        info!("优化数据库...");
        self.execute("VACUUM").await?;
        info!("数据库优化完成");
        Ok(())
    }

    /// 备份数据库
    pub async fn backup<P: AsRef<Path>>(&self, dest: P) -> Result<()> {
        if self.config.in_memory {
            return Err(AgentMemError::internal_error("内存数据库不支持备份"));
        }

        info!("备份数据库到: {:?}", dest.as_ref());

        // 创建目标目录
        if let Some(parent) = dest.as_ref().parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AgentMemError::internal_error(format!("创建备份目录失败: {e}")))?;
        }

        // 复制数据库文件
        tokio::fs::copy(&self.config.path, dest.as_ref())
            .await
            .map_err(|e| AgentMemError::internal_error(format!("备份数据库失败: {e}")))?;

        info!("数据库备份完成");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedded_database_creation() {
        let config = EmbeddedDatabaseConfig::in_memory();
        let _db = EmbeddedDatabase::new(config);
    }

    #[tokio::test]
    #[cfg(feature = "embedded-db")]
    async fn test_embedded_database_initialize() {
        let config = EmbeddedDatabaseConfig::in_memory();
        let db = EmbeddedDatabase::new(config);

        let result = db.initialize().await;
        assert!(result.is_ok());

        let result = db.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "embedded-db")]
    async fn test_embedded_database_execute() {
        let config = EmbeddedDatabaseConfig::in_memory();
        let db = EmbeddedDatabase::new(config);

        db.initialize().await.unwrap();

        // 获取连接
        let conn = db.get_connection().await.unwrap();

        // 创建表
        let result = conn
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", ())
            .await;
        if let Err(e) = &result {
            eprintln!("执行 CREATE TABLE 失败: {e:?}");
        }
        assert!(result.is_ok());

        // 插入数据
        let result = conn
            .execute("INSERT INTO test (name) VALUES ('test')", ())
            .await;
        if let Err(e) = &result {
            eprintln!("执行 INSERT 失败: {e:?}");
        }
        assert!(result.is_ok());

        db.shutdown().await.unwrap();
    }
}
