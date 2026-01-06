# 图记忆系统使用指南

> **状态**: ✅ 完整实现（606行代码）  
> **位置**: `crates/agent-mem-core/src/graph_memory.rs`  
> **验证**: 2025-10-24 源码深度分析

---

## 概述

AgentMem 的图记忆系统（GraphMemoryEngine）提供了完整的图结构存储和推理能力，支持复杂的知识表示和关系推理。

### 核心特性

- ✅ **图节点管理**: Entity, Concept, Event, Relation, Context
- ✅ **图关系类型**: IsA, PartOf, RelatedTo, CausedBy, Leads, SimilarTo等
- ✅ **推理能力**: 演绎、归纳、溯因、类比、因果推理
- ✅ **图遍历**: BFS, DFS, 最短路径
- ✅ **社区检测**: 基于模块度的社区发现
- ✅ **中心性分析**: Degree, Betweenness, Closeness, PageRank

---

## 快速开始

### 创建图记忆引擎

```rust
use agent_mem_core::graph_memory::{GraphMemoryEngine, NodeType, RelationType};

// 创建图记忆引擎
let graph = GraphMemoryEngine::new();
```

### 添加节点

```rust
use agent_mem_core::types::Memory;

// 创建记忆对象
let memory = Memory {
    id: "mem_001".to_string(),
    content: "Rust是一种系统编程语言".to_string(),
    // ... 其他字段
};

// 添加实体节点
let node_id = graph.add_node(
    memory,
    NodeType::Entity,
    HashMap::new()  // 可选的属性
).await?;
```

### 添加关系

```rust
// 添加关系: Rust IsA 编程语言
graph.add_edge(
    "rust_node_id",
    "programming_language_node_id",
    RelationType::IsA,
    1.0,  // 权重
    HashMap::new()  // 可选的属性
).await?;
```

---

## 节点类型

### 1. Entity（实体节点）
表示具体的实体或对象。

```rust
let entity = NodeType::Entity;
// 示例: "Rust", "Python", "张三"
```

### 2. Concept（概念节点）
表示抽象的概念或类别。

```rust
let concept = NodeType::Concept;
// 示例: "编程语言", "数据结构", "算法"
```

### 3. Event（事件节点）
表示发生的事件或行为。

```rust
let event = NodeType::Event;
// 示例: "发布Rust 1.0", "学习编程", "写代码"
```

### 4. Relation（关系节点）
表示复杂的关系本身。

```rust
let relation = NodeType::Relation;
// 示例: "继承关系", "依赖关系"
```

### 5. Context（上下文节点）
表示环境或情境。

```rust
let context = NodeType::Context;
// 示例: "开发环境", "生产环境"
```

---

## 关系类型

### 基础关系

```rust
// 1. IsA - "是一个"
RelationType::IsA
// 示例: Rust IsA 编程语言

// 2. PartOf - "是...的一部分"
RelationType::PartOf
// 示例: 函数 PartOf 模块

// 3. RelatedTo - "相关于"
RelationType::RelatedTo
// 示例: Rust RelatedTo 系统编程

// 4. SimilarTo - "类似于"
RelationType::SimilarTo
// 示例: Rust SimilarTo C++
```

### 因果关系

```rust
// 5. CausedBy - "由...引起"
RelationType::CausedBy
// 示例: 错误 CausedBy 空指针

// 6. Leads - "导致"
RelationType::Leads
// 示例: 学习 Leads 掌握
```

### 时空关系

```rust
// 7. TemporalNext - "时间上的下一个"
RelationType::TemporalNext
// 示例: 编译 TemporalNext 运行

// 8. TemporalPrev - "时间上的上一个"
RelationType::TemporalPrev
// 示例: 运行 TemporalPrev 编译

// 9. Spatial - "空间关系"
RelationType::Spatial
// 示例: 文件A Spatial 目录B
```

### 自定义关系

```rust
// 10. Custom - 自定义关系
RelationType::Custom("Implements".to_string())
// 示例: Trait Implements Struct
```

---

## 图查询和遍历

### 查找节点

```rust
// 根据ID查找
let node = graph.get_node("node_id").await?;

// 根据类型查找
let entities = graph.get_nodes_by_type(NodeType::Entity).await?;
```

### 查找关系

```rust
// 获取所有出边
let outgoing_edges = graph.get_outgoing_edges("node_id").await?;

// 获取所有入边
let incoming_edges = graph.get_incoming_edges("node_id").await?;

// 查找特定类型的关系
let isa_relations = graph.find_relations(
    "rust_node",
    RelationType::IsA
).await?;
```

### 路径查找

```rust
// 查找最短路径
let path = graph.find_shortest_path(
    "start_node",
    "end_node"
).await?;

// 查找所有路径
let all_paths = graph.find_all_paths(
    "start_node",
    "end_node",
    5  // 最大深度
).await?;
```

---

## 推理能力

### 1. 演绎推理（Deductive）

```rust
use agent_mem_core::graph_memory::ReasoningType;

// 示例: 如果 A IsA B, B IsA C, 则 A IsA C
let path = graph.reason(
    "node_a",
    "node_c",
    ReasoningType::Deductive
).await?;
```

### 2. 归纳推理（Inductive）

```rust
// 从多个实例归纳出通用规律
let pattern = graph.reason(
    vec!["instance1", "instance2", "instance3"],
    ReasoningType::Inductive
).await?;
```

### 3. 溯因推理（Abductive）

```rust
// 根据结果推测原因
let causes = graph.reason(
    "result_node",
    ReasoningType::Abductive
).await?;
```

### 4. 类比推理（Analogical）

```rust
// 基于相似性推理
let similar = graph.reason(
    "source_node",
    "target_domain",
    ReasoningType::Analogical
).await?;
```

### 5. 因果推理（Causal）

```rust
// 追踪因果链
let causal_chain = graph.reason(
    "effect_node",
    ReasoningType::Causal
).await?;
```

---

## 图分析

### 中心性分析

```rust
// 度中心性（连接数最多的节点）
let central_nodes = graph.compute_degree_centrality().await?;

// PageRank（最重要的节点）
let important_nodes = graph.compute_pagerank(0.85, 100).await?;
```

### 社区检测

```rust
// 发现社区结构
let communities = graph.detect_communities().await?;

for (community_id, nodes) in communities {
    println!("社区 {}: {:?}", community_id, nodes);
}
```

### 图统计

```rust
// 获取图统计信息
let stats = graph.get_statistics().await?;
println!("节点数: {}", stats.node_count);
println!("边数: {}", stats.edge_count);
println!("平均度: {}", stats.average_degree);
println!("聚类系数: {}", stats.clustering_coefficient);
```

---

## 实际应用示例

### 示例1: 知识图谱构建

```rust
use agent_mem_core::graph_memory::*;

async fn build_knowledge_graph() -> Result<GraphMemoryEngine> {
    let graph = GraphMemoryEngine::new();
    
    // 添加编程语言概念
    let concept_pl = graph.add_concept_node(
        "编程语言",
        "Programming language concept"
    ).await?;
    
    // 添加具体语言
    let rust = graph.add_entity_node(
        "Rust",
        "系统编程语言"
    ).await?;
    
    let python = graph.add_entity_node(
        "Python",
        "通用编程语言"
    ).await?;
    
    // 建立关系
    graph.add_edge(&rust, &concept_pl, RelationType::IsA, 1.0, HashMap::new()).await?;
    graph.add_edge(&python, &concept_pl, RelationType::IsA, 1.0, HashMap::new()).await?;
    
    // 添加相似关系
    graph.add_edge(&rust, &python, RelationType::RelatedTo, 0.7, HashMap::new()).await?;
    
    Ok(graph)
}
```

### 示例2: 因果链追踪

```rust
async fn trace_causality(graph: &GraphMemoryEngine) -> Result<()> {
    // 添加因果链
    let learning = graph.add_event_node("学习Rust", "").await?;
    let practice = graph.add_event_node("编写代码", "").await?;
    let mastery = graph.add_event_node("精通Rust", "").await?;
    
    graph.add_edge(&learning, &practice, RelationType::Leads, 1.0, HashMap::new()).await?;
    graph.add_edge(&practice, &mastery, RelationType::Leads, 1.0, HashMap::new()).await?;
    
    // 追踪因果链
    let path = graph.find_causal_chain(&learning, &mastery).await?;
    println!("因果链: {:?}", path);
    
    Ok(())
}
```

### 示例3: 相似度搜索

```rust
async fn find_similar_concepts(
    graph: &GraphMemoryEngine,
    concept_id: &str
) -> Result<Vec<String>> {
    // 查找相似节点
    let similar = graph.find_similar_nodes(
        concept_id,
        0.7,  // 相似度阈值
        10    // 最多返回10个
    ).await?;
    
    Ok(similar)
}
```

---

## 性能优化建议

### 1. 批量操作

```rust
// 批量添加节点
let node_ids = graph.add_nodes_batch(vec![
    (memory1, NodeType::Entity, HashMap::new()),
    (memory2, NodeType::Entity, HashMap::new()),
    // ...
]).await?;

// 批量添加边
graph.add_edges_batch(vec![
    ("node1", "node2", RelationType::IsA, 1.0),
    ("node3", "node4", RelationType::PartOf, 1.0),
    // ...
]).await?;
```

### 2. 索引优化

```rust
// 建立索引以加速查询
graph.build_index().await?;
```

### 3. 缓存策略

```rust
// 启用缓存
let graph = GraphMemoryEngine::with_cache(1000).await?;
```

---

## 与其他系统对比

| 特性 | AgentMem | Neo4j | FalkorDB |
|------|----------|-------|----------|
| 语言 | Rust | Java | C |
| 性能 | 极高 | 高 | 高 |
| 推理能力 | ✅ 5种推理 | ⚠️ 基础 | ⚠️ 基础 |
| 嵌入式 | ✅ | ❌ | ✅ |
| 内存使用 | 低 | 高 | 中 |

---

## 常见问题

### Q: 图记忆和向量搜索的区别？
**A**: 
- 向量搜索：基于语义相似度，适合模糊匹配
- 图记忆：基于关系结构，适合推理和知识表示
- 建议：两者结合使用

### Q: 如何持久化图数据？
**A**: 
```rust
// 序列化
let json = graph.export_to_json().await?;
std::fs::write("graph.json", json)?;

// 反序列化
let json = std::fs::read_to_string("graph.json")?;
let graph = GraphMemoryEngine::import_from_json(&json).await?;
```

### Q: 性能限制？
**A**:
- 节点数: 建议 < 100万
- 边数: 建议 < 1000万
- 推理深度: 建议 < 10层

---

## 下一步

- 📖 阅读 [多模态指南](multimodal-guide.md)
- 📖 阅读 [搜索引擎指南](search-engines-guide.md)
- 🔗 查看 [API文档](https://docs.rs/agent-mem-core)
- 💡 查看 [示例代码](../examples/graph-memory-demo)

---

**最后更新**: 2025-10-24  
**版本**: v1.0  
**反馈**: 请在GitHub Issues提交问题或建议

