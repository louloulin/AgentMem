//! Causal Reasoning Engine
//!
//! Phase 4.1: 因果推理引擎
//! - 因果知识图构建
//! - 因果推理引擎
//! - 因果链检索
//! - 因果解释生成
//!
//! 参考REMI架构，实现个人因果知识图和因果推理引擎

use agent_mem_traits::{Result, AgentMemError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// 因果知识图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    /// 节点ID
    pub id: String,
    /// 节点内容
    pub content: String,
    /// 节点类型（事件、状态、动作等）
    pub node_type: CausalNodeType,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 属性
    pub properties: HashMap<String, serde_json::Value>,
}

/// 因果节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CausalNodeType {
    /// 事件
    Event,
    /// 状态
    State,
    /// 动作
    Action,
    /// 条件
    Condition,
}

/// 因果关系边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    /// 边ID
    pub id: String,
    /// 原因节点ID
    pub cause_id: String,
    /// 结果节点ID
    pub effect_id: String,
    /// 因果强度 (0.0-1.0)
    pub strength: f32,
    /// 时间延迟
    pub time_delay_seconds: i64,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 支持证据
    pub evidence: Vec<String>,
    /// 因果关系类型
    pub relation_type: CausalRelationType,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 因果关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CausalRelationType {
    /// 直接因果（A 直接导致 B）
    Direct,
    /// 间接因果（A 通过中间步骤导致 B）
    Indirect,
    /// 必要条件（A 是 B 的必要条件）
    Necessary,
    /// 充分条件（A 是 B 的充分条件）
    Sufficient,
    /// 促进因素（A 促进 B 发生）
    Facilitating,
    /// 抑制因素（A 抑制 B 发生）
    Inhibiting,
}

/// 因果链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalChain {
    /// 链ID
    pub id: String,
    /// 节点序列
    pub nodes: Vec<String>,
    /// 边序列
    pub edges: Vec<String>,
    /// 总体置信度
    pub overall_confidence: f32,
    /// 链长度（跳数）
    pub length: usize,
    /// 推理解释
    pub explanation: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 因果推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReasoningResult {
    /// 推理链列表
    pub chains: Vec<CausalChain>,
    /// 最佳链
    pub best_chain: Option<CausalChain>,
    /// 总体置信度
    pub overall_confidence: f32,
    /// 推理解释
    pub explanation: String,
    /// 推理时间
    pub reasoned_at: DateTime<Utc>,
}

/// 因果推理引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReasoningConfig {
    /// 最大链长度
    pub max_chain_length: usize,
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 启用缓存
    pub enable_cache: bool,
    /// 缓存TTL（秒）
    pub cache_ttl: i64,
}

impl Default for CausalReasoningConfig {
    fn default() -> Self {
        Self {
            max_chain_length: 5,
            min_confidence: 0.5,
            enable_cache: true,
            cache_ttl: 3600,
        }
    }
}

/// 因果推理引擎
pub struct CausalReasoningEngine {
    /// 因果知识图节点
    nodes: Arc<RwLock<HashMap<String, CausalNode>>>,
    /// 因果关系边
    edges: Arc<RwLock<HashMap<String, CausalEdge>>>,
    /// 邻接表（从原因到结果）
    cause_to_effect: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 反向邻接表（从结果到原因）
    effect_to_cause: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 配置
    config: CausalReasoningConfig,
    /// 推理缓存
    reasoning_cache: Arc<RwLock<HashMap<(String, String), CachedReasoningResult>>>,
}

/// 缓存的推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedReasoningResult {
    result: CausalReasoningResult,
    cached_at: DateTime<Utc>,
}

impl CausalReasoningEngine {
    /// 创建新的因果推理引擎
    pub fn new(config: CausalReasoningConfig) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
            cause_to_effect: Arc::new(RwLock::new(HashMap::new())),
            effect_to_cause: Arc::new(RwLock::new(HashMap::new())),
            config,
            reasoning_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(CausalReasoningConfig::default())
    }

    /// 添加因果节点
    pub async fn add_node(&self, node: CausalNode) -> Result<()> {
        let node_id = node.id.clone();
        let mut nodes = self.nodes.write().await;
        nodes.insert(node_id.clone(), node);
        info!("添加因果节点: {}", node_id);
        Ok(())
    }

    /// 添加因果关系
    pub async fn add_causal_relation(&self, edge: CausalEdge) -> Result<()> {
        // 验证节点存在
        {
            let nodes = self.nodes.read().await;
            if !nodes.contains_key(&edge.cause_id) {
                return Err(AgentMemError::not_found(format!(
                    "原因节点不存在: {}",
                    edge.cause_id
                )));
            }
            if !nodes.contains_key(&edge.effect_id) {
                return Err(AgentMemError::not_found(format!(
                    "结果节点不存在: {}",
                    edge.effect_id
                )));
            }
        }

        // 添加边
        {
            let mut edges = self.edges.write().await;
            edges.insert(edge.id.clone(), edge.clone());
        }

        // 更新邻接表
        {
            let mut cause_to_effect = self.cause_to_effect.write().await;
            cause_to_effect
                .entry(edge.cause_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.id.clone());
        }

        {
            let mut effect_to_cause = self.effect_to_cause.write().await;
            effect_to_cause
                .entry(edge.effect_id.clone())
                .or_insert_with(Vec::new)
                .push(edge.id.clone());
        }

        info!(
            "添加因果关系: {} -> {} (强度: {:.2})",
            edge.cause_id, edge.effect_id, edge.strength
        );
        Ok(())
    }

    /// 因果链检索
    ///
    /// 从原因节点到结果节点查找所有可能的因果链
    pub async fn find_causal_chains(
        &self,
        cause_id: &str,
        effect_id: &str,
    ) -> Result<CausalReasoningResult> {
        // 检查缓存
        if self.config.enable_cache {
            let cache_key = (cause_id.to_string(), effect_id.to_string());
            {
                let cache = self.reasoning_cache.read().await;
                if let Some(cached) = cache.get(&cache_key) {
                    let age = Utc::now().signed_duration_since(cached.cached_at);
                    if age.num_seconds() < self.config.cache_ttl {
                        debug!("使用缓存的推理结果");
                        return Ok(cached.result.clone());
                    }
                }
            }
        }

        info!("查找因果链: {} -> {}", cause_id, effect_id);

        // 使用广度优先搜索查找所有路径
        let chains = self.bfs_find_chains(cause_id, effect_id).await?;

        // 构建推理结果
        let best_chain = chains
            .iter()
            .max_by(|a, b| {
                a.overall_confidence
                    .partial_cmp(&b.overall_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();

        let overall_confidence = best_chain
            .as_ref()
            .map(|c| c.overall_confidence)
            .unwrap_or(0.0);

        let explanation = self.generate_explanation(&chains, cause_id, effect_id);

        let result = CausalReasoningResult {
            chains: chains.clone(),
            best_chain,
            overall_confidence,
            explanation,
            reasoned_at: Utc::now(),
        };

        // 缓存结果
        if self.config.enable_cache {
            let cache_key = (cause_id.to_string(), effect_id.to_string());
            let mut cache = self.reasoning_cache.write().await;
            cache.insert(
                cache_key,
                CachedReasoningResult {
                    result: result.clone(),
                    cached_at: Utc::now(),
                },
            );
        }

        Ok(result)
    }

    /// 广度优先搜索查找因果链
    async fn bfs_find_chains(
        &self,
        start: &str,
        target: &str,
    ) -> Result<Vec<CausalChain>> {
        let mut chains = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        // 初始状态：从起始节点开始
        queue.push_back((start.to_string(), vec![start.to_string()], vec![], 1.0));

        while let Some((current, path, edges, confidence)) = queue.pop_front() {
            if current == target {
                // 找到目标，构建链
                let chain = CausalChain {
                    id: Uuid::new_v4().to_string(),
                    nodes: path.clone(),
                    edges: edges.clone(),
                    overall_confidence: confidence,
                    length: path.len() - 1,
                    explanation: self.generate_chain_explanation(&path, &edges).await?,
                    created_at: Utc::now(),
                };

                if chain.overall_confidence >= self.config.min_confidence
                    && chain.length <= self.config.max_chain_length
                {
                    chains.push(chain);
                }
                continue;
            }

            // 检查路径长度
            if path.len() > self.config.max_chain_length {
                continue;
            }

            // 获取当前节点的所有因果关系
            let cause_to_effect = self.cause_to_effect.read().await;
            if let Some(edge_ids) = cause_to_effect.get(&current) {
                let edges_map = self.edges.read().await;
                for edge_id in edge_ids {
                    if let Some(edge) = edges_map.get(edge_id) {
                        let next = edge.effect_id.clone();
                        let visit_key = format!("{current}:{next}");

                        if !visited.contains(&visit_key) {
                            visited.insert(visit_key);
                            let mut new_path = path.clone();
                            new_path.push(next.clone());
                            let mut new_edges = edges.clone();
                            new_edges.push(edge_id.clone());
                            let new_confidence = confidence * edge.strength * edge.confidence;
                            queue.push_back((next, new_path, new_edges, new_confidence));
                        }
                    }
                }
            }
        }

        Ok(chains)
    }

    /// 生成因果解释
    fn generate_explanation(
        &self,
        chains: &[CausalChain],
        cause_id: &str,
        effect_id: &str,
    ) -> String {
        if chains.is_empty() {
            return format!("未找到从 {cause_id} 到 {effect_id} 的因果链");
        }

        if let Some(best) = chains.first() {
            format!(
                "找到 {} 条因果链，最佳链长度: {}，置信度: {:.2}。{}",
                chains.len(),
                best.length,
                best.overall_confidence,
                best.explanation
            )
        } else {
            format!("找到 {} 条因果链", chains.len())
        }
    }

    /// 生成链解释
    async fn generate_chain_explanation(
        &self,
        nodes: &[String],
        edges: &[String],
    ) -> Result<String> {
        let nodes_map = self.nodes.read().await;
        let edges_map = self.edges.read().await;

        let mut parts = Vec::new();
        for (i, edge_id) in edges.iter().enumerate() {
            if let Some(edge) = edges_map.get(edge_id) {
                if let (Some(cause), Some(effect)) =
                    (nodes_map.get(&edge.cause_id), nodes_map.get(&edge.effect_id))
                {
                    parts.push(format!(
                        "{} ({} -> {}, 强度: {:.2})",
                        i + 1,
                        &cause.content[..cause.content.len().min(30)],
                        &effect.content[..effect.content.len().min(30)],
                        edge.strength
                    ));
                }
            }
        }

        Ok(format!("因果链: {}", parts.join(" -> ")))
    }

    /// 获取节点的所有原因
    pub async fn get_causes(&self, node_id: &str) -> Result<Vec<CausalNode>> {
        let effect_to_cause = self.effect_to_cause.read().await;
        let edges_map = self.edges.read().await;
        let nodes_map = self.nodes.read().await;

        let mut causes = Vec::new();
        if let Some(edge_ids) = effect_to_cause.get(node_id) {
            for edge_id in edge_ids {
                if let Some(edge) = edges_map.get(edge_id) {
                    if let Some(node) = nodes_map.get(&edge.cause_id) {
                        causes.push(node.clone());
                    }
                }
            }
        }

        Ok(causes)
    }

    /// 获取节点的所有结果
    pub async fn get_effects(&self, node_id: &str) -> Result<Vec<CausalNode>> {
        let cause_to_effect = self.cause_to_effect.read().await;
        let edges_map = self.edges.read().await;
        let nodes_map = self.nodes.read().await;

        let mut effects = Vec::new();
        if let Some(edge_ids) = cause_to_effect.get(node_id) {
            for edge_id in edge_ids {
                if let Some(edge) = edges_map.get(edge_id) {
                    if let Some(node) = nodes_map.get(&edge.effect_id) {
                        effects.push(node.clone());
                    }
                }
            }
        }

        Ok(effects)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CausalReasoningStats {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;

        CausalReasoningStats {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            avg_confidence: edges
                .values()
                .map(|e| e.confidence)
                .sum::<f32>()
                / edges.len().max(1) as f32,
            avg_strength: edges
                .values()
                .map(|e| e.strength)
                .sum::<f32>()
                / edges.len().max(1) as f32,
        }
    }
}

/// 因果推理统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalReasoningStats {
    /// 总节点数
    pub total_nodes: usize,
    /// 总边数
    pub total_edges: usize,
    /// 平均置信度
    pub avg_confidence: f32,
    /// 平均强度
    pub avg_strength: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_causal_reasoning() -> anyhow::Result<()> {
        let engine = CausalReasoningEngine::with_defaults();

        // 添加节点
        let node1 = CausalNode {
            id: "node1".to_string(),
            content: "用户点击按钮".to_string(),
            node_type: CausalNodeType::Action,
            timestamp: Utc::now(),
            properties: HashMap::new(),
        }
        };

        let node2 = CausalNode {
            id: "node2".to_string(),
            content: "系统发送请求".to_string(),
            node_type: CausalNodeType::Event,
            timestamp: Utc::now(),
            properties: HashMap::new(),
        };

        let node3 = CausalNode {
            id: "node3".to_string(),
            content: "服务器处理请求".to_string(),
            node_type: CausalNodeType::Event,
            timestamp: Utc::now(),
            properties: HashMap::new(),
        };

        engine.add_node(node1).await?;
        engine.add_node(node2).await?;
        engine.add_node(node3).await?;

        // 添加因果关系
        let edge1 = CausalEdge {
            id: "edge1".to_string(),
            cause_id: "node1".to_string(),
            effect_id: "node2".to_string(),
            strength: 0.9,
            time_delay_seconds: 0,
            confidence: 0.95,
            evidence: vec!["log1".to_string()],
            relation_type: CausalRelationType::Direct,
            created_at: Utc::now(),
        };

        let edge2 = CausalEdge {
            id: "edge2".to_string(),
            cause_id: "node2".to_string(),
            effect_id: "node3".to_string(),
            strength: 0.8,
            time_delay_seconds: 100,
            confidence: 0.9,
            evidence: vec!["log2".to_string()],
            relation_type: CausalRelationType::Direct,
            created_at: Utc::now(),
        };

        engine.add_causal_relation(edge1).await?;
        engine.add_causal_relation(edge2).await?;

        // 查找因果链
        let result = engine.find_causal_chains("node1", "node3").await?;
        assert!(!result.chains.is_empty());
        assert!(result.overall_confidence > 0.0);
    }
}
