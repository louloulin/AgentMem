//! 评估指标计算模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 准确性指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    /// F1 Score
    pub f1_score: f64,
    /// BLEU-1 Score
    pub bleu1_score: f64,
    /// ROUGE-L Score
    pub rouge_l_score: f64,
    /// Cosine Similarity
    pub cosine_similarity: f64,
    /// LLM-as-a-Judge Score (如果可用)
    pub llm_judge_score: Option<f64>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 平均搜索延迟 (ms)
    pub avg_search_latency_ms: f64,
    /// 平均生成延迟 (ms)
    pub avg_generation_latency_ms: f64,
    /// P95搜索延迟 (ms)
    pub p95_search_latency_ms: f64,
    /// P95总延迟 (ms)
    pub p95_total_latency_ms: f64,
    /// 平均Token消耗
    pub avg_tokens: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_search_latency_ms: 0.0,
            avg_generation_latency_ms: 0.0,
            p95_search_latency_ms: 0.0,
            p95_total_latency_ms: 0.0,
            avg_tokens: 0,
        }
    }
}

/// 指标计算器
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// 计算所有准确性指标
    pub fn calculate_accuracy_metrics(
        expected: &str,
        actual: &str,
    ) -> AccuracyMetrics {
        let f1_score = Self::calculate_f1_score(expected, actual);
        let bleu1_score = Self::calculate_bleu1(expected, actual);
        let rouge_l_score = Self::calculate_rouge_l(expected, actual);
        let cosine_similarity = Self::calculate_cosine_similarity(expected, actual);

        AccuracyMetrics {
            f1_score,
            bleu1_score,
            rouge_l_score,
            cosine_similarity,
            llm_judge_score: None, // TODO: 实现LLM-as-a-Judge
        }
    }

    /// 计算F1 Score
    fn calculate_f1_score(expected: &str, actual: &str) -> f64 {
        let expected_lower = expected.to_lowercase();
        let actual_lower = actual.to_lowercase();
        let expected_words: Vec<&str> = expected_lower.split_whitespace().collect();
        let actual_words: Vec<&str> = actual_lower.split_whitespace().collect();

        if expected_words.is_empty() || actual_words.is_empty() {
            return 0.0;
        }

        // 计算交集
        let mut matches = 0;
        let mut expected_counts = HashMap::new();
        for word in &expected_words {
            *expected_counts.entry(word).or_insert(0) += 1;
        }

        for word in &actual_words {
            if let Some(count) = expected_counts.get_mut(word) {
                if *count > 0 {
                    *count -= 1;
                    matches += 1;
                }
            }
        }

        let precision = matches as f64 / actual_words.len() as f64;
        let recall = matches as f64 / expected_words.len() as f64;

        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        }
    }

    /// 计算BLEU-1 Score
    fn calculate_bleu1(expected: &str, actual: &str) -> f64 {
        let expected_lower = expected.to_lowercase();
        let actual_lower = actual.to_lowercase();
        let expected_words: Vec<&str> = expected_lower.split_whitespace().collect();
        let actual_words: Vec<&str> = actual_lower.split_whitespace().collect();

        if expected_words.is_empty() || actual_words.is_empty() {
            return 0.0;
        }

        // 计算unigram精确度
        let mut matches = 0;
        let mut expected_counts = HashMap::new();
        for word in &expected_words {
            *expected_counts.entry(word).or_insert(0) += 1;
        }

        for word in &actual_words {
            if let Some(count) = expected_counts.get_mut(word) {
                if *count > 0 {
                    *count -= 1;
                    matches += 1;
                }
            }
        }

        matches as f64 / actual_words.len() as f64
    }

    /// 计算ROUGE-L Score (简化版，基于最长公共子序列)
    fn calculate_rouge_l(expected: &str, actual: &str) -> f64 {
        let expected_lower = expected.to_lowercase();
        let actual_lower = actual.to_lowercase();
        let expected_words: Vec<&str> = expected_lower.split_whitespace().collect();
        let actual_words: Vec<&str> = actual_lower.split_whitespace().collect();

        if expected_words.is_empty() || actual_words.is_empty() {
            return 0.0;
        }

        // 计算最长公共子序列长度
        let lcs_length = Self::longest_common_subsequence(&expected_words, &actual_words);

        let precision = lcs_length as f64 / actual_words.len() as f64;
        let recall = lcs_length as f64 / expected_words.len() as f64;

        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        }
    }

    /// 计算最长公共子序列
    fn longest_common_subsequence(a: &[&str], b: &[&str]) -> usize {
        let m = a.len();
        let n = b.len();
        let mut dp = vec![vec![0; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if a[i - 1] == b[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    // 修正下标：应该比较左侧和上侧的子问题
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        dp[m][n]
    }

    /// 计算Cosine Similarity (简化版，基于词频)
    fn calculate_cosine_similarity(expected: &str, actual: &str) -> f64 {
        let expected_lower = expected.to_lowercase();
        let actual_lower = actual.to_lowercase();
        let expected_words: Vec<&str> = expected_lower.split_whitespace().collect();
        let actual_words: Vec<&str> = actual_lower.split_whitespace().collect();

        if expected_words.is_empty() || actual_words.is_empty() {
            return 0.0;
        }

        // 构建词频向量
        let mut all_words = std::collections::HashSet::new();
        for word in &expected_words {
            all_words.insert(*word);
        }
        for word in &actual_words {
            all_words.insert(*word);
        }

        let mut expected_vec = vec![0.0; all_words.len()];
        let mut actual_vec = vec![0.0; all_words.len()];

        let word_index: HashMap<&str, usize> = all_words
            .iter()
            .enumerate()
            .map(|(i, &word)| (word, i))
            .collect();

        for word in &expected_words {
            if let Some(&idx) = word_index.get(word) {
                expected_vec[idx] += 1.0;
            }
        }

        for word in &actual_words {
            if let Some(&idx) = word_index.get(word) {
                actual_vec[idx] += 1.0;
            }
        }

        // 计算余弦相似度
        let dot_product: f64 = expected_vec
            .iter()
            .zip(actual_vec.iter())
            .map(|(a, b)| a * b)
            .sum();

        let expected_norm: f64 = expected_vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        let actual_norm: f64 = actual_vec.iter().map(|x| x * x).sum::<f64>().sqrt();

        if expected_norm == 0.0 || actual_norm == 0.0 {
            0.0
        } else {
            dot_product / (expected_norm * actual_norm)
        }
    }

    /// 计算综合得分（基于多个指标的平均值）
    pub fn calculate_composite_score(metrics: &AccuracyMetrics) -> f64 {
        let mut scores = vec![
            metrics.f1_score,
            metrics.bleu1_score,
            metrics.rouge_l_score,
            metrics.cosine_similarity,
        ];

        if let Some(llm_score) = metrics.llm_judge_score {
            scores.push(llm_score);
        }

        if scores.is_empty() {
            return 0.0;
        }

        scores.iter().sum::<f64>() / scores.len() as f64
    }
}
