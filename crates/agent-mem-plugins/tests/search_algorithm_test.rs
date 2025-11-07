//! Search algorithm integration tests

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

/// Search result
#[derive(Debug, Clone, Deserialize, Serialize)]
struct SearchResult {
    memory_id: String,
    score: f32,
    matched_terms: Vec<String>,
}

#[test]
fn test_keyword_search_algorithm() {
    // Create test memories
    let memories = vec![
        Memory {
            id: "1".to_string(),
            content: "Rust is a systems programming language".to_string(),
            memory_type: "Semantic".to_string(),
            metadata: HashMap::new(),
        },
        Memory {
            id: "2".to_string(),
            content: "Python is great for data science".to_string(),
            memory_type: "Episodic".to_string(),
            metadata: HashMap::new(),
        },
        Memory {
            id: "3".to_string(),
            content: "Rust provides memory safety without garbage collection".to_string(),
            memory_type: "Semantic".to_string(),
            metadata: HashMap::new(),
        },
    ];

    // Simulate keyword search
    let query = "Rust programming";
    let mut results: Vec<SearchResult> = Vec::new();

    for memory in &memories {
        let query_lower = query.to_lowercase();
        let content_lower = memory.content.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

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
            results.push(SearchResult {
                memory_id: memory.id.clone(),
                score,
                matched_terms,
            });
        }
    }

    // Sort by score
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Verify results
    assert!(!results.is_empty(), "Should find matching memories");
    assert_eq!(results[0].memory_id, "1", "Exact match should rank first");
    assert!(
        results[0].score > results[1].score,
        "Scores should be ordered"
    );
}

#[test]
fn test_fuzzy_search_similarity() {
    // Test fuzzy string similarity
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

    // Test exact matches
    assert_eq!(fuzzy_similarity("hello", "hello"), 1.0);

    // Test similar words
    let sim1 = fuzzy_similarity("hello", "hallo");
    assert!(sim1 > 0.7, "Similar words should have high similarity");

    // Test different words
    let sim2 = fuzzy_similarity("hello", "world");
    assert!(sim2 < 0.5, "Different words should have low similarity");

    // Test typos
    let sim3 = fuzzy_similarity("programming", "programing");
    assert!(sim3 > 0.8, "Typos should still have high similarity");
}

#[test]
fn test_search_result_ranking() {
    let mut results = vec![
        SearchResult {
            memory_id: "1".to_string(),
            score: 5.0,
            matched_terms: vec!["rust".to_string()],
        },
        SearchResult {
            memory_id: "2".to_string(),
            score: 10.0,
            matched_terms: vec!["rust".to_string(), "programming".to_string()],
        },
        SearchResult {
            memory_id: "3".to_string(),
            score: 3.0,
            matched_terms: vec!["programming".to_string()],
        },
    ];

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Verify ranking
    assert_eq!(results[0].memory_id, "2");
    assert_eq!(results[1].memory_id, "1");
    assert_eq!(results[2].memory_id, "3");
}

#[test]
fn test_search_with_limit() {
    let memories: Vec<Memory> = (0..20)
        .map(|i| Memory {
            id: i.to_string(),
            content: format!("Memory content {}", i),
            memory_type: "Semantic".to_string(),
            metadata: HashMap::new(),
        })
        .collect();

    // All memories match
    let mut results: Vec<SearchResult> = memories
        .iter()
        .enumerate()
        .map(|(i, m)| SearchResult {
            memory_id: m.id.clone(),
            score: (20 - i) as f32,
            matched_terms: vec!["content".to_string()],
        })
        .collect();

    // Sort and limit
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results.truncate(10);

    assert_eq!(results.len(), 10, "Should limit results to 10");
}

#[test]
fn test_rerank_with_boost_factors() {
    let mut results = vec![
        SearchResult {
            memory_id: "1".to_string(),
            score: 5.0,
            matched_terms: vec!["rust".to_string()],
        },
        SearchResult {
            memory_id: "2".to_string(),
            score: 3.0,
            matched_terms: vec!["python".to_string()],
        },
        SearchResult {
            memory_id: "3".to_string(),
            score: 4.0,
            matched_terms: vec!["javascript".to_string()],
        },
    ];

    // Apply boost factors
    let boost_factors: HashMap<String, f32> = [
        ("1".to_string(), 1.0),
        ("2".to_string(), 2.0), // Boost Python result
        ("3".to_string(), 1.0),
    ]
    .iter()
    .cloned()
    .collect();

    for result in &mut results {
        if let Some(boost) = boost_factors.get(&result.memory_id) {
            result.score *= boost;
        }
    }

    // Re-sort by score
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Python result should now be first due to boost
    assert_eq!(results[0].memory_id, "2");
    assert_eq!(results[0].score, 6.0); // 3.0 * 2.0
}

#[test]
fn test_semantic_search_simulation() {
    // Simulate semantic search scoring
    fn calculate_semantic_score(query_terms: &[&str], content_terms: &[&str]) -> f32 {
        let mut score = 0.0f32;

        for query_term in query_terms {
            for content_term in content_terms {
                if query_term == content_term {
                    score += 5.0;
                }
            }
        }

        // Length-normalized score
        let query_len = query_terms.len() as f32;
        let content_len = content_terms.len() as f32;
        let length_factor = 1.0 / (1.0 + (query_len - content_len).abs() / query_len);
        score * length_factor
    }

    let query_terms = vec!["rust", "programming"];
    let content1_terms = vec!["rust", "is", "a", "programming", "language"];
    let content2_terms = vec!["python", "is", "great"];

    let score1 = calculate_semantic_score(&query_terms, &content1_terms);
    let score2 = calculate_semantic_score(&query_terms, &content2_terms);

    assert!(score1 > score2, "Matching content should have higher score");
    assert!(score1 > 0.0, "Should have positive score for matches");
}

#[test]
fn test_empty_query_handling() {
    let memories = vec![Memory {
        id: "1".to_string(),
        content: "Test content".to_string(),
        memory_type: "Semantic".to_string(),
        metadata: HashMap::new(),
    }];

    let query = "";
    let query_terms: Vec<&str> = query.split_whitespace().collect();

    assert!(query_terms.is_empty(), "Empty query should have no terms");
}

#[test]
fn test_memory_type_boosting() {
    let memories = vec![
        Memory {
            id: "1".to_string(),
            content: "Rust programming".to_string(),
            memory_type: "Semantic".to_string(),
            metadata: HashMap::new(),
        },
        Memory {
            id: "2".to_string(),
            content: "Rust programming".to_string(),
            memory_type: "Episodic".to_string(),
            metadata: HashMap::new(),
        },
    ];

    let mut results: Vec<SearchResult> = memories
        .iter()
        .map(|m| {
            let mut score = 10.0; // Base score for matching content

            // Boost score based on memory type
            if m.memory_type == "Semantic" {
                score *= 1.2;
            }

            SearchResult {
                memory_id: m.id.clone(),
                score,
                matched_terms: vec!["rust".to_string()],
            }
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Semantic memory should rank higher due to boost
    assert_eq!(results[0].memory_id, "1");
    assert!(results[0].score > results[1].score);
}
