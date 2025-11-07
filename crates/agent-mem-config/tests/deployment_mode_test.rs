//! 部署模式配置测试

use agent_mem_config::storage::{
    DeploymentMode, EmbeddedModeConfig, PoolConfig, ServerModeConfig, VectorServiceType,
    VectorStoreConfig,
};
use std::path::PathBuf;

#[test]
fn test_embedded_mode_default() {
    let config = EmbeddedModeConfig::default();
    assert_eq!(config.database_path, PathBuf::from("./data/agentmem.db"));
    assert_eq!(config.vector_path, PathBuf::from("./data/vectors"));
    assert_eq!(config.vector_dimension, 1536);
    assert!(config.enable_wal);
    assert_eq!(config.cache_size_kb, 10240);
}

#[test]
fn test_server_mode_default() {
    let config = ServerModeConfig::default();
    assert_eq!(config.database_url, "postgresql://localhost:5432/agentmem");
    assert_eq!(config.vector_service, VectorServiceType::PgVector);
    assert_eq!(config.vector_dimension, 1536);
}

#[test]
fn test_pool_config_default() {
    let config = PoolConfig::default();
    assert_eq!(config.min_connections, 2);
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.connect_timeout_seconds, 30);
    assert_eq!(config.idle_timeout_seconds, 600);
    assert_eq!(config.max_lifetime_seconds, 1800);
}

#[test]
fn test_deployment_mode_embedded() {
    let mode = DeploymentMode::embedded("./test_data");

    match mode {
        DeploymentMode::Embedded(config) => {
            assert_eq!(
                config.database_path,
                PathBuf::from("./test_data/agentmem.db")
            );
            assert_eq!(config.vector_path, PathBuf::from("./test_data/vectors"));
            assert_eq!(config.vector_dimension, 1536);
            assert!(config.enable_wal);
        }
        _ => panic!("Expected Embedded mode"),
    }
}

#[test]
fn test_deployment_mode_server_with_pgvector() {
    let mode = DeploymentMode::server_with_pgvector("postgresql://localhost:5432/test".to_string());

    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::PgVector);
            assert_eq!(config.vector_config.base.provider, "pgvector");
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_deployment_mode_server_with_lancedb() {
    let mode = DeploymentMode::server_with_lancedb(
        "postgresql://localhost:5432/test".to_string(),
        "./vectors".to_string(),
    );

    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::LanceDB);
            assert_eq!(config.vector_config.base.provider, "lancedb");
            assert_eq!(config.vector_config.base.url, Some("./vectors".to_string()));
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_deployment_mode_server_with_qdrant() {
    let mode = DeploymentMode::server_with_qdrant(
        "postgresql://localhost:5432/test".to_string(),
        "http://localhost:6333".to_string(),
        "memories".to_string(),
    );

    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::Qdrant);
            assert_eq!(config.vector_config.base.provider, "qdrant");
            assert_eq!(
                config.vector_config.base.url,
                Some("http://localhost:6333".to_string())
            );
            assert_eq!(
                config.vector_config.base.collection_name,
                Some("memories".to_string())
            );
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_deployment_mode_server_with_pinecone() {
    let mode = DeploymentMode::server_with_pinecone(
        "postgresql://localhost:5432/test".to_string(),
        "test-api-key".to_string(),
        "test-index".to_string(),
    );

    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::Pinecone);
            assert_eq!(config.vector_config.base.provider, "pinecone");
            assert_eq!(
                config.vector_config.base.api_key,
                Some("test-api-key".to_string())
            );
            assert_eq!(
                config.vector_config.base.index_name,
                Some("test-index".to_string())
            );
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_deployment_mode_server_with_milvus() {
    let mode = DeploymentMode::server_with_milvus(
        "postgresql://localhost:5432/test".to_string(),
        "localhost:19530".to_string(),
        "memories".to_string(),
    );

    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::Milvus);
            assert_eq!(config.vector_config.base.provider, "milvus");
            assert_eq!(
                config.vector_config.base.url,
                Some("localhost:19530".to_string())
            );
            assert_eq!(
                config.vector_config.base.collection_name,
                Some("memories".to_string())
            );
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_deployment_mode_server_with_vector_service() {
    let vector_config = VectorStoreConfig::default();
    let mode = DeploymentMode::server_with_vector_service(
        "postgresql://localhost:5432/test".to_string(),
        VectorServiceType::Chroma,
        vector_config.clone(),
    );

    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::Chroma);
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_vector_service_type_name() {
    assert_eq!(VectorServiceType::PgVector.name(), "pgvector");
    assert_eq!(VectorServiceType::LanceDB.name(), "lancedb");
    assert_eq!(VectorServiceType::Pinecone.name(), "pinecone");
    assert_eq!(VectorServiceType::Qdrant.name(), "qdrant");
    assert_eq!(VectorServiceType::Milvus.name(), "milvus");
    assert_eq!(VectorServiceType::Weaviate.name(), "weaviate");
    assert_eq!(VectorServiceType::Chroma.name(), "chroma");
    assert_eq!(VectorServiceType::Elasticsearch.name(), "elasticsearch");
    assert_eq!(VectorServiceType::Redis.name(), "redis");
    assert_eq!(VectorServiceType::MongoDB.name(), "mongodb");
    assert_eq!(VectorServiceType::Supabase.name(), "supabase");
    assert_eq!(VectorServiceType::FAISS.name(), "faiss");
    assert_eq!(VectorServiceType::AzureAISearch.name(), "azure_ai_search");
    assert_eq!(VectorServiceType::Memory.name(), "memory");
}

#[test]
fn test_vector_service_type_is_cloud_hosted() {
    assert!(VectorServiceType::Pinecone.is_cloud_hosted());
    assert!(VectorServiceType::Supabase.is_cloud_hosted());
    assert!(VectorServiceType::AzureAISearch.is_cloud_hosted());

    assert!(!VectorServiceType::PgVector.is_cloud_hosted());
    assert!(!VectorServiceType::LanceDB.is_cloud_hosted());
    assert!(!VectorServiceType::Qdrant.is_cloud_hosted());
    assert!(!VectorServiceType::Memory.is_cloud_hosted());
}

#[test]
fn test_vector_service_type_supports_embedded() {
    assert!(VectorServiceType::LanceDB.supports_embedded());
    assert!(VectorServiceType::FAISS.supports_embedded());
    assert!(VectorServiceType::Memory.supports_embedded());

    assert!(!VectorServiceType::PgVector.supports_embedded());
    assert!(!VectorServiceType::Pinecone.supports_embedded());
    assert!(!VectorServiceType::Qdrant.supports_embedded());
}

#[test]
fn test_serde_deployment_mode_embedded() {
    let mode = DeploymentMode::embedded("./data");
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: DeploymentMode = serde_json::from_str(&json).unwrap();

    match deserialized {
        DeploymentMode::Embedded(config) => {
            assert_eq!(config.database_path, PathBuf::from("./data/agentmem.db"));
            assert_eq!(config.vector_path, PathBuf::from("./data/vectors"));
        }
        _ => panic!("Expected Embedded mode"),
    }
}

#[test]
fn test_serde_deployment_mode_server() {
    let mode = DeploymentMode::server_with_pgvector("postgresql://localhost:5432/test".to_string());
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: DeploymentMode = serde_json::from_str(&json).unwrap();

    match deserialized {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_service, VectorServiceType::PgVector);
        }
        _ => panic!("Expected Server mode"),
    }
}

#[test]
fn test_serde_vector_service_type() {
    let service = VectorServiceType::Qdrant;
    let json = serde_json::to_string(&service).unwrap();
    assert_eq!(json, "\"qdrant\"");

    let deserialized: VectorServiceType = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, VectorServiceType::Qdrant);
}
