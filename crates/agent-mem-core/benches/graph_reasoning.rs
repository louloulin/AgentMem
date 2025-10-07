//! Graph Reasoning Benchmarks
//!
//! 基准测试 AgentMem 的图推理性能
//!
//! 测试目标:
//! - 节点添加: < 10ms
//! - 边添加: < 5ms
//! - 路径查找: < 100ms
//! - 推理操作: < 200ms

use agent_mem_core::{
    graph_memory::{GraphMemoryEngine, NodeType, RelationType, ReasoningType},
    types::{Memory, MemoryType},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// 创建测试记忆
fn create_test_memory(id: &str, content: &str) -> Memory {
    Memory {
        id: id.to_string(),
        agent_id: "test_agent".to_string(),
        user_id: Some("test_user".to_string()),
        memory_type: MemoryType::Semantic,
        content: content.to_string(),
        embedding: Some(vec![0.1; 384]),
        score: Some(0.8),
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        accessed_at: chrono::Utc::now(),
        access_count: 0,
    }
}

/// 基准测试: 添加图节点
fn bench_add_graph_node(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("add_graph_node", |b| {
        let mut counter = 0;
        b.iter(|| {
            let engine = GraphMemoryEngine::new();
            counter += 1;
            rt.block_on(async {
                let memory = create_test_memory(
                    &format!("node_{}", counter),
                    &format!("Test node {}", counter),
                );
                let node_id = engine
                    .add_node(black_box(memory), black_box(NodeType::Concept))
                    .await
                    .unwrap();
                black_box(node_id)
            })
        });
    });
}

/// 基准测试: 添加图边
fn bench_add_graph_edge(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let engine = GraphMemoryEngine::new();

    // 预先创建两个节点
    let (node1, node2) = rt.block_on(async {
        let memory1 = create_test_memory("node_1", "First node");
        let memory2 = create_test_memory("node_2", "Second node");
        let id1 = engine.add_node(memory1, NodeType::Concept).await.unwrap();
        let id2 = engine.add_node(memory2, NodeType::Concept).await.unwrap();
        (id1, id2)
    });

    c.bench_function("add_graph_edge", |b| {
        b.iter(|| {
            rt.block_on(async {
                let edge_id = engine
                    .add_edge(
                        black_box(node1.clone()),
                        black_box(node2.clone()),
                        black_box(RelationType::Causal),
                        black_box(0.9),
                    )
                    .await
                    .unwrap();
                black_box(edge_id)
            })
        });
    });
}

/// 基准测试: 查找最短路径
fn bench_find_shortest_path(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let engine = GraphMemoryEngine::new();

    // 创建一个小图: A -> B -> C -> D
    let (node_a, node_d) = rt.block_on(async {
        let memory_a = create_test_memory("node_a", "Node A");
        let memory_b = create_test_memory("node_b", "Node B");
        let memory_c = create_test_memory("node_c", "Node C");
        let memory_d = create_test_memory("node_d", "Node D");

        let id_a = engine.add_node(memory_a, NodeType::Concept).await.unwrap();
        let id_b = engine.add_node(memory_b, NodeType::Concept).await.unwrap();
        let id_c = engine.add_node(memory_c, NodeType::Concept).await.unwrap();
        let id_d = engine.add_node(memory_d, NodeType::Concept).await.unwrap();

        engine
            .add_edge(id_a.clone(), id_b.clone(), RelationType::Causal, 1.0)
            .await
            .unwrap();
        engine
            .add_edge(id_b.clone(), id_c.clone(), RelationType::Causal, 1.0)
            .await
            .unwrap();
        engine
            .add_edge(id_c.clone(), id_d.clone(), RelationType::Causal, 1.0)
            .await
            .unwrap();

        (id_a, id_d)
    });

    c.bench_function("find_shortest_path", |b| {
        b.iter(|| {
            rt.block_on(async {
                let paths = engine
                    .find_shortest_paths(
                        black_box(&node_a),
                        black_box(&node_d),
                        black_box(5),
                    )
                    .await
                    .unwrap();
                black_box(paths)
            })
        });
    });
}

/// 基准测试: 图推理
fn bench_graph_reasoning(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let engine = GraphMemoryEngine::new();

    // 创建一个小图
    let (node_a, node_d) = rt.block_on(async {
        let memory_a = create_test_memory("node_a", "Node A");
        let memory_b = create_test_memory("node_b", "Node B");
        let memory_c = create_test_memory("node_c", "Node C");
        let memory_d = create_test_memory("node_d", "Node D");

        let id_a = engine.add_node(memory_a, NodeType::Concept).await.unwrap();
        let id_b = engine.add_node(memory_b, NodeType::Concept).await.unwrap();
        let id_c = engine.add_node(memory_c, NodeType::Concept).await.unwrap();
        let id_d = engine.add_node(memory_d, NodeType::Concept).await.unwrap();

        engine
            .add_edge(id_a.clone(), id_b.clone(), RelationType::Causal, 1.0)
            .await
            .unwrap();
        engine
            .add_edge(id_b.clone(), id_c.clone(), RelationType::Causal, 1.0)
            .await
            .unwrap();
        engine
            .add_edge(id_c.clone(), id_d.clone(), RelationType::Causal, 1.0)
            .await
            .unwrap();

        (id_a, id_d)
    });

    let mut group = c.benchmark_group("graph_reasoning");

    for reasoning_type in [
        ReasoningType::Deductive,
        ReasoningType::Inductive,
        ReasoningType::Abductive,
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", reasoning_type)),
            reasoning_type,
            |b, &reasoning_type| {
                b.iter(|| {
                    rt.block_on(async {
                        let paths = engine
                            .reason(
                                black_box(&node_a),
                                black_box(&node_d),
                                black_box(reasoning_type),
                            )
                            .await
                            .unwrap();
                        black_box(paths)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试: 获取邻居节点
fn bench_get_neighbors(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let engine = GraphMemoryEngine::new();

    // 创建一个中心节点和多个邻居
    let center_node = rt.block_on(async {
        let memory_center = create_test_memory("center", "Center node");
        let center_id = engine
            .add_node(memory_center, NodeType::Concept)
            .await
            .unwrap();

        // 添加10个邻居
        for i in 0..10 {
            let memory = create_test_memory(&format!("neighbor_{}", i), &format!("Neighbor {}", i));
            let neighbor_id = engine.add_node(memory, NodeType::Concept).await.unwrap();
            engine
                .add_edge(
                    center_id.clone(),
                    neighbor_id,
                    RelationType::Similar,
                    0.8,
                )
                .await
                .unwrap();
        }

        center_id
    });

    c.bench_function("get_neighbors", |b| {
        b.iter(|| {
            rt.block_on(async {
                let neighbors = engine
                    .get_neighbors(black_box(&center_node))
                    .await
                    .unwrap();
                black_box(neighbors)
            })
        });
    });
}

/// 基准测试: 批量添加节点
fn bench_batch_add_nodes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_add_nodes");

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let engine = GraphMemoryEngine::new();
                    rt.block_on(async {
                        let mut node_ids = Vec::new();
                        for i in 0..size {
                            let memory = create_test_memory(
                                &format!("batch_node_{}", i),
                                &format!("Batch node {}", i),
                            );
                            let node_id = engine
                                .add_node(black_box(memory), black_box(NodeType::Concept))
                                .await
                                .unwrap();
                            node_ids.push(node_id);
                        }
                        black_box(node_ids)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试: 批量添加边
fn bench_batch_add_edges(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_add_edges");

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let engine = GraphMemoryEngine::new();
                    rt.block_on(async {
                        // 先创建节点
                        let mut node_ids = Vec::new();
                        for i in 0..size {
                            let memory = create_test_memory(
                                &format!("node_{}", i),
                                &format!("Node {}", i),
                            );
                            let node_id = engine
                                .add_node(memory, NodeType::Concept)
                                .await
                                .unwrap();
                            node_ids.push(node_id);
                        }

                        // 添加边（连接相邻节点）
                        let mut edge_ids = Vec::new();
                        for i in 0..size - 1 {
                            let edge_id = engine
                                .add_edge(
                                    black_box(node_ids[i].clone()),
                                    black_box(node_ids[i + 1].clone()),
                                    black_box(RelationType::Causal),
                                    black_box(0.9),
                                )
                                .await
                                .unwrap();
                            edge_ids.push(edge_id);
                        }
                        black_box(edge_ids)
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_add_graph_node,
    bench_add_graph_edge,
    bench_find_shortest_path,
    bench_graph_reasoning,
    bench_get_neighbors,
    bench_batch_add_nodes,
    bench_batch_add_edges,
);

criterion_main!(benches);

