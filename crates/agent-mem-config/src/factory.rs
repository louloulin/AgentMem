//! Configuration factory

use crate::MemoryConfig;
use agent_mem_traits::{LLMConfig, Result, VectorStoreConfig};
use std::env;

/// Configuration factory for creating configurations
pub struct ConfigFactory;

impl ConfigFactory {
    /// Create default memory configuration
    pub fn create_memory_config() -> MemoryConfig {
        MemoryConfig::default()
    }

    /// Create LLM configuration for a specific provider
    pub fn create_llm_config(provider: &str) -> LLMConfig {
        match provider {
            "openai" => LLMConfig {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                ..Default::default()
            },
            "anthropic" => LLMConfig {
                provider: "anthropic".to_string(),
                model: "claude-3-sonnet-20240229".to_string(),
                ..Default::default()
            },
            "azure" => LLMConfig {
                provider: "azure".to_string(),
                model: "gpt-35-turbo".to_string(),
                ..Default::default()
            },
            "gemini" => LLMConfig {
                provider: "gemini".to_string(),
                model: "gemini-pro".to_string(),
                ..Default::default()
            },
            "ollama" => LLMConfig {
                provider: "ollama".to_string(),
                model: "llama2".to_string(),
                base_url: Some("http://localhost:11434".to_string()),
                ..Default::default()
            },
            _ => LLMConfig::default(),
        }
    }

    /// Create vector store configuration for specific providers
    pub fn create_vector_store_config(provider: &str) -> VectorStoreConfig {
        match provider {
            "lancedb" => VectorStoreConfig {
                provider: "lancedb".to_string(),
                path: "./data/vectors".to_string(),
                table_name: "memories".to_string(),
                dimension: Some(1536),
                ..Default::default()
            },
            "pinecone" => VectorStoreConfig {
                provider: "pinecone".to_string(),
                index_name: Some("default-index".to_string()),
                dimension: Some(1536),
                ..Default::default()
            },
            "qdrant" => VectorStoreConfig {
                provider: "qdrant".to_string(),
                path: "http://localhost:6333".to_string(),
                table_name: "memories".to_string(),
                dimension: Some(1536),
                ..Default::default()
            },
            _ => VectorStoreConfig::default(),
        }
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Result<MemoryConfig> {
        let llm_provider =
            env::var("AGENT_MEM_LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
        let vector_provider =
            env::var("AGENT_MEM_VECTOR_PROVIDER").unwrap_or_else(|_| "lancedb".to_string());
        let _graph_provider = env::var("AGENT_MEM_GRAPH_PROVIDER").ok();

        let mut llm_config = Self::create_llm_config(&llm_provider);
        let vector_store_config = Self::create_vector_store_config(&vector_provider);

        // Set API keys from environment
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            llm_config.api_key = Some(api_key);
        }
        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            if llm_provider == "anthropic" {
                llm_config.api_key = Some(api_key);
            }
        }
        if let Ok(_api_key) = env::var("PINECONE_API_KEY") {
            // Set Pinecone API key in storage config if needed
        }

        let mut embedder_config = crate::memory::EmbedderConfig::default();
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            embedder_config.api_key = Some(api_key);
        }

        Ok(MemoryConfig {
            llm: llm_config,
            vector_store: vector_store_config,
            graph_store: None, // TODO: Add graph store config from env
            embedder: embedder_config,
            ..Default::default()
        })
    }

    /// Create configuration from TOML file
    pub fn from_file(path: &str) -> Result<MemoryConfig> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            agent_mem_traits::AgentMemError::config_error(format!(
                "Failed to read config file: {e}"
            ))
        })?;

        let config: MemoryConfig = toml::from_str(&content).map_err(|e| {
            agent_mem_traits::AgentMemError::config_error(format!(
                "Failed to parse config file: {e}"
            ))
        })?;

        crate::validation::validate_memory_config(&config)?;

        Ok(config)
    }

    /// Create configuration using the config crate (supports multiple formats)
    pub fn from_config_sources() -> Result<MemoryConfig> {
        let settings = config::Config::builder()
            // Start with default values
            .add_source(config::File::with_name("config/default").required(false))
            // Add environment-specific config
            .add_source(
                config::File::with_name(&format!(
                    "config/{}",
                    env::var("AGENT_MEM_ENV").unwrap_or_else(|_| "development".to_string())
                ))
                .required(false),
            )
            // Add local config (gitignored)
            .add_source(config::File::with_name("config/local").required(false))
            // Add environment variables with prefix
            .add_source(config::Environment::with_prefix("AGENT_MEM"))
            .build()
            .map_err(|e| {
                agent_mem_traits::AgentMemError::config_error(format!(
                    "Failed to build config: {e}"
                ))
            })?;

        let config: MemoryConfig = settings.try_deserialize().map_err(|e| {
            agent_mem_traits::AgentMemError::config_error(format!(
                "Failed to deserialize config: {e}"
            ))
        })?;

        crate::validation::validate_memory_config(&config)?;

        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_file(config: &MemoryConfig, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(config).map_err(|e| {
            agent_mem_traits::AgentMemError::config_error(format!(
                "Failed to serialize config: {e}"
            ))
        })?;

        std::fs::write(path, content).map_err(|e| {
            agent_mem_traits::AgentMemError::config_error(format!(
                "Failed to write config file: {e}"
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_memory_config() {
        let config = ConfigFactory::create_memory_config();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.vector_store.provider, "memory"); // Changed to "memory" for zero-config embedded mode
    }

    #[test]
    fn test_create_llm_config() {
        let openai_config = ConfigFactory::create_llm_config("openai");
        assert_eq!(openai_config.provider, "openai");
        assert_eq!(openai_config.model, "gpt-3.5-turbo");

        let anthropic_config = ConfigFactory::create_llm_config("anthropic");
        assert_eq!(anthropic_config.provider, "anthropic");
        assert_eq!(anthropic_config.model, "claude-3-sonnet-20240229");
    }

    #[test]
    fn test_create_vector_store_config() {
        let config = ConfigFactory::create_vector_store_config("lancedb");
        assert_eq!(config.provider, "lancedb");
        assert_eq!(config.path, "./data/vectors");
    }

    #[test]
    fn test_from_file() {
        let config_content = r#"
[llm]
provider = "openai"
model = "gpt-4"
api_key = "test-key"

[vector_store]
provider = "lancedb"
path = "./test_data"
table_name = "test_memories"
dimension = 1536

[embedder]
provider = "openai"
model = "text-embedding-ada-002"
api_key = "test-key"
dimension = 1536

[session]
session_timeout_seconds = 7200
enable_telemetry = true

[intelligence]
similarity_threshold = 0.85
clustering_threshold = 0.7
enable_conflict_detection = true
enable_memory_summarization = true
importance_scoring = true
enable_intelligent_extraction = true
enable_decision_engine = true
enable_deduplication = true

[intelligence.fact_extraction]
max_facts_per_memory = 10
confidence_threshold = 0.7

[intelligence.decision_engine]
confidence_threshold = 0.75
enable_auto_merge = true

[intelligence.deduplication]
similarity_threshold = 0.95
check_window_size = 100
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = ConfigFactory::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.vector_store.provider, "lancedb");
        assert_eq!(config.intelligence.similarity_threshold, 0.85);
    }

    #[test]
    fn test_save_to_file() {
        let mut config = ConfigFactory::create_memory_config();
        // Set API key to make config valid
        config.llm.api_key = Some("test-key".to_string());
        config.embedder.api_key = Some("test-key".to_string());

        let temp_file = NamedTempFile::new().unwrap();

        ConfigFactory::save_to_file(&config, temp_file.path().to_str().unwrap()).unwrap();

        // Verify we can read it back
        let loaded_config = ConfigFactory::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.llm.provider, loaded_config.llm.provider);
    }
}
