//! 图存储工厂实现

// use crate::graph::{MemgraphStore, Neo4jStore};
use agent_mem_config::memory::GraphStoreConfig;
use agent_mem_traits::{AgentMemError, GraphStore, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// 图存储提供商枚举
pub enum GraphStoreEnum {
    #[cfg(feature = "neo4j")]
    Neo4j(Neo4jStore),
    #[cfg(feature = "memgraph")]
    Memgraph(MemgraphStore),
    #[cfg(not(any(feature = "neo4j", feature = "memgraph")))]
    _Phantom(std::marker::PhantomData<()>),
}

#[async_trait]
impl GraphStore for GraphStoreEnum {
    async fn add_entities(
        &self,
        entities: &[agent_mem_traits::Entity],
        session: &agent_mem_traits::Session,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "neo4j")]
            GraphStoreEnum::Neo4j(store) => store.add_entities(entities, session).await,
            #[cfg(feature = "memgraph")]
            GraphStoreEnum::Memgraph(store) => store.add_entities(entities, session).await,
            #[cfg(not(any(feature = "neo4j", feature = "memgraph")))]
            GraphStoreEnum::_Phantom(_) => unreachable!(),
        }
    }

    async fn add_relations(
        &self,
        relations: &[agent_mem_traits::Relation],
        session: &agent_mem_traits::Session,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "neo4j")]
            GraphStoreEnum::Neo4j(store) => store.add_relations(relations, session).await,
            #[cfg(feature = "memgraph")]
            GraphStoreEnum::Memgraph(store) => store.add_relations(relations, session).await,
            #[cfg(not(any(feature = "neo4j", feature = "memgraph")))]
            GraphStoreEnum::_Phantom(_) => unreachable!(),
        }
    }

    async fn search_graph(
        &self,
        query: &str,
        session: &agent_mem_traits::Session,
    ) -> Result<Vec<agent_mem_traits::GraphResult>> {
        match self {
            #[cfg(feature = "neo4j")]
            GraphStoreEnum::Neo4j(store) => store.search_graph(query, session).await,
            #[cfg(feature = "memgraph")]
            GraphStoreEnum::Memgraph(store) => store.search_graph(query, session).await,
            #[cfg(not(any(feature = "neo4j", feature = "memgraph")))]
            GraphStoreEnum::_Phantom(_) => unreachable!(),
        }
    }

    async fn get_neighbors(
        &self,
        entity_id: &str,
        depth: usize,
    ) -> Result<Vec<agent_mem_traits::Entity>> {
        match self {
            #[cfg(feature = "neo4j")]
            GraphStoreEnum::Neo4j(store) => store.get_neighbors(entity_id, depth).await,
            #[cfg(feature = "memgraph")]
            GraphStoreEnum::Memgraph(store) => store.get_neighbors(entity_id, depth).await,
            #[cfg(not(any(feature = "neo4j", feature = "memgraph")))]
            GraphStoreEnum::_Phantom(_) => unreachable!(),
        }
    }

    async fn reset(&self) -> Result<()> {
        match self {
            #[cfg(feature = "neo4j")]
            GraphStoreEnum::Neo4j(store) => store.reset().await,
            #[cfg(feature = "memgraph")]
            GraphStoreEnum::Memgraph(store) => store.reset().await,
            #[cfg(not(any(feature = "neo4j", feature = "memgraph")))]
            GraphStoreEnum::_Phantom(_) => unreachable!(),
        }
    }
}

/// 图存储工厂
pub struct GraphStoreFactory;

impl GraphStoreFactory {
    /// 根据配置创建图存储实例
    pub async fn create_graph_store(
        config: &GraphStoreConfig,
    ) -> Result<Arc<dyn GraphStore + Send + Sync>> {
        match config.provider.as_str() {
            "neo4j" => {
                #[cfg(feature = "neo4j")]
                {
                    let store = Neo4jStore::new(config.clone()).await?;
                    let store_enum = GraphStoreEnum::Neo4j(store);
                    Ok(Arc::new(store_enum))
                }
                #[cfg(not(feature = "neo4j"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "Neo4j feature not enabled",
                    ))
                }
            }
            "memgraph" => {
                #[cfg(feature = "memgraph")]
                {
                    let store = MemgraphStore::new(config.clone()).await?;
                    let store_enum = GraphStoreEnum::Memgraph(store);
                    Ok(Arc::new(store_enum))
                }
                #[cfg(not(feature = "memgraph"))]
                {
                    Err(AgentMemError::unsupported_provider(
                        "Memgraph feature not enabled",
                    ))
                }
            }
            _ => Err(AgentMemError::unsupported_provider(&config.provider)),
        }
    }

    /// 获取支持的图存储提供商列表
    pub fn supported_providers() -> Vec<&'static str> {
        #[allow(unused_mut)]
        let mut providers = Vec::new();

        #[cfg(feature = "neo4j")]
        providers.push("neo4j");

        #[cfg(feature = "memgraph")]
        providers.push("memgraph");

        providers
    }

    /// 检查提供商是否受支持
    pub fn is_provider_supported(provider: &str) -> bool {
        Self::supported_providers().contains(&provider)
    }

    /// 创建Neo4j存储
    #[cfg(feature = "neo4j")]
    pub async fn create_neo4j_store(
        uri: &str,
        username: &str,
        password: &str,
    ) -> Result<Arc<dyn GraphStore + Send + Sync>> {
        let config = GraphStoreConfig {
            provider: "neo4j".to_string(),
            uri: uri.to_string(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            database: Some("neo4j".to_string()),
        };
        Self::create_graph_store(&config).await
    }

    /// 创建Memgraph存储
    #[cfg(feature = "memgraph")]
    pub async fn create_memgraph_store(
        uri: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Arc<dyn GraphStore + Send + Sync>> {
        let config = GraphStoreConfig {
            provider: "memgraph".to_string(),
            uri: uri.to_string(),
            username: username.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            database: None,
        };
        Self::create_graph_store(&config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_providers() {
        let providers = GraphStoreFactory::supported_providers();

        #[cfg(feature = "neo4j")]
        assert!(providers.contains(&"neo4j"));

        #[cfg(feature = "memgraph")]
        assert!(providers.contains(&"memgraph"));
    }

    #[test]
    fn test_is_provider_supported() {
        #[cfg(feature = "neo4j")]
        assert!(GraphStoreFactory::is_provider_supported("neo4j"));

        #[cfg(feature = "memgraph")]
        assert!(GraphStoreFactory::is_provider_supported("memgraph"));

        assert!(!GraphStoreFactory::is_provider_supported(
            "unsupported_provider"
        ));
    }

    #[test]
    fn test_create_graph_store_unsupported() {
        let config = GraphStoreConfig {
            provider: "unsupported".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: None,
            database: None,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(GraphStoreFactory::create_graph_store(&config));
        assert!(result.is_err());
    }

    #[cfg(feature = "neo4j")]
    #[tokio::test]
    async fn test_create_neo4j_store() {
        let result =
            GraphStoreFactory::create_neo4j_store("bolt://localhost:7687", "neo4j", "password")
                .await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "memgraph")]
    #[tokio::test]
    async fn test_create_memgraph_store() {
        let result = GraphStoreFactory::create_memgraph_store(
            "bolt://localhost:7687",
            Some("memgraph"),
            Some("password"),
        )
        .await;
        assert!(result.is_ok());
    }
}
