//! Memory Adapter - 将agent-mem的Memory API适配为LumosAI的Memory trait
//! 
//! 完全基于agent-mem的统一Memory API实现

use async_trait::async_trait;
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::llm::Role as LumosRole;
use lumosai_core::Result as LumosResult;
use agent_mem::{Memory as AgentMemApi, AddMemoryOptions, GetAllOptions};
use std::sync::Arc;
use tracing::{info, warn};

/// AgentMem Backend for LumosAI
/// 
/// 使用agent-mem的完整Memory API（不是repository层）
pub struct AgentMemBackend {
    memory_api: Arc<AgentMemApi>,
    agent_id: String,
    user_id: String,
}

impl AgentMemBackend {
    pub fn new(memory_api: Arc<AgentMemApi>, agent_id: String, user_id: String) -> Self {
        Self {
            memory_api,
            agent_id,
            user_id,
        }
    }
}

#[async_trait]
impl LumosMemory for AgentMemBackend {
    async fn store(&self, message: &LumosMessage) -> LumosResult<()> {
        info!("💾 Storing message to AgentMem: role={:?}, agent_id={}, user_id={}", 
              message.role, self.agent_id, self.user_id);
        
        // 转换LumosMessage为content string
        let role_str = match message.role {
            LumosRole::System => "system",
            LumosRole::User => "user",
            LumosRole::Assistant => "assistant",
            LumosRole::Tool => "tool",
            LumosRole::Function => "function",
            LumosRole::Custom(ref custom) => custom.as_str(),
        };
        
        let content = format!("[{}]: {}", role_str, message.content);
        
        // 构建metadata
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("role".to_string(), role_str.to_string());
        metadata.insert("source".to_string(), "lumosai".to_string());
        
        // 使用agent-mem的Memory API
        let options = AddMemoryOptions {
            agent_id: Some(self.agent_id.clone()),
            user_id: Some(self.user_id.clone()),
            metadata,
            infer: false,  // 不需要复杂推理
            ..Default::default()
        };
        
        // ✅ 调用agent-mem的add_with_options API
        let _result = self.memory_api.add_with_options(content, options).await
            .map_err(|e| {
                let err_msg = format!("Failed to store memory: {}", e);
                warn!("{}", err_msg);
                lumosai_core::Error::Other(err_msg)
            })?;
        
        info!("✅ Stored memory to AgentMem");
        Ok(())
    }
    
    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        // ⚡ 性能优化: 减少检索数量以降低prompt tokens和响应时间
        let limit = config.last_messages.unwrap_or(3);  // 从10降到3
        info!("🔍 Retrieving memories: agent_id={}, user_id={}, limit={}", 
              self.agent_id, self.user_id, limit);
        
        // ✅ 使用agent-mem的get_all API
        let options = GetAllOptions {
            agent_id: Some(self.agent_id.clone()),
            user_id: Some(self.user_id.clone()),
            limit: Some(limit),
            ..Default::default()
        };
        
        let memories = self.memory_api.get_all(options).await
            .map_err(|e| {
                let err_msg = format!("Failed to retrieve memories: {}", e);
                warn!("{}", err_msg);
                lumosai_core::Error::Other(err_msg)
            })?;
        
        info!("✅ Retrieved {} historical messages from AgentMem", memories.len());
        
        // 转换MemoryItem为LumosMessage
        let messages = memories.into_iter()
            .filter_map(|mem| {
                // 从metadata中提取role（metadata是HashMap<String, Value>）
                let role_str = mem.metadata
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user");
                
                let role = match role_str {
                    "system" => LumosRole::System,
                    "assistant" => LumosRole::Assistant,
                    "tool" => LumosRole::Tool,
                    _ => LumosRole::User,
                };
                
                // 移除格式前缀 "[role]: "
                let content = if mem.content.starts_with('[') {
                    mem.content.splitn(2, "]: ").nth(1)
                        .unwrap_or(&mem.content)
                        .to_string()
                } else {
                    mem.content
                };
                
                Some(LumosMessage {
                    role,
                    content,
                    metadata: None,
                    name: None,
                })
            })
            .collect();
        
        Ok(messages)
    }
}
