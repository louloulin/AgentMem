//! Orchestrator Module - 记忆编排器模块
//!
//! 将原来的orchestrator.rs拆分为多个模块，保持高内聚低耦合

pub mod core;
pub mod utils;
pub mod initialization;
pub mod storage;
pub mod retrieval;
pub mod intelligence;
pub mod multimodal;
pub mod batch;

// 重新导出核心类型
pub use core::{MemoryOrchestrator, OrchestratorConfig, CompletedOperation};
pub use utils::UtilsModule;
pub use initialization::{InitializationModule, IntelligenceComponents};
pub use storage::StorageModule;
pub use retrieval::RetrievalModule;
pub use intelligence::IntelligenceModule;
pub use multimodal::MultimodalModule;
pub use batch::BatchModule;

#[cfg(test)]
mod tests;

