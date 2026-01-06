//! MongoDB 文档存储后端实现
//!
//! MongoDB 是一个高性能的 NoSQL 文档数据库，特别适合存储结构化的记忆数据。
//! 支持复杂查询、聚合操作和水平扩展。

use agent_mem_traits::{Result, VectorData, VectorSearchResult, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "mongodb")]
use bson::{doc, Document};
#[cfg(feature = "mongodb")]
use futures::stream::StreamExt;
#[cfg(feature = "mongodb")]
use mongodb::{Client, Collection, Database};

/// MongoDB 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    /// 连接字符串
    pub connection_string: String,
    /// 数据库名称
    pub database_name: String,
    /// 集合名称
    pub collection_name: String,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
    /// 查询超时时间（秒）
    pub query_timeout: u64,
    /// 最大连接池大小
    pub max_pool_size: u32,
    /// 最小连接池大小
    pub min_pool_size: u32,
    /// 是否启用 TLS
    pub enable_tls: bool,
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            connection_string: "mongodb://localhost:27017".to_string(),
            database_name: "agentmem".to_string(),
            collection_name: "vectors".to_string(),
            connection_timeout: 30,
            query_timeout: 30,
            max_pool_size: 10,
            min_pool_size: 1,
            enable_tls: false,
        }
    }
}

/// MongoDB 文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoVectorDocument {
    /// 文档 ID
    #[serde(rename = "_id")]
    pub id: String,
    /// 向量数据
    pub vector: Vec<f32>,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 创建时间戳
    pub created_at: i64,
    /// 更新时间戳
    pub updated_at: i64,
    /// 向量维度
    pub dimension: usize,
}

impl From<VectorData> for MongoVectorDocument {
    fn from(data: VectorData) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: data.id,
            dimension: data.vector.len(),
            vector: data.vector,
            metadata: data.metadata,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<MongoVectorDocument> for VectorData {
    fn from(doc: MongoVectorDocument) -> Self {
        Self {
            id: doc.id,
            vector: doc.vector,
            metadata: doc.metadata,
        }
    }
}

/// MongoDB 搜索结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MongoSearchResult {
    /// 文档
    pub document: MongoVectorDocument,
    /// 相似度分数
    pub similarity: f32,
    /// 距离
    pub distance: f32,
}

/// MongoDB 存储实现
pub struct MongoDBStore {
    config: MongoDBConfig,
    #[cfg(feature = "mongodb")]
    client: Client,
    #[cfg(feature = "mongodb")]
    database: Database,
    #[cfg(feature = "mongodb")]
    collection: Collection<MongoVectorDocument>,
    #[cfg(not(feature = "mongodb"))]
    // 当没有 MongoDB 特性时，使用内存实现作为占位符
    vectors: std::sync::Arc<std::sync::Mutex<HashMap<String, MongoVectorDocument>>>,
}

impl MongoDBStore {
    /// 创建新的 MongoDB 存储实例
    pub async fn new(config: MongoDBConfig) -> Result<Self> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB 客户端
            let client = Client::with_uri_str(&config.connection_string)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to connect to MongoDB: {}",
                        e
                    ))
                })?;

            let database = client.database(&config.database_name);
            let collection = database.collection::<MongoVectorDocument>(&config.collection_name);

            let store = Self {
                config,
                client,
                database,
                collection,
            };

            // 验证连接
            store.verify_connection().await?;
            Ok(store)
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let store = Self {
                config,
                vectors: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            };

            // 验证连接
            store.verify_connection().await?;
            Ok(store)
        }
    }

    /// 验证 MongoDB 连接
    async fn verify_connection(&self) -> Result<()> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB ping 命令
            self.client
                .database("admin")
                .run_command(doc! {"ping": 1}, None)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "MongoDB ping failed: {}",
                        e
                    ))
                })?;
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 本地连接验证（无 MongoDB 特性时）
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(())
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

    /// 创建索引（在实际实现中）
    async fn _create_indexes(&self) -> Result<()> {
        // 在实际实现中，这里应该创建必要的索引
        // 例如：向量字段的索引、元数据字段的索引等
        Ok(())
    }

    /// 执行聚合查询（在实际实现中）
    async fn _aggregate_search(
        &self,
        _pipeline: Vec<serde_json::Value>,
    ) -> Result<Vec<MongoSearchResult>> {
        // 在实际实现中，这里应该执行 MongoDB 聚合管道
        // 用于复杂的向量搜索和过滤
        Ok(vec![])
    }
}

#[async_trait]
impl VectorStore for MongoDBStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB 操作
            let mut documents = Vec::new();

            for vector_data in vectors {
                let id = if vector_data.id.is_empty() {
                    format!("mongo_{}", uuid::Uuid::new_v4())
                } else {
                    vector_data.id.clone()
                };

                let mut doc = MongoVectorDocument::from(vector_data);
                doc.id = id.clone();

                documents.push(doc);
                ids.push(id);
            }

            // 批量插入到 MongoDB
            if !documents.is_empty() {
                self.collection
                    .insert_many(documents, None)
                    .await
                    .map_err(|e| {
                        agent_mem_traits::AgentMemError::storage_error(&format!(
                            "Failed to insert vectors: {}",
                            e
                        ))
                    })?;
            }
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let mut store = self.vectors.lock().unwrap();

            for vector_data in vectors {
                let id = if vector_data.id.is_empty() {
                    format!("mongo_{}", uuid::Uuid::new_v4())
                } else {
                    vector_data.id.clone()
                };

                let mut doc = MongoVectorDocument::from(vector_data);
                doc.id = id.clone();

                store.insert(id.clone(), doc);
                ids.push(id);
            }
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let mut results = Vec::new();

        #[cfg(feature = "mongodb")]
        {
            // 使用 MongoDB 查询所有文档，然后在内存中计算相似度
            // 在生产环境中，可以使用 MongoDB Atlas Vector Search 或其他向量搜索解决方案
            let cursor = self.collection.find(None, None).await.map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to query vectors: {}",
                    e
                ))
            })?;

            let cursor_results = cursor.collect::<Vec<_>>().await;

            let mut documents = Vec::new();
            for result in cursor_results {
                match result {
                    Ok(doc) => documents.push(doc),
                    Err(e) => {
                        return Err(agent_mem_traits::AgentMemError::storage_error(&format!(
                            "Failed to collect results: {}",
                            e
                        )))
                    }
                }
            }

            for doc in documents {
                let similarity = self.cosine_similarity(&query_vector, &doc.vector);
                let distance = self.euclidean_distance(&query_vector, &doc.vector);

                // 应用阈值过滤
                if let Some(threshold) = threshold {
                    if similarity < threshold {
                        continue;
                    }
                }

                results.push(VectorSearchResult {
                    id: doc.id.clone(),
                    vector: doc.vector.clone(),
                    metadata: doc.metadata.clone(),
                    similarity,
                    distance,
                });
            }
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let store = self.vectors.lock().unwrap();

            for (_, doc) in store.iter() {
                let similarity = self.cosine_similarity(&query_vector, &doc.vector);
                let distance = self.euclidean_distance(&query_vector, &doc.vector);

                // 应用阈值过滤
                if let Some(threshold) = threshold {
                    if similarity < threshold {
                        continue;
                    }
                }

                results.push(VectorSearchResult {
                    id: doc.id.clone(),
                    vector: doc.vector.clone(),
                    metadata: doc.metadata.clone(),
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

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB 删除操作
            for id in ids {
                self.collection
                    .delete_one(doc! {"_id": &id}, None)
                    .await
                    .map_err(|e| {
                        agent_mem_traits::AgentMemError::storage_error(&format!(
                            "Failed to delete vector {}: {}",
                            id, e
                        ))
                    })?;
            }
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let mut store = self.vectors.lock().unwrap();
            for id in ids {
                store.remove(&id);
            }
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB 更新操作
            for vector_data in vectors {
                let id = vector_data.id.clone();

                // 先查找现有文档以保持创建时间
                if let Ok(Some(existing_doc)) =
                    self.collection.find_one(doc! {"_id": &id}, None).await
                {
                    let mut updated_doc = MongoVectorDocument::from(vector_data);
                    updated_doc.id = id.clone();
                    updated_doc.created_at = existing_doc.created_at; // 保持原创建时间
                    updated_doc.updated_at = chrono::Utc::now().timestamp();

                    self.collection
                        .replace_one(doc! {"_id": &id}, &updated_doc, None)
                        .await
                        .map_err(|e| {
                            agent_mem_traits::AgentMemError::storage_error(&format!(
                                "Failed to update vector {}: {}",
                                id, e
                            ))
                        })?;
                }
            }
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let mut store = self.vectors.lock().unwrap();

            for vector_data in vectors {
                let id = vector_data.id.clone();

                if let Some(existing_doc) = store.get(&id) {
                    let mut updated_doc = MongoVectorDocument::from(vector_data);
                    updated_doc.id = id.clone();
                    updated_doc.created_at = existing_doc.created_at; // 保持原创建时间
                    updated_doc.updated_at = chrono::Utc::now().timestamp();

                    store.insert(id, updated_doc);
                }
            }
        }

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB 查询
            let doc = self
                .collection
                .find_one(doc! {"_id": id}, None)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to get vector {}: {}",
                        id, e
                    ))
                })?;

            Ok(doc.map(|d| VectorData::from(d)))
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let store = self.vectors.lock().unwrap();
            Ok(store.get(id).map(|doc| VectorData::from(doc.clone())))
        }
    }

    async fn count_vectors(&self) -> Result<usize> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB count_documents
            let count = self
                .collection
                .count_documents(doc! {}, None)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to count vectors: {}",
                        e
                    ))
                })?;

            Ok(count as usize)
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let store = self.vectors.lock().unwrap();
            Ok(store.len())
        }
    }

    async fn clear(&self) -> Result<()> {
        #[cfg(feature = "mongodb")]
        {
            // 使用真正的 MongoDB delete_many
            self.collection
                .delete_many(doc! {}, None)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to clear vectors: {}",
                        e
                    ))
                })?;
        }

        #[cfg(not(feature = "mongodb"))]
        {
            // 使用内存实现作为占位符
            let mut store = self.vectors.lock().unwrap();
            store.clear();
        }

        Ok(())
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &std::collections::HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        use crate::utils::VectorStoreDefaults;
        self.default_search_with_filters(query_vector, limit, filters, threshold)
            .await
    }

    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        use crate::utils::VectorStoreDefaults;
        self.default_health_check("MongoDB").await
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        use crate::utils::VectorStoreDefaults;
        self.default_get_stats(1536).await // 默认维度
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        use crate::utils::VectorStoreDefaults;
        self.default_add_vectors_batch(batches).await
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        use crate::utils::VectorStoreDefaults;
        self.default_delete_vectors_batch(id_batches).await
    }
}
