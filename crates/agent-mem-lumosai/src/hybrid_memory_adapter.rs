//! Hybrid Memory Adapter - 工作记忆 + 语义记忆混合架构
//!
//! 参考Mastra最佳实践，实现分层记忆：
//! - 工作记忆：最近N条消息（内存，快速访问）
//! - 语义记忆：长期记忆（AgentMem，语义搜索）

use agent_mem::Memory as AgentMemApi;
use async_trait::async_trait;
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use lumosai_core::Result as LumosResult;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::memory_adapter::AgentMemBackend;

/// 混合记忆配置
#[derive(Debug, Clone)]
pub struct HybridMemoryConfig {
    /// 工作记忆容量（最近N条消息）
    pub working_memory_capacity: usize,
    /// 启用工作记忆
    pub enable_working_memory: bool,
    /// 启用语义记忆
    pub enable_semantic_memory: bool,
    /// 工作记忆优先阈值（如果last_messages <= 此值，只使用工作记忆）
    pub working_memory_threshold: usize,
}

impl Default for HybridMemoryConfig {
    fn default() -> Self {
        Self {
            working_memory_capacity: 20, // 最近20条消息
            enable_working_memory: true,
            enable_semantic_memory: true,
            working_memory_threshold: 10, // 10条以内只用工作记忆
        }
    }
}

/// 混合记忆Backend
pub struct HybridMemoryBackend {
    /// 工作记忆：最近N条消息（内存）
    working_memory: Arc<RwLock<VecDeque<LumosMessage>>>,
    /// 语义记忆：AgentMem Backend
    semantic_memory: Arc<AgentMemBackend>,
    /// 配置
    config: HybridMemoryConfig,
}

impl HybridMemoryBackend {
    /// 创建新的混合记忆Backend
    pub fn new(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
        config: HybridMemoryConfig,
    ) -> Self {
        let semantic_memory = Arc::new(AgentMemBackend::new(
            memory_api,
            agent_id.clone(),
            user_id.clone(),
        ));

        Self {
            working_memory: Arc::new(RwLock::new(VecDeque::with_capacity(
                config.working_memory_capacity,
            ))),
            semantic_memory,
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
    ) -> Self {
        Self::new(memory_api, agent_id, user_id, HybridMemoryConfig::default())
    }

    /// 从工作记忆检索
    async fn retrieve_from_working_memory(
        &self,
        config: &MemoryConfig,
    ) -> Vec<LumosMessage> {
        let limit = config.last_messages.unwrap_or(self.config.working_memory_capacity);
        let working = self.working_memory.read().await;

        working
            .iter()
            .rev()
            .take(limit.min(self.config.working_memory_capacity))
            .cloned()
            .collect()
    }

    /// 从语义记忆检索
    async fn retrieve_from_semantic_memory(
        &self,
        config: &MemoryConfig,
    ) -> LumosResult<Vec<LumosMessage>> {
        self.semantic_memory.retrieve(config).await
    }
}

#[async_trait]
impl LumosMemory for HybridMemoryBackend {
    async fn store(&self, message: &LumosMessage) -> LumosResult<()> {
        let store_start = std::time::Instant::now();

        // 1. 存储到工作记忆（立即，0ms）
        if self.config.enable_working_memory {
            let working_start = std::time::Instant::now();
            {
                let mut working = self.working_memory.write().await;
                working.push_back(message.clone());

                // LRU淘汰：如果超过容量，移除最旧的消息
                while working.len() > self.config.working_memory_capacity {
                    working.pop_front();
                }
            }
            let working_duration = working_start.elapsed();
            debug!("   Working memory store: {:?}", working_duration);
        }

        // 2. 异步存储到语义记忆（后台，不阻塞）
        if self.config.enable_semantic_memory {
            let semantic = self.semantic_memory.clone();
            let msg = message.clone();
            tokio::spawn(async move {
                let semantic_start = std::time::Instant::now();
                if let Err(e) = semantic.store(&msg).await {
                    warn!("   ⚠️  Async semantic memory store failed: {}", e);
                } else {
                    let semantic_duration = semantic_start.elapsed();
                    debug!("   Semantic memory store (async): {:?}", semantic_duration);
                }
            });
        }

        let total_duration = store_start.elapsed();
        info!("✅ [HYBRID-STORE] Completed in {:?}", total_duration);

        Ok(())
    }

    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        let retrieve_start = std::time::Instant::now();
        info!("🔍 [HYBRID-RETRIEVE] Starting");

        let mut results = Vec::new();

        // 策略1: 如果只需要最近消息且数量 <= threshold，只使用工作记忆
        if self.config.enable_working_memory {
            if let Some(_last_n) = config.last_messages {
                if _last_n <= self.config.working_memory_threshold && config.query.is_none() {
                    let working_start = std::time::Instant::now();
                    let working_results = self.retrieve_from_working_memory(config).await;
                    let working_duration = working_start.elapsed();

                    info!(
                        "   ✅ Working memory only: {:?}, Found: {} messages",
                        working_duration,
                        working_results.len()
                    );

                    return Ok(working_results);
                }
            }
        }

        // 策略2: 如果有query，使用语义记忆
        if config.query.is_some() && self.config.enable_semantic_memory {
            let semantic_start = std::time::Instant::now();
            match self.retrieve_from_semantic_memory(config).await {
                Ok(semantic_results) => {
                    let semantic_duration = semantic_start.elapsed();
                    info!(
                        "   ✅ Semantic memory: {:?}, Found: {} messages",
                        semantic_duration,
                        semantic_results.len()
                    );
                    results.extend(semantic_results);
                }
                Err(e) => {
                    warn!("   ⚠️  Semantic memory retrieval failed: {}", e);
                }
            }
        }

        // 策略3: 如果需要最近消息，从工作记忆补充
        if self.config.enable_working_memory {
            if let Some(last_n) = config.last_messages {
                let working_start = std::time::Instant::now();
                let working_results = self.retrieve_from_working_memory(config).await;
                let working_duration = working_start.elapsed();

                // 合并结果（去重）
                let mut seen = std::collections::HashSet::new();
                for msg in results.iter() {
                    seen.insert(format!("{:?}:{}", msg.role, msg.content));
                }

                for msg in working_results {
                    let key = format!("{:?}:{}", msg.role, msg.content);
                    if !seen.contains(&key) {
                        seen.insert(key);
                        results.push(msg);
                    }
                }

                info!(
                    "   ✅ Working memory supplement: {:?}, Total: {} messages",
                    working_duration,
                    results.len()
                );
            }
        }

        let total_duration = retrieve_start.elapsed();
        info!(
            "✅ [HYBRID-RETRIEVE] Completed in {:?}, Total: {} messages",
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
    async fn test_working_memory_only() {
        // 测试工作记忆单独使用
        // 需要mock AgentMemApi
    }

    #[tokio::test]
    async fn test_hybrid_retrieval() {
        // 测试混合检索
        // 需要mock AgentMemApi
    }
}

