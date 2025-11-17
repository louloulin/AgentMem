use agent_mem::types::{AddMemoryOptions, SearchOptions};
use agent_mem::Memory;
use agent_mem_traits::{AgentMemError, MemoryItem, Result};
use std::collections::HashMap;

/// 轻量包装，兼容历史 SimpleMemory API
#[derive(Clone)]
pub struct SimpleMemory {
    inner: Memory,
}

impl SimpleMemory {
    /// 创建零配置 Memory（自动选择持久化或内存模式）
    pub async fn new() -> Result<Self> {
        let inner = Memory::new().await?;
        Ok(Self { inner })
    }

    /// 添加带元数据的记忆，返回首个事件 ID
    pub async fn add_with_metadata(
        &self,
        content: impl Into<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let mut options = AddMemoryOptions::default();
        if let Some(meta) = metadata {
            options.metadata = meta;
        }

        let result = self.inner.add_with_options(content.into(), options).await?;
        result
            .results
            .first()
            .map(|event| event.id.clone())
            .ok_or_else(|| AgentMemError::internal_error("添加记忆未返回 ID".to_string()))
    }

    /// 简单搜索
    pub async fn search(&self, query: impl Into<String>) -> Result<Vec<MemoryItem>> {
        self.inner.search(query).await
    }

    /// 带 limit 的搜索
    pub async fn search_with_limit(
        &self,
        query: impl Into<String>,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        let mut options = SearchOptions::default();
        options.limit = Some(limit);
        self.inner.search_with_options(query, options).await
    }
}

