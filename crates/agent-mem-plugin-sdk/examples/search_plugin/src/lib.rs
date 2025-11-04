//! Search Algorithm Plugin Example
//!
//! Demonstrates custom search and ranking algorithms for memory retrieval.

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory item for search
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Memory {
    id: String,
    content: String,
    memory_type: String,
    metadata: HashMap<String, serde_json::Value>,
}

/// Search request
#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    memories: Vec<Memory>,
    algorithm: Option<String>, // "fuzzy", "semantic", "keyword"
    limit: Option<usize>,
}

/// Search result
#[derive(Serialize)]
struct SearchResult {
    memory_id: String,
    score: f32,
    matched_terms: Vec<String>,
}

/// Search response
#[derive(Serialize)]
struct SearchResponse {
    success: bool,
    results: Vec<SearchResult>,
    algorithm_used: String,
    error: Option<String>,
}

/// Perform search with custom algorithm
#[plugin_fn]
pub fn search(input: String) -> FnResult<String> {
    // Parse input
    let request: SearchRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let response = SearchResponse {
                success: false,
                results: vec![],
                algorithm_used: "none".to_string(),
                error: Some(format!("Invalid request: {}", e)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    // Log search request
    log(
        LogLevel::Info,
        &format!(
            "Searching {} memories with query: {}",
            request.memories.len(),
            request.query
        ),
    )?;

    // Select algorithm
    let algorithm = request.algorithm.as_deref().unwrap_or("keyword");
    let mut results = match algorithm {
        "fuzzy" => fuzzy_search(&request.query, &request.memories),
        "semantic" => semantic_search(&request.query, &request.memories),
        "keyword" => keyword_search(&request.query, &request.memories),
        _ => keyword_search(&request.query, &request.memories),
    };

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Apply limit
    let limit = request.limit.unwrap_or(10);
    results.truncate(limit);

    let response = SearchResponse {
        success: true,
        results,
        algorithm_used: algorithm.to_string(),
        error: None,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Keyword-based search (exact and partial matches)
fn keyword_search(query: &str, memories: &[Memory]) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    memories
        .iter()
        .filter_map(|memory| {
            let content_lower = memory.content.to_lowercase();
            let mut score = 0.0f32;
            let mut matched_terms = Vec::new();

            // Check for exact phrase match
            if content_lower.contains(&query_lower) {
                score += 10.0;
                matched_terms.push(query.to_string());
            }

            // Check for individual term matches
            for term in &query_terms {
                if content_lower.contains(term) {
                    score += 2.0;
                    matched_terms.push(term.to_string());
                }
            }

            // Boost score based on memory type
            if memory.memory_type == "Semantic" {
                score *= 1.2;
            }

            if score > 0.0 {
                Some(SearchResult {
                    memory_id: memory.id.clone(),
                    score,
                    matched_terms,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Fuzzy search (tolerates typos and variations)
fn fuzzy_search(query: &str, memories: &[Memory]) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    memories
        .iter()
        .filter_map(|memory| {
            let content_lower = memory.content.to_lowercase();
            let content_terms: Vec<&str> = content_lower.split_whitespace().collect();
            let mut score = 0.0f32;
            let mut matched_terms = Vec::new();

            for query_term in &query_terms {
                for content_term in &content_terms {
                    let similarity = fuzzy_similarity(query_term, content_term);
                    if similarity > 0.7 {
                        score += similarity * 3.0;
                        matched_terms.push(content_term.to_string());
                    }
                }
            }

            if score > 0.0 {
                Some(SearchResult {
                    memory_id: memory.id.clone(),
                    score,
                    matched_terms,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Semantic search (simulated - in real implementation would use embeddings)
fn semantic_search(query: &str, memories: &[Memory]) -> Vec<SearchResult> {
    // In a real implementation, this would:
    // 1. Generate embeddings for query and memories
    // 2. Calculate cosine similarity
    // 3. Return ranked results
    //
    // For now, we'll simulate semantic similarity using simple heuristics
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    memories
        .iter()
        .map(|memory| {
            let content_lower = memory.content.to_lowercase();
            let content_terms: Vec<&str> = content_lower.split_whitespace().collect();

            // Simulate semantic similarity
            let mut score = 0.0f32;
            let mut matched_terms = Vec::new();

            // Term overlap score
            for query_term in &query_terms {
                for content_term in &content_terms {
                    if query_term == content_term {
                        score += 5.0;
                        matched_terms.push(content_term.to_string());
                    } else if fuzzy_similarity(query_term, content_term) > 0.8 {
                        score += 3.0;
                    }
                }
            }

            // Length-normalized score
            let query_len = query_terms.len() as f32;
            let content_len = content_terms.len() as f32;
            let length_factor = 1.0 / (1.0 + (query_len - content_len).abs() / query_len);
            score *= length_factor;

            SearchResult {
                memory_id: memory.id.clone(),
                score,
                matched_terms,
            }
        })
        .collect()
}

/// Calculate fuzzy similarity between two strings (simple Levenshtein-based)
fn fuzzy_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0;
    }

    let distance = levenshtein_distance(a, b);
    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate Levenshtein distance
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

/// Rerank search results using custom scoring
#[plugin_fn]
pub fn rerank(input: String) -> FnResult<String> {
    #[derive(Deserialize)]
    struct RerankRequest {
        results: Vec<SearchResult>,
        boost_factors: HashMap<String, f32>,
    }

    #[derive(Serialize)]
    struct RerankResponse {
        success: bool,
        results: Vec<SearchResult>,
    }

    let request: RerankRequest = serde_json::from_str(&input)?;
    let mut results = request.results;

    // Apply boost factors
    for result in &mut results {
        if let Some(boost) = request.boost_factors.get(&result.memory_id) {
            result.score *= boost;
        }
    }

    // Re-sort by score
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let response = RerankResponse {
        success: true,
        results,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = serde_json::json!({
        "name": "search-plugin",
        "version": "0.1.0",
        "description": "Custom search algorithms: keyword, fuzzy, and semantic search",
        "author": "AgentMem Team",
        "plugin_type": "SearchAlgorithm",
        "required_capabilities": ["SearchAccess", "LoggingAccess"]
    });

    Ok(metadata.to_string())
}

