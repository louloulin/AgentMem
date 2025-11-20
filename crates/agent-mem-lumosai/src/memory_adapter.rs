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
        let store_start = std::time::Instant::now();
        
        info!("💾 [MEMORY-STORE] Starting");
        info!("   Role: {:?}, Content length: {}", 
              message.role, message.content.len());
        
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
        let api_call_start = std::time::Instant::now();
        let _result = self.memory_api.add_with_options(content, options).await
            .map_err(|e| {
                let err_msg = format!("Failed to store memory: {}", e);
                warn!("   ❌ {}", err_msg);
                lumosai_core::Error::Other(err_msg)
            })?;
        let api_call_duration = api_call_start.elapsed();
        
        info!("   ⏱️  API call: {:?}", api_call_duration);
        
        let total_duration = store_start.elapsed();
        info!("✅ [MEMORY-STORE] Completed in {:?}", total_duration);
        
        if total_duration.as_millis() > 500 {
            warn!("   ⚠️  Store took > 500ms, consider async storage");
        }
        
        Ok(())
    }
    
    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        let retrieve_start = std::time::Instant::now();
        // ⚡ 性能优化: 减少检索数量以降低prompt tokens和响应时间
        let limit = config.last_messages.unwrap_or(1);  // ⭐ 优化: 只检索1条历史
        
        info!("🔍 [MEMORY-RETRIEVE] Starting");
        info!("   Agent: {}, User: {}, Limit: {}", 
              self.agent_id, self.user_id, limit);
        
        // ✅ 使用agent-mem的get_all API
        let options = GetAllOptions {
            agent_id: Some(self.agent_id.clone()),
            user_id: Some(self.user_id.clone()),
            limit: Some(limit),
            ..Default::default()
        };
        
        let db_query_start = std::time::Instant::now();
        let memories = self.memory_api.get_all(options).await
            .map_err(|e| {
                let err_msg = format!("Failed to retrieve memories: {}", e);
                warn!("   ❌ {}", err_msg);
                lumosai_core::Error::Other(err_msg)
            })?;
        let db_query_duration = db_query_start.elapsed();
        
        info!("   ⏱️  Database query: {:?}, Found: {} memories", 
              db_query_duration, memories.len());
        
        // 转换MemoryItem为LumosMessage
        let messages: Vec<LumosMessage> = memories.into_iter()
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
    
    let total_duration = retrieve_start.elapsed();
    info!("✅ [MEMORY-RETRIEVE] Completed in {:?}, Returned: {} messages", 
          total_duration, messages.len());
    
    if total_duration.as_millis() > 100 {
        warn!("   ⚠️  Retrieve took > 100ms, consider caching");
    }
    
    Ok(messages)
    }
}
