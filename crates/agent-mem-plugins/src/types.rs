//! Common types for plugin management

use agent_mem_plugin_sdk::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    Registered,
    Loading,
    Loaded,
    Running,
    Stopped,
    Error(String),
}

/// Registered plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredPlugin {
    pub id: String,
    pub metadata: PluginMetadata,
    pub path: String,
    pub status: PluginStatus,
    pub config: PluginConfig,
    pub registered_at: DateTime<Utc>,
    pub last_loaded_at: Option<DateTime<Utc>>,
}

/// Loaded plugin instance
pub struct LoadedPlugin {
    pub id: String,
    pub metadata: PluginMetadata,
    pub plugin: extism::Plugin,
}

