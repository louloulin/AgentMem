//! BM25 search implementation
//! 
//! BM25 (Best Matching 25) is a ranking function used for information retrieval.

use super::SearchConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BM25 document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Document {
    /// Document ID
    pub id: String,
    /// Tokenized content
    pub tokens: Vec<String>,
    /// Original content
    pub content: String,
}

impl BM25Document {
    /// Create new BM25 document
    pub fn new(id: String, content: String) -> Self {
        let tokens = tokenize(&content);
        Self { id, tokens, content }
    }
}

/// Tokenize text into words
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// BM25 scorer
#[derive(Debug, Clone)]
pub struct BM25Scorer {
    /// BM25 k1 parameter
    k1: f32,
    /// BM25 b parameter
    b: f32,
    /// Document frequency map
    doc_freq: HashMap<String, usize>,
    /// Total documents
    num_docs: usize,
    /// Average document length
    avg_doc_len: f32,
}

impl BM25Scorer {
    /// Create new BM25 scorer
    pub fn new() -> Self {
        Self {
            k1: 1.5,
            b: 0.75,
            doc_freq: HashMap::new(),
            num_docs: 0,
            avg_doc_len: 0.0,
        }
    }

    /// Set k1 parameter
    pub fn with_k1(mut self, k1: f32) -> Self {
        self.k1 = k1;
        self
    }

    /// Set b parameter
    pub fn with_b(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    /// Build index from documents
    pub fn build_index(&mut self, documents: &[BM25Document]) {
        self.num_docs = documents.len();
        
        // Calculate average document length
        let total_len: usize = documents.iter().map(|d| d.tokens.len()).sum();
        self.avg_doc_len = if self.num_docs > 0 {
            total_len as f32 / self.num_docs as f32
        } else {
            0.0
        };

        // Build document frequency map
        self.doc_freq.clear();
        for doc in documents {
            let mut seen = std::collections::HashSet::new();
            for token in &doc.tokens {
                if seen.insert(token.clone()) {
                    *self.doc_freq.entry(token.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    /// Calculate BM25 score for a query against a document
    pub fn score(&self, query_tokens: &[String], doc: &BM25Document) -> f32 {
        let doc_len = doc.tokens.len() as f32;
        let mut score = 0.0f32;

        for qt in query_tokens {
            let tf = doc.tokens.iter().filter(|t| *t == qt).count() as f32;
            if tf == 0.0 {
                continue;
            }

            let df = self.doc_freq.get(qt).copied().unwrap_or(0) as f32;
            let idf = if df > 0.0 {
                ((self.num_docs as f32 - df + 0.5) / (df + 0.5)).ln().max(0.0)
            } else {
                0.0
            };

            let numerator = tf * (self.k1 + 1.0);
            let denominator = tf + self.k1 * (1.0 - self.b + self.b * doc_len / self.avg_doc_len.max(1.0));
            score += idf * numerator / denominator;
        }

        score
    }

    /// Calculate IDF for a term
    pub fn idf(&self, term: &str) -> f32 {
        let df = self.doc_freq.get(term).copied().unwrap_or(0) as f32;
        if df > 0.0 {
            ((self.num_docs as f32 - df + 0.5) / (df + 0.5)).ln().max(0.0)
        } else {
            0.0
        }
    }
}

impl Default for BM25Scorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello World! This is a test.");
        assert_eq!(tokens, vec!["hello", "world", "this", "is", "a", "test"]);
    }

    #[test]
    fn test_bm25_document() {
        let doc = BM25Document::new("1".to_string(), "Hello World".to_string());
        assert_eq!(doc.id, "1");
        assert!(!doc.tokens.is_empty());
    }

    #[test]
    fn test_bm25_scorer_build_index() {
        let docs = vec![
            BM25Document::new("1".to_string(), "hello world".to_string()),
            BM25Document::new("2".to_string(), "hello there".to_string()),
        ];
        
        let mut scorer = BM25Scorer::new();
        scorer.build_index(&docs);
        
        assert_eq!(scorer.num_docs, 2);
        assert!(scorer.avg_doc_len > 0.0);
    }

    #[test]
    fn test_bm25_scorer_idf() {
        let scorer = BM25Scorer {
            k1: 1.5,
            b: 0.75,
            doc_freq: [("hello".to_string(), 2)].into_iter().collect(),
            num_docs: 2,
            avg_doc_len: 2.0,
        };
        
        let idf = scorer.idf("hello");
        assert!(idf >= 0.0);
    }
}
