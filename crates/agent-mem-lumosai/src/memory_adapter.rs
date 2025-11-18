//! Memory Adapter - 将AgentMem作为LumosAI的Memory Backend

use async_trait::async_trait;
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::llm::Role as LumosRole;
use lumosai_core::Result as LumosResult;
use agent_mem_core::engine::MemoryEngine;
use agent_mem_core::hierarchy::MemoryScope;
use agent_mem_traits::{Content, AttributeKey, AttributeValue, MemoryId};
use agent_mem_traits::abstractions::Memory as AgentMemMemory;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// AgentMem Backend for LumosAI
/// 
/// 将AgentMem的专业记忆管理能力包装为LumosAI Memory trait
pub struct AgentMemBackend {
    engine: Arc<MemoryEngine>,
    agent_id: String,
    user_id: String,
}

impl AgentMemBackend {
    pub fn new(
        engine: Arc<MemoryEngine>,
        agent_id: String,
        user_id: String,
    ) -> Self {
        Self {
            engine,
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
        
        // 转换LumosMessage为AgentMem Memory
        let role_str = match message.role {
            LumosRole::System => "system",
            LumosRole::User => "user",
            LumosRole::Assistant => "assistant",
            LumosRole::Tool => "tool",
            LumosRole::Function => "function",
            LumosRole::Custom(ref custom) => custom.as_str(),
        };
        
        let content_text = format!("[{}]: {}", role_str, message.content);
        let content = Content::Text(content_text);
        
        // 创建Memory
        let mut memory = AgentMemMemory {
            id: MemoryId::new(),
            content,
            attributes: Default::default(),
            relations: Default::default(),
            metadata: Default::default(),
        };
        
        // 设置属性
        memory.attributes.set(
            AttributeKey::core("agent_id"),
            AttributeValue::String(self.agent_id.clone())
        );
        memory.attributes.set(
            AttributeKey::core("user_id"),
            AttributeValue::String(self.user_id.clone())
        );
        memory.attributes.set(
            AttributeKey::user("role"),
            AttributeValue::String(role_str.to_string())
        );
        
        // 如果有metadata，也保存
        if let Some(metadata) = message.metadata.as_ref() {
            memory.attributes.set(
                AttributeKey::user("original_metadata"),
                AttributeValue::String(serde_json::to_string(metadata).unwrap_or_default())
            );
        }
        
        // 调用MemoryEngine存储
        let memory_id = self.engine.add_memory(memory).await
            .map_err(|e| {
                let err_msg = format!("Failed to store memory: {}", e);
                warn!("{}", err_msg);
                lumosai_core::Error::Other(err_msg)
            })?;
        
        info!("✅ Stored memory to AgentMem: id={}", memory_id);
        Ok(())
    }
    
    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        // 确定检索限制
        let limit = config.last_messages.unwrap_or(10);
        info!("🔍 Retrieving memories: agent_id={}, user_id={}, limit={}", 
              self.agent_id, self.user_id, limit);
        
        // 构建搜索scope
        let scope = Some(MemoryScope::User {
            agent_id: self.agent_id.clone(),
            user_id: self.user_id.clone(),
        });
        
        // ✅ 修复：使用空查询来获取最近的消息（按时间排序）
        // search_memories会自动按created_at DESC排序
        info!("Calling engine.search_memories with empty query");
        let memories = self.engine.search_memories("", scope, Some(limit))
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to retrieve memories: {}", e);
                warn!("{}", err_msg);
                lumosai_core::Error::Other(err_msg)
            })?;
        
        info!("✅ Retrieved {} historical messages from AgentMem", memories.len());
        
        // 转换Memory为LumosMessage
        let messages = memories.into_iter()
            .map(|mem| {
                // 提取content
                let content_text = match &mem.content {
                    Content::Text(t) => t.clone(),
                    Content::Structured(v) => v.to_string(),
                    Content::Vector(_) => "[vector]".to_string(),
                    Content::Multimodal(_) => "[multimodal]".to_string(),
                    Content::Binary(_) => "[binary]".to_string(),
                };
                
                // 解析role
                let role = mem.attributes
                    .get(&AttributeKey::user("role"))
                    .and_then(|v| v.as_string())
                    .map(|s| match s.as_str() {
                        "system" => LumosRole::System,
                        "assistant" => LumosRole::Assistant,
                        "tool" => LumosRole::Tool,
                        _ => LumosRole::User,
                    })
                    .unwrap_or(LumosRole::User);
                
                // 提取metadata
                let metadata = mem.attributes
                    .get(&AttributeKey::user("original_metadata"))
                    .and_then(|v| v.as_string())
                    .and_then(|s| serde_json::from_str(s).ok());
                
                LumosMessage {
                    role,
                    content: content_text,
                    metadata,
                    name: None,
                }
            })
            .collect();
        
        Ok(messages)
    }
    
}

// clear方法不在Memory trait中，作为独立实现
impl AgentMemBackend {
    #[allow(dead_code)]
    async fn _clear(&self) -> LumosResult<()> {
        warn!("Memory clear requested but not implemented in AgentMem");
        // AgentMem没有直接的clear方法，保留为未来功能
        Ok(())
    }
}
