//! Pipeline type definitions
//! 
//! Defines pipeline processing structures for memory operations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Pipeline context for stage execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineContext {
    /// Pipeline name
    pub name: String,
    /// Session ID
    pub session_id: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Agent ID
    pub agent_id: Option<String>,
    /// Custom metadata
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl PipelineContext {
    /// Create new pipeline context
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            session_id: None,
            user_id: None,
            agent_id: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Set agent ID
    pub fn with_agent(mut self, agent_id: &str) -> Self {
        self.agent_id = Some(agent_id.to_string());
        self
    }

    /// Set metadata
    pub fn with_metadata<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Stage result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageResult<T> {
    /// Success with value
    Success(T),
    /// Warning with value
    Warning(T, String),
    /// Error
    Error(String),
    /// Skip (condition not met)
    Skipped,
}

impl<T> StageResult<T> {
    /// Check if result is success
    pub fn is_success(&self) -> bool {
        matches!(self, StageResult::Success(_))
    }

    /// Check if result is error
    pub fn is_error(&self) -> bool {
        matches!(self, StageResult::Error(_))
    }

    /// Get value if success
    pub fn value(&self) -> Option<&T> {
        match self {
            StageResult::Success(v) | StageResult::Warning(v, _) => Some(v),
            _ => None,
        }
    }

    /// Map value if success
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> StageResult<U> {
        match self {
            StageResult::Success(v) => StageResult::Success(f(v)),
            StageResult::Warning(v, w) => StageResult::Warning(f(v), w),
            StageResult::Error(e) => StageResult::Error(e),
            StageResult::Skipped => StageResult::Skipped,
        }
    }
}

/// Pipeline stage trait
#[async_trait]
pub trait PipelineStage<I, O>: Send + Sync {
    /// Stage name
    fn name(&self) -> &str;
    
    /// Process input and return output
    async fn process(&self, ctx: &PipelineContext, input: I) -> StageResult<O>;
}

/// Simple pipeline processor (manually implement Debug since dyn traits don't)
pub struct Pipeline<I, O> {
    stages: Vec<Box<dyn PipelineStage<I, O>>>,
}

impl<I, O> Pipeline<I, O> {
    /// Create new pipeline
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage
    pub fn add_stage<S: PipelineStage<I, O> + 'static>(&mut self, stage: S) -> &mut Self {
        self.stages.push(Box::new(stage));
        self
    }

    /// Get stage count
    pub fn len(&self) -> usize {
        self.stages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }
}

impl<I, O> Default for Pipeline<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> std::fmt::Debug for Pipeline<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("stage_count", &self.stages.len())
            .finish()
    }
}

/// Type alias for condition function
pub type ConditionFn = Box<dyn Fn(&PipelineContext) -> bool + Send + Sync>;

/// DAG edge
#[derive(Debug, Clone)]
pub struct DagEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
}

/// DAG node for complex pipelines
pub struct DagNode<I, O> {
    /// Node ID
    pub id: String,
    /// Stage
    pub stage: Box<dyn PipelineStage<I, O>>,
    /// Condition function (optional)
    pub condition: Option<ConditionFn>,
}

impl<I, O> DagNode<I, O> {
    /// Create new DAG node
    pub fn new<S: PipelineStage<I, O> + 'static>(id: &str, stage: S) -> Self {
        Self {
            id: id.to_string(),
            stage: Box::new(stage),
            condition: None,
        }
    }

    /// Add condition
    pub fn with_condition<F: Fn(&PipelineContext) -> bool + Send + Sync + 'static>(mut self, cond: F) -> Self {
        self.condition = Some(Box::new(cond));
        self
    }

    /// Check if condition is met (or no condition)
    pub fn should_execute(&self, ctx: &PipelineContext) -> bool {
        self.condition.as_ref().map_or(true, |c| c(ctx))
    }
}

impl<I, O> std::fmt::Debug for DagNode<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DagNode")
            .field("id", &self.id)
            .field("has_condition", &self.condition.is_some())
            .finish()
    }
}

/// DAG pipeline for complex processing flows
pub struct DagPipeline<I, O> {
    /// Nodes by ID
    nodes: Vec<DagNode<I, O>>,
    /// Edges
    edges: Vec<DagEdge>,
}

impl<I, O> DagPipeline<I, O> {
    /// Create new DAG pipeline
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node
    pub fn add_node<N: Into<String>, S: PipelineStage<I, O> + 'static>(&mut self, id: N, stage: S) -> &mut Self {
        self.nodes.push(DagNode::new(id.into().as_str(), stage));
        self
    }

    /// Add an edge
    pub fn add_edge<F: Into<String>, T: Into<String>>(&mut self, from: F, to: T) -> &mut Self {
        self.edges.push(DagEdge {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Get node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl<I, O> Default for DagPipeline<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> std::fmt::Debug for DagPipeline<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DagPipeline")
            .field("node_count", &self.nodes.len())
            .field("edge_count", &self.edges.len())
            .finish()
    }
}
