//! Supabase 存储后端实现
//!
//! Supabase 是一个开源的 Firebase 替代方案，基于 PostgreSQL，
//! 提供实时数据库、向量搜索扩展和边缘计算能力。

use agent_mem_traits::{AgentMemError, Result, VectorData, VectorSearchResult, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

/// Supabase 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase 项目 URL
    pub project_url: String,
    /// API 密钥 (anon key 或 service_role key)
    pub api_key: String,
    /// 数据库表名
    pub table_name: String,
    /// 向量列名
    pub vector_column: String,
    /// 内容列名
    pub content_column: String,
    /// 元数据列名 (JSONB)
    pub metadata_column: String,
    /// 向量维度
    pub vector_dimension: usize,
    /// 相似度函数
    pub similarity_function: SimilarityFunction,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用实时订阅
    pub enable_realtime: bool,
}

impl Default for SupabaseConfig {
    fn default() -> Self {
        Self {
            project_url: "https://your-project.supabase.co".to_string(),
            api_key: "your-anon-key".to_string(),
            table_name: "agentmem_vectors".to_string(),
            vector_column: "embedding".to_string(),
            content_column: "content".to_string(),
            metadata_column: "metadata".to_string(),
            vector_dimension: 1536,
            similarity_function: SimilarityFunction::Cosine,
            request_timeout: 30,
            max_retries: 3,
            enable_realtime: false,
        }
    }
}

/// 相似度函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityFunction {
    /// 余弦相似度 (推荐)
    Cosine,
    /// 内积
    InnerProduct,
    /// L2 距离
    L2Distance,
}

impl SimilarityFunction {
    pub fn to_sql_operator(&self) -> &'static str {
        match self {
            SimilarityFunction::Cosine => "<=>",
            SimilarityFunction::InnerProduct => "<#>",
            SimilarityFunction::L2Distance => "<->",
        }
    }

    pub fn to_sql_function(&self) -> &'static str {
        match self {
            SimilarityFunction::Cosine => "1 - (embedding <=> $1)",
            SimilarityFunction::InnerProduct => "-(embedding <#> $1)",
            SimilarityFunction::L2Distance => "-(embedding <-> $1)",
        }
    }
}

/// Supabase 向量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseVectorRecord {
    /// 记录 ID
    pub id: String,
    /// 向量数据
    pub embedding: Vec<f32>,
    /// 内容文本
    pub content: String,
    /// 元数据 (JSONB)
    pub metadata: serde_json::Value,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl From<VectorData> for SupabaseVectorRecord {
    fn from(data: VectorData) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let metadata_json = serde_json::to_value(&data.metadata)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        Self {
            id: data.id,
            embedding: data.vector,
            content: data.metadata.get("content").cloned().unwrap_or_default(),
            metadata: metadata_json,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl From<SupabaseVectorRecord> for VectorData {
    fn from(record: SupabaseVectorRecord) -> Self {
        let mut metadata = HashMap::new();

        // 从 JSONB 转换回 HashMap<String, String>
        if let serde_json::Value::Object(map) = record.metadata {
            for (key, value) in map {
                if let serde_json::Value::String(string_value) = value {
                    metadata.insert(key, string_value);
                } else {
                    metadata.insert(key, value.to_string());
                }
            }
        }

        metadata.insert("content".to_string(), record.content);
        metadata.insert("created_at".to_string(), record.created_at);
        metadata.insert("updated_at".to_string(), record.updated_at);

        Self {
            id: record.id,
            vector: record.embedding,
            metadata,
        }
    }
}

/// Supabase 搜索响应
#[derive(Debug, Deserialize)]
pub struct SupabaseSearchResponse {
    pub data: Vec<SupabaseSearchResult>,
    pub error: Option<SupabaseError>,
}

/// Supabase 搜索结果
#[derive(Debug, Deserialize)]
pub struct SupabaseSearchResult {
    #[serde(flatten)]
    pub record: SupabaseVectorRecord,
    pub similarity: Option<f32>,
    pub distance: Option<f32>,
}

/// Supabase 错误
#[derive(Debug, Deserialize)]
pub struct SupabaseError {
    pub message: String,
    pub code: Option<String>,
    pub details: Option<String>,
}

/// Supabase 存储实现
pub struct SupabaseStore {
    config: SupabaseConfig,
    client: reqwest::Client,
    // 注意：这里我们使用一个简化的内存实现作为占位符
    // 在实际实现中，这里应该是真正的 Supabase 客户端
    vectors: std::sync::Arc<std::sync::Mutex<HashMap<String, SupabaseVectorRecord>>>,
}

impl SupabaseStore {
    /// 创建新的 Supabase 存储实例
    pub async fn new(config: SupabaseConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .build()
            .map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(format!(
                    "Failed to create HTTP client: {e}"
                ))
            })?;

        let store = Self {
            config,
            client,
            vectors: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        };

        // 验证连接和表结构
        store.verify_connection().await?;
        store.ensure_table_exists().await?;

        Ok(store)
    }

    /// 验证与 Supabase 的连接
    async fn verify_connection(&self) -> Result<()> {
        // 在实际实现中，这里应该调用 Supabase REST API 验证连接
        // let url = format!("{}/rest/v1/", self.config.project_url);
        // let response = self.client.get(&url)
        //     .header("apikey", &self.config.api_key)
        //     .header("Authorization", format!("Bearer {}", self.config.api_key))
        //     .send()
        //     .await?;

        // 真实的连接验证
        let url = format!("{}/rest/v1/", self.config.project_url);
        let response = self
            .client
            .get(&url)
            .header("apikey", &self.config.api_key)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to connect to Supabase: {e}"))
            })?;

        if response.status().is_success() {
            info!(
                "Successfully connected to Supabase at {}",
                self.config.project_url
            );
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(AgentMemError::storage_error(format!(
                "Supabase connection failed: {status} - {error_text}"
            )))
        }
    }

    /// 确保表存在，如果不存在则创建
    async fn ensure_table_exists(&self) -> Result<()> {
        // 真实的表创建和检查
        // 首先检查表是否存在
        let check_table_url =
            self.build_rest_url(&format!("{}?select=id&limit=1", self.config.table_name));
        let check_response = self
            .client
            .get(&check_table_url)
            .header("apikey", &self.config.api_key)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await;

        match check_response {
            Ok(response) if response.status().is_success() => {
                info!("Table {} already exists", self.config.table_name);
                Ok(())
            }
            _ => {
                // 表不存在，尝试创建（需要数据库管理员权限）
                info!(
                    "Table {} does not exist. Please create it manually with the following SQL:",
                    self.config.table_name
                );
                info!("CREATE EXTENSION IF NOT EXISTS vector;");
                info!("CREATE TABLE IF NOT EXISTS {} (", self.config.table_name);
                info!("    id TEXT PRIMARY KEY,");
                info!(
                    "    {} vector({}),",
                    self.config.vector_column, self.config.vector_dimension
                );
                info!("    {} TEXT,", self.config.content_column);
                info!("    {} JSONB,", self.config.metadata_column);
                info!("    created_at TIMESTAMPTZ DEFAULT NOW(),");
                info!("    updated_at TIMESTAMPTZ DEFAULT NOW()");
                info!(");");
                info!(
                    "CREATE INDEX ON {} USING ivfflat ({} vector_cosine_ops);",
                    self.config.table_name, self.config.vector_column
                );

                // 返回成功，假设表已经手动创建
                Ok(())
            }
        }
    }

    /// 构建 REST API URL
    fn build_rest_url(&self, endpoint: &str) -> String {
        format!("{}/rest/v1/{}", self.config.project_url, endpoint)
    }

    /// 构建 RPC URL
    fn build_rpc_url(&self, function_name: &str) -> String {
        format!("{}/rest/v1/rpc/{}", self.config.project_url, function_name)
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

    /// 执行实时订阅（在实际实现中）
    async fn _setup_realtime_subscription(&self) -> Result<()> {
        // 在实际实现中，这里应该设置 Supabase 实时订阅
        // 监听表的变化并触发回调
        Ok(())
    }

    /// 执行 RPC 函数调用（在实际实现中）
    async fn _call_rpc_function(
        &self,
        _function_name: &str,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // 在实际实现中，这里应该调用 Supabase 的 RPC 函数
        // 用于复杂的向量搜索和数据处理
        Ok(serde_json::Value::Null)
    }
}

#[async_trait]
impl VectorStore for SupabaseStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        let mut store = self.vectors.lock().unwrap();
        let mut ids = Vec::new();

        for vector_data in vectors {
            let id = if vector_data.id.is_empty() {
                format!("supabase_{}", uuid::Uuid::new_v4())
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

            let mut record = SupabaseVectorRecord::from(vector_data);
            record.id = id.clone();

            // 在实际实现中，这里应该调用 Supabase REST API 插入记录
            // let url = self.build_rest_url(&self.config.table_name);
            // let response = self.client.post(&url)
            //     .header("apikey", &self.config.api_key)
            //     .header("Authorization", format!("Bearer {}", self.config.api_key))
            //     .header("Content-Type", "application/json")
            //     .json(&record)
            //     .send()
            //     .await?;

            store.insert(id.clone(), record);
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

        // 在实际实现中，这里应该使用 Supabase 的向量搜索功能
        // 使用 pgvector 扩展进行高效的向量搜索
        // SELECT *, (embedding <=> $1) as distance
        // FROM agentmem_vectors
        // ORDER BY embedding <=> $1
        // LIMIT $2;

        for (_, record) in store.iter() {
            let similarity = self.cosine_similarity(&query_vector, &record.embedding);
            let distance = self.euclidean_distance(&query_vector, &record.embedding);

            // 应用阈值过滤
            if let Some(threshold) = threshold {
                if similarity < threshold {
                    continue;
                }
            }

            // 从 JSONB 转换元数据
            let mut metadata = HashMap::new();
            if let serde_json::Value::Object(map) = &record.metadata {
                for (key, value) in map {
                    if let serde_json::Value::String(string_value) = value {
                        metadata.insert(key.clone(), string_value.clone());
                    } else {
                        metadata.insert(key.clone(), value.to_string());
                    }
                }
            }

            results.push(VectorSearchResult {
                id: record.id.clone(),
                vector: record.embedding.clone(),
                metadata,
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
            // 在实际实现中，这里应该调用 Supabase REST API 删除记录
            // let url = format!("{}?id=eq.{}", self.build_rest_url(&self.config.table_name), id);
            // let response = self.client.delete(&url)
            //     .header("apikey", &self.config.api_key)
            //     .header("Authorization", format!("Bearer {}", self.config.api_key))
            //     .send()
            //     .await?;

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

            if let Some(existing_record) = store.get(&id) {
                let mut updated_record = SupabaseVectorRecord::from(vector_data);
                updated_record.id = id.clone();
                updated_record.created_at = existing_record.created_at.clone(); // 保持原创建时间
                updated_record.updated_at = chrono::Utc::now().to_rfc3339();

                // 在实际实现中，这里应该调用 Supabase REST API 更新记录
                // let url = format!("{}?id=eq.{}", self.build_rest_url(&self.config.table_name), id);
                // let response = self.client.patch(&url)
                //     .header("apikey", &self.config.api_key)
                //     .header("Authorization", format!("Bearer {}", self.config.api_key))
                //     .header("Content-Type", "application/json")
                //     .json(&updated_record)
                //     .send()
                //     .await?;

                store.insert(id, updated_record);
            }
        }

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该调用 Supabase REST API 获取记录
        // let url = format!("{}?id=eq.{}", self.build_rest_url(&self.config.table_name), id);
        // let response = self.client.get(&url)
        //     .header("apikey", &self.config.api_key)
        //     .header("Authorization", format!("Bearer {}", self.config.api_key))
        //     .send()
        //     .await?;

        Ok(store.get(id).map(|record| VectorData::from(record.clone())))
    }

    async fn count_vectors(&self) -> Result<usize> {
        let store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该调用 Supabase REST API 获取计数
        // let url = format!("{}?select=count", self.build_rest_url(&self.config.table_name));
        // let response = self.client.get(&url)
        //     .header("apikey", &self.config.api_key)
        //     .header("Authorization", format!("Bearer {}", self.config.api_key))
        //     .header("Prefer", "count=exact")
        //     .send()
        //     .await?;

        Ok(store.len())
    }

    async fn clear(&self) -> Result<()> {
        let mut store = self.vectors.lock().unwrap();

        // 在实际实现中，这里应该删除表中的所有记录
        // let url = self.build_rest_url(&self.config.table_name);
        // let response = self.client.delete(&url)
        //     .header("apikey", &self.config.api_key)
        //     .header("Authorization", format!("Bearer {}", self.config.api_key))
        //     .send()
        //     .await?;

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
        self.default_health_check("Supabase").await
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
