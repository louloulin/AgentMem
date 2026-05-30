//! Data Lineage Tracking for AgentMem
//!
//! This module provides comprehensive data lineage tracking, enabling:
//! - Trace the origin and transformation of memories
//! - Impact analysis for changes
//! - GDPR compliance (right to be forgotten)
//! - Data provenance visualization
//!
//! # Features
//!
//! - Forward lineage (data derivations)
//! - Backward lineage (data origins)
//! - Impact analysis for memory modifications
//! - GDPR-compliant data deletion with cascade
//! - Lineage graph visualization data

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Data lineage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageConfig {
    /// Enable lineage tracking
    pub enable_tracking: bool,
    /// Maximum lineage depth to track
    pub max_depth: u32,
    /// Enable GDPR mode (for right to be forgotten)
    pub gdpr_mode: bool,
    /// Enable impact analysis
    pub enable_impact_analysis: bool,
}

impl Default for LineageConfig {
    fn default() -> Self {
        Self {
            enable_tracking: true,
            max_depth: 100,
            gdpr_mode: false,
            enable_impact_analysis: true,
        }
    }
}

/// Represents a data transformation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    /// Unique transformation ID
    pub transformation_id: String,
    /// Type of transformation
    pub transformation_type: TransformationType,
    /// Memory ID before transformation
    pub source_memory_id: Option<String>,
    /// Memory IDs after transformation
    pub target_memory_ids: Vec<String>,
    /// Agent that performed the transformation
    pub agent_id: String,
    /// Transformation timestamp
    pub timestamp: DateTime<Utc>,
    /// Transformation parameters/metadata
    pub parameters: HashMap<String, String>,
    /// Human-readable description
    pub description: Option<String>,
}

/// Transformation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransformationType {
    /// Memory was created
    Create,
    /// Memory was updated
    Update,
    /// Memory was deleted
    Delete,
    /// Memory was merged from multiple sources
    Merge,
    /// Memory was split into multiple targets
    Split,
    /// Memory was copied
    Copy,
    /// Memory was compressed
    Compress,
    /// Memory was summarized
    Summarize,
    /// Memory was translated/transformed
    Transform,
    /// Memory was promoted to higher hierarchy level
    Promote,
    /// Memory was demoted to lower hierarchy level
    Demote,
    /// Memory was shared with another agent
    Share,
    /// Memory was imported from external source
    Import,
    /// Memory was exported to external target
    Export,
}

/// A node in the lineage graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    /// Memory/resource ID
    pub id: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last modification time
    pub modified_at: DateTime<Utc>,
    /// Node metadata
    pub metadata: HashMap<String, String>,
    /// Is this a root node (no predecessors)
    pub is_root: bool,
    /// Is this a leaf node (no successors)
    pub is_leaf: bool,
}

/// Memory type for lineage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Working memory
    Working,
    /// Episodic memory
    Episodic,
    /// Semantic memory
    Semantic,
    /// Procedural memory
    Procedural,
    /// Core memory
    Core,
    /// Resource memory
    Resource,
}

impl Default for MemoryType {
    fn default() -> Self {
        MemoryType::Working
    }
}

/// An edge in the lineage graph (transformation relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Source node ID
    pub source_id: String,
    /// Target node ID
    pub target_id: String,
    /// Transformation that created this edge
    pub transformation_id: String,
    /// Edge weight (e.g., percentage of data from source)
    pub weight: f32,
}

/// Complete lineage graph for a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageGraph {
    /// All nodes in the graph
    pub nodes: Vec<LineageNode>,
    /// All edges in the graph
    pub edges: Vec<LineageEdge>,
    /// Root node IDs
    pub root_ids: Vec<String>,
    /// Leaf node IDs
    pub leaf_ids: Vec<String>,
    /// Graph statistics
    pub stats: LineageStats,
}

/// Lineage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageStats {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Maximum depth in the graph
    pub max_depth: u32,
    /// Is the graph acyclic
    pub is_acyclic: bool,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Memory that was analyzed
    pub source_id: String,
    /// Direct descendants that would be affected
    pub direct_impact: Vec<ImpactEntry>,
    /// All descendants (transitive closure)
    pub full_impact: Vec<ImpactEntry>,
    /// Statistics
    pub stats: ImpactStats,
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
}

/// Single impact entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEntry {
    /// Affected memory ID
    pub memory_id: String,
    /// Impact type
    pub impact_type: ImpactType,
    /// Depth from source (0 = direct)
    pub depth: u32,
    /// Impact percentage (0.0 - 1.0)
    pub impact_percentage: f32,
    /// Path from source to this node
    pub path: Vec<String>,
}

/// Impact types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactType {
    /// Direct modification impact
    DirectModification,
    /// Data dependency impact
    DataDependency,
    /// Derived data impact
    DerivedData,
    /// Reference impact
    Reference,
    /// No impact
    None,
}

/// Impact statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactStats {
    /// Total affected memories
    pub total_affected: usize,
    /// Direct impacts count
    pub direct_count: usize,
    /// Indirect impacts count
    pub indirect_count: usize,
    /// Maximum impact depth
    pub max_impact_depth: u32,
}

/// GDPR deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprDeletionRequest {
    /// User/subject ID to delete
    pub subject_id: String,
    /// Memory IDs to delete
    pub memory_ids: Vec<String>,
    /// Cascade deletion flag
    pub cascade: bool,
    /// Request timestamp
    pub requested_at: DateTime<Utc>,
    /// Deletion status
    pub status: DeletionStatus,
}

/// Deletion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionStatus {
    /// Deletion is pending
    Pending,
    /// Deletion is in progress
    InProgress,
    /// Deletion is complete
    Completed,
    /// Deletion failed
    Failed(String),
}

/// GDPR deletion report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprDeletionReport {
    /// Original request
    pub request: GdprDeletionRequest,
    /// All deleted memory IDs
    pub deleted_ids: Vec<String>,
    /// All affected memory IDs
    pub affected_ids: Vec<String>,
    /// Cascaded deletions
    pub cascaded_deletions: Vec<CascadeDeletion>,
    /// Report timestamp
    pub completed_at: DateTime<Utc>,
}

/// Cascade deletion entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeDeletion {
    /// Memory ID that triggered cascade
    pub trigger_id: String,
    /// Memory ID that was cascade-deleted
    pub deleted_id: String,
    /// Reason for cascade
    pub reason: String,
}

/// Lineage query options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageQuery {
    /// Starting memory ID
    pub memory_id: String,
    /// Direction of traversal
    pub direction: LineageDirection,
    /// Maximum depth to traverse
    pub max_depth: Option<u32>,
    /// Include metadata
    pub include_metadata: bool,
    /// Include transformation details
    pub include_transformations: bool,
}

/// Lineage traversal direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineageDirection {
    /// Forward (successors/children)
    Forward,
    /// Backward (predecessors/parents)
    Backward,
    /// Both directions
    Both,
}

/// Lineage tracking engine
pub struct LineageTracker {
    /// Configuration
    config: LineageConfig,
    /// Store: memory_id -> node
    nodes: Arc<RwLock<HashMap<String, LineageNode>>>,
    /// Store: memory_id -> transformation
    transformations: Arc<RwLock<HashMap<String, Transformation>>>,
    /// Store: memory_id -> Vec<transformation_id>
    memory_to_transformations: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Store: transformation_id -> transformation
    transformation_index: Arc<RwLock<HashMap<String, String>>>, // transformation_id -> serialized
    /// GDPR requests
    gdpr_requests: Arc<RwLock<Vec<GdprDeletionRequest>>>,
}

impl LineageTracker {
    /// Create a new lineage tracker
    pub fn new(config: LineageConfig) -> Self {
        Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            transformations: Arc::new(RwLock::new(HashMap::new())),
            memory_to_transformations: Arc::new(RwLock::new(HashMap::new())),
            transformation_index: Arc::new(RwLock::new(HashMap::new())),
            gdpr_requests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default configuration
    pub fn default_tracker() -> Self {
        Self::new(LineageConfig::default())
    }

    /// Record a memory creation
    pub async fn record_creation(
        &self,
        memory_id: String,
        memory_type: MemoryType,
        agent_id: String,
        metadata: HashMap<String, String>,
    ) -> String {
        let transformation_id = format!("create_{}_{}", memory_id, Utc::now().timestamp_nanos_opt().unwrap_or(0));

        // Create node
        let now = Utc::now();
        let node = LineageNode {
            id: memory_id.clone(),
            memory_type: memory_type.clone(),
            created_at: now,
            modified_at: now,
            metadata: metadata.clone(),
            is_root: true,
            is_leaf: true,
        };

        // Create transformation
        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Create,
            source_memory_id: None,
            target_memory_ids: vec![memory_id.clone()],
            agent_id,
            timestamp: now,
            parameters: metadata,
            description: Some(format!("Memory created: {}", memory_id)),
        };

        // Store
        self.store_node(node).await;
        self.store_transformation(memory_id, transformation).await;

        transformation_id
    }

    /// Record a memory update
    pub async fn record_update(
        &self,
        memory_id: String,
        agent_id: String,
        description: Option<String>,
    ) -> String {
        let transformation_id = format!("update_{}_{}", memory_id, Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let now = Utc::now();

        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Update,
            source_memory_id: Some(memory_id.clone()),
            target_memory_ids: vec![memory_id.clone()],
            agent_id,
            timestamp: now,
            parameters: HashMap::new(),
            description,
        };

        // Update node modification time
        self.update_node_modified(&memory_id, now).await;
        self.store_transformation(memory_id, transformation).await;

        transformation_id
    }

    /// Record a memory deletion
    pub async fn record_deletion(
        &self,
        memory_id: String,
        agent_id: String,
    ) -> String {
        let transformation_id = format!("delete_{}_{}", memory_id, Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let now = Utc::now();

        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Delete,
            source_memory_id: Some(memory_id.clone()),
            target_memory_ids: vec![],
            agent_id,
            timestamp: now,
            parameters: HashMap::new(),
            description: Some(format!("Memory deleted: {}", memory_id)),
        };

        // Mark node as leaf (deleted)
        self.mark_node_deleted(&memory_id).await;
        self.store_transformation(memory_id, transformation).await;

        transformation_id
    }

    /// Record a memory merge (multiple sources -> one target)
    pub async fn record_merge(
        &self,
        source_ids: Vec<String>,
        target_id: String,
        target_type: MemoryType,
        agent_id: String,
        description: Option<String>,
    ) -> String {
        let transformation_id = format!("merge_{}_{}", target_id, Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let now = Utc::now();

        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Merge,
            source_memory_id: None, // Multiple sources
            target_memory_ids: vec![target_id.clone()],
            agent_id,
            timestamp: now,
            parameters: HashMap::from([("source_count".to_string(), source_ids.len().to_string())]),
            description,
        };

        // Create target node
        let node = LineageNode {
            id: target_id.clone(),
            memory_type: target_type,
            created_at: now,
            modified_at: now,
            metadata: HashMap::new(),
            is_root: source_ids.is_empty(),
            is_leaf: true,
        };

        // Mark sources as no longer leaf
        for source_id in &source_ids {
            self.mark_node_non_leaf(source_id).await;
        }

        self.store_node(node).await;

        // Store transformation for each source
        for source_id in source_ids {
            self.store_transformation(source_id, transformation.clone()).await;
        }

        transformation_id
    }

    /// Record a memory split (one source -> multiple targets)
    pub async fn record_split(
        &self,
        source_id: String,
        target_ids: Vec<String>,
        target_type: MemoryType,
        agent_id: String,
        description: Option<String>,
    ) -> String {
        let transformation_id = format!("split_{}_{}", source_id, Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let now = Utc::now();

        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Split,
            source_memory_id: Some(source_id.clone()),
            target_memory_ids: target_ids.clone(),
            agent_id,
            timestamp: now,
            parameters: HashMap::from([("target_count".to_string(), target_ids.len().to_string())]),
            description,
        };

        // Mark source as no longer leaf
        self.mark_node_non_leaf(&source_id).await;

        // Create target nodes
        for target_id in &target_ids {
            let node = LineageNode {
                id: target_id.clone(),
                memory_type: target_type.clone(),
                created_at: now,
                modified_at: now,
                metadata: HashMap::new(),
                is_root: false,
                is_leaf: true,
            };
            self.store_node(node).await;
            self.store_transformation(target_id.clone(), transformation.clone()).await;
        }

        transformation_id
    }

    /// Record memory promotion (hierarchy level change)
    pub async fn record_promotion(
        &self,
        memory_id: String,
        from_type: MemoryType,
        to_type: MemoryType,
        agent_id: String,
    ) -> String {
        let transformation_id = format!("promote_{}_{}", memory_id, Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let now = Utc::now();

        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Promote,
            source_memory_id: Some(memory_id.clone()),
            target_memory_ids: vec![memory_id.clone()],
            agent_id,
            timestamp: now,
            parameters: HashMap::from([
                ("from_type".to_string(), format!("{:?}", from_type)),
                ("to_type".to_string(), format!("{:?}", to_type)),
            ]),
            description: Some(format!("Memory promoted from {:?} to {:?}", from_type, to_type)),
        };

        // Update node type
        self.update_node_type(&memory_id, to_type).await;

        self.store_transformation(memory_id, transformation).await;

        transformation_id
    }

    /// Record memory sharing
    pub async fn record_share(
        &self,
        memory_id: String,
        from_agent: String,
        to_agent: String,
    ) -> String {
        let transformation_id = format!("share_{}_{}_{}", memory_id, from_agent, Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let now = Utc::now();
        let agent_id = from_agent.clone();
        let from_agent_clone = from_agent.clone();
        let to_agent_clone = to_agent.clone();

        let transformation = Transformation {
            transformation_id: transformation_id.clone(),
            transformation_type: TransformationType::Share,
            source_memory_id: Some(memory_id.clone()),
            target_memory_ids: vec![],
            agent_id,
            timestamp: now,
            parameters: HashMap::from([
                ("from_agent".to_string(), from_agent_clone),
                ("to_agent".to_string(), to_agent_clone),
            ]),
            description: Some(format!("Memory shared with {}", to_agent)),
        };

        self.store_transformation(memory_id, transformation).await;

        transformation_id
    }

    /// Get lineage graph for a memory
    pub async fn get_lineage(&self, query: LineageQuery) -> LineageGraph {
        let max_depth = query.max_depth.unwrap_or(self.config.max_depth);
        let mut visited = HashSet::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut root_ids = Vec::new();
        let mut leaf_ids = Vec::new();

        // BFS traversal
        let mut queue: VecDeque<(String, u32, Option<String>)> = VecDeque::new();
        queue.push_back((query.memory_id.clone(), 0, None));

        while let Some((current_id, depth, parent_id)) = queue.pop_front() {
            if visited.contains(&current_id) || depth > max_depth {
                continue;
            }
            visited.insert(current_id.clone());

            // Get node
            if let Some(node) = self.get_node(&current_id).await {
                nodes.push(node.clone());

                if node.is_root {
                    root_ids.push(current_id.clone());
                }
                if node.is_leaf {
                    leaf_ids.push(current_id.clone());
                }
            }

            // Add edge from parent
            if let Some(pid) = parent_id {
                edges.push(LineageEdge {
                    source_id: pid,
                    target_id: current_id.clone(),
                    transformation_id: String::new(),
                    weight: 1.0,
                });
            }

            // Get successors/predecessors based on direction
            let related_ids = match query.direction {
                LineageDirection::Forward => self.get_successors(&current_id).await,
                LineageDirection::Backward => self.get_predecessors(&current_id).await,
                LineageDirection::Both => {
                    let mut both = self.get_successors(&current_id).await;
                    both.extend(self.get_predecessors(&current_id).await);
                    both
                }
            };

            for related_id in related_ids {
                if !visited.contains(&related_id) {
                    queue.push_back((related_id, depth + 1, Some(current_id.clone())));
                }
            }
        }

        let max_actual_depth = nodes.iter()
            .map(|n| {
                let node = n;
                0
            })
            .max()
            .unwrap_or(0) as u32;

        let stats = LineageStats {
            node_count: nodes.len(),
            edge_count: edges.len(),
            max_depth: max_actual_depth,
            is_acyclic: self.check_acyclic(&nodes, &edges),
        };

        LineageGraph {
            nodes,
            edges,
            root_ids,
            leaf_ids,
            stats,
        }
    }

    /// Analyze impact of modifying a memory
    pub async fn analyze_impact(&self, memory_id: &str) -> ImpactAnalysis {
        let mut direct_impact = Vec::new();
        let mut full_impact = Vec::new();
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(String, u32, Vec<String>)> = VecDeque::new();

        queue.push_back((memory_id.to_string(), 0, vec![memory_id.to_string()]));

        while let Some((current_id, depth, path)) = queue.pop_front() {
            if visited.contains(&current_id) || depth > self.config.max_depth {
                continue;
            }
            visited.insert(current_id.clone());

            // Get direct successors
            let successors = self.get_successors(&current_id).await;

            for succ_id in successors {
                let succ_path = {
                    let mut p = path.clone();
                    p.push(succ_id.clone());
                    p
                };

                let impact_entry = ImpactEntry {
                    memory_id: succ_id.clone(),
                    impact_type: if depth == 0 {
                        ImpactType::DirectModification
                    } else {
                        ImpactType::DerivedData
                    },
                    depth: depth + 1,
                    impact_percentage: 1.0 / (depth + 1) as f32,
                    path: succ_path.clone(),
                };

                if depth == 0 {
                    direct_impact.push(impact_entry.clone());
                }
                full_impact.push(impact_entry.clone());

                if !visited.contains(&succ_id) {
                    queue.push_back((succ_id, depth + 1, succ_path));
                }
            }
        }

        let direct_count = direct_impact.len();
        let total_affected = full_impact.len();
        let max_depth = full_impact.iter().map(|e| e.depth).max().unwrap_or(0);

        ImpactAnalysis {
            source_id: memory_id.to_string(),
            direct_impact,
            full_impact,
            stats: ImpactStats {
                total_affected,
                direct_count,
                indirect_count: total_affected - direct_count,
                max_impact_depth: max_depth,
            },
            timestamp: Utc::now(),
        }
    }

    /// Process GDPR deletion request
    pub async fn process_gdpr_deletion(&self, request: GdprDeletionRequest) -> GdprDeletionReport {
        let mut deleted_ids = Vec::new();
        let mut affected_ids = Vec::new();
        let mut cascaded_deletions = Vec::new();
        let mut visited = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        // Start with requested memory IDs
        for memory_id in &request.memory_ids {
            queue.push_back(memory_id.clone());
        }

        // BFS to find all derived memories
        while let Some(current_id) = queue.pop_front() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            // Check if this memory has any predecessor that should be preserved
            let predecessors = self.get_predecessors(&current_id).await;
            let has_external_predecessor = predecessors.iter().any(|p| {
                !request.memory_ids.contains(p) && !visited.contains(p)
            });

            if request.cascade || !has_external_predecessor {
                deleted_ids.push(current_id.clone());

                // Find successors
                let successors = self.get_successors(&current_id).await;
                for succ_id in successors {
                    if !visited.contains(&succ_id) {
                        affected_ids.push(succ_id.clone());
                        cascaded_deletions.push(CascadeDeletion {
                            trigger_id: current_id.clone(),
                            deleted_id: succ_id.clone(),
                            reason: "Cascaded from GDPR deletion".to_string(),
                        });
                        queue.push_back(succ_id);
                    }
                }
            } else {
                affected_ids.push(current_id.clone());
            }
        }

        // Mark all deleted IDs
        for id in &deleted_ids {
            self.mark_node_deleted(id).await;
        }

        GdprDeletionReport {
            request: request.clone(),
            deleted_ids,
            affected_ids,
            cascaded_deletions,
            completed_at: Utc::now(),
        }
    }

    /// Store a lineage node
    async fn store_node(&self, node: LineageNode) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id.clone(), node);
    }

    /// Get a node by ID
    async fn get_node(&self, memory_id: &str) -> Option<LineageNode> {
        let nodes = self.nodes.read().await;
        nodes.get(memory_id).cloned()
    }

    /// Store a transformation
    async fn store_transformation(&self, memory_id: String, transformation: Transformation) {
        let trans_id = transformation.transformation_id.clone();
        let mut transforms = self.transformations.write().await;
        transforms.insert(trans_id.clone(), transformation);

        let mut mem_to_trans = self.memory_to_transformations.write().await;
        mem_to_trans
            .entry(memory_id)
            .or_insert_with(Vec::new)
            .push(trans_id.clone());

        let mut index = self.transformation_index.write().await;
        index.insert(trans_id.clone(), trans_id);
    }

    /// Update node modification time
    async fn update_node_modified(&self, memory_id: &str, timestamp: DateTime<Utc>) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(memory_id) {
            node.modified_at = timestamp;
        }
    }

    /// Update node type
    async fn update_node_type(&self, memory_id: &str, new_type: MemoryType) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(memory_id) {
            node.memory_type = new_type;
            node.modified_at = Utc::now();
        }
    }

    /// Mark node as deleted (leaf)
    async fn mark_node_deleted(&self, memory_id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(memory_id) {
            node.is_leaf = true;
            node.metadata.insert("deleted".to_string(), Utc::now().to_rfc3339());
        }
    }

    /// Mark node as non-leaf
    async fn mark_node_non_leaf(&self, memory_id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(memory_id) {
            node.is_leaf = false;
        }
    }

    /// Get successor memory IDs
    async fn get_successors(&self, memory_id: &str) -> Vec<String> {
        let transforms = self.transformations.read().await;
        let mem_to_trans = self.memory_to_transformations.read().await;

        let mut successors = Vec::new();
        if let Some(trans_ids) = mem_to_trans.get(memory_id) {
            for trans_id in trans_ids {
                if let Some(trans) = transforms.get(trans_id) {
                    for target in &trans.target_memory_ids {
                        if target != memory_id {
                            successors.push(target.clone());
                        }
                    }
                }
            }
        }
        successors
    }

    /// Get predecessor memory IDs
    async fn get_predecessors(&self, memory_id: &str) -> Vec<String> {
        let transforms = self.transformations.read().await;
        let mut predecessors = Vec::new();

        for trans in transforms.values() {
            if trans.target_memory_ids.contains(&memory_id.to_string()) {
                if let Some(ref source_id) = trans.source_memory_id {
                    if source_id != memory_id {
                        predecessors.push(source_id.clone());
                    }
                }
            }
        }
        predecessors
    }

    /// Check if the graph is acyclic
    fn check_acyclic(&self, nodes: &[LineageNode], edges: &[LineageEdge]) -> bool {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for node in nodes {
            in_degree.insert(node.id.clone(), 0);
            adjacency.insert(node.id.clone(), Vec::new());
        }

        for edge in edges {
            if let Some(adj) = adjacency.get_mut(&edge.source_id) {
                adj.push(edge.target_id.clone());
            }
            if let Some(deg) = in_degree.get_mut(&edge.target_id) {
                *deg += 1;
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut visited_count = 0;

        while let Some(node_id) = queue.pop_front() {
            visited_count += 1;
            if let Some(adj) = adjacency.get(&node_id) {
                for neighbor in adj {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        visited_count == nodes.len()
    }

    /// Get all memory IDs for a subject (for GDPR)
    pub async fn get_subject_memories(&self, subject_id: &str) -> Vec<String> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| {
                n.metadata.get("subject_id") == Some(&subject_id.to_string())
                    || n.metadata.get("owner_id") == Some(&subject_id.to_string())
                    || n.metadata.get("agent_id") == Some(&subject_id.to_string())
            })
            .map(|n| n.id.clone())
            .collect()
    }

    /// Get lineage statistics
    pub async fn get_stats(&self) -> LineageTrackerStats {
        let nodes = self.nodes.read().await;
        let transforms = self.transformations.read().await;

        LineageTrackerStats {
            total_nodes: nodes.len(),
            total_transformations: transforms.len(),
            nodes_by_type: {
                let mut counts: HashMap<String, usize> = HashMap::new();
                for node in nodes.values() {
                    *counts.entry(format!("{:?}", node.memory_type)).or_insert(0) += 1;
                }
                counts
            },
        }
    }
}

/// Lineage tracker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageTrackerStats {
    /// Total number of tracked nodes
    pub total_nodes: usize,
    /// Total number of transformations
    pub total_transformations: usize,
    /// Node counts by type
    pub nodes_by_type: HashMap<String, usize>,
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_creation() {
        let tracker = LineageTracker::default_tracker();

        let trans_id = tracker
            .record_creation(
                "memory1".to_string(),
                MemoryType::Working,
                "agent1".to_string(),
                HashMap::new(),
            )
            .await;

        assert!(trans_id.starts_with("create_memory1"));
    }

    #[tokio::test]
    async fn test_record_merge() {
        let tracker = LineageTracker::default_tracker();

        // Create source memories
        tracker
            .record_creation(
                "source1".to_string(),
                MemoryType::Episodic,
                "agent1".to_string(),
                HashMap::new(),
            )
            .await;
        tracker
            .record_creation(
                "source2".to_string(),
                MemoryType::Episodic,
                "agent1".to_string(),
                HashMap::new(),
            )
            .await;

        // Merge into target
        tracker
            .record_merge(
                vec!["source1".to_string(), "source2".to_string()],
                "merged".to_string(),
                MemoryType::Semantic,
                "agent1".to_string(),
                Some("Merged episodic memories".to_string()),
            )
            .await;

        // Get lineage
        let graph = tracker
            .get_lineage(LineageQuery {
                memory_id: "merged".to_string(),
                direction: LineageDirection::Backward,
                max_depth: Some(10),
                include_metadata: false,
                include_transformations: false,
            })
            .await;

        assert!(graph.nodes.len() >= 3); // merged + sources
    }

    #[tokio::test]
    async fn test_impact_analysis() {
        let tracker = LineageTracker::default_tracker();

        // Create chain: source1 -> merged -> derived
        tracker
            .record_creation(
                "source1".to_string(),
                MemoryType::Working,
                "agent1".to_string(),
                HashMap::new(),
            )
            .await;

        tracker
            .record_merge(
                vec!["source1".to_string()],
                "merged".to_string(),
                MemoryType::Semantic,
                "agent1".to_string(),
                None,
            )
            .await;

        tracker
            .record_split(
                "merged".to_string(),
                vec!["derived1".to_string()],
                MemoryType::Procedural,
                "agent1".to_string(),
                None,
            )
            .await;

        // Analyze impact of source1 deletion
        let impact = tracker.analyze_impact("source1").await;

        assert!(impact.stats.total_affected >= 2); // merged and derived
        assert!(impact.stats.direct_count >= 1); // merged
    }

    #[tokio::test]
    async fn test_gdpr_deletion() {
        let tracker = LineageTracker::default_tracker();

        // Create memory owned by subject
        tracker
            .record_creation(
                "user_memory".to_string(),
                MemoryType::Working,
                "agent1".to_string(),
                HashMap::from([("owner_id".to_string(), "user123".to_string())]),
            )
            .await;

        // Create derived memory
        tracker
            .record_merge(
                vec!["user_memory".to_string()],
                "derived_memory".to_string(),
                MemoryType::Semantic,
                "agent1".to_string(),
                None,
            )
            .await;

        // Process GDPR deletion
        let request = GdprDeletionRequest {
            subject_id: "user123".to_string(),
            memory_ids: vec!["user_memory".to_string()],
            cascade: true,
            requested_at: Utc::now(),
            status: DeletionStatus::Pending,
        };

        let report = tracker.process_gdpr_deletion(request).await;

        assert!(report.deleted_ids.contains(&"user_memory".to_string()));
    }
}
