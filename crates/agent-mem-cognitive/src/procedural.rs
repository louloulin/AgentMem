//! Procedural Memory Module
//! 
//! Procedural memories store skills, procedures, and how-to knowledge.

use serde::{Deserialize, Serialize};


/// Step in a procedure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    /// Step number
    pub step: usize,
    /// Description
    pub description: String,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Postconditions
    pub postconditions: Vec<String>,
}

/// Procedural knowledge (skill or procedure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    /// Procedure ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Steps
    pub steps: Vec<ProcedureStep>,
    /// Prerequisites
    pub prerequisites: Vec<String>,
    /// Practice count (reinforcement)
    pub practice_count: usize,
    /// Success rate
    pub success_rate: f32,
}

impl Procedure {
    /// Create new procedure
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            steps: Vec::new(),
            prerequisites: Vec::new(),
            practice_count: 0,
            success_rate: 0.5,
        }
    }

    /// Add step
    pub fn add_step(&mut self, step: ProcedureStep) {
        self.steps.push(step);
    }

    /// Record practice
    pub fn record_practice(&mut self, success: bool) {
        self.practice_count += 1;
        let total = self.practice_count as f32;
        let current = self.success_rate * (total - 1.0);
        self.success_rate = if success { (current + 1.0) / total } else { current / total };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedure_creation() {
        let mut proc = Procedure::new("proc-1".to_string(), "Make Coffee".to_string(), "How to make coffee".to_string());
        proc.add_step(ProcedureStep {
            step: 1,
            description: "Boil water".to_string(),
            preconditions: vec![],
            postconditions: vec!["Water hot".to_string()],
        });
        assert_eq!(proc.steps.len(), 1);
    }
}
