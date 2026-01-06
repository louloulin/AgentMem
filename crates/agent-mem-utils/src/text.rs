//! Text processing utilities

use regex::Regex;

/// Clean and normalize text content
pub fn clean_text(text: &str) -> String {
    // Remove extra whitespace
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    let cleaned = whitespace_regex.replace_all(text.trim(), " ");
    cleaned.to_string()
}

/// Extract sentences from text
pub fn extract_sentences(text: &str) -> Vec<String> {
    // Simple sentence splitting on periods, exclamation marks, and question marks
    let sentence_regex = Regex::new(r"[.!?]+\s*").unwrap();
    sentence_regex
        .split(text)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Truncate text to a maximum length
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        let truncated = &text[..max_length];
        format!("{truncated}...")
    }
}

/// Count words in text
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Extract keywords from text (simple implementation)
pub fn extract_keywords(text: &str, min_length: usize) -> Vec<String> {
    let word_regex = Regex::new(r"\b\w+\b").unwrap();
    let mut keywords: Vec<String> = word_regex
        .find_iter(text)
        .map(|m| m.as_str().to_lowercase())
        .filter(|word| word.len() >= min_length)
        .collect();

    // Remove duplicates and sort
    keywords.sort();
    keywords.dedup();
    keywords
}

/// Check if text contains any of the given patterns
pub fn contains_patterns(text: &str, patterns: &[&str]) -> bool {
    let text_lower = text.to_lowercase();
    patterns
        .iter()
        .any(|pattern| text_lower.contains(&pattern.to_lowercase()))
}

/// Remove URLs from text
pub fn remove_urls(text: &str) -> String {
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    url_regex.replace_all(text, "").to_string()
}

/// Remove email addresses from text
pub fn remove_emails(text: &str) -> String {
    let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    email_regex.replace_all(text, "").to_string()
}

/// Calculate text similarity using simple Jaccard similarity
pub fn jaccard_similarity(text1: &str, text2: &str) -> f32 {
    let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let text = "  This   has    extra   spaces  ";
        let result = clean_text(text);
        assert_eq!(result, "This has extra spaces");
    }

    #[test]
    fn test_extract_sentences() {
        let text = "First sentence. Second sentence! Third sentence?";
        let sentences = extract_sentences(text);
        assert_eq!(
            sentences,
            vec!["First sentence", "Second sentence", "Third sentence"]
        );
    }

    #[test]
    fn test_truncate_text() {
        let text = "This is a long text that should be truncated";
        let result = truncate_text(text, 20);
        assert_eq!(result, "This is a long text ...");
    }

    #[test]
    fn test_word_count() {
        let text = "This is a test sentence";
        assert_eq!(word_count(text), 5);
    }

    #[test]
    fn test_extract_keywords() {
        let text = "This is a test with some important keywords";
        let keywords = extract_keywords(text, 3);
        assert!(keywords.contains(&"test".to_string()));
        assert!(keywords.contains(&"important".to_string()));
        assert!(keywords.contains(&"keywords".to_string()));
    }

    #[test]
    fn test_jaccard_similarity() {
        let text1 = "the quick brown fox";
        let text2 = "the quick red fox";
        let similarity = jaccard_similarity(text1, text2);
        assert!(similarity > 0.5); // Should be 0.75
    }

    #[test]
    fn test_remove_urls() {
        let text = "Check out https://example.com for more info";
        let result = remove_urls(text);
        assert_eq!(result, "Check out  for more info");
    }
}
