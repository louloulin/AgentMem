//! UnifiedStorageCoordinator 集成示例
//!
//! 展示如何将UnifiedStorageCoordinator集成到现有代码中
//!
//! 这个文件仅作为示例，展示集成方式，不参与实际编译

use agent_mem_core::storage::coordinator::{CacheConfig, UnifiedStorageCoordinator};
use agent_mem_core::storage::factory::Repositories;
use agent_mem_traits::{MemoryV4 as Memory, VectorStore};
use std::sync::Arc;

/// 示例：如何在server中初始化UnifiedStorageCoordinator
///
/// 这个函数展示了如何创建和配置UnifiedStorageCoordinator
#[allow(dead_code)]
pub async fn create_coordinator_example(
    repositories: Arc<Repositories>,
    vector_store: Arc<dyn VectorStore + Send + Sync>,
) -> Arc<UnifiedStorageCoordinator> {
    // 创建coordinator
    let coordinator = UnifiedStorageCoordinator::new(
        repositories.memories.clone(),
        vector_store,
        Some(CacheConfig::default()),
    );

    Arc::new(coordinator)
}

/// 示例：在delete_memory中使用coordinator
///
/// 替换现有的双重删除逻辑
#[allow(dead_code)]
pub async fn delete_memory_with_coordinator_example(
    coordinator: Arc<UnifiedStorageCoordinator>,
    id: &str,
) -> Result<(), String> {
    coordinator
        .delete_memory(id)
        .await
        .map_err(|e| e.to_string())
}

/// 示例：在add_memory中使用coordinator
///
/// 需要先获取embedding，然后调用coordinator
#[allow(dead_code)]
pub async fn add_memory_with_coordinator_example(
    coordinator: Arc<UnifiedStorageCoordinator>,
    memory: &Memory,
    embedding: Option<Vec<f32>>,
) -> Result<String, String> {
    coordinator
        .add_memory(memory, embedding)
        .await
        .map_err(|e| e.to_string())
}

/// 示例：在get_memory中使用coordinator（带缓存）
#[allow(dead_code)]
pub async fn get_memory_with_coordinator_example(
    coordinator: Arc<UnifiedStorageCoordinator>,
    id: &str,
) -> Result<Option<Memory>, String> {
    coordinator
        .get_memory(id)
        .await
        .map_err(|e| e.to_string())
}

