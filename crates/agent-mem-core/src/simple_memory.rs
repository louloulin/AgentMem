//! Simplified Memory API (Mem0-style)
//!
//! This module provides a simplified, user-friendly API for AgentMem,
//! similar to Mem0's interface. It automatically handles configuration,
//! initialization, and intelligent features.
//!
//! # Example
//!
//! ```rust,no_run
//! use agent_mem_core::SimpleMemory;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simple initialization
//!     let mem = SimpleMemory::new().await?;
//!     
//!     // Add memory
//!     let id = mem.add("I love pizza").await?;
//!     
//!     // Search memories
//!     let results = mem.search("What do you know about me?").await?;
//!     
//!     Ok(())
//! }
//! ```

use crate::manager::MemoryManager;
use crate::types::{Memory, MemoryQuery, MemorySearchResult};
use agent_mem_config::memory::{
    DecisionEngineConfig, DeduplicationConfig, FactExtractionConfig, IntelligenceConfig,
    EmbedderConfig, PerformanceConfig, SessionConfig,
};
use agent_mem_config::MemoryConfig;
use agent_mem_traits::{
    AgentMemError, DecisionEngine, FactExtractor, LLMConfig, MemoryItem, MemoryType, Result,
    VectorStoreConfig,
};
use agent_mem_llm::LLMProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Simplified Memory interface (Mem0-style)
///
/// This struct provides a simple, user-friendly API for memory management,
/// automatically handling configuration and intelligent features.
///
/// **Important**: SimpleMemory uses in-memory storage by default, which means
/// data is lost when the process exits. For production use with persistent storage,
/// use the Agent-based API instead:
///
/// ```rust,no_run
/// use agent_mem_core::agents::CoreAgent;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Production: Uses persistent storage (LibSQL by default)
///     let agent = CoreAgent::from_env("agent1".to_string()).await?;
///
///     // Development: Uses in-memory storage
///     let mem = agent_mem_core::SimpleMemory::new().await?;
///     Ok(())
/// }
/// ```
pub struct SimpleMemory {
    /// Internal memory manager
    manager: Arc<MemoryManager>,
    /// Default user ID
    default_user_id: Option<String>,
    /// Default agent ID
    default_agent_id: String,
}

impl SimpleMemory {
    /// Create a new SimpleMemory with automatic configuration
    ///
    /// **Note**: This uses in-memory storage which is not persistent.
    /// Data will be lost when the process exits.
    ///
    /// For production use with persistent storage, use the Agent-based API:
    /// - `CoreAgent::from_env()` - Persistent core memory
    /// - `EpisodicAgent::from_env()` - Persistent episodic memory
    /// - `SemanticAgent::from_env()` - Persistent semantic memory
    ///
    /// This method automatically:
    /// - Creates an in-memory storage backend
    /// - Enables intelligent features by default
    /// - Suitable for development and testing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Development/Testing: In-memory storage
    ///     let mem = SimpleMemory::new().await?;
    ///
    ///     // Add memory (will be lost on exit)
    ///     let id = mem.add("I love pizza").await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self> {
        info!("Initializing SimpleMemory with in-memory storage (development mode)");
        info!("For production use with persistent storage, use Agent::from_env() instead");

        // Create configuration
        let config = Self::create_config()?;

        // Create memory manager without intelligent components
        // For intelligent features, use `with_intelligence()` instead
        let manager = MemoryManager::with_config(config);

        info!("SimpleMemory initialized successfully");

        Ok(Self {
            manager: Arc::new(manager),
            default_user_id: None,
            default_agent_id: "default".to_string(),
        })
    }

    /// Create a SimpleMemory instance with intelligent components
    ///
    /// # Arguments
    /// * `fact_extractor` - Optional fact extraction component
    /// * `decision_engine` - Optional decision making component
    /// * `llm_provider` - Optional LLM provider for intelligent features
    ///
    /// # Example
    /// ```ignore
    /// use agent_mem_core::SimpleMemory;
    /// use agent_mem_intelligence::{FactExtractor, MemoryDecisionEngine};
    /// use agent_mem_llm::providers::OpenAIProvider;
    ///
    /// let llm = Arc::new(OpenAIProvider::new(config)?);
    /// let fact_extractor = Arc::new(FactExtractor::new(llm.clone()));
    /// let decision_engine = Arc::new(MemoryDecisionEngine::new(llm.clone()));
    ///
    /// let mem = SimpleMemory::with_intelligence(
    ///     Some(fact_extractor),
    ///     Some(decision_engine),
    ///     Some(llm),
    /// ).await?;
    /// ```
    pub async fn with_intelligence(
        fact_extractor: Option<Arc<dyn FactExtractor>>,
        decision_engine: Option<Arc<dyn DecisionEngine>>,
        llm_provider: Option<Arc<dyn LLMProvider>>,
    ) -> Result<Self> {
        info!("Initializing SimpleMemory with intelligent components");

        let config = Self::create_config()?;
        let manager = MemoryManager::with_intelligent_components(
            config,
            fact_extractor,
            decision_engine,
            llm_provider,
        );

        info!("SimpleMemory with intelligence initialized successfully");

        Ok(Self {
            manager: Arc::new(manager),
            default_user_id: None,
            default_agent_id: "default".to_string(),
        })
    }

    /// Create a new SimpleMemory with custom configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    /// use agent_mem_config::MemoryConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = MemoryConfig::default();
    ///     let mem = SimpleMemory::with_config(config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_config(config: MemoryConfig) -> Result<Self> {
        info!("Initializing SimpleMemory with custom configuration");

        // Create memory manager without intelligent components
        let manager = MemoryManager::with_config(config);

        Ok(Self {
            manager: Arc::new(manager),
            default_user_id: None,
            default_agent_id: "default".to_string(),
        })
    }

    /// Create a SimpleMemory instance with custom configuration and intelligent components
    ///
    /// # Arguments
    /// * `config` - Memory configuration
    /// * `fact_extractor` - Optional fact extraction component
    /// * `decision_engine` - Optional decision making component
    /// * `llm_provider` - Optional LLM provider for intelligent features
    pub async fn with_config_and_intelligence(
        config: MemoryConfig,
        fact_extractor: Option<Arc<dyn FactExtractor>>,
        decision_engine: Option<Arc<dyn DecisionEngine>>,
        llm_provider: Option<Arc<dyn LLMProvider>>,
    ) -> Result<Self> {
        info!("Initializing SimpleMemory with custom configuration and intelligent components");

        let manager = MemoryManager::with_intelligent_components(
            config,
            fact_extractor,
            decision_engine,
            llm_provider,
        );

        Ok(Self {
            manager: Arc::new(manager),
            default_user_id: None,
            default_agent_id: "default".to_string(),
        })
    }

    /// Set the default user ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?.with_user("alice");
    ///     Ok(())
    /// }
    /// ```
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.default_user_id = Some(user_id.into());
        self
    }

    /// Set the default agent ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?.with_agent("my-agent");
    ///     Ok(())
    /// }
    /// ```
    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.default_agent_id = agent_id.into();
        self
    }

    /// Add a memory
    ///
    /// This method automatically:
    /// - Extracts facts from the content using LLM
    /// - Finds similar existing memories
    /// - Decides whether to ADD/UPDATE/DELETE/MERGE
    /// - Executes the appropriate action
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     let id = mem.add("I love pizza").await?;
    ///     println!("Memory added with ID: {}", id);
    ///     Ok(())
    /// }
    /// ```
    pub async fn add(&self, content: impl Into<String>) -> Result<String> {
        self.add_with_metadata(content, None).await
    }

    /// Add a memory with metadata
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    /// use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     
    ///     let mut metadata = HashMap::new();
    ///     metadata.insert("category".to_string(), "food".to_string());
    ///     
    ///     let id = mem.add_with_metadata("I love pizza", Some(metadata)).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_with_metadata(
        &self,
        content: impl Into<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        debug!("Adding memory with intelligent processing");
        
        self.manager
            .add_memory(
                self.default_agent_id.clone(),
                self.default_user_id.clone(),
                content.into(),
                None, // Auto-infer memory type
                None, // Auto-calculate importance
                metadata,
            )
            .await
    }

    /// Search memories
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     let results = mem.search("What do you know about me?").await?;
    ///
    ///     for memory in results {
    ///         println!("Memory: {}", memory.content);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn search(&self, query: impl Into<String>) -> Result<Vec<MemoryItem>> {
        self.search_with_limit(query, 10).await
    }

    /// Search memories with custom limit
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     let results = mem.search_with_limit("pizza", 5).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_with_limit(
        &self,
        query: impl Into<String>,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        debug!("Searching memories with query");

        let mut query_obj = MemoryQuery::new(self.default_agent_id.clone());
        query_obj.text_query = Some(query.into());
        query_obj.limit = limit;

        if let Some(user_id) = &self.default_user_id {
            query_obj = query_obj.with_user_id(user_id.clone());
        }

        let results = self.manager.search_memories(query_obj).await?;

        // Convert MemorySearchResult to MemoryItem
        Ok(results
            .into_iter()
            .map(|r| MemoryItem::from(r.memory))
            .collect())
    }

    /// Get all memories for the current user/agent
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     let all_memories = mem.get_all().await?;
    ///     println!("Total memories: {}", all_memories.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_all(&self) -> Result<Vec<MemoryItem>> {
        debug!("Getting all memories");

        // Use search with empty query to get all
        let mut query_obj = MemoryQuery::new(self.default_agent_id.clone());
        query_obj.limit = 1000; // Large limit to get all

        if let Some(user_id) = &self.default_user_id {
            query_obj = query_obj.with_user_id(user_id.clone());
        }

        let results = self.manager.search_memories(query_obj).await?;

        Ok(results
            .into_iter()
            .map(|r| MemoryItem::from(r.memory))
            .collect())
    }

    /// Update a memory
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     mem.update("mem_123", "I love Rust programming").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn update(&self, memory_id: impl Into<String>, content: impl Into<String>) -> Result<()> {
        debug!("Updating memory");

        self.manager
            .update_memory(&memory_id.into(), Some(content.into()), None, None)
            .await
    }

    /// Delete a memory
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     mem.delete("mem_123").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete(&self, memory_id: impl Into<String>) -> Result<()> {
        debug!("Deleting memory");

        self.manager.delete_memory(&memory_id.into()).await?;
        Ok(())
    }

    /// Delete all memories for the current user/agent
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use agent_mem_core::SimpleMemory;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mem = SimpleMemory::new().await?;
    ///     mem.delete_all().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn delete_all(&self) -> Result<()> {
        debug!("Deleting all memories");

        // Get all memories and delete them one by one
        let all_memories = self.get_all().await?;

        for memory in all_memories {
            self.manager.delete_memory(&memory.id).await?;
        }

        Ok(())
    }

    // Helper methods

    /// Create configuration with optional intelligent features
    fn create_config() -> Result<MemoryConfig> {
        Ok(MemoryConfig {
            llm: LLMConfig::default(),
            vector_store: VectorStoreConfig::default(),
            graph_store: None,
            embedder: EmbedderConfig::default(),
            session: SessionConfig::default(),
            intelligence: IntelligenceConfig {
                similarity_threshold: 0.8,
                clustering_threshold: 0.7,
                enable_conflict_detection: true,
                enable_memory_summarization: true,
                importance_scoring: true,
                enable_intelligent_extraction: false,  // Disabled by default, enable via with_intelligence()
                enable_decision_engine: false,         // Disabled by default, enable via with_intelligence()
                enable_deduplication: false,          // Optional
                fact_extraction: FactExtractionConfig {
                    min_confidence: 0.7,
                    extract_entities: true,
                    extract_relations: true,
                    max_facts_per_message: 10,
                },
                decision_engine: DecisionEngineConfig {
                    similarity_threshold: 0.85,
                    min_decision_confidence: 0.6,
                    enable_intelligent_merge: true,
                    max_similar_memories: 5,
                },
                deduplication: DeduplicationConfig {
                    similarity_threshold: 0.9,
                    time_window_seconds: Some(3600),
                    merge_strategy: "intelligent_merge".to_string(),
                },
            },
            performance: PerformanceConfig::default(),
        })
    }
}

impl Default for SimpleMemory {
    fn default() -> Self {
        // Note: This will panic if initialization fails
        // Users should use SimpleMemory::new().await instead
        panic!("Use SimpleMemory::new().await instead of Default::default()")
    }
}

