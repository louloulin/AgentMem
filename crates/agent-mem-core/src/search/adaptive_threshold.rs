//! Adaptive Threshold Calculator - 自适应阈值计算器
//!
//! 根据查询类型、长度、复杂度等因素动态计算最优相似度阈值

use super::query_classifier::{QueryFeatures, QueryType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 自适应阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveThresholdConfig {
    /// 各查询类型的基础阈值
    pub base_thresholds: HashMap<QueryType, f32>,
    
    /// 查询长度影响因子
    pub length_factor: f32,
    
    /// 查询复杂度影响因子
    pub complexity_factor: f32,
    
    /// 是否启用历史反馈
    pub enable_historical_feedback: bool,
    
    /// 最小阈值
    pub min_threshold: f32,
    
    /// 最大阈值
    pub max_threshold: f32,
}

impl Default for AdaptiveThresholdConfig {
    fn default() -> Self {
        let mut base_thresholds = HashMap::new();
        base_thresholds.insert(QueryType::ExactId, 0.0);
        base_thresholds.insert(QueryType::ShortKeyword, 0.1);
        base_thresholds.insert(QueryType::NaturalLanguage, 0.3);
        base_thresholds.insert(QueryType::Semantic, 0.5);
        base_thresholds.insert(QueryType::Temporal, 0.0);
        
        Self {
            base_thresholds,
            length_factor: 0.3,
            complexity_factor: 0.2,
            enable_historical_feedback: true,
            min_threshold: 0.0,
            max_threshold: 0.9,
        }
    }
}

/// 历史统计数据
#[derive(Debug, Clone, Default)]
pub struct HistoricalStats {
    /// 各查询类型的平均相似度
    avg_scores: HashMap<QueryType, f32>,
    
    /// 各查询类型的调整值
    adjustments: HashMap<QueryType, f32>,
    
    /// 查询次数
    query_counts: HashMap<QueryType, usize>,
}

impl HistoricalStats {
    /// 更新统计数据
    pub fn update(&mut self, query_type: QueryType, score: f32) {
        // 更新平均分数
        let count = self.query_counts.get(&query_type).copied().unwrap_or(0);
        let avg = self.avg_scores.get(&query_type).copied().unwrap_or(0.0);
        
        let new_avg = (avg * count as f32 + score) / (count + 1) as f32;
        self.avg_scores.insert(query_type, new_avg);
        self.query_counts.insert(query_type, count + 1);
        
        // 计算调整值：如果平均分数偏低，降低阈值
        let adjustment = if new_avg < 0.5 {
            -0.05 // 降低阈值
        } else if new_avg > 0.8 {
            0.05 // 提高阈值
        } else {
            0.0
        };
        
        self.adjustments.insert(query_type, adjustment);
    }
    
    /// 获取调整值
    pub fn get_adjustment(&self, query_type: &QueryType) -> Option<f32> {
        self.adjustments.get(query_type).copied()
    }
    
    /// 获取平均分数
    pub fn get_avg_score(&self, query_type: &QueryType) -> Option<f32> {
        self.avg_scores.get(query_type).copied()
    }
}

/// 自适应阈值计算器
pub struct AdaptiveThresholdCalculator {
    config: AdaptiveThresholdConfig,
    
    /// 历史统计（线程安全）
    historical_stats: Arc<RwLock<HistoricalStats>>,
}

impl AdaptiveThresholdCalculator {
    /// 创建新的自适应阈值计算器
    pub fn new(config: AdaptiveThresholdConfig) -> Self {
        Self {
            config,
            historical_stats: Arc::new(RwLock::new(HistoricalStats::default())),
        }
    }
    
    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(AdaptiveThresholdConfig::default())
    }
    
    /// 计算阈值
    pub async fn calculate(
        &self,
        query: &str,
        query_type: &QueryType,
        features: &QueryFeatures,
    ) -> f32 {
        // 1. 获取基础阈值
        let mut threshold = self.config.base_thresholds
            .get(query_type)
            .copied()
            .unwrap_or(0.3);
        
        // 2. 查询长度调整
        let length_adjustment = self.calculate_length_adjustment(features.length);
        threshold += length_adjustment * self.config.length_factor;
        
        // 3. 查询复杂度调整
        let complexity = self.calculate_complexity(query, features);
        let complexity_adjustment = (complexity - 0.5) * self.config.complexity_factor;
        threshold += complexity_adjustment;
        
        // 4. 历史反馈调整（可选）
        if self.config.enable_historical_feedback {
            if let Ok(stats) = self.historical_stats.try_read() {
                if let Some(adjustment) = stats.get_adjustment(query_type) {
                    threshold += adjustment;
                }
            }
        }
        
        // 5. 特殊情况处理
        threshold = self.apply_special_rules(query, query_type, features, threshold);
        
        // 6. 限制范围
        threshold.clamp(self.config.min_threshold, self.config.max_threshold)
    }
    
    /// 计算长度调整
    fn calculate_length_adjustment(&self, length: usize) -> f32 {
        match length {
            0..=5 => -0.3,      // 超短查询，大幅降低阈值
            6..=10 => -0.2,     // 短查询，降低阈值
            11..=20 => -0.1,    // 较短查询，略微降低
            21..=40 => 0.0,     // 正常长度，不调整
            41..=80 => 0.1,     // 长查询，提高阈值
            81..=150 => 0.15,   // 很长查询，提高阈值
            _ => 0.2,           // 超长查询，显著提高阈值
        }
    }
    
    /// 计算查询复杂度 (0.0 - 1.0)
    fn calculate_complexity(&self, query: &str, features: &QueryFeatures) -> f32 {
        let mut score = 0.0;
        
        // 1. 词数贡献 (最多0.4)
        let word_score = (features.word_count as f32 / 20.0).min(0.4);
        score += word_score;
        
        // 2. 特殊字符贡献 (0.2)
        if features.has_special_chars {
            score += 0.2;
        }
        
        // 3. 问题型贡献 (0.2)
        if features.is_question {
            score += 0.2;
        }
        
        // 4. 混合语言贡献 (0.2)
        if matches!(features.language, super::query_classifier::Language::Mixed) {
            score += 0.2;
        }
        
        score.clamp(0.0, 1.0)
    }
    
    /// 应用特殊规则
    fn apply_special_rules(
        &self,
        query: &str,
        query_type: &QueryType,
        features: &QueryFeatures,
        threshold: f32,
    ) -> f32 {
        let mut adjusted = threshold;
        
        // 规则1: 全数字查询，阈值设为0
        if query.chars().all(|c| c.is_numeric() || c.is_whitespace()) {
            return 0.0;
        }
        
        // 规则2: 全大写查询（可能是缩写），降低阈值
        if features.is_uppercase && features.length <= 10 {
            adjusted -= 0.2;
        }
        
        // 规则3: 单个词的查询，降低阈值
        if features.word_count == 1 {
            adjusted -= 0.15;
        }
        
        // 规则4: 精确ID类型，强制为0
        if matches!(query_type, QueryType::ExactId | QueryType::Temporal) {
            return 0.0;
        }
        
        // 规则5: 语义查询类型，确保不低于0.4
        if matches!(query_type, QueryType::Semantic) {
            adjusted = adjusted.max(0.4);
        }
        
        adjusted
    }
    
    /// 记录查询反馈
    pub async fn record_feedback(&self, query_type: QueryType, score: f32) {
        if self.config.enable_historical_feedback {
            if let Ok(mut stats) = self.historical_stats.try_write() {
                stats.update(query_type, score);
            }
        }
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> Option<HistoricalStats> {
        self.historical_stats.try_read().ok().map(|s| s.clone())
    }
    
    /// 重置统计数据
    pub async fn reset_stats(&self) {
        if let Ok(mut stats) = self.historical_stats.try_write() {
            *stats = HistoricalStats::default();
        }
    }
}

/// 阈值计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdCalculation {
    /// 最终阈值
    pub threshold: f32,
    
    /// 基础阈值
    pub base_threshold: f32,
    
    /// 长度调整
    pub length_adjustment: f32,
    
    /// 复杂度调整
    pub complexity_adjustment: f32,
    
    /// 历史反馈调整
    pub historical_adjustment: f32,
    
    /// 应用的特殊规则
    pub special_rules: Vec<String>,
}

impl AdaptiveThresholdCalculator {
    /// 计算阈值（带详细信息）
    pub async fn calculate_with_details(
        &self,
        query: &str,
        query_type: &QueryType,
        features: &QueryFeatures,
    ) -> ThresholdCalculation {
        let base_threshold = self.config.base_thresholds
            .get(query_type)
            .copied()
            .unwrap_or(0.3);
        
        let length_adjustment = self.calculate_length_adjustment(features.length) 
            * self.config.length_factor;
        
        let complexity = self.calculate_complexity(query, features);
        let complexity_adjustment = (complexity - 0.5) * self.config.complexity_factor;
        
        let historical_adjustment = if self.config.enable_historical_feedback {
            self.historical_stats.try_read()
                .ok()
                .and_then(|stats| stats.get_adjustment(query_type))
                .unwrap_or(0.0)
        } else {
            0.0
        };
        
        let mut threshold = base_threshold + length_adjustment + 
            complexity_adjustment + historical_adjustment;
        
        let mut special_rules = Vec::new();
        
        // 应用特殊规则并记录
        if query.chars().all(|c| c.is_numeric() || c.is_whitespace()) {
            threshold = 0.0;
            special_rules.push("all_numeric".to_string());
        }
        
        if features.is_uppercase && features.length <= 10 {
            threshold -= 0.2;
            special_rules.push("uppercase_abbreviation".to_string());
        }
        
        if features.word_count == 1 {
            threshold -= 0.15;
            special_rules.push("single_word".to_string());
        }
        
        threshold = threshold.clamp(self.config.min_threshold, self.config.max_threshold);
        
        ThresholdCalculation {
            threshold,
            base_threshold,
            length_adjustment,
            complexity_adjustment,
            historical_adjustment,
            special_rules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::query_classifier::{Language, QueryClassifier};
    
    #[tokio::test]
    async fn test_adaptive_threshold_exact_id() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        let classifier = QueryClassifier::with_default_config();
        
        let query = "P000001";
        let features = classifier.extract_features(query);
        let threshold = calculator.calculate(query, &QueryType::ExactId, &features).await;
        
        assert_eq!(threshold, 0.0);
    }
    
    #[tokio::test]
    async fn test_adaptive_threshold_short_keyword() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        let classifier = QueryClassifier::with_default_config();
        
        let query = "Apple";
        let features = classifier.extract_features(query);
        let threshold = calculator.calculate(query, &QueryType::ShortKeyword, &features).await;
        
        // 短关键词应该有较低的阈值
        assert!(threshold < 0.3);
    }
    
    #[tokio::test]
    async fn test_adaptive_threshold_semantic() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        let classifier = QueryClassifier::with_default_config();
        
        let query = "What is the meaning of life, the universe, and everything?";
        let features = classifier.extract_features(query);
        let threshold = calculator.calculate(query, &QueryType::Semantic, &features).await;
        
        // 语义查询应该有较高的阈值
        assert!(threshold >= 0.4);
    }
    
    #[tokio::test]
    async fn test_length_adjustment() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        
        assert!(calculator.calculate_length_adjustment(5) < 0.0);
        assert_eq!(calculator.calculate_length_adjustment(30), 0.0);
        assert!(calculator.calculate_length_adjustment(100) > 0.0);
    }
    
    #[tokio::test]
    async fn test_complexity_calculation() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        let classifier = QueryClassifier::with_default_config();
        
        let simple_features = QueryFeatures {
            length: 10,
            word_count: 2,
            has_special_chars: false,
            is_question: false,
            has_numbers: false,
            is_uppercase: false,
            language: Language::English,
        };
        
        let complex_features = QueryFeatures {
            length: 50,
            word_count: 10,
            has_special_chars: true,
            is_question: true,
            has_numbers: true,
            is_uppercase: false,
            language: Language::Mixed,
        };
        
        let simple_complexity = calculator.calculate_complexity("hello world", &simple_features);
        let complex_complexity = calculator.calculate_complexity(
            "What's the best way to learn Rust?", 
            &complex_features
        );
        
        assert!(simple_complexity < complex_complexity);
    }
    
    #[tokio::test]
    async fn test_special_rules() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        let classifier = QueryClassifier::with_default_config();
        
        // 全数字查询
        let query = "123456";
        let features = classifier.extract_features(query);
        let threshold = calculator.calculate(query, &QueryType::ShortKeyword, &features).await;
        assert_eq!(threshold, 0.0);
        
        // 单词查询
        let query = "Rust";
        let features = classifier.extract_features(query);
        let threshold = calculator.calculate(query, &QueryType::ShortKeyword, &features).await;
        assert!(threshold < 0.1);
    }
    
    #[tokio::test]
    async fn test_historical_feedback() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        
        // 记录低分数反馈
        calculator.record_feedback(QueryType::ShortKeyword, 0.2).await;
        calculator.record_feedback(QueryType::ShortKeyword, 0.3).await;
        
        let stats = calculator.get_stats().await.unwrap();
        let adjustment = stats.get_adjustment(&QueryType::ShortKeyword);
        
        // 应该建议降低阈值
        assert!(adjustment.is_some());
        assert!(adjustment.unwrap() < 0.0);
    }
    
    #[tokio::test]
    async fn test_calculate_with_details() {
        let calculator = AdaptiveThresholdCalculator::with_default_config();
        let classifier = QueryClassifier::with_default_config();
        
        let query = "What is AI?";
        let features = classifier.extract_features(query);
        let details = calculator.calculate_with_details(
            query, 
            &QueryType::Semantic, 
            &features
        ).await;
        
        assert!(details.threshold >= 0.0 && details.threshold <= 1.0);
        assert_eq!(details.base_threshold, 0.5); // Semantic base threshold
    }
}

