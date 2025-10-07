//! Memory Operations Benchmarks
//!
//! 基准测试 AgentMem 的核心记忆操作性能
//!
//! 测试目标:
//! - 记忆创建: < 5ms
//! - 记忆检索: < 3ms
//! - 语义搜索: < 25ms
//! - 批量操作: < 100ms (100条记忆)

use agent_mem_core::{
    manager::MemoryManager,
    types::{Memory, MemoryType},
};
use agent_mem_traits::Vector;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// 创建测试用的 MemoryManager
fn create_test_manager() -> MemoryManager {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        MemoryManager::new_in_memory().await.unwrap()
    })
}

/// 创建测试记忆
fn create_test_memory(id: usize) -> Memory {
    Memory {
        id: format!("test_memory_{}", id),
        agent_id: "test_agent".to_string(),
        user_id: Some("test_user".to_string()),
        memory_type: MemoryType::Episodic,
        content: format!("This is test memory number {}", id),
        embedding: Some(vec![0.1; 384]), // 模拟 embedding
        score: Some(0.8),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        accessed_at: chrono::Utc::now(),
        access_count: 0,
    }
}

/// 基准测试: 记忆创建
fn bench_memory_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = create_test_manager();

    c.bench_function("memory_creation", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            rt.block_on(async {
                let memory_id = manager
                    .add_memory(
                        black_box("test_agent".to_string()),
                        black_box(Some("test_user".to_string())),
                        black_box(format!("Test memory content {}", counter)),
                        black_box(Some(MemoryType::Episodic)),
                        black_box(Some(0.8)),
                        black_box(None),
                    )
                    .await
                    .unwrap();
                black_box(memory_id)
            })
        });
    });
}

/// 基准测试: 记忆检索
fn bench_memory_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = create_test_manager();

    // 预先创建一些记忆
    let memory_ids: Vec<String> = rt.block_on(async {
        let mut ids = Vec::new();
        for i in 0..100 {
            let id = manager
                .add_memory(
                    "test_agent".to_string(),
                    Some("test_user".to_string()),
                    format!("Test memory {}", i),
                    Some(MemoryType::Episodic),
                    Some(0.8),
                    None,
                )
                .await
                .unwrap();
            ids.push(id);
        }
        ids
    });

    c.bench_function("memory_retrieval", |b| {
        let mut index = 0;
        b.iter(|| {
            let memory_id = &memory_ids[index % memory_ids.len()];
            index += 1;
            rt.block_on(async {
                let memory = manager.get_memory(black_box(memory_id)).await.unwrap();
                black_box(memory)
            })
        });
    });
}

/// 基准测试: 记忆搜索
fn bench_memory_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = create_test_manager();

    // 预先创建一些记忆
    rt.block_on(async {
        for i in 0..100 {
            manager
                .add_memory(
                    "test_agent".to_string(),
                    Some("test_user".to_string()),
                    format!("Test memory about topic {}", i % 10),
                    Some(MemoryType::Episodic),
                    Some(0.8),
                    None,
                )
                .await
                .unwrap();
        }
    });

    c.bench_function("memory_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let results = manager
                    .search_memories(
                        black_box("topic"),
                        black_box(Some("test_agent")),
                        black_box(Some(10)),
                        black_box(Some(0.5)),
                    )
                    .await
                    .unwrap();
                black_box(results)
            })
        });
    });
}

/// 基准测试: 批量记忆创建
fn bench_batch_memory_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_memory_creation");
    
    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let manager = create_test_manager();
                    rt.block_on(async {
                        let mut ids = Vec::new();
                        for i in 0..size {
                            let id = manager
                                .add_memory(
                                    black_box("test_agent".to_string()),
                                    black_box(Some("test_user".to_string())),
                                    black_box(format!("Batch memory {}", i)),
                                    black_box(Some(MemoryType::Episodic)),
                                    black_box(Some(0.8)),
                                    black_box(None),
                                )
                                .await
                                .unwrap();
                            ids.push(id);
                        }
                        black_box(ids)
                    })
                });
            },
        );
    }
    group.finish();
}

/// 基准测试: 记忆更新
fn bench_memory_update(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = create_test_manager();

    // 预先创建一个记忆
    let memory_id = rt.block_on(async {
        manager
            .add_memory(
                "test_agent".to_string(),
                Some("test_user".to_string()),
                "Original content".to_string(),
                Some(MemoryType::Episodic),
                Some(0.8),
                None,
            )
            .await
            .unwrap()
    });

    c.bench_function("memory_update", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            rt.block_on(async {
                let mut memory = manager.get_memory(&memory_id).await.unwrap().unwrap();
                memory.content = black_box(format!("Updated content {}", counter));
                manager.update_memory(black_box(memory)).await.unwrap();
            })
        });
    });
}

/// 基准测试: 记忆删除
fn bench_memory_deletion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memory_deletion", |b| {
        b.iter(|| {
            let manager = create_test_manager();
            rt.block_on(async {
                // 创建一个记忆
                let memory_id = manager
                    .add_memory(
                        "test_agent".to_string(),
                        Some("test_user".to_string()),
                        "To be deleted".to_string(),
                        Some(MemoryType::Episodic),
                        Some(0.8),
                        None,
                    )
                    .await
                    .unwrap();
                
                // 删除它
                manager.delete_memory(black_box(&memory_id)).await.unwrap();
            })
        });
    });
}

/// 基准测试: 按类型列出记忆
fn bench_list_memories_by_type(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = create_test_manager();

    // 预先创建不同类型的记忆
    rt.block_on(async {
        for memory_type in [
            MemoryType::Episodic,
            MemoryType::Semantic,
            MemoryType::Procedural,
            MemoryType::Working,
        ] {
            for i in 0..25 {
                manager
                    .add_memory(
                        "test_agent".to_string(),
                        Some("test_user".to_string()),
                        format!("Memory of type {:?} - {}", memory_type, i),
                        Some(memory_type),
                        Some(0.8),
                        None,
                    )
                    .await
                    .unwrap();
            }
        }
    });

    c.bench_function("list_memories_by_type", |b| {
        b.iter(|| {
            rt.block_on(async {
                let memories = manager
                    .list_memories(
                        black_box(Some("test_agent")),
                        black_box(Some("test_user")),
                        black_box(Some(MemoryType::Episodic)),
                        black_box(Some(50)),
                    )
                    .await
                    .unwrap();
                black_box(memories)
            })
        });
    });
}

criterion_group!(
    benches,
    bench_memory_creation,
    bench_memory_retrieval,
    bench_memory_search,
    bench_batch_memory_creation,
    bench_memory_update,
    bench_memory_deletion,
    bench_list_memories_by_type,
);

criterion_main!(benches);

