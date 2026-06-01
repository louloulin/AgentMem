//! Knowledge Memory Module
//! 
//! Knowledge memories store structured knowledge graphs and domain expertise.

use std::collections::{HashMap};
use serde::{Deserialize, Serialize};

/// Knowledge graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub weight: f32,
}

/// Knowledge node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: HashMap<String, String>,
}

impl KnowledgeNode {
    pub fn new(id: String, label: String, node_type: &str) -> Self {
        Self {
            id,
            label,
            node_type: node_type.to_string(),
            properties: HashMap::new(),
        }
    }
}

/// Knowledge graph
pub struct KnowledgeGraph {
    nodes: HashMap<String, KnowledgeNode>,
    edges: Vec<KnowledgeEdge>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: KnowledgeNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: KnowledgeEdge) {
        self.edges.push(edge);
    }

    pub fn get_node(&self, id: &str) -> Option<&KnowledgeNode> {
        self.nodes.get(id)
    }

    pub fn get_neighbors(&self, id: &str) -> Vec<&KnowledgeNode> {
        self.edges.iter()
            .filter(|e| e.from == id)
            .filter_map(|e| self.nodes.get(&e.to))
            .collect()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_graph() {
        let mut kg = KnowledgeGraph::new();
        kg.add_node(KnowledgeNode::new("n1".to_string(), "Rust".to_string(), "language"));
        kg.add_node(KnowledgeNode::new("n2".to_string(), "Systems".to_string(), "domain"));
        kg.add_edge(KnowledgeEdge {
            from: "n1".to_string(),
            to: "n2".to_string(),
            relation: "used_for".to_string(),
            weight: 1.0,
        });
        assert_eq!(kg.node_count(), 2);
    }
}
