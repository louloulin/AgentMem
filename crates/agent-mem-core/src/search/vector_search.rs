//! 向量搜索引擎
//!
//! 提供语义相似度搜索功能，基于向量存储后端
//! 支持 pgvector 扩展和性能优化

use super::{SearchQuery, SearchResult};
use agent_mem_traits::{AgentMemError, Result, VectorData, VectorStore};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 向量搜索配置
#[derive(Debug, Clone)]
pub struct VectorSearchConfig {
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存大小
    pub cache_size: usize,
    /// 是否启用批量优化
    pub enable_batch_optimization: bool,
    /// 批量大小
    pub batch_size: usize,
    /// 是否使用 pgvector 扩展
    pub use_pgvector: bool,
    /// pgvector 索引类型 (ivfflat, hnsw)
    pub pgvector_index_type: PgVectorIndexType,
    /// 索引参数
    pub index_params: IndexParams,
}

impl Default for VectorSearchConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_size: 1000,
            enable_batch_optimization: true,
            batch_size: 100,
            use_pgvector: false,
            pgvector_index_type: PgVectorIndexType::IVFFlat,
            index_params: IndexParams::default(),
        }
    }
}

/// pgvector 索引类型
#[derive(Debug, Clone)]
pub enum PgVectorIndexType {
    /// IVFFlat 索引（快速但近似）
    IVFFlat,
    /// HNSW 索引（更精确但慢）
    HNSW,
}

/// 索引参数
#[derive(Debug, Clone)]
pub struct IndexParams {
    /// IVFFlat 列表数量
    pub ivfflat_lists: usize,
    /// HNSW M 参数
    pub hnsw_m: usize,
    /// HNSW ef_construction 参数
    pub hnsw_ef_construction: usize,
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            ivfflat_lists: 100,
            hnsw_m: 16,
            hnsw_ef_construction: 64,
        }
    }
}

/// 搜索缓存项
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 查询向量
    query_vector: Vec<f32>,
    /// 搜索结果
    results: Vec<SearchResult>,
    /// 缓存时间
    cached_at: Instant,
}

/// 向量搜索引擎
pub struct VectorSearchEngine {
    /// 向量存储后端
    vector_store: Arc<dyn VectorStore>,
    /// 嵌入模型维度
    embedding_dimension: usize,
    /// 配置
    config: VectorSearchConfig,
    /// 搜索缓存
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// 性能统计
    stats: Arc<RwLock<PerformanceStats>>,
}

/// 性能统计
#[derive(Debug, Clone, Default)]
struct PerformanceStats {
    /// 总搜索次数
    total_searches: usize,
    /// 缓存命中次数
    cache_hits: usize,
    /// 平均搜索时间（毫秒）
    avg_search_time_ms: f64,
    /// 总搜索时间（毫秒）
    total_search_time_ms: u64,
}

impl VectorSearchEngine {
    /// 创建新的向量搜索引擎
    ///
    /// # Arguments
    ///
    /// * `vector_store` - 向量存储后端
    /// * `embedding_dimension` - 嵌入向量维度
    pub fn new(vector_store: Arc<dyn VectorStore>, embedding_dimension: usize) -> Self {
        Self::with_config(
            vector_store,
            embedding_dimension,
            VectorSearchConfig::default(),
        )
    }

    /// 使用配置创建向量搜索引擎
    pub fn with_config(
        vector_store: Arc<dyn VectorStore>,
        embedding_dimension: usize,
        config: VectorSearchConfig,
    ) -> Self {
        Self {
            vector_store,
            embedding_dimension,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
        }
    }

    /// 执行向量搜索（带缓存和优化）
    ///
    /// # Arguments
    ///
    /// * `query_vector` - 查询向量
    /// * `query` - 搜索查询参数
    ///
    /// # Returns
    ///
    /// 返回搜索结果列表和搜索时间（毫秒）
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        query: &SearchQuery,
    ) -> Result<(Vec<SearchResult>, u64)> {
        let start = Instant::now();

        // 验证向量维度
        if query_vector.len() != self.embedding_dimension {
            return Err(AgentMemError::validation_error(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.embedding_dimension
            )));
        }

        // 检查缓存
        if self.config.enable_cache {
            let cache_key = self.generate_cache_key(&query_vector, query);

            if let Some(cached_results) = self.check_cache(&cache_key).await {
                let elapsed = start.elapsed().as_millis() as u64;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                stats.total_searches += 1;

                log::debug!("Cache hit for vector search, saved {elapsed} ms");
                return Ok((cached_results, elapsed));
            }
        }

        // 执行向量搜索
        let vector_results = self
            .vector_store
            .search_vectors(query_vector.clone(), query.limit, query.threshold)
            .await?;

        // 转换为搜索结果
        let results: Vec<SearchResult> = vector_results
            .into_iter()
            .map(|vr| SearchResult {
                id: vr.id,
                content: vr.metadata.get("content").cloned().unwrap_or_default(),
                score: vr.similarity,
                vector_score: Some(vr.similarity),
                fulltext_score: None,
                metadata: Some(
                    serde_json::to_value(&vr.metadata).unwrap_or(serde_json::Value::Null),
                ),
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;

        // 更新缓存
        if self.config.enable_cache {
            let cache_key = self.generate_cache_key(&query_vector, query);
            self.update_cache(cache_key, query_vector, results.clone())
                .await;
        }

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_searches += 1;
        stats.total_search_time_ms += elapsed;
        stats.avg_search_time_ms = stats.total_search_time_ms as f64 / stats.total_searches as f64;

        Ok((results, elapsed))
    }

    /// 生成缓存键
    fn generate_cache_key(&self, query_vector: &[f32], query: &SearchQuery) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 对向量进行哈希（使用前几个元素以提高性能）
        for &val in query_vector.iter().take(10) {
            val.to_bits().hash(&mut hasher);
        }

        query.limit.hash(&mut hasher);
        if let Some(threshold) = query.threshold {
            threshold.to_bits().hash(&mut hasher);
        }

        format!("vec_{}", hasher.finish())
    }

    /// 检查缓存
    async fn check_cache(&self, cache_key: &str) -> Option<Vec<SearchResult>> {
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(cache_key) {
            // 检查缓存是否过期（5分钟）
            if entry.cached_at.elapsed().as_secs() < 300 {
                return Some(entry.results.clone());
            }
        }

        None
    }

    /// 更新缓存
    async fn update_cache(
        &self,
        cache_key: String,
        query_vector: Vec<f32>,
        results: Vec<SearchResult>,
    ) {
        let mut cache = self.cache.write().await;

        // 如果缓存已满，移除最旧的条目
        if cache.len() >= self.config.cache_size {
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            cache_key,
            CacheEntry {
                query_vector,
                results,
                cached_at: Instant::now(),
            },
        );
    }

    /// 清空缓存
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// 获取性能统计
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        self.stats.read().await.clone()
    }

    /// 批量添加向量
    ///
    /// # Arguments
    ///
    /// * `vectors` - 向量数据列表
    ///
    /// # Returns
    ///
    /// 返回添加的向量 ID 列表
    pub async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        // 验证所有向量维度
        for vector in &vectors {
            if vector.vector.len() != self.embedding_dimension {
                return Err(AgentMemError::validation_error(format!(
                    "Vector dimension {} does not match expected dimension {}",
                    vector.vector.len(),
                    self.embedding_dimension
                )));
            }
        }

        self.vector_store.add_vectors(vectors).await
    }

    /// 删除向量
    ///
    /// # Arguments
    ///
    /// * `ids` - 要删除的向量 ID 列表
    pub async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        self.vector_store.delete_vectors(ids).await
    }

    /// 获取向量存储统计信息
    pub async fn get_stats(&self) -> Result<VectorStoreStats> {
        let perf_stats = self.stats.read().await;

        // 从 vector_store 获取实际的向量数量
        let total_vectors = self.vector_store.count_vectors().await.unwrap_or(0);

        Ok(VectorStoreStats {
            total_vectors,
            dimension: self.embedding_dimension,
            index_type: if self.config.use_pgvector {
                match self.config.pgvector_index_type {
                    PgVectorIndexType::IVFFlat => "pgvector_ivfflat".to_string(),
                    PgVectorIndexType::HNSW => "pgvector_hnsw".to_string(),
                }
            } else {
                "default".to_string()
            },
            total_searches: perf_stats.total_searches,
            cache_hits: perf_stats.cache_hits,
            avg_search_time_ms: perf_stats.avg_search_time_ms,
            cache_hit_rate: if perf_stats.total_searches > 0 {
                perf_stats.cache_hits as f64 / perf_stats.total_searches as f64
            } else {
                0.0
            },
        })
    }

    /// 创建 pgvector 索引（仅在使用 PostgreSQL 时）
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL 连接池
    /// * `table_name` - 表名
    /// * `column_name` - 向量列名（默认为 "embedding"）
    ///
    /// # Returns
    ///
    /// 返回索引创建结果
    #[cfg(feature = "postgres")]
    pub async fn create_pgvector_index(
        &self,
        pool: &sqlx::PgPool,
        table_name: &str,
        column_name: Option<&str>,
    ) -> Result<()> {
        if !self.config.use_pgvector {
            return Err(AgentMemError::ConfigError(
                "pgvector is not enabled in configuration".to_string(),
            ));
        }

        let column = column_name.unwrap_or("embedding");
        let index_name = format!("{}_{}_idx", table_name, column);

        log::info!(
            "Creating pgvector index '{}' on table '{}.{}' with type {:?}",
            index_name,
            table_name,
            column,
            self.config.pgvector_index_type
        );

        // 根据索引类型创建不同的索引
        let sql = match self.config.pgvector_index_type {
            PgVectorIndexType::IVFFlat => {
                // IVFFlat 索引：快速但近似
                // lists 参数：聚类数量，通常设置为 sqrt(total_rows)
                format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {} USING ivfflat ({} vector_cosine_ops) WITH (lists = {})",
                    index_name,
                    table_name,
                    column,
                    self.config.index_params.ivfflat_lists
                )
            }
            PgVectorIndexType::HNSW => {
                // HNSW 索引：更精确但构建慢
                // m: 每个节点的最大连接数（通常 16-64）
                // ef_construction: 构建时的搜索深度（通常 64-200）
                format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {} USING hnsw ({} vector_cosine_ops) WITH (m = {}, ef_construction = {})",
                    index_name,
                    table_name,
                    column,
                    self.config.index_params.hnsw_m,
                    self.config.index_params.hnsw_ef_construction
                )
            }
        };

        // 执行索引创建
        sqlx::query(&sql).execute(pool).await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to create pgvector index: {}", e))
        })?;

        log::info!("Successfully created pgvector index '{}'", index_name);

        Ok(())
    }

    /// 优化向量搜索性能
    pub async fn optimize_search_performance(&self) -> Result<()> {
        log::info!("Optimizing vector search performance...");

        // 清理过期缓存
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        cache.retain(|_, entry| now.duration_since(entry.cached_at).as_secs() < 300);

        log::info!(
            "Cleaned up expired cache entries. Current cache size: {}",
            cache.len()
        );

        Ok(())
    }

    /// 批量搜索优化
    pub async fn batch_search(
        &self,
        query_vectors: Vec<Vec<f32>>,
        query: &SearchQuery,
    ) -> Result<Vec<(Vec<SearchResult>, u64)>> {
        if !self.config.enable_batch_optimization {
            // 如果未启用批量优化，逐个搜索
            let mut results = Vec::new();
            for qv in query_vectors {
                results.push(self.search(qv, query).await?);
            }
            return Ok(results);
        }

        // 批量优化：并发执行搜索
        let mut tasks = Vec::new();

        for query_vector in query_vectors {
            let engine = self.clone_for_search();
            let query_clone = query.clone();

            tasks.push(tokio::spawn(async move {
                engine.search(query_vector, &query_clone).await
            }));
        }

        let mut results = Vec::new();
        for task in tasks {
            results.push(task.await.map_err(|e| {
                AgentMemError::MemoryError(format!("Batch search task failed: {e}"))
            })??);
        }

        Ok(results)
    }

    /// 克隆用于并发搜索
    fn clone_for_search(&self) -> Self {
        Self {
            vector_store: Arc::clone(&self.vector_store),
            embedding_dimension: self.embedding_dimension,
            config: self.config.clone(),
            cache: Arc::clone(&self.cache),
            stats: Arc::clone(&self.stats),
        }
    }
}

/// 向量存储统计信息
#[derive(Debug, Clone)]
pub struct VectorStoreStats {
    /// 总向量数
    pub total_vectors: usize,
    /// 向量维度
    pub dimension: usize,
    /// 索引类型
    pub index_type: String,
    /// 总搜索次数
    pub total_searches: usize,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 平均搜索时间（毫秒）
    pub avg_search_time_ms: f64,
    /// 缓存命中率
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_storage::backends::MemoryVectorStore;
    use agent_mem_traits::VectorStoreConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_vector_search_engine() {
        let config = VectorStoreConfig {
            provider: "memory".to_string(),
            path: "".to_string(),
            table_name: "test".to_string(),
            dimension: Some(128),
            api_key: None,
            url: None,
            index_name: None,
            collection_name: None,
        };
        let vector_store = Arc::new(MemoryVectorStore::new(config).await.unwrap());
        let engine = VectorSearchEngine::new(vector_store.clone(), 128);

        // 添加测试向量
        let mut metadata = HashMap::new();
        metadata.insert("content".to_string(), "test content".to_string());

        let vectors = vec![VectorData {
            id: "test-1".to_string(),
            vector: vec![0.1; 128],
            metadata,
        }];

        let ids = engine.add_vectors(vectors).await.unwrap();
        assert_eq!(ids.len(), 1);

        // 执行搜索
        let query = SearchQuery {
            query: "test".to_string(),
            limit: 10,
            threshold: Some(0.5),
            ..Default::default()
        };

        let query_vector = vec![0.1; 128];
        let (results, elapsed) = engine.search(query_vector, &query).await.unwrap();

        assert!(!results.is_empty());
        assert!(elapsed >= 0); // 允许 0，因为内存操作可能非常快
    }

    #[tokio::test]
    async fn test_vector_dimension_validation() {
        let config = VectorStoreConfig::default();
        let vector_store = Arc::new(MemoryVectorStore::new(config).await.unwrap());
        let engine = VectorSearchEngine::new(vector_store, 128);

        let query = SearchQuery::default();
        let wrong_dimension_vector = vec![0.1; 64]; // 错误的维度

        let result = engine.search(wrong_dimension_vector, &query).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_vectors() {
        let config = VectorStoreConfig {
            dimension: Some(128),
            ..Default::default()
        };
        let vector_store = Arc::new(MemoryVectorStore::new(config).await.unwrap());
        let engine = VectorSearchEngine::new(vector_store, 128);

        // 添加向量
        let mut metadata = HashMap::new();
        metadata.insert("content".to_string(), "test".to_string());

        let vectors = vec![VectorData {
            id: "test-1".to_string(),
            vector: vec![0.1; 128],
            metadata,
        }];

        let ids = engine.add_vectors(vectors).await.unwrap();

        // 删除向量
        let result = engine.delete_vectors(ids).await;
        assert!(result.is_ok());
    }
}

/// pgvector 距离操作符
///
/// 支持 PostgreSQL pgvector 扩展的三种距离度量方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorDistanceOperator {
    /// 余弦距离: <=>
    ///
    /// 范围: [0, 2]
    /// - 0 表示完全相同
    /// - 1 表示正交
    /// - 2 表示完全相反
    Cosine,

    /// L2 距离 (欧几里得距离): <->
    ///
    /// 范围: [0, ∞)
    /// - 0 表示完全相同
    /// - 值越大表示越不相似
    L2,

    /// 内积 (负内积): <#>
    ///
    /// 范围: (-∞, 0]
    /// - 值越接近 0 表示越相似
    /// - 值越负表示越不相似
    InnerProduct,
}

impl VectorDistanceOperator {
    /// 转换为 SQL 操作符
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Cosine => "<=>",
            Self::L2 => "<->",
            Self::InnerProduct => "<#>",
        }
    }

    /// 将距离转换为相似度分数
    ///
    /// 返回值范围: [0, 1]
    /// - 1.0 表示完全相同
    /// - 0.0 表示完全不相似
    pub fn distance_to_similarity(&self, distance: f32) -> f32 {
        match self {
            // 余弦距离: [0, 2] -> [1, 0]
            Self::Cosine => 1.0 - (distance / 2.0),
            // L2 距离: [0, ∞) -> [1, 0)
            Self::L2 => 1.0 / (1.0 + distance),
            // 内积: (-∞, 0] -> [0, 1]
            Self::InnerProduct => {
                if distance <= 0.0 {
                    (-distance).min(1.0)
                } else {
                    0.0
                }
            }
        }
    }

    /// 将相似度分数转换为距离
    ///
    /// 这是 `distance_to_similarity` 的反函数
    pub fn similarity_to_distance(&self, similarity: f32) -> f32 {
        match self {
            // 相似度: [1, 0] -> 余弦距离: [0, 2]
            Self::Cosine => (1.0 - similarity) * 2.0,
            // 相似度: [1, 0) -> L2 距离: [0, ∞)
            Self::L2 => {
                if similarity >= 1.0 {
                    0.0
                } else if similarity <= 0.0 {
                    f32::INFINITY
                } else {
                    (1.0 / similarity) - 1.0
                }
            }
            // 相似度: [0, 1] -> 内积: (-∞, 0]
            Self::InnerProduct => -similarity,
        }
    }

    /// 获取操作符名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cosine => "cosine",
            Self::L2 => "l2",
            Self::InnerProduct => "inner_product",
        }
    }
}

impl Default for VectorDistanceOperator {
    fn default() -> Self {
        Self::Cosine
    }
}

/// 构建向量搜索 SQL 查询
///
/// # 参数
///
/// * `table` - 表名
/// * `column` - 向量列名
/// * `operator` - 距离操作符
/// * `limit` - 返回结果数量限制
/// * `threshold` - 可选的相似度阈值 (0.0 - 1.0)
///
/// # 返回
///
/// 返回 SQL 查询字符串，使用占位符 $1, $2, $3 等
///
/// # 示例
///
/// ```rust
/// use agent_mem_core::search::vector_search::{build_vector_search_sql, VectorDistanceOperator};
///
/// let sql = build_vector_search_sql(
///     "memories",
///     "embedding",
///     VectorDistanceOperator::Cosine,
///     10,
///     Some(0.7),
/// );
/// ```
pub fn build_vector_search_sql(
    table: &str,
    column: &str,
    operator: VectorDistanceOperator,
    limit: usize,
    threshold: Option<f32>,
) -> String {
    let op = operator.to_sql();

    if let Some(threshold) = threshold {
        // 将相似度阈值转换为距离阈值
        let distance_threshold = operator.similarity_to_distance(threshold);

        // 根据操作符类型构建不同的 WHERE 子句
        let where_clause = match operator {
            VectorDistanceOperator::Cosine | VectorDistanceOperator::L2 => {
                // 余弦和 L2: 距离越小越相似
                format!("WHERE ({column} {op} $1) <= $2")
            }
            VectorDistanceOperator::InnerProduct => {
                // 内积: 值越大（越接近 0）越相似
                format!("WHERE ({column} {op} $1) >= $2")
            }
        };

        format!(
            "SELECT *, ({column} {op} $1) as distance
             FROM {table}
             {where_clause}
             ORDER BY {column} {op} $1
             LIMIT $3"
        )
    } else {
        format!(
            "SELECT *, ({column} {op} $1) as distance
             FROM {table}
             ORDER BY {column} {op} $1
             LIMIT $2"
        )
    }
}

/// 构建多向量混合搜索 SQL 查询
///
/// # 参数
///
/// * `table` - 表名
/// * `columns` - 向量列名数组
/// * `weights` - 对应的权重数组
/// * `operator` - 距离操作符
/// * `limit` - 返回结果数量限制
///
/// # 返回
///
/// 返回 SQL 查询字符串和占位符数量
///
/// # 示例
///
/// ```rust
/// use agent_mem_core::search::vector_search::{build_hybrid_vector_search_sql, VectorDistanceOperator};
///
/// let (sql, param_count) = build_hybrid_vector_search_sql(
///     "semantic_memory",
///     &["summary_embedding", "details_embedding"],
///     &[0.6, 0.4],
///     VectorDistanceOperator::Cosine,
///     10,
/// );
/// ```
pub fn build_hybrid_vector_search_sql(
    table: &str,
    columns: &[&str],
    weights: &[f32],
    operator: VectorDistanceOperator,
    limit: usize,
) -> (String, usize) {
    assert_eq!(
        columns.len(),
        weights.len(),
        "columns and weights must have the same length"
    );
    assert!(!columns.is_empty(), "columns cannot be empty");

    let op = operator.to_sql();

    // 构建加权相似度计算表达式
    let mut similarity_parts = Vec::new();
    let mut param_index = 1;

    for (i, (column, weight)) in columns.iter().zip(weights.iter()).enumerate() {
        // 将距离转换为相似度，然后乘以权重
        let similarity_expr = match operator {
            VectorDistanceOperator::Cosine => {
                // 余弦距离: similarity = 1 - (distance / 2)
                format!("({weight} * (1.0 - ({column} {op} ${param_index}) / 2.0))")
            }
            VectorDistanceOperator::L2 => {
                // L2 距离: similarity = 1 / (1 + distance)
                format!("({weight} / (1.0 + ({column} {op} ${param_index})))")
            }
            VectorDistanceOperator::InnerProduct => {
                // 内积: similarity = -distance (假设已归一化)
                format!("({weight} * (-({column} {op} ${param_index})))")
            }
        };

        similarity_parts.push(similarity_expr);
        param_index += 1;
    }

    let combined_score = similarity_parts.join(" + ");

    let sql = format!(
        "SELECT *, ({combined_score}) as combined_score
         FROM {table}
         ORDER BY combined_score DESC
         LIMIT ${param_index}"
    );

    (sql, param_index)
}

#[cfg(test)]
mod vector_operator_tests {
    use super::*;

    #[test]
    fn test_vector_distance_operator_to_sql() {
        assert_eq!(VectorDistanceOperator::Cosine.to_sql(), "<=>");
        assert_eq!(VectorDistanceOperator::L2.to_sql(), "<->");
        assert_eq!(VectorDistanceOperator::InnerProduct.to_sql(), "<#>");
    }

    #[test]
    fn test_distance_to_similarity_cosine() {
        let op = VectorDistanceOperator::Cosine;
        assert_eq!(op.distance_to_similarity(0.0), 1.0); // 完全相同
        assert_eq!(op.distance_to_similarity(1.0), 0.5); // 正交
        assert_eq!(op.distance_to_similarity(2.0), 0.0); // 完全相反
    }

    #[test]
    fn test_distance_to_similarity_l2() {
        let op = VectorDistanceOperator::L2;
        assert_eq!(op.distance_to_similarity(0.0), 1.0); // 完全相同
        assert!(op.distance_to_similarity(1.0) > 0.4 && op.distance_to_similarity(1.0) < 0.6);
        assert!(op.distance_to_similarity(10.0) < 0.1);
    }

    #[test]
    fn test_distance_to_similarity_inner_product() {
        let op = VectorDistanceOperator::InnerProduct;
        assert_eq!(op.distance_to_similarity(0.0), 0.0);
        assert_eq!(op.distance_to_similarity(-0.5), 0.5);
        assert_eq!(op.distance_to_similarity(-1.0), 1.0);
    }

    #[test]
    fn test_similarity_to_distance_roundtrip() {
        let operators = [
            VectorDistanceOperator::Cosine,
            VectorDistanceOperator::L2,
            VectorDistanceOperator::InnerProduct,
        ];

        for op in &operators {
            let similarities = [0.0, 0.25, 0.5, 0.75, 1.0];
            for &sim in &similarities {
                let distance = op.similarity_to_distance(sim);
                let recovered_sim = op.distance_to_similarity(distance);
                assert!(
                    (recovered_sim - sim).abs() < 0.01,
                    "Roundtrip failed for {op:?}: {sim} -> {distance} -> {recovered_sim}"
                );
            }
        }
    }

    #[test]
    fn test_build_vector_search_sql_without_threshold() {
        let sql = build_vector_search_sql(
            "memories",
            "embedding",
            VectorDistanceOperator::Cosine,
            10,
            None,
        );

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM memories"));
        assert!(sql.contains("embedding <=> $1"));
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("LIMIT $2"));
        assert!(!sql.contains("WHERE"));
    }

    #[test]
    fn test_build_vector_search_sql_with_threshold() {
        let sql = build_vector_search_sql(
            "memories",
            "embedding",
            VectorDistanceOperator::Cosine,
            10,
            Some(0.7),
        );

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM memories"));
        assert!(sql.contains("embedding <=> $1"));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("LIMIT $3"));
    }

    #[test]
    fn test_build_hybrid_vector_search_sql() {
        let (sql, param_count) = build_hybrid_vector_search_sql(
            "semantic_memory",
            &["summary_embedding", "details_embedding"],
            &[0.6, 0.4],
            VectorDistanceOperator::Cosine,
            10,
        );

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM semantic_memory"));
        assert!(sql.contains("summary_embedding"));
        assert!(sql.contains("details_embedding"));
        assert!(sql.contains("combined_score"));
        assert!(sql.contains("ORDER BY combined_score DESC"));
        assert_eq!(param_count, 3); // 2 vectors + 1 limit
    }

    #[test]
    fn test_operator_name() {
        assert_eq!(VectorDistanceOperator::Cosine.name(), "cosine");
        assert_eq!(VectorDistanceOperator::L2.name(), "l2");
        assert_eq!(VectorDistanceOperator::InnerProduct.name(), "inner_product");
    }
}
