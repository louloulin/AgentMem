/// 时序推理引擎模块
///
/// 实现时序知识图谱的高级推理能力，包括：
/// - 时序逻辑推理
/// - 因果关系推断
/// - 多跳时序推理
/// - 反事实推理
/// - 时序模式识别
/// - 预测性推理
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::graph_memory::MemoryId;
use crate::temporal_graph::{TemporalGraphEngine, TemporalNode, TimeRange};

/// 时序推理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemporalReasoningType {
    /// 时序逻辑推理（基于时间顺序）
    TemporalLogic,
    /// 因果推理（原因->结果）
    Causal,
    /// 多跳推理（多步推理链）
    MultiHop,
    /// 反事实推理（假设性推理）
    Counterfactual,
    /// 预测性推理（未来预测）
    Predictive,
}

/// 时序推理路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalReasoningPath {
    /// 节点序列
    pub nodes: Vec<MemoryId>,
    /// 边序列
    pub edges: Vec<Uuid>,
    /// 时间序列（每个节点的有效时间）
    pub timestamps: Vec<DateTime<Utc>>,
    /// 推理类型
    pub reasoning_type: TemporalReasoningType,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 推理解释
    pub explanation: String,
}

/// 因果关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalRelation {
    /// 原因节点
    pub cause: MemoryId,
    /// 结果节点
    pub effect: MemoryId,
    /// 因果强度 (0.0-1.0)
    pub strength: f32,
    /// 时间延迟
    pub time_delay: Duration,
    /// 置信度
    pub confidence: f32,
    /// 支持证据
    pub evidence: Vec<Uuid>,
}

/// 时序模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPattern {
    /// 模式类型
    pub pattern_type: PatternType,
    /// 涉及的节点
    pub nodes: Vec<MemoryId>,
    /// 时间间隔
    pub time_intervals: Vec<Duration>,
    /// 模式频率
    pub frequency: usize,
    /// 置信度
    pub confidence: f32,
    /// 模式描述
    pub description: String,
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternType {
    /// 周期性模式
    Periodic,
    /// 序列模式
    Sequential,
    /// 并发模式
    Concurrent,
    /// 因果链模式
    CausalChain,
}

/// 反事实场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualScenario {
    /// 原始事件
    pub original_event: MemoryId,
    /// 假设的改变
    pub hypothetical_change: String,
    /// 预测的结果
    pub predicted_outcomes: Vec<MemoryId>,
    /// 置信度
    pub confidence: f32,
    /// 推理依据
    pub reasoning: String,
}

/// 预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// 预测的事件
    pub predicted_event: String,
    /// 预测时间
    pub predicted_time: DateTime<Utc>,
    /// 置信度
    pub confidence: f32,
    /// 基于的模式
    pub based_on_patterns: Vec<TemporalPattern>,
    /// 推理路径
    pub reasoning_path: Option<TemporalReasoningPath>,
}

/// 时序推理引擎
pub struct TemporalReasoningEngine {
    /// 时序图引擎
    temporal_graph: Arc<TemporalGraphEngine>,
    /// 因果关系缓存
    causal_relations: Arc<RwLock<HashMap<(MemoryId, MemoryId), CausalRelation>>>,
    /// 时序模式缓存
    temporal_patterns: Arc<RwLock<Vec<TemporalPattern>>>,
    /// 推理配置
    config: TemporalReasoningConfig,
}

/// 时序推理配置
#[derive(Debug, Clone)]
pub struct TemporalReasoningConfig {
    /// 最大推理深度
    pub max_reasoning_depth: usize,
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 最大时间窗口
    pub max_time_window: Duration,
    /// 因果关系最小强度
    pub min_causal_strength: f32,
    /// 模式识别最小频率
    pub min_pattern_frequency: usize,
}

impl Default for TemporalReasoningConfig {
    fn default() -> Self {
        Self {
            max_reasoning_depth: 5,
            min_confidence: 0.6,
            max_time_window: Duration::days(365),
            min_causal_strength: 0.5,
            min_pattern_frequency: 3,
        }
    }
}

impl TemporalReasoningEngine {
    /// 创建新的时序推理引擎
    pub fn new(temporal_graph: Arc<TemporalGraphEngine>) -> Self {
        Self::with_config(temporal_graph, TemporalReasoningConfig::default())
    }

    /// 使用自定义配置创建引擎
    pub fn with_config(
        temporal_graph: Arc<TemporalGraphEngine>,
        config: TemporalReasoningConfig,
    ) -> Self {
        Self {
            temporal_graph,
            causal_relations: Arc::new(RwLock::new(HashMap::new())),
            temporal_patterns: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// 时序逻辑推理
    ///
    /// 基于时间顺序进行推理，找出事件之间的时序关系
    pub async fn temporal_logic_reasoning(
        &self,
        start_node: &MemoryId,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TemporalReasoningPath>> {
        let mut paths = Vec::new();

        // 获取起始节点的时序版本
        let start_versions = self
            .temporal_graph
            .get_node_versions(start_node)
            .await
            .ok_or_else(|| anyhow!("Start node not found"))?;

        // 对每个版本进行时序推理
        for start_version in start_versions {
            if start_version.valid_time.start > end_time {
                continue;
            }

            // 查找在时间窗口内的相关节点
            let time_range = TimeRange::new(start_version.valid_time.start, Some(end_time));
            let related_nodes = self
                .temporal_graph
                .query_nodes_in_window(&crate::temporal_graph::TimeWindowQuery {
                    time_range,
                    include_ended: true,
                    include_future: false,
                    min_strength: None,
                })
                .await?;

            // 构建时序推理路径
            for node in related_nodes {
                if node.node.id == *start_node {
                    continue;
                }

                let time_diff = node.valid_time.start - start_version.valid_time.start;
                if time_diff.num_seconds() > 0 {
                    // 找到时间上后续的节点
                    let path = TemporalReasoningPath {
                        nodes: vec![start_node.clone(), node.node.id.clone()],
                        edges: vec![],
                        timestamps: vec![start_version.valid_time.start, node.valid_time.start],
                        reasoning_type: TemporalReasoningType::TemporalLogic,
                        confidence: self.calculate_temporal_confidence(&time_diff),
                        explanation: format!(
                            "Event {} occurred {} after event {}",
                            node.node.id,
                            self.format_duration(&time_diff),
                            start_node
                        ),
                    };
                    paths.push(path);
                }
            }
        }

        Ok(paths)
    }

    /// 计算时序置信度
    fn calculate_temporal_confidence(&self, time_diff: &Duration) -> f32 {
        // 时间差越小，置信度越高
        let days = time_diff.num_days().abs() as f32;
        let max_days = self.config.max_time_window.num_days() as f32;

        (1.0 - (days / max_days)).max(0.0).min(1.0)
    }

    /// 格式化时间间隔
    fn format_duration(&self, duration: &Duration) -> String {
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;

        if days > 0 {
            format!("{days} days {hours} hours")
        } else if hours > 0 {
            format!("{hours} hours {minutes} minutes")
        } else {
            format!("{minutes} minutes")
        }
    }

    /// 因果关系推断
    ///
    /// 分析事件之间的因果关系，识别原因和结果
    pub async fn causal_inference(
        &self,
        cause_node: &MemoryId,
        effect_node: &MemoryId,
    ) -> Result<Option<CausalRelation>> {
        // 检查缓存
        let cache_key = (cause_node.clone(), effect_node.clone());
        {
            let cache = self.causal_relations.read().await;
            if let Some(relation) = cache.get(&cache_key) {
                return Ok(Some(relation.clone()));
            }
        }

        // 获取节点的时序版本
        let cause_versions = self
            .temporal_graph
            .get_node_versions(cause_node)
            .await
            .ok_or_else(|| anyhow!("Cause node not found"))?;

        let effect_versions = self
            .temporal_graph
            .get_node_versions(effect_node)
            .await
            .ok_or_else(|| anyhow!("Effect node not found"))?;

        // 分析时间关系
        let mut best_relation: Option<CausalRelation> = None;
        let mut max_strength = 0.0;

        for cause_v in &cause_versions {
            for effect_v in &effect_versions {
                // 原因必须在结果之前
                if cause_v.valid_time.start >= effect_v.valid_time.start {
                    continue;
                }

                let time_delay = effect_v.valid_time.start - cause_v.valid_time.start;

                // 计算因果强度
                let strength = self
                    .calculate_causal_strength(cause_v, effect_v, &time_delay)
                    .await?;

                if strength > max_strength && strength >= self.config.min_causal_strength {
                    max_strength = strength;

                    // 查找支持证据（连接两个节点的边）
                    let evidence = self.find_causal_evidence(cause_node, effect_node).await?;

                    best_relation = Some(CausalRelation {
                        cause: cause_node.clone(),
                        effect: effect_node.clone(),
                        strength,
                        time_delay,
                        confidence: self.calculate_causal_confidence(strength, &time_delay),
                        evidence,
                    });
                }
            }
        }

        // 缓存结果
        if let Some(ref relation) = best_relation {
            self.causal_relations
                .write()
                .await
                .insert(cache_key, relation.clone());
        }

        Ok(best_relation)
    }

    /// 计算因果强度
    async fn calculate_causal_strength(
        &self,
        cause: &TemporalNode,
        effect: &TemporalNode,
        time_delay: &Duration,
    ) -> Result<f32> {
        let mut strength = 0.0;

        // 1. 时间接近性（时间越近，因果关系越强）
        let time_score = self.calculate_temporal_confidence(time_delay);
        strength += time_score * 0.4;

        // 2. 节点类型相关性
        let type_score = if cause.node.node_type == effect.node.node_type {
            0.8
        } else {
            0.5
        };
        strength += type_score * 0.3;

        // 3. 关系强度（如果存在直接连接）
        let relation_score = self
            .get_direct_relation_strength(&cause.node.id, &effect.node.id)
            .await
            .unwrap_or(0.5);
        strength += relation_score * 0.3;

        Ok(strength.min(1.0))
    }

    /// 获取直接关系强度
    async fn get_direct_relation_strength(&self, from: &MemoryId, to: &MemoryId) -> Option<f32> {
        // 查询时间窗口内的边
        let edges = self
            .temporal_graph
            .query_edges_in_window(&crate::temporal_graph::TimeWindowQuery {
                time_range: TimeRange::from(Utc::now() - self.config.max_time_window),
                include_ended: true,
                include_future: false,
                min_strength: None,
            })
            .await
            .ok()?;

        // 查找连接两个节点的边
        for edge in edges {
            if edge.edge.from_node == *from && edge.edge.to_node == *to {
                return Some(edge.edge.weight);
            }
        }

        None
    }

    /// 查找因果证据
    async fn find_causal_evidence(&self, cause: &MemoryId, effect: &MemoryId) -> Result<Vec<Uuid>> {
        let mut evidence = Vec::new();

        let edges = self
            .temporal_graph
            .query_edges_in_window(&crate::temporal_graph::TimeWindowQuery {
                time_range: TimeRange::from(Utc::now() - self.config.max_time_window),
                include_ended: true,
                include_future: false,
                min_strength: None,
            })
            .await?;

        for edge in edges {
            if edge.edge.from_node == *cause && edge.edge.to_node == *effect {
                evidence.push(edge.edge.id);
            }
        }

        Ok(evidence)
    }

    /// 计算因果置信度
    fn calculate_causal_confidence(&self, strength: f32, time_delay: &Duration) -> f32 {
        let time_confidence = self.calculate_temporal_confidence(time_delay);
        (strength + time_confidence) / 2.0
    }

    /// 多跳时序推理
    ///
    /// 执行多步推理，找出复杂的推理链
    pub async fn multi_hop_reasoning(
        &self,
        start_node: &MemoryId,
        target_node: &MemoryId,
        max_hops: usize,
    ) -> Result<Vec<TemporalReasoningPath>> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // 初始化队列
        queue.push_back((
            start_node.clone(),
            vec![start_node.clone()],
            vec![],
            vec![Utc::now()],
            0,
        ));
        visited.insert(start_node.clone());

        while let Some((current, node_path, edge_path, time_path, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            // 找到目标节点
            if current == *target_node {
                let confidence = self.calculate_path_confidence(&node_path, &time_path);

                if confidence >= self.config.min_confidence {
                    paths.push(TemporalReasoningPath {
                        nodes: node_path.clone(),
                        edges: edge_path.clone(),
                        timestamps: time_path.clone(),
                        reasoning_type: TemporalReasoningType::MultiHop,
                        confidence,
                        explanation: format!(
                            "Multi-hop reasoning path with {} hops",
                            node_path.len() - 1
                        ),
                    });
                }
                continue;
            }

            // 获取当前节点的时序版本
            if let Some(versions) = self.temporal_graph.get_node_versions(&current).await {
                if let Some(current_version) = versions.last() {
                    // 查找后续节点
                    let time_range = TimeRange::new(
                        current_version.valid_time.start,
                        Some(current_version.valid_time.start + self.config.max_time_window),
                    );

                    if let Ok(related_nodes) = self
                        .temporal_graph
                        .query_nodes_in_window(&crate::temporal_graph::TimeWindowQuery {
                            time_range,
                            include_ended: true,
                            include_future: false,
                            min_strength: None,
                        })
                        .await
                    {
                        for next_node in related_nodes {
                            if visited.contains(&next_node.node.id) {
                                continue;
                            }

                            let mut new_node_path = node_path.clone();
                            new_node_path.push(next_node.node.id.clone());

                            let mut new_time_path = time_path.clone();
                            new_time_path.push(next_node.valid_time.start);

                            queue.push_back((
                                next_node.node.id.clone(),
                                new_node_path,
                                edge_path.clone(),
                                new_time_path,
                                depth + 1,
                            ));

                            visited.insert(next_node.node.id.clone());
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    /// 计算路径置信度
    fn calculate_path_confidence(&self, nodes: &[MemoryId], timestamps: &[DateTime<Utc>]) -> f32 {
        if nodes.len() < 2 || timestamps.len() < 2 {
            return 0.0;
        }

        let mut total_confidence = 0.0;
        let mut count = 0;

        for i in 0..timestamps.len() - 1 {
            let time_diff = timestamps[i + 1] - timestamps[i];
            total_confidence += self.calculate_temporal_confidence(&time_diff);
            count += 1;
        }

        if count > 0 {
            total_confidence / count as f32
        } else {
            0.0
        }
    }

    /// 反事实推理
    ///
    /// 分析"如果...会怎样"的假设场景
    pub async fn counterfactual_reasoning(
        &self,
        original_event: &MemoryId,
        hypothetical_change: String,
    ) -> Result<CounterfactualScenario> {
        // 获取原始事件
        let original_versions = self
            .temporal_graph
            .get_node_versions(original_event)
            .await
            .ok_or_else(|| anyhow!("Original event not found"))?;

        let original = original_versions
            .last()
            .ok_or_else(|| anyhow!("No version found"))?;

        // 查找原始事件之后的相关事件
        let time_range = TimeRange::new(
            original.valid_time.start,
            Some(original.valid_time.start + Duration::days(30)),
        );

        let subsequent_events = self
            .temporal_graph
            .query_nodes_in_window(&crate::temporal_graph::TimeWindowQuery {
                time_range,
                include_ended: true,
                include_future: false,
                min_strength: None,
            })
            .await?;

        // 分析哪些事件可能受到影响
        let mut predicted_outcomes = Vec::new();
        let mut total_confidence = 0.0;
        let mut count = 0;

        for event in subsequent_events {
            if event.node.id == *original_event {
                continue;
            }

            // 检查是否存在因果关系
            if let Ok(Some(causal)) = self.causal_inference(original_event, &event.node.id).await {
                if causal.strength >= self.config.min_causal_strength {
                    predicted_outcomes.push(event.node.id.clone());
                    total_confidence += causal.confidence;
                    count += 1;
                }
            }
        }

        let confidence = if count > 0 {
            total_confidence / count as f32
        } else {
            0.5
        };

        let num_outcomes = predicted_outcomes.len();

        Ok(CounterfactualScenario {
            original_event: original_event.clone(),
            hypothetical_change,
            predicted_outcomes,
            confidence,
            reasoning: format!(
                "If the original event changed, {num_outcomes} subsequent events might be affected"
            ),
        })
    }

    /// 时序模式识别
    ///
    /// 识别重复出现的时序模式
    pub async fn identify_temporal_patterns(
        &self,
        time_range: TimeRange,
    ) -> Result<Vec<TemporalPattern>> {
        let mut patterns = Vec::new();

        // 获取时间窗口内的所有节点
        let nodes = self
            .temporal_graph
            .query_nodes_in_window(&crate::temporal_graph::TimeWindowQuery {
                time_range: time_range.clone(),
                include_ended: true,
                include_future: false,
                min_strength: None,
            })
            .await?;

        // 按时间排序
        let mut sorted_nodes = nodes;
        sorted_nodes.sort_by_key(|n| n.valid_time.start);

        // 识别序列模式
        let sequential_patterns = self.identify_sequential_patterns(&sorted_nodes).await?;
        patterns.extend(sequential_patterns);

        // 识别周期性模式
        let periodic_patterns = self.identify_periodic_patterns(&sorted_nodes).await?;
        patterns.extend(periodic_patterns);

        // 识别因果链模式
        let causal_patterns = self.identify_causal_chain_patterns(&sorted_nodes).await?;
        patterns.extend(causal_patterns);

        // 缓存模式
        self.temporal_patterns
            .write()
            .await
            .extend(patterns.clone());

        Ok(patterns)
    }

    /// 识别序列模式
    async fn identify_sequential_patterns(
        &self,
        nodes: &[TemporalNode],
    ) -> Result<Vec<TemporalPattern>> {
        let mut patterns = Vec::new();

        // 查找重复的节点类型序列
        let window_size = 3;
        for i in 0..nodes.len().saturating_sub(window_size) {
            let window = &nodes[i..i + window_size];

            // 计算时间间隔
            let mut intervals = Vec::new();
            for j in 0..window.len() - 1 {
                intervals.push(window[j + 1].valid_time.start - window[j].valid_time.start);
            }

            // 检查是否有相似的序列
            let frequency = self.count_similar_sequences(nodes, window, &intervals);

            if frequency >= self.config.min_pattern_frequency {
                patterns.push(TemporalPattern {
                    pattern_type: PatternType::Sequential,
                    nodes: window.iter().map(|n| n.node.id.clone()).collect(),
                    time_intervals: intervals,
                    frequency,
                    confidence: (frequency as f32 / nodes.len() as f32).min(1.0),
                    description: format!("Sequential pattern with {frequency} occurrences"),
                });
            }
        }

        Ok(patterns)
    }

    /// 计算相似序列的数量
    fn count_similar_sequences(
        &self,
        all_nodes: &[TemporalNode],
        pattern: &[TemporalNode],
        intervals: &[Duration],
    ) -> usize {
        let mut count = 0;
        let tolerance = Duration::hours(1);

        for i in 0..all_nodes.len().saturating_sub(pattern.len()) {
            let window = &all_nodes[i..i + pattern.len()];

            // 检查节点类型是否匹配
            let types_match = window
                .iter()
                .zip(pattern.iter())
                .all(|(a, b)| a.node.node_type == b.node.node_type);

            if !types_match {
                continue;
            }

            // 检查时间间隔是否相似
            let mut intervals_match = true;
            for j in 0..window.len() - 1 {
                let actual_interval = window[j + 1].valid_time.start - window[j].valid_time.start;
                let expected_interval = intervals[j];
                let diff = (actual_interval - expected_interval).num_seconds().abs();

                if diff > tolerance.num_seconds() {
                    intervals_match = false;
                    break;
                }
            }

            if intervals_match {
                count += 1;
            }
        }

        count
    }

    /// 识别周期性模式
    async fn identify_periodic_patterns(
        &self,
        nodes: &[TemporalNode],
    ) -> Result<Vec<TemporalPattern>> {
        let mut patterns = Vec::new();

        // 按节点类型分组
        let mut type_groups: HashMap<String, Vec<&TemporalNode>> = HashMap::new();
        for node in nodes {
            type_groups
                .entry(format!("{:?}", node.node.node_type))
                .or_default()
                .push(node);
        }

        // 对每个类型检查周期性
        for (node_type, group) in type_groups {
            if group.len() < self.config.min_pattern_frequency {
                continue;
            }

            // 计算时间间隔
            let mut intervals = Vec::new();
            for i in 0..group.len() - 1 {
                intervals.push(group[i + 1].valid_time.start - group[i].valid_time.start);
            }

            // 检查间隔是否相似（周期性）
            if let Some(avg_interval) = self.calculate_average_interval(&intervals) {
                let is_periodic = self.check_periodicity(&intervals, &avg_interval);

                if is_periodic {
                    patterns.push(TemporalPattern {
                        pattern_type: PatternType::Periodic,
                        nodes: group.iter().map(|n| n.node.id.clone()).collect(),
                        time_intervals: vec![avg_interval],
                        frequency: group.len(),
                        confidence: 0.8,
                        description: format!(
                            "Periodic pattern for {} with interval {}",
                            node_type,
                            self.format_duration(&avg_interval)
                        ),
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// 计算平均时间间隔
    fn calculate_average_interval(&self, intervals: &[Duration]) -> Option<Duration> {
        if intervals.is_empty() {
            return None;
        }

        let total_seconds: i64 = intervals.iter().map(|d| d.num_seconds()).sum();
        let avg_seconds = total_seconds / intervals.len() as i64;

        Some(Duration::seconds(avg_seconds))
    }

    /// 检查周期性
    fn check_periodicity(&self, intervals: &[Duration], avg_interval: &Duration) -> bool {
        let tolerance = avg_interval.num_seconds() / 10; // 10% tolerance

        intervals.iter().all(|interval| {
            let diff = (interval.num_seconds() - avg_interval.num_seconds()).abs();
            diff <= tolerance
        })
    }

    /// 识别因果链模式
    async fn identify_causal_chain_patterns(
        &self,
        nodes: &[TemporalNode],
    ) -> Result<Vec<TemporalPattern>> {
        let mut patterns = Vec::new();

        // 查找因果链
        for i in 0..nodes.len().saturating_sub(2) {
            let chain = &nodes[i..i + 3];

            // 检查是否形成因果链
            let mut is_causal_chain = true;
            let mut intervals = Vec::new();

            for j in 0..chain.len() - 1 {
                if let Ok(Some(causal)) = self
                    .causal_inference(&chain[j].node.id, &chain[j + 1].node.id)
                    .await
                {
                    if causal.strength >= self.config.min_causal_strength {
                        intervals.push(causal.time_delay);
                    } else {
                        is_causal_chain = false;
                        break;
                    }
                } else {
                    is_causal_chain = false;
                    break;
                }
            }

            if is_causal_chain {
                patterns.push(TemporalPattern {
                    pattern_type: PatternType::CausalChain,
                    nodes: chain.iter().map(|n| n.node.id.clone()).collect(),
                    time_intervals: intervals,
                    frequency: 1,
                    confidence: 0.9,
                    description: "Causal chain pattern detected".to_string(),
                });
            }
        }

        Ok(patterns)
    }

    /// 预测性推理
    ///
    /// 基于历史模式预测未来事件
    pub async fn predictive_reasoning(
        &self,
        current_context: &[MemoryId],
    ) -> Result<Vec<PredictionResult>> {
        let mut predictions = Vec::new();

        // 获取已识别的模式
        let patterns = self.temporal_patterns.read().await;

        for pattern in patterns.iter() {
            // 检查当前上下文是否匹配模式的开始
            if self.context_matches_pattern_start(current_context, pattern) {
                // 预测下一个事件
                if let Some(next_event) = pattern.nodes.last() {
                    let predicted_time = self.predict_next_event_time(pattern);

                    predictions.push(PredictionResult {
                        predicted_event: format!("Event similar to {next_event}"),
                        predicted_time,
                        confidence: pattern.confidence,
                        based_on_patterns: vec![pattern.clone()],
                        reasoning_path: None,
                    });
                }
            }
        }

        Ok(predictions)
    }

    /// 检查上下文是否匹配模式开始
    fn context_matches_pattern_start(
        &self,
        context: &[MemoryId],
        pattern: &TemporalPattern,
    ) -> bool {
        if context.is_empty() || pattern.nodes.is_empty() {
            return false;
        }

        // 简化实现：检查最后一个上下文节点是否匹配模式的第一个节点
        context.last() == pattern.nodes.first()
    }

    /// 预测下一个事件的时间
    fn predict_next_event_time(&self, pattern: &TemporalPattern) -> DateTime<Utc> {
        let now = Utc::now();

        if let Some(avg_interval) = pattern.time_intervals.first() {
            now + *avg_interval
        } else {
            now + Duration::days(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_memory::GraphMemoryEngine;

    #[tokio::test]
    async fn test_temporal_reasoning_config() {
        let config = TemporalReasoningConfig::default();
        assert_eq!(config.max_reasoning_depth, 5);
        assert_eq!(config.min_confidence, 0.6);
    }

    #[tokio::test]
    async fn test_temporal_confidence_calculation() {
        let base_engine = Arc::new(GraphMemoryEngine::new());
        let temporal_graph = Arc::new(TemporalGraphEngine::new(base_engine));
        let engine = TemporalReasoningEngine::new(temporal_graph);

        let short_duration = Duration::hours(1);
        let confidence = engine.calculate_temporal_confidence(&short_duration);
        assert!(confidence > 0.9);

        let long_duration = Duration::days(365);
        let confidence = engine.calculate_temporal_confidence(&long_duration);
        assert!(confidence < 0.1);
    }

    #[tokio::test]
    async fn test_format_duration() {
        let base_engine = Arc::new(GraphMemoryEngine::new());
        let temporal_graph = Arc::new(TemporalGraphEngine::new(base_engine));
        let engine = TemporalReasoningEngine::new(temporal_graph);

        let duration = Duration::days(2) + Duration::hours(3);
        let formatted = engine.format_duration(&duration);
        assert!(formatted.contains("2 days"));
        assert!(formatted.contains("3 hours"));
    }
}
