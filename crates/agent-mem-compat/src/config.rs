//! Configuration for Mem0 compatibility layer

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for Mem0 compatibility client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mem0Config {
    /// Vector store configuration
    pub vector_store: VectorStoreConfig,

    /// LLM configuration
    pub llm: LlmConfig,

    /// Embedding configuration
    pub embedder: EmbedderConfig,

    /// Memory configuration
    #[serde(default)]
    pub memory: MemoryConfig,

    /// Custom configuration
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// Provider name (e.g., "chroma", "pinecone", "qdrant")
    pub provider: String,

    /// Configuration specific to the provider
    pub config: HashMap<String, serde_json::Value>,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider name (e.g., "openai", "anthropic", "ollama")
    pub provider: String,

    /// Model name
    pub model: String,

    /// Configuration specific to the provider
    pub config: HashMap<String, serde_json::Value>,
}

/// Embedder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedderConfig {
    /// Provider name (e.g., "openai", "huggingface", "local")
    pub provider: String,

    /// Model name
    pub model: String,

    /// Configuration specific to the provider
    pub config: HashMap<String, serde_json::Value>,
}

/// Memory-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Default memory type for new memories
    #[serde(default = "default_memory_type")]
    pub default_type: String,

    /// Enable automatic memory consolidation
    #[serde(default = "default_true")]
    pub auto_consolidation: bool,

    /// Enable importance scoring
    #[serde(default = "default_true")]
    pub importance_scoring: bool,

    /// Maximum number of memories to return in search
    #[serde(default = "default_search_limit")]
    pub default_search_limit: usize,

    /// Memory retention policy
    #[serde(default)]
    pub retention: RetentionConfig,
}

/// Memory retention configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetentionConfig {
    /// Maximum age of memories in days (None = no limit)
    pub max_age_days: Option<u32>,

    /// Maximum number of memories per user (None = no limit)
    pub max_memories_per_user: Option<usize>,

    /// Minimum importance score to retain (None = no limit)
    pub min_importance: Option<f32>,
}

impl Default for Mem0Config {
    fn default() -> Self {
        Self {
            vector_store: VectorStoreConfig {
                provider: "chroma".to_string(),
                config: HashMap::new(),
            },
            llm: LlmConfig {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                config: HashMap::new(),
            },
            embedder: EmbedderConfig {
                provider: "openai".to_string(),
                model: "text-embedding-ada-002".to_string(),
                config: HashMap::new(),
            },
            memory: MemoryConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            default_type: default_memory_type(),
            auto_consolidation: default_true(),
            importance_scoring: default_true(),
            default_search_limit: default_search_limit(),
            retention: RetentionConfig::default(),
        }
    }
}

fn default_memory_type() -> String {
    "episodic".to_string()
}

fn default_true() -> bool {
    true
}

fn default_search_limit() -> usize {
    10
}

impl Mem0Config {
    /// Create a new configuration with OpenAI defaults
    pub fn openai() -> Self {
        let mut config = Self::default();

        // Set OpenAI-specific defaults
        config.llm.config.insert(
            "api_key".to_string(),
            serde_json::Value::String(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        );

        config.embedder.config.insert(
            "api_key".to_string(),
            serde_json::Value::String(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        );

        config
    }

    /// Create a new configuration with Anthropic defaults
    pub fn anthropic() -> Self {
        let mut config = Self::default();

        config.llm.provider = "anthropic".to_string();
        config.llm.model = "claude-3-sonnet-20240229".to_string();
        config.llm.config.insert(
            "api_key".to_string(),
            serde_json::Value::String(std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()),
        );

        config
    }

    /// Create a new configuration with local/offline defaults
    pub fn local() -> Self {
        let mut config = Self::default();

        config.llm.provider = "ollama".to_string();
        config.llm.model = "llama2".to_string();
        config.embedder.provider = "local".to_string();
        config.embedder.model = "all-MiniLM-L6-v2".to_string();

        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), crate::error::Mem0Error> {
        // Validate vector store config
        if self.vector_store.provider.is_empty() {
            return Err(crate::error::Mem0Error::ConfigError {
                message: "Vector store provider cannot be empty".to_string(),
            });
        }

        // Validate LLM config
        if self.llm.provider.is_empty() || self.llm.model.is_empty() {
            return Err(crate::error::Mem0Error::ConfigError {
                message: "LLM provider and model cannot be empty".to_string(),
            });
        }

        // Validate embedder config
        if self.embedder.provider.is_empty() || self.embedder.model.is_empty() {
            return Err(crate::error::Mem0Error::ConfigError {
                message: "Embedder provider and model cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}
