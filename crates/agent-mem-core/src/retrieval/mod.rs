//! AgentMem 7.0 主动检索机制
//!
//! 本模块实现了基于 MIRIX 架构的主动检索系统，包括：
//! - TopicExtractor: 基于 LLM 的主题提取
//! - RetrievalRouter: 智能路由和多策略检索
//! - ContextSynthesizer: 多源记忆融合和上下文合成
//! - AgentRegistry: Agent 注册表，支持真实 Agent 调用
//!
//! 参考 MIRIX 的设计模式，但针对 Rust 的特性进行了优化

pub mod agent_registry;
pub mod router;
pub mod synthesizer;
pub mod topic_extractor;

#[cfg(test)]
mod tests;

// Re-export main types
pub use agent_registry::AgentRegistry;
pub use router::{
    RetrievalRouter, RetrievalRouterConfig, RetrievalStrategy, RouteDecision, RoutingResult,
};
pub use synthesizer::{
    ConflictResolution, ContextSynthesizer, ContextSynthesizerConfig, SynthesisResult,
};
pub use topic_extractor::{
    ExtractedTopic, TopicCategory, TopicExtractor, TopicExtractorConfig, TopicHierarchy,
};

use crate::types::MemoryType;
use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 检索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRequest {
    /// 查询文本
    pub query: String,
    /// 目标记忆类型（可选）
    pub target_memory_types: Option<Vec<MemoryType>>,
    /// 最大结果数量
    pub max_results: usize,
    /// 检索策略偏好
    pub preferred_strategy: Option<RetrievalStrategy>,
    /// 上下文信息
    pub context: Option<HashMap<String, serde_json::Value>>,
    /// 是否启用主题提取
    pub enable_topic_extraction: bool,
    /// 是否启用上下文合成
    pub enable_context_synthesis: bool,
}

/// 检索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResponse {
    /// 检索到的记忆项
    pub memories: Vec<RetrievedMemory>,
    /// 提取的主题
    pub extracted_topics: Vec<ExtractedTopic>,
    /// 路由决策信息
    pub routing_info: RouteDecision,
    /// 合成结果
    pub synthesis_result: Option<SynthesisResult>,
    /// 总处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 置信度分数
    pub confidence_score: f32,
}

/// 检索到的记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedMemory {
    /// 记忆ID
    pub id: String,
    /// 记忆类型
    pub memory_type: MemoryType,
    /// 内容
    pub content: String,
    /// 相关性分数
    pub relevance_score: f32,
    /// 来源智能体
    pub source_agent: String,
    /// 检索策略
    pub retrieval_strategy: RetrievalStrategy,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 主动检索系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRetrievalConfig {
    /// 主题提取器配置
    pub topic_extractor: TopicExtractorConfig,
    /// 路由器配置
    pub router: RetrievalRouterConfig,
    /// 合成器配置
    pub synthesizer: ContextSynthesizerConfig,
    /// 默认最大结果数
    pub default_max_results: usize,
    /// 默认置信度阈值
    pub default_confidence_threshold: f32,
    /// 是否启用缓存
    pub enable_caching: bool,
    /// 缓存过期时间（秒）
    pub cache_ttl_seconds: u64,
}

impl Default for ActiveRetrievalConfig {
    fn default() -> Self {
        Self {
            topic_extractor: TopicExtractorConfig::default(),
            router: RetrievalRouterConfig::default(),
            synthesizer: ContextSynthesizerConfig::default(),
            default_max_results: 10,
            default_confidence_threshold: 0.5,
            enable_caching: true,
            cache_ttl_seconds: 300, // 5分钟
        }
    }
}

/// 主动检索系统
///
/// 整合主题提取、智能路由和上下文合成功能，
/// 提供统一的主动检索接口
pub struct ActiveRetrievalSystem {
    /// 主题提取器
    topic_extractor: Arc<TopicExtractor>,
    /// 检索路由器
    router: Arc<RetrievalRouter>,
    /// 上下文合成器
    synthesizer: Arc<ContextSynthesizer>,
    /// Agent 注册表
    agent_registry: Arc<RwLock<AgentRegistry>>,
    /// 系统配置
    config: ActiveRetrievalConfig,
    /// 检索缓存
    cache: Arc<RwLock<HashMap<String, (RetrievalResponse, std::time::Instant)>>>,
    /// 是否使用真实 Agent（false 则使用 Mock）
    use_real_agents: bool,
}

impl ActiveRetrievalSystem {
    /// 创建新的主动检索系统
    pub async fn new(config: ActiveRetrievalConfig) -> Result<Self> {
        let topic_extractor = Arc::new(TopicExtractor::new(config.topic_extractor.clone()).await?);
        let router = Arc::new(RetrievalRouter::new(config.router.clone()).await?);
        let synthesizer = Arc::new(ContextSynthesizer::new(config.synthesizer.clone()).await?);

        Ok(Self {
            topic_extractor,
            router,
            synthesizer,
            agent_registry: Arc::new(RwLock::new(AgentRegistry::new())),
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            use_real_agents: false, // 默认使用 Mock
        })
    }

    /// 启用真实 Agent 调用
    pub fn enable_real_agents(&mut self) {
        self.use_real_agents = true;
    }

    /// 禁用真实 Agent 调用（使用 Mock）
    pub fn disable_real_agents(&mut self) {
        self.use_real_agents = false;
    }

    /// 获取 Agent 注册表的引用
    pub fn agent_registry(&self) -> Arc<RwLock<AgentRegistry>> {
        Arc::clone(&self.agent_registry)
    }

    /// 执行主动检索
    pub async fn retrieve(&self, request: RetrievalRequest) -> Result<RetrievalResponse> {
        let start_time = std::time::Instant::now();

        // 检查缓存
        if self.config.enable_caching {
            if let Some(cached_response) = self.check_cache(&request).await? {
                return Ok(cached_response);
            }
        }

        // 1. 主题提取
        let extracted_topics = if request.enable_topic_extraction {
            self.topic_extractor
                .extract_topics(&request.query, request.context.as_ref())
                .await?
        } else {
            Vec::new()
        };

        // 2. 智能路由
        let routing_result = self
            .router
            .route_retrieval(&request, &extracted_topics)
            .await?;

        // 3. 执行检索
        let memories = self.execute_retrieval(&request, &routing_result).await?;

        // 4. 上下文合成
        let synthesis_result = if request.enable_context_synthesis && !memories.is_empty() {
            Some(
                self.synthesizer
                    .synthesize_context(&memories, &request)
                    .await?,
            )
        } else {
            None
        };

        let processing_time_ms = start_time.elapsed().as_millis().max(1) as u64;
        let confidence_score = self.calculate_confidence_score(&memories, &synthesis_result);

        let response = RetrievalResponse {
            memories,
            extracted_topics,
            routing_info: routing_result.decision,
            synthesis_result,
            processing_time_ms,
            confidence_score,
        };

        // 缓存结果
        if self.config.enable_caching {
            self.cache_response(&request, &response).await?;
        }

        Ok(response)
    }

    /// 检查缓存
    async fn check_cache(&self, request: &RetrievalRequest) -> Result<Option<RetrievalResponse>> {
        let cache_key = self.generate_cache_key(request);
        let cache = self.cache.read().await;

        if let Some((response, timestamp)) = cache.get(&cache_key) {
            if timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                return Ok(Some(response.clone()));
            }
        }

        Ok(None)
    }

    /// 缓存响应
    async fn cache_response(
        &self,
        request: &RetrievalRequest,
        response: &RetrievalResponse,
    ) -> Result<()> {
        let cache_key = self.generate_cache_key(request);
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, (response.clone(), std::time::Instant::now()));
        Ok(())
    }

    /// 生成缓存键
    fn generate_cache_key(&self, request: &RetrievalRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.query.hash(&mut hasher);
        request.target_memory_types.hash(&mut hasher);
        request.max_results.hash(&mut hasher);
        request.preferred_strategy.hash(&mut hasher);

        format!("retrieval_{}", hasher.finish())
    }

    /// 执行实际的检索操作
    async fn execute_retrieval(
        &self,
        request: &RetrievalRequest,
        routing_result: &RoutingResult,
    ) -> Result<Vec<RetrievedMemory>> {
        use std::time::Instant;

        let start_time = Instant::now();
        let mut all_memories = Vec::new();

        // 根据路由决策的目标记忆类型执行检索
        for memory_type in &routing_result.decision.target_memory_types {
            // 获取该记忆类型的检索策略和权重
            let strategy = routing_result
                .decision
                .selected_strategies
                .first()
                .cloned()
                .unwrap_or(RetrievalStrategy::StringMatch);

            let strategy_weight = routing_result
                .decision
                .strategy_weights
                .get(&strategy)
                .copied()
                .unwrap_or(1.0);

            // 执行针对特定记忆类型的检索
            let memories = self
                .retrieve_from_memory_type(
                    request,
                    memory_type,
                    &strategy,
                    strategy_weight,
                )
                .await?;

            all_memories.extend(memories);
        }

        // 按相关性分数降序排序
        all_memories.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 截断到最大结果数
        all_memories.truncate(request.max_results);

        let elapsed = start_time.elapsed();
        log::info!(
            "Retrieved {} memories from {} memory types in {:?}",
            all_memories.len(),
            routing_result.decision.target_memory_types.len(),
            elapsed
        );

        Ok(all_memories)
    }

    /// 从特定记忆类型检索
    async fn retrieve_from_memory_type(
        &self,
        request: &RetrievalRequest,
        memory_type: &MemoryType,
        strategy: &RetrievalStrategy,
        strategy_weight: f32,
    ) -> Result<Vec<RetrievedMemory>> {
        let agent_id = format!("{memory_type:?}Agent");

        // 如果启用了真实 Agent，尝试调用真实 Agent
        if self.use_real_agents {
            let registry = self.agent_registry.read().await;

            if registry.has_agent(memory_type).await {
                log::debug!(
                    "Using real agent for {agent_id} with {strategy:?} strategy"
                );

                // 构建任务请求
                let task = crate::coordination::TaskRequest {
                    task_id: uuid::Uuid::new_v4().to_string(),
                    memory_type: *memory_type,
                    operation: "search".to_string(),
                    parameters: serde_json::json!({
                        "query": request.query,
                        "max_results": request.max_results,
                        "strategy": format!("{:?}", strategy),
                    }),
                    priority: 5, // Normal priority
                    timeout: Some(std::time::Duration::from_secs(5)),
                    retry_count: 0,
                };

                // 调用真实 Agent
                match registry.execute_task(memory_type, task).await {
                    Ok(response) => {
                        // 将 Agent 响应转换为 RetrievedMemory
                        return self.convert_agent_response_to_memories(
                            response,
                            memory_type,
                            &agent_id,
                            strategy_weight,
                        );
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to execute task on real agent {agent_id}: {e}. Falling back to mock."
                        );
                        // 失败时回退到 Mock
                    }
                }
            } else {
                log::debug!(
                    "No real agent registered for {memory_type:?}, using mock"
                );
            }
        }

        // 使用 Mock 结果（默认或回退）
        let mock_results = self.generate_mock_results(
            request,
            memory_type,
            &agent_id,
            strategy,
            strategy_weight,
        );

        log::debug!(
            "Retrieved {} mock results from {} using {:?} strategy",
            mock_results.len(),
            agent_id,
            strategy
        );

        Ok(mock_results)
    }

    /// 将 Agent 响应转换为检索记忆
    fn convert_agent_response_to_memories(
        &self,
        response: crate::coordination::TaskResponse,
        memory_type: &MemoryType,
        agent_id: &str,
        strategy_weight: f32,
    ) -> Result<Vec<RetrievedMemory>> {
        // 检查响应是否成功
        if !response.success {
            log::warn!(
                "Agent {} returned unsuccessful response: {:?}",
                agent_id,
                response.error
            );
            return Ok(Vec::new());
        }

        // 从响应中提取记忆数据
        let memories_data = response
            .data
            .as_ref()
            .and_then(|d| d.get("memories"))
            .and_then(|v| v.as_array());

        if let Some(memories) = memories_data {
            let retrieved_memories: Vec<RetrievedMemory> = memories
                .iter()
                .filter_map(|mem| {
                    let content = mem.get("content")?.as_str()?.to_string();
                    let score = mem.get("score")?.as_f64().unwrap_or(0.5) as f32;

                    Some(RetrievedMemory {
                        id: mem.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        memory_type: *memory_type,
                        content,
                        relevance_score: score * strategy_weight,
                        source_agent: agent_id.to_string(),
                        retrieval_strategy: RetrievalStrategy::StringMatch, // 默认策略
                        metadata: mem
                            .get("metadata")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                })
                .collect();

            log::info!(
                "Converted {} memories from real agent {}",
                retrieved_memories.len(),
                agent_id
            );

            Ok(retrieved_memories)
        } else {
            log::warn!(
                "No memories found in agent response from {agent_id}"
            );
            Ok(Vec::new())
        }
    }

    /// 生成模拟检索结果
    fn generate_mock_results(
        &self,
        request: &RetrievalRequest,
        memory_type: &MemoryType,
        agent_id: &str,
        strategy: &RetrievalStrategy,
        strategy_weight: f32,
    ) -> Vec<RetrievedMemory> {
        // 根据查询生成 1-3 个模拟结果
        let result_count = (request.query.len() % 3) + 1;
        let mut results = Vec::new();

        for i in 0..result_count.min(request.max_results) {
            // 计算相关性分数：基础分数 * 策略权重 * 位置惩罚
            let base_score = 0.9 - (i as f32 * 0.1);
            let position_penalty = 1.0 - (i as f32 * 0.05);
            let relevance_score = base_score * strategy_weight * position_penalty;

            let memory = RetrievedMemory {
                id: format!("{}_{}_result_{}", memory_type.to_string().to_lowercase(), agent_id, i),
                memory_type: *memory_type,
                content: format!(
                    "Mock {} memory result {} for query: '{}' (strategy: {:?})",
                    memory_type,
                    i + 1,
                    request.query,
                    strategy
                ),
                relevance_score,
                source_agent: agent_id.to_string(),
                retrieval_strategy: strategy.clone(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert(
                        "mock".to_string(),
                        serde_json::json!(true),
                    );
                    map.insert(
                        "query".to_string(),
                        serde_json::json!(request.query.clone()),
                    );
                    map.insert(
                        "memory_type".to_string(),
                        serde_json::json!(memory_type.to_string()),
                    );
                    map.insert(
                        "strategy".to_string(),
                        serde_json::json!(format!("{:?}", strategy)),
                    );
                    map
                },
            };

            results.push(memory);
        }

        results
    }

    /// 计算置信度分数
    fn calculate_confidence_score(
        &self,
        memories: &[RetrievedMemory],
        synthesis_result: &Option<SynthesisResult>,
    ) -> f32 {
        if memories.is_empty() {
            return 0.0;
        }

        let avg_relevance: f32 =
            memories.iter().map(|m| m.relevance_score).sum::<f32>() / memories.len() as f32;
        let synthesis_boost = synthesis_result
            .as_ref()
            .map(|s| s.confidence_score * 0.2)
            .unwrap_or(0.0);

        (avg_relevance + synthesis_boost).min(1.0)
    }

    /// 清理过期缓存
    pub async fn cleanup_cache(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let now = std::time::Instant::now();

        cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < self.config.cache_ttl_seconds
        });

        Ok(())
    }

    /// 获取系统统计信息
    pub async fn get_stats(&self) -> Result<RetrievalStats> {
        let cache = self.cache.read().await;

        Ok(RetrievalStats {
            cache_size: cache.len(),
            topic_extractor_stats: self.topic_extractor.get_stats().await?,
            router_stats: self.router.get_stats().await?,
            synthesizer_stats: self.synthesizer.get_stats().await?,
        })
    }
}

/// 检索系统统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalStats {
    /// 缓存大小
    pub cache_size: usize,
    /// 主题提取器统计
    pub topic_extractor_stats: serde_json::Value,
    /// 路由器统计
    pub router_stats: serde_json::Value,
    /// 合成器统计
    pub synthesizer_stats: serde_json::Value,
}
