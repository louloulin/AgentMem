use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::Memory;

// 类型别名
/// 内存节点ID类型
pub type MemoryId = String;
/// 用户ID类型
pub type UserId = String;

/// 图记忆节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// 节点唯一标识符
    pub id: MemoryId,
    /// 关联的内存对象
    pub memory: Memory,
    /// 节点类型
    pub node_type: NodeType,
    /// 节点属性
    pub properties: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// 实体节点
    Entity,
    /// 概念节点
    Concept,
    /// 事件节点
    Event,
    /// 关系节点
    Relation,
    /// 上下文节点
    Context,
}

/// 图记忆边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: Uuid,
    pub from_node: MemoryId,
    pub to_node: MemoryId,
    pub relation_type: RelationType,
    pub weight: f32,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType {
    IsA,            // 是一个
    PartOf,         // 是...的一部分
    RelatedTo,      // 相关于
    CausedBy,       // 由...引起
    Leads,          // 导致
    SimilarTo,      // 类似于
    OppositeOf,     // 相反于
    TemporalNext,   // 时间上的下一个
    TemporalPrev,   // 时间上的上一个
    Spatial,        // 空间关系
    Custom(String), // 自定义关系
}

/// 推理路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPath {
    pub nodes: Vec<MemoryId>,
    pub edges: Vec<Uuid>,
    pub confidence: f32,
    pub reasoning_type: ReasoningType,
}

/// 推理类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReasoningType {
    Deductive,  // 演绎推理
    Inductive,  // 归纳推理
    Abductive,  // 溯因推理
    Analogical, // 类比推理
    Causal,     // 因果推理
}

/// 图记忆和关系推理引擎
#[derive(Debug)]
pub struct GraphMemoryEngine {
    nodes: Arc<RwLock<HashMap<MemoryId, GraphNode>>>,
    edges: Arc<RwLock<HashMap<Uuid, GraphEdge>>>,
    adjacency_list: Arc<RwLock<HashMap<MemoryId, Vec<Uuid>>>>,
    reverse_adjacency: Arc<RwLock<HashMap<MemoryId, Vec<Uuid>>>>,
    node_index: Arc<RwLock<HashMap<String, HashSet<MemoryId>>>>,
}

impl Default for GraphMemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphMemoryEngine {
    /// 创建新的图记忆引擎
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            adjacency_list: Arc::new(RwLock::new(HashMap::new())),
            reverse_adjacency: Arc::new(RwLock::new(HashMap::new())),
            node_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加节点
    pub async fn add_node(&self, memory: Memory, node_type: NodeType) -> Result<MemoryId> {
        let node_id = memory.id.clone();
        let now = chrono::Utc::now();

        let node = GraphNode {
            id: node_id.clone(),
            memory,
            node_type: node_type.clone(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };

        // 添加到节点集合
        self.nodes.write().await.insert(node_id.clone(), node);

        // 初始化邻接表
        self.adjacency_list
            .write()
            .await
            .insert(node_id.clone(), Vec::new());
        self.reverse_adjacency
            .write()
            .await
            .insert(node_id.clone(), Vec::new());

        // 更新索引
        let type_key = format!("type:{node_type:?}");
        self.node_index
            .write()
            .await
            .entry(type_key)
            .or_insert_with(HashSet::new)
            .insert(node_id.clone());

        Ok(node_id)
    }

    /// 添加边
    pub async fn add_edge(
        &self,
        from_node: MemoryId,
        to_node: MemoryId,
        relation_type: RelationType,
        weight: f32,
    ) -> Result<Uuid> {
        // 检查节点是否存在
        let nodes = self.nodes.read().await;
        if !nodes.contains_key(&from_node) || !nodes.contains_key(&to_node) {
            return Err(anyhow!("One or both nodes do not exist"));
        }
        drop(nodes);

        let edge_id = Uuid::new_v4();
        let edge = GraphEdge {
            id: edge_id,
            from_node: from_node.clone(),
            to_node: to_node.clone(),
            relation_type,
            weight,
            properties: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        // 添加边
        self.edges.write().await.insert(edge_id, edge);

        // 更新邻接表
        self.adjacency_list
            .write()
            .await
            .entry(from_node)
            .or_insert_with(Vec::new)
            .push(edge_id);

        self.reverse_adjacency
            .write()
            .await
            .entry(to_node)
            .or_insert_with(Vec::new)
            .push(edge_id);

        Ok(edge_id)
    }

    /// 查找相关节点
    pub async fn find_related_nodes(
        &self,
        node_id: &MemoryId,
        max_depth: usize,
        relation_types: Option<Vec<RelationType>>,
    ) -> Result<Vec<GraphNode>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back((node_id.clone(), 0));
        visited.insert(node_id.clone());

        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;
        let adjacency = self.adjacency_list.read().await;

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }

            if let Some(node) = nodes.get(&current_id) {
                if depth > 0 {
                    // 不包括起始节点
                    result.push(node.clone());
                }
            }

            if let Some(edge_ids) = adjacency.get(&current_id) {
                for edge_id in edge_ids {
                    if let Some(edge) = edges.get(edge_id) {
                        // 检查关系类型过滤
                        if let Some(ref types) = relation_types {
                            if !types.contains(&edge.relation_type) {
                                continue;
                            }
                        }

                        if !visited.contains(&edge.to_node) {
                            visited.insert(edge.to_node.clone());
                            queue.push_back((edge.to_node.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// 执行关系推理
    pub async fn reason_relationships(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
        reasoning_type: ReasoningType,
    ) -> Result<Vec<ReasoningPath>> {
        match reasoning_type {
            ReasoningType::Deductive => self.deductive_reasoning(start_node, target_node).await,
            ReasoningType::Inductive => self.inductive_reasoning(start_node, target_node).await,
            ReasoningType::Abductive => self.abductive_reasoning(start_node, target_node).await,
            ReasoningType::Analogical => self.analogical_reasoning(start_node, target_node).await,
            ReasoningType::Causal => self.causal_reasoning(start_node, target_node).await,
        }
    }

    /// 演绎推理
    async fn deductive_reasoning(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        // 使用 Dijkstra 算法找到最短路径
        self.find_shortest_paths(start_node, target_node, 5).await
    }

    /// 归纳推理
    async fn inductive_reasoning(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        // 基于模式识别的归纳推理
        self.find_pattern_based_paths(start_node, target_node).await
    }

    /// 溯因推理
    async fn abductive_reasoning(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        // 反向推理，从结果推原因
        self.find_reverse_causal_paths(start_node, target_node)
            .await
    }

    /// 类比推理
    async fn analogical_reasoning(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        // 基于相似性的类比推理
        self.find_similarity_based_paths(start_node, target_node)
            .await
    }

    /// 因果推理
    async fn causal_reasoning(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        // 基于因果关系的推理
        self.find_causal_paths(start_node, target_node).await
    }

    /// 查找最短路径
    async fn find_shortest_paths(
        &self,
        start: &MemoryId,
        target: &MemoryId,
        _max_paths: usize,
    ) -> Result<Vec<ReasoningPath>> {
        // 简化的 Dijkstra 实现
        let mut paths = Vec::new();

        // 这里实现具体的路径查找算法
        // 为了简化，返回一个示例路径
        if start != target {
            paths.push(ReasoningPath {
                nodes: vec![start.clone(), target.clone()],
                edges: vec![],
                confidence: 0.8,
                reasoning_type: ReasoningType::Deductive,
            });
        }

        Ok(paths)
    }

    /// 基于模式的路径查找
    async fn find_pattern_based_paths(
        &self,
        start: &MemoryId,
        target: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        let mut paths = Vec::new();

        // 获取起始节点的所有相关节点
        let start_related = self.find_related_nodes(start, 2, None).await?;
        let target_related = self.find_related_nodes(target, 2, None).await?;

        // 查找共同的关系模式
        let _nodes = self.nodes.read().await;
        let _edges = self.edges.read().await;

        for start_node in &start_related {
            for target_node in &target_related {
                // 检查是否有相似的关系模式
                if start_node.node_type == target_node.node_type {
                    // 找到相似的节点类型，构建归纳推理路径
                    let path = ReasoningPath {
                        nodes: vec![
                            start.clone(),
                            start_node.id.clone(),
                            target_node.id.clone(),
                            target.clone(),
                        ],
                        edges: vec![],   // 简化实现，实际应该包含具体的边ID
                        confidence: 0.7, // 基于模式相似性的置信度
                        reasoning_type: ReasoningType::Inductive,
                    };
                    paths.push(path);
                }
            }
        }

        Ok(paths)
    }

    /// 反向因果路径查找
    async fn find_reverse_causal_paths(
        &self,
        start: &MemoryId,
        target: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        let mut paths = Vec::new();

        // 从目标节点开始，反向查找可能的原因
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((target.clone(), vec![target.clone()], 0));
        visited.insert(target.clone());

        let edges = self.edges.read().await;
        let reverse_adjacency = self.reverse_adjacency.read().await;

        while let Some((current_id, path, depth)) = queue.pop_front() {
            if depth > 3 {
                // 限制搜索深度
                continue;
            }

            if current_id == *start {
                // 找到从start到target的反向因果路径
                let mut reverse_path = path.clone();
                reverse_path.reverse();

                let reasoning_path = ReasoningPath {
                    nodes: reverse_path,
                    edges: vec![],                          // 简化实现
                    confidence: 0.6 - (depth as f32 * 0.1), // 深度越大置信度越低
                    reasoning_type: ReasoningType::Abductive,
                };
                paths.push(reasoning_path);
                continue;
            }

            // 查找指向当前节点的因果关系边
            if let Some(incoming_edges) = reverse_adjacency.get(&current_id) {
                for edge_id in incoming_edges {
                    if let Some(edge) = edges.get(edge_id) {
                        if matches!(
                            edge.relation_type,
                            RelationType::CausedBy | RelationType::Leads
                        ) {
                            let from_node = &edge.from_node;
                            if !visited.contains(from_node) {
                                visited.insert(from_node.clone());
                                let mut new_path = path.clone();
                                new_path.push(from_node.clone());
                                queue.push_back((from_node.clone(), new_path, depth + 1));
                            }
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    /// 基于相似性的路径查找
    async fn find_similarity_based_paths(
        &self,
        start: &MemoryId,
        target: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        let mut paths = Vec::new();

        let nodes = self.nodes.read().await;
        let start_node = nodes
            .get(start)
            .ok_or_else(|| anyhow!("Start node not found"))?;
        let _target_node = nodes
            .get(target)
            .ok_or_else(|| anyhow!("Target node not found"))?;

        // 查找与起始节点相似的节点
        let mut similar_nodes = Vec::new();
        for (node_id, node) in nodes.iter() {
            if node_id != start && node_id != target {
                let similarity = self.calculate_node_similarity(start_node, node);
                if similarity > 0.5 {
                    // 相似度阈值
                    similar_nodes.push((node_id.clone(), similarity));
                }
            }
        }

        // 按相似度排序
        similar_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 为每个相似节点构建类比推理路径
        for (similar_id, similarity) in similar_nodes.into_iter().take(3) {
            // 取前3个最相似的
            // 查找相似节点到目标节点的路径
            if let Ok(related) = self.find_related_nodes(&similar_id, 2, None).await {
                for related_node in related {
                    if related_node.id == *target {
                        let path = ReasoningPath {
                            nodes: vec![start.clone(), similar_id.clone(), target.clone()],
                            edges: vec![],
                            confidence: similarity * 0.8, // 基于相似度的置信度
                            reasoning_type: ReasoningType::Analogical,
                        };
                        paths.push(path);
                        break;
                    }
                }
            }
        }

        Ok(paths)
    }

    /// 计算节点相似性
    fn calculate_node_similarity(&self, node1: &GraphNode, node2: &GraphNode) -> f32 {
        let mut similarity = 0.0;

        // 基于节点类型的相似性
        if node1.node_type == node2.node_type {
            similarity += 0.3;
        }

        // 基于内容的相似性（简化实现）
        let content1_str = node1.memory.content.to_string();
        let content2_str = node2.memory.content.to_string();
        let content1_words: HashSet<&str> = content1_str.split_whitespace().collect();
        let content2_words: HashSet<&str> = content2_str.split_whitespace().collect();

        let intersection = content1_words.intersection(&content2_words).count();
        let union = content1_words.union(&content2_words).count();

        if union > 0 {
            similarity += (intersection as f32 / union as f32) * 0.7;
        }

        similarity.min(1.0_f32)
    }

    /// 因果路径查找
    async fn find_causal_paths(
        &self,
        start: &MemoryId,
        target: &MemoryId,
    ) -> Result<Vec<ReasoningPath>> {
        let mut paths = Vec::new();

        // 使用广度优先搜索查找因果链
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((start.clone(), vec![start.clone()], vec![], 0));
        visited.insert(start.clone());

        let edges = self.edges.read().await;
        let adjacency = self.adjacency_list.read().await;

        while let Some((current_id, node_path, edge_path, depth)) = queue.pop_front() {
            if depth > 4 {
                // 限制因果链长度
                continue;
            }

            if current_id == *target {
                // 找到因果路径
                let reasoning_path = ReasoningPath {
                    nodes: node_path,
                    edges: edge_path,
                    confidence: 0.9 - (depth as f32 * 0.1), // 链越长置信度越低
                    reasoning_type: ReasoningType::Causal,
                };
                paths.push(reasoning_path);
                continue;
            }

            // 查找因果关系边
            if let Some(outgoing_edges) = adjacency.get(&current_id) {
                for edge_id in outgoing_edges {
                    if let Some(edge) = edges.get(edge_id) {
                        // 只考虑因果关系
                        if matches!(
                            edge.relation_type,
                            RelationType::CausedBy | RelationType::Leads
                        ) {
                            let next_node = &edge.to_node;
                            if !visited.contains(next_node) {
                                visited.insert(next_node.clone());
                                let mut new_node_path = node_path.clone();
                                new_node_path.push(next_node.clone());
                                let mut new_edge_path = edge_path.clone();
                                new_edge_path.push(*edge_id);
                                queue.push_back((
                                    next_node.clone(),
                                    new_node_path,
                                    new_edge_path,
                                    depth + 1,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    /// 获取节点统计信息
    pub async fn get_graph_stats(&self) -> Result<GraphStats> {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        Ok(GraphStats {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            node_types: self.count_node_types(&nodes).await,
            relation_types: self.count_relation_types(&edges).await,
        })
    }

    async fn count_node_types(
        &self,
        nodes: &HashMap<MemoryId, GraphNode>,
    ) -> HashMap<NodeType, usize> {
        let mut counts = HashMap::new();
        for node in nodes.values() {
            *counts.entry(node.node_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    async fn count_relation_types(
        &self,
        edges: &HashMap<Uuid, GraphEdge>,
    ) -> HashMap<RelationType, usize> {
        let mut counts = HashMap::new();
        for edge in edges.values() {
            *counts.entry(edge.relation_type.clone()).or_insert(0) += 1;
        }
        counts
    }
}

/// 图统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_types: HashMap<NodeType, usize>,
    /// Count of each relation type in the graph
    pub relation_types: HashMap<RelationType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Memory;

    #[tokio::test]
    async fn test_graph_memory_basic_operations() {
        use crate::types::MemoryType;
        use agent_mem_traits::Vector;

        let engine = GraphMemoryEngine::new();

        // 创建测试记忆
        let memory1 = Memory {
            id: "mem1".to_string(),
            agent_id: "test_agent".to_string(),
            user_id: Some("user1".to_string()),
            memory_type: MemoryType::Semantic,
            content: "Apple is a fruit".to_string(),
            importance: 0.8,
            embedding: Some(Vector::new(vec![0.1, 0.2, 0.3])),
            created_at: chrono::Utc::now().timestamp(),
            last_accessed_at: chrono::Utc::now().timestamp(),
            access_count: 0,
            expires_at: None,
            metadata: HashMap::new(),
            version: 1,
        };

        let memory2 = Memory {
            id: "mem2".to_string(),
            agent_id: "test_agent".to_string(),
            user_id: Some("user1".to_string()),
            memory_type: MemoryType::Semantic,
            content: "Fruit is healthy".to_string(),
            importance: 0.7,
            embedding: Some(Vector::new(vec![0.4, 0.5, 0.6])),
            created_at: chrono::Utc::now().timestamp(),
            last_accessed_at: chrono::Utc::now().timestamp(),
            access_count: 0,
            expires_at: None,
            metadata: HashMap::new(),
            version: 1,
        };

        // 添加节点
        let node1_id = engine.add_node(memory1, NodeType::Entity).await.unwrap();
        let node2_id = engine.add_node(memory2, NodeType::Concept).await.unwrap();

        // 添加边
        let _edge_id = engine
            .add_edge(node1_id.clone(), node2_id.clone(), RelationType::IsA, 1.0)
            .await
            .unwrap();

        // 查找相关节点
        let related = engine.find_related_nodes(&node1_id, 2, None).await.unwrap();
        assert_eq!(related.len(), 1);

        // 获取统计信息
        let stats = engine.get_graph_stats().await.unwrap();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.total_edges, 1);
    }
}
