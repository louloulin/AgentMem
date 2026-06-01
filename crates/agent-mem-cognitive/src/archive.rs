//! Archive Memory Manager
//! 
//! Manages long-term archived memories with efficient retrieval
//! 
//! Features:
//! - Efficient storage for cold memories
//! - Semantic search in archives
//! - Automatic cleanup policies
//! - Tier transition management

use crate::hierarchy::{MemoryTier, TieredMemoryItem};
use std::collections::HashMap;
use std::sync::RwLock;

/// 归档记忆配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveConfig {
    /// 最大归档数量 (0 = 无限制)
    pub max_items: usize,
    /// 自动清理阈值 (保留最近N个)
    pub retain_recent: usize,
    /// 归档时间戳 (天)
    pub archive_after_days: i64,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            max_items: 100_000,
            retain_recent: 10_000,
            archive_after_days: 30,
        }
    }
}

/// 归档记忆管理器
pub struct ArchiveMemoryManager {
    /// 归档记忆存储
    items: RwLock<HashMap<String, ArchivedItem>>,
    /// 配置
    config: ArchiveConfig,
}

/// 归档记忆项
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchivedItem {
    pub id: String,
    pub content: String,
    pub importance: f32,
    pub access_count: u32,
    pub archived_at: i64,
    pub original_tier: MemoryTier,
    pub tags: Vec<String>,
    /// 摘要/关键词用于快速检索
    pub summary: String,
}

impl ArchivedItem {
    pub fn from_tiered(item: &TieredMemoryItem, archived_at: i64, original_tier: MemoryTier) -> Self {
        Self {
            id: item.id.clone(),
            content: item.content.clone(),
            importance: item.importance,
            access_count: item.access_count,
            archived_at,
            original_tier,
            tags: Vec::new(),
            summary: Self::generate_summary(&item.content),
        }
    }
    
    fn generate_summary(content: &str) -> String {
        // 简单摘要: 取前100个字符
        let summary = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.to_string()
        };
        summary
    }
    
    /// 检索评分 (用于相关性排序)
    pub fn relevance_score(&self, query: &str, query_importance: f32) -> f32 {
        let content_lower = self.content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        // 基础分数
        let mut score = self.importance * 0.3;
        
        // 相关性分数
        if content_lower.contains(&query_lower) {
            score += 0.5;
        }
        
        // 访问频率分数
        score += (self.access_count as f32 / 100.0).min(0.2);
        
        // 时间衰减 (越新分数越高)
        let now = chrono::Utc::now().timestamp();
        let days_old = (now - self.archived_at) / 86400;
        let recency = ((30 - days_old).max(0) as f32 / 30.0) * 0.2;
        score += recency;
        
        // 重要性加权
        score += query_importance * 0.1;
        
        score
    }
}

impl ArchiveMemoryManager {
    pub fn new(config: ArchiveConfig) -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            config,
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(ArchiveConfig::default())
    }
    
    /// 添加到归档
    pub fn archive(&self, item: ArchivedItem) {
        let mut items = self.items.write().unwrap();
        
        // 检查容量
        if self.config.max_items > 0 && items.len() >= self.config.max_items {
            // 执行清理
            self.cleanup_internal(&mut items);
        }
        
        items.insert(item.id.clone(), item);
    }
    
    /// 从 MemoryTier 转换
    pub fn archive_from_tiered(&self, tiered: &TieredMemoryItem, original_tier: MemoryTier) {
        let archived = ArchivedItem::from_tiered(
            tiered,
            chrono::Utc::now().timestamp(),
            original_tier,
        );
        self.archive(archived);
    }
    
    /// 搜索归档记忆
    pub fn search(&self, query: &str, limit: usize, min_score: f32) -> Vec<ArchivedItem> {
        let items = self.items.read().unwrap();
        let mut results: Vec<_> = items
            .values()
            .map(|item| {
                let score = item.relevance_score(query, 0.5);
                (score, item.clone())
            })
            .filter(|(score, _)| *score >= min_score)
            .collect();
        
        // 按分数排序
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        results
            .into_iter()
            .take(limit)
            .map(|(_, item)| item)
            .collect()
    }
    
    /// 获取最近归档
    pub fn get_recent(&self, limit: usize) -> Vec<ArchivedItem> {
        let items = self.items.read().unwrap();
        let mut all: Vec<_> = items.values().cloned().collect();
        
        // 按归档时间排序
        all.sort_by_key(|b| std::cmp::Reverse(b.archived_at));
        
        all.into_iter().take(limit).collect()
    }
    
    /// 恢复记忆 (从归档移回活跃)
    pub fn restore(&self, id: &str) -> Option<ArchivedItem> {
        let mut items = self.items.write().unwrap();
        items.remove(id)
    }
    
    /// 删除归档项
    pub fn delete(&self, id: &str) -> bool {
        let mut items = self.items.write().unwrap();
        items.remove(id).is_some()
    }
    
    /// 获取统计
    pub fn stats(&self) -> ArchiveStats {
        let items = self.items.read().unwrap();
        let total = items.len();
        
        let total_importance: f32 = items.values().map(|i| i.importance).sum();
        let avg_importance = if total > 0 { total_importance / total as f32 } else { 0.0 };
        
        let total_access: u64 = items.values().map(|i| i.access_count as u64).sum();
        let avg_access = if total > 0 { total_access / total as u64 } else { 0 };
        
        ArchiveStats {
            total_items: total,
            avg_importance,
            avg_access_count: avg_access as u32,
            capacity: self.config.max_items,
            utilization: if self.config.max_items > 0 {
                total as f32 / self.config.max_items as f32
            } else {
                0.0
            },
        }
    }
    
    /// 清理过期项
    fn cleanup_internal(&self, items: &mut HashMap<String, ArchivedItem>) {
        let retain = self.config.retain_recent;
        if retain == 0 {
            return;
        }
        
        if items.len() <= retain {
            return;
        }
        
        // 收集所有项并排序
        let mut all: Vec<_> = items.iter().collect();
        all.sort_by_key(|b| std::cmp::Reverse(b.1.archived_at));
        
        // 保留最近的
        let to_keep: Vec<_> = all.into_iter().take(retain).map(|(k, _)| k.clone()).collect();
        
        // 删除其他的
        items.retain(|k, _| to_keep.contains(k));
    }
    
    /// 清理 (公开方法)
    pub fn cleanup(&self) {
        let mut items = self.items.write().unwrap();
        self.cleanup_internal(&mut items);
    }
}

/// 归档统计
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveStats {
    pub total_items: usize,
    pub avg_importance: f32,
    pub avg_access_count: u32,
    pub capacity: usize,
    pub utilization: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::TieredMemoryItem;

    #[test]
    fn test_archive_basic() {
        let manager = ArchiveMemoryManager::with_defaults();
        
        let tiered = TieredMemoryItem::new(
            "test1".to_string(),
            "Test content".to_string(),
            MemoryTier::Core,
        );
        
        manager.archive_from_tiered(&tiered, MemoryTier::Core);
        
        let stats = manager.stats();
        assert_eq!(stats.total_items, 1);
    }
    
    #[test]
    fn test_search() {
        let manager = ArchiveMemoryManager::with_defaults();
        
        let tiered = TieredMemoryItem::new(
            "test1".to_string(),
            "Rust programming language tutorial".to_string(),
            MemoryTier::Core,
        );
        
        manager.archive_from_tiered(&tiered, MemoryTier::Core);
        
        let results = manager.search("Rust", 10, 0.3);
        assert!(!results.is_empty());
    }
    
    #[test]
    fn test_restore() {
        let manager = ArchiveMemoryManager::with_defaults();
        
        let tiered = TieredMemoryItem::new(
            "test1".to_string(),
            "Test content".to_string(),
            MemoryTier::Core,
        );
        
        manager.archive_from_tiered(&tiered, MemoryTier::Core);
        
        let restored = manager.restore("test1");
        assert!(restored.is_some());
        
        let stats = manager.stats();
        assert_eq!(stats.total_items, 0);
    }
}
