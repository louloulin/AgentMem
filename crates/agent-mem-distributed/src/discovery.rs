//! Service discovery for distributed AgentMem

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub enabled: bool,
    pub service_name: String,
    pub discovery_interval_seconds: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_name: "agentmem".to_string(),
            discovery_interval_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: Uuid,
    pub name: String,
    pub address: SocketAddr,
    pub metadata: HashMap<String, String>,
}

pub struct ServiceDiscovery {
    config: DiscoveryConfig,
    services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
}

impl ServiceDiscovery {
    pub async fn new(config: DiscoveryConfig) -> Result<Self> {
        Ok(Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Service discovery started");
        Ok(())
    }

    pub async fn register_service(&self, service: ServiceInfo) -> Result<()> {
        let mut services = self.services.write().await;
        services.insert(service.id, service.clone());
        info!("Registered service: {}", service.name);
        Ok(())
    }

    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Service discovery shutdown");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_discovery() {
        let config = DiscoveryConfig::default();
        let discovery = ServiceDiscovery::new(config).await.unwrap();

        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: "test-service".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            metadata: HashMap::new(),
        };

        discovery.register_service(service.clone()).await.unwrap();
        let services = discovery.discover_services().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
    }
}
