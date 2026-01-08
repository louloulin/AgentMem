//! Semantic Hierarchy Index (SHIMI-style)
//!
//! Phase 4.3: 语义层次索引
//! - 语义层次结构构建
//! - 基于意义的检索
//! - 层次遍历优化
//!
//! 参考SHIMI架构，实现基于语义层次的索引和检索

use agent_mem_traits::{Result, AgentMemError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 语义层次节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNode {
    /// 节点ID
    pub id: String,
    /// 节点内容
    pub content: String,
    /// 语义含义（抽象表示）
    pub meaning: SemanticMeaning,
    /// 层次级别（0为根节点，数字越大越具体）
    pub level: usize,
    /// 父节点ID
    pub parent_id: Option<String>,
    /// 子节点ID列表
    pub child_ids: Vec<String>,
    /// 语义标签
    pub semantic_tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 属性
    pub properties: HashMap<String, serde_json::Value>,
}

/// 语义含义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMeaning {
    /// 核心概念
    pub core_concept: String,
    /// 语义类别
    pub category: SemanticCategory,
    /// 抽象程度 (0.0-1.0, 0.0最抽象, 1.0最具体)
    pub abstraction_level: f32,
    /// 相关概念
    pub related_concepts: Vec<String>,
    /// 语义向量（可选）
    pub semantic_vector: Option<Vec<f32>>,
}

/// 语义类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SemanticCategory {
    /// 实体
    Entity,
    /// 概念
    Concept,
    /// 关系
    Relation,
    /// 事件
    Event,
    /// 属性
    Attribute,
    /// 动作
    Action,
}

/// 语义层次索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHierarchyConfig {
    /// 最大层次深度
    pub max_depth: usize,
    /// 最小抽象度阈值
    pub min_abstraction: f32,
    /// 启用语义向量
    pub enable_semantic_vectors: bool,
    /// 启用缓存
    pub enable_cache: bool,
    /// 缓存TTL（秒）
    pub cache_ttl: i64,
}

impl Default for SemanticHierarchyConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            min_abstraction: 0.0,
            enable_semantic_vectors: true,
            enable_cache: true,
            cache_ttl: 3600,
        }
    }
}

/// 语义层次索引
pub struct SemanticHierarchyIndex {
    /// 语义节点
    nodes: Arc<RwLock<HashMap<String, SemanticNode>>>,
    /// 层次索引（level -> node_ids）
    level_index: Arc<RwLock<HashMap<usize, Vec<String>>>>,
    /// 类别索引（category -> node_ids）
    category_index: Arc<RwLock<HashMap<SemanticCategory, Vec<String>>>>,
    /// 概念索引（concept -> node_ids）
    concept_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 配置
    config: SemanticHierarchyConfig,
    /// 检索缓存
    search_cache: Arc<RwLock<HashMap<String, CachedSearchResult>>>,
}

/// 缓存的搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedSearchResult {
    results: Vec<SemanticNode>,
    cached_at: DateTime<Utc>,
}

/// 语义检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    /// 匹配的节点
    pub node: SemanticNode,
    /// 相关性分数
    pub relevance_score: f32,
    /// 语义相似度
    pub semantic_similarity: f32,
    /// 层次路径（从根到当前节点）
    pub hierarchy_path: Vec<String>,
}

impl SemanticHierarchyIndex {
    /// 创建新的语义层次索引
    pub fn new(config: SemanticHierarchyConfig) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            level_index: Arc::new(RwLock::new(HashMap::new())),
            category_index: Arc::new(RwLock::new(HashMap::new())),
            concept_index: Arc::new(RwLock::new(HashMap::new())),
            config,
            search_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(SemanticHierarchyConfig::default())
    }

    /// 添加语义节点
    pub async fn add_node(&self, node: SemanticNode) -> Result<()> {
        let node_id = node.id.clone();
        
        // 验证层次深度
        if node.level > self.config.max_depth {
            return Err(AgentMemError::validation_error(format!(
                "节点层次 {} 超过最大深度 {}",
                node.level, self.config.max_depth
            )));
        }

        // 验证父节点存在
        if let Some(ref parent_id) = node.parent_id {
            let nodes = self.nodes.read().await;
            if !nodes.contains_key(parent_id) {
                return Err(AgentMemError::not_found(format!(
                    "父节点不存在: {parent_id}"
                )));
            }
        }

        // 添加到节点集合
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node_id.clone(), node.clone());
        }

        // 更新层次索引
        {
            let mut level_index = self.level_index.write().await;
            level_index
                .entry(node.level)
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // 更新类别索引
        {
            let mut category_index = self.category_index.write().await;
            category_index
                .entry(node.meaning.category.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // 更新概念索引
        {
            let mut concept_index = self.concept_index.write().await;
            concept_index
                .entry(node.meaning.core_concept.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // 更新父节点的子节点列表
        if let Some(ref parent_id) = node.parent_id {
            let mut nodes = self.nodes.write().await;
            if let Some(parent) = nodes.get_mut(parent_id) {
                parent.child_ids.push(node_id.clone());
            }
        }

        info!("添加语义节点: {} (层次: {}, 类别: {:?})", node_id, node.level, node.meaning.category);
        Ok(())
    }

    /// 基于意义的检索
    ///
    /// 根据语义含义检索相关节点
    pub async fn search_by_meaning(
        &self,
        query_meaning: &SemanticMeaning,
        max_results: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        // 检查缓存
        if self.config.enable_cache {
            let cache_key = format!("meaning:{:?}:{}", query_meaning.category, query_meaning.core_concept);
            {
                let cache = self.search_cache.read().await;
                if let Some(cached) = cache.get(&cache_key) {
                    let age = Utc::now().signed_duration_since(cached.cached_at);
                    if age.num_seconds() < self.config.cache_ttl {
                        debug!("使用缓存的语义检索结果");
                        return Ok(self.convert_to_search_results(&cached.results, query_meaning).await);
                    }
                }
            }
        }

        info!("基于意义检索: {:?} - {}", query_meaning.category, query_meaning.core_concept);

        let nodes = self.nodes.read().await;
        let mut candidates = Vec::new();

        // 1. 按类别匹配
        if let Some(category_nodes) = self.category_index.read().await.get(&query_meaning.category) {
            for node_id in category_nodes {
                if let Some(node) = nodes.get(node_id) {
                    candidates.push(node.clone());
                }
            }
        }

        // 2. 按概念匹配
        if let Some(concept_nodes) = self.concept_index.read().await.get(&query_meaning.core_concept) {
            for node_id in concept_nodes {
                if let Some(node) = nodes.get(node_id) {
                    if !candidates.iter().any(|n| n.id == node.id) {
                        candidates.push(node.clone());
                    }
                }
            }
        }

        // 3. 计算相关性分数
        let mut results: Vec<SemanticSearchResult> = Vec::new();
        for node in &candidates {
            let relevance = self.calculate_relevance(node, query_meaning);
            let similarity = self.calculate_semantic_similarity(node, query_meaning);
            let path = self.get_hierarchy_path_sync(&nodes, &node.id);

            results.push(SemanticSearchResult {
                node: node.clone(),
                relevance_score: relevance,
                semantic_similarity: similarity,
                hierarchy_path: path,
            });
        }

        // 4. 按相关性排序
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 5. 限制结果数量
        results.truncate(max_results);

        // 6. 缓存结果
        if self.config.enable_cache {
            let cache_key = format!("meaning:{:?}:{}", query_meaning.category, query_meaning.core_concept);
            let mut cache = self.search_cache.write().await;
            cache.insert(
                cache_key,
                CachedSearchResult {
                    results: results.iter().map(|r| r.node.clone()).collect(),
                    cached_at: Utc::now(),
                },
            );
        }

        Ok(results)
    }

    /// 层次遍历
    ///
    /// 从指定节点开始，按层次遍历子节点
    pub async fn traverse_hierarchy(
        &self,
        start_node_id: &str,
        max_depth: Option<usize>,
    ) -> Result<Vec<SemanticNode>> {
        let nodes = self.nodes.read().await;
        let start_node = nodes
            .get(start_node_id)
            .ok_or_else(|| AgentMemError::not_found(format!("节点不存在: {start_node_id}")))?;

        let max_depth = max_depth.unwrap_or(self.config.max_depth);
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((start_node_id.to_string(), 0));
        visited.insert(start_node_id.to_string());

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }

            if let Some(node) = nodes.get(&current_id) {
                if depth > 0 {
                    result.push(node.clone());
                }

                // 添加子节点到队列
                for child_id in &node.child_ids {
                    if !visited.contains(child_id) {
                        visited.insert(child_id.clone());
                        queue.push_back((child_id.clone(), depth + 1));
                    }
                }
            }
        }

        Ok(result)
    }

    /// 计算相关性分数
    fn calculate_relevance(&self, node: &SemanticNode, query: &SemanticMeaning) -> f32 {
        let mut score = 0.0;

        // 类别匹配
        if node.meaning.category == query.category {
            score += 0.4;
        }

        // 概念匹配
        if node.meaning.core_concept == query.core_concept {
            score += 0.4;
        }

        // 相关概念匹配
        let common_concepts: usize = node
            .meaning
            .related_concepts
            .iter()
            .filter(|c| query.related_concepts.contains(c))
            .count();
        score += (common_concepts as f32 / query.related_concepts.len().max(1) as f32) * 0.2;

        score.min(1.0)
    }

    /// 计算语义相似度
    fn calculate_semantic_similarity(&self, node: &SemanticNode, query: &SemanticMeaning) -> f32 {
        // 如果有语义向量，使用向量相似度
        if let (Some(node_vec), Some(query_vec)) = (&node.meaning.semantic_vector, &query.semantic_vector) {
            if node_vec.len() == query_vec.len() {
                return self.cosine_similarity(node_vec, query_vec);
            }
        }

        // 否则使用基于抽象度的相似度
        let abstraction_diff = (node.meaning.abstraction_level - query.abstraction_level).abs();
        1.0 - abstraction_diff
    }

    /// 余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// 获取层次路径（同步版本）
    fn get_hierarchy_path_sync(
        &self,
        nodes: &HashMap<String, SemanticNode>,
        node_id: &str,
    ) -> Vec<String> {
        let mut path = Vec::new();
        let mut current_id = Some(node_id.to_string());

        while let Some(id) = current_id {
            path.push(id.clone());
            if let Some(node) = nodes.get(&id) {
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    /// 获取层次路径（异步版本，用于兼容）
    async fn get_hierarchy_path(
        &self,
        nodes: &HashMap<String, SemanticNode>,
        node_id: &str,
    ) -> Vec<String> {
        self.get_hierarchy_path_sync(nodes, node_id)
    }

    /// 转换为搜索结果
    async fn convert_to_search_results(
        &self,
        nodes: &[SemanticNode],
        query: &SemanticMeaning,
    ) -> Vec<SemanticSearchResult> {
        let nodes_map: HashMap<String, SemanticNode> = nodes.iter().map(|n| (n.id.clone(), n.clone())).collect();
        
        let mut results = Vec::new();
        for node in nodes {
            let relevance = self.calculate_relevance(node, query);
            let similarity = self.calculate_semantic_similarity(node, query);
            let path = self.get_hierarchy_path_sync(&nodes_map, &node.id);

            results.push(SemanticSearchResult {
                node: node.clone(),
                relevance_score: relevance,
                semantic_similarity: similarity,
                hierarchy_path: path,
            });
        }
        results
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> SemanticHierarchyStats {
        let nodes = self.nodes.read().await;
        let level_index = self.level_index.read().await;

        SemanticHierarchyStats {
            total_nodes: nodes.len(),
            nodes_by_level: level_index
                .iter()
                .map(|(level, ids)| (*level, ids.len()))
                .collect(),
            max_level: level_index.keys().max().copied().unwrap_or(0),
        }
    }
}

/// 语义层次统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHierarchyStats {
    /// 总节点数
    pub total_nodes: usize,
    /// 各层次的节点数
    pub nodes_by_level: HashMap<usize, usize>,
    /// 最大层次
    pub max_level: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_hierarchy() -> anyhow::Result<()> {
        let index = SemanticHierarchyIndex::with_defaults();

        // 添加根节点
        let root = SemanticNode {
            id: "root".to_string(),
            content: "根概念".to_string(),
            meaning: SemanticMeaning {
                core_concept: "概念".to_string(),
                category: SemanticCategory::Concept,
                abstraction_level: 0.0,
                related_concepts: vec![],
                semantic_vector: None,
            },
            level: 0,
            parent_id: None,
            child_ids: Vec::new(),
            semantic_tags: vec![],
            created_at: Utc::now(),
            properties: HashMap::new(),
        Ok(())
        };

        index.add_node(root).await?;

        // 添加子节点
        let child = SemanticNode {
            id: "child1".to_string(),
            content: "子概念1".to_string(),
            meaning: SemanticMeaning {
                core_concept: "具体概念1".to_string(),
                category: SemanticCategory::Concept,
                abstraction_level: 0.5,
                related_concepts: vec!["概念".to_string()],
                semantic_vector: None,
            },
            level: 1,
            parent_id: Some("root".to_string()),
            child_ids: Vec::new(),
            semantic_tags: vec![],
            created_at: Utc::now(),
            properties: HashMap::new(),
        };

        index.add_node(child).await?;

        // 测试层次遍历
        let traversed = index.traverse_hierarchy("root", Some(2)).await?;
        assert_eq!(traversed.len(), 1);
        assert_eq!(traversed[0].id, "child1");

        // 测试基于意义的检索
        let query = SemanticMeaning {
            core_concept: "具体概念1".to_string(),
            category: SemanticCategory::Concept,
            abstraction_level: 0.5,
            related_concepts: vec![],
            semantic_vector: None,
        };

        let results = index.search_by_meaning(&query, 10).await?;
        assert!(!results.is_empty());
    }
}

