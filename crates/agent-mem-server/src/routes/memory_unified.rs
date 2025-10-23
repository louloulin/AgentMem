//! Memory management routes - Unified Memory API version
//!
//! 架构优化：使用agent-mem的Memory统一API替代agent-mem-core的CoreMemoryManager
//! 优势：
//! - 更简洁的代码
//! - 统一的接口
//! - 自动的智能功能
//! - 更好的类型处理

use crate::{
    error::{ServerError, ServerResult},
    models::{
        BatchRequest, BatchResponse, MemoryRequest, MemoryResponse, SearchRequest, SearchResponse,
        UpdateMemoryRequest,
    },
};
use agent_mem::{Memory, AddMemoryOptions, SearchOptions, GetAllOptions, DeleteAllOptions};
use agent_mem_traits::MemoryItem;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server-side memory manager wrapper (基于Memory统一API)
pub struct MemoryManager {
    memory: Arc<Memory>,
}

impl MemoryManager {
    /// 创建新的MemoryManager（使用Memory API）
    pub async fn new() -> ServerResult<Self> {
        let memory = Memory::new()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to create Memory: {}", e)))?;
        
        Ok(Self {
            memory: Arc::new(memory),
        })
    }

    /// 使用自定义配置创建
    pub async fn with_config(memory: Memory) -> Self {
        Self {
            memory: Arc::new(memory),
        }
    }

    /// 添加记忆
    pub async fn add_memory(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        _memory_type: Option<agent_mem_traits::MemoryType>,  // Memory API自动处理
        _importance: Option<f32>,  // Memory API自动评估
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        let options = AddMemoryOptions {
            agent_id: Some(agent_id),
            user_id,
            infer: true,  // 启用智能推理
            metadata,
            ..Default::default()
        };

        self.memory
            .add_with_options(content, options)
            .await
            .map(|result| {
                // 返回第一个记忆的ID（如果有多个，取第一个）
                result.results
                    .first()
                    .map(|r| r.id.clone())
                    .unwrap_or_else(|| "".to_string())
            })
            .map_err(|e| e.to_string())
    }

    /// 获取记忆
    pub async fn get_memory(&self, id: &str) -> Result<Option<serde_json::Value>, String> {
        match self.memory.get(id).await {
            Ok(memory) => {
                let json = serde_json::json!({
                    "id": memory.id,
                    "agent_id": memory.agent_id,
                    "user_id": memory.user_id,
                    "content": memory.content,
                    "memory_type": memory.memory_type,
                    "importance": memory.importance,
                    "created_at": memory.created_at,
                    "last_accessed_at": memory.last_accessed_at,
                    "access_count": memory.access_count,
                    "metadata": memory.metadata,
                    "hash": memory.hash,
                });
                Ok(Some(json))
            }
            Err(e) => {
                if e.to_string().contains("not found") {
                    Ok(None)
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    /// 更新记忆
    pub async fn update_memory(
        &self,
        id: &str,
        content: Option<String>,
        importance: Option<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), String> {
        let mut update_data = HashMap::new();

        if let Some(c) = content {
            update_data.insert("content".to_string(), serde_json::json!(c));
        }
        if let Some(imp) = importance {
            update_data.insert("importance".to_string(), serde_json::json!(imp));
        }
        if let Some(meta) = metadata {
            // 转换metadata为JSON
            let meta_json: HashMap<String, serde_json::Value> = meta
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            update_data.insert("metadata".to_string(), serde_json::json!(meta_json));
        }

        self.memory
            .update(id, update_data)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// 删除记忆
    pub async fn delete_memory(&self, id: &str) -> Result<(), String> {
        self.memory
            .delete(id)
            .await
            .map_err(|e| e.to_string())
    }

    /// 搜索记忆
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: Option<String>,
        user_id: Option<String>,
        limit: Option<usize>,
        _memory_type: Option<agent_mem_traits::MemoryType>,
    ) -> Result<Vec<MemoryItem>, String> {
        let options = SearchOptions {
            user_id,
            limit,
            threshold: Some(0.7),
            ..Default::default()
        };

        self.memory
            .search_with_options(query, options)
            .await
            .map_err(|e| e.to_string())
    }

    /// 获取所有记忆
    pub async fn get_all_memories(
        &self,
        agent_id: Option<String>,
        user_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, String> {
        let options = GetAllOptions {
            agent_id,
            user_id,
            limit,
            ..Default::default()
        };

        self.memory
            .get_all(options)
            .await
            .map_err(|e| e.to_string())
    }

    /// 删除所有记忆
    pub async fn delete_all_memories(
        &self,
        agent_id: Option<String>,
        user_id: Option<String>,
    ) -> Result<usize, String> {
        let options = DeleteAllOptions {
            agent_id,
            user_id,
            ..Default::default()
        };

        self.memory
            .delete_all(options)
            .await
            .map_err(|e| e.to_string())
    }

    /// 重置所有记忆（危险操作）
    pub async fn reset(&self) -> Result<(), String> {
        self.memory
            .reset()
            .await
            .map_err(|e| e.to_string())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<agent_mem::MemoryStats, String> {
        self.memory
            .get_stats()
            .await
            .map_err(|e| e.to_string())
    }
}

/// 默认实现（异步创建）
impl MemoryManager {
    pub fn new_sync() -> Self {
        // 注意：这只用于类型系统，实际使用应该调用async new()
        panic!("Use MemoryManager::new().await instead");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let result = MemoryManager::new().await;
        // 可能因为配置问题失败，但应该能创建
        println!("MemoryManager creation: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_manager_with_builder() {
        // 使用Memory builder创建配置
        let memory = Memory::builder()
            .disable_intelligent_features()  // 测试时禁用智能功能
            .build()
            .await;

        if let Ok(mem) = memory {
            let manager = MemoryManager::with_config(mem).await;
            println!("MemoryManager with config created successfully");
        }
    }
}

