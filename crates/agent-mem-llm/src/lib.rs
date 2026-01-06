//! # Agent Memory LLM Integration
//!
//! LLM集成模块，为AgentMem记忆平台提供多种LLM提供商支持。
//!
//! 本模块提供：
//! - LLM工厂模式，支持多种提供商
//! - 统一的LLM接口抽象
//! - 提示词管理系统
//! - 错误处理和重试机制
//! - 特性门控支持

pub mod cache;
pub mod client;
pub mod factory;
pub mod metrics;
pub mod prompts;
pub mod providers;
pub mod retry;

pub use cache::{CacheStats, CachedResult, LLMCache};
pub use client::LLMClient;
pub use factory::LLMFactory;
pub use metrics::{LLMMetrics, LLMMonitor, LLMStats};
pub use retry::{ErrorType, RetryConfig, RetryExecutor};

// 重新导出常用类型
pub use agent_mem_traits::{AgentMemError, LLMConfig, LLMProvider, Message, ModelInfo, Result};

// 导出测试提供商（用于其他 crate 的测试）
pub use providers::LocalTestProvider;
