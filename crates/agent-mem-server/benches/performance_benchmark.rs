//! AgentMem 性能基准测试
//!
//! 使用 Criterion 进行性能基准测试
//!
//! 运行方式:
//! ```bash
//! cargo bench --package agent-mem-server
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::time::Duration;

// ============================================================================
// 数据结构性能测试
// ============================================================================

/// 测试 JSON 序列化性能
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    // 小型对象
    group.bench_function("small_object", |b| {
        let data = json!({
            "id": "test-123",
            "name": "Test Agent",
            "state": "active"
        });
        b.iter(|| black_box(serde_json::to_string(&data).unwrap()));
    });

    // 中型对象
    group.bench_function("medium_object", |b| {
        let data = json!({
            "id": "test-123",
            "name": "Test Agent",
            "description": "A test agent for benchmarking",
            "organization_id": "org-456",
            "state": "active",
            "config": {
                "llm_provider": "openai",
                "llm_model": "gpt-4",
                "temperature": 0.7,
                "max_tokens": 1000
            },
            "metadata": {
                "created_at": "2025-10-15T00:00:00Z",
                "updated_at": "2025-10-15T00:00:00Z",
                "tags": ["test", "benchmark", "performance"]
            }
        });
        b.iter(|| black_box(serde_json::to_string(&data).unwrap()));
    });

    // 大型对象（包含数组）
    group.bench_function("large_object", |b| {
        let memories: Vec<_> = (0..100)
            .map(|i| {
                json!({
                    "id": format!("mem-{}", i),
                    "content": format!("Memory content {}", i),
                    "importance": 0.5 + (i as f64 / 200.0),
                    "memory_type": "episodic"
                })
            })
            .collect();

        let data = json!({
            "agent_id": "test-123",
            "memories": memories,
            "total": 100
        });

        b.iter(|| black_box(serde_json::to_string(&data).unwrap()));
    });

    group.finish();
}

/// 测试 JSON 反序列化性能
fn bench_json_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_deserialization");

    // 小型对象
    group.bench_function("small_object", |b| {
        let json_str = r#"{"id":"test-123","name":"Test Agent","state":"active"}"#;
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(json_str).unwrap()));
    });

    // 中型对象
    group.bench_function("medium_object", |b| {
        let json_str = r#"{
            "id":"test-123",
            "name":"Test Agent",
            "description":"A test agent",
            "organization_id":"org-456",
            "state":"active",
            "config":{"llm_provider":"openai","llm_model":"gpt-4"}
        }"#;
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(json_str).unwrap()));
    });

    group.finish();
}

// ============================================================================
// 字符串操作性能测试
// ============================================================================

/// 测试字符串操作性能
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // 字符串拼接
    group.bench_function("string_concatenation", |b| {
        b.iter(|| {
            let mut result = String::new();
            for i in 0..100 {
                result.push_str(&format!("item-{} ", i));
            }
            black_box(result)
        });
    });

    // 字符串格式化
    group.bench_function("string_formatting", |b| {
        b.iter(|| {
            black_box(format!(
                "Agent ID: {}, Name: {}, State: {}",
                "test-123", "Test Agent", "active"
            ))
        });
    });

    // 字符串搜索
    group.bench_function("string_search", |b| {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);
        b.iter(|| black_box(text.contains("fox")));
    });

    group.finish();
}

// ============================================================================
// 集合操作性能测试
// ============================================================================

/// 测试 Vec 操作性能
fn bench_vec_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_operations");

    // Vec 创建和填充
    group.bench_function("vec_creation", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(i);
            }
            black_box(vec)
        });
    });

    // Vec 迭代
    group.bench_function("vec_iteration", |b| {
        let vec: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let sum: i32 = vec.iter().sum();
            black_box(sum)
        });
    });

    // Vec 过滤
    group.bench_function("vec_filter", |b| {
        let vec: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let filtered: Vec<_> = vec.iter().filter(|&&x| x % 2 == 0).collect();
            black_box(filtered)
        });
    });

    group.finish();
}

// ============================================================================
// 内存分配性能测试
// ============================================================================

/// 测试内存分配性能
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // 小对象分配
    group.bench_function("small_allocation", |b| {
        b.iter(|| {
            let data = vec![0u8; 100];
            black_box(data)
        });
    });

    // 中等对象分配
    group.bench_function("medium_allocation", |b| {
        b.iter(|| {
            let data = vec![0u8; 10_000];
            black_box(data)
        });
    });

    // 大对象分配
    group.bench_function("large_allocation", |b| {
        b.iter(|| {
            let data = vec![0u8; 1_000_000];
            black_box(data)
        });
    });

    group.finish();
}

// ============================================================================
// 哈希操作性能测试
// ============================================================================

/// 测试哈希操作性能
fn bench_hash_operations(c: &mut Criterion) {
    use std::collections::HashMap;

    let mut group = c.benchmark_group("hash_operations");

    // HashMap 插入
    group.bench_function("hashmap_insert", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..1000 {
                map.insert(format!("key-{}", i), i);
            }
            black_box(map)
        });
    });

    // HashMap 查找
    group.bench_function("hashmap_lookup", |b| {
        let mut map = HashMap::new();
        for i in 0..1000 {
            map.insert(format!("key-{}", i), i);
        }

        b.iter(|| black_box(map.get("key-500")));
    });

    group.finish();
}

// ============================================================================
// 并发性能测试
// ============================================================================

/// 测试并发操作性能
fn bench_concurrent_operations(c: &mut Criterion) {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let mut group = c.benchmark_group("concurrent_operations");

    // Arc + Mutex 性能
    group.bench_function("arc_mutex", |b| {
        b.iter(|| {
            let counter = Arc::new(Mutex::new(0));
            let mut handles = vec![];

            for _ in 0..10 {
                let counter = Arc::clone(&counter);
                let handle = thread::spawn(move || {
                    for _ in 0..100 {
                        let mut num = counter.lock().unwrap();
                        *num += 1;
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            black_box(counter)
        });
    });

    group.finish();
}

// ============================================================================
// 基准测试组配置
// ============================================================================

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets =
        bench_json_serialization,
        bench_json_deserialization,
        bench_string_operations,
        bench_vec_operations,
        bench_memory_allocation,
        bench_hash_operations,
        bench_concurrent_operations
}

criterion_main!(benches);
