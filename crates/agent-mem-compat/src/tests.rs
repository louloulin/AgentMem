//! Tests for Mem0 compatibility layer

#[cfg(test)]
mod tests {
    use crate::Mem0Config;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_mem0_config_creation() {
        let config = Mem0Config::default();
        assert_eq!(config.vector_store.provider, "chroma");
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.embedder.provider, "openai");
    }

    #[tokio::test]
    async fn test_mem0_config_openai() {
        let config = Mem0Config::openai();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.embedder.provider, "openai");
    }

    #[tokio::test]
    async fn test_mem0_config_validation() {
        let config = Mem0Config::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.vector_store.provider = "".to_string();
        assert!(invalid_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_memory_type_parsing() {
        use crate::utils::parse_memory_type;
        use agent_mem_traits::MemoryType;

        assert_eq!(parse_memory_type("episodic"), MemoryType::Episodic);
        assert_eq!(parse_memory_type("semantic"), MemoryType::Semantic);
        assert_eq!(parse_memory_type("procedural"), MemoryType::Procedural);
        assert_eq!(parse_memory_type("working"), MemoryType::Working);
        assert_eq!(parse_memory_type("unknown"), MemoryType::Episodic);
    }

    #[tokio::test]
    async fn test_user_id_validation() {
        use crate::utils::validate_user_id;

        assert!(validate_user_id("valid_user_123").is_ok());
        assert!(validate_user_id("").is_err());
        assert!(validate_user_id(&"x".repeat(256)).is_err());
        assert!(validate_user_id("user\0id").is_err());
    }

    #[tokio::test]
    async fn test_memory_content_validation() {
        use crate::utils::validate_memory_content;

        assert!(validate_memory_content("Valid content").is_ok());
        assert!(validate_memory_content("").is_err());
        assert!(validate_memory_content(&"x".repeat(100_001)).is_err());
    }

    #[tokio::test]
    async fn test_importance_score_calculation() {
        use crate::utils::calculate_importance_score;

        let metadata = HashMap::new();
        let score = calculate_importance_score("This is important information", &metadata);
        assert!(score > 0.5);

        let score = calculate_importance_score("urgent task", &metadata);
        assert!(score > 0.5); // 0.5 - 0.1 + 0.15 = 0.55
    }

    #[tokio::test]
    async fn test_keyword_extraction() {
        use crate::utils::extract_keywords;

        let keywords = extract_keywords("This is an important message about machine learning");
        assert!(keywords.contains(&"important".to_string()));
        assert!(keywords.contains(&"message".to_string()));
        assert!(keywords.contains(&"machine".to_string()));
        assert!(keywords.contains(&"learning".to_string()));
        assert!(!keywords.contains(&"this".to_string())); // Stop word
        assert!(!keywords.contains(&"is".to_string())); // Stop word
    }

    #[tokio::test]
    async fn test_metadata_sanitization() {
        use crate::utils::sanitize_metadata;

        let mut metadata = HashMap::new();
        metadata.insert(
            "key1".to_string(),
            serde_json::Value::String("value1".to_string()),
        );
        metadata.insert("key2".to_string(), serde_json::Value::Null);
        metadata.insert(
            "key3".to_string(),
            serde_json::Value::String("x".repeat(15000)),
        );

        sanitize_metadata(&mut metadata);

        assert!(metadata.contains_key("key1"));
        assert!(!metadata.contains_key("key2")); // Null value removed
        assert!(metadata.contains_key("key3"));

        if let Some(serde_json::Value::String(value)) = metadata.get("key3") {
            assert!(value.contains("...[truncated]"));
        }
    }
}
