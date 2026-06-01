//! Hybrid search implementation
//! 
//! Combines vector and BM25 search using Reciprocal Rank Fusion (RRF).

use super::{SearchConfig, SearchResult};
use serde::{Deserialize, Serialize};

/// Hybrid search result combining vector and BM25 scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// Document ID
    pub id: String,
    /// Combined score
    pub score: f32,
    /// Vector search score
    pub vector_score: Option<f32>,
    /// BM25 score
    pub bm25_score: Option<f32>,
    /// Rank in result set
    pub rank: usize,
}

impl HybridSearchResult {
    /// Create new hybrid result
    pub fn new(id: String, score: f32) -> Self {
        Self {
            id,
            score,
            vector_score: None,
            bm25_score: None,
            rank: 0,
        }
    }
}

/// Reciprocal Rank Fusion (RRF) scorer
/// 
/// RRF is a rank aggregation method that combines multiple ranking signals.
/// Formula: score = Σ (1 / (k + rank)), where k is a constant (usually 60).
#[derive(Debug, Clone)]
pub struct RRFScorer {
    /// RRF k parameter
    pub k: u32,
}

impl RRFScorer {
    /// Create new RRF scorer
    pub fn new() -> Self {
        Self { k: 60 }
    }

    /// Set k parameter
    pub fn with_k(mut self, k: u32) -> Self {
        self.k = k;
        self
    }

    /// Calculate RRF score for a document
    /// 
    /// Parameters:
    /// - ranks: List of ranks from different sources (1-indexed)
    pub fn score(&self, ranks: &[usize]) -> f32 {
        ranks.iter().map(|r| 1.0 / (self.k as f32 + *r as f32)).sum()
    }
}

impl Default for RRFScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Hybrid searcher combining vector and BM25
pub struct HybridSearcher {
    rrf_scorer: RRFScorer,
    vector_weight: f32,
    bm25_weight: f32,
}

impl HybridSearcher {
    /// Create new hybrid searcher
    pub fn new() -> Self {
        Self {
            rrf_scorer: RRFScorer::new(),
            vector_weight: 0.5,
            bm25_weight: 0.5,
        }
    }

    /// Set vector weight
    pub fn with_vector_weight(mut self, weight: f32) -> Self {
        self.vector_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set BM25 weight
    pub fn with_bm25_weight(mut self, weight: f32) -> Self {
        self.bm25_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Combine scores using weighted average
    pub fn combine_scores(&self, vector_score: Option<f32>, bm25_score: Option<f32>) -> f32 {
        let v_score = vector_score.unwrap_or(0.0);
        let b_score = bm25_score.unwrap_or(0.0);
        
        let total_weight = if vector_score.is_some() { self.vector_weight } else { 0.0 }
            + if bm25_score.is_some() { self.bm25_weight } else { 0.0 };
        
        if total_weight > 0.0 {
            (v_score * self.vector_weight + b_score * self.bm25_weight) / total_weight
        } else {
            0.0
        }
    }
}

impl Default for HybridSearcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Rank aggregation result
#[derive(Debug, Clone)]
pub struct RankAggregation {
    doc_id: String,
    rrf_score: f32,
    #[allow(dead_code)]
    vector_rank: Option<usize>,
    #[allow(dead_code)]
    bm25_rank: Option<usize>,
}

impl RankAggregation {
    /// Get document ID
    pub fn doc_id(&self) -> &str {
        &self.doc_id
    }

    /// Get RRF score
    pub fn rrf_score(&self) -> f32 {
        self.rrf_score
    }
}

/// Check if two f32 values are approximately equal (within 0.001)
fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_scorer() {
        let scorer = RRFScorer::new();
        // Single rank: 1 / (60 + 1) = 0.01639...
        let score = scorer.score(&[1]);
        assert!((score - 1.0 / 61.0).abs() < 0.0001);
    }

    #[test]
    fn test_rrf_scorer_multiple_ranks() {
        let scorer = RRFScorer::new();
        // Two ranks: 1/(60+1) + 1/(60+2)
        let score = scorer.score(&[1, 2]);
        let expected = 1.0 / 61.0 + 1.0 / 62.0;
        assert!((score - expected).abs() < 0.0001);
    }

    #[test]
    fn test_hybrid_searcher_combine_scores() {
        let searcher = HybridSearcher::new();
        let combined = searcher.combine_scores(Some(0.9), Some(0.7));
        // (0.9*0.5 + 0.7*0.5) / 1.0 = 0.8
        assert!(approx_eq(combined, 0.8));
    }

    #[test]
    fn test_hybrid_searcher_partial_scores() {
        let searcher = HybridSearcher::new();
        let combined = searcher.combine_scores(Some(0.9), None);
        assert!(approx_eq(combined, 0.9)); // Only vector score
    }

    #[test]
    fn test_hybrid_searcher_with_weights() {
        let searcher = HybridSearcher::new()
            .with_vector_weight(0.7)
            .with_bm25_weight(0.3);
        let combined = searcher.combine_scores(Some(1.0), Some(0.5));
        // (1.0*0.7 + 0.5*0.3) / 1.0 = 0.85
        assert!(approx_eq(combined, 0.85));
    }
}
