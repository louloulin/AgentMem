//! Memory Export/Import Module

use crate::types::Memory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryExport {
    pub version: String,
    pub timestamp: String,
    pub memories: Vec<MemoryExportItem>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryExportItem {
    pub id: String,
    pub agent_id: String,
    pub user_id: Option<String>,
    pub memory_type: String,
    pub content: String,
    pub importance: f32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryImportResult {
    pub total: usize,
    pub imported: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl MemoryExport {
    pub fn new(memories: Vec<Memory>) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let items = memories
            .into_iter()
            .map(|m| {
                // Extract all values before moving any part
                let id = m.id.clone();
                let agent_id = m.agent_id().to_string();
                let user_id = m.user_id().map(|s| s.to_string());
                let memory_type = m.memory_type().as_str().to_string();
                let importance = m.importance();
                
                let content_str = match m.content {
                    crate::types::Content::Text(s) => s,
                    other => format!("{:?}", other),
                };
                
                MemoryExportItem {
                    id,
                    agent_id,
                    user_id,
                    memory_type,
                    content: content_str,
                    importance,
                    created_at: chrono::Utc::now().to_rfc3339(),
                }
            })
            .collect();

        Self {
            version: "1.0".to_string(),
            timestamp,
            memories: items,
            metadata: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl MemoryImportResult {
    pub fn success(total: usize) -> Self {
        Self {
            total,
            imported: total,
            failed: 0,
            errors: vec![],
        }
    }

    pub fn with_errors(total: usize, errors: Vec<String>) -> Self {
        let failed = errors.len();
        Self {
            total,
            imported: total.saturating_sub(failed),
            failed,
            errors,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;
    use crate::types::MemoryType;

    #[test]
    fn test_export_creation() {
        let memory = Memory::new(
            "agent-1".to_string(),
            Some("user-1".to_string()),
            MemoryType::Semantic,
            "Test content".to_string(),
            0.8,
        );
        
        let export = MemoryExport::new(vec![memory]);
        assert_eq!(export.memories.len(), 1);
        assert_eq!(export.version, "1.0");
    }

    #[test]
    fn test_export_json() {
        let memory = Memory::new(
            "agent-1".to_string(),
            None,
            MemoryType::Core,
            "Important data".to_string(),
            1.0,
        );
        
        let export = MemoryExport::new(vec![memory]);
        let json = export.to_json().unwrap();
        assert!(json.contains("agent-1"));
        assert!(json.contains("Important data"));
    }

    #[test]
    fn test_import_result() {
        let result = MemoryImportResult::success(10);
        assert_eq!(result.imported, 10);
        assert_eq!(result.failed, 0);
        
        let result_with_errors = MemoryImportResult::with_errors(
            10,
            vec!["Error 1".to_string(), "Error 2".to_string()],
        );
        assert_eq!(result_with_errors.imported, 8);
        assert_eq!(result_with_errors.failed, 2);
    }
}
