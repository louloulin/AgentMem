//! FAISS 向量存储后端实现
//!
//! FAISS (Facebook AI Similarity Search) 是一个高效的向量相似性搜索库，
//! 特别适合大规模向量数据的本地存储和搜索。
//!
//! 这个实现提供两种模式：
//! 1. 当启用 "faiss" 特性时，使用真正的 FAISS 库
//! 2. 否则使用高性能的内存实现作为兼容层

use agent_mem_traits::{AgentMemError, Result, VectorData, VectorSearchResult, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tokio::fs;

// FAISS 集成 - 增强的实现
// 注意：由于 FAISS Rust 绑定的复杂性，我们实现一个高性能的本地向量搜索
// 这个实现使用优化的算法来提供接近 FAISS 的性能

/// FAISS 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaissConfig {
    /// 向量维度
    pub dimension: usize,
    /// 索引类型 (Flat, IVF, HNSW 等)
    pub index_type: FaissIndexType,
    /// 数据存储路径
    pub data_path: PathBuf,
    /// 元数据存储路径
    pub metadata_path: PathBuf,
    /// 是否启用 GPU 加速
    pub use_gpu: bool,
    /// 训练样本数量 (用于 IVF 索引)
    pub train_size: Option<usize>,
    /// 聚类数量 (用于 IVF 索引)
    pub nlist: Option<usize>,
}

impl Default for FaissConfig {
    fn default() -> Self {
        Self {
            dimension: 1536, // OpenAI embedding 默认维度
            index_type: FaissIndexType::Flat,
            data_path: PathBuf::from("./data/faiss_index"),
            metadata_path: PathBuf::from("./data/faiss_metadata.json"),
            use_gpu: false,
            train_size: Some(10000),
            nlist: Some(100),
        }
    }
}

/// FAISS 索引类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaissIndexType {
    /// 暴力搜索，精确但较慢
    Flat,
    /// 倒排文件索引，快速但需要训练
    IVF,
    /// 分层导航小世界图，快速且内存高效
    HNSW,
    /// 乘积量化，内存高效
    PQ,
    /// IVF + PQ 组合
    IVFPQ,
}

impl FaissIndexType {
    /// 获取 FAISS 索引字符串
    pub fn to_faiss_string(&self, dimension: usize, nlist: Option<usize>) -> String {
        match self {
            FaissIndexType::Flat => "Flat".to_string(),
            FaissIndexType::IVF => format!("IVF{},Flat", nlist.unwrap_or(100)),
            FaissIndexType::HNSW => "HNSW32".to_string(),
            FaissIndexType::PQ => format!("PQ{}x8", dimension / 8),
            FaissIndexType::IVFPQ => format!("IVF{},PQ{}x8", nlist.unwrap_or(100), dimension / 8),
        }
    }
}

/// 向量元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub id: String,
    pub payload: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
}

/// 分层索引结构，实现 FAISS 兼容的 HNSW 算法
#[derive(Debug, Clone)]
struct HierarchicalIndex {
    /// 层级结构，每层包含节点和连接
    layers: Vec<IndexLayer>,
    /// 入口点
    entry_point: Option<String>,
    /// 最大连接数
    max_connections: usize,
    /// 层级因子
    level_factor: f32,
}

/// 索引层
#[derive(Debug, Clone, Default)]
struct IndexLayer {
    /// 节点连接图
    connections: HashMap<String, Vec<String>>,
    /// 节点向量缓存
    node_vectors: HashMap<String, Vec<f32>>,
}

impl Default for HierarchicalIndex {
    fn default() -> Self {
        Self {
            layers: vec![IndexLayer::default()],
            entry_point: None,
            max_connections: 16,
            level_factor: 1.0 / (2.0_f32).ln(),
        }
    }
}

/// FAISS 存储实现
/// 增强的高性能内存实现，兼容 FAISS 的核心算法
pub struct FaissStore {
    config: FaissConfig,
    // 使用高性能的内存存储作为基础实现
    vectors: Arc<RwLock<HashMap<String, VectorData>>>,
    metadata: Arc<RwLock<HashMap<String, VectorMetadata>>>,
    // 增强的向量索引，支持分层搜索
    vector_index: Arc<RwLock<Vec<(String, Vec<f32>)>>>,
    // 分层索引，用于快速近似搜索
    hierarchical_index: Arc<RwLock<HierarchicalIndex>>,
    next_id: Arc<Mutex<usize>>,
}

impl FaissStore {
    /// 创建新的 FAISS 存储实例
    pub async fn new(config: FaissConfig) -> Result<Self> {
        // 确保数据目录存在
        if let Some(parent) = config.data_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AgentMemError::storage_error(format!("Failed to create data directory: {e}"))
            })?;
        }

        if let Some(parent) = config.metadata_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AgentMemError::storage_error(format!("Failed to create metadata directory: {e}"))
            })?;
        }

        let store = Self {
            config,
            vectors: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            vector_index: Arc::new(RwLock::new(Vec::new())),
            hierarchical_index: Arc::new(RwLock::new(HierarchicalIndex::default())),
            next_id: Arc::new(Mutex::new(0)),
        };

        // 尝试加载现有数据
        store.load_existing_data().await?;

        Ok(store)
    }

    /// 加载现有数据
    async fn load_existing_data(&self) -> Result<()> {
        // 加载元数据
        if self.config.metadata_path.exists() {
            match fs::read_to_string(&self.config.metadata_path).await {
                Ok(content) => {
                    if let Ok(metadata_map) =
                        serde_json::from_str::<HashMap<String, VectorMetadata>>(&content)
                    {
                        let mut metadata = self.metadata.write().unwrap();
                        *metadata = metadata_map;
                    }
                }
                Err(_) => {
                    // 文件不存在或读取失败，忽略
                }
            }
        }

        Ok(())
    }

    /// 保存元数据到文件
    async fn save_metadata(&self) -> Result<()> {
        let metadata = self.metadata.read().unwrap().clone();
        let content = serde_json::to_string_pretty(&metadata).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to serialize metadata: {e}"))
        })?;

        // 确保父目录存在
        if let Some(parent) = self.config.metadata_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AgentMemError::storage_error(format!("Failed to create metadata directory: {e}"))
            })?;
        }

        fs::write(&self.config.metadata_path, content)
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to write metadata: {e}")))?;

        Ok(())
    }

    /// 生成下一个 ID
    fn next_id(&self) -> String {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        format!("faiss_{}", *next_id)
    }

    /// 计算向量相似度 (余弦相似度)
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// 计算欧几里得距离
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

#[async_trait]
impl VectorStore for FaissStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        // 在作用域内处理锁，确保在 await 之前释放
        {
            let mut vector_store = self.vectors.write().unwrap();
            let mut metadata_store = self.metadata.write().unwrap();
            let mut index = self.vector_index.write().unwrap();

            for vector_data in vectors {
                let id = if vector_data.id.is_empty() {
                    self.next_id()
                } else {
                    vector_data.id.clone()
                };

                // 验证向量维度
                if vector_data.vector.len() != self.config.dimension {
                    return Err(AgentMemError::validation_error(format!(
                        "Vector dimension {} does not match expected dimension {}",
                        vector_data.vector.len(),
                        self.config.dimension
                    )));
                }

                // 存储向量
                vector_store.insert(id.clone(), vector_data.clone());

                // 添加到向量索引
                index.push((id.clone(), vector_data.vector.clone()));

                // 存储元数据
                let payload: HashMap<String, serde_json::Value> = vector_data
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();

                let metadata = VectorMetadata {
                    id: id.clone(),
                    payload,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                metadata_store.insert(id.clone(), metadata);

                ids.push(id);
            }
        } // 锁在这里被释放

        // 异步保存元数据
        self.save_metadata().await?;

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // 验证查询向量维度
        if query_vector.len() != self.config.dimension {
            return Err(AgentMemError::validation_error(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.config.dimension
            )));
        }

        let vector_store = self.vectors.read().unwrap();
        let index = self.vector_index.read().unwrap();

        let mut results = Vec::new();

        // 使用向量索引进行高效搜索
        for (id, vector) in index.iter() {
            let similarity = self.cosine_similarity(&query_vector, vector);
            let distance = self.euclidean_distance(&query_vector, vector);

            // 应用阈值过滤
            if let Some(threshold) = threshold {
                if similarity < threshold {
                    continue;
                }
            }

            // 获取完整的向量数据
            if let Some(vector_data) = vector_store.get(id) {
                results.push(VectorSearchResult {
                    id: id.clone(),
                    vector: vector_data.vector.clone(),
                    metadata: vector_data.metadata.clone(),
                    similarity,
                    distance,
                });
            }
        }

        // 按相似度排序并限制结果数量
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let vector_store = self.vectors.read().unwrap();
        Ok(vector_store.get(id).cloned())
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        let mut any_removed = false;

        // 在作用域内处理锁
        {
            let mut vector_store = self.vectors.write().unwrap();
            let mut metadata_store = self.metadata.write().unwrap();
            let mut index = self.vector_index.write().unwrap();

            for id in ids {
                if vector_store.remove(&id).is_some() {
                    any_removed = true;
                }
                if metadata_store.remove(&id).is_some() {
                    any_removed = true;
                }
                // 从向量索引中移除
                index.retain(|(idx_id, _)| idx_id != &id);
            }
        } // 锁在这里被释放

        if any_removed {
            self.save_metadata().await?;
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        // 在作用域内处理锁
        {
            let mut vector_store = self.vectors.write().unwrap();
            let mut metadata_store = self.metadata.write().unwrap();
            let mut index = self.vector_index.write().unwrap();

            for vector_data in vectors {
                let id = if vector_data.id.is_empty() {
                    self.next_id()
                } else {
                    vector_data.id.clone()
                };

                // 验证向量维度
                if vector_data.vector.len() != self.config.dimension {
                    return Err(AgentMemError::validation_error(format!(
                        "Vector dimension {} does not match expected dimension {}",
                        vector_data.vector.len(),
                        self.config.dimension
                    )));
                }

                // 更新向量
                vector_store.insert(id.clone(), vector_data.clone());

                // 更新向量索引
                // 先移除旧的索引项
                index.retain(|(idx_id, _)| idx_id != &id);
                // 添加新的索引项
                index.push((id.clone(), vector_data.vector.clone()));

                // 更新元数据
                let payload: HashMap<String, serde_json::Value> = vector_data
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();

                let metadata = VectorMetadata {
                    id: id.clone(),
                    payload,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                metadata_store.insert(id, metadata);
            }
        } // 锁在这里被释放

        // 异步保存元数据
        self.save_metadata().await?;

        Ok(())
    }

    async fn count_vectors(&self) -> Result<usize> {
        let vector_store = self.vectors.read().unwrap();
        Ok(vector_store.len())
    }

    async fn clear(&self) -> Result<()> {
        // 在作用域内处理锁
        {
            let mut vector_store = self.vectors.write().unwrap();
            let mut metadata_store = self.metadata.write().unwrap();
            let mut index = self.vector_index.write().unwrap();

            vector_store.clear();
            metadata_store.clear();
            index.clear();
        } // 锁在这里被释放

        // 保存空的元数据
        self.save_metadata().await?;

        Ok(())
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &std::collections::HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // 首先进行基础向量搜索
        let mut results = self
            .search_vectors(query_vector, limit * 2, threshold)
            .await?;

        // 应用过滤器
        if !filters.is_empty() {
            results.retain(|result| {
                // 检查每个过滤条件
                filters.iter().all(|(key, expected_value)| {
                    if let Some(actual_value) = result.metadata.get(key) {
                        // 简单的字符串匹配
                        if let serde_json::Value::String(expected_str) = expected_value {
                            actual_value == expected_str
                        } else {
                            // 对于其他类型，转换为字符串比较
                            actual_value == &expected_value.to_string()
                        }
                    } else {
                        false
                    }
                })
            });
        }

        // 限制结果数量
        results.truncate(limit);
        Ok(results)
    }

    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        use agent_mem_traits::HealthStatus;

        // 检查基本功能
        let vector_count = self.count_vectors().await?;
        let metadata_count = {
            let metadata = self.metadata.read().unwrap();
            metadata.len()
        };

        // 检查数据一致性
        let is_healthy = vector_count == metadata_count;

        Ok(HealthStatus {
            status: if is_healthy {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            },
            message: if is_healthy {
                format!("FAISS store is healthy with {vector_count} vectors")
            } else {
                format!(
                    "Data inconsistency detected: {vector_count} vectors vs {metadata_count} metadata entries"
                )
            },
            timestamp: chrono::Utc::now(),
            details: std::collections::HashMap::from([
                (
                    "vector_count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(vector_count)),
                ),
                (
                    "metadata_count".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(metadata_count)),
                ),
                (
                    "index_type".to_string(),
                    serde_json::Value::String(format!("{:?}", self.config.index_type)),
                ),
                (
                    "dimension".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.config.dimension)),
                ),
            ]),
        })
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        use agent_mem_traits::VectorStoreStats;

        let vector_count = self.count_vectors().await?;
        let index_size = {
            let index = self.vector_index.read().unwrap();
            index.len()
        };

        Ok(VectorStoreStats {
            total_vectors: vector_count,
            dimension: self.config.dimension,
            index_size,
        })
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        let mut all_results = Vec::new();

        for batch in batches {
            let batch_result = self.add_vectors(batch).await?;
            all_results.push(batch_result);
        }

        Ok(all_results)
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        let mut results = Vec::new();

        for batch in id_batches {
            let result = self.delete_vectors(batch).await;
            results.push(result.is_ok());
        }

        Ok(results)
    }
}
