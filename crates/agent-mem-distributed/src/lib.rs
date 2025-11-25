//! AgentMem Distributed Computing Module
//!
//! This module provides distributed computing capabilities including:
//! - Cluster mode support
//! - Data sharding strategies
//! - Consistent hashing
//! - Failure detection and recovery
//! - Load balancing

pub mod cluster;
pub mod consensus;
pub mod coordination;
pub mod discovery;
pub mod load_balancer;
pub mod replication;
pub mod sharding;

// Re-export main types
pub use cluster::{ClusterConfig, ClusterManager, ClusterNode, NodeStatus};
pub use consensus::{ConsensusConfig, ConsensusManager, ConsensusState};
pub use coordination::{CoordinationConfig, CoordinationManager};
pub use discovery::{DiscoveryConfig, ServiceDiscovery, ServiceInfo};
pub use load_balancer::{LoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};
pub use replication::{ReplicationConfig, ReplicationManager, ReplicationStrategy};
pub use sharding::{ShardConfig, ShardKey, ShardManager, ShardStrategy};

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Distributed system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Cluster configuration
    pub cluster: ClusterConfig,
    /// Sharding configuration
    pub sharding: ShardConfig,
    /// Consensus configuration
    pub consensus: ConsensusConfig,
    /// Service discovery configuration
    pub discovery: DiscoveryConfig,
    /// Coordination configuration
    pub coordination: CoordinationConfig,
    /// Replication configuration
    pub replication: ReplicationConfig,
    /// Load balancer configuration
    pub load_balancer: LoadBalancerConfig,
    /// Enable distributed mode
    pub enable_distributed: bool,
    /// Enable high availability
    pub enable_ha: bool,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            cluster: ClusterConfig::default(),
            sharding: ShardConfig::default(),
            consensus: ConsensusConfig::default(),
            discovery: DiscoveryConfig::default(),
            coordination: CoordinationConfig::default(),
            replication: ReplicationConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
            enable_distributed: false,
            enable_ha: false,
        }
    }
}

/// Main distributed system manager
pub struct DistributedManager {
    config: DistributedConfig,
    cluster_manager: Arc<ClusterManager>,
    shard_manager: Arc<ShardManager>,
    consensus_manager: Arc<ConsensusManager>,
    service_discovery: Arc<ServiceDiscovery>,
    coordination_manager: Arc<CoordinationManager>,
    replication_manager: Arc<ReplicationManager>,
    load_balancer: Arc<LoadBalancer>,
    node_id: Uuid,
    is_leader: Arc<RwLock<bool>>,
}

impl DistributedManager {
    /// Create a new distributed manager
    pub async fn new(config: DistributedConfig) -> Result<Self> {
        let node_id = Uuid::new_v4();

        let cluster_manager = Arc::new(ClusterManager::new(config.cluster.clone(), node_id).await?);
        let shard_manager = Arc::new(ShardManager::new(config.sharding.clone()).await?);
        let consensus_manager =
            Arc::new(ConsensusManager::new(config.consensus.clone(), node_id).await?);
        let service_discovery = Arc::new(ServiceDiscovery::new(config.discovery.clone()).await?);
        let coordination_manager =
            Arc::new(CoordinationManager::new(config.coordination.clone()).await?);
        let replication_manager =
            Arc::new(ReplicationManager::new(config.replication.clone()).await?);
        let load_balancer = Arc::new(LoadBalancer::new(config.load_balancer.clone()).await?);

        let manager = Self {
            config,
            cluster_manager,
            shard_manager,
            consensus_manager,
            service_discovery,
            coordination_manager,
            replication_manager,
            load_balancer,
            node_id,
            is_leader: Arc::new(RwLock::new(false)),
        };

        if manager.config.enable_distributed {
            manager.start_distributed_services().await?;
        }

        Ok(manager)
    }

    /// Start distributed services
    async fn start_distributed_services(&self) -> Result<()> {
        // Start cluster management
        self.cluster_manager.start().await?;

        // Start service discovery
        self.service_discovery.start().await?;

        // Start consensus if enabled
        if self.config.enable_ha {
            self.consensus_manager.start().await?;
        }

        // Start coordination
        self.coordination_manager.start().await?;

        // Start replication
        self.replication_manager.start().await?;

        tracing::info!("Distributed services started for node {}", self.node_id);
        Ok(())
    }

    /// Get cluster manager
    pub fn cluster_manager(&self) -> Arc<ClusterManager> {
        Arc::clone(&self.cluster_manager)
    }

    /// Get shard manager
    pub fn shard_manager(&self) -> Arc<ShardManager> {
        Arc::clone(&self.shard_manager)
    }

    /// Get consensus manager
    pub fn consensus_manager(&self) -> Arc<ConsensusManager> {
        Arc::clone(&self.consensus_manager)
    }

    /// Get service discovery
    pub fn service_discovery(&self) -> Arc<ServiceDiscovery> {
        Arc::clone(&self.service_discovery)
    }

    /// Get coordination manager
    pub fn coordination_manager(&self) -> Arc<CoordinationManager> {
        Arc::clone(&self.coordination_manager)
    }

    /// Get replication manager
    pub fn replication_manager(&self) -> Arc<ReplicationManager> {
        Arc::clone(&self.replication_manager)
    }

    /// Get load balancer
    pub fn load_balancer(&self) -> Arc<LoadBalancer> {
        Arc::clone(&self.load_balancer)
    }

    /// Get node ID
    pub fn node_id(&self) -> Uuid {
        self.node_id
    }

    /// Check if this node is the leader
    pub async fn is_leader(&self) -> bool {
        *self.is_leader.read().await
    }

    /// Get cluster status
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        let cluster_info = self.cluster_manager.get_cluster_info().await?;
        let shard_info = self.shard_manager.get_shard_info().await?;
        let consensus_state = if self.config.enable_ha {
            Some(self.consensus_manager.get_state().await?)
        } else {
            None
        };

        Ok(ClusterStatus {
            node_id: self.node_id,
            is_leader: self.is_leader().await,
            cluster_info,
            shard_info,
            consensus_state,
        })
    }

    /// Shutdown distributed services
    pub async fn shutdown(&self) -> Result<()> {
        self.replication_manager.shutdown().await?;
        self.coordination_manager.shutdown().await?;

        if self.config.enable_ha {
            self.consensus_manager.shutdown().await?;
        }

        self.service_discovery.shutdown().await?;
        self.cluster_manager.shutdown().await?;

        tracing::info!("Distributed services shutdown for node {}", self.node_id);
        Ok(())
    }
}

/// Cluster status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub node_id: Uuid,
    pub is_leader: bool,
    pub cluster_info: ClusterInfo,
    pub shard_info: ShardInfo,
    pub consensus_state: Option<ConsensusState>,
}

/// Cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub leader_node: Option<Uuid>,
    pub nodes: HashMap<Uuid, ClusterNode>,
}

/// Shard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub total_shards: usize,
    pub local_shards: Vec<u32>,
    pub shard_distribution: HashMap<u32, Vec<Uuid>>,
}

/// Distributed operation trait
#[async_trait::async_trait]
pub trait DistributedOperation: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;

    /// Execute operation across the cluster
    async fn execute_distributed(
        &self,
        input: Self::Input,
        manager: &DistributedManager,
    ) -> Result<Self::Output>;

    /// Get required shards for this operation
    fn get_required_shards(&self, input: &Self::Input) -> Vec<u32>;

    /// Check if operation requires consensus
    fn requires_consensus(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_manager_creation() {
        let config = DistributedConfig::default();
        let manager = DistributedManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_distributed_manager_node_id() {
        let config = DistributedConfig::default();
        let manager = DistributedManager::new(config).await.unwrap();
        let node_id = manager.node_id();
        assert_ne!(node_id, Uuid::nil());
    }

    #[tokio::test]
    async fn test_distributed_manager_shutdown() {
        let config = DistributedConfig::default();
        let manager = DistributedManager::new(config).await.unwrap();
        let result = manager.shutdown().await;
        assert!(result.is_ok());
    }
}
