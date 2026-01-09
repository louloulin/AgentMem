//! Ebbinghaus Forgetting Curve
//!
//! Implementation of the Ebbinghaus forgetting curve based on cognitive science research.
//!
//! # Theory
//!
//! The Ebbinghaus forgetting curve describes the exponential decline of memory retention
//! over time. The formula is:
//!
//! ```text
//! R(t) = e^(-t/S)
//!
//! where:
//! - R(t) = retention rate at time t
//! - t = time (same unit as S)
//! - S = strength of memory (time when retention is 1/e ≈ 36.8%)
//! ```
//!
//! # Example
//!
//! ```
//! use agent_mem_forgetting::EbbinghausCurve;
//!
//! // Create curve with memory strength of 1 day
//! let curve = EbbinghausCurve::with_strength(1.0);
//!
//! // Retention after 1 day
//! let retention = curve.retention(1.0); // ≈ 0.368 (36.8%)
//!
//! // Retention after 2 days
//! let retention = curve.retention(2.0); // ≈ 0.135 (13.5%)
//! ```

use serde::{Deserialize, Serialize};
use std::f64::consts::E;

/// Forgetting curve trait
///
/// Defines how memory retention changes over time.
pub trait ForgettingCurve: Send + Sync {
    /// Calculate retention rate at given time
    ///
    /// # Parameters
    ///
    /// - `time_units`: Time elapsed since memory creation (in same unit as strength)
    ///
    /// # Returns
    ///
    /// Retention rate (0-1, where 1 = perfect retention, 0 = completely forgotten)
    fn retention(&self, time_units: f64) -> f64;

    /// Check if memory should be forgotten
    ///
    /// # Parameters
    ///
    /// - `time_units`: Time elapsed since memory creation
    /// - `threshold`: Minimum retention rate to keep memory (default 0.1 = 10%)
    ///
    /// # Returns
    ///
    /// True if memory should be forgotten
    fn should_forget(&self, time_units: f64, threshold: f64) -> bool {
        self.retention(time_units) < threshold
    }

    /// Get curve parameters for debugging
    fn parameters(&self) -> ForgettingCurveParams;
}

/// Parameters for forgetting curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingCurveParams {
    /// Curve type
    pub curve_type: String,

    /// Key parameters
    pub params: Vec<(String, f64)>,
}

/// Ebbinghaus forgetting curve
///
/// Based on Hermann Ebbinghaus's pioneering research (1885).
/// The formula is: R(t) = e^(-t/S)
///
/// # Parameters
///
/// - `strength` (S): Memory strength, defined as time when retention drops to 1/e ≈ 36.8%
///   - Higher strength = slower forgetting
///   - Typical values: 1.0 (weak) to 30.0 (strong)
///
/// # Example
///
/// ```
/// use agent_mem_forgetting::EbbinghausCurve;
///
/// // Weak memory (forgets in 1 day)
/// let weak = EbbinghausCurve::with_strength(1.0);
/// assert_eq!(weak.retention(1.0), 0.367); // ≈ 1/e
///
/// // Strong memory (takes 7 days to forget to 36.8%)
/// let strong = EbbinghausCurve::with_strength(7.0);
/// assert_eq!(strong.retention(7.0), 0.367); // ≈ 1/e
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbbinghausCurve {
    /// Memory strength (S)
    ///
    /// Time when retention drops to 1/e ≈ 36.8%
    strength: f64,
}

impl EbbinghausCurve {
    /// Create new Ebbinghaus curve with specific strength
    ///
    /// # Parameters
    ///
    /// - `strength`: Memory strength S (must be > 0)
    ///
    /// # Panics
    ///
    /// Panics if strength <= 0
    pub fn with_strength(strength: f64) -> Self {
        assert!(strength > 0.0, "Memory strength must be positive");
        Self { strength }
    }

    /// Create weak memory curve (strength = 1 day)
    ///
    /// Suitable for temporary working memory
    pub fn weak() -> Self {
        Self::with_strength(1.0)
    }

    /// Create normal memory curve (strength = 7 days)
    ///
    /// Typical for episodic memories
    pub fn normal() -> Self {
        Self::with_strength(7.0)
    }

    /// Create strong memory curve (strength = 30 days)
    ///
    /// Suitable for important semantic memories
    pub fn strong() -> Self {
        Self::with_strength(30.0)
    }

    /// Get memory strength
    pub fn strength(&self) -> f64 {
        self.strength
    }

    /// Calculate time when retention will drop below threshold
    ///
    /// # Parameters
    ///
    /// - `threshold`: Target retention rate (0-1)
    ///
    /// # Returns
    ///
    /// Time units when retention drops below threshold
    pub fn time_to_threshold(&self, threshold: f64) -> f64 {
        assert!(threshold > 0.0 && threshold < 1.0, "Threshold must be in (0, 1)");
        -threshold.ln() * self.strength
    }
}

impl ForgettingCurve for EbbinghausCurve {
    fn retention(&self, time_units: f64) -> f64 {
        // R(t) = e^(-t/S)
        let retention = (-time_units / self.strength).exp();

        // Clamp to [0, 1]
        retention.clamp(0.0, 1.0)
    }

    fn parameters(&self) -> ForgettingCurveParams {
        ForgettingCurveParams {
            curve_type: "Ebbinghaus".to_string(),
            params: vec![("strength".to_string(), self.strength)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebbinghaus_basic() {
        let curve = EbbinghausCurve::with_strength(1.0);

        // At t=0, retention should be 1.0 (perfect)
        assert!((curve.retention(0.0) - 1.0).abs() < 0.01);

        // At t=S, retention should be 1/e ≈ 0.368
        let retention = curve.retention(1.0);
        assert!((retention - 1.0 / E).abs() < 0.01);
    }

    #[test]
    fn test_ebbinghaus_weak() {
        let curve = EbbinghausCurve::weak();

        // After 1 day, retention drops to 36.8%
        assert!((curve.retention(1.0) - 0.368).abs() < 0.01);

        // After 2 days, retention drops to ~13.5%
        assert!((curve.retention(2.0) - 0.135).abs() < 0.01);
    }

    #[test]
    fn test_ebbinghaus_normal() {
        let curve = EbbinghausCurve::normal();

        // After 7 days, retention drops to 36.8%
        assert!((curve.retention(7.0) - 0.368).abs() < 0.01);

        // After 1 day, retention is still high (~86.5%)
        let retention = curve.retention(1.0);
        assert!((retention - (-1.0 / 7.0).exp()).abs() < 0.01);
    }

    #[test]
    fn test_ebbinghaus_strong() {
        let curve = EbbinghausCurve::strong();

        // After 30 days, retention drops to 36.8%
        assert!((curve.retention(30.0) - 0.368).abs() < 0.01);

        // After 7 days, retention is still high (~79%)
        let retention = curve.retention(7.0);
        assert!(retention > 0.75);
    }

    #[test]
    fn test_should_forget() {
        let curve = EbbinghausCurve::weak();

        // At 1 day, retention is 36.8% (above 10% threshold)
        assert!(!curve.should_forget(1.0, 0.1));

        // At 3 days, retention is ~5% (below 10% threshold)
        assert!(curve.should_forget(3.0, 0.1));
    }

    #[test]
    fn test_time_to_threshold() {
        let curve = EbbinghausCurve::with_strength(1.0);

        // Time to reach 10% retention
        let time = curve.time_to_threshold(0.1);
        assert!(time > 2.0 && time < 3.0); // Should be ~2.3 days

        // Time to reach 36.8% retention (1/e)
        let time = curve.time_to_threshold(1.0 / E);
        assert!((time - 1.0).abs() < 0.01); // Should be exactly 1.0
    }

    #[test]
    fn test_parameters() {
        let curve = EbbinghausCurve::with_strength(5.0);
        let params = curve.parameters();

        assert_eq!(params.curve_type, "Ebbinghaus");
        assert_eq!(params.params.len(), 1);
        assert_eq!(params.params[0].0, "strength");
        assert_eq!(params.params[0].1, 5.0);
    }

    #[test]
    #[should_panic(expected = "Memory strength must be positive")]
    fn test_invalid_strength() {
        EbbinghausCurve::with_strength(0.0);
    }

    #[test]
    #[should_panic(expected = "Threshold must be in (0, 1)")]
    fn test_invalid_threshold_high() {
        let curve = EbbinghausCurve::weak();
        curve.time_to_threshold(1.0);
    }

    #[test]
    #[should_panic(expected = "Threshold must be in (0, 1)")]
    fn test_invalid_threshold_zero() {
        let curve = EbbinghausCurve::weak();
        curve.time_to_threshold(0.0);
    }
}
