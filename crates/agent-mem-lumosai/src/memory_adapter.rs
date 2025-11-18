//! Memory Adapter - 将AgentMem作为LumosAI的Memory Backend

use async_trait::async_trait;
use lumosai_core::memory::Memory as LumosMemory;
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::llm::Role as LumosRole;
use lumosai_core::Result as LumosResult;
use agent_mem_core::engine::MemoryEngine;
use agent_mem_core::hierarchy::MemoryScope;
use agent_mem_traits::{Memory, Content, AttributeKey, AttributeValue, MemoryId};
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
        debug!("Storing message to AgentMem: {:?}", message.role);
        
        // 转换LumosMessage为AgentMem Memory
        let role_str = match message.role {
            LumosRole::System => "system",
            LumosRole::User => "user",
            LumosRole::Assistant => "assistant",
            LumosRole::Tool => "tool",
        };
        
        let content_text = format!("[{}]: {}", role_str, message.content);
        let content = Content::text(content_text);
        
        // 创建Memory
        let mut memory = Memory::new(
            MemoryId::new(),
            content,
        );
        
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
        if let Some(metadata) = &message.metadata {
            memory.attributes.set(
                AttributeKey::user("original_metadata"),
                AttributeValue::String(metadata.to_string())
            );
        }
        
        // 调用MemoryEngine存储
        let memory_id = self.engine.add_memory(memory).await
            .map_err(|e| lumosai_core::Error::Other(format!("Failed to store memory: {}", e)))?;
        
        info!("✅ Stored memory to AgentMem: {}", memory_id);
        Ok(())
    }
    
    async fn retrieve(&self, query: &str, limit: usize) -> LumosResult<Vec<LumosMessage>> {
        debug!("Retrieving memories from AgentMem: query='{}', limit={}", query, limit);
        
        // 构建搜索scope
        let scope = Some(MemoryScope::User {
            agent_id: self.agent_id.clone(),
            user_id: self.user_id.clone(),
        });
        
        // 调用MemoryEngine搜索
        let memories = self.engine.search_memories(query, scope, Some(limit))
            .await
            .map_err(|e| lumosai_core::Error::Other(format!("Failed to retrieve memories: {}", e)))?;
        
        info!("Retrieved {} memories from AgentMem", memories.len());
        
        // 转换Memory为LumosMessage
        let messages = memories.into_iter()
            .map(|mem| {
                // 提取content
                let content_text = match &mem.content {
                    Content::Text(t) => t.clone(),
                    Content::Structured(v) => v.to_string(),
                    _ => String::new(),
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
    
    async fn clear(&self) -> LumosResult<()> {
        warn!("Memory clear requested but not implemented in AgentMem");
        // AgentMem没有直接的clear方法，保留为未来功能
        Ok(())
    }
}
