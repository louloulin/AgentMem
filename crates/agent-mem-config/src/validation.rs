//! Configuration validation

use crate::MemoryConfig;
use agent_mem_traits::{AgentMemError, LLMConfig, Result, VectorStoreConfig};

/// Validate memory configuration
pub fn validate_memory_config(config: &MemoryConfig) -> Result<()> {
    validate_llm_config(&config.llm)?;
    validate_storage_config(&config.vector_store)?;

    if let Some(graph_config) = &config.graph_store {
        validate_graph_config(graph_config)?;
    }

    validate_embedder_config(&config.embedder)?;
    validate_session_config(&config.session)?;
    validate_intelligence_config(&config.intelligence)?;

    Ok(())
}

/// Validate LLM configuration
pub fn validate_llm_config(config: &LLMConfig) -> Result<()> {
    if config.provider.is_empty() {
        return Err(AgentMemError::invalid_config(
            "LLM provider cannot be empty",
        ));
    }

    if config.model.is_empty() {
        return Err(AgentMemError::invalid_config("LLM model cannot be empty"));
    }

    // Check provider-specific requirements
    match config.provider.as_str() {
        "openai" | "anthropic" | "azure" | "gemini" => {
            if config.api_key.is_none() {
                return Err(AgentMemError::invalid_config(format!(
                    "{} provider requires an API key",
                    config.provider
                )));
            }
        }
        "ollama" => {
            if config.base_url.is_none() {
                return Err(AgentMemError::invalid_config(
                    "Ollama provider requires a base URL",
                ));
            }
        }
        _ => {
            return Err(AgentMemError::unsupported_provider(&config.provider));
        }
    }

    // Validate temperature range
    if let Some(temp) = config.temperature {
        if !(0.0..=2.0).contains(&temp) {
            return Err(AgentMemError::invalid_config(
                "Temperature must be between 0.0 and 2.0",
            ));
        }
    }

    // Validate max_tokens
    if let Some(max_tokens) = config.max_tokens {
        if max_tokens == 0 {
            return Err(AgentMemError::invalid_config(
                "max_tokens must be greater than 0",
            ));
        }
    }

    Ok(())
}

/// Validate storage configuration
pub fn validate_storage_config(config: &VectorStoreConfig) -> Result<()> {
    if config.provider.is_empty() {
        return Err(AgentMemError::invalid_config(
            "Vector store provider cannot be empty",
        ));
    }

    if config.dimension == Some(0) {
        return Err(AgentMemError::invalid_config(
            "Vector dimension must be greater than 0",
        ));
    }

    // Check provider-specific requirements
    match config.provider.as_str() {
        "memory" => {
            // In-memory vector store for zero-config embedded mode
            // No additional requirements
        }
        "lancedb" => {
            if config.path.is_empty() {
                return Err(AgentMemError::invalid_config("LanceDB requires a path"));
            }
        }
        "pinecone" => {
            if config.api_key.is_none() {
                return Err(AgentMemError::invalid_config(
                    "Pinecone requires an API key",
                ));
            }
            if config.index_name.is_none() {
                return Err(AgentMemError::invalid_config(
                    "Pinecone requires an index name",
                ));
            }
        }
        "qdrant" => {
            if config.path.is_empty() {
                return Err(AgentMemError::invalid_config("Qdrant requires a URL"));
            }
        }
        _ => {
            return Err(AgentMemError::unsupported_provider(&config.provider));
        }
    }

    Ok(())
}

/// Validate graph configuration
pub fn validate_graph_config(config: &crate::memory::GraphStoreConfig) -> Result<()> {
    if config.provider.is_empty() {
        return Err(AgentMemError::invalid_config(
            "Graph store provider cannot be empty",
        ));
    }

    if config.uri.is_empty() {
        return Err(AgentMemError::invalid_config(
            "Graph store URI cannot be empty",
        ));
    }

    match config.provider.as_str() {
        "neo4j" | "memgraph" => {
            // Valid providers
        }
        _ => {
            return Err(AgentMemError::unsupported_provider(&config.provider));
        }
    }

    Ok(())
}

/// Validate embedder configuration
pub fn validate_embedder_config(config: &crate::memory::EmbedderConfig) -> Result<()> {
    if config.provider.is_empty() {
        return Err(AgentMemError::invalid_config(
            "Embedder provider cannot be empty",
        ));
    }

    if config.model.is_empty() {
        return Err(AgentMemError::invalid_config(
            "Embedder model cannot be empty",
        ));
    }

    if config.dimension == 0 {
        return Err(AgentMemError::invalid_config(
            "Embedding dimension must be greater than 0",
        ));
    }

    // Check provider-specific requirements
    match config.provider.as_str() {
        "openai" => {
            if config.api_key.is_none() {
                return Err(AgentMemError::invalid_config(
                    "OpenAI embedder requires an API key",
                ));
            }
        }
        "huggingface" | "local" => {
            // These providers might not require API keys
        }
        _ => {
            return Err(AgentMemError::unsupported_provider(&config.provider));
        }
    }

    Ok(())
}

/// Validate session configuration
pub fn validate_session_config(config: &crate::memory::SessionConfig) -> Result<()> {
    if config.session_timeout_seconds == 0 {
        return Err(AgentMemError::invalid_config(
            "session_timeout_seconds must be greater than 0",
        ));
    }

    Ok(())
}

/// Validate intelligence configuration
pub fn validate_intelligence_config(config: &crate::memory::IntelligenceConfig) -> Result<()> {
    if config.similarity_threshold < 0.0 || config.similarity_threshold > 1.0 {
        return Err(AgentMemError::invalid_config(
            "similarity_threshold must be between 0.0 and 1.0",
        ));
    }

    if config.clustering_threshold < 0.0 || config.clustering_threshold > 1.0 {
        return Err(AgentMemError::invalid_config(
            "clustering_threshold must be between 0.0 and 1.0",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::*;

    #[test]
    fn test_validate_llm_config() {
        let mut config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        assert!(validate_llm_config(&config).is_ok());

        // Test invalid temperature
        config.temperature = Some(3.0);
        assert!(validate_llm_config(&config).is_err());
    }

    #[test]
    fn test_validate_storage_config() {
        let config = VectorStoreConfig {
            provider: "lancedb".to_string(),
            path: "./data/vectors".to_string(),
            table_name: "memories".to_string(),
            dimension: Some(1536),
            ..Default::default()
        };
        assert!(validate_storage_config(&config).is_ok());

        // Test invalid dimension
        let mut invalid_config = config.clone();
        invalid_config.dimension = Some(0);
        assert!(validate_storage_config(&invalid_config).is_err());
    }

    #[test]
    fn test_validate_embedder_config() {
        let config = EmbedderConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        assert!(validate_embedder_config(&config).is_ok());

        // Test invalid dimension
        let mut invalid_config = config.clone();
        invalid_config.dimension = 0;
        assert!(validate_embedder_config(&invalid_config).is_err());
    }
}
