//! Orchestrator Module - 记忆编排器模块
//!
//! 将原来的orchestrator.rs拆分为多个模块，保持高内聚低耦合

pub mod batch;
pub mod core;
pub mod initialization;
pub mod intelligence;
pub mod intelligence_tests;
pub mod multimodal;
pub mod multimodal_tests;
pub mod retrieval;
pub mod retrieval_tests;
pub mod storage;
pub mod utils;

// 重新导出核心类型
pub use batch::BatchModule;
pub use core::{CompletedOperation, MemoryOrchestrator, OrchestratorConfig};
pub use initialization::{InitializationModule, IntelligenceComponents};
pub use intelligence::IntelligenceModule;
pub use multimodal::MultimodalModule;
pub use retrieval::RetrievalModule;
pub use storage::StorageModule;
pub use utils::UtilsModule;

#[cfg(test)]
mod tests;
