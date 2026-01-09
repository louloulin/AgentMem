//! AgentMem Working Memory Service
//!
//! Fast in-memory temporary context storage for conversations.
//!
//! # Features
//!
//! - Session-based memory isolation
//! - Priority-based retrieval
//! - Automatic expiration cleanup
//! - EventBus integration for event notifications
//! - High-performance concurrent access (DashMap)
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_working_memory::{WorkingMemoryService, WorkingMemoryConfig};
//! use agent_mem_traits::WorkingMemoryItem;
//! use chrono::Utc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create service with default config
//!     let service = WorkingMemoryService::new(WorkingMemoryConfig::default()).await?;
//!
//!     // Add item to working memory
//!     let item = WorkingMemoryItem {
//!         id: "item-1".to_string(),
//!         session_id: "session-123".to_string(),
//!         content: "User prefers concise answers".to_string(),
//!         priority: 5,
//!         expires_at: None,
//!         created_at: Utc::now(),
//!         user_id: "user-1".to_string(),
//!         agent_id: "agent-1".to_string(),
//!         metadata: serde_json::json!({}),
//!     };
//!
//!     service.add_item(item).await?;
//!
//!     // Get all session items
//!     let items = service.get_session_items("session-123").await?;
//!     println!("Found {} items", items.len());
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod service;

pub use config::WorkingMemoryConfig;
pub use service::WorkingMemoryService;

// Re-exports from agent-mem-traits
pub use agent_mem_traits::WorkingMemoryItem;

use agent_mem_traits::Result;

/// Default capacity for working memory (items per session)
pub const DEFAULT_CAPACITY: usize = 100;

/// Default TTL for working memory items (5 minutes)
pub const DEFAULT_TTL_SECONDS: i64 = 300;

/// Default cleanup interval (1 minute)
pub const DEFAULT_CLEANUP_INTERVAL_SECONDS: u64 = 60;
