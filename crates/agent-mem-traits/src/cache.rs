//! 智能功能缓存 trait 定义
//!
//! 为智能记忆处理提供缓存接口，减少 LLM 调用

use crate::{ExtractedFact, MemoryDecision};
use async_trait::async_trait;

/// 智能功能缓存 trait
#[async_trait]
pub trait IntelligenceCache: Send + Sync {
    /// 获取缓存的事实提取结果
    async fn get_facts(&self, key: &str) -> Option<Vec<ExtractedFact>>;
    
    /// 设置事实提取结果到缓存
    async fn set_facts(&self, key: &str, facts: Vec<ExtractedFact>);
    
    /// 获取缓存的决策结果
    async fn get_decision(&self, key: &str) -> Option<MemoryDecision>;
    
    /// 设置决策结果到缓存
    async fn set_decision(&self, key: &str, decision: MemoryDecision);
    
    /// 清空缓存
    async fn clear(&self);
    
    /// 获取缓存统计信息
    async fn stats(&self) -> CacheStats;
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            size: 0,
            hit_rate: 0.0,
        }
    }
    
    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

