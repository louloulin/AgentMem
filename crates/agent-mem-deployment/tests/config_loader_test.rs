//! 配置加载器测试

use agent_mem_config::{DeploymentMode, VectorServiceType};
use agent_mem_deployment::{ConfigLoader, ConfigFormat};
use std::path::PathBuf;

#[test]
fn test_load_embedded_config_from_toml() {
    let toml_content = r#"
[deployment]
mode = "embedded"

[embedded]
database_path = "./data/test.db"
vector_path = "./data/vectors"
vector_dimension = 1536
enable_wal = true
cache_size_kb = 10240
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

    let mode = result.unwrap();
    match mode {
        DeploymentMode::Embedded(config) => {
            assert_eq!(config.database_path, PathBuf::from("./data/test.db"));
            assert_eq!(config.vector_path, PathBuf::from("./data/vectors"));
            assert_eq!(config.vector_dimension, 1536);
            assert!(config.enable_wal);
            assert_eq!(config.cache_size_kb, 10240);
        }
        _ => panic!("Expected embedded mode"),
    }
}

#[test]
fn test_load_server_config_from_toml() {
    let toml_content = r#"
[deployment]
mode = "server"

[server]
database_url = "postgresql://localhost:5432/test"
vector_dimension = 768
vector_service = "pgvector"
index_type = "ivfflat"

[server.pool]
max_connections = 20
min_connections = 5
connect_timeout_secs = 30
idle_timeout_secs = 600
max_lifetime_secs = 1800

[server.index_params]
ivfflat_lists = 100
hnsw_m = 16
hnsw_ef_construction = 64
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

    let mode = result.unwrap();
    match mode {
        DeploymentMode::Server(config) => {
            assert_eq!(config.database_url, "postgresql://localhost:5432/test");
            assert_eq!(config.vector_dimension, 768);
            assert_eq!(config.vector_service, VectorServiceType::PgVector);
            assert_eq!(config.pool_config.max_connections, 20);
            assert_eq!(config.pool_config.min_connections, 5);
            assert_eq!(config.pool_config.connect_timeout_seconds, 30);
        }
        _ => panic!("Expected server mode"),
    }
}

#[test]
fn test_load_embedded_config_from_json() {
    let json_content = r#"
{
    "deployment": {
        "mode": "embedded"
    },
    "embedded": {
        "database_path": "./data/test.db",
        "vector_path": "./data/vectors",
        "vector_dimension": 1536,
        "enable_wal": true,
        "cache_size_kb": 10240
    }
}
"#;

    let result = ConfigLoader::load_from_str(json_content, ConfigFormat::Json);
    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

    let mode = result.unwrap();
    match mode {
        DeploymentMode::Embedded(config) => {
            assert_eq!(config.database_path, PathBuf::from("./data/test.db"));
            assert_eq!(config.vector_dimension, 1536);
        }
        _ => panic!("Expected embedded mode"),
    }
}

#[test]
fn test_load_embedded_config_from_yaml() {
    let yaml_content = r#"
deployment:
  mode: embedded

embedded:
  database_path: ./data/test.db
  vector_path: ./data/vectors
  vector_dimension: 1536
  enable_wal: true
  cache_size_kb: 10240
"#;

    let result = ConfigLoader::load_from_str(yaml_content, ConfigFormat::Yaml);
    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

    let mode = result.unwrap();
    match mode {
        DeploymentMode::Embedded(config) => {
            assert_eq!(config.database_path, PathBuf::from("./data/test.db"));
            assert_eq!(config.vector_dimension, 1536);
        }
        _ => panic!("Expected embedded mode"),
    }
}

#[test]
fn test_load_config_with_defaults() {
    let toml_content = r#"
[deployment]
mode = "embedded"

[embedded]
database_path = "./data/test.db"
vector_path = "./data/vectors"
vector_dimension = 1536
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

    let mode = result.unwrap();
    match mode {
        DeploymentMode::Embedded(config) => {
            // 验证默认值
            assert!(config.enable_wal);
            assert_eq!(config.cache_size_kb, 10240);
        }
        _ => panic!("Expected embedded mode"),
    }
}

#[test]
fn test_load_server_config_with_different_vector_services() {
    let services = vec![
        ("pgvector", VectorServiceType::PgVector),
        ("lancedb", VectorServiceType::LanceDB),
        ("pinecone", VectorServiceType::Pinecone),
        ("qdrant", VectorServiceType::Qdrant),
        ("milvus", VectorServiceType::Milvus),
        ("weaviate", VectorServiceType::Weaviate),
        ("chroma", VectorServiceType::Chroma),
        ("elasticsearch", VectorServiceType::Elasticsearch),
        ("redis", VectorServiceType::Redis),
        ("mongodb", VectorServiceType::MongoDB),
        ("supabase", VectorServiceType::Supabase),
        ("faiss", VectorServiceType::FAISS),
        ("azure_ai_search", VectorServiceType::AzureAISearch),
        ("memory", VectorServiceType::Memory),
    ];

    for (service_name, expected_type) in services {
        let toml_content = format!(
            r#"
[deployment]
mode = "server"

[server]
database_url = "postgresql://localhost:5432/test"
vector_dimension = 768
vector_service = "{service_name}"

[server.pool]
max_connections = 20
"#
        );

        let result = ConfigLoader::load_from_str(&toml_content, ConfigFormat::Toml);
        assert!(
            result.is_ok(),
            "Failed to load config for {}: {:?}",
            service_name,
            result.err()
        );

        let mode = result.unwrap();
        match mode {
            DeploymentMode::Server(config) => {
                assert_eq!(
                    config.vector_service, expected_type,
                    "Mismatch for service: {service_name}"
                );
            }
            _ => panic!("Expected server mode for {service_name}"),
        }
    }
}

#[test]
fn test_load_config_missing_mode() {
    let toml_content = r#"
[embedded]
database_path = "./data/test.db"
vector_path = "./data/vectors"
vector_dimension = 1536
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_err(), "Should fail with missing deployment mode");
}

#[test]
fn test_load_config_invalid_mode() {
    let toml_content = r#"
[deployment]
mode = "invalid"

[embedded]
database_path = "./data/test.db"
vector_path = "./data/vectors"
vector_dimension = 1536
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_err(), "Should fail with invalid mode");
}

#[test]
fn test_load_config_missing_embedded_section() {
    let toml_content = r#"
[deployment]
mode = "embedded"
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_err(), "Should fail with missing embedded section");
}

#[test]
fn test_load_config_missing_server_section() {
    let toml_content = r#"
[deployment]
mode = "server"
"#;

    let result = ConfigLoader::load_from_str(toml_content, ConfigFormat::Toml);
    assert!(result.is_err(), "Should fail with missing server section");
}

#[test]
fn test_config_format_from_extension() {
    assert_eq!(
        ConfigFormat::from_extension(PathBuf::from("config.toml").as_path()),
        Some(ConfigFormat::Toml)
    );
    assert_eq!(
        ConfigFormat::from_extension(PathBuf::from("config.json").as_path()),
        Some(ConfigFormat::Json)
    );
    assert_eq!(
        ConfigFormat::from_extension(PathBuf::from("config.yaml").as_path()),
        Some(ConfigFormat::Yaml)
    );
    assert_eq!(
        ConfigFormat::from_extension(PathBuf::from("config.yml").as_path()),
        Some(ConfigFormat::Yaml)
    );
    assert_eq!(
        ConfigFormat::from_extension(PathBuf::from("config.txt").as_path()),
        None
    );
}

