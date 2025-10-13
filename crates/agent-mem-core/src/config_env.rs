//! Environment-based Configuration
//!
//! This module provides utilities to configure AgentMem from environment variables,
//! enabling zero-configuration usage similar to mem0.
//!
//! # Supported Environment Variables
//!
//! ## Database Configuration
//! - `DATABASE_URL`: Full database connection string (takes precedence)
//! - `AGENTMEM_DB_PATH`: Path to LibSQL database file (default: "agentmem.db")
//! - `AGENTMEM_DB_BACKEND`: Backend type ("postgres" or "libsql", default: "libsql")
//!
//! ## LLM Configuration
//! - `OPENAI_API_KEY`: OpenAI API key
//! - `ANTHROPIC_API_KEY`: Anthropic API key
//! - `AGENTMEM_LLM_PROVIDER`: LLM provider ("openai", "anthropic", "ollama", default: auto-detect)
//! - `AGENTMEM_LLM_MODEL`: LLM model name (default: provider-specific)
//!
//! ## Embedding Configuration
//! - `AGENTMEM_EMBEDDING_PROVIDER`: Embedding provider ("openai", "local", default: "openai")
//! - `AGENTMEM_EMBEDDING_MODEL`: Embedding model name (default: "text-embedding-3-small")
//!
//! ## Vector Store Configuration
//! - `AGENTMEM_VECTOR_STORE`: Vector store provider ("qdrant", "pinecone", "weaviate", etc.)
//! - `QDRANT_URL`: Qdrant server URL
//! - `PINECONE_API_KEY`: Pinecone API key
//! - `WEAVIATE_URL`: Weaviate server URL

use agent_mem_storage::factory::{create_factory, StorageBackend, StorageConfig};
use agent_mem_storage::factory::AllStores;
use agent_mem_traits::Result;
use std::env;

/// Default database path for LibSQL
const DEFAULT_LIBSQL_PATH: &str = "agentmem.db";

/// LLM provider configuration from environment
#[derive(Debug, Clone)]
pub struct LLMEnvConfig {
    /// LLM provider (openai, anthropic, ollama, etc.)
    pub provider: Option<String>,
    /// LLM model name
    pub model: Option<String>,
    /// API key (if applicable)
    pub api_key: Option<String>,
}

/// Embedding provider configuration from environment
#[derive(Debug, Clone)]
pub struct EmbeddingEnvConfig {
    /// Embedding provider (openai, local, etc.)
    pub provider: Option<String>,
    /// Embedding model name
    pub model: Option<String>,
    /// API key (if applicable)
    pub api_key: Option<String>,
}

/// Vector store configuration from environment
#[derive(Debug, Clone)]
pub struct VectorStoreEnvConfig {
    /// Vector store provider (qdrant, pinecone, weaviate, etc.)
    pub provider: Option<String>,
    /// Connection URL or endpoint
    pub url: Option<String>,
    /// API key (if applicable)
    pub api_key: Option<String>,
}

/// Get storage configuration from environment variables
///
/// Reads the following environment variables:
/// - `DATABASE_URL`: Full database connection string (takes precedence)
/// - `AGENTMEM_DB_PATH`: Path to LibSQL database file (default: "agentmem.db")
/// - `AGENTMEM_DB_BACKEND`: Backend type ("postgres" or "libsql", default: "libsql")
///
/// # Examples
///
/// ```bash
/// # Use LibSQL with default path
/// cargo run
///
/// # Use LibSQL with custom path
/// export AGENTMEM_DB_PATH="./data/memory.db"
/// cargo run
///
/// # Use PostgreSQL
/// export DATABASE_URL="postgresql://user:pass@localhost/agentmem"
/// cargo run
/// ```
pub fn get_storage_config_from_env() -> Result<StorageConfig> {
    // Check for DATABASE_URL first (standard convention)
    if let Ok(database_url) = env::var("DATABASE_URL") {
        let backend = if database_url.starts_with("postgres://")
            || database_url.starts_with("postgresql://")
        {
            StorageBackend::PostgreSQL
        } else if database_url.starts_with("libsql://")
            || database_url.starts_with("file:")
            || database_url.ends_with(".db")
        {
            StorageBackend::LibSQL
        } else {
            // Default to LibSQL for unknown formats
            StorageBackend::LibSQL
        };

        return Ok(StorageConfig::new(backend, database_url));
    }

    // Check for AGENTMEM_DB_BACKEND
    let backend = match env::var("AGENTMEM_DB_BACKEND")
        .unwrap_or_else(|_| "libsql".to_string())
        .to_lowercase()
        .as_str()
    {
        "postgres" | "postgresql" => StorageBackend::PostgreSQL,
        "libsql" | "sqlite" => StorageBackend::LibSQL,
        _ => StorageBackend::LibSQL, // Default to LibSQL
    };

    // Get connection string based on backend
    let connection = match backend {
        StorageBackend::PostgreSQL => {
            env::var("POSTGRES_URL").unwrap_or_else(|_| {
                "postgresql://postgres:postgres@localhost:5432/agentmem".to_string()
            })
        }
        StorageBackend::LibSQL => {
            let path = env::var("AGENTMEM_DB_PATH").unwrap_or_else(|_| DEFAULT_LIBSQL_PATH.to_string());
            if path.starts_with("file:") || path.starts_with("libsql://") {
                path
            } else {
                format!("file:{}", path)
            }
        }
    };

    Ok(StorageConfig::new(backend, connection))
}

/// Create all memory stores from environment configuration
///
/// This is a convenience function that:
/// 1. Reads configuration from environment variables
/// 2. Creates a storage factory
/// 3. Initializes all memory stores
///
/// # Example
///
/// ```no_run
/// use agent_mem_core::config_env::create_stores_from_env;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let stores = create_stores_from_env().await?;
/// // Use stores.core, stores.episodic, etc.
/// # Ok(())
/// # }
/// ```
pub async fn create_stores_from_env() -> Result<AllStores> {
    let config = get_storage_config_from_env()?;
    let factory = create_factory(config).await?;
    factory.create_all_stores().await
}

/// Check if database configuration is available
///
/// Returns true if any of the following environment variables are set:
/// - DATABASE_URL
/// - AGENTMEM_DB_PATH
/// - POSTGRES_URL
pub fn has_database_config() -> bool {
    env::var("DATABASE_URL").is_ok()
        || env::var("AGENTMEM_DB_PATH").is_ok()
        || env::var("POSTGRES_URL").is_ok()
}

/// Get LLM configuration from environment variables
///
/// Reads the following environment variables:
/// - `OPENAI_API_KEY`: OpenAI API key
/// - `ANTHROPIC_API_KEY`: Anthropic API key
/// - `AGENTMEM_LLM_PROVIDER`: LLM provider (openai, anthropic, ollama, etc.)
/// - `AGENTMEM_LLM_MODEL`: LLM model name
///
/// # Examples
///
/// ```bash
/// # Use OpenAI
/// export OPENAI_API_KEY="sk-..."
/// export AGENTMEM_LLM_PROVIDER="openai"
/// export AGENTMEM_LLM_MODEL="gpt-4"
///
/// # Use Anthropic
/// export ANTHROPIC_API_KEY="sk-ant-..."
/// export AGENTMEM_LLM_PROVIDER="anthropic"
/// export AGENTMEM_LLM_MODEL="claude-3-opus-20240229"
///
/// # Use Ollama (local)
/// export AGENTMEM_LLM_PROVIDER="ollama"
/// export AGENTMEM_LLM_MODEL="llama2"
/// ```
pub fn get_llm_config_from_env() -> LLMEnvConfig {
    // Auto-detect provider based on API keys if not explicitly set
    let provider = env::var("AGENTMEM_LLM_PROVIDER").ok().or_else(|| {
        if env::var("OPENAI_API_KEY").is_ok() {
            Some("openai".to_string())
        } else if env::var("ANTHROPIC_API_KEY").is_ok() {
            Some("anthropic".to_string())
        } else {
            None
        }
    });

    // Get API key based on provider
    let api_key = match provider.as_deref() {
        Some("openai") => env::var("OPENAI_API_KEY").ok(),
        Some("anthropic") => env::var("ANTHROPIC_API_KEY").ok(),
        _ => None,
    };

    let model = env::var("AGENTMEM_LLM_MODEL").ok();

    LLMEnvConfig {
        provider,
        model,
        api_key,
    }
}

/// Get embedding configuration from environment variables
///
/// Reads the following environment variables:
/// - `AGENTMEM_EMBEDDING_PROVIDER`: Embedding provider (openai, local, etc.)
/// - `AGENTMEM_EMBEDDING_MODEL`: Embedding model name
/// - `OPENAI_API_KEY`: OpenAI API key (if using OpenAI embeddings)
///
/// # Examples
///
/// ```bash
/// # Use OpenAI embeddings
/// export OPENAI_API_KEY="sk-..."
/// export AGENTMEM_EMBEDDING_PROVIDER="openai"
/// export AGENTMEM_EMBEDDING_MODEL="text-embedding-3-small"
///
/// # Use local embeddings
/// export AGENTMEM_EMBEDDING_PROVIDER="local"
/// export AGENTMEM_EMBEDDING_MODEL="all-MiniLM-L6-v2"
/// ```
pub fn get_embedding_config_from_env() -> EmbeddingEnvConfig {
    let provider = env::var("AGENTMEM_EMBEDDING_PROVIDER")
        .ok()
        .or_else(|| {
            // Default to openai if API key is available
            if env::var("OPENAI_API_KEY").is_ok() {
                Some("openai".to_string())
            } else {
                None
            }
        });

    let api_key = if provider.as_deref() == Some("openai") {
        env::var("OPENAI_API_KEY").ok()
    } else {
        None
    };

    let model = env::var("AGENTMEM_EMBEDDING_MODEL").ok();

    EmbeddingEnvConfig {
        provider,
        model,
        api_key,
    }
}

/// Get vector store configuration from environment variables
///
/// Reads the following environment variables:
/// - `AGENTMEM_VECTOR_STORE`: Vector store provider
/// - `QDRANT_URL`: Qdrant server URL
/// - `PINECONE_API_KEY`: Pinecone API key
/// - `WEAVIATE_URL`: Weaviate server URL
///
/// # Examples
///
/// ```bash
/// # Use Qdrant
/// export AGENTMEM_VECTOR_STORE="qdrant"
/// export QDRANT_URL="http://localhost:6333"
///
/// # Use Pinecone
/// export AGENTMEM_VECTOR_STORE="pinecone"
/// export PINECONE_API_KEY="..."
///
/// # Use Weaviate
/// export AGENTMEM_VECTOR_STORE="weaviate"
/// export WEAVIATE_URL="http://localhost:8080"
/// ```
pub fn get_vector_store_config_from_env() -> VectorStoreEnvConfig {
    let provider = env::var("AGENTMEM_VECTOR_STORE").ok();

    let (url, api_key) = match provider.as_deref() {
        Some("qdrant") => (env::var("QDRANT_URL").ok(), None),
        Some("pinecone") => (None, env::var("PINECONE_API_KEY").ok()),
        Some("weaviate") => (env::var("WEAVIATE_URL").ok(), None),
        _ => (None, None),
    };

    VectorStoreEnvConfig {
        provider,
        url,
        api_key,
    }
}

/// Check if LLM configuration is available in environment
pub fn has_llm_config() -> bool {
    env::var("OPENAI_API_KEY").is_ok()
        || env::var("ANTHROPIC_API_KEY").is_ok()
        || env::var("AGENTMEM_LLM_PROVIDER").is_ok()
}

/// Check if embedding configuration is available in environment
pub fn has_embedding_config() -> bool {
    env::var("AGENTMEM_EMBEDDING_PROVIDER").is_ok() || env::var("OPENAI_API_KEY").is_ok()
}

/// Check if vector store configuration is available in environment
pub fn has_vector_store_config() -> bool {
    env::var("AGENTMEM_VECTOR_STORE").is_ok()
        || env::var("QDRANT_URL").is_ok()
        || env::var("PINECONE_API_KEY").is_ok()
        || env::var("WEAVIATE_URL").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        // Clear environment
        env::remove_var("DATABASE_URL");
        env::remove_var("AGENTMEM_DB_PATH");
        env::remove_var("AGENTMEM_DB_BACKEND");

        let config = get_storage_config_from_env().unwrap();
        assert_eq!(config.backend, StorageBackend::LibSQL);
        assert!(config.connection.contains("agentmem.db"));
    }

    #[test]
    fn test_database_url_postgres() {
        env::set_var("DATABASE_URL", "postgresql://localhost/test");

        let config = get_storage_config_from_env().unwrap();
        assert_eq!(config.backend, StorageBackend::PostgreSQL);
        assert_eq!(config.connection, "postgresql://localhost/test");

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_database_url_libsql() {
        env::set_var("DATABASE_URL", "file:test.db");

        let config = get_storage_config_from_env().unwrap();
        assert_eq!(config.backend, StorageBackend::LibSQL);
        assert_eq!(config.connection, "file:test.db");

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_custom_libsql_path() {
        env::remove_var("DATABASE_URL");
        env::set_var("AGENTMEM_DB_PATH", "custom.db");

        let config = get_storage_config_from_env().unwrap();
        assert_eq!(config.backend, StorageBackend::LibSQL);
        assert!(config.connection.contains("custom.db"));

        env::remove_var("AGENTMEM_DB_PATH");
    }

    #[test]
    fn test_has_database_config() {
        env::remove_var("DATABASE_URL");
        env::remove_var("AGENTMEM_DB_PATH");
        env::remove_var("POSTGRES_URL");

        assert!(!has_database_config());

        env::set_var("DATABASE_URL", "test");
        assert!(has_database_config());

        env::remove_var("DATABASE_URL");
    }
}

