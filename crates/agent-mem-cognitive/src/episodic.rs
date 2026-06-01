//! Episodic Memory Module
//! 
//! Episodic memories store specific events and experiences with temporal context.



use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Episodic memory event with temporal context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    /// Event ID
    pub event_id: String,
    /// What happened
    pub action: String,
    /// Who was involved
    pub participants: Vec<String>,
    /// When it happened
    pub timestamp: DateTime<Utc>,
    /// Location context (optional)
    pub location: Option<String>,
    /// Emotional valence (-1.0 to 1.0)
    pub emotional_valence: f32,
    /// Outcome result
    pub outcome: Option<String>,
}

impl EpisodicEvent {
    /// Create new episodic event
    pub fn new(event_id: String, action: String) -> Self {
        Self {
            event_id,
            action,
            participants: Vec::new(),
            timestamp: Utc::now(),
            location: None,
            emotional_valence: 0.0,
            outcome: None,
        }
    }

    /// Add participant
    pub fn with_participant(mut self, participant: &str) -> Self {
        self.participants.push(participant.to_string());
        self
    }

    /// Set emotional valence
    pub fn with_emotion(mut self, valence: f32) -> Self {
        self.emotional_valence = valence.clamp(-1.0, 1.0);
        self
    }
}

/// Episodic memory store
pub struct EpisodicMemoryStore {
    events: Vec<EpisodicEvent>,
}

impl EpisodicMemoryStore {
    /// Create new episodic store
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Add event
    pub fn add_event(&mut self, event: EpisodicEvent) {
        self.events.push(event);
    }

    /// Get recent events
    pub fn recent_events(&self, limit: usize) -> Vec<&EpisodicEvent> {
        self.events.iter().rev().take(limit).collect()
    }

    /// Get events by participant
    pub fn events_by_participant(&self, participant: &str) -> Vec<&EpisodicEvent> {
        self.events.iter()
            .filter(|e| e.participants.iter().any(|p| p == participant))
            .collect()
    }
}

impl Default for EpisodicMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episodic_event_creation() {
        let event = EpisodicEvent::new("evt-1".to_string(), "User asked a question".to_string())
            .with_participant("user-1")
            .with_emotion(0.5);
        assert_eq!(event.action, "User asked a question");
        assert_eq!(event.participants.len(), 1);
    }

    #[test]
    fn test_episodic_store() {
        let mut store = EpisodicMemoryStore::new();
        store.add_event(EpisodicEvent::new("1".to_string(), "Event 1".to_string()));
        store.add_event(EpisodicEvent::new("2".to_string(), "Event 2".to_string()));
        assert_eq!(store.events.len(), 2);
    }
}
