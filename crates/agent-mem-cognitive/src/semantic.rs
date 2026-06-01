//! Semantic Memory Module
//! 
//! Semantic memories store facts, concepts, and general knowledge.


use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Semantic concept with facts and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConcept {
    /// Concept ID
    pub id: String,
    /// Concept name
    pub name: String,
    /// Definition
    pub definition: String,
    /// Related concepts
    pub related: Vec<String>,
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
}

impl SemanticConcept {
    /// Create new semantic concept
    pub fn new(id: String, name: String, definition: String) -> Self {
        Self {
            id,
            name,
            definition,
            related: Vec::new(),
            attributes: HashMap::new(),
            confidence: 1.0,
        }
    }

    /// Add related concept
    pub fn add_relation(&mut self, related_id: &str) {
        if !self.related.contains(&related_id.to_string()) {
            self.related.push(related_id.to_string());
        }
    }

    /// Add attribute
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }
}

/// Semantic knowledge graph
pub struct SemanticKnowledgeGraph {
    concepts: HashMap<String, SemanticConcept>,
}

impl SemanticKnowledgeGraph {
    /// Create new knowledge graph
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
        }
    }

    /// Add concept
    pub fn add_concept(&mut self, concept: SemanticConcept) {
        self.concepts.insert(concept.id.clone(), concept);
    }

    /// Get concept by ID
    pub fn get(&self, id: &str) -> Option<&SemanticConcept> {
        self.concepts.get(id)
    }

    /// Get related concepts
    pub fn get_related(&self, id: &str) -> Vec<&SemanticConcept> {
        self.concepts.get(id)
            .map(|c| c.related.iter()
                .filter_map(|rid| self.concepts.get(rid))
                .collect())
            .unwrap_or_default()
    }

    /// Concept count
    pub fn len(&self) -> usize {
        self.concepts.len()
    }
}

impl Default for SemanticKnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_concept_creation() {
        let mut concept = SemanticConcept::new(
            "concept-1".to_string(),
            "Rust".to_string(),
            "Systems programming language".to_string(),
        );
        concept.add_relation("concept-2");
        concept.add_attribute("paradigm", "multi-paradigm");
        assert_eq!(concept.related.len(), 1);
        assert!(concept.attributes.contains_key("paradigm"));
    }

    #[test]
    fn test_knowledge_graph() {
        let mut graph = SemanticKnowledgeGraph::new();
        graph.add_concept(SemanticConcept::new("1".to_string(), "A".to_string(), "A is B".to_string()));
        graph.add_concept(SemanticConcept::new("2".to_string(), "B".to_string(), "B is C".to_string()));
        assert_eq!(graph.len(), 2);
    }
}
