//! Type conversion between AgentMem and MemVid

use crate::error::{MemvidError, Result};
use agent_mem_traits::{
    AttributeKey, AttributeSet, AttributeValue, Content, Memory, MemoryId, MetadataV4,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Converter for AgentMem Memory <-> MemVid Frame
pub struct MemoryConverter;

impl MemoryConverter {
    /// Convert AgentMem Memory to MemVid frame data
    pub fn memory_to_frame(memory: &Memory) -> Result<FrameData> {
        let content_bytes = Self::serialize_content(&memory.content)?;
        let metadata_json = Self::serialize_metadata(&memory.attributes, &memory.metadata)?;

        // Create tags from attributes
        let mut tags = HashMap::new();
        tags.insert("memory_id".to_string(), memory.id.as_str().to_string());
        tags.insert(
            "memory_type".to_string(),
            Self::get_memory_type_name(&memory.attributes),
        );

        // Add user/agent/session info
        if let Some(user_id) = memory.attributes.get(&AttributeKey::core("user_id")) {
            if let AttributeValue::String(id) = user_id {
                tags.insert("user_id".to_string(), id.clone());
            }
        }
        if let Some(agent_id) = memory.attributes.get(&AttributeKey::core("agent_id")) {
            if let AttributeValue::String(id) = agent_id {
                tags.insert("agent_id".to_string(), id.clone());
            }
        }

        Ok(FrameData {
            content: content_bytes,
            metadata: metadata_json,
            tags,
            timestamp: memory.metadata.created_at,
            vector: Self::extract_vector(&memory.content)?,
        })
    }

    /// Convert MemVid frame data to AgentMem Memory
    pub fn frame_to_memory(frame: &FrameData) -> Result<Memory> {
        let content = Self::deserialize_content(&frame.content)?;
        let (attributes, metadata) = Self::deserialize_metadata(&frame.metadata)?;

        // Extract ID from tags
        let memory_id = frame
            .tags
            .get("memory_id")
            .map(|id| MemoryId::from_string(id.clone()))
            .unwrap_or_else(MemoryId::new);

        Ok(Memory {
            id: memory_id,
            content,
            attributes,
            relations: Default::default(), // Relations loaded separately
            metadata,
        })
    }

    /// Serialize content to bytes
    fn serialize_content(content: &Content) -> Result<Vec<u8>> {
        serde_json::to_vec(content)
            .map_err(|e| MemvidError::Serialization(format!("content: {}", e)))
    }

    /// Deserialize content from bytes
    fn deserialize_content(bytes: &[u8]) -> Result<Content> {
        serde_json::from_slice(bytes)
            .map_err(|e| MemvidError::Deserialization(format!("content: {}", e)))
    }

    /// Serialize attributes and metadata to JSON
    fn serialize_metadata(attributes: &AttributeSet, metadata: &MetadataV4) -> Result<String> {
        let mut map = serde_json::Map::new();

        // Serialize attributes
        for (key, value) in &attributes.attributes {
            let key_str = format!("{}.{}", key.namespace, key.name);
            if let Some(val) = Self::attribute_value_to_json(value) {
                map.insert(key_str, val);
            }
        }

        // Add system metadata
        map.insert(
            "created_at".to_string(),
            serde_json::Value::String(metadata.created_at.to_rfc3339()),
        );
        map.insert(
            "updated_at".to_string(),
            serde_json::Value::String(metadata.updated_at.to_rfc3339()),
        );
        map.insert(
            "access_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metadata.access_count)),
        );

        serde_json::to_string(&map)
            .map_err(|e| MemvidError::Serialization(format!("metadata: {}", e)))
    }

    /// Deserialize metadata JSON to attributes and metadata
    fn deserialize_metadata(json: &str) -> Result<(AttributeSet, MetadataV4)> {
        let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(json)
            .map_err(|e| MemvidError::Deserialization(format!("metadata: {}", e)))?;

        let mut attributes = AttributeSet::new();
        let mut metadata = MetadataV4::default();

        for (key_str, value) in map {
            match key_str.as_str() {
                "created_at" => {
                    if let Ok(dt) = serde_json::from_value::<DateTime<Utc>>(value.clone()) {
                        metadata.created_at = dt;
                    }
                }
                "updated_at" => {
                    if let Ok(dt) = serde_json::from_value::<DateTime<Utc>>(value.clone()) {
                        metadata.updated_at = dt;
                    }
                }
                "access_count" => {
                    metadata.access_count = value.as_u64().unwrap_or(0) as u32;
                }
                _ => {
                    // Parse as attribute
                    if let Some(dot_pos) = key_str.find('.') {
                        let namespace = key_str[..dot_pos].to_string();
                        let name = key_str[dot_pos + 1..].to_string();
                        let key = AttributeKey { namespace, name };
                        let attr_value = Self::json_to_attribute_value(&value)?;
                        attributes.set(key, attr_value);
                    }
                }
            }
        }

        Ok((attributes, metadata))
    }

    /// Convert AttributeValue to JSON value
    fn attribute_value_to_json(value: &AttributeValue) -> Option<serde_json::Value> {
        match value {
            AttributeValue::String(s) => Some(serde_json::Value::String(s.clone())),
            AttributeValue::Number(n) => {
                serde_json::Number::from_f64(*n).map(serde_json::Value::Number)
            }
            AttributeValue::Integer(i) => {
                Some(serde_json::Value::Number(serde_json::Number::from(*i)))
            }
            AttributeValue::Boolean(b) => Some(serde_json::Value::Bool(*b)),
            AttributeValue::DateTime(dt) => Some(serde_json::Value::String(dt.to_rfc3339())),
            AttributeValue::List(items) => {
                let vals: Vec<serde_json::Value> = items
                    .iter()
                    .filter_map(|v| Self::attribute_value_to_json(v))
                    .collect();
                Some(serde_json::Value::Array(vals))
            }
            AttributeValue::Map(map) => {
                let obj: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .filter_map(|(k, v)| {
                        Self::attribute_value_to_json(v).map(|val| (k.clone(), val))
                    })
                    .collect();
                Some(serde_json::Value::Object(obj))
            }
            AttributeValue::Null => Some(serde_json::Value::Null),
        }
    }

    /// Convert JSON value to AttributeValue
    fn json_to_attribute_value(value: &serde_json::Value) -> Result<AttributeValue> {
        match value {
            serde_json::Value::Null => Ok(AttributeValue::Null),
            serde_json::Value::Bool(b) => Ok(AttributeValue::Boolean(*b)),
            serde_json::Value::Number(n) => {
                // Try as integer first, then fall back to float
                if let Some(i) = n.as_i64() {
                    Ok(AttributeValue::Integer(i))
                } else {
                    Ok(AttributeValue::Number(n.as_f64().unwrap_or(0.0)))
                }
            }
            serde_json::Value::String(s) => {
                // Try parsing as datetime
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    Ok(AttributeValue::DateTime(dt.with_timezone(&Utc)))
                } else {
                    Ok(AttributeValue::String(s.clone()))
                }
            }
            serde_json::Value::Array(items) => {
                let vals: Result<Vec<AttributeValue>> = items
                    .iter()
                    .map(|v| Self::json_to_attribute_value(v))
                    .collect();
                Ok(AttributeValue::List(vals?))
            }
            serde_json::Value::Object(map) => {
                let vals: Result<HashMap<String, AttributeValue>> = map
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), Self::json_to_attribute_value(v)?)))
                    .collect();
                Ok(AttributeValue::Map(vals?))
            }
        }
    }

    /// Extract memory type name from attributes
    fn get_memory_type_name(attributes: &AttributeSet) -> String {
        attributes
            .get(&AttributeKey::core("memory_type"))
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "episodic".to_string())
    }

    /// Extract vector from content if present
    fn extract_vector(content: &Content) -> Result<Option<Vec<f32>>> {
        match content {
            Content::Vector(v) => Ok(Some(v.clone())),
            Content::Text(_)
            | Content::Structured(_)
            | Content::Binary(_)
            | Content::Multimodal(_) => Ok(None),
        }
    }
}

/// Frame data structure for MemVid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    /// Serialized content
    pub content: Vec<u8>,

    /// Metadata as JSON string
    pub metadata: String,

    /// Tags for filtering
    pub tags: HashMap<String, String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional vector embedding
    pub vector: Option<Vec<f32>>,
}

/// Converter for MemVid frames
pub trait FrameConverter: Send + Sync {
    /// Convert frame to bytes for storage
    fn to_bytes(&self, frame: &FrameData) -> Result<Vec<u8>>;

    /// Convert bytes to frame
    fn from_bytes(&self, bytes: &[u8]) -> Result<FrameData>;
}

/// Default frame converter using MessagePack
pub struct MsgPackConverter;

impl FrameConverter for MsgPackConverter {
    fn to_bytes(&self, frame: &FrameData) -> Result<Vec<u8>> {
        // Use JSON for now, can be optimized with MessagePack later
        serde_json::to_vec(frame).map_err(|e| MemvidError::Serialization(format!("frame: {}", e)))
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<FrameData> {
        serde_json::from_slice(bytes)
            .map_err(|e| MemvidError::Deserialization(format!("frame: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_to_frame_conversion() {
        let memory = Memory {
            id: MemoryId::from_string("test-id".to_string()),
            content: Content::text("Hello, world!"),
            attributes: AttributeSet::new().with_attribute(
                AttributeKey::core("user_id"),
                AttributeValue::String("user-123".to_string()),
            ),
            relations: Default::default(),
            metadata: MetadataV4::default(),
        };

        let frame = MemoryConverter::memory_to_frame(&memory).unwrap();
        assert_eq!(frame.tags.get("memory_id"), Some(&"test-id".to_string()));
        assert_eq!(frame.tags.get("user_id"), Some(&"user-123".to_string()));
    }

    #[test]
    fn test_frame_to_memory_conversion() {
        let mut tags = HashMap::new();
        tags.insert("memory_id".to_string(), "test-id".to_string());

        let frame = FrameData {
            content: serde_json::to_vec(&Content::text("Hello, world!")).unwrap(),
            metadata: "{}".to_string(),
            tags,
            timestamp: Utc::now(),
            vector: None,
        };

        let memory = MemoryConverter::frame_to_memory(&frame).unwrap();
        assert_eq!(memory.id.as_str(), "test-id");
    }
}
