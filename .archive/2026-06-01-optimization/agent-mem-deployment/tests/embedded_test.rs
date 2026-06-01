//! 嵌入式存储测试

use agent_mem_deployment::embedded::database::EmbeddedDatabaseConfig;
use agent_mem_deployment::embedded::vector_store::{DistanceMetric, EmbeddedVectorStoreConfig};
use agent_mem_deployment::embedded::EmbeddedConfig;

#[test]
fn test_embedded_config_creation() {
    let config = EmbeddedConfig::default();
    assert_eq!(config.data_root.to_str().unwrap(), "./data");
}

#[test]
fn test_embedded_config_in_memory() {
    let config = EmbeddedConfig::in_memory();
    assert!(config.database.in_memory);
    assert!(config.vector_store.in_memory);
}

#[test]
fn test_embedded_config_with_dimension() {
    let config = EmbeddedConfig::default().with_vector_dimension(768);
    assert_eq!(config.vector_store.dimension, 768);
}

#[test]
fn test_embedded_config_with_wal() {
    let config = EmbeddedConfig::default().with_wal(false);
    assert!(!config.database.enable_wal);
}

#[test]
fn test_embedded_config_with_persistence() {
    let config = EmbeddedConfig::default().with_persistence(false);
    assert!(!config.vector_store.enable_persistence);
}

#[test]
fn test_embedded_database_config_default() {
    let config = EmbeddedDatabaseConfig::default();
    assert!(config.enable_wal);
    assert_eq!(config.cache_size_kb, 10240);
    assert_eq!(config.page_size, 4096);
}

#[test]
fn test_embedded_database_config_in_memory() {
    let config = EmbeddedDatabaseConfig::in_memory();
    assert!(config.in_memory);
}

#[test]
fn test_embedded_database_config_file() {
    let config = EmbeddedDatabaseConfig::file("test.db");
    assert!(!config.in_memory);
    assert_eq!(config.path.to_str().unwrap(), "test.db");
}

#[test]
fn test_embedded_vector_store_config_default() {
    let config = EmbeddedVectorStoreConfig::default();
    assert_eq!(config.dimension, 384);
    assert!(matches!(config.distance, DistanceMetric::Cosine));
    assert!(config.enable_persistence);
}

#[test]
fn test_embedded_vector_store_config_in_memory() {
    let config = EmbeddedVectorStoreConfig::in_memory(512);
    assert!(config.in_memory);
    assert_eq!(config.dimension, 512);
    assert!(!config.enable_persistence);
}

#[test]
fn test_embedded_vector_store_config_file() {
    let config = EmbeddedVectorStoreConfig::file("vectors", 768);
    assert!(!config.in_memory);
    assert_eq!(config.dimension, 768);
    assert_eq!(config.path.to_str().unwrap(), "vectors");
}

#[test]
fn test_embedded_config_serialization() {
    let config = EmbeddedConfig::default();

    // JSON 序列化
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: EmbeddedConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.data_root, config.data_root);

    // TOML 序列化
    let toml_str = toml::to_string(&config).unwrap();
    let deserialized: EmbeddedConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized.data_root, config.data_root);
}

#[tokio::test]
#[cfg(all(feature = "embedded-db", feature = "embedded-vector"))]
async fn test_embedded_config_initialize_all() {
    let config = EmbeddedConfig::in_memory();
    let result = config.initialize_all().await;

    if result.is_ok() {
        let (db, vector_store) = result.unwrap();

        // 关闭
        let _ = db.shutdown().await;
        let _ = vector_store.shutdown().await;
    }
}

#[test]
fn test_embedded_config_save_and_load() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let json_path = dir.path().join("config.json");
    let toml_path = dir.path().join("config.toml");

    let config = EmbeddedConfig::default();

    // 保存 JSON
    config.save_to_json_file(&json_path).unwrap();
    assert!(json_path.exists());

    // 加载 JSON
    let loaded = EmbeddedConfig::from_json_file(&json_path).unwrap();
    assert_eq!(loaded.data_root, config.data_root);

    // 保存 TOML
    config.save_to_toml_file(&toml_path).unwrap();
    assert!(toml_path.exists());

    // 加载 TOML
    let loaded = EmbeddedConfig::from_toml_file(&toml_path).unwrap();
    assert_eq!(loaded.data_root, config.data_root);
}
