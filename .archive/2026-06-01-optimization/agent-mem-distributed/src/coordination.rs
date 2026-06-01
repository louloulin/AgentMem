//! Coordination services for distributed AgentMem

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationConfig {
    pub enabled: bool,
    pub coordination_timeout_seconds: u64,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            coordination_timeout_seconds: 30,
        }
    }
}

pub struct CoordinationManager {
    config: CoordinationConfig,
    locks: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl CoordinationManager {
    pub async fn new(config: CoordinationConfig) -> Result<Self> {
        Ok(Self {
            config,
            locks: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Coordination manager started");
        Ok(())
    }

    pub async fn acquire_lock(&self, key: &str, node_id: Uuid) -> Result<bool> {
        let mut locks = self.locks.write().await;
        if locks.contains_key(key) {
            Ok(false)
        } else {
            locks.insert(key.to_string(), node_id);
            Ok(true)
        }
    }

    pub async fn release_lock(&self, key: &str, node_id: Uuid) -> Result<bool> {
        let mut locks = self.locks.write().await;
        if let Some(&lock_holder) = locks.get(key) {
            if lock_holder == node_id {
                locks.remove(key);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Coordination manager shutdown");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordination_manager() {
        let config = CoordinationConfig::default();
        let manager = CoordinationManager::new(config).await.unwrap();

        let node_id = Uuid::new_v4();
        let acquired = manager.acquire_lock("test_key", node_id).await.unwrap();
        assert!(acquired);

        let released = manager.release_lock("test_key", node_id).await.unwrap();
        assert!(released);
    }
}
