//! 智能记忆压缩引擎
//!
//! 实现基于学术研究的智能压缩算法，包括重要性驱动压缩、语义保持压缩、
//! 时间感知压缩和自适应压缩策略。

use agent_mem_traits::{AgentMemError, MemoryItem, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 压缩引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// 启用重要性驱动压缩
    pub enable_importance_compression: bool,
    /// 启用语义保持压缩
    pub enable_semantic_compression: bool,
    /// 启用时间感知压缩
    pub enable_temporal_compression: bool,
    /// 启用自适应压缩
    pub enable_adaptive_compression: bool,
    /// 最小重要性阈值
    pub min_importance_threshold: f32,
    /// 压缩率目标 (0.0-1.0)
    pub target_compression_ratio: f32,
    /// 语义相似度阈值
    pub semantic_similarity_threshold: f32,
    /// 时间衰减因子
    pub temporal_decay_factor: f32,
    /// 自适应学习率
    pub adaptive_learning_rate: f32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enable_importance_compression: true,
            enable_semantic_compression: true,
            enable_temporal_compression: true,
            enable_adaptive_compression: true,
            min_importance_threshold: 0.3,
            target_compression_ratio: 0.7,
            semantic_similarity_threshold: 0.85,
            temporal_decay_factor: 0.95,
            adaptive_learning_rate: 0.1,
        }
    }
}

/// 重要性评估器
#[derive(Debug)]
pub struct ImportanceEvaluator {
    /// 访问频率权重
    access_frequency_weight: f32,
    /// 最近访问权重
    recency_weight: f32,
    /// 内容质量权重
    content_quality_weight: f32,
    /// 关联度权重
    relationship_weight: f32,
}

impl Default for ImportanceEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportanceEvaluator {
    /// 创建新的重要性评估器
    pub fn new() -> Self {
        Self {
            access_frequency_weight: 0.3,
            recency_weight: 0.25,
            content_quality_weight: 0.25,
            relationship_weight: 0.2,
        }
    }

    /// 评估记忆的重要性分数
    pub async fn evaluate_importance(
        &self,
        memory: &MemoryItem,
        context: &CompressionContext,
    ) -> Result<f32> {
        let mut score = 0.0;

        // 访问频率评分
        let access_score = self
            .calculate_access_frequency_score(memory, context)
            .await?;
        score += access_score * self.access_frequency_weight;

        // 最近访问评分
        let recency_score = self.calculate_recency_score(memory).await?;
        score += recency_score * self.recency_weight;

        // 内容质量评分
        let quality_score = self.calculate_content_quality_score(memory).await?;
        score += quality_score * self.content_quality_weight;

        // 关联度评分
        let relationship_score = self.calculate_relationship_score(memory, context).await?;
        score += relationship_score * self.relationship_weight;

        Ok(score.clamp(0.0, 1.0))
    }

    async fn calculate_access_frequency_score(
        &self,
        memory: &MemoryItem,
        context: &CompressionContext,
    ) -> Result<f32> {
        let access_count = context.access_stats.get(&memory.id).unwrap_or(&0);
        let max_access = context.max_access_count.max(1);
        Ok(*access_count as f32 / max_access as f32)
    }

    async fn calculate_recency_score(&self, memory: &MemoryItem) -> Result<f32> {
        let now = chrono::Utc::now().timestamp();
        let memory_time = memory.created_at.timestamp();
        let time_diff = (now - memory_time) as f32;
        let max_time_diff = 30.0 * 24.0 * 3600.0; // 30 days in seconds

        Ok(1.0 - (time_diff / max_time_diff).min(1.0))
    }

    async fn calculate_content_quality_score(&self, memory: &MemoryItem) -> Result<f32> {
        // 基于内容长度、结构化程度等评估质量
        let content_length = memory.content.len() as f32;
        let length_score = (content_length / 1000.0).min(1.0); // 标准化到1000字符

        // 检查是否包含结构化信息
        let structure_score = if memory.content.contains('\n') || memory.content.contains(':') {
            0.2
        } else {
            0.0
        };

        Ok((length_score * 0.8 + structure_score).min(1.0))
    }

    async fn calculate_relationship_score(
        &self,
        memory: &MemoryItem,
        context: &CompressionContext,
    ) -> Result<f32> {
        // 基于与其他记忆的关联度评估
        let relationship_count = context.relationship_counts.get(&memory.id).unwrap_or(&0);
        let max_relationships = context.max_relationship_count.max(1);
        Ok(*relationship_count as f32 / max_relationships as f32)
    }
}

/// 语义分析器
#[derive(Debug)]
pub struct SemanticAnalyzer {
    /// 嵌入维度
    embedding_dim: usize,
    /// PCA 组件数
    pca_components: usize,
}

impl SemanticAnalyzer {
    /// 创建新的语义分析器
    pub fn new(embedding_dim: usize) -> Self {
        Self {
            embedding_dim,
            pca_components: (embedding_dim / 4).max(64), // 默认压缩到1/4维度
        }
    }

    /// 分析记忆的语义相似度
    pub async fn analyze_semantic_similarity(
        &self,
        memory1: &MemoryItem,
        memory2: &MemoryItem,
    ) -> Result<f32> {
        // 计算嵌入向量的余弦相似度
        if let (Some(emb1), Some(emb2)) = (&memory1.embedding, &memory2.embedding) {
            let similarity = self.cosine_similarity(emb1, emb2)?;
            Ok(similarity)
        } else {
            // 如果没有嵌入向量，使用文本相似度
            Ok(self.text_similarity(&memory1.content, &memory2.content))
        }
    }

    /// 使用 PCA 进行语义压缩
    pub async fn compress_semantics(
        &self,
        memories: &[MemoryItem],
    ) -> Result<Vec<CompressedMemory>> {
        let mut compressed_memories = Vec::new();

        // 提取嵌入向量
        let embeddings: Vec<Vec<f32>> = memories
            .iter()
            .filter_map(|m| m.embedding.clone())
            .collect();

        if embeddings.is_empty() {
            // 如果没有嵌入向量，返回原始记忆
            for memory in memories {
                compressed_memories.push(CompressedMemory {
                    original_id: memory.id.clone(),
                    compressed_content: memory.content.clone(),
                    compression_ratio: 1.0,
                    semantic_hash: self.calculate_semantic_hash(&memory.content),
                    importance_score: 0.5, // 默认重要性
                });
            }
            return Ok(compressed_memories);
        }

        // 应用 PCA 降维
        let compressed_embeddings = self.apply_pca(&embeddings).await?;

        // 创建压缩记忆
        for (i, memory) in memories.iter().enumerate() {
            if i < compressed_embeddings.len() {
                let compressed_content = self
                    .compress_content_by_semantics(&memory.content, &compressed_embeddings[i])
                    .await?;
                // Calculate compression ratio using the newly created compressed_content
                let compression_ratio = self.calculate_compression_ratio(
                    &memory.content,
                    &compressed_content,
                );
                
                compressed_memories.push(CompressedMemory {
                    original_id: memory.id.clone(),
                    compressed_content,
                    compression_ratio,
                    semantic_hash: self.calculate_semantic_hash(&memory.content),
                    importance_score: 0.5, // 将由重要性评估器更新
                });
            }
        }

        Ok(compressed_memories)
    }

    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> Result<f32> {
        if vec1.len() != vec2.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions don't match",
            ));
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            Ok(0.0)
        } else {
            Ok(dot_product / (norm1 * norm2))
        }
    }

    fn text_similarity(&self, text1: &str, text2: &str) -> f32 {
        // 简单的文本相似度计算（基于共同词汇）
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    async fn apply_pca(&self, embeddings: &[Vec<f32>]) -> Result<Vec<Vec<f32>>> {
        // 简化的 PCA 实现（在实际应用中应使用专业的线性代数库）
        let n_samples = embeddings.len();
        let n_features = embeddings[0].len();
        let n_components = self.pca_components.min(n_features);

        // 计算均值
        let mut mean = vec![0.0; n_features];
        for embedding in embeddings {
            for (i, &val) in embedding.iter().enumerate() {
                mean[i] += val;
            }
        }
        for val in &mut mean {
            *val /= n_samples as f32;
        }

        // 中心化数据
        let mut centered_data = Vec::new();
        for embedding in embeddings {
            let mut centered = Vec::new();
            for (i, &val) in embedding.iter().enumerate() {
                centered.push(val - mean[i]);
            }
            centered_data.push(centered);
        }

        // 简化的降维：取前 n_components 个维度
        let mut compressed = Vec::new();
        for centered in centered_data {
            let mut comp = Vec::new();
            for i in 0..n_components {
                if i < centered.len() {
                    comp.push(centered[i]);
                } else {
                    comp.push(0.0);
                }
            }
            compressed.push(comp);
        }

        Ok(compressed)
    }

    async fn compress_content_by_semantics(
        &self,
        content: &str,
        _compressed_embedding: &[f32],
    ) -> Result<String> {
        // 基于语义的内容压缩（简化实现）
        let sentences: Vec<&str> = content.split('.').collect();
        if sentences.len() <= 2 {
            return Ok(content.to_string());
        }

        // 保留最重要的句子（前半部分和最后一句）
        let keep_count = (sentences.len() / 2).max(1);
        let mut compressed_sentences = Vec::new();

        for (i, sentence) in sentences.iter().enumerate() {
            if i < keep_count || i == sentences.len() - 1 {
                compressed_sentences.push(sentence.trim());
            }
        }

        Ok(compressed_sentences.join(". "))
    }

    fn calculate_compression_ratio(&self, original: &str, compressed: &str) -> f32 {
        if original.is_empty() {
            return 1.0;
        }
        compressed.len() as f32 / original.len() as f32
    }

    fn calculate_semantic_hash(&self, content: &str) -> String {
        // 简单的语义哈希（基于内容的哈希）
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// 压缩上下文
#[derive(Debug)]
pub struct CompressionContext {
    /// 访问统计
    pub access_stats: HashMap<String, usize>,
    /// 最大访问次数
    pub max_access_count: usize,
    /// 关联度统计
    pub relationship_counts: HashMap<String, usize>,
    /// 最大关联度
    pub max_relationship_count: usize,
    /// 查询模式
    pub query_patterns: Vec<String>,
}

impl Default for CompressionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionContext {
    /// 创建新的压缩上下文
    pub fn new() -> Self {
        Self {
            access_stats: HashMap::new(),
            max_access_count: 0,
            relationship_counts: HashMap::new(),
            max_relationship_count: 0,
            query_patterns: Vec::new(),
        }
    }

    /// 更新访问统计
    pub fn update_access_stats(&mut self, memory_id: String, access_count: usize) {
        self.access_stats.insert(memory_id, access_count);
        self.max_access_count = self.max_access_count.max(access_count);
    }

    /// 更新关联度统计
    pub fn update_relationship_stats(&mut self, memory_id: String, relationship_count: usize) {
        self.relationship_counts
            .insert(memory_id, relationship_count);
        self.max_relationship_count = self.max_relationship_count.max(relationship_count);
    }
}

/// 压缩后的记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedMemory {
    /// 原始记忆ID
    pub original_id: String,
    /// 压缩后的内容
    pub compressed_content: String,
    /// 压缩比率
    pub compression_ratio: f32,
    /// 语义哈希
    pub semantic_hash: String,
    /// 重要性分数
    pub importance_score: f32,
}

/// 压缩策略特征
#[async_trait::async_trait]
pub trait CompressionStrategy: Send + Sync {
    /// 策略名称
    fn name(&self) -> &str;

    /// 应用压缩策略
    async fn compress(
        &self,
        memories: &[MemoryItem],
        context: &CompressionContext,
    ) -> Result<Vec<CompressedMemory>>;

    /// 评估压缩效果
    async fn evaluate_compression(
        &self,
        original: &[MemoryItem],
        compressed: &[CompressedMemory],
    ) -> Result<CompressionMetrics>;
}

/// 重要性驱动压缩策略
#[derive(Debug)]
pub struct ImportanceDrivenCompression {
    importance_evaluator: Arc<ImportanceEvaluator>,
    min_importance_threshold: f32,
}

impl ImportanceDrivenCompression {
    /// 创建新的重要性驱动压缩策略
    pub fn new(min_importance_threshold: f32) -> Self {
        Self {
            importance_evaluator: Arc::new(ImportanceEvaluator::new()),
            min_importance_threshold,
        }
    }
}

#[async_trait::async_trait]
impl CompressionStrategy for ImportanceDrivenCompression {
    fn name(&self) -> &str {
        "importance_driven"
    }

    async fn compress(
        &self,
        memories: &[MemoryItem],
        context: &CompressionContext,
    ) -> Result<Vec<CompressedMemory>> {
        let mut compressed_memories = Vec::new();

        for memory in memories {
            let importance_score = self
                .importance_evaluator
                .evaluate_importance(memory, context)
                .await?;

            if importance_score >= self.min_importance_threshold {
                // 高重要性记忆：轻度压缩
                let compression_ratio =
                    0.8 + (importance_score - self.min_importance_threshold) * 0.2;
                let compressed_content = self
                    .light_compress(&memory.content, compression_ratio)
                    .await?;

                compressed_memories.push(CompressedMemory {
                    original_id: memory.id.clone(),
                    compressed_content,
                    compression_ratio,
                    semantic_hash: self.calculate_hash(&memory.content),
                    importance_score,
                });
            } else {
                // 低重要性记忆：重度压缩或丢弃
                if importance_score > 0.1 {
                    let compression_ratio = 0.3;
                    let compressed_content = self
                        .heavy_compress(&memory.content, compression_ratio)
                        .await?;

                    compressed_memories.push(CompressedMemory {
                        original_id: memory.id.clone(),
                        compressed_content,
                        compression_ratio,
                        semantic_hash: self.calculate_hash(&memory.content),
                        importance_score,
                    });
                }
                // 极低重要性记忆被丢弃
            }
        }

        Ok(compressed_memories)
    }

    async fn evaluate_compression(
        &self,
        original: &[MemoryItem],
        compressed: &[CompressedMemory],
    ) -> Result<CompressionMetrics> {
        let original_size: usize = original.iter().map(|m| m.content.len()).sum();
        let compressed_size: usize = compressed.iter().map(|m| m.compressed_content.len()).sum();

        let compression_ratio = if original_size > 0 {
            compressed_size as f32 / original_size as f32
        } else {
            1.0
        };

        let information_retention = compressed.len() as f32 / original.len() as f32;

        Ok(CompressionMetrics {
            compression_ratio,
            information_retention,
            memory_count_reduction: original.len() - compressed.len(),
            average_importance: compressed.iter().map(|m| m.importance_score).sum::<f32>()
                / compressed.len() as f32,
        })
    }
}

impl ImportanceDrivenCompression {
    async fn light_compress(&self, content: &str, _ratio: f32) -> Result<String> {
        // 轻度压缩：移除多余空白和重复内容
        let compressed = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(compressed)
    }

    async fn heavy_compress(&self, content: &str, _ratio: f32) -> Result<String> {
        // 重度压缩：提取关键信息
        let words: Vec<&str> = content.split_whitespace().collect();
        let keep_count = (words.len() / 3).max(5); // 保留1/3的词汇，最少5个

        let compressed = words
            .into_iter()
            .take(keep_count)
            .collect::<Vec<_>>()
            .join(" ");
        Ok(compressed + "...")
    }

    fn calculate_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// 时间感知压缩策略
#[derive(Debug)]
pub struct TemporalAwareCompression {
    decay_factor: f32,
    time_window_days: i64,
}

impl TemporalAwareCompression {
    /// 创建新的时间感知压缩策略
    pub fn new(decay_factor: f32, time_window_days: i64) -> Self {
        Self {
            decay_factor,
            time_window_days,
        }
    }

    fn calculate_temporal_weight(&self, memory: &MemoryItem) -> f32 {
        let now = chrono::Utc::now().timestamp();
        let memory_time = memory.created_at.timestamp();
        let days_old = (now - memory_time) / (24 * 3600);

        if days_old <= 0 {
            1.0
        } else if days_old >= self.time_window_days {
            0.1
        } else {
            self.decay_factor.powf(days_old as f32)
        }
    }
}

#[async_trait::async_trait]
impl CompressionStrategy for TemporalAwareCompression {
    fn name(&self) -> &str {
        "temporal_aware"
    }

    async fn compress(
        &self,
        memories: &[MemoryItem],
        _context: &CompressionContext,
    ) -> Result<Vec<CompressedMemory>> {
        let mut compressed_memories = Vec::new();

        for memory in memories {
            let temporal_weight = self.calculate_temporal_weight(memory);
            let compression_ratio = 0.3 + temporal_weight * 0.7; // 新记忆压缩率高，旧记忆压缩率低

            let compressed_content = if temporal_weight > 0.7 {
                // 新记忆：保持完整
                memory.content.clone()
            } else if temporal_weight > 0.3 {
                // 中等年龄记忆：中度压缩
                self.moderate_compress(&memory.content).await?
            } else {
                // 旧记忆：重度压缩
                self.heavy_compress(&memory.content).await?
            };

            compressed_memories.push(CompressedMemory {
                original_id: memory.id.clone(),
                compressed_content,
                compression_ratio,
                semantic_hash: self.calculate_hash(&memory.content),
                importance_score: temporal_weight,
            });
        }

        Ok(compressed_memories)
    }

    async fn evaluate_compression(
        &self,
        original: &[MemoryItem],
        compressed: &[CompressedMemory],
    ) -> Result<CompressionMetrics> {
        let original_size: usize = original.iter().map(|m| m.content.len()).sum();
        let compressed_size: usize = compressed.iter().map(|m| m.compressed_content.len()).sum();

        let compression_ratio = if original_size > 0 {
            compressed_size as f32 / original_size as f32
        } else {
            1.0
        };

        Ok(CompressionMetrics {
            compression_ratio,
            information_retention: 1.0, // 时间感知压缩保留所有记忆
            memory_count_reduction: 0,
            average_importance: compressed.iter().map(|m| m.importance_score).sum::<f32>()
                / compressed.len() as f32,
        })
    }
}

impl TemporalAwareCompression {
    async fn moderate_compress(&self, content: &str) -> Result<String> {
        // 中度压缩：保留主要句子
        let sentences: Vec<&str> = content.split('.').collect();
        let keep_count = (sentences.len() * 2 / 3).max(1);

        let compressed = sentences
            .into_iter()
            .take(keep_count)
            .collect::<Vec<_>>()
            .join(". ");
        Ok(compressed)
    }

    async fn heavy_compress(&self, content: &str) -> Result<String> {
        // 重度压缩：只保留关键词
        let words: Vec<&str> = content.split_whitespace().collect();
        let keep_count = (words.len() / 4).max(3);

        let compressed = words
            .into_iter()
            .take(keep_count)
            .collect::<Vec<_>>()
            .join(" ");
        Ok(compressed + "...")
    }

    fn calculate_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// 压缩指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetrics {
    /// 压缩比率
    pub compression_ratio: f32,
    /// 信息保留率
    pub information_retention: f32,
    /// 记忆数量减少
    pub memory_count_reduction: usize,
    /// 平均重要性
    pub average_importance: f32,
}

/// 自适应控制器
#[derive(Debug)]
pub struct AdaptiveController {
    /// 学习率
    learning_rate: f32,
    /// 性能历史
    performance_history: Arc<RwLock<Vec<CompressionMetrics>>>,
    /// 策略权重
    strategy_weights: Arc<RwLock<HashMap<String, f32>>>,
    /// 目标指标
    target_metrics: CompressionMetrics,
}

impl AdaptiveController {
    /// 创建新的自适应控制器
    pub fn new(learning_rate: f32, target_metrics: CompressionMetrics) -> Self {
        Self {
            learning_rate,
            performance_history: Arc::new(RwLock::new(Vec::new())),
            strategy_weights: Arc::new(RwLock::new(HashMap::new())),
            target_metrics,
        }
    }

    /// 更新策略权重
    pub async fn update_weights(
        &self,
        strategy_name: String,
        metrics: CompressionMetrics,
    ) -> Result<()> {
        let mut weights = self.strategy_weights.write().await;
        let mut history = self.performance_history.write().await;

        // 计算性能分数
        let performance_score = self.calculate_performance_score(&metrics);

        // 更新权重
        let current_weight = weights.get(&strategy_name).unwrap_or(&0.5);
        let new_weight = current_weight + self.learning_rate * (performance_score - current_weight);
        weights.insert(strategy_name, new_weight.clamp(0.0, 1.0));

        // 记录历史
        history.push(metrics);
        if history.len() > 100 {
            history.remove(0); // 保持历史记录在合理范围内
        }

        Ok(())
    }

    /// 获取策略权重
    pub async fn get_strategy_weight(&self, strategy_name: &str) -> f32 {
        let weights = self.strategy_weights.read().await;
        *weights.get(strategy_name).unwrap_or(&0.5)
    }

    /// 获取推荐的压缩策略
    pub async fn recommend_strategy(&self, context: &CompressionContext) -> Result<String> {
        let weights = self.strategy_weights.read().await;

        // 基于上下文和历史性能推荐策略
        if context.query_patterns.len() > 10 {
            // 高查询频率：推荐重要性驱动压缩
            Ok("importance_driven".to_string())
        } else if context.access_stats.len() > 1000 {
            // 大量记忆：推荐时间感知压缩
            Ok("temporal_aware".to_string())
        } else {
            // 默认：选择权重最高的策略
            let best_strategy = weights
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(name, _)| name.clone())
                .unwrap_or_else(|| "importance_driven".to_string());

            Ok(best_strategy)
        }
    }

    fn calculate_performance_score(&self, metrics: &CompressionMetrics) -> f32 {
        // 综合评分：平衡压缩率和信息保留率
        let compression_score =
            (self.target_metrics.compression_ratio - metrics.compression_ratio).abs();
        let retention_score =
            (self.target_metrics.information_retention - metrics.information_retention).abs();
        let importance_score =
            (self.target_metrics.average_importance - metrics.average_importance).abs();

        // 分数越低越好，转换为0-1的性能分数
        let total_error = compression_score + retention_score + importance_score;
        (1.0 - total_error.min(1.0)).max(0.0)
    }
}

/// 智能压缩引擎
pub struct IntelligentCompressionEngine {
    /// 重要性评估器
    importance_evaluator: Arc<ImportanceEvaluator>,
    /// 语义分析器
    semantic_analyzer: Arc<SemanticAnalyzer>,
    /// 压缩策略
    compression_strategies: HashMap<String, Box<dyn CompressionStrategy>>,
    /// 自适应控制器
    adaptive_controller: Arc<AdaptiveController>,
    /// 配置
    config: CompressionConfig,
}

impl IntelligentCompressionEngine {
    /// 创建新的智能压缩引擎
    pub fn new(config: CompressionConfig) -> Self {
        let target_metrics = CompressionMetrics {
            compression_ratio: config.target_compression_ratio,
            information_retention: 0.8,
            memory_count_reduction: 0,
            average_importance: config.min_importance_threshold,
        };

        let adaptive_controller = Arc::new(AdaptiveController::new(
            config.adaptive_learning_rate,
            target_metrics,
        ));

        let mut strategies: HashMap<String, Box<dyn CompressionStrategy>> = HashMap::new();

        if config.enable_importance_compression {
            strategies.insert(
                "importance_driven".to_string(),
                Box::new(ImportanceDrivenCompression::new(
                    config.min_importance_threshold,
                )),
            );
        }

        if config.enable_temporal_compression {
            strategies.insert(
                "temporal_aware".to_string(),
                Box::new(TemporalAwareCompression::new(
                    config.temporal_decay_factor,
                    30,
                )),
            );
        }

        Self {
            importance_evaluator: Arc::new(ImportanceEvaluator::new()),
            semantic_analyzer: Arc::new(SemanticAnalyzer::new(768)), // 默认768维嵌入
            compression_strategies: strategies,
            adaptive_controller,
            config,
        }
    }

    /// 压缩记忆集合
    pub async fn compress_memories(
        &self,
        memories: &[MemoryItem],
        context: &CompressionContext,
    ) -> Result<Vec<CompressedMemory>> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        // 获取推荐的压缩策略
        let strategy_name = self.adaptive_controller.recommend_strategy(context).await?;

        // 应用压缩策略
        let compressed_memories =
            if let Some(strategy) = self.compression_strategies.get(&strategy_name) {
                strategy.compress(memories, context).await?
            } else {
                // 回退到默认策略
                self.default_compress(memories, context).await?
            };

        // 如果启用语义压缩，进一步优化
        let final_compressed = if self.config.enable_semantic_compression {
            self.apply_semantic_compression(&compressed_memories)
                .await?
        } else {
            compressed_memories
        };

        // 评估压缩效果并更新自适应控制器
        if let Some(strategy) = self.compression_strategies.get(&strategy_name) {
            let metrics = strategy
                .evaluate_compression(memories, &final_compressed)
                .await?;
            self.adaptive_controller
                .update_weights(strategy_name, metrics)
                .await?;
        }

        Ok(final_compressed)
    }

    /// 解压缩记忆
    pub async fn decompress_memory(&self, compressed: &CompressedMemory) -> Result<MemoryItem> {
        use agent_mem_traits::Session;

        // 简化的解压缩实现
        Ok(MemoryItem {
            id: compressed.original_id.clone(),
            content: compressed.compressed_content.clone(),
            hash: None,
            metadata: HashMap::new(),
            score: Some(compressed.importance_score),
            created_at: chrono::Utc::now(),
            updated_at: Some(chrono::Utc::now()),
            session: Session::new(),
            memory_type: agent_mem_traits::MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "default".to_string(),
            user_id: None,
            importance: compressed.importance_score,
            embedding: None,
            last_accessed_at: chrono::Utc::now(),
            access_count: 0,
            expires_at: None,
            version: 1,
        })
    }

    /// 获取压缩统计信息
    pub async fn get_compression_stats(&self) -> Result<CompressionStats> {
        let history = self.adaptive_controller.performance_history.read().await;
        let weights = self.adaptive_controller.strategy_weights.read().await;

        let total_compressions = history.len();
        let average_compression_ratio = if total_compressions > 0 {
            history.iter().map(|m| m.compression_ratio).sum::<f32>() / total_compressions as f32
        } else {
            1.0
        };

        let average_retention = if total_compressions > 0 {
            history.iter().map(|m| m.information_retention).sum::<f32>() / total_compressions as f32
        } else {
            1.0
        };

        Ok(CompressionStats {
            total_compressions,
            average_compression_ratio,
            average_information_retention: average_retention,
            strategy_weights: weights.clone(),
            enabled_strategies: self.compression_strategies.keys().cloned().collect(),
        })
    }

    async fn default_compress(
        &self,
        memories: &[MemoryItem],
        context: &CompressionContext,
    ) -> Result<Vec<CompressedMemory>> {
        let mut compressed_memories = Vec::new();

        for memory in memories {
            let importance_score = self
                .importance_evaluator
                .evaluate_importance(memory, context)
                .await?;

            compressed_memories.push(CompressedMemory {
                original_id: memory.id.clone(),
                compressed_content: memory.content.clone(),
                compression_ratio: 1.0,
                semantic_hash: self.calculate_hash(&memory.content),
                importance_score,
            });
        }

        Ok(compressed_memories)
    }

    async fn apply_semantic_compression(
        &self,
        compressed_memories: &[CompressedMemory],
    ) -> Result<Vec<CompressedMemory>> {
        // 基于语义相似度进一步压缩
        let mut final_compressed = Vec::new();
        let mut processed = std::collections::HashSet::new();

        for (i, memory) in compressed_memories.iter().enumerate() {
            if processed.contains(&i) {
                continue;
            }

            let mut similar_memories = vec![memory.clone()];

            // 查找语义相似的记忆
            for (j, other_memory) in compressed_memories.iter().enumerate() {
                if i != j && !processed.contains(&j) {
                    let similarity = self.calculate_text_similarity(
                        &memory.compressed_content,
                        &other_memory.compressed_content,
                    );
                    if similarity > self.config.semantic_similarity_threshold {
                        similar_memories.push(other_memory.clone());
                        processed.insert(j);
                    }
                }
            }

            // 合并相似记忆
            if similar_memories.len() > 1 {
                let merged = self.merge_similar_memories(&similar_memories).await?;
                final_compressed.push(merged);
            } else {
                final_compressed.push(memory.clone());
            }

            processed.insert(i);
        }

        Ok(final_compressed)
    }

    async fn merge_similar_memories(
        &self,
        memories: &[CompressedMemory],
    ) -> Result<CompressedMemory> {
        // 合并相似记忆的内容
        let merged_content = memories
            .iter()
            .map(|m| m.compressed_content.as_str())
            .collect::<Vec<_>>()
            .join(" | ");

        let average_importance =
            memories.iter().map(|m| m.importance_score).sum::<f32>() / memories.len() as f32;
        let average_compression_ratio =
            memories.iter().map(|m| m.compression_ratio).sum::<f32>() / memories.len() as f32;

        Ok(CompressedMemory {
            original_id: format!("merged_{}", Uuid::new_v4()),
            compressed_content: merged_content,
            compression_ratio: average_compression_ratio,
            semantic_hash: self.calculate_hash(&memories[0].compressed_content),
            importance_score: average_importance,
        })
    }

    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f32 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    fn calculate_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// 压缩统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// 总压缩次数
    pub total_compressions: usize,
    /// 平均压缩比率
    pub average_compression_ratio: f32,
    /// 平均信息保留率
    pub average_information_retention: f32,
    /// 策略权重
    pub strategy_weights: HashMap<String, f32>,
    /// 启用的策略
    pub enabled_strategies: Vec<String>,
}
