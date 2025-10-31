//! 增强学习机制
//! 
//! 从实际使用反馈中持续学习和优化查询权重策略

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::search::{QueryFeatures, SearchWeights};

/// 反馈条目（持久化版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub id: String,
    pub query_pattern: String,  // 查询模式（归类后的）
    pub features: QueryFeatures,
    pub weights: SearchWeights,
    pub effectiveness: f32,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
}

/// 查询模式（用于分类相似查询）
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum QueryPattern {
    ExactMatch,           // 精确匹配查询
    SemanticQuestion,     // 语义问句
    TemporalQuery,        // 时间相关查询
    SimpleKeyword,        // 简单关键词
    ComplexSemantic,      // 复杂语义查询
    MixedQuery,           // 混合查询
}

impl QueryPattern {
    /// 从查询特征识别模式
    pub fn from_features(features: &QueryFeatures) -> Self {
        if features.has_exact_terms {
            QueryPattern::ExactMatch
        } else if features.has_temporal_indicator {
            QueryPattern::TemporalQuery
        } else if features.is_question && features.semantic_complexity > 0.6 {
            QueryPattern::SemanticQuestion
        } else if features.semantic_complexity < 0.3 {
            QueryPattern::SimpleKeyword
        } else if features.semantic_complexity > 0.7 {
            QueryPattern::ComplexSemantic
        } else {
            QueryPattern::MixedQuery
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            QueryPattern::ExactMatch => "exact_match",
            QueryPattern::SemanticQuestion => "semantic_question",
            QueryPattern::TemporalQuery => "temporal_query",
            QueryPattern::SimpleKeyword => "simple_keyword",
            QueryPattern::ComplexSemantic => "complex_semantic",
            QueryPattern::MixedQuery => "mixed_query",
        }
    }
}

/// 统计数据
#[derive(Debug, Clone, Default)]
pub struct PatternStatistics {
    pub total_queries: usize,
    pub avg_effectiveness: f32,
    pub best_weights: SearchWeights,
    pub weight_variance: f32,
    pub last_updated: Option<DateTime<Utc>>,
}

/// 学习配置
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// 学习率（控制权重更新速度）
    pub learning_rate: f32,
    
    /// 最小样本数（少于此数量不进行学习）
    pub min_samples: usize,
    
    /// 衰减因子（旧数据的权重衰减）
    pub decay_factor: f32,
    
    /// 是否启用自动优化
    pub enable_auto_optimization: bool,
    
    /// 优化间隔（秒）
    pub optimization_interval_secs: u64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            min_samples: 10,
            decay_factor: 0.95,
            enable_auto_optimization: true,
            optimization_interval_secs: 3600,  // 1小时
        }
    }
}

/// 增强学习引擎
pub struct LearningEngine {
    config: LearningConfig,
    
    /// 按查询模式分类的统计数据
    pattern_stats: Arc<RwLock<HashMap<QueryPattern, PatternStatistics>>>,
    
    /// 反馈历史（内存缓存，有大小限制）
    feedback_history: Arc<RwLock<Vec<FeedbackRecord>>>,
    
    /// 最大历史记录数
    max_history_size: usize,
    
    /// 持久化仓库（可选）
    #[cfg(feature = "libsql")]
    repository: Option<Arc<dyn crate::storage::libsql::LearningRepositoryTrait>>,
}

impl LearningEngine {
    /// 创建新的学习引擎
    pub fn new(config: LearningConfig) -> Self {
        Self {
            config,
            pattern_stats: Arc::new(RwLock::new(HashMap::new())),
            feedback_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,  // 内存中最多保留1000条
            #[cfg(feature = "libsql")]
            repository: None,
        }
    }
    
    /// 创建带持久化的学习引擎
    #[cfg(feature = "libsql")]
    pub fn with_persistence(
        config: LearningConfig,
        repository: Arc<dyn crate::storage::libsql::LearningRepositoryTrait>,
    ) -> Self {
        Self {
            config,
            pattern_stats: Arc::new(RwLock::new(HashMap::new())),
            feedback_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
            repository: Some(repository),
        }
    }
    
    /// 从存储加载历史数据
    #[cfg(feature = "libsql")]
    pub async fn load_from_storage(&self) -> agent_mem_traits::Result<()> {
        if let Some(repo) = &self.repository {
            // 加载最近的反馈记录
            let records = repo.get_recent_feedback(self.max_history_size).await?;
            
            // 加载到内存
            let mut history = self.feedback_history.write().await;
            *history = records.clone();
            drop(history);
            
            // 重建统计数据
            for record in records {
                let pattern = QueryPattern::from_features(&record.features);
                self.update_statistics(pattern, record).await;
            }
        }
        Ok(())
    }
    
    /// 记录反馈
    pub async fn record_feedback(
        &self,
        features: QueryFeatures,
        weights: SearchWeights,
        effectiveness: f32,
        user_id: Option<String>,
    ) {
        let pattern = QueryPattern::from_features(&features);
        let record = FeedbackRecord {
            id: uuid::Uuid::new_v4().to_string(),
            query_pattern: pattern.as_str().to_string(),
            features,
            weights,
            effectiveness,
            timestamp: Utc::now(),
            user_id,
        };
        
        // 保存到数据库（如果启用持久化）
        #[cfg(feature = "libsql")]
        if let Some(repo) = &self.repository {
            // 异步保存，忽略错误（不阻塞主流程）
            let _ = repo.create_feedback(&record).await;
        }
        
        // 添加到历史记录
        let mut history = self.feedback_history.write().await;
        history.push(record.clone());
        
        // 限制历史大小
        if history.len() > self.max_history_size {
            history.drain(0..100);  // 删除最旧的100条
        }
        drop(history);
        
        // 更新统计数据
        self.update_statistics(pattern, record).await;
    }
    
    /// 更新模式统计数据
    async fn update_statistics(&self, pattern: QueryPattern, record: FeedbackRecord) {
        let mut stats = self.pattern_stats.write().await;
        let pattern_stat = stats.entry(pattern).or_insert_with(PatternStatistics::default);
        
        // 增量更新统计
        let _n = pattern_stat.total_queries as f32;
        
        // 加权移动平均更新平均效果
        if pattern_stat.total_queries == 0 {
            pattern_stat.avg_effectiveness = record.effectiveness;
            pattern_stat.best_weights = record.weights;
        } else {
            // 指数移动平均
            let alpha = self.config.learning_rate;
            pattern_stat.avg_effectiveness = 
                alpha * record.effectiveness + (1.0 - alpha) * pattern_stat.avg_effectiveness;
            
            // 如果新权重效果更好，逐步调整最佳权重
            if record.effectiveness > pattern_stat.avg_effectiveness * 1.1 {
                // 向更好的权重移动
                pattern_stat.best_weights.vector_weight = 
                    alpha * record.weights.vector_weight + 
                    (1.0 - alpha) * pattern_stat.best_weights.vector_weight;
                pattern_stat.best_weights.fulltext_weight = 
                    alpha * record.weights.fulltext_weight + 
                    (1.0 - alpha) * pattern_stat.best_weights.fulltext_weight;
                pattern_stat.best_weights.normalize();
            }
        }
        
        pattern_stat.total_queries += 1;
        pattern_stat.last_updated = Some(Utc::now());
    }
    
    /// 获取推荐权重
    pub async fn get_recommended_weights(&self, features: &QueryFeatures) -> Option<SearchWeights> {
        let pattern = QueryPattern::from_features(features);
        let stats = self.pattern_stats.read().await;
        
        if let Some(pattern_stat) = stats.get(&pattern) {
            // 如果有足够的样本，返回学习到的最佳权重
            if pattern_stat.total_queries >= self.config.min_samples {
                return Some(pattern_stat.best_weights.clone());
            }
        }
        
        None
    }
    
    /// 获取所有模式的统计信息
    pub async fn get_all_statistics(&self) -> HashMap<QueryPattern, PatternStatistics> {
        self.pattern_stats.read().await.clone()
    }
    
    /// 获取反馈历史
    pub async fn get_feedback_history(&self) -> Vec<FeedbackRecord> {
        self.feedback_history.read().await.clone()
    }
    
    /// 清除历史数据
    pub async fn clear_history(&self) {
        self.feedback_history.write().await.clear();
        self.pattern_stats.write().await.clear();
    }
    
    /// 执行优化分析
    pub async fn optimize(&self) -> OptimizationReport {
        let stats = self.pattern_stats.read().await;
        let history = self.feedback_history.read().await;
        
        let mut report = OptimizationReport {
            total_samples: history.len(),
            patterns_analyzed: stats.len(),
            improvements: Vec::new(),
            timestamp: Utc::now(),
        };
        
        // 分析每个模式的优化情况
        for (pattern, stat) in stats.iter() {
            if stat.total_queries >= self.config.min_samples {
                report.improvements.push(PatternImprovement {
                    pattern: pattern.clone(),
                    samples: stat.total_queries,
                    avg_effectiveness: stat.avg_effectiveness,
                    recommended_weights: stat.best_weights.clone(),
                });
            }
        }
        
        report
    }
}

/// 优化报告
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub total_samples: usize,
    pub patterns_analyzed: usize,
    pub improvements: Vec<PatternImprovement>,
    pub timestamp: DateTime<Utc>,
}

/// 模式改进信息
#[derive(Debug, Clone)]
pub struct PatternImprovement {
    pub pattern: QueryPattern,
    pub samples: usize,
    pub avg_effectiveness: f32,
    pub recommended_weights: SearchWeights,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_learning_engine_basic() {
        let config = LearningConfig::default();
        let engine = LearningEngine::new(config);
        
        // 记录一些反馈
        let features = QueryFeatures {
            has_exact_terms: true,
            semantic_complexity: 0.3,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 20,
            is_question: false,
        };
        
        let weights = SearchWeights {
            vector_weight: 0.3,
            fulltext_weight: 0.7,
            confidence: 0.8,
        };
        
        for i in 0..15 {
            let effectiveness = 0.85 + (i as f32 * 0.01);
            engine.record_feedback(
                features.clone(),
                weights.clone(),
                effectiveness,
                None,
            ).await;
        }
        
        // 获取推荐权重
        let recommended = engine.get_recommended_weights(&features).await;
        assert!(recommended.is_some());
        
        let rec_weights = recommended.unwrap();
        assert!(rec_weights.fulltext_weight > rec_weights.vector_weight);
    }
    
    #[tokio::test]
    async fn test_pattern_classification() {
        let exact_match = QueryFeatures {
            has_exact_terms: true,
            semantic_complexity: 0.3,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 20,
            is_question: false,
        };
        assert_eq!(
            QueryPattern::from_features(&exact_match),
            QueryPattern::ExactMatch
        );
        
        let semantic = QueryFeatures {
            has_exact_terms: false,
            semantic_complexity: 0.8,
            has_temporal_indicator: false,
            entity_count: 0,
            query_length: 100,
            is_question: true,
        };
        assert_eq!(
            QueryPattern::from_features(&semantic),
            QueryPattern::SemanticQuestion
        );
    }
    
    #[tokio::test]
    async fn test_optimization_report() {
        let config = LearningConfig {
            min_samples: 5,
            ..Default::default()
        };
        let engine = LearningEngine::new(config);
        
        // 记录多种模式的反馈
        let patterns = vec![
            (QueryPattern::ExactMatch, 0.9),
            (QueryPattern::SemanticQuestion, 0.85),
            (QueryPattern::SimpleKeyword, 0.75),
        ];
        
        for (pattern_type, effectiveness) in patterns {
            let features = match pattern_type {
                QueryPattern::ExactMatch => QueryFeatures {
                    has_exact_terms: true,
                    semantic_complexity: 0.3,
                    has_temporal_indicator: false,
                    entity_count: 0,
                    query_length: 20,
                    is_question: false,
                },
                _ => QueryFeatures {
                    has_exact_terms: false,
                    semantic_complexity: 0.7,
                    has_temporal_indicator: false,
                    entity_count: 0,
                    query_length: 50,
                    is_question: true,
                },
            };
            
            for _ in 0..10 {
                engine.record_feedback(
                    features.clone(),
                    SearchWeights {
                        vector_weight: 0.5,
                        fulltext_weight: 0.5,
                        confidence: 0.8,
                    },
                    effectiveness,
                    None,
                ).await;
            }
        }
        
        let report = engine.optimize().await;
        assert!(report.total_samples >= 30);
        assert!(report.improvements.len() >= 2);
    }
}

