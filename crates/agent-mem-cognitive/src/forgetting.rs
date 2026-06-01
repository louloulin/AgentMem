//! Forgetting Curve Module
//! 
//! Implements the Ebbinghaus forgetting curve for memory decay.
//! Formula: R = e^(-t/S) where R is retention, t is time, S is stability

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

/// Ebbinghaus forgetting curve calculator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingCurve {
    /// Stability constant (S) - determines how fast memory decays
    /// Higher S = slower decay (more stable memory)
    pub stability: f32,
    /// Base retention at t=0 (usually 1.0)
    pub initial_retention: f32,
}

impl ForgettingCurve {
    /// Create new forgetting curve with default stability
    pub fn new() -> Self {
        Self {
            stability: 1.0,
            initial_retention: 1.0,
        }
    }

    /// Create with custom stability
    /// 
    /// - stability = 0.5: Fast decay (e.g., working memory)
    /// - stability = 1.0: Normal decay
    /// - stability = 2.0: Slow decay (e.g., well-practiced skills)
    /// - stability = 5.0+: Very slow decay (e.g., expert knowledge)
    pub fn with_stability(stability: f32) -> Self {
        Self {
            stability: stability.max(0.1),
            initial_retention: 1.0,
        }
    }

    /// Calculate retention at time t (in hours)
    /// 
    /// Formula: R = R0 * e^(-t/S)
    pub fn retention_at(&self, hours_elapsed: f32) -> f32 {
        self.initial_retention * (-hours_elapsed / self.stability).exp()
    }

    /// Get retention based on timestamps
    pub fn retention_since(&self, created_at: DateTime<Utc>) -> f32 {
        let hours = (Utc::now() - created_at).num_hours() as f32;
        self.retention_at(hours)
    }

    /// Calculate optimal review interval
    /// 
    /// Based on desired retention level and current stability
    pub fn optimal_interval(&self, target_retention: f32) -> Duration {
        // t = -S * ln(R/R0)
        let hours = -self.stability * (target_retention / self.initial_retention).ln();
        Duration::hours(hours.max(1.0) as i64)
    }

    /// Update stability based on successful recall
    /// 
    /// Each successful review increases stability (Spaced Repetition)
    pub fn reinforce(&self, current_stability: f32) -> f32 {
        // Stability increases by ~10% per successful review
        current_stability * 1.1
    }
}

impl Default for ForgettingCurve {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory decay status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecayStatus {
    Fresh,        // > 90% retention
    Stable,        // 50-90% retention
    Decaying,      // 20-50% retention
    Critical,      // < 20% retention
    Forgotten,     // < 5% retention
}

impl ForgettingCurve {
    /// Get decay status based on retention
    pub fn status(&self, retention: f32) -> DecayStatus {
        if retention >= 0.9 {
            DecayStatus::Fresh
        } else if retention >= 0.5 {
            DecayStatus::Stable
        } else if retention >= 0.2 {
            DecayStatus::Decaying
        } else if retention >= 0.05 {
            DecayStatus::Critical
        } else {
            DecayStatus::Forgotten
        }
    }

    /// Check if memory needs review
    pub fn needs_review(&self, retention: f32, threshold: f32) -> bool {
        retention < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_calculation() {
        let curve = ForgettingCurve::with_stability(1.0);
        // At t=0, retention should be 1.0
        assert!((curve.retention_at(0.0) - 1.0).abs() < 0.001);
        // At t=1h with S=1, retention ~0.368
        let r = curve.retention_at(1.0);
        assert!(r < 0.5 && r > 0.2);
    }

    #[test]
    fn test_stability_effect() {
        let fast_decay = ForgettingCurve::with_stability(0.5);
        let slow_decay = ForgettingCurve::with_stability(2.0);
        
        let r_fast = fast_decay.retention_at(1.0);
        let r_slow = slow_decay.retention_at(1.0);
        
        // Slow decay should have higher retention
        assert!(r_slow > r_fast);
    }

    #[test]
    fn test_optimal_interval() {
        let curve = ForgettingCurve::with_stability(1.0);
        let interval = curve.optimal_interval(0.5);
        // Should be positive duration
        assert!(interval.num_hours() > 0);
    }

    #[test]
    fn test_reinforcement() {
        let curve = ForgettingCurve::new();
        let new_stability = curve.reinforce(1.0);
        assert!(new_stability > 1.0);
    }

    #[test]
    fn test_decay_status() {
        let curve = ForgettingCurve::new();
        assert_eq!(curve.status(0.95), DecayStatus::Fresh);
        assert_eq!(curve.status(0.7), DecayStatus::Stable);
        assert_eq!(curve.status(0.35), DecayStatus::Decaying);
        assert_eq!(curve.status(0.1), DecayStatus::Critical);
        assert_eq!(curve.status(0.02), DecayStatus::Forgotten);
    }
}
