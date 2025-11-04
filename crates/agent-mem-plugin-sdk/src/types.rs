//! Common types for AgentMem plugins

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin type
    pub plugin_type: PluginType,

    /// Required capabilities
    pub required_capabilities: Vec<Capability>,

    /// Configuration schema
    pub config_schema: Option<serde_json::Value>,
}

/// Plugin type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    MemoryProcessor,
    CodeAnalyzer,
    SearchAlgorithm,
    DataSource,
    Multimodal,
    Custom(String),
}

/// Host capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    MemoryAccess,
    StorageAccess,
    SearchAccess,
    LLMAccess,
    NetworkAccess,
    FileSystemAccess,
    LoggingAccess,
    ConfigAccess,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }
}

/// Generic plugin request
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRequest<T> {
    pub id: String,
    pub operation: String,
    pub data: T,
    pub metadata: HashMap<String, String>,
}

/// Generic plugin response
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl<T> PluginResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: HashMap::new(),
        }
    }
}

/// Memory object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: String,
    pub user_id: String,
    pub agent_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub memory: Memory,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Code analysis input
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeInput {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
}

/// Code analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub language: String,
    pub functions: Vec<Function>,
    pub imports: Vec<String>,
    pub patterns: Vec<CodePattern>,
    pub complexity: i32,
}

/// Function definition
#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub parameters: Vec<String>,
}

/// Code pattern
#[derive(Debug, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: String,
    pub description: String,
    pub location: String,
}
