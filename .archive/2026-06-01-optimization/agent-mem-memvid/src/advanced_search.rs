//! Advanced search capabilities using MemVid's built-in search engines
//!
//! This module provides a high-level search interface that leverages:
//! - Tantivy for full-text search (when "lex" feature is enabled)
//! - HNSW for vector similarity search (when "vec" feature is enabled)

use crate::error::{MemvidError, Result};
use memvid_core::{Memvid, SearchHit, SearchRequest, SearchResponse};

/// Advanced search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub top_k: usize,
    /// Number of characters for text snippets
    pub snippet_chars: usize,
    /// Filter by URI pattern
    pub uri_pattern: Option<String>,
    /// Time range filter (start timestamp)
    pub after_ts: Option<i64>,
    /// Time range filter (end timestamp)
    pub before_ts: Option<i64>,
    /// Enable fuzzy search
    pub fuzzy: bool,
    /// Enable phrase search (exact phrase matching)
    pub phrase: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            top_k: 10,
            snippet_chars: 200,
            uri_pattern: Some("mv2://memory/".to_string()),
            after_ts: None,
            before_ts: None,
            fuzzy: false,
            phrase: false,
        }
    }
}

impl SearchOptions {
    /// Create new search options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of results
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set snippet length
    pub fn with_snippet_chars(mut self, chars: usize) -> Self {
        self.snippet_chars = chars;
        self
    }

    /// Enable fuzzy search
    pub fn with_fuzzy(mut self, fuzzy: bool) -> Self {
        self.fuzzy = fuzzy;
        self
    }

    /// Enable phrase search
    pub fn with_phrase(mut self, phrase: bool) -> Self {
        self.phrase = phrase;
        self
    }

    /// Filter by time range
    pub fn with_time_range(mut self, after: Option<i64>, before: Option<i64>) -> Self {
        self.after_ts = after;
        self.before_ts = before;
        self
    }

    /// Build query string from options
    fn build_query(&self, base_query: &str) -> String {
        let mut query = base_query.to_string();

        if self.phrase {
            // Wrap in quotes for exact phrase matching
            query = format!("\"{}\"", query);
        }

        if self.fuzzy {
            // Add fuzzy operator (~)
            query = format!("{}~", query);
        }

        query
    }
}

/// Enhanced search result with Memory objects
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The original MemVid search hit
    pub hit: SearchHit,
    /// Extracted memory ID (if available)
    pub memory_id: Option<String>,
    /// Relevance score (0-1)
    pub score: f32,
    /// Text snippet
    pub snippet: String,
}

/// Advanced search engine for AgentMem memories
pub struct AdvancedSearch {
    _path: String,
}

impl AdvancedSearch {
    /// Create a new advanced search instance
    pub fn new(path: impl Into<String>) -> Self {
        Self { _path: path.into() }
    }

    /// Full-text search with options
    pub fn search(
        &self,
        mem: &mut Memvid,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let query = options.build_query(query);

        let request = SearchRequest {
            query: query.clone(),
            top_k: options.top_k,
            snippet_chars: options.snippet_chars,
            uri: options.uri_pattern.clone(),
            scope: None,
            cursor: None,
            no_sketch: false,
            as_of_frame: None,
            as_of_ts: None,
        };

        let response: SearchResponse = mem
            .search(request)
            .map_err(|e| MemvidError::Memvid(format!("Search failed: {}", e)))?;

        // Convert to our SearchResult format
        let results: Vec<SearchResult> = response
            .hits
            .into_iter()
            .map(|hit| {
                // Extract memory ID from URI
                let memory_id = hit.uri.strip_prefix("mv2://memory/").map(|s| s.to_string());

                let score = hit.score.unwrap_or(0.0);
                let snippet = hit.text.clone();

                SearchResult {
                    hit,
                    memory_id,
                    score,
                    snippet,
                }
            })
            .collect();

        Ok(results)
    }

    /// Simple full-text search
    pub fn search_simple(
        &self,
        mem: &mut Memvid,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchHit>> {
        let request = SearchRequest {
            query: query.to_string(),
            top_k,
            snippet_chars: 200,
            uri: Some("mv2://memory/".to_string()),
            scope: None,
            cursor: None,
            no_sketch: false,
            as_of_frame: None,
            as_of_ts: None,
        };

        let response = mem
            .search(request)
            .map_err(|e| MemvidError::Memvid(format!("Search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Fuzzy search for approximate matching
    pub fn search_fuzzy(
        &self,
        mem: &mut Memvid,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchHit>> {
        // Add fuzzy operator
        let fuzzy_query = format!("{}~", query);

        let request = SearchRequest {
            query: fuzzy_query,
            top_k,
            snippet_chars: 200,
            uri: Some("mv2://memory/".to_string()),
            scope: None,
            cursor: None,
            no_sketch: false,
            as_of_frame: None,
            as_of_ts: None,
        };

        let response = mem
            .search(request)
            .map_err(|e| MemvidError::Memvid(format!("Fuzzy search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Phrase search for exact matching
    pub fn search_phrase(
        &self,
        mem: &mut Memvid,
        phrase: &str,
        top_k: usize,
    ) -> Result<Vec<SearchHit>> {
        // Wrap in quotes for exact phrase matching
        let phrase_query = format!("\"{}\"", phrase);

        let request = SearchRequest {
            query: phrase_query,
            top_k,
            snippet_chars: 200,
            uri: Some("mv2://memory/".to_string()),
            scope: None,
            cursor: None,
            no_sketch: false,
            as_of_frame: None,
            as_of_ts: None,
        };

        let response = mem
            .search(request)
            .map_err(|e| MemvidError::Memvid(format!("Phrase search failed: {}", e)))?;

        Ok(response.hits)
    }

    /// Multi-field search (search across content, tags, and metadata)
    pub fn search_multi(
        &self,
        mem: &mut Memvid,
        queries: Vec<&str>,
        top_k: usize,
    ) -> Result<Vec<SearchHit>> {
        // Combine queries with OR
        let combined_query = queries.join(" OR ");

        let request = SearchRequest {
            query: combined_query,
            top_k,
            snippet_chars: 200,
            uri: Some("mv2://memory/".to_string()),
            scope: None,
            cursor: None,
            no_sketch: false,
            as_of_frame: None,
            as_of_ts: None,
        };

        let response = mem
            .search(request)
            .map_err(|e| MemvidError::Memvid(format!("Multi-field search failed: {}", e)))?;

        Ok(response.hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.top_k, 10);
        assert_eq!(options.snippet_chars, 200);
        assert!(!options.fuzzy);
        assert!(!options.phrase);
    }

    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions::new()
            .with_top_k(20)
            .with_fuzzy(true)
            .with_phrase(true);

        assert_eq!(options.top_k, 20);
        assert!(options.fuzzy);
        assert!(options.phrase);
    }

    #[test]
    fn test_build_query_simple() {
        let options = SearchOptions::new();
        let query = options.build_query("hello world");
        assert_eq!(query, "hello world");
    }

    #[test]
    fn test_build_query_fuzzy() {
        let options = SearchOptions::new().with_fuzzy(true);
        let query = options.build_query("hello");
        assert_eq!(query, "hello~");
    }

    #[test]
    fn test_build_query_phrase() {
        let options = SearchOptions::new().with_phrase(true);
        let query = options.build_query("hello world");
        assert_eq!(query, "\"hello world\"");
    }
}
