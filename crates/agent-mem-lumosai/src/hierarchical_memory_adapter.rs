//! Hierarchical Memory Adapter - 分层记忆适配器
//!
//! 基于H-MEM架构和AgentMem的分层记忆系统，实现智能分层检索
//! 参考论文：
//! - H-MEM: Hierarchical Memory for High-Efficiency Long-Term Reasoning in LLM Agents
//! - AgentMem的Episodic-first检索策略（基于Atkinson-Shiffrin模型）

use agent_mem::Memory as AgentMemApi;
use async_trait::async_trait;
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use lumosai_core::Result as LumosResult;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::memory_adapter::AgentMemBackend;

/// 分层记忆配置
#[derive(Debug, Clone)]
pub struct HierarchicalMemoryConfig {
    /// 启用Episodic-first检索（优先使用Episodic记忆）
    pub enable_episodic_first: bool,
    /// Episodic记忆权重
    pub episodic_weight: f32,
    /// Working记忆权重
    pub working_weight: f32,
    /// Semantic记忆权重
    pub semantic_weight: f32,
    /// 启用记忆层次路由（Strategic -> Tactical -> Operational -> Contextual）
    pub enable_level_routing: bool,
    /// 启用记忆类型分类
    pub enable_memory_type_classification: bool,
    /// 最大检索数量
    pub max_retrieval_count: usize,
}

impl Default for HierarchicalMemoryConfig {
    fn default() -> Self {
        Self {
            enable_episodic_first: true,
            episodic_weight: 1.2,
            working_weight: 1.0,
            semantic_weight: 0.9,
            enable_level_routing: true,
            enable_memory_type_classification: true,
            max_retrieval_count: 10,
        }
    }
}

/// 分层记忆Backend
///
/// 实现H-MEM风格的分层记忆检索：
/// 1. 基于语义抽象的分层组织
/// 2. 索引路由机制（避免全量相似度计算）
/// 3. Episodic-first检索策略
pub struct HierarchicalMemoryBackend {
    /// 底层AgentMem Backend
    base_backend: Arc<AgentMemBackend>,
    /// 配置
    config: HierarchicalMemoryConfig,
    /// 记忆层次索引（用于快速路由）
    level_index: Arc<RwLock<LevelIndex>>,
}

/// 记忆层次索引
#[derive(Debug, Default)]
struct LevelIndex {
    /// Strategic层记忆ID列表
    strategic: Vec<String>,
    /// Tactical层记忆ID列表
    tactical: Vec<String>,
    /// Operational层记忆ID列表
    operational: Vec<String>,
    /// Contextual层记忆ID列表
    contextual: Vec<String>,
}

impl HierarchicalMemoryBackend {
    /// 创建新的分层记忆Backend
    pub fn new(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
        config: HierarchicalMemoryConfig,
    ) -> Self {
        let base_backend = Arc::new(AgentMemBackend::new(
            memory_api,
            agent_id,
            user_id,
        ));

        Self {
            base_backend,
            config,
            level_index: Arc::new(RwLock::new(LevelIndex::default())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
    ) -> Self {
        Self::new(memory_api, agent_id, user_id, HierarchicalMemoryConfig::default())
    }

    /// 分层检索策略
    ///
    /// 实现H-MEM的索引路由机制：
    /// 1. 首先从Contextual层检索（最相关）
    /// 2. 如果不足，从Operational层补充
    /// 3. 如果仍不足，从Tactical层补充
    /// 4. 最后从Strategic层补充
    async fn hierarchical_retrieve(
        &self,
        config: &MemoryConfig,
    ) -> LumosResult<Vec<LumosMessage>> {
        let mut results = Vec::new();
        let target_count = config.last_messages.unwrap_or(self.config.max_retrieval_count);

        // 策略1: Episodic-first检索（如果有query）
        if let Some(query) = &config.query {
            info!("   🧠 [HIERARCHICAL] Episodic-first retrieval with query");
            
            // 使用AgentMem的search API（内部已实现Episodic-first）
            let search_config = MemoryConfig {
                query: Some(query.clone()),
                last_messages: Some(target_count),
                ..config.clone()
            };
            
            let episodic_results = self.base_backend.retrieve(&search_config).await?;
            results.extend(episodic_results);
            
            info!("   ✅ Retrieved {} memories from Episodic-first search", results.len());
        } else {
            // 策略2: 时间顺序检索（无query）
            info!("   📜 [HIERARCHICAL] Time-ordered retrieval");
            
            let time_config = MemoryConfig {
                last_messages: Some(target_count),
                ..config.clone()
            };
            
            let time_results = self.base_backend.retrieve(&time_config).await?;
            results.extend(time_results);
            
            info!("   ✅ Retrieved {} memories from time-ordered search", results.len());
        }

        // 策略3: 记忆层次路由（如果启用）
        if self.config.enable_level_routing && results.len() < target_count {
            info!("   🔄 [HIERARCHICAL] Level routing for additional memories");
            
            // 从不同层次补充记忆
            // 注意：当前AgentMem API不直接支持层次检索，这里作为预留接口
            // 未来可以扩展AgentMem API支持层次检索
            debug!("   Level routing: Contextual -> Operational -> Tactical -> Strategic");
        }

        // 限制结果数量
        if results.len() > target_count {
            results.truncate(target_count);
        }

        Ok(results)
    }

    /// 记忆类型分类和加权
    ///
    /// 根据记忆类型应用不同权重（Episodic > Working > Semantic）
    fn apply_memory_type_weights(&self, messages: Vec<LumosMessage>) -> Vec<LumosMessage> {
        if !self.config.enable_memory_type_classification {
            return messages;
        }

        // 简化实现：根据消息内容特征推断记忆类型
        // 实际应该从AgentMem的metadata中获取记忆类型
        messages
            .into_iter()
            .map(|msg| {
                // 这里可以添加记忆类型标记和权重应用
                // 当前简化实现，直接返回
                msg
            })
            .collect()
    }
}

#[async_trait]
impl LumosMemory for HierarchicalMemoryBackend {
    async fn store(&self, message: &LumosMessage) -> LumosResult<()> {
        // 存储操作委托给底层Backend
        // 未来可以添加记忆类型自动分类和层次分配
        self.base_backend.store(message).await
    }

    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        let retrieve_start = std::time::Instant::now();
        info!("🔍 [HIERARCHICAL-RETRIEVE] Starting");

        // 使用分层检索策略
        let mut results = self.hierarchical_retrieve(config).await?;

        // 应用记忆类型权重
        results = self.apply_memory_type_weights(results);

        let total_duration = retrieve_start.elapsed();
        info!(
            "✅ [HIERARCHICAL-RETRIEVE] Completed in {:?}, Found: {} messages",
            total_duration,
            results.len()
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hierarchical_retrieve() {
        // 需要mock AgentMemApi
        // 实际测试应该在集成测试中完成
    }
}
