//! 🆕 Phase 2: 智能缓存键构建
//!
//! 提供细粒度的缓存键构建和失效机制，充分复用现有缓存系统

use agent_mem_traits::abstractions::MemoryId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// 智能缓存键构建器
/// 
/// 复用现有缓存键构建逻辑，增强为支持细粒度失效
#[derive(Debug, Clone)]
pub struct SmartCacheKeyBuilder {
    /// 缓存键前缀映射（用于失效）
    key_prefixes: Arc<RwLock<std::collections::HashMap<String, Vec<String>>>>,
}

impl SmartCacheKeyBuilder {
    /// 创建新的智能缓存键构建器
    pub fn new() -> Self {
        Self {
            key_prefixes: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 构建记忆查询缓存键（复用现有逻辑）
    /// 
    /// 格式: `memory:{agent_id}:{user_id}:{scope}:{type}:{query_hash}`
    pub async fn build_memory_query_key(
        &self,
        agent_id: &str,
        user_id: Option<&str>,
        scope: Option<&str>,
        memory_type: Option<&str>,
        query: &str,
    ) -> String {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let query_hash = hasher.finish();

        let user_part = user_id.unwrap_or("_global");
        let scope_part = scope.unwrap_or("_any");
        let type_part = memory_type.unwrap_or("_any");

        let key = format!(
            "memory:{agent_id}:{user_part}:{scope_part}:{type_part}:{query_hash}"
        );

        // 记录键前缀用于失效
        self.record_key_prefix("memory", &key).await;

        key
    }

    /// 构建记忆ID缓存键（复用现有逻辑）
    /// 
    /// 格式: `memory_id:{id}`
    pub async fn build_memory_id_key(&self, id: &MemoryId) -> String {
        let key = format!("memory_id:{}", id.as_str());
        self.record_key_prefix("memory_id", &key).await;
        key
    }

    /// 构建向量搜索缓存键（复用现有逻辑）
    /// 
    /// 格式: `vector_search:{query_hash}:{limit}:{threshold}`
    pub async fn build_vector_search_key(
        &self,
        query_vector: &[f32],
        limit: usize,
        threshold: Option<f32>,
    ) -> String {
        let mut hasher = DefaultHasher::new();
        // 只使用前10个元素以提高性能（复用现有逻辑）
        for &val in query_vector.iter().take(10) {
            val.to_bits().hash(&mut hasher);
        }
        limit.hash(&mut hasher);
        if let Some(t) = threshold {
            t.to_bits().hash(&mut hasher);
        }

        let key = format!("vector_search:{}", hasher.finish());
        self.record_key_prefix("vector_search", &key).await;
        key
    }

    /// 记录键前缀（用于细粒度失效）
    async fn record_key_prefix(&self, prefix: &str, key: &str) {
        let mut prefixes = self.key_prefixes.write().await;
        prefixes
            .entry(prefix.to_string())
            .or_insert_with(Vec::new)
            .push(key.to_string());
    }

    /// 失效指定前缀的所有键
    pub async fn invalidate_by_prefix(&self, prefix: &str) -> usize {
        let mut prefixes = self.key_prefixes.write().await;
        let count = prefixes
            .remove(prefix)
            .map(|keys| keys.len())
            .unwrap_or(0);
        debug!("Invalidated {} cache keys with prefix: {}", count, prefix);
        count
    }

    /// 失效指定记忆ID的所有相关键
    pub async fn invalidate_memory_keys(&self, memory_id: &MemoryId) -> usize {
        // 失效记忆ID键
        let _id_key = self.build_memory_id_key(memory_id).await;
        let mut count = self.invalidate_by_prefix("memory_id").await;

        // 失效相关查询键（通过前缀匹配）
        let mut prefixes = self.key_prefixes.write().await;
        let memory_prefix = "memory:".to_string();
        if let Some(keys) = prefixes.get_mut(&memory_prefix) {
            let before = keys.len();
            keys.retain(|k| !k.contains(memory_id.as_str()));
            count += before - keys.len();
        }

        debug!("Invalidated {} cache keys for memory: {}", count, memory_id.as_str());
        count
    }

    /// 失效指定Agent的所有键
    pub async fn invalidate_agent_keys(&self, agent_id: &str) -> usize {
        let mut prefixes = self.key_prefixes.write().await;
        let mut count = 0;

        // 失效所有包含agent_id的键
        for keys in prefixes.values_mut() {
            let before = keys.len();
            keys.retain(|k| !k.contains(agent_id));
            count += before - keys.len();
        }

        debug!("Invalidated {} cache keys for agent: {}", count, agent_id);
        count
    }

    /// 获取统计信息
    pub async fn stats(&self) -> SmartKeyStats {
        let prefixes = self.key_prefixes.read().await;
        let total_keys = prefixes.values().map(|v| v.len()).sum();
        SmartKeyStats {
            total_keys,
            prefix_count: prefixes.len(),
        }
    }
}

impl Default for SmartCacheKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 智能缓存键统计
#[derive(Debug, Clone)]
pub struct SmartKeyStats {
    /// 总键数
    pub total_keys: usize,
    /// 前缀数量
    pub prefix_count: usize,
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_memory_query_key() {
        let builder = SmartCacheKeyBuilder::new();
        let key1 = builder.build_memory_query_key(
            "agent-1",
            Some("user-1"),
            Some("Global"),
            Some("Episodic"),
            "test query",
        ).await;
        let key2 = builder.build_memory_query_key(
            "agent-1",
            Some("user-1"),
            Some("Global"),
            Some("Episodic"),
            "test query",
        ).await;

        // 相同参数应该生成相同键
        assert_eq!(key1, key2);
        assert!(key1.starts_with("memory:agent-1:user-1:"));
    }

    #[tokio::test]
    async fn test_invalidate_by_prefix() {
        let builder = SmartCacheKeyBuilder::new();
        builder.build_memory_query_key("agent-1", None, None, None, "query1").await;
        builder.build_memory_query_key("agent-1", None, None, None, "query2").await;

        let count = builder.invalidate_by_prefix("memory").await;
        assert!(count >= 2);
    }

    #[tokio::test]
    async fn test_invalidate_memory_keys() {
        let builder = SmartCacheKeyBuilder::new();
        let memory_id = MemoryId::from_string("test-id".to_string());
        
        builder.build_memory_id_key(&memory_id).await;
        let count = builder.invalidate_memory_keys(&memory_id).await;
        assert!(count > 0);
    }
}
