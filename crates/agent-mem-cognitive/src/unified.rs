//! Unified Memory Manager
//! 
//! 统一记忆管理器 - 整合所有记忆组件的单一入口
//! 
//! 特性:
//! - 统一的API接口
//! - 自动层级管理
//! - 智能复习触发
//! - 归档管理

use crate::hierarchy::{MemoryTier, TieredMemoryItem, MemoryHierarchy};
use crate::tiering::{SmartTiering, TieringConfig};
use crate::archive::{ArchiveMemoryManager, ArchiveConfig};
use crate::review::{ReviewTriggerManager, ReviewConfig, ReviewTrigger};
use chrono::Utc;
use std::sync::RwLock;

/// 统一配置
#[derive(Debug, Clone)]
pub struct UnifiedConfig {
    pub hierarchy: HierarchyConfig,
    pub tiering: TieringConfig,
    pub archive: ArchiveConfig,
    pub review: ReviewConfig,
}

#[derive(Debug, Clone)]
pub struct HierarchyConfig {
    pub working_capacity: usize,
    pub core_capacity: usize,
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            hierarchy: HierarchyConfig {
                working_capacity: 100,
                core_capacity: 1000,
            },
            tiering: TieringConfig::default(),
            archive: ArchiveConfig::default(),
            review: ReviewConfig::default(),
        }
    }
}

/// 统一记忆管理器
pub struct UnifiedMemoryManager {
    hierarchy: RwLock<MemoryHierarchy>,
    tiering: SmartTiering,
    archive: ArchiveMemoryManager,
    review: ReviewTriggerManager,
}

impl UnifiedMemoryManager {
    pub fn new(config: UnifiedConfig) -> Self {
        Self {
            hierarchy: RwLock::new(MemoryHierarchy::new(
                config.hierarchy.working_capacity,
                config.hierarchy.core_capacity,
            )),
            tiering: SmartTiering::new(config.tiering),
            archive: ArchiveMemoryManager::new(config.archive),
            review: ReviewTriggerManager::new(config.review),
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(UnifiedConfig::default())
    }
    
    /// 添加记忆
    pub fn add(&self, id: String, content: String, importance: f32) {
        let mut item = TieredMemoryItem::new(id.clone(), content, MemoryTier::Working);
        item.importance = importance;
        
        // 添加到层级
        {
            let mut hierarchy = self.hierarchy.write().unwrap();
            hierarchy.add(item);
        }
        
        // 注册复习
        self.review.register(id, String::new(), importance);
    }
    
    /// 访问记忆
    pub fn access(&self, id: &str) -> Option<String> {
        let mut hierarchy = self.hierarchy.write().unwrap();
        let item = hierarchy.access(id)?;
        item.access_count += 1;
        item.last_accessed = Utc::now().timestamp();
        
        // 记录复习访问
        self.review.record_access(id);
        
        Some(item.content.clone())
    }
    
    /// 搜索记忆
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let hierarchy = self.hierarchy.read().unwrap();
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        // 搜索工作记忆
        for item in hierarchy.get_tier(MemoryTier::Working) {
            if item.content.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    id: item.id.clone(),
                    content: item.content.clone(),
                    tier: MemoryTier::Working,
                    relevance: self.calculate_relevance(item, query),
                });
            }
        }
        
        // 搜索核心记忆
        for item in hierarchy.get_tier(MemoryTier::Core) {
            if item.content.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    id: item.id.clone(),
                    content: item.content.clone(),
                    tier: MemoryTier::Core,
                    relevance: self.calculate_relevance(item, query),
                });
            }
        }
        
        // 搜索归档
        for item in self.archive.search(query, limit, 0.3) {
            results.push(SearchResult {
                id: item.id.clone(),
                content: item.content.clone(),
                tier: MemoryTier::Archive,
                relevance: 0.5,
            });
        }
        
        // 排序
        results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        results.truncate(limit);
        
        results
    }
    
    fn calculate_relevance(&self, item: &TieredMemoryItem, query: &str) -> f32 {
        let mut score = item.importance;
        if item.content.to_lowercase().contains(&query.to_lowercase()) {
            score += 0.3;
        }
        score += (item.access_count as f32 / 100.0).min(0.2);
        score
    }
    
    /// 执行复习
    pub fn review(&self, id: &str) -> bool {
        self.review.review(id)
    }
    
    /// 获取待复习项
    pub fn get_pending_reviews(&self, limit: usize) -> Vec<ReviewTrigger> {
        self.review.get_pending_reviews(limit)
    }
    
    /// 重新平衡层级
    pub fn rebalance(&self) {
        let mut hierarchy = self.hierarchy.write().unwrap();
        self.tiering.rebalance(&mut hierarchy);
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> UnifiedStats {
        let hierarchy = self.hierarchy.read().unwrap();
        let hierarchy_stats = hierarchy.stats();
        let archive_stats = self.archive.stats();
        let review_stats = self.review.stats();
        
        UnifiedStats {
            working_count: hierarchy_stats.working_count,
            core_count: hierarchy_stats.core_count,
            archive_count: archive_stats.total_items,
            pending_reviews: review_stats.pending_reviews,
            avg_retention: review_stats.avg_retention,
        }
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub tier: MemoryTier,
    pub relevance: f32,
}

/// 统一统计
#[derive(Debug, Clone)]
pub struct UnifiedStats {
    pub working_count: usize,
    pub core_count: usize,
    pub archive_count: usize,
    pub pending_reviews: usize,
    pub avg_retention: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_access() {
        let manager = UnifiedMemoryManager::with_defaults();
        
        manager.add("test1".to_string(), "Hello world".to_string(), 0.8);
        
        let result = manager.access("test1");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Hello world");
    }
    
    #[test]
    fn test_search() {
        let manager = UnifiedMemoryManager::with_defaults();
        
        manager.add("test1".to_string(), "Rust programming".to_string(), 0.8);
        manager.add("test2".to_string(), "Python programming".to_string(), 0.7);
        
        let results = manager.search("Rust", 10);
        assert!(!results.is_empty());
        assert!(results[0].content.contains("Rust"));
    }
    
    #[test]
    fn test_stats() {
        let manager = UnifiedMemoryManager::with_defaults();
        
        manager.add("test1".to_string(), "Hello".to_string(), 0.8);
        
        let stats = manager.stats();
        assert_eq!(stats.working_count, 1);
    }
}
