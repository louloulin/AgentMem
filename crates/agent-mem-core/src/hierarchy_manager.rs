//! Advanced Hierarchical Memory Structure Management
//!
//! Provides sophisticated memory hierarchy management with dynamic
//! structure adjustment and optimization capabilities.

use crate::hierarchical_service::{HierarchicalMemoryRecord, HierarchicalMemoryService};
use crate::hierarchy::{MemoryLevel, MemoryScope};
use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Advanced hierarchy management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyManagerConfig {
    /// Maximum depth of memory hierarchy
    pub max_hierarchy_depth: usize,
    /// Threshold for automatic hierarchy restructuring
    pub restructure_threshold: f64,
    /// Enable automatic hierarchy optimization
    pub enable_auto_optimization: bool,
    /// Hierarchy rebalancing interval in hours
    pub rebalance_interval_hours: u64,
    /// Maximum memories per hierarchy node
    pub max_memories_per_node: usize,
    /// Enable hierarchy compression
    pub enable_compression: bool,
}

impl Default for HierarchyManagerConfig {
    fn default() -> Self {
        Self {
            max_hierarchy_depth: 10,
            restructure_threshold: 0.8,
            enable_auto_optimization: true,
            rebalance_interval_hours: 24,
            max_memories_per_node: 1000,
            enable_compression: true,
        }
    }
}

/// Hierarchy node representing a structural unit in memory hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyNode {
    pub id: String,
    pub scope: MemoryScope,
    pub level: MemoryLevel,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub memory_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_count: u64,
    pub importance_score: f64,
    pub compression_ratio: f64,
    pub metadata: HashMap<String, String>,
}

impl HierarchyNode {
    pub fn new(scope: MemoryScope, level: MemoryLevel) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            scope,
            level,
            parent_id: None,
            children_ids: Vec::new(),
            memory_ids: Vec::new(),
            created_at: now,
            updated_at: now,
            access_count: 0,
            importance_score: 0.0,
            compression_ratio: 1.0,
            metadata: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child_id: String) {
        if !self.children_ids.contains(&child_id) {
            self.children_ids.push(child_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn add_memory(&mut self, memory_id: String) {
        if !self.memory_ids.contains(&memory_id) {
            self.memory_ids.push(memory_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn calculate_depth(&self, hierarchy: &HashMap<String, HierarchyNode>) -> usize {
        let mut depth = 0;
        let mut current_id = self.parent_id.as_ref();

        while let Some(parent_id) = current_id {
            if let Some(parent) = hierarchy.get(parent_id) {
                depth += 1;
                current_id = parent.parent_id.as_ref();
            } else {
                break;
            }
        }

        depth
    }
}

/// Hierarchy statistics for analysis and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyStatistics {
    pub total_nodes: usize,
    pub total_memories: usize,
    pub max_depth: usize,
    pub average_depth: f64,
    pub nodes_per_level: HashMap<MemoryLevel, usize>,
    pub memories_per_scope: HashMap<String, usize>,
    pub compression_efficiency: f64,
    pub access_patterns: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

/// Advanced hierarchical memory structure manager
pub struct HierarchyManager {
    config: HierarchyManagerConfig,
    hierarchy_service: Arc<HierarchicalMemoryService>,
    nodes: Arc<RwLock<HashMap<String, HierarchyNode>>>,
    scope_index: Arc<RwLock<HashMap<MemoryScope, Vec<String>>>>,
    level_index: Arc<RwLock<HashMap<MemoryLevel, Vec<String>>>>,
    statistics: Arc<RwLock<HierarchyStatistics>>,
    last_optimization: Arc<RwLock<DateTime<Utc>>>,
}

impl HierarchyManager {
    /// Create a new hierarchy manager
    pub async fn new(
        config: HierarchyManagerConfig,
        hierarchy_service: Arc<HierarchicalMemoryService>,
    ) -> Result<Self> {
        let manager = Self {
            config,
            hierarchy_service,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            scope_index: Arc::new(RwLock::new(HashMap::new())),
            level_index: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(HierarchyStatistics {
                total_nodes: 0,
                total_memories: 0,
                max_depth: 0,
                average_depth: 0.0,
                nodes_per_level: HashMap::new(),
                memories_per_scope: HashMap::new(),
                compression_efficiency: 1.0,
                access_patterns: HashMap::new(),
                last_updated: Utc::now(),
            })),
            last_optimization: Arc::new(RwLock::new(Utc::now())),
        };

        // Initialize with default hierarchy structure
        manager.initialize_default_hierarchy().await?;

        Ok(manager)
    }

    /// Add a memory to the hierarchy with automatic placement
    pub async fn add_memory_to_hierarchy(
        &self,
        memory: &HierarchicalMemoryRecord,
    ) -> Result<String> {
        // Find optimal placement in hierarchy
        let optimal_node_id = self.find_optimal_placement(memory).await?;

        // Add memory to the node
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(&optimal_node_id) {
                node.add_memory(memory.id.clone());
                node.access_count += 1;
            }
        }

        // Update indices and statistics
        self.update_indices_after_addition(memory).await?;
        self.update_statistics().await?;

        // Check if optimization is needed
        if self.config.enable_auto_optimization {
            self.check_and_optimize().await?;
        }

        Ok(optimal_node_id)
    }

    /// Find optimal placement for a memory in the hierarchy
    async fn find_optimal_placement(&self, memory: &HierarchicalMemoryRecord) -> Result<String> {
        let nodes = self.nodes.read().await;

        // Find nodes matching scope and level
        let matching_nodes: Vec<_> = nodes
            .values()
            .filter(|node| {
                node.scope == memory.scope
                    && node.level == memory.level
                    && node.memory_ids.len() < self.config.max_memories_per_node
            })
            .collect();

        if let Some(best_node) = matching_nodes.first() {
            Ok(best_node.id.clone())
        } else {
            // Create new node if no suitable node exists
            self.create_hierarchy_node(memory.scope.clone(), memory.level.clone())
                .await
        }
    }

    /// Create a new hierarchy node
    async fn create_hierarchy_node(
        &self,
        scope: MemoryScope,
        level: MemoryLevel,
    ) -> Result<String> {
        let mut node = HierarchyNode::new(scope.clone(), level.clone());

        // Find appropriate parent
        if let Some(parent_id) = self.find_parent_node(&scope, &level).await? {
            node.parent_id = Some(parent_id.clone());

            // Update parent's children list
            let mut nodes = self.nodes.write().await;
            if let Some(parent) = nodes.get_mut(&parent_id) {
                parent.add_child(node.id.clone());
            }
            nodes.insert(node.id.clone(), node.clone());
        } else {
            // Root node
            let mut nodes = self.nodes.write().await;
            nodes.insert(node.id.clone(), node.clone());
        }

        // Update indices
        {
            let mut scope_index = self.scope_index.write().await;
            scope_index
                .entry(scope)
                .or_insert_with(Vec::new)
                .push(node.id.clone());
        }
        {
            let mut level_index = self.level_index.write().await;
            level_index
                .entry(level)
                .or_insert_with(Vec::new)
                .push(node.id.clone());
        }

        Ok(node.id)
    }

    /// Find appropriate parent node for a new node
    async fn find_parent_node(
        &self,
        scope: &MemoryScope,
        level: &MemoryLevel,
    ) -> Result<Option<String>> {
        let nodes = self.nodes.read().await;

        // Look for parent in higher level or broader scope
        let parent_candidates: Vec<_> = nodes
            .values()
            .filter(|node| {
                self.is_valid_parent(&node.scope, scope) && self.is_higher_level(&node.level, level)
            })
            .collect();

        // Select best parent based on compatibility and load
        if let Some(best_parent) = parent_candidates
            .into_iter()
            .min_by_key(|node| node.children_ids.len())
        {
            Ok(Some(best_parent.id.clone()))
        } else {
            Ok(None)
        }
    }

    /// Check if one scope can be parent of another
    fn is_valid_parent(&self, parent_scope: &MemoryScope, child_scope: &MemoryScope) -> bool {
        match (parent_scope, child_scope) {
            (MemoryScope::Global, _) => true,
            (MemoryScope::Agent(_), MemoryScope::User { .. }) => true,
            (MemoryScope::Agent(_), MemoryScope::Session { .. }) => true,
            (MemoryScope::User { .. }, MemoryScope::Session { .. }) => true,
            _ => false,
        }
    }

    /// Check if one level is higher than another
    fn is_higher_level(&self, parent_level: &MemoryLevel, child_level: &MemoryLevel) -> bool {
        use MemoryLevel::*;
        match (parent_level, child_level) {
            (Strategic, Tactical) | (Strategic, Operational) | (Strategic, Contextual) => true,
            (Tactical, Operational) | (Tactical, Contextual) => true,
            (Operational, Contextual) => true,
            _ => false,
        }
    }

    /// Initialize default hierarchy structure
    async fn initialize_default_hierarchy(&self) -> Result<()> {
        // Create root nodes for each level
        let levels = vec![
            MemoryLevel::Strategic,
            MemoryLevel::Tactical,
            MemoryLevel::Operational,
            MemoryLevel::Contextual,
        ];

        for level in levels {
            self.create_hierarchy_node(MemoryScope::Global, level)
                .await?;
        }

        Ok(())
    }

    /// Update indices after adding a memory
    async fn update_indices_after_addition(&self, memory: &HierarchicalMemoryRecord) -> Result<()> {
        // Update scope-based statistics
        {
            let mut stats = self.statistics.write().await;
            let scope_key = format!("{:?}", memory.scope);
            *stats.memories_per_scope.entry(scope_key).or_insert(0) += 1;
            stats.total_memories += 1;
        }

        Ok(())
    }

    /// Update hierarchy statistics
    async fn update_statistics(&self) -> Result<()> {
        let nodes = self.nodes.read().await;
        let mut stats = self.statistics.write().await;

        stats.total_nodes = nodes.len();
        stats.max_depth = nodes
            .values()
            .map(|node| node.calculate_depth(&nodes))
            .max()
            .unwrap_or(0);

        // Calculate average depth
        let total_depth: usize = nodes
            .values()
            .map(|node| node.calculate_depth(&nodes))
            .sum();
        stats.average_depth = if nodes.is_empty() {
            0.0
        } else {
            total_depth as f64 / nodes.len() as f64
        };

        // Update nodes per level
        stats.nodes_per_level.clear();
        for node in nodes.values() {
            *stats.nodes_per_level.entry(node.level.clone()).or_insert(0) += 1;
        }

        stats.last_updated = Utc::now();

        Ok(())
    }

    /// Check if optimization is needed and perform it
    async fn check_and_optimize(&self) -> Result<()> {
        let last_optimization = *self.last_optimization.read().await;
        let now = Utc::now();

        if now.signed_duration_since(last_optimization).num_hours()
            >= self.config.rebalance_interval_hours as i64
        {
            self.optimize_hierarchy().await?;
            *self.last_optimization.write().await = now;
        }

        Ok(())
    }

    /// Optimize the hierarchy structure
    async fn optimize_hierarchy(&self) -> Result<()> {
        // Identify overloaded nodes
        let overloaded_nodes = self.find_overloaded_nodes().await?;

        // Rebalance overloaded nodes
        for node_id in overloaded_nodes {
            self.rebalance_node(&node_id).await?;
        }

        // Compress underutilized branches
        if self.config.enable_compression {
            self.compress_hierarchy().await?;
        }

        Ok(())
    }

    /// Find nodes that exceed capacity limits
    async fn find_overloaded_nodes(&self) -> Result<Vec<String>> {
        let nodes = self.nodes.read().await;
        let overloaded: Vec<String> = nodes
            .values()
            .filter(|node| node.memory_ids.len() > self.config.max_memories_per_node)
            .map(|node| node.id.clone())
            .collect();

        Ok(overloaded)
    }

    /// Rebalance a specific node by splitting or redistributing
    async fn rebalance_node(&self, node_id: &str) -> Result<()> {
        // Implementation would split overloaded nodes or redistribute memories
        // This is a placeholder for the complex rebalancing logic
        Ok(())
    }

    /// Compress hierarchy by merging underutilized nodes
    async fn compress_hierarchy(&self) -> Result<()> {
        // Implementation would identify and merge nodes with low utilization
        // This is a placeholder for the compression logic
        Ok(())
    }

    /// Get hierarchy statistics
    pub async fn get_statistics(&self) -> HierarchyStatistics {
        self.statistics.read().await.clone()
    }

    /// Get all nodes in the hierarchy
    pub async fn get_all_nodes(&self) -> HashMap<String, HierarchyNode> {
        self.nodes.read().await.clone()
    }

    /// Search for nodes matching criteria
    pub async fn search_nodes(
        &self,
        scope: Option<MemoryScope>,
        level: Option<MemoryLevel>,
        min_importance: Option<f64>,
    ) -> Result<Vec<HierarchyNode>> {
        let nodes = self.nodes.read().await;
        let mut results = Vec::new();

        for node in nodes.values() {
            let mut matches = true;

            if let Some(ref target_scope) = scope {
                if &node.scope != target_scope {
                    matches = false;
                }
            }

            if let Some(ref target_level) = level {
                if &node.level != target_level {
                    matches = false;
                }
            }

            if let Some(min_imp) = min_importance {
                if node.importance_score < min_imp {
                    matches = false;
                }
            }

            if matches {
                results.push(node.clone());
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical_service::HierarchicalServiceConfig;

    #[tokio::test]
    async fn test_hierarchy_manager_creation() {
        let config = HierarchyManagerConfig::default();
        let service_config = HierarchicalServiceConfig::default();
        let service = Arc::new(
            HierarchicalMemoryService::new(service_config)
                .await
                .unwrap(),
        );

        let manager = HierarchyManager::new(config, service).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_hierarchy_node_creation() {
        let node = HierarchyNode::new(MemoryScope::Global, MemoryLevel::Strategic);
        assert_eq!(node.scope, MemoryScope::Global);
        assert_eq!(node.level, MemoryLevel::Strategic);
        assert!(node.children_ids.is_empty());
        assert!(node.memory_ids.is_empty());
    }

    #[tokio::test]
    async fn test_node_depth_calculation() {
        let mut hierarchy = HashMap::new();

        let mut root = HierarchyNode::new(MemoryScope::Global, MemoryLevel::Strategic);
        let mut child = HierarchyNode::new(MemoryScope::Global, MemoryLevel::Tactical);
        let mut grandchild = HierarchyNode::new(MemoryScope::Global, MemoryLevel::Operational);

        child.parent_id = Some(root.id.clone());
        grandchild.parent_id = Some(child.id.clone());

        hierarchy.insert(root.id.clone(), root.clone());
        hierarchy.insert(child.id.clone(), child.clone());
        hierarchy.insert(grandchild.id.clone(), grandchild.clone());

        assert_eq!(root.calculate_depth(&hierarchy), 0);
        assert_eq!(child.calculate_depth(&hierarchy), 1);
        assert_eq!(grandchild.calculate_depth(&hierarchy), 2);
    }
}
