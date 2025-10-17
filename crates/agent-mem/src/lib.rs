//! # AgentMem - 统一记忆管理 API
//!
//! AgentMem 是一个极简易用的 AI Agent 记忆管理系统，提供统一的 API 接口，
//! 支持多种记忆类型、智能功能和存储后端。
//!
//! ## 核心特性
//!
//! - **极简易用**: 一行代码初始化，开箱即用
//! - **智能功能**: 自动事实提取、决策引擎、记忆去重
//! - **功能完整**: 对话、可视化、备份恢复、用户管理
//! - **性能卓越**: Rust 实现，超越 Python 性能
//! - **灵活配置**: 支持零配置到完整配置的渐进式复杂度
//!
//! ## 快速开始
//!
//! ### 零配置模式
//!
//! ```rust,no_run
//! use agent_mem::Memory;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 零配置初始化
//!     let mem = Memory::new().await?;
//!     
//!     // 添加记忆
//!     mem.add("I love pizza").await?;
//!     
//!     // 搜索记忆
//!     let results = mem.search("What do you know about me?").await?;
//!     for result in results {
//!         println!("- {}", result.content);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Builder 模式
//!
//! ```rust,no_run
//! use agent_mem::Memory;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mem = Memory::builder()
//!         .with_storage("libsql://agentmem.db")
//!         .with_llm("openai", "gpt-4")
//!         .with_embedder("openai", "text-embedding-3-small")
//!         .enable_intelligent_features()
//!         .build()
//!         .await?;
//!     
//!     mem.add("I love pizza").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## 架构设计
//!
//! ```text
//! Memory (统一 API)
//!     ↓
//! MemoryOrchestrator (智能编排)
//!     ↓
//! 8 个专门 Agents (CoreAgent, EpisodicAgent, etc.)
//!     ↓
//! Storage Layer (LibSQL, PostgreSQL, etc.)
//! ```

pub mod auto_config;
pub mod builder;
pub mod chat;
pub mod memory;
pub mod orchestrator;
pub mod types;
pub mod visualization;

// 重新导出核心类型
pub use builder::MemoryBuilder;
pub use memory::Memory;
pub use types::{
    AddMemoryOptions, MemoryStats, SearchOptions,
    // TODO: 在任务 2.1 和 2.2 中导出这些类型
    // ChatOptions, MemoryVisualization,
};

// 重新导出 traits 中的常用类型
pub use agent_mem_traits::{
    AgentMemError, MemoryItem, MemoryType, Result,
};

// 重新导出 core 中的 Agent 类型（用于高级用户）
pub use agent_mem_core::{
    CoreAgent, EpisodicAgent, SemanticAgent, ProceduralAgent,
    ResourceAgent, WorkingAgent, KnowledgeAgent, ContextualAgent,
};

