//! P1 Input Validation Layer Tests
//!
//! Comprehensive tests for the input validation layer implemented in P1.
//!
//! Run with:
//! ```bash
//! cargo test --package agent-mem-server test_p1_validation
//! ```

use agent_mem_server::middleware::validation::*;

#[cfg(test)]
mod validation_tests {
    use super::*;
    use std::collections::HashMap;

    // ==================== Hash Optimization Tests ====================

    #[test]
    fn test_generate_cache_key_consistency() {
        // Test that same inputs generate same cache key
        let key1 = agent_mem_server::routes::memory::cache::generate_cache_key(
            "test query",
            Some("agent-123"),
            Some("user-456"),
            Some(10),
        );

        let key2 = agent_mem_server::routes::memory::cache::generate_cache_key(
            "test query",
            Some("agent-123"),
            Some("user-456"),
            Some(10),
        );

        assert_eq!(key1, key2, "Same inputs should generate same cache key");
    }

    #[test]
    fn test_generate_cache_key_uniqueness() {
        // Test that different inputs generate different cache keys
        let key1 = agent_mem_server::routes::memory::cache::generate_cache_key(
            "query one",
            Some("agent-123"),
            Some("user-456"),
            Some(10),
        );

        let key2 = agent_mem_server::routes::memory::cache::generate_cache_key(
            "query two",
            Some("agent-123"),
            Some("user-456"),
            Some(10),
        );

        assert_ne!(key1, key2, "Different queries should generate different cache keys");
    }

    #[test]
    fn test_generate_cache_key_performance() {
        use std::time::Instant;

        // Performance test: should generate keys very quickly (< 1μs per key)
        let iterations = 10_000;
        let start = Instant::now();

        for i in 0..iterations {
            let _ = agent_mem_server::routes::memory::cache::generate_cache_key(
                &format!("test query {}", i),
                Some(&format!("agent-{}", i % 100)),
                Some(&format!("user-{}", i % 100)),
                Some(10 + i % 90),
            );
        }

        let duration = start.elapsed();
        let avg_time = duration / iterations;

        // XxHash64 should be < 1μs per hash
        assert!(
            avg_time.as_micros() < 1,
            "Hash function too slow: {}μs per hash (expected < 1μs)",
            avg_time.as_micros()
        );

        println!(
            "✅ Hash performance: {} hashes in {:?} ({}μs per hash)",
            iterations,
            duration,
            avg_time.as_micros()
        );
    }

    // ==================== Add Memory Request Tests ====================

    #[test]
    fn test_add_memory_valid_request() {
        let result = validate_add_memory_request(
            "This is a valid memory content about Rust programming".to_string(),
            None,
            None,
            Some(0.7),
            Some("agent-123".to_string()),
            Some("session-456".to_string()),
        );

        assert!(result.is_ok(), "Valid request should pass validation");
    }

    #[test]
    fn test_add_memory_empty_content() {
        let result = validate_add_memory_request(
            "".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Empty content should fail validation");
        assert!(result.unwrap_err().contains("content"));
    }

    #[test]
    fn test_add_memory_content_too_long() {
        let result = validate_add_memory_request(
            "a".repeat(50_001), // Exceeds MAX_CONTENT_LENGTH
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Content too long should fail validation");
        assert!(result.unwrap_err().contains("length"));
    }

    #[test]
    fn test_add_memory_contains_script_tag() {
        let result = validate_add_memory_request(
            "Check out this <script>alert('xss')</script> content".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Content with script tag should fail validation");
        assert!(result.unwrap_err().contains("html_or_script"));
    }

    #[test]
    fn test_add_memory_contains_iframe_tag() {
        let result = validate_add_memory_request(
            "Here's an iframe <iframe src='evil.com'></iframe> example".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Content with iframe tag should fail validation");
    }

    #[test]
    fn test_add_memory_contains_javascript_protocol() {
        let result = validate_add_memory_request(
            "Click javascript:alert('xss') here".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Content with javascript: protocol should fail validation");
    }

    #[test]
    fn test_add_memory_contains_event_handler() {
        let result = validate_add_memory_request(
            "Image with onload='alert(1)' event".to_string(),
            None,
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Content with event handler should fail validation");
    }

    #[test]
    fn test_add_memory_invalid_importance_too_high() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            Some(1.5), // Exceeds max 1.0
            None,
            None,
        );

        assert!(result.is_err(), "Importance > 1.0 should fail validation");
    }

    #[test]
    fn test_add_memory_invalid_importance_negative() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            Some(-0.1), // Negative
            None,
            None,
        );

        assert!(result.is_err(), "Negative importance should fail validation");
    }

    #[test]
    fn test_add_memory_valid_importance_boundaries() {
        // Test minimum boundary
        let result_min = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            Some(0.0),
            None,
            None,
        );
        assert!(result_min.is_ok(), "Importance = 0.0 should be valid");

        // Test maximum boundary
        let result_max = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            Some(1.0),
            None,
            None,
        );
        assert!(result_max.is_ok(), "Importance = 1.0 should be valid");
    }

    #[test]
    fn test_add_memory_metadata_too_many_entries() {
        let mut metadata = HashMap::new();
        for i in 0..51 { // Exceeds MAX_METADATA_ENTRIES (50)
            metadata.insert(format!("key{}", i), "value".to_string());
        }

        let result = validate_add_memory_request(
            "Valid content".to_string(),
            Some(metadata),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Too many metadata entries should fail validation");
        assert!(result.unwrap_err().contains("Metadata entries count"));
    }

    #[test]
    fn test_add_memory_metadata_invalid_key_characters() {
        let mut metadata = HashMap::new();
        metadata.insert("invalid key!".to_string(), "value".to_string());

        let result = validate_add_memory_request(
            "Valid content".to_string(),
            Some(metadata),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Invalid metadata key characters should fail validation");
        assert!(result.unwrap_err().contains("invalid_metadata_key"));
    }

    #[test]
    fn test_add_memory_metadata_key_too_long() {
        let mut metadata = HashMap::new();
        metadata.insert("a".repeat(101), "value".to_string()); // Exceeds MAX_METADATA_KEY_LENGTH

        let result = validate_add_memory_request(
            "Valid content".to_string(),
            Some(metadata),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Metadata key too long should fail validation");
        assert!(result.unwrap_err().contains("Metadata key length"));
    }

    #[test]
    fn test_add_memory_metadata_value_too_long() {
        let mut metadata = HashMap::new();
        metadata.insert("valid_key".to_string(), "a".repeat(1_001)); // Exceeds MAX_METADATA_VALUE_LENGTH

        let result = validate_add_memory_request(
            "Valid content".to_string(),
            Some(metadata),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Metadata value too long should fail validation");
        assert!(result.unwrap_err().contains("Metadata value length"));
    }

    #[test]
    fn test_add_memory_valid_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("valid_key".to_string(), "valid_value".to_string());
        metadata.insert("another_key".to_string(), "another_value".to_string());

        let result = validate_add_memory_request(
            "Valid content".to_string(),
            Some(metadata),
            None,
            None,
            None,
            None,
        );

        assert!(result.is_ok(), "Valid metadata should pass validation");
    }

    #[test]
    fn test_add_memory_tags_too_many() {
        let tags: Vec<String> = (0..21).map(|i| format!("tag{}", i)).collect(); // Exceeds MAX_TAGS_COUNT

        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            Some(tags),
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Too many tags should fail validation");
        assert!(result.unwrap_err().contains("Tags count"));
    }

    #[test]
    fn test_add_memory_tag_invalid_characters() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            Some(vec!["invalid tag!".to_string()]),
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Tag with invalid characters should fail validation");
        assert!(result.unwrap_err().contains("invalid_tag"));
    }

    #[test]
    fn test_add_memory_tag_too_long() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            Some(vec!["a".repeat(51)]), // Exceeds MAX_TAG_LENGTH
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Tag too long should fail validation");
        assert!(result.unwrap_err().contains("Tag length"));
    }

    #[test]
    fn test_add_memory_valid_tags() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            Some(vec!["rust".to_string(), "programming".to_string(), "web".to_string()]),
            None,
            None,
            None,
        );

        assert!(result.is_ok(), "Valid tags should pass validation");
    }

    #[test]
    fn test_add_memory_agent_id_too_long() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            None,
            Some("a".repeat(101)), // Exceeds max length
            None,
        );

        assert!(result.is_err(), "Agent ID too long should fail validation");
    }

    #[test]
    fn test_add_memory_session_id_too_long() {
        let result = validate_add_memory_request(
            "Valid content".to_string(),
            None,
            None,
            None,
            None,
            Some("a".repeat(101)), // Exceeds max length
        );

        assert!(result.is_err(), "Session ID too long should fail validation");
    }

    // ==================== Update Memory Request Tests ====================

    #[test]
    fn test_update_memory_valid_request() {
        let result = validate_update_memory_request(
            "memory-123".to_string(),
            "Updated content".to_string(),
            None,
            None,
            Some(0.8),
        );

        assert!(result.is_ok(), "Valid update request should pass validation");
    }

    #[test]
    fn test_update_memory_empty_id() {
        let result = validate_update_memory_request(
            "".to_string(),
            "Updated content".to_string(),
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Empty ID should fail validation");
    }

    #[test]
    fn test_update_memory_id_too_long() {
        let result = validate_update_memory_request(
            "a".repeat(101),
            "Updated content".to_string(),
            None,
            None,
            None,
        );

        assert!(result.is_err(), "ID too long should fail validation");
    }

    #[test]
    fn test_update_memory_content_with_html() {
        let result = validate_update_memory_request(
            "memory-123".to_string(),
            "Updated <script>alert('xss')</script> content".to_string(),
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Content with HTML should fail validation");
    }

    // ==================== Search Memory Request Tests ====================

    #[test]
    fn test_search_valid_request() {
        let result = validate_search_request(
            "rust programming".to_string(),
            10,
            Some("agent-123".to_string()),
            Some(vec!["rust".to_string()]),
            Some(0.3),
        );

        assert!(result.is_ok(), "Valid search request should pass validation");
    }

    #[test]
    fn test_search_empty_query() {
        let result = validate_search_request(
            "".to_string(),
            10,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Empty query should fail validation");
    }

    #[test]
    fn test_search_query_too_long() {
        let result = validate_search_request(
            "a".repeat(1_001), // Exceeds max length
            10,
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Query too long should fail validation");
    }

    #[test]
    fn test_search_limit_too_low() {
        let result = validate_search_request(
            "rust".to_string(),
            0, // Below min 1
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Limit < 1 should fail validation");
    }

    #[test]
    fn test_search_limit_too_high() {
        let result = validate_search_request(
            "rust".to_string(),
            101, // Exceeds max 100
            None,
            None,
            None,
        );

        assert!(result.is_err(), "Limit > 100 should fail validation");
    }

    #[test]
    fn test_search_valid_limit_boundaries() {
        // Test minimum boundary
        let result_min = validate_search_request(
            "rust".to_string(),
            1,
            None,
            None,
            None,
        );
        assert!(result_min.is_ok(), "Limit = 1 should be valid");

        // Test maximum boundary
        let result_max = validate_search_request(
            "rust".to_string(),
            100,
            None,
            None,
            None,
        );
        assert!(result_max.is_ok(), "Limit = 100 should be valid");
    }

    #[test]
    fn test_search_invalid_tag() {
        let result = validate_search_request(
            "rust".to_string(),
            10,
            None,
            Some(vec!["invalid tag!".to_string()]),
            None,
        );

        assert!(result.is_err(), "Invalid tag should fail validation");
    }

    #[test]
    fn test_search_min_importance_out_of_range() {
        let result_min_high = validate_search_request(
            "rust".to_string(),
            10,
            None,
            None,
            Some(1.5), // Exceeds max
        );

        assert!(result_min_high.is_err(), "Min importance > 1.0 should fail validation");

        let result_min_negative = validate_search_request(
            "rust".to_string(),
            10,
            None,
            None,
            Some(-0.1), // Negative
        );

        assert!(result_min_negative.is_err(), "Negative min importance should fail validation");
    }

    // ==================== Delete Memory Request Tests ====================

    #[test]
    fn test_delete_valid_request() {
        let result = validate_delete_request("memory-123".to_string());

        assert!(result.is_ok(), "Valid delete request should pass validation");
    }

    #[test]
    fn test_delete_empty_id() {
        let result = validate_delete_request("".to_string());

        assert!(result.is_err(), "Empty ID should fail validation");
    }

    #[test]
    fn test_delete_id_too_long() {
        let result = validate_delete_request("a".repeat(101));

        assert!(result.is_err(), "ID too long should fail validation");
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_complex_valid_request() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "programming".to_string());
        metadata.insert("language".to_string(), "rust".to_string());
        metadata.insert("difficulty".to_string(), "advanced".to_string());

        let result = validate_add_memory_request(
            "Learn about Rust ownership and borrowing system for memory safety".to_string(),
            Some(metadata),
            Some(vec![
                "rust".to_string(),
                "programming".to_string(),
                "memory-safety".to_string(),
            ]),
            Some(0.9),
            Some("agent-expert".to_string()),
            Some("session-learning-123".to_string()),
        );

        assert!(result.is_ok(), "Complex valid request should pass validation");
    }

    #[test]
    fn test_multiple_validation_errors() {
        let mut metadata = HashMap::new();
        metadata.insert("invalid key!".to_string(), "value".to_string());

        let result = validate_add_memory_request(
            "<script>alert('xss')</script>".to_string(),
            Some(metadata),
            Some(vec!["invalid tag!".to_string()]),
            Some(2.5), // Invalid importance
            Some("a".repeat(200)), // Invalid agent_id
            None,
        );

        assert!(result.is_err(), "Request with multiple errors should fail validation");
        // The error should mention at least one of the issues
        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("script") || error_msg.contains("invalid") || error_msg.contains("range"),
            "Error should mention one of the validation failures"
        );
    }

    #[test]
    fn test_edge_case_max_values() {
        let mut metadata = HashMap::new();
        for i in 0..50 {
            metadata.insert(format!("key{}", i), "a".repeat(1_000));
        }

        let tags: Vec<String> = (0..20).map(|i| format!("tag{}", i)).collect();

        let result = validate_add_memory_request(
            "a".repeat(50_000), // Max content length
            Some(metadata),
            Some(tags),
            Some(1.0),
            Some("a".repeat(100)), // Max agent_id length
            Some("a".repeat(100)), // Max session_id length
        );

        assert!(result.is_ok(), "Request at maximum limits should be valid");
    }

    #[test]
    fn test_edge_case_min_values() {
        let result = validate_add_memory_request(
            "a".to_string(), // Min content length
            None,
            None,
            Some(0.0), // Min importance
            None,
            None,
        );

        assert!(result.is_ok(), "Request at minimum values should be valid");
    }
}
