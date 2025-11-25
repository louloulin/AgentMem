//! Consensus management for distributed AgentMem
//!
//! This module provides consensus capabilities for distributed coordination
//! and leader election.

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, info};
use uuid::Uuid;

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Election timeout (milliseconds)
    pub election_timeout_ms: u64,
    /// Heartbeat interval (milliseconds)
    pub heartbeat_interval_ms: u64,
    /// Maximum log entries per append
    pub max_log_entries: usize,
    /// Enable consensus
    pub enabled: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            election_timeout_ms: 5000,
            heartbeat_interval_ms: 1000,
            max_log_entries: 100,
            enabled: false,
        }
    }
}

/// Consensus state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusState {
    Follower,
    Candidate,
    Leader,
}

/// Log entry for consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Consensus manager (simplified Raft-like implementation)
pub struct ConsensusManager {
    config: ConsensusConfig,
    node_id: Uuid,
    state: Arc<RwLock<ConsensusState>>,
    current_term: Arc<RwLock<u64>>,
    voted_for: Arc<RwLock<Option<Uuid>>>,
    log: Arc<RwLock<Vec<LogEntry>>>,
    commit_index: Arc<RwLock<u64>>,
    last_applied: Arc<RwLock<u64>>,
    peers: Arc<RwLock<HashMap<Uuid, PeerInfo>>>,
    is_running: Arc<RwLock<bool>>,
}

/// Peer information
#[derive(Debug, Clone)]
struct PeerInfo {
    node_id: Uuid,
    next_index: u64,
    match_index: u64,
    last_heartbeat: Instant,
}

impl ConsensusManager {
    /// Create a new consensus manager
    pub async fn new(config: ConsensusConfig, node_id: Uuid) -> Result<Self> {
        let manager = Self {
            config,
            node_id,
            state: Arc::new(RwLock::new(ConsensusState::Follower)),
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            log: Arc::new(RwLock::new(Vec::new())),
            commit_index: Arc::new(RwLock::new(0)),
            last_applied: Arc::new(RwLock::new(0)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        };

        info!("Consensus manager created for node {}", node_id);
        Ok(manager)
    }

    /// Start consensus services
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Consensus is disabled");
            return Ok(());
        }

        *self.is_running.write().await = true;

        // Start election timer
        self.start_election_timer().await;

        // Start heartbeat timer (if leader)
        self.start_heartbeat_timer().await;

        info!("Consensus services started for node {}", self.node_id);
        Ok(())
    }

    /// Start election timer
    async fn start_election_timer(&self) {
        let state = Arc::clone(&self.state);
        let current_term = Arc::clone(&self.current_term);
        let voted_for = Arc::clone(&self.voted_for);
        let is_running = Arc::clone(&self.is_running);
        let node_id = self.node_id;
        let election_timeout = Duration::from_millis(self.config.election_timeout_ms);

        tokio::spawn(async move {
            while *is_running.read().await {
                sleep(election_timeout).await;

                let current_state = state.read().await.clone();
                if current_state == ConsensusState::Follower
                    || current_state == ConsensusState::Candidate
                {
                    // Start election
                    info!("Node {} starting election", node_id);

                    *state.write().await = ConsensusState::Candidate;
                    *current_term.write().await += 1;
                    *voted_for.write().await = Some(node_id);

                    // In a real implementation, would send vote requests to peers
                    // For simplicity, assume we become leader if no other leader exists
                    sleep(Duration::from_millis(100)).await;
                    *state.write().await = ConsensusState::Leader;
                    info!("Node {} became leader", node_id);
                }
            }
        });
    }

    /// Start heartbeat timer
    async fn start_heartbeat_timer(&self) {
        let state = Arc::clone(&self.state);
        let is_running = Arc::clone(&self.is_running);
        let node_id = self.node_id;
        let heartbeat_interval = Duration::from_millis(self.config.heartbeat_interval_ms);

        tokio::spawn(async move {
            let mut interval = interval(heartbeat_interval);

            while *is_running.read().await {
                interval.tick().await;

                let current_state = state.read().await.clone();
                if current_state == ConsensusState::Leader {
                    // Send heartbeats to followers
                    debug!("Node {} sending heartbeats", node_id);
                    // In a real implementation, would send append entries to peers
                }
            }
        });
    }

    /// Add a peer to the consensus group
    pub async fn add_peer(&self, peer_id: Uuid) -> Result<()> {
        let mut peers = self.peers.write().await;
        let peer_info = PeerInfo {
            node_id: peer_id,
            next_index: 1,
            match_index: 0,
            last_heartbeat: Instant::now(),
        };
        peers.insert(peer_id, peer_info);

        info!("Added peer {} to consensus group", peer_id);
        Ok(())
    }

    /// Remove a peer from the consensus group
    pub async fn remove_peer(&self, peer_id: Uuid) -> Result<()> {
        let mut peers = self.peers.write().await;
        peers.remove(&peer_id);

        info!("Removed peer {} from consensus group", peer_id);
        Ok(())
    }

    /// Append a log entry (leader only)
    pub async fn append_entry(&self, command: String) -> Result<u64> {
        let state = self.state.read().await.clone();
        if state != ConsensusState::Leader {
            return Err(AgentMemError::memory_error(
                "Only leader can append entries",
            ));
        }

        let mut log = self.log.write().await;
        let current_term = *self.current_term.read().await;
        let index = log.len() as u64 + 1;

        let entry = LogEntry {
            term: current_term,
            index,
            command,
            timestamp: chrono::Utc::now(),
        };

        log.push(entry);
        info!("Appended log entry at index {}", index);

        // In a real implementation, would replicate to followers
        Ok(index)
    }

    /// Get current state
    pub async fn get_state(&self) -> Result<ConsensusState> {
        Ok(self.state.read().await.clone())
    }

    /// Check if this node is the leader
    pub async fn is_leader(&self) -> bool {
        *self.state.read().await == ConsensusState::Leader
    }

    /// Get current term
    pub async fn get_current_term(&self) -> u64 {
        *self.current_term.read().await
    }

    /// Get log length
    pub async fn get_log_length(&self) -> usize {
        self.log.read().await.len()
    }

    /// Get commit index
    pub async fn get_commit_index(&self) -> u64 {
        *self.commit_index.read().await
    }

    /// Shutdown consensus services
    pub async fn shutdown(&self) -> Result<()> {
        *self.is_running.write().await = false;
        *self.state.write().await = ConsensusState::Follower;

        info!("Consensus services shutdown for node {}", self.node_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_manager_creation() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = ConsensusManager::new(config, node_id).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_consensus_state() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = ConsensusManager::new(config, node_id).await.unwrap();

        let state = manager.get_state().await.unwrap();
        assert_eq!(state, ConsensusState::Follower);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = ConsensusManager::new(config, node_id).await.unwrap();

        let peer_id = Uuid::new_v4();
        manager.add_peer(peer_id).await.unwrap();

        let peers = manager.peers.read().await;
        assert!(peers.contains_key(&peer_id));
    }

    #[tokio::test]
    async fn test_log_append() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = ConsensusManager::new(config, node_id).await.unwrap();

        // Set as leader to allow log append
        *manager.state.write().await = ConsensusState::Leader;

        let result = manager.append_entry("test command".to_string()).await;
        assert!(result.is_ok());

        let log_length = manager.get_log_length().await;
        assert_eq!(log_length, 1);
    }
}
