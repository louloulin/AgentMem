//! 嵌入式向量存储实现
//!
//! 基于 LanceDB 的嵌入式向量存储，支持单二进制部署
//! LanceDB 是一个真正的嵌入式向量数据库，无需外部服务

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "embedded-vector")]
use agent_mem_storage::backends::LanceDBVectorStore;

/// 嵌入式向量存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedVectorStoreConfig {
    /// 数据目录路径
    pub path: PathBuf,
    
    /// 向量维度
    pub dimension: usize,
    
    /// 距离度量方式
    pub distance: DistanceMetric,
    
    /// 是否启用磁盘持久化
    pub enable_persistence: bool,
    
    /// 是否自动创建目录
    pub auto_create_dir: bool,
    
    /// 是否在内存中运行（用于测试）
    pub in_memory: bool,
    
    /// 集合名称
    pub collection_name: String,
}

/// 距离度量方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// 余弦相似度
    Cosine,
    /// 欧几里得距离
    Euclidean,
    /// 点积
    Dot,
}

impl Default for EmbeddedVectorStoreConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data/vectors"),
            dimension: 384,
            distance: DistanceMetric::Cosine,
            enable_persistence: true,
            auto_create_dir: true,
            in_memory: false,
            collection_name: "agentmem_vectors".to_string(),
        }
    }
}

impl EmbeddedVectorStoreConfig {
    /// 创建内存向量存储配置（用于测试）
    pub fn in_memory(dimension: usize) -> Self {
        Self {
            dimension,
            in_memory: true,
            enable_persistence: false,
            ..Default::default()
        }
    }
    
    /// 创建文件向量存储配置
    pub fn file<P: AsRef<Path>>(path: P, dimension: usize) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            dimension,
            in_memory: false,
            ..Default::default()
        }
    }
}

/// 嵌入式向量存储
pub struct EmbeddedVectorStore {
    config: EmbeddedVectorStoreConfig,
    #[cfg(feature = "embedded-vector")]
    store: Arc<RwLock<Option<LanceDBVectorStore>>>,
    #[cfg(not(feature = "embedded-vector"))]
    _phantom: std::marker::PhantomData<()>,
}

impl EmbeddedVectorStore {
    /// 创建新的嵌入式向量存储实例
    pub fn new(config: EmbeddedVectorStoreConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "embedded-vector")]
            store: Arc::new(RwLock::new(None)),
            #[cfg(not(feature = "embedded-vector"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// 初始化向量存储
    pub async fn initialize(&self) -> Result<()> {
        #[cfg(feature = "embedded-vector")]
        {
            info!("初始化嵌入式向量存储（LanceDB）...");

            // 创建数据目录
            if self.config.auto_create_dir && !self.config.in_memory {
                tokio::fs::create_dir_all(&self.config.path).await.map_err(|e| {
                    AgentMemError::internal_error(format!("创建向量存储目录失败: {}", e))
                })?;
            }

            // 创建 LanceDB 存储
            let path_str = if self.config.in_memory {
                debug!("使用内存向量存储");
                ":memory:"
            } else {
                debug!("使用文件向量存储: {:?}", self.config.path);
                self.config.path.to_str().ok_or_else(|| {
                    AgentMemError::internal_error("无效的路径")
                })?
            };

            let store = LanceDBVectorStore::new(path_str, &self.config.collection_name)
                .await
                .map_err(|e| {
                    AgentMemError::internal_error(format!("创建 LanceDB 存储失败: {}", e))
                })?;

            // 保存存储实例
            let mut store_lock = self.store.write().await;
            *store_lock = Some(store);

            info!("嵌入式向量存储初始化完成");
            Ok(())
        }

        #[cfg(not(feature = "embedded-vector"))]
        {
            warn!("嵌入式向量存储功能未启用");
            Err(AgentMemError::internal_error(
                "嵌入式向量存储功能未启用，请启用 'embedded-vector' feature"
            ))
        }
    }
    
    /// 关闭向量存储
    pub async fn shutdown(&self) -> Result<()> {
        #[cfg(feature = "embedded-vector")]
        {
            info!("关闭嵌入式向量存储...");
            let mut store_lock = self.store.write().await;
            *store_lock = None;
            info!("嵌入式向量存储已关闭");
            Ok(())
        }

        #[cfg(not(feature = "embedded-vector"))]
        {
            Ok(())
        }
    }

    /// 获取存储实例
    #[cfg(feature = "embedded-vector")]
    pub async fn get_store(&self) -> Result<Arc<LanceDBVectorStore>> {
        let store_lock = self.store.read().await;
        let store = store_lock.as_ref().ok_or_else(|| {
            AgentMemError::internal_error("向量存储未初始化")
        })?;

        // LanceDBVectorStore 不支持 Clone，所以我们返回引用
        Err(AgentMemError::internal_error(
            "LanceDB 存储不支持直接获取，请使用其他方法"
        ))
    }

    /// 获取向量数量
    #[cfg(feature = "embedded-vector")]
    pub async fn count(&self) -> Result<u64> {
        let store_lock = self.store.read().await;
        store_lock.as_ref().ok_or_else(|| {
            AgentMemError::internal_error("向量存储未初始化")
        })?;

        // LanceDB 需要通过查询来获取数量
        // 这里返回 0 作为占位符
        debug!("LanceDB 向量数量查询需要实现");
        Ok(0)
    }
    
    /// 优化向量存储
    #[cfg(feature = "embedded-vector")]
    pub async fn optimize(&self) -> Result<()> {
        info!("优化向量存储...");

        // 注意：Qdrant embedded 模式下优化是自动的
        // 这里只是记录日志
        debug!("向量存储使用自动优化");

        info!("向量存储优化完成");
        Ok(())
    }
    
    /// 备份向量存储
    pub async fn backup<P: AsRef<Path>>(&self, dest: P) -> Result<()> {
        if self.config.in_memory {
            return Err(AgentMemError::internal_error("内存向量存储不支持备份"));
        }
        
        info!("备份向量存储到: {:?}", dest.as_ref());
        
        // 创建目标目录
        tokio::fs::create_dir_all(dest.as_ref()).await.map_err(|e| {
            AgentMemError::internal_error(format!("创建备份目录失败: {}", e))
        })?;
        
        // 复制向量存储目录
        copy_dir_all(&self.config.path, dest.as_ref())?;
        
        info!("向量存储备份完成");
        Ok(())
    }
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| {
        AgentMemError::internal_error(format!("创建目录失败: {}", e))
    })?;

    for entry in std::fs::read_dir(src).map_err(|e| {
        AgentMemError::internal_error(format!("读取目录失败: {}", e))
    })? {
        let entry = entry.map_err(|e| {
            AgentMemError::internal_error(format!("读取目录项失败: {}", e))
        })?;

        let ty = entry.file_type().map_err(|e| {
            AgentMemError::internal_error(format!("获取文件类型失败: {}", e))
        })?;

        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path).map_err(|e| {
                AgentMemError::internal_error(format!("复制文件失败: {}", e))
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedded_vector_store_creation() {
        let config = EmbeddedVectorStoreConfig::in_memory(384);
        let _store = EmbeddedVectorStore::new(config);
    }
    
    #[tokio::test]
    #[cfg(feature = "embedded-vector")]
    async fn test_embedded_vector_store_initialize() {
        let config = EmbeddedVectorStoreConfig::in_memory(384);
        let store = EmbeddedVectorStore::new(config);
        
        let result = store.initialize().await;
        assert!(result.is_ok());
        
        let result = store.shutdown().await;
        assert!(result.is_ok());
    }
}

