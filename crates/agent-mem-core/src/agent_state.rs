//! Agent State Management
//!
//! This module implements the agent state machine and lifecycle management,
//! inspired by MIRIX's agent state management but optimized for Rust.
//!
//! ## State Machine
//!
//! ```text
//! Idle -> Thinking -> Executing -> Idle
//!   |         |           |
//!   |         v           v
//!   +----> Waiting <------+
//!   |                     |
//!   +----> Error <--------+
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Agent state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AgentState {
    /// Agent is idle and ready to receive messages
    #[default]
    Idle,
    /// Agent is thinking/processing a message
    Thinking,
    /// Agent is executing tool calls
    Executing,
    /// Agent is waiting for external input or tool results
    Waiting,
    /// Agent encountered an error
    Error,
}

impl AgentState {
    /// Convert state to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentState::Idle => "idle",
            AgentState::Thinking => "thinking",
            AgentState::Executing => "executing",
            AgentState::Waiting => "waiting",
            AgentState::Error => "error",
        }
    }

    /// Parse state from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "idle" => Some(AgentState::Idle),
            "thinking" => Some(AgentState::Thinking),
            "executing" => Some(AgentState::Executing),
            "waiting" => Some(AgentState::Waiting),
            "error" => Some(AgentState::Error),
            _ => None,
        }
    }

    /// Check if the state is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, AgentState::Idle | AgentState::Error)
    }

    /// Check if the state is an active state
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            AgentState::Thinking | AgentState::Executing | AgentState::Waiting
        )
    }
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Agent state machine
///
/// Manages state transitions and validates state changes according to the state machine rules.
#[derive(Debug, Clone)]
pub struct AgentStateMachine {
    /// Current state
    current_state: AgentState,
    /// Agent ID
    agent_id: String,
    /// Last state change timestamp
    last_state_change: DateTime<Utc>,
    /// Error message (if in Error state)
    error_message: Option<String>,
}

impl AgentStateMachine {
    /// Create a new state machine for an agent
    pub fn new(agent_id: String) -> Self {
        Self {
            current_state: AgentState::Idle,
            agent_id,
            last_state_change: Utc::now(),
            error_message: None,
        }
    }

    /// Create a state machine with an initial state
    pub fn with_state(agent_id: String, state: AgentState) -> Self {
        Self {
            current_state: state,
            agent_id,
            last_state_change: Utc::now(),
            error_message: None,
        }
    }

    /// Get the current state
    pub fn current_state(&self) -> &AgentState {
        &self.current_state
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get the last state change timestamp
    pub fn last_state_change(&self) -> DateTime<Utc> {
        self.last_state_change
    }

    /// Get the error message (if in Error state)
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Transition to a new state
    ///
    /// Returns `Ok(())` if the transition is valid, `Err(String)` otherwise.
    pub fn transition(&mut self, new_state: AgentState) -> Result<(), String> {
        // Validate the transition
        if !self.is_valid_transition(&new_state) {
            return Err(format!(
                "Invalid state transition from {} to {}",
                self.current_state, new_state
            ));
        }

        // Clear error message if transitioning out of Error state
        if self.current_state == AgentState::Error && new_state != AgentState::Error {
            self.error_message = None;
        }

        // Update state
        self.current_state = new_state;
        self.last_state_change = Utc::now();

        Ok(())
    }

    /// Transition to Error state with an error message
    pub fn transition_to_error(&mut self, error_message: String) -> Result<(), String> {
        self.error_message = Some(error_message);
        self.current_state = AgentState::Error;
        self.last_state_change = Utc::now();
        Ok(())
    }

    /// Check if a transition is valid
    fn is_valid_transition(&self, new_state: &AgentState) -> bool {
        use AgentState::*;

        match (&self.current_state, new_state) {
            // From Idle
            (Idle, Thinking) => true,
            (Idle, Error) => true,

            // From Thinking
            (Thinking, Executing) => true,
            (Thinking, Idle) => true,
            (Thinking, Error) => true,

            // From Executing
            (Executing, Idle) => true,
            (Executing, Waiting) => true,
            (Executing, Error) => true,

            // From Waiting
            (Waiting, Executing) => true,
            (Waiting, Idle) => true,
            (Waiting, Error) => true,

            // From Error
            (Error, Idle) => true,

            // Same state (no-op)
            (current, new) if current == new => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Reset the state machine to Idle
    pub fn reset(&mut self) {
        self.current_state = AgentState::Idle;
        self.last_state_change = Utc::now();
        self.error_message = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_string_conversion() {
        assert_eq!(AgentState::Idle.as_str(), "idle");
        assert_eq!(AgentState::Thinking.as_str(), "thinking");
        assert_eq!(AgentState::Executing.as_str(), "executing");
        assert_eq!(AgentState::Waiting.as_str(), "waiting");
        assert_eq!(AgentState::Error.as_str(), "error");

        assert_eq!(AgentState::from_str("idle"), Some(AgentState::Idle));
        assert_eq!(AgentState::from_str("thinking"), Some(AgentState::Thinking));
        assert_eq!(
            AgentState::from_str("executing"),
            Some(AgentState::Executing)
        );
        assert_eq!(AgentState::from_str("waiting"), Some(AgentState::Waiting));
        assert_eq!(AgentState::from_str("error"), Some(AgentState::Error));
        assert_eq!(AgentState::from_str("invalid"), None);
    }

    #[test]
    fn test_agent_state_properties() {
        assert!(AgentState::Idle.is_terminal());
        assert!(!AgentState::Idle.is_active());

        assert!(!AgentState::Thinking.is_terminal());
        assert!(AgentState::Thinking.is_active());

        assert!(AgentState::Error.is_terminal());
        assert!(!AgentState::Error.is_active());
    }

    #[test]
    fn test_state_machine_creation() {
        let sm = AgentStateMachine::new("agent-1".to_string());
        assert_eq!(sm.current_state(), &AgentState::Idle);
        assert_eq!(sm.agent_id(), "agent-1");
        assert!(sm.error_message().is_none());
    }

    #[test]
    fn test_valid_transitions() {
        let mut sm = AgentStateMachine::new("agent-1".to_string());

        // Idle -> Thinking
        assert!(sm.transition(AgentState::Thinking).is_ok());
        assert_eq!(sm.current_state(), &AgentState::Thinking);

        // Thinking -> Executing
        assert!(sm.transition(AgentState::Executing).is_ok());
        assert_eq!(sm.current_state(), &AgentState::Executing);

        // Executing -> Idle
        assert!(sm.transition(AgentState::Idle).is_ok());
        assert_eq!(sm.current_state(), &AgentState::Idle);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut sm = AgentStateMachine::new("agent-1".to_string());

        // Idle -> Executing (invalid)
        assert!(sm.transition(AgentState::Executing).is_err());

        // Idle -> Waiting (invalid)
        assert!(sm.transition(AgentState::Waiting).is_err());
    }

    #[test]
    fn test_error_state_transition() {
        let mut sm = AgentStateMachine::new("agent-1".to_string());

        // Transition to error
        assert!(sm.transition_to_error("Test error".to_string()).is_ok());
        assert_eq!(sm.current_state(), &AgentState::Error);
        assert_eq!(sm.error_message(), Some("Test error"));

        // Error -> Idle
        assert!(sm.transition(AgentState::Idle).is_ok());
        assert_eq!(sm.current_state(), &AgentState::Idle);
        assert!(sm.error_message().is_none());
    }

    #[test]
    fn test_reset() {
        let mut sm = AgentStateMachine::new("agent-1".to_string());

        // Transition to some state
        sm.transition(AgentState::Thinking).unwrap();
        sm.transition(AgentState::Executing).unwrap();

        // Reset
        sm.reset();
        assert_eq!(sm.current_state(), &AgentState::Idle);
        assert!(sm.error_message().is_none());
    }
}
