//! Smart Tiering Module
//! 
//! 实现智能分层器 - 基于重要性自动调整记忆层级
//! 
//! 策略:
//! 1. 高频访问 + 高重要性 -> 晋升到核心
//! 2. 低频访问 + 低重要性 -> 降级到归档
//! 3. 新记忆 -> 进入工作层

use crate::hierarchy::{MemoryTier, TieredMemoryItem, MemoryHierarchy};

/// 智能分层器配置
#[derive(Debug, Clone)]
pub struct TieringConfig {
    /// 晋升阈值: 访问次数
    pub promote_access_threshold: u32,
    /// 晋升阈值: 重要性
    pub promote_importance_threshold: f32,
    /// 降级阈值: 访问次数
    pub demote_access_threshold: u32,
    /// 降级阈值: 重要性
    pub demote_importance_threshold: f32,
    /// 降级阈值: 距上次访问天数
    pub demote_days_threshold: i64,
}

impl Default for TieringConfig {
    fn default() -> Self {
        Self {
            promote_access_threshold: 10,
            promote_importance_threshold: 0.7,
            demote_access_threshold: 2,
            demote_importance_threshold: 0.3,
            demote_days_threshold: 7,
        }
    }
}

/// 智能分层器
pub struct SmartTiering {
    config: TieringConfig,
}

impl SmartTiering {
    pub fn new(config: TieringConfig) -> Self {
        Self { config }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(TieringConfig::default())
    }
    
    /// 评估记忆是否应该晋升
    pub fn should_promote(&self, item: &TieredMemoryItem) -> bool {
        item.access_count >= self.config.promote_access_threshold
            && item.importance >= self.config.promote_importance_threshold
    }
    
    /// 评估记忆是否应该降级
    pub fn should_demote(&self, item: &TieredMemoryItem) -> bool {
        item.access_count <= self.config.demote_access_threshold
            && (item.importance <= self.config.demote_importance_threshold
                || self.is_stale(item))
    }
    
    /// 检查记忆是否过期
    fn is_stale(&self, item: &TieredMemoryItem) -> bool {
        let now = chrono::Utc::now().timestamp();
        let days_since_access = (now - item.last_accessed) / 86400;
        days_since_access >= self.demote_days_threshold()
    }
    
    pub fn demote_days_threshold(&self) -> i64 {
        self.config.demote_days_threshold
    }
    
    /// 智能分层 - 对整个层级系统进行评估和调整
    pub fn rebalance(&self, hierarchy: &mut MemoryHierarchy) {
        // 评估工作记忆中的项
        let working_items: Vec<_> = hierarchy.get_tier(MemoryTier::Working).to_vec();
        for item in working_items {
            if self.should_promote(&item) {
                let mut promoted = item.clone();
                promoted.tier = MemoryTier::Core;
                hierarchy.add(promoted);
            } else if self.should_demote(&item) {
                let mut demoted = item.clone();
                demoted.tier = MemoryTier::Archive;
                hierarchy.add(demoted);
            }
        }
        
        // 评估核心记忆中的项
        let core_items: Vec<_> = hierarchy.get_tier(MemoryTier::Core).to_vec();
        for item in core_items {
            if self.should_demote(&item) {
                let mut demoted = item.clone();
                demoted.tier = MemoryTier::Archive;
                hierarchy.add(demoted);
            }
        }
    }
    
    /// 计算记忆的推荐层级
    pub fn recommend_tier(&self, item: &TieredMemoryItem) -> MemoryTier {
        if self.should_promote(item) {
            MemoryTier::Core
        } else if self.should_demote(item) {
            MemoryTier::Archive
        } else {
            item.tier
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promote_threshold() {
        let tiering = SmartTiering::with_defaults();
        
        let mut item = TieredMemoryItem::new("1".to_string(), "test".to_string(), MemoryTier::Working);
        item.access_count = 15;
        item.importance = 0.8;
        
        assert!(tiering.should_promote(&item));
    }
    
    #[test]
    fn test_demote_threshold() {
        let tiering = SmartTiering::with_defaults();
        
        let mut item = TieredMemoryItem::new("1".to_string(), "test".to_string(), MemoryTier::Core);
        item.access_count = 1;
        item.importance = 0.2;
        
        assert!(tiering.should_demote(&item));
    }
    
    #[test]
    fn test_recommend_tier() {
        let tiering = SmartTiering::with_defaults();
        
        let mut item = TieredMemoryItem::new("1".to_string(), "test".to_string(), MemoryTier::Working);
        item.access_count = 15;
        item.importance = 0.8;
        
        assert_eq!(tiering.recommend_tier(&item), MemoryTier::Core);
    }
}
