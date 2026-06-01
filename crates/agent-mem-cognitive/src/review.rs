//! Intelligent Review Trigger
//! 
//! 基于遗忘曲线智能触发记忆复习
//! 
//! 机制:
//! - 监测记忆衰减状态
//! - 在最佳时机触发复习
//! - 动态调整复习间隔
//! - 强化重要记忆

use crate::forgetting::{ForgettingCurve, DecayStatus};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;

/// 复习触发器配置
#[derive(Debug, Clone)]
pub struct ReviewConfig {
    /// 触发阈值: 记忆保留率低于此值时触发复习
    pub trigger_threshold: f32,
    /// 最小复习间隔 (秒)
    pub min_review_interval: i64,
    /// 最大复习间隔 (秒)
    pub max_review_interval: i64,
    /// 是否启用自适应间隔
    pub adaptive_interval: bool,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            trigger_threshold: 0.5,      // 50% 保留率时触发
            min_review_interval: 3600,  // 1小时
            max_review_interval: 86400, // 24小时
            adaptive_interval: true,
        }
    }
}

/// 复习项
#[derive(Debug, Clone)]
pub struct ReviewItem {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub decay_status: DecayStatus,
    pub stability: f32,
    pub review_count: u32,
    pub last_reviewed: DateTime<Utc>,
    pub next_review: DateTime<Utc>,
    pub priority: ReviewPriority,
}

/// 复习优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewPriority {
    Critical = 4,  // 即将遗忘
    High = 3,      // 重要但稳定
    Medium = 2,    // 一般
    Low = 1,       // 可以延迟
}

/// 复习触发事件
#[derive(Debug, Clone)]
pub struct ReviewTrigger {
    pub id: String,
    pub priority: ReviewPriority,
    pub retention: f32,
    pub suggested_interval: i64,
    pub reason: String,
}

/// 智能复习触发器
pub struct ReviewTriggerManager {
    config: ReviewConfig,
    items: RwLock<HashMap<String, ReviewItem>>,
}

impl ReviewTriggerManager {
    pub fn new(config: ReviewConfig) -> Self {
        Self {
            config,
            items: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(ReviewConfig::default())
    }
    
    /// 注册记忆用于复习追踪
    pub fn register(&self, id: String, content: String, importance: f32) {
        let mut items = self.items.write().unwrap();
        
        let now = Utc::now();
        let curve = ForgettingCurve::with_stability(1.0);
        let retention = curve.retention_at(0.0);
        let decay_status = curve.status(retention);
        
        let item = ReviewItem {
            id: id.clone(),
            content,
            created_at: now,
            decay_status,
            stability: 1.0,
            review_count: 0,
            last_reviewed: now,
            next_review: now,
            priority: Self::calculate_priority(importance, retention),
        };
        
        items.insert(id, item);
    }
    
    /// 更新记忆访问
    pub fn record_access(&self, id: &str) {
        let mut items = self.items.write().unwrap();
        
        if let Some(item) = items.get_mut(id) {
            let curve = ForgettingCurve::with_stability(item.stability);
            let hours = (Utc::now() - item.created_at).num_hours() as f32;
            let retention = curve.retention_at(hours);
            item.decay_status = curve.status(retention);
        }
    }
    
    /// 执行复习
    pub fn review(&self, id: &str) -> bool {
        let mut items = self.items.write().unwrap();
        
        if let Some(item) = items.get_mut(id) {
            item.review_count += 1;
            item.last_reviewed = Utc::now();
            
            // 根据复习次数调整稳定性 (使用 ForgettingCurve.reinforce)
            let curve = ForgettingCurve::new();
            item.stability = curve.reinforce(item.stability);
            item.stability = item.stability.min(10.0); // 上限
            
            // 计算下次复习间隔
            item.next_review = self.calculate_next_review_time(item);
            
            true
        } else {
            false
        }
    }
    
    /// 获取待复习项
    pub fn get_pending_reviews(&self, limit: usize) -> Vec<ReviewTrigger> {
        let items = self.items.read().unwrap();
        let now = Utc::now();
        
        let mut pending: Vec<_> = items
            .values()
            .filter(|item| item.next_review <= now)
            .map(|item| {
                let curve = ForgettingCurve::with_stability(item.stability);
                let hours = (now - item.created_at).num_hours() as f32;
                let retention = curve.retention_at(hours);
                
                ReviewTrigger {
                    id: item.id.clone(),
                    priority: item.priority,
                    retention,
                    suggested_interval: self.calculate_suggested_interval(item),
                    reason: Self::generate_reason(retention, item.review_count),
                }
            })
            .collect();
        
        // 按优先级和保留率排序
        pending.sort_by(|a, b| {
            let priority_cmp = b.priority.cmp(&a.priority);
            if priority_cmp == std::cmp::Ordering::Equal {
                a.retention.partial_cmp(&b.retention).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                priority_cmp
            }
        });
        
        pending.into_iter().take(limit).collect()
    }
    
    /// 检查是否需要触发复习
    pub fn should_trigger_review(&self, id: &str) -> bool {
        let items = self.items.read().unwrap();
        
        if let Some(item) = items.get(id) {
            let now = Utc::now();
            let curve = ForgettingCurve::with_stability(item.stability);
            let hours = (now - item.created_at).num_hours() as f32;
            let retention = curve.retention_at(hours);
            
            item.next_review <= now || retention < self.config.trigger_threshold
        } else {
            false
        }
    }
    
    /// 计算下次复习时间
    fn calculate_next_review_time(&self, item: &ReviewItem) -> DateTime<Utc> {
        let base_interval = match item.priority {
            ReviewPriority::Critical => chrono::Duration::seconds(self.config.min_review_interval),
            ReviewPriority::High => chrono::Duration::seconds(self.config.min_review_interval * 2),
            ReviewPriority::Medium => chrono::Duration::seconds(self.config.min_review_interval * 4),
            ReviewPriority::Low => chrono::Duration::seconds(self.config.min_review_interval * 8),
        };
        
        // 根据复习次数指数增长
        let multiplier = if self.config.adaptive_interval {
            (2.0_f32).powi(item.review_count.min(5) as i32)
        } else {
            1.0 + item.review_count as f32 * 0.5
        };
        
        let interval_secs = ((base_interval.num_seconds() as f32) * multiplier * item.stability.min(2.0)) as i64;
        let interval_secs = interval_secs.clamp(self.config.min_review_interval, self.config.max_review_interval);
        
        item.last_reviewed + chrono::Duration::seconds(interval_secs)
    }
    
    /// 计算建议复习间隔
    fn calculate_suggested_interval(&self, item: &ReviewItem) -> i64 {
        let base = chrono::Duration::seconds(self.config.min_review_interval * 4);
        let multiplier = (2.0_f32).powi(item.review_count.min(5) as i32);
        ((base.num_seconds() as f32) * multiplier) as i64
    }
    
    /// 计算优先级
    fn calculate_priority(importance: f32, retention: f32) -> ReviewPriority {
        if retention < 0.3 || importance > 0.9 {
            ReviewPriority::Critical
        } else if importance > 0.7 || retention < 0.5 {
            ReviewPriority::High
        } else if importance > 0.4 {
            ReviewPriority::Medium
        } else {
            ReviewPriority::Low
        }
    }
    
    /// 生成原因描述
    fn generate_reason(retention: f32, review_count: u32) -> String {
        if retention < 0.3 {
            "即将遗忘 - 紧急复习".to_string()
        } else if retention < 0.5 {
            "记忆衰减 - 需要强化".to_string()
        } else if review_count == 0 {
            "首次学习 - 建议复习".to_string()
        } else {
            format!("复习周期 {} 次后巩固", review_count)
        }
    }
    
    /// 获取统计
    pub fn stats(&self) -> ReviewStats {
        let items = self.items.read().unwrap();
        let now = Utc::now();
        
        let total = items.len();
        let pending = items.values().filter(|i| i.next_review <= now).count();
        
        let avg_review_count = if total > 0 {
            items.values().map(|i| i.review_count as f32).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        let avg_retention = if total > 0 {
            items.values().map(|i| {
                let curve = ForgettingCurve::with_stability(i.stability);
                let hours = (now - i.created_at).num_hours() as f32;
                curve.retention_at(hours)
            }).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        ReviewStats {
            total_tracked: total,
            pending_reviews: pending,
            avg_review_count,
            avg_retention,
            trigger_threshold: self.config.trigger_threshold,
        }
    }
}

/// 复习统计
#[derive(Debug, Clone)]
pub struct ReviewStats {
    pub total_tracked: usize,
    pub pending_reviews: usize,
    pub avg_review_count: f32,
    pub avg_retention: f32,
    pub trigger_threshold: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let manager = ReviewTriggerManager::with_defaults();
        manager.register("test1".to_string(), "Test content".to_string(), 0.8);
        
        let stats = manager.stats();
        assert_eq!(stats.total_tracked, 1);
    }
    
    #[test]
    fn test_review() {
        let manager = ReviewTriggerManager::with_defaults();
        manager.register("test1".to_string(), "Test content".to_string(), 0.8);
        
        let result = manager.review("test1");
        assert!(result);
        
        let stats = manager.stats();
        assert_eq!(stats.avg_review_count, 1.0);
    }
    
    #[test]
    fn test_pending_reviews() {
        let manager = ReviewTriggerManager::with_defaults();
        manager.register("test1".to_string(), "Test content".to_string(), 0.8);
        
        // 立即检查应该有pending
        let pending = manager.get_pending_reviews(10);
        assert!(!pending.is_empty());
    }
}
