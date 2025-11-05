//! Plugin management HTTP API routes
//!
//! Provides HTTP endpoints for managing WASM plugins in AgentMem.

#[cfg(feature = "plugins")]
use agent_mem::plugins::{PluginStatus, RegisteredPlugin};
#[cfg(feature = "plugins")]
use agent_mem::plugins::sdk::{Capability, PluginConfig, PluginMetadata, PluginType};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::error::{ServerError, ServerResult};
use crate::routes::memory::MemoryManager;

/// Request to register a new plugin
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RegisterPluginRequest {
    /// Plugin ID (unique identifier)
    pub id: String,
    /// Plugin metadata
    pub metadata: PluginMetadataDto,
    /// Path to the WASM plugin file
    pub path: String,
    /// Plugin configuration
    #[serde(default)]
    pub config: PluginConfigDto,
}

/// Plugin metadata DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginMetadataDto {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginTypeDto,
    pub required_capabilities: Vec<CapabilityDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
}

/// Plugin type DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PluginTypeDto {
    MemoryProcessor,
    CodeAnalyzer,
    SearchAlgorithm,
    DataSource,
    Multimodal,
    Custom(String),
}

/// Capability DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDto {
    MemoryAccess,
    StorageAccess,
    SearchAccess,
    LlmAccess,
    NetworkAccess,
    FileSystemAccess,
    LoggingAccess,
    ConfigAccess,
}

/// Plugin configuration DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PluginConfigDto {
    #[serde(default)]
    pub settings: serde_json::Value,
}

/// Plugin response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginTypeDto,
    pub status: PluginStatusDto,
    pub required_capabilities: Vec<CapabilityDto>,
    pub registered_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_loaded_at: Option<String>,
}

/// Plugin status DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatusDto {
    Registered,
    Loading,
    Loaded,
    Running,
    Stopped,
    Error { message: String },
}

/// List all registered plugins
///
/// Returns a list of all registered plugins with their metadata and status.
#[cfg(feature = "plugins")]
#[utoipa::path(
    get,
    path = "/api/v1/plugins",
    tag = "plugins",
    responses(
        (status = 200, description = "List of registered plugins", body = Vec<PluginResponse>),
        (status = 500, description = "Internal server error", body = ServerError)
    )
)]
pub async fn list_plugins(
    State(memory_manager): State<Arc<MemoryManager>>,
) -> ServerResult<Json<Vec<PluginResponse>>> {
    info!("Listing all registered plugins");
    
    #[cfg(feature = "plugins")]
    {
        let plugins = memory_manager.memory.list_plugins().await;
        
        let response: Vec<PluginResponse> = plugins
            .into_iter()
            .map(|metadata| PluginResponse {
                id: metadata.name.clone(), // 使用 name 作为 ID
                name: metadata.name,
                version: metadata.version,
                description: metadata.description,
                author: metadata.author,
                plugin_type: convert_plugin_type(&metadata.plugin_type),
                status: PluginStatusDto::Registered, // 默认状态
                required_capabilities: metadata
                    .required_capabilities
                    .iter()
                    .map(convert_capability)
                    .collect(),
                registered_at: chrono::Utc::now().to_rfc3339(),
                last_loaded_at: None,
            })
            .collect();
        
        debug!("Found {} plugins", response.len());
        Ok(Json(response))
    }
    
    #[cfg(not(feature = "plugins"))]
    {
        Err(ServerError::internal("Plugins feature is not enabled"))
    }
}

/// Register a new plugin
///
/// Registers a new WASM plugin with the specified metadata and configuration.
#[cfg(feature = "plugins")]
#[utoipa::path(
    post,
    path = "/api/v1/plugins",
    tag = "plugins",
    request_body = RegisterPluginRequest,
    responses(
        (status = 201, description = "Plugin registered successfully", body = PluginResponse),
        (status = 400, description = "Invalid request", body = ServerError),
        (status = 500, description = "Internal server error", body = ServerError)
    )
)]
pub async fn register_plugin(
    State(memory_manager): State<Arc<MemoryManager>>,
    Json(request): Json<RegisterPluginRequest>,
) -> ServerResult<(StatusCode, Json<PluginResponse>)> {
    info!("Registering plugin: {}", request.id);
    
    // Convert DTO to domain model
    let metadata = PluginMetadata {
        name: request.metadata.name.clone(),
        version: request.metadata.version.clone(),
        description: request.metadata.description.clone(),
        author: request.metadata.author.clone(),
        plugin_type: convert_plugin_type_from_dto(&request.metadata.plugin_type),
        required_capabilities: request
            .metadata
            .required_capabilities
            .iter()
            .map(convert_capability_from_dto)
            .collect(),
        config_schema: request.metadata.config_schema,
    };
    
    let mut config = PluginConfig::default();
    if let serde_json::Value::Object(map) = request.config.settings {
        for (key, value) in map {
            config.settings.insert(key, value);
        }
    }
    
    let plugin = RegisteredPlugin {
        id: request.id.clone(),
        metadata: metadata.clone(),
        path: request.path,
        status: PluginStatus::Registered,
        config,
        registered_at: chrono::Utc::now(),
        last_loaded_at: None,
    };
    
    // Register the plugin
    memory_manager.memory
        .register_plugin(plugin)
        .await
        .map_err(|e| ServerError::ServerError(format!("Failed to register plugin: {}", e)))?;
    
    info!("Plugin {} registered successfully", request.id);
    
    // Build response
    let response = PluginResponse {
        id: request.id,
        name: metadata.name,
        version: metadata.version,
        description: metadata.description,
        author: metadata.author,
        plugin_type: convert_plugin_type(&metadata.plugin_type),
        status: PluginStatusDto::Registered,
        required_capabilities: metadata
            .required_capabilities
            .iter()
            .map(convert_capability)
            .collect(),
        registered_at: chrono::Utc::now().to_rfc3339(),
        last_loaded_at: None,
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get plugin details by ID
///
/// Returns detailed information about a specific plugin.
#[cfg(feature = "plugins")]
#[utoipa::path(
    get,
    path = "/api/v1/plugins/{id}",
    tag = "plugins",
    params(
        ("id" = String, Path, description = "Plugin ID")
    ),
    responses(
        (status = 200, description = "Plugin details", body = PluginResponse),
        (status = 404, description = "Plugin not found", body = ServerError),
        (status = 500, description = "Internal server error", body = ServerError)
    )
)]
pub async fn get_plugin(
    State(memory_manager): State<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<PluginResponse>> {
    debug!("Getting plugin: {}", id);
    
    let plugins = memory_manager.memory.list_plugins().await;
    
    // Find plugin by name (using name as ID)
    let plugin = plugins
        .into_iter()
        .find(|p| p.name == id)
        .ok_or_else(|| ServerError::NotFound(format!("Plugin not found: {}", id)))?;
    
    let response = PluginResponse {
        id: plugin.name.clone(),
        name: plugin.name,
        version: plugin.version,
        description: plugin.description,
        author: plugin.author,
        plugin_type: convert_plugin_type(&plugin.plugin_type),
        status: PluginStatusDto::Registered,
        required_capabilities: plugin
            .required_capabilities
            .iter()
            .map(convert_capability)
            .collect(),
        registered_at: chrono::Utc::now().to_rfc3339(),
        last_loaded_at: None,
    };
    
    Ok(Json(response))
}

// Conversion helpers

#[cfg(feature = "plugins")]
fn convert_plugin_type(pt: &PluginType) -> PluginTypeDto {
    match pt {
        PluginType::MemoryProcessor => PluginTypeDto::MemoryProcessor,
        PluginType::CodeAnalyzer => PluginTypeDto::CodeAnalyzer,
        PluginType::SearchAlgorithm => PluginTypeDto::SearchAlgorithm,
        PluginType::DataSource => PluginTypeDto::DataSource,
        PluginType::Multimodal => PluginTypeDto::Multimodal,
        PluginType::Custom(s) => PluginTypeDto::Custom(s.clone()),
    }
}

#[cfg(feature = "plugins")]
fn convert_plugin_type_from_dto(dto: &PluginTypeDto) -> PluginType {
    match dto {
        PluginTypeDto::MemoryProcessor => PluginType::MemoryProcessor,
        PluginTypeDto::CodeAnalyzer => PluginType::CodeAnalyzer,
        PluginTypeDto::SearchAlgorithm => PluginType::SearchAlgorithm,
        PluginTypeDto::DataSource => PluginType::DataSource,
        PluginTypeDto::Multimodal => PluginType::Multimodal,
        PluginTypeDto::Custom(s) => PluginType::Custom(s.clone()),
    }
}

#[cfg(feature = "plugins")]
fn convert_capability(cap: &Capability) -> CapabilityDto {
    match cap {
        Capability::MemoryAccess => CapabilityDto::MemoryAccess,
        Capability::StorageAccess => CapabilityDto::StorageAccess,
        Capability::SearchAccess => CapabilityDto::SearchAccess,
        Capability::LLMAccess => CapabilityDto::LlmAccess,
        Capability::NetworkAccess => CapabilityDto::NetworkAccess,
        Capability::FileSystemAccess => CapabilityDto::FileSystemAccess,
        Capability::LoggingAccess => CapabilityDto::LoggingAccess,
        Capability::ConfigAccess => CapabilityDto::ConfigAccess,
    }
}

#[cfg(feature = "plugins")]
fn convert_capability_from_dto(dto: &CapabilityDto) -> Capability {
    match dto {
        CapabilityDto::MemoryAccess => Capability::MemoryAccess,
        CapabilityDto::StorageAccess => Capability::StorageAccess,
        CapabilityDto::SearchAccess => Capability::SearchAccess,
        CapabilityDto::LlmAccess => Capability::LLMAccess,
        CapabilityDto::NetworkAccess => Capability::NetworkAccess,
        CapabilityDto::FileSystemAccess => Capability::FileSystemAccess,
        CapabilityDto::LoggingAccess => Capability::LoggingAccess,
        CapabilityDto::ConfigAccess => Capability::ConfigAccess,
    }
}

// Placeholder functions for when plugins feature is not enabled
#[cfg(not(feature = "plugins"))]
pub async fn list_plugins(
    State(_memory_manager): State<Arc<MemoryManager>>,
) -> ServerResult<Json<Vec<PluginResponse>>> {
    Err(ServerError::ServerError(
        "Plugins feature is not enabled".to_string(),
    ))
}

#[cfg(not(feature = "plugins"))]
pub async fn register_plugin(
    State(_memory_manager): State<Arc<MemoryManager>>,
    Json(_request): Json<RegisterPluginRequest>,
) -> ServerResult<(StatusCode, Json<PluginResponse>)> {
    Err(ServerError::ServerError(
        "Plugins feature is not enabled".to_string(),
    ))
}

#[cfg(not(feature = "plugins"))]
pub async fn get_plugin(
    State(_memory_manager): State<Arc<MemoryManager>>,
    Path(_id): Path<String>,
) -> ServerResult<Json<PluginResponse>> {
    Err(ServerError::ServerError(
        "Plugins feature is not enabled".to_string(),
    ))
}

