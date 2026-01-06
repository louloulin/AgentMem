//! Data sharding for distributed AgentMem
//!
//! This module provides data sharding capabilities including
//! consistent hashing, shard distribution, and rebalancing.

use agent_mem_traits::{AgentMemError, Result};
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Shard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Number of shards
    pub shard_count: u32,
    /// Replication factor
    pub replication_factor: usize,
    /// Virtual nodes per physical node
    pub virtual_nodes: usize,
    /// Sharding strategy
    pub strategy: ShardStrategy,
    /// Enable automatic rebalancing
    pub enable_rebalancing: bool,
    /// Rebalancing threshold (load difference)
    pub rebalancing_threshold: f64,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            shard_count: 256,
            replication_factor: 3,
            virtual_nodes: 100,
            strategy: ShardStrategy::ConsistentHash,
            enable_rebalancing: true,
            rebalancing_threshold: 0.2, // 20% load difference
        }
    }
}

/// Sharding strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardStrategy {
    ConsistentHash,
    RangePartition,
    HashPartition,
    Custom(String),
}

/// Shard key for routing data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ShardKey {
    pub key: String,
    pub namespace: Option<String>,
}

impl ShardKey {
    pub fn new(key: String) -> Self {
        Self {
            key,
            namespace: None,
        }
    }

    pub fn with_namespace(key: String, namespace: String) -> Self {
        Self {
            key,
            namespace: Some(namespace),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut context = Context::new(&SHA256);
        if let Some(ns) = &self.namespace {
            context.update(ns.as_bytes());
            context.update(b":");
        }
        context.update(self.key.as_bytes());
        let digest = context.finish();

        // Convert first 8 bytes to u64
        let bytes = digest.as_ref();
        u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }
}

/// Shard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub id: u32,
    pub primary_node: Uuid,
    pub replica_nodes: Vec<Uuid>,
    pub key_range: Option<(String, String)>,
    pub data_size: u64,
    pub item_count: u64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Simple consistent hash ring implementation
#[derive(Debug, Clone)]
struct HashRing {
    nodes: HashMap<Uuid, u32>, // node_id -> weight
    ring: Vec<(u64, Uuid)>,    // (hash, node_id) sorted by hash
}

impl HashRing {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            ring: Vec::new(),
        }
    }

    fn add_node(&mut self, node_id: Uuid, weight: u32) {
        self.nodes.insert(node_id, weight);
        self.rebuild_ring();
    }

    fn remove_node(&mut self, node_id: Uuid) {
        self.nodes.remove(&node_id);
        self.rebuild_ring();
    }

    fn rebuild_ring(&mut self) {
        self.ring.clear();

        for (&node_id, &weight) in &self.nodes {
            // Create virtual nodes based on weight
            let virtual_count = weight as usize * 100; // 100 virtual nodes per weight unit
            for i in 0..virtual_count {
                let virtual_key = format!("{}:{}", node_id, i);
                let hash = self.hash_key(&virtual_key);
                self.ring.push((hash, node_id));
            }
        }

        self.ring.sort_by_key(|&(hash, _)| hash);
    }

    fn get_node(&self, key: &str) -> Option<Uuid> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = self.hash_key(key);

        // Find first node with hash >= key hash
        match self.ring.binary_search_by_key(&hash, |&(h, _)| h) {
            Ok(index) => Some(self.ring[index].1),
            Err(index) => {
                if index >= self.ring.len() {
                    // Wrap around to first node
                    Some(self.ring[0].1)
                } else {
                    Some(self.ring[index].1)
                }
            }
        }
    }

    fn hash_key(&self, key: &str) -> u64 {
        let mut context = Context::new(&SHA256);
        context.update(key.as_bytes());
        let digest = context.finish();

        let bytes = digest.as_ref();
        u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }
}

/// Shard manager
pub struct ShardManager {
    config: ShardConfig,
    shards: Arc<RwLock<HashMap<u32, Shard>>>,
    hash_ring: Arc<RwLock<HashRing>>,
    node_shards: Arc<RwLock<HashMap<Uuid, HashSet<u32>>>>,
}

impl ShardManager {
    /// Create a new shard manager
    pub async fn new(config: ShardConfig) -> Result<Self> {
        let manager = Self {
            config,
            shards: Arc::new(RwLock::new(HashMap::new())),
            hash_ring: Arc::new(RwLock::new(HashRing::new())),
            node_shards: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize shards
        manager.initialize_shards().await?;

        info!(
            "Shard manager created with {} shards",
            manager.config.shard_count
        );
        Ok(manager)
    }

    /// Initialize shards
    async fn initialize_shards(&self) -> Result<()> {
        let mut shards = self.shards.write().await;

        for shard_id in 0..self.config.shard_count {
            let shard = Shard {
                id: shard_id,
                primary_node: Uuid::nil(), // Will be assigned when nodes join
                replica_nodes: Vec::new(),
                key_range: None,
                data_size: 0,
                item_count: 0,
                last_accessed: chrono::Utc::now(),
            };
            shards.insert(shard_id, shard);
        }

        info!("Initialized {} shards", self.config.shard_count);
        Ok(())
    }

    /// Add a node to the sharding system
    pub async fn add_node(&self, node_id: Uuid, weight: u32) -> Result<()> {
        {
            let mut hash_ring = self.hash_ring.write().await;
            hash_ring.add_node(node_id, weight);
        }

        // Rebalance shards
        self.rebalance_shards().await?;

        info!("Added node {} to sharding system", node_id);
        Ok(())
    }

    /// Remove a node from the sharding system
    pub async fn remove_node(&self, node_id: Uuid) -> Result<()> {
        {
            let mut hash_ring = self.hash_ring.write().await;
            hash_ring.remove_node(node_id);
        }

        // Reassign shards from removed node
        self.reassign_shards_from_node(node_id).await?;

        info!("Removed node {} from sharding system", node_id);
        Ok(())
    }

    /// Get shard ID for a key
    pub async fn get_shard_id(&self, key: &ShardKey) -> Result<u32> {
        match self.config.strategy {
            ShardStrategy::ConsistentHash => {
                let hash = key.hash();
                Ok((hash % self.config.shard_count as u64) as u32)
            }
            ShardStrategy::HashPartition => {
                let hash = key.hash();
                Ok((hash % self.config.shard_count as u64) as u32)
            }
            ShardStrategy::RangePartition => {
                // Simplified range partitioning
                let first_char = key.key.chars().next().unwrap_or('a') as u32;
                Ok(first_char % self.config.shard_count)
            }
            ShardStrategy::Custom(_) => {
                // Custom strategy would be implemented here
                Err(AgentMemError::memory_error(
                    "Custom sharding strategy not implemented",
                ))
            }
        }
    }

    /// Get primary node for a shard
    pub async fn get_primary_node(&self, shard_id: u32) -> Result<Option<Uuid>> {
        let shards = self.shards.read().await;
        if let Some(shard) = shards.get(&shard_id) {
            if shard.primary_node != Uuid::nil() {
                Ok(Some(shard.primary_node))
            } else {
                Ok(None)
            }
        } else {
            Err(AgentMemError::memory_error(&format!(
                "Shard {} not found",
                shard_id
            )))
        }
    }

    /// Get replica nodes for a shard
    pub async fn get_replica_nodes(&self, shard_id: u32) -> Result<Vec<Uuid>> {
        let shards = self.shards.read().await;
        if let Some(shard) = shards.get(&shard_id) {
            Ok(shard.replica_nodes.clone())
        } else {
            Err(AgentMemError::memory_error(&format!(
                "Shard {} not found",
                shard_id
            )))
        }
    }

    /// Get all nodes for a shard (primary + replicas)
    pub async fn get_shard_nodes(&self, shard_id: u32) -> Result<Vec<Uuid>> {
        let shards = self.shards.read().await;
        if let Some(shard) = shards.get(&shard_id) {
            let mut nodes = vec![shard.primary_node];
            nodes.extend(shard.replica_nodes.iter().cloned());
            nodes.retain(|&id| id != Uuid::nil());
            Ok(nodes)
        } else {
            Err(AgentMemError::memory_error(&format!(
                "Shard {} not found",
                shard_id
            )))
        }
    }

    /// Get shards assigned to a node
    pub async fn get_node_shards(&self, node_id: Uuid) -> Result<Vec<u32>> {
        let node_shards = self.node_shards.read().await;
        Ok(node_shards
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect())
    }

    /// Rebalance shards across nodes
    async fn rebalance_shards(&self) -> Result<()> {
        if !self.config.enable_rebalancing {
            return Ok(());
        }

        let hash_ring = self.hash_ring.read().await;
        let mut shards = self.shards.write().await;
        let mut node_shards = self.node_shards.write().await;

        // Clear current assignments
        node_shards.clear();

        // Reassign shards using consistent hashing
        for (shard_id, shard) in shards.iter_mut() {
            let shard_key = format!("shard_{}", shard_id);

            if let Some(primary_node_id) = hash_ring.get_node(&shard_key) {
                shard.primary_node = primary_node_id;

                // Add to node_shards mapping
                node_shards
                    .entry(primary_node_id)
                    .or_insert_with(HashSet::new)
                    .insert(*shard_id);

                // Assign replicas (simplified - would use more sophisticated logic)
                shard.replica_nodes.clear();
                for i in 1..=self.config.replication_factor {
                    let replica_key = format!("shard_{}_replica_{}", shard_id, i);
                    if let Some(replica_node_id) = hash_ring.get_node(&replica_key) {
                        if replica_node_id != primary_node_id {
                            shard.replica_nodes.push(replica_node_id);
                            node_shards
                                .entry(replica_node_id)
                                .or_insert_with(HashSet::new)
                                .insert(*shard_id);
                        }
                    }
                }
            }
        }

        info!("Rebalanced {} shards across nodes", shards.len());
        Ok(())
    }

    /// Reassign shards from a removed node
    async fn reassign_shards_from_node(&self, removed_node_id: Uuid) -> Result<()> {
        let mut shards = self.shards.write().await;
        let mut node_shards = self.node_shards.write().await;

        // Remove node from mappings
        node_shards.remove(&removed_node_id);

        // Find shards that need reassignment
        let mut shards_to_reassign = Vec::new();
        for (shard_id, shard) in shards.iter_mut() {
            if shard.primary_node == removed_node_id {
                shards_to_reassign.push(*shard_id);
            }
            shard.replica_nodes.retain(|&id| id != removed_node_id);
        }

        // Reassign shards (simplified - would use consistent hashing)
        for shard_id in shards_to_reassign {
            if let Some(shard) = shards.get_mut(&shard_id) {
                // Promote first replica to primary if available
                if let Some(new_primary) = shard.replica_nodes.first().cloned() {
                    shard.primary_node = new_primary;
                    shard.replica_nodes.remove(0);

                    node_shards
                        .entry(new_primary)
                        .or_insert_with(HashSet::new)
                        .insert(shard_id);
                } else {
                    // No replicas available - shard becomes unavailable
                    shard.primary_node = Uuid::nil();
                    warn!("Shard {} has no available nodes", shard_id);
                }
            }
        }

        info!("Reassigned shards from removed node {}", removed_node_id);
        Ok(())
    }

    /// Get shard information
    pub async fn get_shard_info(&self) -> Result<super::ShardInfo> {
        let shards = self.shards.read().await;

        let local_shards = vec![]; // Would be populated with actual local shards
        let mut shard_distribution = HashMap::new();

        for (shard_id, shard) in shards.iter() {
            let mut nodes = vec![shard.primary_node];
            nodes.extend(shard.replica_nodes.iter().cloned());
            nodes.retain(|&id| id != Uuid::nil());
            shard_distribution.insert(*shard_id, nodes);
        }

        Ok(super::ShardInfo {
            total_shards: shards.len(),
            local_shards,
            shard_distribution,
        })
    }

    /// Update shard statistics
    pub async fn update_shard_stats(
        &self,
        shard_id: u32,
        data_size: u64,
        item_count: u64,
    ) -> Result<()> {
        let mut shards = self.shards.write().await;
        if let Some(shard) = shards.get_mut(&shard_id) {
            shard.data_size = data_size;
            shard.item_count = item_count;
            shard.last_accessed = chrono::Utc::now();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shard_manager_creation() {
        let config = ShardConfig::default();
        let manager = ShardManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_shard_key_hashing() {
        let key1 = ShardKey::new("test_key".to_string());
        let key2 = ShardKey::new("test_key".to_string());
        let key3 = ShardKey::new("different_key".to_string());

        assert_eq!(key1.hash(), key2.hash());
        assert_ne!(key1.hash(), key3.hash());
    }

    #[tokio::test]
    async fn test_shard_assignment() {
        let config = ShardConfig::default();
        let manager = ShardManager::new(config).await.unwrap();

        let key = ShardKey::new("test_key".to_string());
        let shard_id = manager.get_shard_id(&key).await.unwrap();

        assert!(shard_id < 256); // Default shard count
    }

    #[tokio::test]
    async fn test_node_management() {
        let config = ShardConfig::default();
        let manager = ShardManager::new(config).await.unwrap();

        let node_id = Uuid::new_v4();
        manager.add_node(node_id, 100).await.unwrap();

        let _node_shards = manager.get_node_shards(node_id).await.unwrap();
        // After rebalancing, node should have some shards assigned
        // (exact count depends on consistent hashing implementation)
    }
}
