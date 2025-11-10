//! V4 Migration Layer
//!
//! Provides migration utilities and adapters to transition from old API to V4 abstractions.
//! This module helps maintain backward compatibility during the migration period.

use agent_mem_traits::{
    AttributeKey, AttributeSet, AttributeValue, Content, Memory as MemoryV4, MemoryId,
    MemoryItem as LegacyMemoryItem, Metadata, Query, QueryIntent, RelationGraph,
};
use agent_mem_traits::types::Relation as LegacyRelation;  // 旧Relation (source/target/relation/confidence)
use chrono::Utc;
use std::collections::HashMap;

/// Migration context for tracking conversions
pub struct MigrationContext {
    pub converted_count: usize,
    pub errors: Vec<MigrationError>,
}

#[derive(Debug, Clone)]
pub struct MigrationError {
    pub item_id: String,
    pub error: String,
}

impl MigrationContext {
    pub fn new() -> Self {
        Self {
            converted_count: 0,
            errors: vec![],
        }
    }
}

impl Default for MigrationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert legacy MemoryItem to V4 Memory
pub fn legacy_to_v4(item: &LegacyMemoryItem) -> MemoryV4 {
    let mut attributes = AttributeSet::new();
    
    // Map core fields
    if let Some(user_id) = &item.user_id {
        attributes.set(
            AttributeKey::core("user_id"),
            AttributeValue::String(user_id.clone()),
        );
    }
    
    attributes.set(
        AttributeKey::core("agent_id"),
        AttributeValue::String(item.agent_id.clone()),
    );
    
    attributes.set(
        AttributeKey::core("memory_type"),
        AttributeValue::String(item.memory_type.as_str().to_string()),
    );
    
    attributes.set(
        AttributeKey::core("importance"),
        AttributeValue::Number(item.importance as f64),
    );
    
    // Map metadata
    for (key, value) in &item.metadata {
        let attr_key = AttributeKey::new("metadata", key);
        let attr_value = match value {
            serde_json::Value::String(s) => AttributeValue::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    AttributeValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    AttributeValue::Number(f)
                } else {
                    AttributeValue::String(value.to_string())
                }
            }
            serde_json::Value::Bool(b) => AttributeValue::Boolean(*b),
            _ => AttributeValue::String(value.to_string()),
        };
        attributes.set(attr_key, attr_value);
    }
    
    // Map entities as attributes
    for (idx, entity) in item.entities.iter().enumerate() {
        attributes.set(
            AttributeKey::new("entity", &format!("{}_{}", idx, entity.name)),
            AttributeValue::String(entity.entity_type.clone()),
        );
    }
    
    // Build relations graph
    let mut relations = RelationGraph::new();
    for relation in &item.relations {
        // Convert legacy relation to V4 format
        relations.add_outgoing(agent_mem_traits::Relation {
            relation_type: relation.relation_type.clone(),
            source: relation.source.clone(),
            target: relation.target.clone(),
            confidence: relation.confidence,
        });
    }
    
    MemoryV4 {
        id: MemoryId::from_string(item.id.clone()),
        content: Content::text(item.content.clone()),
        attributes,
        relations,
        metadata: Metadata {
            created_at: item.created_at,
            updated_at: item.updated_at.unwrap_or(item.created_at),
            accessed_at: item.last_accessed_at,
            access_count: item.access_count,
            version: item.version,
            hash: item.hash.clone(),
        },
    }
}

/// Convert V4 Memory to legacy MemoryItem
pub fn v4_to_legacy(memory: &MemoryV4) -> LegacyMemoryItem {
    use agent_mem_traits::types::{Entity, MemoryType, Relation as LegacyRelation, Session};
    
    let user_id = memory
        .attributes
        .get(&AttributeKey::core("user_id"))
        .and_then(|v| v.as_string())
        .cloned();
    
    let agent_id = memory
        .attributes
        .get(&AttributeKey::core("agent_id"))
        .and_then(|v| v.as_string())
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    
    let importance = memory
        .attributes
        .get(&AttributeKey::core("importance"))
        .and_then(|v| v.as_number())
        .unwrap_or(0.5) as f32;
    
    let memory_type_str = memory
        .attributes
        .get(&AttributeKey::core("memory_type"))
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .unwrap_or("episodic");
    
    let memory_type = MemoryType::parse_type(memory_type_str).unwrap_or(MemoryType::Episodic);
    
    let content = match &memory.content {
        Content::Text(s) => s.clone(),
        Content::Structured(v) => v.to_string(),
        _ => String::new(),
    };
    
    // Extract entities from attributes
    let entities: Vec<Entity> = memory
        .attributes
        .attributes
        .iter()
        .filter(|(k, _)| k.namespace == "entity")
        .map(|(k, v)| Entity {
            id: uuid::Uuid::new_v4().to_string(),
            name: k.name.clone(),
            entity_type: v.as_string().cloned().unwrap_or_default(),
            attributes: HashMap::new(),
        })
        .collect();
    
    // Convert relations
    let relations: Vec<LegacyRelation> = memory
        .relations
        .outgoing
        .iter()
        .map(|r| LegacyRelation {
            relation_type: r.relation_type.clone(),
            source: memory.id.0.clone(),
            target: r.target.clone(),
            confidence: r.confidence,
        })
        .collect();
    
    // Extract metadata
    let metadata: HashMap<String, serde_json::Value> = memory
        .attributes
        .attributes
        .iter()
        .filter(|(k, _)| k.namespace == "metadata")
        .map(|(k, v)| {
            let value = match v {
                AttributeValue::String(s) => serde_json::Value::String(s.clone()),
                AttributeValue::Number(n) => {
                    serde_json::Value::Number(serde_json::Number::from_f64(n.clone()).unwrap())
                }
                AttributeValue::Integer(i) => serde_json::Value::Number(i.clone().into()),
                AttributeValue::Boolean(b) => serde_json::Value::Bool(b.clone()),
                _ => serde_json::Value::String(format!("{:?}", v)),
            };
            (k.name.clone(), value)
        })
        .collect();
    
    LegacyMemoryItem {
        id: memory.id.0.clone(),
        content,
        hash: memory.metadata.hash.clone(),
        metadata,
        score: None,
        created_at: memory.metadata.created_at,
        updated_at: Some(memory.metadata.updated_at),
        session: Session::new().with_user_id(user_id.clone()).with_agent_id(Some(agent_id.clone())),
        memory_type,
        entities,
        relations,
        agent_id,
        user_id,
        importance,
        embedding: None,
        last_accessed_at: memory.metadata.accessed_at,
        access_count: memory.metadata.access_count,
        expires_at: None,
        version: memory.metadata.version,
    }
}

/// Batch convert legacy items to V4
pub fn batch_legacy_to_v4(items: &[LegacyMemoryItem]) -> (Vec<MemoryV4>, MigrationContext) {
    let mut context = MigrationContext::new();
    let mut memories = Vec::with_capacity(items.len());
    
    for item in items {
        memories.push(legacy_to_v4(item));
        context.converted_count += 1;
    }
    
    (memories, context)
}

/// Batch convert V4 memories to legacy
pub fn batch_v4_to_legacy(memories: &[MemoryV4]) -> (Vec<LegacyMemoryItem>, MigrationContext) {
    let mut context = MigrationContext::new();
    let mut items = Vec::with_capacity(memories.len());
    
    for memory in memories {
        items.push(v4_to_legacy(memory));
        context.converted_count += 1;
    }
    
    (items, context)
}

/// Convert simple string query to V4 Query
pub fn string_to_v4_query(query: impl Into<String>) -> Query {
    Query::new(QueryIntent::natural_language(query))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::types::MemoryType;
    
    #[test]
    fn test_legacy_to_v4_conversion() {
        use agent_mem_traits::Session;
        
        let legacy = LegacyMemoryItem {
            id: "test-id".to_string(),
            content: "Test content".to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: None,
            created_at: Utc::now(),
            updated_at: None,
            session: Session::new(),
            memory_type: MemoryType::Episodic,
            entities: vec![],
            relations: vec![],
            agent_id: "agent-1".to_string(),
            user_id: Some("user-1".to_string()),
            importance: 0.8,
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 1,
            expires_at: None,
            version: 1,
        };
        
        let v4 = legacy_to_v4(&legacy);
        
        assert_eq!(v4.id.0, "test-id");
        assert!(matches!(v4.content, Content::Text(_)));
        assert!(v4
            .attributes
            .get(&AttributeKey::core("agent_id"))
            .is_some());
    }
    
    #[test]
    fn test_v4_to_legacy_conversion() {
        let mut attributes = AttributeSet::new();
        attributes.set(
            AttributeKey::core("agent_id"),
            AttributeValue::String("agent-1".to_string()),
        );
        attributes.set(
            AttributeKey::core("importance"),
            AttributeValue::Number(0.8),
        );
        
        let v4 = MemoryV4 {
            id: MemoryId::from_string("test-id".to_string()),
            content: Content::text("Test content"),
            attributes,
            relations: RelationGraph::new(),
            metadata: Metadata::default(),
        };
        
        let legacy = v4_to_legacy(&v4);
        
        assert_eq!(legacy.id, "test-id");
        assert_eq!(legacy.content, "Test content");
        assert_eq!(legacy.agent_id, "agent-1");
        assert!((legacy.importance - 0.8).abs() < 0.01);
    }
    
    #[test]
    fn test_roundtrip_conversion() {
        use agent_mem_traits::Session;
        
        let original = LegacyMemoryItem {
            id: "roundtrip-test".to_string(),
            content: "Roundtrip content".to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: None,
            created_at: Utc::now(),
            updated_at: None,
            session: Session::new(),
            memory_type: MemoryType::Semantic,
            entities: vec![],
            relations: vec![],
            agent_id: "agent-roundtrip".to_string(),
            user_id: Some("user-roundtrip".to_string()),
            importance: 0.75,
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 5,
            expires_at: None,
            version: 2,
        };
        
        let v4 = legacy_to_v4(&original);
        let converted_back = v4_to_legacy(&v4);
        
        assert_eq!(original.id, converted_back.id);
        assert_eq!(original.content, converted_back.content);
        assert_eq!(original.agent_id, converted_back.agent_id);
        assert_eq!(original.user_id, converted_back.user_id);
    }
}

