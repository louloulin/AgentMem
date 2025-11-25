//! Load balancing for distributed AgentMem

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub enabled: bool,
    pub strategy: LoadBalancingStrategy,
    pub health_check_interval_seconds: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check_interval_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ConsistentHash,
}

pub struct LoadBalancer {
    config: LoadBalancerConfig,
    nodes: Arc<RwLock<Vec<Uuid>>>,
    current_index: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    pub async fn new(config: LoadBalancerConfig) -> Result<Self> {
        Ok(Self {
            config,
            nodes: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn add_node(&self, node_id: Uuid) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.push(node_id);
        info!("Added node {} to load balancer", node_id);
        Ok(())
    }

    pub async fn remove_node(&self, node_id: Uuid) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.retain(|&id| id != node_id);
        info!("Removed node {} from load balancer", node_id);
        Ok(())
    }

    pub async fn select_node(&self) -> Result<Option<Uuid>> {
        let nodes = self.nodes.read().await;
        if nodes.is_empty() {
            return Ok(None);
        }

        match self.config.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut index = self.current_index.write().await;
                let selected = nodes[*index];
                *index = (*index + 1) % nodes.len();
                Ok(Some(selected))
            }
            _ => {
                // Simplified - just return first node for other strategies
                Ok(nodes.first().cloned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balancer() {
        let config = LoadBalancerConfig::default();
        let lb = LoadBalancer::new(config).await.unwrap();

        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();

        lb.add_node(node1).await.unwrap();
        lb.add_node(node2).await.unwrap();

        let selected1 = lb.select_node().await.unwrap();
        let selected2 = lb.select_node().await.unwrap();

        assert!(selected1.is_some());
        assert!(selected2.is_some());
        assert_ne!(selected1, selected2); // Round robin should select different nodes
    }
}
