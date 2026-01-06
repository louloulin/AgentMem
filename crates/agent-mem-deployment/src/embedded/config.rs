//! 嵌入式配置管理
//!
//! 统一管理嵌入式数据库和向量存储的配置

use super::database::{EmbeddedDatabase, EmbeddedDatabaseConfig};
use super::vector_store::{EmbeddedVectorStore, EmbeddedVectorStoreConfig};
use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// 嵌入式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedConfig {
    /// 数据库配置
    pub database: EmbeddedDatabaseConfig,

    /// 向量存储配置
    pub vector_store: EmbeddedVectorStoreConfig,

    /// 数据根目录
    pub data_root: PathBuf,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        let data_root = PathBuf::from("./data");

        Self {
            database: EmbeddedDatabaseConfig {
                path: data_root.join("agentmem.db"),
                ..Default::default()
            },
            vector_store: EmbeddedVectorStoreConfig {
                path: data_root.join("vectors"),
                ..Default::default()
            },
            data_root,
        }
    }
}

impl EmbeddedConfig {
    /// 创建新的嵌入式配置
    pub fn new<P: AsRef<Path>>(data_root: P) -> Self {
        let data_root = data_root.as_ref().to_path_buf();

        Self {
            database: EmbeddedDatabaseConfig {
                path: data_root.join("agentmem.db"),
                ..Default::default()
            },
            vector_store: EmbeddedVectorStoreConfig {
                path: data_root.join("vectors"),
                ..Default::default()
            },
            data_root,
        }
    }

    /// 创建内存配置（用于测试）
    pub fn in_memory() -> Self {
        Self {
            database: EmbeddedDatabaseConfig::in_memory(),
            vector_store: EmbeddedVectorStoreConfig::in_memory(384),
            data_root: PathBuf::from(":memory:"),
        }
    }

    /// 设置向量维度
    pub fn with_vector_dimension(mut self, dimension: usize) -> Self {
        self.vector_store.dimension = dimension;
        self
    }

    /// 设置数据库 WAL 模式
    pub fn with_wal(mut self, enable: bool) -> Self {
        self.database.enable_wal = enable;
        self
    }

    /// 设置向量持久化
    pub fn with_persistence(mut self, enable: bool) -> Self {
        self.vector_store.enable_persistence = enable;
        self
    }

    /// 创建数据库实例
    pub fn create_database(&self) -> EmbeddedDatabase {
        EmbeddedDatabase::new(self.database.clone())
    }

    /// 创建向量存储实例
    pub fn create_vector_store(&self) -> EmbeddedVectorStore {
        EmbeddedVectorStore::new(self.vector_store.clone())
    }

    /// 初始化所有嵌入式存储
    pub async fn initialize_all(&self) -> Result<(EmbeddedDatabase, EmbeddedVectorStore)> {
        info!("初始化所有嵌入式存储...");

        let db = self.create_database();
        db.initialize().await?;

        let vector_store = self.create_vector_store();
        vector_store.initialize().await?;

        info!("所有嵌入式存储初始化完成");
        Ok((db, vector_store))
    }

    /// 从 TOML 文件加载配置
    pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("读取配置文件失败: {e}"))
        })?;

        toml::from_str(&content).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("解析配置文件失败: {e}"))
        })
    }

    /// 保存配置到 TOML 文件
    pub fn save_to_toml_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("序列化配置失败: {e}"))
        })?;

        std::fs::write(path, content).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("写入配置文件失败: {e}"))
        })
    }

    /// 从 JSON 文件加载配置
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("读取配置文件失败: {e}"))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("解析配置文件失败: {e}"))
        })
    }

    /// 保存配置到 JSON 文件
    pub fn save_to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("序列化配置失败: {e}"))
        })?;

        std::fs::write(path, content).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("写入配置文件失败: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_config_creation() {
        let config = EmbeddedConfig::default();
        assert_eq!(config.data_root, PathBuf::from("./data"));
    }

    #[test]
    fn test_embedded_config_in_memory() {
        let config = EmbeddedConfig::in_memory();
        assert!(config.database.in_memory);
        assert!(config.vector_store.in_memory);
    }

    #[test]
    fn test_embedded_config_with_dimension() {
        let config = EmbeddedConfig::default().with_vector_dimension(768);
        assert_eq!(config.vector_store.dimension, 768);
    }

    #[test]
    fn test_embedded_config_serialization() {
        let config = EmbeddedConfig::default();

        // 测试 JSON 序列化
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EmbeddedConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data_root, config.data_root);

        // 测试 TOML 序列化
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: EmbeddedConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.data_root, config.data_root);
    }

    #[tokio::test]
    #[cfg(all(feature = "embedded-db", feature = "embedded-vector"))]
    async fn test_embedded_config_initialize_all() {
        let config = EmbeddedConfig::in_memory();
        let result = config.initialize_all().await;
        assert!(result.is_ok());

        let (db, vector_store) = result.unwrap();

        // 关闭
        db.shutdown().await.unwrap();
        vector_store.shutdown().await.unwrap();
    }
}
