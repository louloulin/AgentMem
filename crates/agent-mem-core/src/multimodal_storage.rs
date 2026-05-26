// Multimodal Storage - 多模态记忆存储
//
// 支持图像记忆的存储和向量生成，为多模态检索奠定基础

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 多模态记忆类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultimodalType {
    /// 图像
    Image,
    /// 音频
    Audio,
    /// 视频
    Video,
}

impl Default for MultimodalType {
    fn default() -> Self {
        MultimodalType::Image
    }
}

/// 多模态记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalMemory {
    /// 记忆 ID
    pub id: String,
    /// 记忆类型
    pub memory_type: MultimodalType,
    /// 原始数据 (Base64 编码)
    pub data: String,
    /// MIME 类型
    pub mime_type: String,
    /// 向量嵌入 (如果已生成)
    pub embedding: Option<Vec<f32>>,
    /// 元数据
    pub metadata: MultimodalMetadata,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 多模态元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultimodalMetadata {
    /// 原始文件名
    pub filename: Option<String>,
    /// 文件大小 (字节)
    pub size_bytes: Option<u64>,
    /// 宽度 (图像/视频)
    pub width: Option<u32>,
    /// 高度 (图像/视频)
    pub height: Option<u32>,
    /// 持续时间 (音频/视频，毫秒)
    pub duration_ms: Option<u64>,
    /// 标签
    pub tags: Vec<String>,
    /// 自定义属性
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

/// 图像向量生成器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVectorizerConfig {
    /// 模型名称 (如 "clip", "siglip")
    pub model_name: String,
    /// 嵌入维度
    pub embedding_dim: usize,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否使用 GPU
    pub use_gpu: bool,
}

impl Default for ImageVectorizerConfig {
    fn default() -> Self {
        Self {
            model_name: "clip-vit-base-patch32".to_string(),
            embedding_dim: 512,
            batch_size: 8,
            use_gpu: false,
        }
    }
}

/// 图像向量生成器 (接口定义)
pub trait ImageVectorizer: Send + Sync {
    /// 生成图像向量
    fn vectorize(&self, image_data: &[u8]) -> Result<Vec<f32>, MultimodalError>;
    
    /// 批量生成向量
    fn vectorize_batch(&self, images: Vec<Vec<u8>>) -> Result<Vec<Vec<f32>>, MultimodalError>;
}

/// 多模态存储引擎
pub struct MultimodalStorage {
    /// 存储后端
    storage: Arc<dyn MultimodalStorageBackend>,
    /// 向量化器
    vectorizer: Arc<dyn ImageVectorizer>,
    /// 配置
    config: MultimodalStorageConfig,
    /// 统计信息
    stats: Arc<RwLock<MultimodalStats>>,
}

/// 多模态存储后端 (接口)
pub trait MultimodalStorageBackend: Send + Sync {
    /// 存储多模态记忆
    fn store(&self, memory: &MultimodalMemory) -> Result<String, MultimodalError>;
    
    /// 获取多模态记忆
    fn get(&self, id: &str) -> Result<Option<MultimodalMemory>, MultimodalError>;
    
    /// 搜索相似记忆
    fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<MultimodalSearchResult>, MultimodalError>;
    
    /// 删除记忆
    fn delete(&self, id: &str) -> Result<bool, MultimodalError>;
    
    /// 列出所有记忆
    fn list(&self, memory_type: Option<MultimodalType>) -> Result<Vec<String>, MultimodalError>;
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalSearchResult {
    /// 记忆 ID
    pub id: String,
    /// 相似度分数
    pub score: f32,
    /// 记忆类型
    pub memory_type: MultimodalType,
    /// 预览数据 (缩略图等)
    pub preview: Option<String>,
}

/// 多模态存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalStorageConfig {
    /// 最大存储大小 (字节)
    pub max_storage_bytes: u64,
    /// 最大文件大小 (字节)
    pub max_file_size_bytes: u64,
    /// 支持的 MIME 类型
    pub supported_mime_types: Vec<String>,
    /// 是否自动生成向量
    pub auto_vectorize: bool,
    /// 缩略图大小
    pub thumbnail_size: u32,
}

impl Default for MultimodalStorageConfig {
    fn default() -> Self {
        Self {
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            max_file_size_bytes: 100 * 1024 * 1024, // 100MB
            supported_mime_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/svg+xml".to_string(),
            ],
            auto_vectorize: true,
            thumbnail_size: 256,
        }
    }
}

/// 统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultimodalStats {
    /// 总存储项数
    pub total_items: u64,
    /// 总存储大小 (字节)
    pub total_size_bytes: u64,
    /// 按类型统计
    pub by_type: std::collections::HashMap<String, u64>,
    /// 向量生成次数
    pub vectorizations: u64,
}

/// 多模态错误
#[derive(Debug, thiserror::Error)]
pub enum MultimodalError {
    /// 存储错误
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// 向量化错误
    #[error("Vectorization error: {0}")]
    Vectorization(String),
    
    /// 无效输入
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// 不支持的格式
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    /// 未找到
    #[error("Not found: {0}")]
    NotFound(String),
}

impl MultimodalStorage {
    /// 创建新的存储引擎
    pub fn new(
        storage: Arc<dyn MultimodalStorageBackend>,
        vectorizer: Arc<dyn ImageVectorizer>,
        config: MultimodalStorageConfig,
    ) -> Self {
        Self {
            storage,
            vectorizer,
            config,
            stats: Arc::new(RwLock::new(MultimodalStats::default())),
        }
    }

    /// 存储图像记忆
    pub async fn store_image(
        &self,
        data: Vec<u8>,
        mime_type: &str,
        metadata: MultimodalMetadata,
    ) -> Result<String, MultimodalError> {
        // 验证 MIME 类型
        if !self.config.supported_mime_types.contains(&mime_type.to_string()) {
            return Err(MultimodalError::UnsupportedFormat(format!(
                "MIME type {} is not supported",
                mime_type
            )));
        }

        // 验证文件大小
        if data.len() as u64 > self.config.max_file_size_bytes {
            return Err(MultimodalError::InvalidInput(format!(
                "File size {} exceeds maximum {}",
                data.len(),
                self.config.max_file_size_bytes
            )));
        }

        // 生成向量 (如果启用)
        let embedding = if self.config.auto_vectorize {
            match self.vectorizer.vectorize(&data) {
                Ok(emb) => {
                    let mut stats = self.stats.write().await;
                    stats.vectorizations += 1;
                    Some(emb)
                }
                Err(e) => {
                    tracing::warn!("Failed to vectorize image: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // 创建记忆项
        let now = chrono::Utc::now();
        let memory = MultimodalMemory {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type: MultimodalType::Image,
            data: base64_encode(&data),
            mime_type: mime_type.to_string(),
            embedding,
            metadata,
            created_at: now,
            updated_at: now,
        };

        // 存储
        let id = self.storage.store(&memory)?;

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_items += 1;
            stats.total_size_bytes += data.len() as u64;
            *stats.by_type.entry("image".to_string()).or_insert(0) += 1;
        }

        Ok(id)
    }

    /// 获取记忆
    pub async fn get(&self, id: &str) -> Result<Option<MultimodalMemory>, MultimodalError> {
        self.storage.get(id)
    }

    /// 搜索相似图像
    pub async fn search_similar(
        &self,
        query_image: &[u8],
        limit: usize,
    ) -> Result<Vec<MultimodalSearchResult>, MultimodalError> {
        // 生成查询向量
        let query_embedding = self.vectorizer.vectorize(query_image)?;
        
        // 搜索
        self.storage.search(&query_embedding, limit)
    }

    /// 搜索相似向量
    pub async fn search_by_embedding(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<MultimodalSearchResult>, MultimodalError> {
        self.storage.search(embedding, limit)
    }

    /// 删除记忆
    pub async fn delete(&self, id: &str) -> Result<bool, MultimodalError> {
        self.storage.delete(id)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> MultimodalStats {
        self.stats.read().await.clone()
    }
}

/// Base64 编码
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

// ============= Mock 实现 (用于测试) =============

/// Mock 图像向量生成器
pub struct MockImageVectorizer {
    dim: usize,
}

impl MockImageVectorizer {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl ImageVectorizer for MockImageVectorizer {
    fn vectorize(&self, _image_data: &[u8]) -> Result<Vec<f32>, MultimodalError> {
        // 生成随机向量作为 mock
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok((0..self.dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
    }

    fn vectorize_batch(&self, images: Vec<Vec<u8>>) -> Result<Vec<Vec<f32>>, MultimodalError> {
        images.into_iter().map(|img| self.vectorize(&img)).collect()
    }
}

/// Mock 存储后端
pub struct InMemoryMultimodalStorage {
    items: Arc<RwLock<std::collections::HashMap<String, MultimodalMemory>>>,
}

impl InMemoryMultimodalStorage {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for InMemoryMultimodalStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MultimodalStorageBackend for InMemoryMultimodalStorage {
    fn store(&self, memory: &MultimodalMemory) -> Result<String, MultimodalError> {
        let id = memory.id.clone();
        let items = self.items.clone();
        let memory = memory.clone();
        tokio::runtime::Handle::current().block_on(async {
            items.write().await.insert(id.clone(), memory);
        });
        Ok(id)
    }

    fn get(&self, id: &str) -> Result<Option<MultimodalMemory>, MultimodalError> {
        let items = self.items.clone();
        let id = id.to_string();
        Ok(tokio::runtime::Handle::current().block_on(async {
            items.read().await.get(&id).cloned()
        }))
    }

    fn search(&self, _embedding: &[f32], limit: usize) -> Result<Vec<MultimodalSearchResult>, MultimodalError> {
        let items = self.items.clone();
        Ok(tokio::runtime::Handle::current().block_on(async {
            items.read().await
                .values()
                .take(limit)
                .filter_map(|m| {
                    m.embedding.as_ref().map(|_| MultimodalSearchResult {
                        id: m.id.clone(),
                        score: 0.8, // Mock score
                        memory_type: m.memory_type,
                        preview: Some(m.data.chars().take(100).collect()),
                    })
                })
                .collect()
        }))
    }

    fn delete(&self, id: &str) -> Result<bool, MultimodalError> {
        let items = self.items.clone();
        let id = id.to_string();
        Ok(tokio::runtime::Handle::current().block_on(async {
            items.write().await.remove(&id).is_some()
        }))
    }

    fn list(&self, memory_type: Option<MultimodalType>) -> Result<Vec<String>, MultimodalError> {
        let items = self.items.clone();
        Ok(tokio::runtime::Handle::current().block_on(async {
            items.read().await
                .values()
                .filter(|m| memory_type.map_or(true, |t| m.memory_type == t))
                .map(|m| m.id.clone())
                .collect()
        }))
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_get() {
        let storage = InMemoryMultimodalStorage::new();
        let vectorizer = MockImageVectorizer::new(512);
        
        let config = MultimodalStorageConfig::default();
        let engine = MultimodalStorage::new(
            Arc::new(storage),
            Arc::new(vectorizer),
            config,
        );
        
        // 创建一个简单的图像数据
        let image_data = vec![0u8; 100];
        let metadata = MultimodalMetadata::default();
        
        let id = engine.store_image(
            image_data,
            "image/png",
            metadata,
        ).await.unwrap();
        
        let retrieved = engine.get(&id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().mime_type, "image/png");
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        let storage = InMemoryMultimodalStorage::new();
        let vectorizer = MockImageVectorizer::new(512);
        
        let config = MultimodalStorageConfig::default();
        let engine = MultimodalStorage::new(
            Arc::new(storage),
            Arc::new(vectorizer),
            config,
        );
        
        let result = engine.store_image(
            vec![0u8; 100],
            "video/quicktime", // 不支持的格式
            MultimodalMetadata::default(),
        ).await;
        
        assert!(result.is_err());
        match result {
            Err(MultimodalError::UnsupportedFormat(_)) => {}
            _ => panic!("Expected UnsupportedFormat error"),
        }
    }

    #[tokio::test]
    async fn test_stats() {
        let storage = InMemoryMultimodalStorage::new();
        let vectorizer = MockImageVectorizer::new(512);
        
        let config = MultimodalStorageConfig::default();
        let engine = MultimodalStorage::new(
            Arc::new(storage),
            Arc::new(vectorizer),
            config,
        );
        
        // 添加几个图像
        for _ in 0..3 {
            engine.store_image(
                vec![0u8; 100],
                "image/png",
                MultimodalMetadata::default(),
            ).await.unwrap();
        }
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_items, 3);
        assert_eq!(stats.vectorizations, 3);
    }
}
