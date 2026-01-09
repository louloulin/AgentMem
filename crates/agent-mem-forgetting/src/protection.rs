//! Memory Protection Mechanism
//!
//! Protection levels for memories to prevent important ones from being forgotten.
//!
//! # Theory
//!
//! Not all memories should be forgotten equally. Important memories (e.g., user preferences,
//! critical context, frequently accessed information) should be protected from the normal
//! forgetting process.
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_forgetting::protection::{MemoryProtection, ProtectionLevel};
//!
//! let protection = MemoryProtection::new();
//!
//! // Protect critical memory
//! protection.set_protection("memory-123", ProtectionLevel::Critical);
//!
//! // Check if protected
//! if protection.is_protected("memory-123") {
//!     println!("This memory won't be forgotten");
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Protection level for memories
///
/// Determines how resistant a memory is to forgetting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProtectionLevel {
    /// No protection - will be forgotten normally
    None = 0,

    /// Low protection - delays forgetting by 2x
    Low = 1,

    /// Medium protection - delays forgetting by 5x
    Medium = 2,

    /// High protection - delays forgetting by 10x
    High = 3,

    /// Critical protection - never forget automatically
    Critical = 4,
}

impl ProtectionLevel {
    /// Get protection multiplier for forgetting time
    ///
    /// # Returns
    ///
    /// Multiplier for time before forgetting (e.g., 2.0 = 2x longer)
    pub fn multiplier(&self) -> f64 {
        match self {
            ProtectionLevel::None => 1.0,
            ProtectionLevel::Low => 2.0,
            ProtectionLevel::Medium => 5.0,
            ProtectionLevel::High => 10.0,
            ProtectionLevel::Critical => f64::MAX,
        }
    }

    /// Check if this level prevents automatic forgetting
    ///
    /// # Returns
    ///
    /// True if memory should never be automatically forgotten
    pub fn is_permanent(&self) -> bool {
        *self == ProtectionLevel::Critical
    }

    /// Get all protection levels
    pub fn all() -> Vec<ProtectionLevel> {
        vec![
            ProtectionLevel::None,
            ProtectionLevel::Low,
            ProtectionLevel::Medium,
            ProtectionLevel::High,
            ProtectionLevel::Critical,
        ]
    }
}

/// Memory protection manager
///
/// Manages protection levels for memories.
pub struct MemoryProtection {
    /// Memory ID -> Protection level mapping
    protections: Arc<RwLock<HashMap<String, ProtectionLevel>>>,

    /// Default protection level for new memories
    default_level: ProtectionLevel,
}

impl MemoryProtection {
    /// Create new memory protection manager
    ///
    /// # Parameters
    ///
    /// - `default_level`: Default protection level for unprotected memories
    pub fn new() -> Self {
        Self {
            protections: Arc::new(RwLock::new(HashMap::new())),
            default_level: ProtectionLevel::None,
        }
    }

    /// Create with custom default protection level
    pub fn with_default(default_level: ProtectionLevel) -> Self {
        Self {
            protections: Arc::new(RwLock::new(HashMap::new())),
            default_level,
        }
    }

    /// Set protection level for a memory
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to protect
    /// - `level`: Protection level
    pub async fn set_protection(&self, memory_id: String, level: ProtectionLevel) {
        let mut protections = self.protections.write().await;
        protections.insert(memory_id, level);
    }

    /// Get protection level for a memory
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to check
    ///
    /// # Returns
    ///
    /// Protection level (or default if not set)
    pub async fn get_protection(&self, memory_id: &str) -> ProtectionLevel {
        let protections = self.protections.read().await;
        protections
            .get(memory_id)
            .copied()
            .unwrap_or(self.default_level)
    }

    /// Check if memory is protected
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to check
    ///
    /// # Returns
    ///
    /// True if memory has any protection level > None
    pub async fn is_protected(&self, memory_id: &str) -> bool {
        self.get_protection(memory_id).await > ProtectionLevel::None
    }

    /// Check if memory is permanently protected
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to check
    ///
    /// # Returns
    ///
    /// True if memory should never be automatically forgotten
    pub async fn is_permanently_protected(&self, memory_id: &str) -> bool {
        self.get_protection(memory_id).await.is_permanent()
    }

    /// Remove protection from memory
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to unprotect
    pub async fn remove_protection(&self, memory_id: &str) {
        let mut protections = self.protections.write().await;
        protections.remove(memory_id);
    }

    /// Clear all protections
    pub async fn clear_all(&self) {
        let mut protections = self.protections.write().await;
        protections.clear();
    }

    /// Get count of protected memories
    pub async fn protected_count(&self) -> usize {
        let protections = self.protections.read().await;
        protections.len()
    }

    /// Get all protected memory IDs with their levels
    pub async fn all_protections(&self) -> Vec<(String, ProtectionLevel)> {
        let protections = self.protections.read().await;
        protections
            .iter()
            .map(|(id, level)| (id.clone(), *level))
            .collect()
    }

    /// Get memories by protection level
    pub async fn by_level(&self, level: ProtectionLevel) -> Vec<String> {
        let protections = self.protections.read().await;
        protections
            .iter()
            .filter(|(_, l)| *l == &level)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Calculate effective time for forgetting
    ///
    /// Adjusts time based on protection level.
    ///
    /// # Parameters
    ///
    /// - `memory_id`: Memory ID to check
    /// - `base_time`: Base time before forgetting
    ///
    /// # Returns
    ///
    /// Adjusted time (multiplied by protection level)
    pub async fn effective_forgetting_time(&self, memory_id: &str, base_time: f64) -> f64 {
        let level = self.get_protection(memory_id).await;
        if level.is_permanent() {
            return f64::MAX;
        }
        base_time * level.multiplier()
    }
}

impl Clone for MemoryProtection {
    fn clone(&self) -> Self {
        Self {
            protections: Arc::clone(&self.protections),
            default_level: self.default_level,
        }
    }
}

impl Default for MemoryProtection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protection_levels() {
        assert_eq!(ProtectionLevel::None.multiplier(), 1.0);
        assert_eq!(ProtectionLevel::Low.multiplier(), 2.0);
        assert_eq!(ProtectionLevel::Medium.multiplier(), 5.0);
        assert_eq!(ProtectionLevel::High.multiplier(), 10.0);
        assert_eq!(ProtectionLevel::Critical.multiplier(), f64::MAX);
    }

    #[tokio::test]
    async fn test_permanent_protection() {
        assert!(!ProtectionLevel::High.is_permanent());
        assert!(ProtectionLevel::Critical.is_permanent());
    }

    #[tokio::test]
    async fn test_set_protection() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::High).await;

        let level = protection.get_protection("mem-1").await;
        assert_eq!(level, ProtectionLevel::High);
    }

    #[tokio::test]
    async fn test_default_protection() {
        let protection = MemoryProtection::new();
        let level = protection.get_protection("unprotected").await;
        assert_eq!(level, ProtectionLevel::None);
    }

    #[tokio::test]
    async fn test_is_protected() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::Low).await;

        assert!(protection.is_protected("mem-1").await);
        assert!(!protection.is_protected("unprotected").await);
    }

    #[tokio::test]
    async fn test_remove_protection() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::High).await;

        assert!(protection.is_protected("mem-1").await);

        protection.remove_protection("mem-1").await;
        assert!(!protection.is_protected("mem-1").await);
    }

    #[tokio::test]
    async fn test_permanent_protection_check() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::Critical).await;

        assert!(protection.is_permanently_protected("mem-1").await);
        assert!(!protection.is_permanently_protected("unprotected").await);
    }

    #[tokio::test]
    async fn test_effective_forgetting_time() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::Medium).await;

        // Medium protection = 5x multiplier
        let time = protection.effective_forgetting_time("mem-1", 10.0).await;
        assert_eq!(time, 50.0);
    }

    #[tokio::test]
    async fn test_permanent_forgetting_time() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::Critical).await;

        let time = protection.effective_forgetting_time("mem-1", 10.0).await;
        assert_eq!(time, f64::MAX);
    }

    #[tokio::test]
    async fn test_by_level() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::High).await;
        protection.set_protection("mem-2".to_string(), ProtectionLevel::High).await;
        protection.set_protection("mem-3".to_string(), ProtectionLevel::Low).await;

        let high_memories = protection.by_level(ProtectionLevel::High).await;
        assert_eq!(high_memories.len(), 2);
        assert!(high_memories.contains(&"mem-1".to_string()));
        assert!(high_memories.contains(&"mem-2".to_string()));
    }

    #[tokio::test]
    async fn test_clear_all() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::High).await;
        protection.set_protection("mem-2".to_string(), ProtectionLevel::Low).await;

        assert_eq!(protection.protected_count().await, 2);

        protection.clear_all().await;
        assert_eq!(protection.protected_count().await, 0);
    }

    #[tokio::test]
    async fn test_all_protections() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::High).await;
        protection.set_protection("mem-2".to_string(), ProtectionLevel::Low).await;

        let all = protection.all_protections().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_custom_default() {
        let protection = MemoryProtection::with_default(ProtectionLevel::Medium);
        let level = protection.get_protection("unprotected").await;
        assert_eq!(level, ProtectionLevel::Medium);
    }

    #[tokio::test]
    async fn test_clone() {
        let protection = MemoryProtection::new();
        protection.set_protection("mem-1".to_string(), ProtectionLevel::High).await;

        let cloned = protection.clone();
        assert!(cloned.is_protected("mem-1").await);
    }
}
