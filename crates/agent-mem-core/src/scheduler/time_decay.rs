//! Time Decay Models
//!
//! 时间衰减模型，用于计算记忆的新鲜度分数。
//!
//! # 指数衰减模型
//!
//! ```text
//! decay_score = exp(-λ * age_in_days)
//!
//! 其中：
//! - λ (lambda): 衰减率，值越大衰减越快
//! - age_in_days: 记忆的年龄（天数）
//! ```
//!
//! # 示例
//!
//! ```
//! use agent_mem_core::scheduler::ExponentialDecayModel;
//!
//! // 创建衰减率为 0.1 的模型（每天衰减 10%）
//! let model = ExponentialDecayModel::new(0.1);
//!
//! // 1 天前的记忆新鲜度
//! let score = model.decay_score(1.0);  // ≈ 0.90
//!
//! // 10 天前的记忆新鲜度
//! let score = model.decay_score(10.0); // ≈ 0.37
//! ```
//!
//! # 参考文献
//!
//! - MemOS: A Memory OS for AI System (ACL 2025)
//! - Time decay models in recommender systems

use serde::{Deserialize, Serialize};

/// 时间衰减模型 trait
///
/// 定义了计算记忆新鲜度的接口。
pub trait TimeDecayModel: Send + Sync {
    /// 计算衰减分数
    ///
    /// # 参数
    ///
    /// - `age_days`: 记忆的年龄（天数）
    ///
    /// # 返回
    ///
    /// 新鲜度分数（0-1 之间，1 表示最新，0 表示完全衰减）
    fn decay_score(&self, age_days: f64) -> f64;

    /// 获取衰减率
    fn decay_rate(&self) -> f64;
}

/// 指数衰减模型
///
/// 基于指数函数计算时间衰减：
/// ```text
/// score = exp(-λ * age)
/// ```
///
/// 这是最常用的衰减模型，具有良好的数学性质。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialDecayModel {
    /// 衰减率（lambda）
    decay_rate: f64,
}

impl ExponentialDecayModel {
    /// 创建新的指数衰减模型
    ///
    /// # 参数
    ///
    /// - `decay_rate`: 衰减率 λ（0 < λ ≤ 1）
    ///
    /// # 示例
    ///
    /// ```
    /// use agent_mem_core::scheduler::ExponentialDecayModel;
    ///
    /// // 每天衰减 10%
    /// let model = ExponentialDecayModel::new(0.1);
    ///
    /// // 每天衰减 20%（更快衰减）
    /// let model = ExponentialDecayModel::new(0.2);
    /// ```
    pub fn new(decay_rate: f64) -> Self {
        assert!(decay_rate > 0.0, "Decay rate must be positive");
        assert!(decay_rate <= 1.0, "Decay rate must be <= 1.0");

        Self { decay_rate }
    }

    /// 创建默认配置的衰减模型（λ = 0.1）
    ///
    /// 这是推荐配置，平衡了新旧记忆的重要性。
    pub fn default_config() -> Self {
        Self::new(0.1)
    }

    /// 创建慢速衰减模型（λ = 0.05）
    ///
    /// 适用于需要长期记忆的场景。
    pub fn slow_decay() -> Self {
        Self::new(0.05)
    }

    /// 创建快速衰减模型（λ = 0.2）
    ///
    /// 适用于强调最新信息的场景。
    pub fn fast_decay() -> Self {
        Self::new(0.2)
    }
}

impl Default for ExponentialDecayModel {
    fn default() -> Self {
        Self::default_config()
    }
}

impl TimeDecayModel for ExponentialDecayModel {
    fn decay_score(&self, age_days: f64) -> f64 {
        // 指数衰减: exp(-λ * age)
        let score = (-self.decay_rate * age_days).exp();

        // 确保分数在 [0, 1] 范围内
        score.clamp(0.0, 1.0)
    }

    fn decay_rate(&self) -> f64 {
        self.decay_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_decay() {
        let model = ExponentialDecayModel::new(0.1);

        // 0 天前（最新）
        assert!((model.decay_score(0.0) - 1.0).abs() < 0.01);

        // 1 天前
        let score_1day = model.decay_score(1.0);
        assert!((score_1day - 0.90).abs() < 0.01);

        // 10 天前
        let score_10days = model.decay_score(10.0);
        assert!((score_10days - 0.37).abs() < 0.01);

        // 100 天前（几乎完全衰减）
        let score_100days = model.decay_score(100.0);
        assert!(score_100days < 0.01);
    }

    #[test]
    fn test_decay_rates() {
        // 快速衰减（0.2）
        let fast = ExponentialDecayModel::new(0.2);
        let fast_score = fast.decay_score(5.0);

        // 慢速衰减（0.05）
        let slow = ExponentialDecayModel::new(0.05);
        let slow_score = slow.decay_score(5.0);

        // 相同时间下，快速衰减的分数应该更低
        assert!(fast_score < slow_score);
    }

    #[test]
    fn test_score_bounds() {
        let model = ExponentialDecayModel::new(0.1);

        // 测试各种年龄
        for age in [0.0, 1.0, 10.0, 100.0, 1000.0].iter() {
            let score = model.decay_score(*age);
            assert!(score >= 0.0 && score <= 1.0);
        }
    }

    #[test]
    fn test_decay_rate_validation() {
        // 有效的衰减率
        assert!(ExponentialDecayModel::new(0.01).decay_rate() > 0.0);
        assert!(ExponentialDecayModel::new(1.0).decay_rate() <= 1.0);

        // 测试预设配置
        let presets = vec![
            ExponentialDecayModel::default_config(),
            ExponentialDecayModel::slow_decay(),
            ExponentialDecayModel::fast_decay(),
        ];

        for model in presets {
            assert!(model.decay_rate() > 0.0);
            assert!(model.decay_rate() <= 1.0);
        }
    }

    #[test]
    #[should_panic(expected = "Decay rate must be positive")]
    fn test_invalid_decay_rate_zero() {
        ExponentialDecayModel::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Decay rate must be positive")]
    fn test_invalid_decay_rate_negative() {
        ExponentialDecayModel::new(-0.1);
    }

    #[test]
    #[should_panic(expected = "Decay rate must be <= 1.0")]
    fn test_invalid_decay_rate_too_large() {
        ExponentialDecayModel::new(1.5);
    }
}
