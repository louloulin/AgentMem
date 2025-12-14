/// 时序知识图谱模块
///
/// 扩展现有图记忆系统以支持时序信息，包括：
/// - 时间戳和时间范围管理
/// - 时序关系建模
/// - 动态关系演化追踪
/// - 时间窗口查询
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::graph_memory::{GraphEdge, GraphMemoryEngine, GraphNode, MemoryId};

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeRange {
    /// 开始时间
    pub start: DateTime<Utc>,
    /// 结束时间（None 表示持续到现在）
    pub end: Option<DateTime<Utc>>,
}

impl TimeRange {
    /// 创建新的时间范围
    pub fn new(start: DateTime<Utc>, end: Option<DateTime<Utc>>) -> Self {
        Self { start, end }
    }

    /// 创建从某个时间点开始的开放范围
    pub fn from(start: DateTime<Utc>) -> Self {
        Self { start, end: None }
    }

    /// 创建单个时间点
    pub fn point(time: DateTime<Utc>) -> Self {
        Self {
            start: time,
            end: Some(time),
        }
    }

    /// 检查时间范围是否包含某个时间点
    pub fn contains(&self, time: &DateTime<Utc>) -> bool {
        if time < &self.start {
            return false;
        }
        if let Some(end) = &self.end {
            time <= end
        } else {
            true
        }
    }

    /// 检查两个时间范围是否重叠
    pub fn overlaps(&self, other: &TimeRange) -> bool {
        // 如果一个范围的开始时间在另一个范围内
        self.contains(&other.start)
            || other.contains(&self.start)
            || (self.end.is_some() && self.end.map(|end| other.contains(&end)).unwrap_or(false))
            || (other.end.is_some() && other.end.map(|end| self.contains(&end)).unwrap_or(false))
    }

    /// 获取持续时间
    pub fn duration(&self) -> Option<Duration> {
        self.end.map(|end| end - self.start)
    }

    /// 检查范围是否仍然有效（未结束）
    pub fn is_active(&self) -> bool {
        self.end.map(|end| end > Utc::now()).unwrap_or(true)
    }
}

/// 时序节点（扩展 GraphNode）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalNode {
    /// 基础节点信息
    pub node: GraphNode,
    /// 有效时间范围
    pub valid_time: TimeRange,
    /// 事务时间（记录创建时间）
    pub transaction_time: DateTime<Utc>,
    /// 节点版本
    pub version: u32,
    /// 前一个版本的ID
    pub previous_version: Option<MemoryId>,
}

impl TemporalNode {
    /// 创建新的时序节点
    pub fn new(node: GraphNode, valid_time: TimeRange) -> Self {
        Self {
            node,
            valid_time,
            transaction_time: Utc::now(),
            version: 1,
            previous_version: None,
        }
    }

    /// 创建节点的新版本
    pub fn new_version(&self, node: GraphNode, valid_time: TimeRange) -> Self {
        Self {
            node,
            valid_time,
            transaction_time: Utc::now(),
            version: self.version + 1,
            previous_version: Some(self.node.id.clone()),
        }
    }
}

/// 时序边（扩展 GraphEdge）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEdge {
    /// 基础边信息
    pub edge: GraphEdge,
    /// 有效时间范围
    pub valid_time: TimeRange,
    /// 关系强度随时间的变化
    pub strength_history: Vec<(DateTime<Utc>, f32)>,
    /// 边版本
    pub version: u32,
    /// 前一个版本的ID
    pub previous_version: Option<Uuid>,
}

impl TemporalEdge {
    /// 创建新的时序边
    pub fn new(edge: GraphEdge, valid_time: TimeRange) -> Self {
        let initial_strength = (edge.created_at, edge.weight);
        Self {
            edge,
            valid_time,
            strength_history: vec![initial_strength],
            version: 1,
            previous_version: None,
        }
    }

    /// 更新关系强度
    pub fn update_strength(&mut self, new_strength: f32) {
        self.edge.weight = new_strength;
        self.strength_history.push((Utc::now(), new_strength));
    }

    /// 获取指定时间点的关系强度
    pub fn get_strength_at(&self, time: &DateTime<Utc>) -> Option<f32> {
        // 找到最接近但不晚于指定时间的强度值
        self.strength_history
            .iter()
            .filter(|(t, _)| t <= time)
            .next_back()
            .map(|(_, strength)| *strength)
    }

    /// 计算关系强度的变化趋势
    pub fn strength_trend(&self) -> f32 {
        if self.strength_history.len() < 2 {
            return 0.0;
        }

        let first = self.strength_history.first()
            .expect("strength_history is already checked to have at least 2 elements")
            .1;
        let last = self.strength_history.last()
            .expect("strength_history is already checked to have at least 2 elements")
            .1;
        last - first
    }
}

/// 时间窗口查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindowQuery {
    /// 查询的时间范围
    pub time_range: TimeRange,
    /// 是否包括已结束的关系
    pub include_ended: bool,
    /// 是否包括未来的关系
    pub include_future: bool,
    /// 最小关系强度阈值
    pub min_strength: Option<f32>,
}

impl Default for TimeWindowQuery {
    fn default() -> Self {
        Self {
            time_range: TimeRange::from(Utc::now() - Duration::days(30)),
            include_ended: true,
            include_future: false,
            min_strength: None,
        }
    }
}

/// 关系演化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipEvolution {
    /// 关系创建
    Created {
        edge_id: Uuid,
        time: DateTime<Utc>,
        initial_strength: f32,
    },
    /// 关系强度变化
    StrengthChanged {
        edge_id: Uuid,
        time: DateTime<Utc>,
        old_strength: f32,
        new_strength: f32,
    },
    /// 关系结束
    Ended {
        edge_id: Uuid,
        time: DateTime<Utc>,
        final_strength: f32,
    },
    /// 关系恢复
    Resumed {
        edge_id: Uuid,
        time: DateTime<Utc>,
        strength: f32,
    },
}

/// 时序图记忆引擎
pub struct TemporalGraphEngine {
    /// 基础图引擎
    base_engine: Arc<GraphMemoryEngine>,
    /// 时序节点存储
    temporal_nodes: Arc<RwLock<HashMap<MemoryId, Vec<TemporalNode>>>>,
    /// 时序边存储
    temporal_edges: Arc<RwLock<HashMap<Uuid, Vec<TemporalEdge>>>>,
    /// 关系演化历史
    evolution_history: Arc<RwLock<Vec<RelationshipEvolution>>>,
    /// 时间索引（按时间范围索引节点）
    time_index: Arc<RwLock<HashMap<String, HashSet<MemoryId>>>>,
}

impl TemporalGraphEngine {
    /// 创建新的时序图引擎
    pub fn new(base_engine: Arc<GraphMemoryEngine>) -> Self {
        Self {
            base_engine,
            temporal_nodes: Arc::new(RwLock::new(HashMap::new())),
            temporal_edges: Arc::new(RwLock::new(HashMap::new())),
            evolution_history: Arc::new(RwLock::new(Vec::new())),
            time_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加时序节点
    pub async fn add_temporal_node(
        &self,
        node: GraphNode,
        valid_time: TimeRange,
    ) -> Result<MemoryId> {
        let temporal_node = TemporalNode::new(node.clone(), valid_time.clone());
        let node_id = node.id.clone();

        // 存储时序节点
        self.temporal_nodes
            .write()
            .await
            .entry(node_id.clone())
            .or_insert_with(Vec::new)
            .push(temporal_node);

        // 更新时间索引
        self.update_time_index(&node_id, &valid_time).await;

        Ok(node_id)
    }

    /// 添加时序边
    pub async fn add_temporal_edge(&self, edge: GraphEdge, valid_time: TimeRange) -> Result<Uuid> {
        let edge_id = edge.id;
        let initial_strength = edge.weight;
        let temporal_edge = TemporalEdge::new(edge, valid_time);

        // 存储时序边
        self.temporal_edges
            .write()
            .await
            .entry(edge_id)
            .or_insert_with(Vec::new)
            .push(temporal_edge);

        // 记录演化事件
        self.evolution_history
            .write()
            .await
            .push(RelationshipEvolution::Created {
                edge_id,
                time: Utc::now(),
                initial_strength,
            });

        Ok(edge_id)
    }

    /// 更新时间索引
    async fn update_time_index(&self, node_id: &MemoryId, time_range: &TimeRange) {
        let time_key = self.time_range_to_key(time_range);
        self.time_index
            .write()
            .await
            .entry(time_key)
            .or_insert_with(HashSet::new)
            .insert(node_id.clone());
    }

    /// 将时间范围转换为索引键
    fn time_range_to_key(&self, time_range: &TimeRange) -> String {
        format!(
            "{}_{}",
            time_range.start.timestamp(),
            time_range.end.map(|e| e.timestamp()).unwrap_or(-1)
        )
    }

    /// 时间窗口查询节点
    pub async fn query_nodes_in_window(
        &self,
        query: &TimeWindowQuery,
    ) -> Result<Vec<TemporalNode>> {
        let nodes = self.temporal_nodes.read().await;
        let mut results = Vec::new();

        for versions in nodes.values() {
            for node in versions {
                // 检查时间范围是否重叠
                if !node.valid_time.overlaps(&query.time_range) {
                    continue;
                }

                // 检查是否包括已结束的节点
                if !query.include_ended && !node.valid_time.is_active() {
                    continue;
                }

                // 检查是否包括未来的节点
                if !query.include_future && node.valid_time.start > Utc::now() {
                    continue;
                }

                results.push(node.clone());
            }
        }

        Ok(results)
    }

    /// 时间窗口查询边
    pub async fn query_edges_in_window(
        &self,
        query: &TimeWindowQuery,
    ) -> Result<Vec<TemporalEdge>> {
        let edges = self.temporal_edges.read().await;
        let mut results = Vec::new();

        for versions in edges.values() {
            for edge in versions {
                // 检查时间范围是否重叠
                if !edge.valid_time.overlaps(&query.time_range) {
                    continue;
                }

                // 检查是否包括已结束的边
                if !query.include_ended && !edge.valid_time.is_active() {
                    continue;
                }

                // 检查是否包括未来的边
                if !query.include_future && edge.valid_time.start > Utc::now() {
                    continue;
                }

                // 检查关系强度阈值
                if let Some(min_strength) = query.min_strength {
                    if edge.edge.weight < min_strength {
                        continue;
                    }
                }

                results.push(edge.clone());
            }
        }

        Ok(results)
    }

    /// 更新边的关系强度
    pub async fn update_edge_strength(&self, edge_id: Uuid, new_strength: f32) -> Result<()> {
        let mut edges = self.temporal_edges.write().await;

        if let Some(versions) = edges.get_mut(&edge_id) {
            if let Some(latest) = versions.last_mut() {
                let old_strength = latest.edge.weight;
                latest.update_strength(new_strength);

                // 记录演化事件
                self.evolution_history
                    .write()
                    .await
                    .push(RelationshipEvolution::StrengthChanged {
                        edge_id,
                        time: Utc::now(),
                        old_strength,
                        new_strength,
                    });

                return Ok(());
            }
        }

        Err(anyhow!("Edge not found: {}", edge_id))
    }

    /// 结束边的有效期
    pub async fn end_edge(&self, edge_id: Uuid) -> Result<()> {
        let mut edges = self.temporal_edges.write().await;

        if let Some(versions) = edges.get_mut(&edge_id) {
            if let Some(latest) = versions.last_mut() {
                let final_strength = latest.edge.weight;
                latest.valid_time.end = Some(Utc::now());

                // 记录演化事件
                self.evolution_history
                    .write()
                    .await
                    .push(RelationshipEvolution::Ended {
                        edge_id,
                        time: Utc::now(),
                        final_strength,
                    });

                return Ok(());
            }
        }

        Err(anyhow!("Edge not found: {}", edge_id))
    }

    /// 恢复已结束的边
    pub async fn resume_edge(
        &self,
        edge_id: Uuid,
        new_strength: f32,
        valid_from: DateTime<Utc>,
    ) -> Result<()> {
        let mut edges = self.temporal_edges.write().await;

        if let Some(versions) = edges.get_mut(&edge_id) {
            if let Some(latest) = versions.last_mut() {
                // 创建新版本
                let mut new_edge = latest.edge.clone();
                new_edge.weight = new_strength;
                new_edge.created_at = valid_from;

                let new_temporal_edge = TemporalEdge {
                    edge: new_edge,
                    valid_time: TimeRange::from(valid_from),
                    strength_history: vec![(valid_from, new_strength)],
                    version: latest.version + 1,
                    previous_version: Some(edge_id),
                };

                versions.push(new_temporal_edge);

                // 记录演化事件
                self.evolution_history
                    .write()
                    .await
                    .push(RelationshipEvolution::Resumed {
                        edge_id,
                        time: valid_from,
                        strength: new_strength,
                    });

                return Ok(());
            }
        }

        Err(anyhow!("Edge not found: {}", edge_id))
    }

    /// 获取关系演化历史
    pub async fn get_evolution_history(
        &self,
        edge_id: Option<Uuid>,
        time_range: Option<TimeRange>,
    ) -> Vec<RelationshipEvolution> {
        let history = self.evolution_history.read().await;

        history
            .iter()
            .filter(|event| {
                // 过滤边ID
                if let Some(id) = edge_id {
                    match event {
                        RelationshipEvolution::Created { edge_id: eid, .. }
                        | RelationshipEvolution::StrengthChanged { edge_id: eid, .. }
                        | RelationshipEvolution::Ended { edge_id: eid, .. }
                        | RelationshipEvolution::Resumed { edge_id: eid, .. } => {
                            if *eid != id {
                                return false;
                            }
                        }
                    }
                }

                // 过滤时间范围
                if let Some(ref range) = time_range {
                    let event_time = match event {
                        RelationshipEvolution::Created { time, .. }
                        | RelationshipEvolution::StrengthChanged { time, .. }
                        | RelationshipEvolution::Ended { time, .. }
                        | RelationshipEvolution::Resumed { time, .. } => time,
                    };

                    if !range.contains(event_time) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// 获取节点的所有版本
    pub async fn get_node_versions(&self, node_id: &MemoryId) -> Option<Vec<TemporalNode>> {
        self.temporal_nodes.read().await.get(node_id).cloned()
    }

    /// 获取边的所有版本
    pub async fn get_edge_versions(&self, edge_id: Uuid) -> Option<Vec<TemporalEdge>> {
        self.temporal_edges.read().await.get(&edge_id).cloned()
    }

    /// 获取指定时间点的节点快照
    pub async fn get_node_at_time(
        &self,
        node_id: &MemoryId,
        time: &DateTime<Utc>,
    ) -> Option<TemporalNode> {
        if let Some(versions) = self.temporal_nodes.read().await.get(node_id) {
            // 找到在指定时间有效的版本
            versions
                .iter()
                .filter(|node| node.valid_time.contains(time))
                .next_back()
                .cloned()
        } else {
            None
        }
    }

    /// 获取指定时间点的边快照
    pub async fn get_edge_at_time(
        &self,
        edge_id: Uuid,
        time: &DateTime<Utc>,
    ) -> Option<TemporalEdge> {
        if let Some(versions) = self.temporal_edges.read().await.get(&edge_id) {
            // 找到在指定时间有效的版本
            versions
                .iter()
                .filter(|edge| edge.valid_time.contains(time))
                .next_back()
                .cloned()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_memory::NodeType;
    use crate::types::{Memory, MemoryType};
    use agent_mem_traits::Vector;

    #[tokio::test]
    async fn test_time_range() {
        let now = Utc::now();
        let future = now + Duration::hours(1);

        let range = TimeRange::new(now, Some(future));
        assert!(range.contains(&now));
        assert!(range.contains(&future));
        assert!(!range.contains(&(future + Duration::hours(1))));
    }

    #[tokio::test]
    async fn test_time_range_overlap() {
        let now = Utc::now();
        let range1 = TimeRange::new(now, Some(now + Duration::hours(2)));
        let range2 = TimeRange::new(now + Duration::hours(1), Some(now + Duration::hours(3)));

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
    }

    #[tokio::test]
    async fn test_temporal_node() {
        // 使用V4 API创建Memory
        let memory = Memory::new(
            "test_agent".to_string(),
            Some("user1".to_string()),
            MemoryType::Semantic,
            "Test content".to_string(),
            0.8,
        );

        let node = GraphNode {
            id: "node1".to_string(),
            memory,
            node_type: NodeType::Entity,
            properties: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let valid_time = TimeRange::from(Utc::now());
        let temporal_node = TemporalNode::new(node, valid_time);

        assert_eq!(temporal_node.version, 1);
        assert!(temporal_node.previous_version.is_none());
    }
}
