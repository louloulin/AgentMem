//! Data Source Plugin Example
//!
//! Demonstrates how to integrate external data sources and transform data into memories.

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data source configuration
#[derive(Deserialize)]
struct DataSourceConfig {
    source_type: String, // "database", "api", "file"
    connection_string: Option<String>,
    query_params: Option<HashMap<String, String>>,
}

/// Raw data from external source
#[derive(Serialize, Deserialize)]
struct RawData {
    id: String,
    content: String,
    timestamp: String,
    metadata: HashMap<String, serde_json::Value>,
}

/// Transformed memory
#[derive(Serialize)]
struct Memory {
    id: String,
    content: String,
    memory_type: String,
    user_id: String,
    metadata: HashMap<String, serde_json::Value>,
}

/// Data fetch request
#[derive(Deserialize)]
struct FetchRequest {
    config: DataSourceConfig,
    query: String,
    limit: Option<usize>,
}

/// Data fetch response
#[derive(Serialize)]
struct FetchResponse {
    success: bool,
    data: Vec<RawData>,
    count: usize,
    error: Option<String>,
}

/// Transform response
#[derive(Serialize)]
struct TransformResponse {
    success: bool,
    memories: Vec<Memory>,
    count: usize,
    error: Option<String>,
}

/// Fetch data from external source
#[plugin_fn]
pub fn fetch_data(input: String) -> FnResult<String> {
    // Parse input
    let request: FetchRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let response = FetchResponse {
                success: false,
                data: vec![],
                count: 0,
                error: Some(format!("Invalid request: {}", e)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    // Log fetch request
    log(
        LogLevel::Info,
        &format!(
            "Fetching data from {} with query: {}",
            request.config.source_type, request.query
        ),
    )?;

    // Simulate fetching data based on source type
    let data = match request.config.source_type.as_str() {
        "database" => fetch_from_database(&request.query, request.limit.unwrap_or(10)),
        "api" => fetch_from_api(&request.query, request.limit.unwrap_or(10)),
        "file" => fetch_from_file(&request.query, request.limit.unwrap_or(10)),
        _ => {
            let response = FetchResponse {
                success: false,
                data: vec![],
                count: 0,
                error: Some(format!("Unknown source type: {}", request.config.source_type)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    let count = data.len();
    let response = FetchResponse {
        success: true,
        data,
        count,
        error: None,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Transform raw data into memories
#[plugin_fn]
pub fn transform_data(input: String) -> FnResult<String> {
    #[derive(Deserialize)]
    struct TransformRequest {
        data: Vec<RawData>,
        user_id: String,
        memory_type: Option<String>,
    }

    // Parse input
    let request: TransformRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let response = TransformResponse {
                success: false,
                memories: vec![],
                count: 0,
                error: Some(format!("Invalid request: {}", e)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    log(
        LogLevel::Info,
        &format!("Transforming {} data items into memories", request.data.len()),
    )?;

    // Transform each data item into a memory
    let memories: Vec<Memory> = request
        .data
        .into_iter()
        .map(|item| {
            let mut metadata = item.metadata;
            metadata.insert("original_id".to_string(), serde_json::json!(item.id));
            metadata.insert("source_timestamp".to_string(), serde_json::json!(item.timestamp));

            Memory {
                id: format!("mem_{}", item.id),
                content: item.content,
                memory_type: request.memory_type.clone().unwrap_or_else(|| "Semantic".to_string()),
                user_id: request.user_id.clone(),
                metadata,
            }
        })
        .collect();

    let count = memories.len();
    let response = TransformResponse {
        success: true,
        memories,
        count,
        error: None,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Fetch and transform in one step
#[plugin_fn]
pub fn fetch_and_transform(input: String) -> FnResult<String> {
    #[derive(Deserialize)]
    struct FetchTransformRequest {
        config: DataSourceConfig,
        query: String,
        user_id: String,
        memory_type: Option<String>,
        limit: Option<usize>,
    }

    // Parse input
    let request: FetchTransformRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let response = TransformResponse {
                success: false,
                memories: vec![],
                count: 0,
                error: Some(format!("Invalid request: {}", e)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    // Fetch data
    let data = match request.config.source_type.as_str() {
        "database" => fetch_from_database(&request.query, request.limit.unwrap_or(10)),
        "api" => fetch_from_api(&request.query, request.limit.unwrap_or(10)),
        "file" => fetch_from_file(&request.query, request.limit.unwrap_or(10)),
        _ => {
            let response = TransformResponse {
                success: false,
                memories: vec![],
                count: 0,
                error: Some(format!("Unknown source type: {}", request.config.source_type)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    // Transform to memories
    let memories: Vec<Memory> = data
        .into_iter()
        .map(|item| {
            let mut metadata = item.metadata;
            metadata.insert("original_id".to_string(), serde_json::json!(item.id));
            metadata.insert("source_timestamp".to_string(), serde_json::json!(item.timestamp));

            Memory {
                id: format!("mem_{}", item.id),
                content: item.content,
                memory_type: request.memory_type.clone().unwrap_or_else(|| "Semantic".to_string()),
                user_id: request.user_id.clone(),
                metadata,
            }
        })
        .collect();

    let count = memories.len();
    let response = TransformResponse {
        success: true,
        memories,
        count,
        error: None,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Simulate fetching from database
fn fetch_from_database(query: &str, limit: usize) -> Vec<RawData> {
    // In a real implementation, this would:
    // 1. Connect to database
    // 2. Execute SQL query
    // 3. Return results
    
    (0..limit)
        .map(|i| RawData {
            id: format!("db_{}", i),
            content: format!("Database record matching '{}': Record {}", query, i),
            timestamp: "2025-11-04T12:00:00Z".to_string(),
            metadata: [
                ("source".to_string(), serde_json::json!("database")),
                ("table".to_string(), serde_json::json!("records")),
                ("query".to_string(), serde_json::json!(query)),
            ]
            .iter()
            .cloned()
            .collect(),
        })
        .collect()
}

/// Simulate fetching from API
fn fetch_from_api(query: &str, limit: usize) -> Vec<RawData> {
    // In a real implementation, this would:
    // 1. Make HTTP request to API
    // 2. Parse JSON response
    // 3. Return results
    
    (0..limit)
        .map(|i| RawData {
            id: format!("api_{}", i),
            content: format!("API result for '{}': Item {}", query, i),
            timestamp: "2025-11-04T12:00:00Z".to_string(),
            metadata: [
                ("source".to_string(), serde_json::json!("api")),
                ("endpoint".to_string(), serde_json::json!("/search")),
                ("query".to_string(), serde_json::json!(query)),
            ]
            .iter()
            .cloned()
            .collect(),
        })
        .collect()
}

/// Simulate fetching from file
fn fetch_from_file(query: &str, limit: usize) -> Vec<RawData> {
    // In a real implementation, this would:
    // 1. Open file
    // 2. Parse content (JSON, CSV, etc.)
    // 3. Filter based on query
    // 4. Return results
    
    (0..limit)
        .map(|i| RawData {
            id: format!("file_{}", i),
            content: format!("File entry containing '{}': Line {}", query, i),
            timestamp: "2025-11-04T12:00:00Z".to_string(),
            metadata: [
                ("source".to_string(), serde_json::json!("file")),
                ("filename".to_string(), serde_json::json!("data.json")),
                ("line_number".to_string(), serde_json::json!(i)),
            ]
            .iter()
            .cloned()
            .collect(),
        })
        .collect()
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = serde_json::json!({
        "name": "datasource-plugin",
        "version": "0.1.0",
        "description": "Integrates external data sources and transforms data into memories",
        "author": "AgentMem Team",
        "plugin_type": "DataSource",
        "required_capabilities": ["NetworkAccess", "LoggingAccess"],
        "supported_sources": ["database", "api", "file"]
    });

    Ok(metadata.to_string())
}

