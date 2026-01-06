//! Memory <-> DbMemory Conversion Layer
//!
//! Provides zero-copy conversions between business model (Memory V4)
//! and database model (DbMemory)

use crate::storage::models::DbMemory;
use agent_mem_traits::{
    AttributeKey, AttributeSet, AttributeValue, Content, MemoryId,
    MemoryV4 as Memory, MetadataV4 as Metadata, RelationGraph, Result,
};
use chrono::Utc;
use std::collections::HashMap;

/// Convert V4 Memory to Database Memory
pub fn memory_to_db(memory: &Memory) -> DbMemory {
    DbMemory {
        id: memory.id.as_str().to_string(),
        organization_id: memory
            .attributes
            .get(&AttributeKey::core("organization_id"))
            .and_then(|v| v.as_string())
            .unwrap_or(&String::new())
            .clone(),
        user_id: memory
            .attributes
            .get(&AttributeKey::core("user_id"))
            .and_then(|v| v.as_string())
            .unwrap_or(&String::new())
            .clone(),
        agent_id: memory
            .attributes
            .get(&AttributeKey::core("agent_id"))
            .and_then(|v| v.as_string())
            .unwrap_or(&String::new())
            .clone(),
        content: match &memory.content {
            Content::Text(t) => t.clone(),
            Content::Structured(v) => v.to_string(),
            _ => String::new(),
        },
        hash: memory.metadata.hash.clone(),
        metadata: serde_json::to_value(&memory.metadata).unwrap_or(serde_json::json!({})),
        score: memory
            .attributes
            .get(&AttributeKey::system("score"))
            .and_then(|v| v.as_number())
            .map(|n| n as f32),
        memory_type: memory
            .attributes
            .get(&AttributeKey::core("memory_type"))
            .and_then(|v| v.as_string())
            .unwrap_or(&"core".to_string())
            .clone(),
        scope: memory
            .attributes
            .get(&AttributeKey::core("scope"))
            .and_then(|v| v.as_string())
            .unwrap_or(&"global".to_string())
            .clone(),
        level: memory
            .attributes
            .get(&AttributeKey::core("level"))
            .and_then(|v| v.as_string())
            .unwrap_or(&"episodic".to_string())
            .clone(),
        importance: memory
            .attributes
            .get(&AttributeKey::system("importance"))
            .and_then(|v| v.as_number())
            .unwrap_or(0.5) as f32,
        access_count: memory.metadata.access_count as i64,
        last_accessed: Some(memory.metadata.accessed_at),
        created_at: memory.metadata.created_at,
        updated_at: memory.metadata.updated_at,
        is_deleted: memory
            .attributes
            .get(&AttributeKey::system("is_deleted"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        created_by_id: memory
            .attributes
            .get(&AttributeKey::system("created_by_id"))
            .and_then(|v| v.as_string())
            .cloned(),
        last_updated_by_id: memory
            .attributes
            .get(&AttributeKey::system("last_updated_by_id"))
            .and_then(|v| v.as_string())
            .cloned(),
    }
}

/// Convert Database Memory to V4 Memory
pub fn db_to_memory(db: &DbMemory) -> Result<Memory> {
    let mut attributes = AttributeSet::new();

    // 填充核心属性（只在非空时插入）
    if !db.organization_id.is_empty() {
        attributes.insert(
            AttributeKey::core("organization_id"),
            AttributeValue::String(db.organization_id.clone()),
        );
    }
    if !db.user_id.is_empty() {
        attributes.insert(
            AttributeKey::core("user_id"),
            AttributeValue::String(db.user_id.clone()),
        );
    }
    if !db.agent_id.is_empty() {
        attributes.insert(
            AttributeKey::core("agent_id"),
            AttributeValue::String(db.agent_id.clone()),
        );
    }
    if !db.memory_type.is_empty() {
        attributes.insert(
            AttributeKey::core("memory_type"),
            AttributeValue::String(db.memory_type.clone()),
        );
    }
    if !db.scope.is_empty() {
        attributes.insert(
            AttributeKey::core("scope"),
            AttributeValue::String(db.scope.clone()),
        );
    }
    if !db.level.is_empty() {
        attributes.insert(
            AttributeKey::core("level"),
            AttributeValue::String(db.level.clone()),
        );
    }

    // 填充系统属性
    if let Some(score) = db.score {
        attributes.insert(
            AttributeKey::system("score"),
            AttributeValue::Number(score as f64),
        );
    }
    attributes.insert(
        AttributeKey::system("importance"),
        AttributeValue::Number(db.importance as f64),
    );
    attributes.insert(
        AttributeKey::system("is_deleted"),
        AttributeValue::Boolean(db.is_deleted),
    );
    if let Some(created_by_id) = &db.created_by_id {
        attributes.insert(
            AttributeKey::system("created_by_id"),
            AttributeValue::String(created_by_id.clone()),
        );
    }
    if let Some(last_updated_by_id) = &db.last_updated_by_id {
        attributes.insert(
            AttributeKey::system("last_updated_by_id"),
            AttributeValue::String(last_updated_by_id.clone()),
        );
    }

    // 构造 Metadata（尝试从 JSON 反序列化，失败则使用默认值）
    let metadata = if let Ok(meta) = serde_json::from_value::<Metadata>(db.metadata.clone()) {
        meta
    } else {
        Metadata {
            created_at: db.created_at,
            updated_at: db.updated_at,
            accessed_at: db.last_accessed.unwrap_or_else(Utc::now),
            access_count: db.access_count as u32,
            version: 1,
            hash: db.hash.clone(),
        }
    };

    // 构造Memory
    Ok(Memory {
        id: MemoryId::from_string(db.id.clone()),
        content: Content::Text(db.content.clone()),
        attributes,
        relations: RelationGraph::new(),
        metadata,
    })
}

/// Batch conversion: Vec<Memory> -> Vec<DbMemory>
pub fn memories_to_db(memories: &[Memory]) -> Vec<DbMemory> {
    memories.iter().map(memory_to_db).collect()
}

/// Batch conversion: Vec<DbMemory> -> Vec<Memory>
pub fn db_to_memories(db_memories: &[DbMemory]) -> Result<Vec<Memory>> {
    db_memories.iter().map(db_to_memory).collect()
}

/// Helper: Convert MemoryItem (Legacy) to Memory (V4)
pub fn legacy_to_v4(item: &agent_mem_traits::MemoryItem) -> Memory {
    let mut attributes = AttributeSet::new();

    // Core attributes
    attributes.insert(
        AttributeKey::core("agent_id"),
        AttributeValue::String(item.agent_id.clone()),
    );
    if let Some(user_id) = &item.user_id {
        attributes.insert(
            AttributeKey::core("user_id"),
            AttributeValue::String(user_id.clone()),
        );
    }

    // System attributes
    if let Some(hash) = item.hash.clone() {
        attributes.insert(AttributeKey::system("hash"), AttributeValue::String(hash));
    }
    if let Some(score) = item.score {
        attributes.insert(
            AttributeKey::system("score"),
            AttributeValue::Number(score as f64),
        );
    }
    attributes.insert(
        AttributeKey::system("importance"),
        AttributeValue::Number(item.importance as f64),
    );

    // Metadata
    let metadata = Metadata {
        created_at: item.created_at,
        updated_at: item.updated_at.unwrap_or(item.created_at),
        accessed_at: item.created_at,
        access_count: 0,
        version: 1,
        hash: item.hash.clone(),
    };

    Memory {
        id: MemoryId::from_string(item.id.clone()),
        content: Content::Text(item.content.clone()),
        attributes,
        relations: RelationGraph::new(),
        metadata,
    }
}

/// Helper: Convert Memory (V4) to MemoryItem (Legacy)
pub fn v4_to_legacy(memory: &Memory) -> agent_mem_traits::MemoryItem {
    use agent_mem_traits::{MemoryItem, MemoryType, Session};

    let agent_id = memory
        .attributes
        .get(&AttributeKey::core("agent_id"))
        .and_then(|v| v.as_string())
        .unwrap_or(&"".to_string())
        .to_string();

    let user_id = memory
        .attributes
        .get(&AttributeKey::core("user_id"))
        .and_then(|v| v.as_string())
        .cloned();

    let hash = memory.metadata.hash.clone();

    let score = memory
        .attributes
        .get(&AttributeKey::system("score"))
        .and_then(|v| v.as_number())
        .map(|n| n as f32);

    let importance = memory
        .attributes
        .get(&AttributeKey::system("importance"))
        .and_then(|v| v.as_number())
        .unwrap_or(0.5) as f32;

    let content = match &memory.content {
        Content::Text(t) => t.clone(),
        Content::Structured(v) => v.to_string(),
        _ => String::new(),
    };

    MemoryItem {
        id: memory.id.as_str().to_string(),
        agent_id: agent_id.clone(),
        user_id: user_id.clone(),
        content,
        hash,
        metadata: HashMap::new(),
        score,
        created_at: memory.metadata.created_at,
        updated_at: Some(memory.metadata.updated_at),
        memory_type: MemoryType::Semantic,
        importance,
        session: Session {
            id: format!("session-{}", memory.id),
            user_id: user_id.clone(),
            agent_id: Some(agent_id),
            run_id: None,
            actor_id: None,
            created_at: memory.metadata.created_at,
            metadata: HashMap::new(),
        },
        entities: Vec::new(),
        relations: Vec::new(),
        embedding: None,
        last_accessed_at: memory.metadata.accessed_at,
        access_count: memory.metadata.access_count,
        expires_at: None,
        version: memory.metadata.version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_conversion() {
        let mut memory = Memory {
            id: MemoryId::new(),
            content: Content::text("Test content"),
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
                version: 1,
                hash: None,
            },
        };

        memory.attributes.insert(
            AttributeKey::core("agent_id"),
            AttributeValue::String("agent_123".to_string()),
        );
        memory.attributes.insert(
            AttributeKey::core("user_id"),
            AttributeValue::String("user_456".to_string()),
        );
        memory.attributes.insert(
            AttributeKey::core("organization_id"),
            AttributeValue::String("org_789".to_string()),
        );

        // Memory -> DbMemory -> Memory
        let db_memory = memory_to_db(&memory);
        let recovered = db_to_memory(&db_memory).unwrap();

        assert_eq!(memory.id.as_str(), recovered.id.as_str());
        assert_eq!(memory.agent_id(), recovered.agent_id());
        assert_eq!(memory.user_id(), recovered.user_id());
    }

    #[test]
    fn test_legacy_conversion() {
        use agent_mem_traits::{MemoryItem, MemoryType, Session};
        use chrono::Utc;

        let legacy = MemoryItem {
            id: "mem_123".to_string(),
            agent_id: "agent_456".to_string(),
            user_id: Some("user_789".to_string()),
            content: "Legacy content".to_string(),
            hash: Some("hash123".to_string()),
            metadata: HashMap::new(),
            score: Some(0.8),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            memory_type: MemoryType::Semantic,
            importance: 0.7,
            session: Session {
                id: "session_1".to_string(),
                user_id: Some("user_789".to_string()),
                agent_id: Some("agent_456".to_string()),
                run_id: None,
                actor_id: None,
                created_at: Utc::now(),
                metadata: HashMap::new(),
            },
            entities: Vec::new(),
            relations: Vec::new(),
            embedding: None,
            last_accessed_at: Utc::now(),
            access_count: 0,
            expires_at: None,
            version: 1,
        };

        // Legacy -> V4 -> Legacy
        let v4 = legacy_to_v4(&legacy);
        let recovered = v4_to_legacy(&v4);

        assert_eq!(legacy.id, recovered.id);
        assert_eq!(legacy.agent_id, recovered.agent_id);
        assert_eq!(legacy.content, recovered.content);
    }

    #[test]
    fn test_conversion_with_all_fields() {
        let mut memory = Memory {
            id: MemoryId::from_string("test_id_123".to_string()),
            content: Content::text("Comprehensive test content"),
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 42,
                version: 2,
                hash: Some("hash_comprehensive".to_string()),
            },
        };

        // Set all possible attributes
        memory.attributes.insert(
            AttributeKey::core("organization_id"),
            AttributeValue::String("org_comprehensive".to_string()),
        );
        memory.set_user_id("user_comprehensive");
        memory.set_agent_id("agent_comprehensive");
        memory.set_memory_type("semantic");
        memory.attributes.insert(
            AttributeKey::core("scope"),
            AttributeValue::String("global".to_string()),
        );
        memory.attributes.insert(
            AttributeKey::core("level"),
            AttributeValue::String("strategic".to_string()),
        );
        memory.set_importance(0.88);
        memory.set_score(0.92);
        memory.attributes.insert(
            AttributeKey::system("is_deleted"),
            AttributeValue::Boolean(false),
        );
        memory.attributes.insert(
            AttributeKey::system("created_by_id"),
            AttributeValue::String("creator_1".to_string()),
        );
        memory.attributes.insert(
            AttributeKey::system("last_updated_by_id"),
            AttributeValue::String("updater_1".to_string()),
        );

        // Convert to DbMemory and back
        let db_memory = memory_to_db(&memory);
        let recovered = db_to_memory(&db_memory).unwrap();

        // Verify all fields
        assert_eq!(memory.id.as_str(), recovered.id.as_str());
        assert_eq!(memory.organization_id(), recovered.organization_id());
        assert_eq!(memory.user_id(), recovered.user_id());
        assert_eq!(memory.agent_id(), recovered.agent_id());
        assert_eq!(memory.memory_type(), recovered.memory_type());
        assert_eq!(memory.scope(), recovered.scope());
        assert_eq!(memory.level(), recovered.level());

        // Verify numeric fields with tolerance
        assert!((memory.importance().unwrap() - recovered.importance().unwrap()).abs() < 0.001);
        assert!((memory.score().unwrap() - recovered.score().unwrap()).abs() < 0.001);

        // Verify metadata
        assert_eq!(
            memory.metadata.access_count,
            recovered.metadata.access_count
        );
        assert_eq!(memory.metadata.version, recovered.metadata.version);
    }

    #[test]
    fn test_conversion_with_missing_optional_fields() {
        // Create Memory with minimal fields
        let memory = Memory {
            id: MemoryId::new(),
            content: Content::text("Minimal content"),
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
                version: 1,
                hash: None,
            },
        };

        // Convert and verify it doesn't crash
        let db_memory = memory_to_db(&memory);
        let recovered = db_to_memory(&db_memory).unwrap();

        assert_eq!(memory.id.as_str(), recovered.id.as_str());
        assert!(recovered.organization_id().is_none());
        assert!(recovered.user_id().is_none());
        assert!(recovered.agent_id().is_none());
    }

    #[test]
    fn test_batch_conversion() {
        let memories: Vec<Memory> = (0..5)
            .map(|i| {
                let mut mem = Memory {
                    id: MemoryId::from_string(format!("batch_mem_{i}")),
                    content: Content::text(format!("Batch content {i}")),
                    attributes: AttributeSet::new(),
                    relations: RelationGraph::new(),
                    metadata: Metadata {
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        accessed_at: Utc::now(),
                        access_count: i as u32,
                        version: 1,
                        hash: None,
                    },
                };
                mem.set_agent_id(format!("agent_{i}"));
                mem.set_importance(0.5 + (i as f64 * 0.1));
                mem
            })
            .collect();

        // Batch convert to DB and back
        let db_memories = memories_to_db(&memories);
        let recovered = db_to_memories(&db_memories).unwrap();

        assert_eq!(memories.len(), recovered.len());
        for (original, recovered) in memories.iter().zip(recovered.iter()) {
            assert_eq!(original.id.as_str(), recovered.id.as_str());
            assert_eq!(original.agent_id(), recovered.agent_id());
        }
    }

    #[test]
    fn test_structured_content_conversion() {
        use serde_json::json;

        let mut memory = Memory {
            id: MemoryId::new(),
            content: Content::Structured(json!({
                "type": "structured_data",
                "nested": {
                    "key": "value",
                    "number": 42
                }
            })),
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
                version: 1,
                hash: None,
            },
        };

        memory.set_agent_id("agent_structured");

        // Convert and verify structured content is serialized
        let db_memory = memory_to_db(&memory);
        assert!(db_memory.content.contains("structured_data"));
        assert!(db_memory.content.contains("nested"));

        let recovered = db_to_memory(&db_memory).unwrap();
        // Content should be recovered as text (serialized JSON)
        match recovered.content {
            Content::Text(text) => {
                assert!(text.contains("structured_data"));
            }
            _ => panic!("Expected text content after conversion"),
        }
    }
}
