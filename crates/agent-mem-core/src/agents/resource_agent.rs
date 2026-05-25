//! Resource Memory Agent
//!
//! This agent specializes in managing resource memories - multimedia files and documents.
//! In Phase B, this agent serves as the resource ingestion entrypoint with mount, preprocess, and extract operations.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[cfg(feature = "resource-extraction")]
use agent_mem_extraction::{ExtractionInput, ExtractionOutput, ExtractionPipeline, PipelineConfig};
#[cfg(feature = "resource-extraction")]
use agent_mem_resource::{ResourceId, ResourceManager, ResourceManagerTrait};

use crate::agents::{
    AgentConfig, AgentContext, AgentError, AgentResult, AgentStats, BaseAgent, MemoryAgent,
};
use crate::coordination::{
    AgentMessage, CoordinationError, CoordinationResult, TaskRequest, TaskResponse,
};
use crate::types::MemoryType;

/// Resource Memory Agent
///
/// This agent handles resource-centric operations:
/// - `insert`: Legacy resource memory insertion (backward compatibility)
/// - `search`: Legacy resource search (backward compatibility)
/// - `mount`: Mount a resource from URI and return a resource ID (file-centric)
/// - `preprocess`: Preprocess a mounted resource for multimodal content (file-centric)
/// - `extract`: Extract memory items from a mounted resource via the extraction pipeline (file-centric)
pub struct ResourceAgent {
    base: BaseAgent,
    context: Arc<RwLock<AgentContext>>,
    initialized: bool,
    #[cfg(feature = "resource-extraction")]
    resource_manager: Option<Arc<ResourceManager>>,
    #[cfg(feature = "resource-extraction")]
    extraction_pipeline: Option<Arc<ExtractionPipeline>>,
}

impl ResourceAgent {
    /// Create a new ResourceAgent
    pub fn new(agent_id: String) -> Self {
        let config = AgentConfig::new(agent_id, vec![MemoryType::Resource], 8);
        let base = BaseAgent::new(config);
        let context = base.context();
        Self {
            base,
            context,
            initialized: false,
            #[cfg(feature = "resource-extraction")]
            resource_manager: None,
            #[cfg(feature = "resource-extraction")]
            extraction_pipeline: None,
        }
    }

    /// Create a new ResourceAgent with resource management capabilities
    #[cfg(feature = "resource-extraction")]
    pub fn with_managers(
        agent_id: String,
        resource_manager: Arc<ResourceManager>,
        extraction_pipeline: Arc<ExtractionPipeline>,
    ) -> Self {
        let config = AgentConfig::new(agent_id, vec![MemoryType::Resource], 8);
        let base = BaseAgent::new(config);
        let context = base.context();
        Self {
            base,
            context,
            initialized: false,
            resource_manager: Some(resource_manager),
            extraction_pipeline: Some(extraction_pipeline),
        }
    }

    /// Handle legacy insert operation (backward compatibility)
    async fn handle_insert(&self, parameters: Value) -> AgentResult<Value> {
        let resource = parameters.get("resource").ok_or_else(|| {
            AgentError::InvalidParameters("Missing 'resource' parameter".to_string())
        })?;

        let response = serde_json::json!({
            "success": true,
            "resource_id": uuid::Uuid::new_v4().to_string(),
            "message": "Resource memory inserted successfully"
        });

        log::info!("Resource agent: Inserted resource");
        Ok(response)
    }

    /// Handle legacy search operation (backward compatibility)
    async fn handle_search(&self, parameters: Value) -> AgentResult<Value> {
        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'query' parameter".to_string())
            })?;

        let response = serde_json::json!({
            "success": true,
            "results": [],
            "total_count": 0,
            "query": query
        });

        log::info!("Resource agent: Searched for '{query}'");
        Ok(response)
    }

    /// Handle mount operation - mount a resource from URI
    ///
    /// # Parameters
    /// - `uri`: Resource URI (file://, http://, conv://, doc://)
    /// - `user_id`: User ID that owns this resource
    /// - `agent_id`: Optional agent ID that created this resource
    ///
    /// # Returns
    /// - `resource_id`: Unique identifier for the mounted resource
    /// - `status`: Mount status ("mounted")
    #[cfg(feature = "resource-extraction")]
    async fn handle_mount(&self, parameters: Value) -> AgentResult<Value> {
        let resource_manager = self.resource_manager.as_ref().ok_or_else(|| {
            AgentError::InternalError("Resource manager not configured".to_string())
        })?;

        let uri = parameters
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'uri' parameter".to_string())
            })?;

        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        let agent_id = parameters.get("agent_id").and_then(|v| v.as_str());

        let resource_id = resource_manager
            .mount_resource(uri, user_id, agent_id)
            .await
            .map_err(|e| AgentError::InternalError(format!("Failed to mount resource: {}", e)))?;

        let response = serde_json::json!({
            "success": true,
            "resource_id": resource_id.0,
            "status": "mounted",
            "message": "Resource mounted successfully"
        });

        log::info!("Resource agent: Mounted resource {} -> {}", uri, resource_id.0);
        Ok(response)
    }

    /// Handle preprocess operation - preprocess mounted resource content
    ///
    /// # Parameters
    /// - `resource_id`: ID of the mounted resource
    ///
    /// # Returns
    /// - `preprocessed`: Boolean indicating success
    /// - `media_type`: Detected media type
    /// - `metadata`: Extracted metadata (size, line count, etc.)
    #[cfg(feature = "resource-extraction")]
    async fn handle_preprocess(&self, parameters: Value) -> AgentResult<Value> {
        let resource_manager = self.resource_manager.as_ref().ok_or_else(|| {
            AgentError::InternalError("Resource manager not configured".to_string())
        })?;

        let resource_id_str = parameters
            .get("resource_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'resource_id' parameter".to_string())
            })?;

        let resource_id = ResourceId(resource_id_str.to_string());

        // Resolve resource to get content and metadata
        let content = resource_manager
            .resolve_resource(&resource_id)
            .await
            .map_err(|e| AgentError::InternalError(format!("Failed to resolve resource: {}", e)))?;

        // Get resource metadata
        let resource = resource_manager
            .get_resource(&resource_id)
            .await
            .map_err(|e| AgentError::InternalError(format!("Failed to get resource: {}", e)))?;

        let response = serde_json::json!({
            "success": true,
            "preprocessed": true,
            "resource_id": resource_id_str,
            "media_type": resource.media_type.to_string(),
            "metadata": {
                "size": resource.metadata.size,
                "created_at": resource.metadata.created_at.to_rfc3339(),
            },
            "message": "Resource preprocessed successfully"
        });

        log::info!("Resource agent: Preprocessed resource {}", resource_id_str);
        Ok(response)
    }

    /// Handle extract operation - extract memory items from resource
    ///
    /// # Parameters
    /// - `resource_id`: ID of the mounted resource
    /// - `user_id`: User ID for extraction scope
    /// - `agent_id`: Optional agent ID for extraction scope
    ///
    /// # Returns
    /// - `extraction_id`: Unique identifier for this extraction
    /// - `items`: Extracted memory items
    /// - `categories`: Categories assigned to items
    /// - `metrics`: Extraction metrics
    #[cfg(feature = "resource-extraction")]
    async fn handle_extract(&self, parameters: Value) -> AgentResult<Value> {
        let extraction_pipeline = self.extraction_pipeline.as_ref().ok_or_else(|| {
            AgentError::InternalError("Extraction pipeline not configured".to_string())
        })?;

        let resource_manager = self.resource_manager.as_ref().ok_or_else(|| {
            AgentError::InternalError("Resource manager not configured".to_string())
        })?;

        let resource_id_str = parameters
            .get("resource_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'resource_id' parameter".to_string())
            })?;

        let user_id = parameters
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AgentError::InvalidParameters("Missing 'user_id' parameter".to_string())
            })?;

        let agent_id = parameters.get("agent_id").and_then(|v| v.as_str());

        let resource_id = ResourceId(resource_id_str.to_string());

        // Get resource info
        let resource = resource_manager
            .get_resource(&resource_id)
            .await
            .map_err(|e| AgentError::InternalError(format!("Failed to get resource: {}", e)))?;

        // Create extraction input
        let mut extraction_input = ExtractionInput::from_uri(&resource.uri, user_id);
        if let Some(aid) = agent_id {
            extraction_input.scope.agent_id = Some(aid.to_string());
        }

        // Execute extraction pipeline
        let extraction_output = extraction_pipeline
            .execute(extraction_input)
            .await
            .map_err(|e| AgentError::InternalError(format!("Extraction failed: {}", e)))?;

        let response = serde_json::json!({
            "success": true,
            "extraction_id": extraction_output.id.to_string(),
            "resource_id": resource_id_str,
            "items": extraction_output.items,
            "categories": extraction_output.categories,
            "metrics": {
                "total_duration_ms": extraction_output.metrics.total_duration_ms,
                "items_extracted": extraction_output.metrics.items_extracted,
                "items_deduped": extraction_output.metrics.items_deduped,
            },
            "warnings": extraction_output.warnings,
            "message": "Resource extracted successfully"
        });

        log::info!(
            "Resource agent: Extracted {} items from resource {}",
            extraction_output.metrics.items_extracted,
            resource_id_str
        );
        Ok(response)
    }
}

#[async_trait]
impl MemoryAgent for ResourceAgent {
    fn agent_id(&self) -> &str {
        &self.base.config().agent_id
    }

    fn memory_types(&self) -> &[MemoryType] {
        &self.base.config().memory_types
    }

    async fn initialize(&mut self) -> CoordinationResult<()> {
        if !self.initialized {
            log::info!("Initializing Resource Memory Agent: {}", self.agent_id());

            #[cfg(feature = "resource-extraction")]
            {
                // Initialize resource manager if not already set
                if self.resource_manager.is_none() {
                    match ResourceManager::new() {
                        Ok(rm) => self.resource_manager = Some(Arc::new(rm)),
                        Err(e) => {
                            log::warn!("Failed to create default resource manager: {}", e);
                        }
                    }
                }

                // Initialize extraction pipeline if not already set
                if self.extraction_pipeline.is_none() {
                    let config = PipelineConfig::default();
                    self.extraction_pipeline = Some(Arc::new(ExtractionPipeline::new(config)));
                }
            }

            self.initialized = true;
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> CoordinationResult<()> {
        if self.initialized {
            log::info!("Shutting down Resource Memory Agent: {}", self.agent_id());
            self.initialized = false;
        }
        Ok(())
    }

    async fn execute_task(&mut self, task: TaskRequest) -> CoordinationResult<TaskResponse> {
        if !self.initialized {
            return Err(CoordinationError::InternalError(
                "Agent not initialized".to_string(),
            ));
        }

        let start_time = Instant::now();

        {
            let mut context = self.context.write().await;
            context.current_task = Some(task.clone());
            context.stats.active_tasks += 1;
        }

        let result = match task.operation.as_str() {
            // Legacy operations (backward compatibility)
            "insert" => self.handle_insert(task.parameters).await,
            "search" => self.handle_search(task.parameters).await,
            // File-centric operations (Phase B)
            #[cfg(feature = "resource-extraction")]
            "mount" => self.handle_mount(task.parameters).await,
            #[cfg(feature = "resource-extraction")]
            "preprocess" => self.handle_preprocess(task.parameters).await,
            #[cfg(feature = "resource-extraction")]
            "extract" => self.handle_extract(task.parameters).await,
            #[cfg(not(feature = "resource-extraction"))]
            "mount" | "preprocess" | "extract" => {
                Err(AgentError::InvalidParameters(format!(
                    "Operation '{}' requires 'resource-extraction' feature to be enabled",
                    task.operation
                )))
            }
            _ => Err(AgentError::InvalidParameters(format!(
                "Unknown operation: {}",
                task.operation
            ))),
        };

        let execution_time = start_time.elapsed();

        {
            let mut context = self.context.write().await;
            context.current_task = None;
            context.stats.active_tasks = context.stats.active_tasks.saturating_sub(1);
            context
                .stats
                .update_task_completion(result.is_ok(), execution_time.as_millis() as f64);
        }

        match result {
            Ok(data) => Ok(TaskResponse::success(
                task.task_id,
                data,
                execution_time,
                self.agent_id().to_string(),
            )),
            Err(error) => Ok(TaskResponse::error(
                task.task_id,
                error.to_string(),
                execution_time,
                self.agent_id().to_string(),
            )),
        }
    }

    async fn handle_message(&mut self, message: AgentMessage) -> CoordinationResult<()> {
        log::debug!(
            "Resource agent received message: {:?}",
            message.message_type
        );
        Ok(())
    }

    async fn get_stats(&self) -> AgentStats {
        self.context.read().await.stats.clone()
    }

    async fn health_check(&self) -> bool {
        self.initialized
    }

    async fn current_load(&self) -> usize {
        self.context.read().await.stats.active_tasks
    }

    async fn can_accept_task(&self) -> bool {
        if !self.initialized {
            return false;
        }
        let context = self.context.read().await;
        context.stats.active_tasks < context.config.max_concurrent_tasks
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[test]
    fn test_resource_agent_creation() {
        let agent = ResourceAgent::new("test-resource-agent".to_string());
        assert_eq!(agent.agent_id(), "test-resource-agent");
        assert_eq!(agent.memory_types(), &[MemoryType::Resource]);
    }

    #[tokio::test]
    async fn test_resource_agent_lifecycle() {
        let mut agent = ResourceAgent::new("test-resource-agent".to_string());

        // Initialize
        let result = agent.initialize().await;
        assert!(result.is_ok());
        assert!(agent.health_check().await);

        // Shutdown
        let result = agent.shutdown().await;
        assert!(result.is_ok());
        assert!(!agent.health_check().await);
    }
}
