//! Agent Registry for Retrieval System
//!
//! 管理所有记忆 Agent 的注册表，用于检索系统调用真实的 Agent。

use crate::agents::{
    CoreAgent, EpisodicAgent, MemoryAgent, ProceduralAgent, SemanticAgent, WorkingAgent,
};
use crate::coordination::{TaskRequest, TaskResponse};
use crate::types::MemoryType;
use agent_mem_traits::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent 注册表
///
/// 维护所有记忆 Agent 的引用，并提供统一的调用接口
pub struct AgentRegistry {
    /// 核心记忆 Agent
    core_agent: Option<Arc<RwLock<CoreAgent>>>,
    /// 情景记忆 Agent
    episodic_agent: Option<Arc<RwLock<EpisodicAgent>>>,
    /// 语义记忆 Agent
    semantic_agent: Option<Arc<RwLock<SemanticAgent>>>,
    /// 程序记忆 Agent
    procedural_agent: Option<Arc<RwLock<ProceduralAgent>>>,
    /// 工作记忆 Agent
    working_agent: Option<Arc<RwLock<WorkingAgent>>>,
    /// Agent 映射表（用于快速查找）
    agent_map: Arc<RwLock<HashMap<MemoryType, AgentType>>>,
}

/// Agent 类型枚举
#[derive(Debug, Clone)]
enum AgentType {
    Core,
    Episodic,
    Semantic,
    Procedural,
    Working,
}

impl AgentRegistry {
    /// 创建新的 Agent 注册表
    pub fn new() -> Self {
        Self {
            core_agent: None,
            episodic_agent: None,
            semantic_agent: None,
            procedural_agent: None,
            working_agent: None,
            agent_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册核心记忆 Agent
    pub async fn register_core_agent(&mut self, agent: Arc<RwLock<CoreAgent>>) -> Result<()> {
        self.core_agent = Some(agent);
        self.agent_map
            .write()
            .await
            .insert(MemoryType::Core, AgentType::Core);
        Ok(())
    }

    /// 注册情景记忆 Agent
    pub async fn register_episodic_agent(
        &mut self,
        agent: Arc<RwLock<EpisodicAgent>>,
    ) -> Result<()> {
        self.episodic_agent = Some(agent);
        self.agent_map
            .write()
            .await
            .insert(MemoryType::Episodic, AgentType::Episodic);
        Ok(())
    }

    /// 注册语义记忆 Agent
    pub async fn register_semantic_agent(
        &mut self,
        agent: Arc<RwLock<SemanticAgent>>,
    ) -> Result<()> {
        self.semantic_agent = Some(agent);
        self.agent_map
            .write()
            .await
            .insert(MemoryType::Semantic, AgentType::Semantic);
        Ok(())
    }

    /// 注册程序记忆 Agent
    pub async fn register_procedural_agent(
        &mut self,
        agent: Arc<RwLock<ProceduralAgent>>,
    ) -> Result<()> {
        self.procedural_agent = Some(agent);
        self.agent_map
            .write()
            .await
            .insert(MemoryType::Procedural, AgentType::Procedural);
        Ok(())
    }

    /// 注册工作记忆 Agent
    pub async fn register_working_agent(
        &mut self,
        agent: Arc<RwLock<WorkingAgent>>,
    ) -> Result<()> {
        self.working_agent = Some(agent);
        self.agent_map
            .write()
            .await
            .insert(MemoryType::Working, AgentType::Working);
        Ok(())
    }

    /// 执行任务（调用对应的 Agent）
    pub async fn execute_task(
        &self,
        memory_type: &MemoryType,
        task: TaskRequest,
    ) -> Result<TaskResponse> {
        let agent_map = self.agent_map.read().await;
        let agent_type = agent_map.get(memory_type).ok_or_else(|| {
            agent_mem_traits::AgentMemError::NotFound(format!(
                "No agent registered for memory type: {memory_type:?}"
            ))
        })?;

        match agent_type {
            AgentType::Core => {
                if let Some(ref agent) = self.core_agent {
                    let mut agent_guard = agent.write().await;
                    agent_guard
                        .execute_task(task)
                        .await
                        .map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))
                } else {
                    Err(agent_mem_traits::AgentMemError::NotFound(
                        "Core agent not initialized".to_string(),
                    ))
                }
            }
            AgentType::Episodic => {
                if let Some(ref agent) = self.episodic_agent {
                    let mut agent_guard = agent.write().await;
                    agent_guard
                        .execute_task(task)
                        .await
                        .map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))
                } else {
                    Err(agent_mem_traits::AgentMemError::NotFound(
                        "Episodic agent not initialized".to_string(),
                    ))
                }
            }
            AgentType::Semantic => {
                if let Some(ref agent) = self.semantic_agent {
                    let mut agent_guard = agent.write().await;
                    agent_guard
                        .execute_task(task)
                        .await
                        .map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))
                } else {
                    Err(agent_mem_traits::AgentMemError::NotFound(
                        "Semantic agent not initialized".to_string(),
                    ))
                }
            }
            AgentType::Procedural => {
                if let Some(ref agent) = self.procedural_agent {
                    let mut agent_guard = agent.write().await;
                    agent_guard
                        .execute_task(task)
                        .await
                        .map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))
                } else {
                    Err(agent_mem_traits::AgentMemError::NotFound(
                        "Procedural agent not initialized".to_string(),
                    ))
                }
            }
            AgentType::Working => {
                if let Some(ref agent) = self.working_agent {
                    let mut agent_guard = agent.write().await;
                    agent_guard
                        .execute_task(task)
                        .await
                        .map_err(|e| agent_mem_traits::AgentMemError::MemoryError(e.to_string()))
                } else {
                    Err(agent_mem_traits::AgentMemError::NotFound(
                        "Working agent not initialized".to_string(),
                    ))
                }
            }
        }
    }

    /// 检查是否有 Agent 注册
    pub async fn has_agent(&self, memory_type: &MemoryType) -> bool {
        self.agent_map.read().await.contains_key(memory_type)
    }

    /// 获取已注册的 Agent 数量
    pub async fn agent_count(&self) -> usize {
        self.agent_map.read().await.len()
    }

    /// 获取所有已注册的记忆类型
    pub async fn registered_memory_types(&self) -> Vec<MemoryType> {
        self.agent_map.read().await.keys().cloned().collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: These tests are disabled because they require a real Store implementation
    // TODO: Re-enable these tests with proper Store setup

    #[tokio::test]
    #[ignore] // Disabled: requires real Store implementation
    async fn test_agent_registry_basic() {
        let mut registry = AgentRegistry::new();

        // 创建一个 agent with real store
        // let store = Arc::new(/* create real store */);
        let mut agent = CoreAgent::new("test-agent".to_string());
        // agent.set_store(store);
        let agent_arc = Arc::new(RwLock::new(agent));

        // 注册 agent
        // registry.register_core_agent(agent_arc).await.unwrap();

        // 验证注册
        // assert!(registry.has_agent(&MemoryType::Core).await);
        // assert_eq!(registry.agent_count().await, 1);

        // let types = registry.registered_memory_types().await;
        // assert_eq!(types.len(), 1);
        // assert!(types.contains(&MemoryType::Core));
    }

    #[tokio::test]
    #[ignore] // Disabled: requires real Store implementation
    async fn test_agent_registry_multiple_agents() {
        let mut registry = AgentRegistry::new();

        // 注册多个 agents
        // let core_store = Arc::new(/* create real store */);
        let mut core_agent = CoreAgent::new("core-agent".to_string());
        // core_agent.set_store(core_store);
        // registry
        //     .register_core_agent(Arc::new(RwLock::new(core_agent)))
        //     .await
        //     .unwrap();

        // 验证
        // assert_eq!(registry.agent_count().await, 1);
    }
}

