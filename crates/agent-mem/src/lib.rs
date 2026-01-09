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
//! - **Memory V4 架构**: 支持多模态内容、开放属性、关系图谱
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
//!
//! ## Memory V4 架构
//!
//! AgentMem 4.0 引入了全新的 Memory V4 架构，提供更强大和灵活的记忆管理能力：
//!
//! - **多模态内容**: 支持文本、结构化数据、向量、二进制等多种内容类型
//! - **开放属性系统**: 使用命名空间的键值对，支持任意元数据
//! - **关系图谱**: 内置关系管理，支持复杂的记忆网络
//! - **强类型查询**: 使用 Query V4 进行语义化查询
//!
//! ### 迁移指南
//!
//! 如果您正在使用旧的 `MemoryItem` API，建议迁移到 Memory V4：
//!
//! ```rust,ignore
//! // 旧 API (已废弃)
//! use agent_mem::MemoryItem;
//!
//! // 新 API (推荐)
//! use agent_mem::MemoryV4;
//! ```
//!
//! 详细迁移指南请参见：`docs/migration/v3_to_v4.md`

pub mod api_simplification;
pub mod auto_config;
pub mod builder;
pub mod chat;
pub mod batch;
pub mod history;
pub mod memory;
pub mod orchestrator;
pub mod search;
pub mod types;
pub mod visualization;

// 重新导出核心类型
pub use api_simplification::{EnhancedError, ErrorEnhancer, FluentMemory, SmartDefaults};
pub use batch::BatchBuilder;
pub use builder::MemoryBuilder;
pub use memory::Memory;
pub use search::{SearchBuilder, SearchOptions};
pub use types::{
    AddMemoryOptions, AddResult, DeleteAllOptions, GetAllOptions, MemoryEvent, MemoryScope,
    MemoryStats, RelationEvent, SearchOptions,
};

// 重新导出 traits 中的常用类型
pub use agent_mem_traits::{AgentMemError, Result};

// 重新导出 Memory V4 类型（推荐使用）
pub use agent_mem_traits::abstractions::{
    AttributeKey, AttributeSet, AttributeValue, Content, Memory as MemoryV4, Metadata, Query,
    QueryIntent, RelationGraph,
};

// Legacy 类型（已废弃，仅用于向后兼容）
#[allow(deprecated)]
pub use agent_mem_traits::{MemoryItem, MemoryType};

// 重新导出 core 中的 Agent 类型（用于高级用户）
pub use agent_mem_core::{
    ContextualAgent, CoreAgent, EpisodicAgent, KnowledgeAgent, ProceduralAgent, ResourceAgent,
    SemanticAgent, WorkingAgent,
};

// 插件系统（可选功能）
#[cfg(feature = "plugins")]
pub use agent_mem_plugins as plugins;

// 插件集成层
pub mod plugin_integration;
#[cfg(feature = "plugins")]
pub use plugin_integration::{PluginEnhancedMemory, PluginHooks};
