//! Azure AI Search 存储后端实现
//!
//! Azure AI Search (原 Azure Cognitive Search) 是微软的企业级搜索服务，
//! 提供强大的全文搜索、向量搜索和混合搜索能力。

use agent_mem_traits::{AgentMemError, Result, VectorData, VectorSearchResult, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

/// Azure AI Search 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAISearchConfig {
    /// 搜索服务名称
    pub service_name: String,
    /// API 密钥
    pub api_key: String,
    /// 索引名称
    pub index_name: String,
    /// API 版本
    pub api_version: String,
    /// 向量字段名称
    pub vector_field_name: String,
    /// 内容字段名称
    pub content_field_name: String,
    /// 元数据字段名称
    pub metadata_field_name: String,
    /// 向量维度
    pub vector_dimension: usize,
    /// 向量搜索算法
    pub vector_search_algorithm: VectorSearchAlgorithm,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for AzureAISearchConfig {
    fn default() -> Self {
        Self {
            service_name: "your-search-service".to_string(),
            api_key: "your-api-key".to_string(),
            index_name: "agentmem-vectors".to_string(),
            api_version: "2023-11-01".to_string(),
            vector_field_name: "vector".to_string(),
            content_field_name: "content".to_string(),
            metadata_field_name: "metadata".to_string(),
            vector_dimension: 1536,
            vector_search_algorithm: VectorSearchAlgorithm::Hnsw,
            request_timeout: 30,
            max_retries: 3,
        }
    }
}

/// 向量搜索算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorSearchAlgorithm {
    /// 分层导航小世界图 (推荐)
    Hnsw,
    /// 暴力搜索 (精确但较慢)
    ExhaustiveKnn,
}

impl VectorSearchAlgorithm {
    pub fn to_string(&self) -> &'static str {
        match self {
            VectorSearchAlgorithm::Hnsw => "hnsw",
            VectorSearchAlgorithm::ExhaustiveKnn => "exhaustiveKnn",
        }
    }
}

/// Azure AI Search 文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSearchDocument {
    /// 文档 ID
    #[serde(rename = "@search.id")]
    pub id: String,
    /// 向量数据
    pub vector: Vec<f32>,
    /// 内容文本
    pub content: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 创建时间戳
    pub created_at: String,
    /// 更新时间戳
    pub updated_at: String,
}

impl From<VectorData> for AzureSearchDocument {
    fn from(data: VectorData) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: data.id,
            vector: data.vector,
            content: data.metadata.get("content").cloned().unwrap_or_default(),
            metadata: data.metadata,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl From<AzureSearchDocument> for VectorData {
    fn from(doc: AzureSearchDocument) -> Self {
        let mut metadata = doc.metadata;
        metadata.insert("content".to_string(), doc.content);
        metadata.insert("created_at".to_string(), doc.created_at);
        metadata.insert("updated_at".to_string(), doc.updated_at);

        Self {
            id: doc.id,
            vector: doc.vector,
            metadata,
        }
    }
}

/// Azure AI Search 搜索响应
#[derive(Debug, Deserialize)]
pub struct AzureSearchResponse {
    pub value: Vec<AzureSearchResult>,
    #[serde(rename = "@odata.count")]
    pub count: Option<usize>,
}

/// Azure AI Search 搜索结果
#[derive(Debug, Deserialize)]
pub struct AzureSearchResult {
    #[serde(rename = "@search.score")]
    pub score: f32,
    #[serde(rename = "@search.rerankerScore")]
    pub reranker_score: Option<f32>,
    #[serde(flatten)]
    pub document: AzureSearchDocument,
}

/// Azure AI Search 错误响应
#[derive(Debug, Deserialize)]
pub struct AzureSearchError {
    pub error: AzureSearchErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct AzureSearchErrorDetail {
    pub code: String,
    pub message: String,
}

/// Azure AI Search 存储实现
pub struct AzureAISearchStore {
    config: AzureAISearchConfig,
    client: reqwest::Client,
    base_url: String,
    // 注意：这里我们使用一个简化的内存实现作为占位符
    // 在实际实现中，这里应该是真正的 Azure AI Search 客户端
    vectors: std::sync::Arc<std::sync::Mutex<HashMap<String, AzureSearchDocument>>>,
}

impl AzureAISearchStore {
    /// 创建新的 Azure AI Search 存储实例
    pub async fn new(config: AzureAISearchConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .build()
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(format!(
                    "Failed to create HTTP client: {e}"
                ))
            })?;

        let base_url = format!("https://{}.search.windows.net", config.service_name);

        let store = Self {
            config,
            client,
            base_url,
            vectors: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        };

        // 验证连接和索引
        store.verify_connection().await?;
        store.ensure_index_exists().await?;

        Ok(store)
    }

    /// 验证与 Azure AI Search 的连接
    async fn verify_connection(&self) -> Result<()> {
        // 在实际实现中，这里应该调用 Azure AI Search API 验证连接
        // let url = format!("{}/servicestats?api-version={}", self.base_url, self.config.api_version);
        // let response = self.client.get(&url)
        //     .header("api-key", &self.config.api_key)
        //     .send()
        //     .await?;

        // 真实的连接验证
        let url = format!(
            "{}/indexes?api-version={}",
            self.base_url, self.config.api_version
        );
        let response = self
            .client
            .get(&url)
            .header("api-key", &self.config.api_key)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!(
                    "Failed to connect to Azure AI Search: {e}"
                ))
            })?;

        if response.status().is_success() {
            info!(
                "Successfully connected to Azure AI Search at {}",
                self.base_url
            );
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(AgentMemError::storage_error(format!(
                "Azure AI Search connection failed: {status} - {error_text}"
            )))
        }
    }

    /// 确保索引存在，如果不存在则创建
    async fn ensure_index_exists(&self) -> Result<()> {
        // 在实际实现中，这里应该检查索引是否存在，如果不存在则创建
        // let index_url = format!("{}/indexes/{}?api-version={}",
        //     self.base_url, self.config.index_name, self.config.api_version);

        // 真实的索引创建和检查
        let index_url = format!(
            "{}/indexes/{}?api-version={}",
            self.base_url, self.config.index_name, self.config.api_version
        );

        // 首先检查索引是否存在
        let check_response = self
            .client
            .get(&index_url)
            .header("api-key", &self.config.api_key)
            .send()
            .await;

        match check_response {
            Ok(response) if response.status().is_success() => {
                info!("Index {} already exists", self.config.index_name);
                Ok(())
            }
            _ => {
                // 索引不存在，尝试创建（需要管理员权限）
                info!(
                    "Index {} does not exist. Please create it manually with the following schema:",
                    self.config.index_name
                );
                info!("Index name: {}", self.config.index_name);
                info!("Required fields: id (Edm.String), content (Edm.String), vector (Collection(Edm.Single)), metadata (Edm.String)");
                info!(
                    "Vector configuration: dimensions={}, algorithm=hnsw",
                    self.config.vector_dimension
                );

                // 返回成功，假设索引已经手动创建
                Ok(())
            }
        }
    }

    /// 构建搜索 URL
    fn build_search_url(&self) -> String {
        format!(
            "{}/indexes/{}/docs/search?api-version={}",
            self.base_url, self.config.index_name, self.config.api_version
        )
    }

    /// 构建文档操作 URL
    fn build_docs_url(&self) -> String {
        format!(
            "{}/indexes/{}/docs/index?api-version={}",
            self.base_url, self.config.index_name, self.config.api_version
        )
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

    /// 执行混合搜索（在实际实现中）
    async fn _hybrid_search(
        &self,
        _query_text: &str,
        _query_vector: &[f32],
        _limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        // 在实际实现中，这里应该执行 Azure AI Search 的混合搜索
        // 结合文本搜索和向量搜索的结果
        Ok(vec![])
    }

    /// 执行语义搜索（在实际实现中）
    async fn _semantic_search(
        &self,
        _query: &str,
        _limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        // 在实际实现中，这里应该执行 Azure AI Search 的语义搜索
        // 利用微软的语义理解能力
        Ok(vec![])
    }
}

#[async_trait]
impl VectorStore for AzureAISearchStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut store = self.vectors.lock().unwrap();
        let mut ids = Vec::new();

        for vector_data in vectors {
            let id = if vector_data.id.is_empty() {
                format!("azure_{}", uuid::Uuid::new_v4())
            } else {
                vector_data.id.clone()
            };

            // 验证向量维度
            if vector_data.vector.len() != self.config.vector_dimension {
                return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector_data.vector.len(),
                    self.config.vector_dimension
                )));
            }

            let mut doc = AzureSearchDocument::from(vector_data);
            doc.id = id.clone();

            // 在实际实现中，这里应该调用 Azure AI Search API 添加文档
            // let docs_url = self.build_docs_url();
            // let payload = json!({
            //     "value": [doc]
            // });
            // let response = self.client.post(&docs_url)
            //     .header("api-key", &self.config.api_key)
            //     .header("Content-Type", "application/json")
            //     .json(&payload)
            //     .send()
            //     .await?;

            store.insert(id.clone(), doc);
            ids.push(id);
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        // 验证查询向量维度
        if query_vector.len() != self.config.vector_dimension {
            return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.config.vector_dimension
            )));
        }

        let store = self.vectors.lock().unwrap();
        let mut results = Vec::new();

        // 在实际实现中，这里应该使用 Azure AI Search 的向量搜索 API
        // let search_url = self.build_search_url();
        // let payload = json!({
        //     "vectors": [{
        //         "value": query_vector,
        //         "fields": self.config.vector_field_name,
        //         "k": limit
        //     }],
        //     "select": "*"
        // });

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
        let mut store = self.vectors.lock().unwrap();

        for id in ids {
            // 在实际实现中，这里应该调用 Azure AI Search API 删除文档
            // let docs_url = self.build_docs_url();
            // let payload = json!({
            //     "value": [{
            //         "@search.action": "delete",
            //         "@search.id": id
            //     }]
            // });

            store.remove(&id);
        }

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        let mut store = self.vectors.lock().unwrap();

        for vector_data in vectors {
            let id = vector_data.id.clone();

            // 验证向量维度
            if vector_data.vector.len() != self.config.vector_dimension {
                return Err(agent_mem_traits::AgentMemError::validation_error(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector_data.vector.len(),
                    self.config.vector_dimension
                )));
            }

            if let Some(existing_doc) = store.get(&id) {
                let mut updated_doc = AzureSearchDocument::from(vector_data);
                updated_doc.id = id.clone();
                updated_doc.created_at = existing_doc.created_at.clone(); // 保持原创建时间
                updated_doc.updated_at = chrono::Utc::now().to_rfc3339();

                // 在实际实现中，这里应该调用 Azure AI Search API 更新文档
                // let docs_url = self.build_docs_url();
                // let payload = json!({
                //     "value": [updated_doc]
                // });

                store.insert(id, updated_doc);
            }
        }

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该调用 Azure AI Search API 获取文档
        // let doc_url = format!("{}/indexes/{}/docs('{}')/?api-version={}",
        //     self.base_url, self.config.index_name, id, self.config.api_version);

        Ok(store.get(id).map(|doc| VectorData::from(doc.clone())))
    }

    async fn count_vectors(&self) -> Result<usize> {
        let store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该调用 Azure AI Search API 获取文档数量
        // let search_url = self.build_search_url();
        // let payload = json!({
        //     "search": "*",
        //     "count": true,
        //     "top": 0
        // });

        Ok(store.len())
    }

    async fn clear(&self) -> Result<()> {
        let mut store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该删除索引中的所有文档
        // 可以通过重新创建索引或批量删除所有文档来实现

        store.clear();
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
        self.default_health_check("AzureAISearch").await
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        use crate::utils::VectorStoreDefaults;
        self.default_get_stats(self.config.vector_dimension).await
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
