/// Memory summarization strategies for prompt optimization
///
/// This module provides various strategies to compress memory content
/// while preserving the most important information.

use std::fmt;

/// Summarization strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummarizationStrategy {
    /// Simple truncation - cuts at max_chars
    SimpleTruncate,
    /// Smart truncation - preserves head and tail
    SmartTruncate,
    /// Key sentences extraction (TODO: Phase 2)
    KeySentences,
}

impl Default for SummarizationStrategy {
    fn default() -> Self {
        Self::SmartTruncate
    }
}

impl fmt::Display for SummarizationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SimpleTruncate => write!(f, "SimpleTruncate"),
            Self::SmartTruncate => write!(f, "SmartTruncate"),
            Self::KeySentences => write!(f, "KeySentences"),
        }
    }
}

/// Memory content summarizer
///
/// Compresses memory content to fit within token limits while
/// preserving semantic meaning.
///
/// # Examples
///
/// ```
/// use agent_mem_core::prompt::MemorySummarizer;
///
/// let summarizer = MemorySummarizer::new(100);
/// let long_text = "a".repeat(200);
/// let summary = summarizer.summarize(&long_text);
/// assert!(summary.len() <= 120); // Includes ellipsis markers
/// ```
pub struct MemorySummarizer {
    max_chars: usize,
    strategy: SummarizationStrategy,
}

impl MemorySummarizer {
    /// Create a new summarizer with specified character limit
    ///
    /// # Arguments
    ///
    /// * `max_chars` - Maximum characters per memory (default strategy: SmartTruncate)
    pub fn new(max_chars: usize) -> Self {
        Self {
            max_chars,
            strategy: SummarizationStrategy::default(),
        }
    }

    /// Create a new summarizer with custom strategy
    pub fn with_strategy(max_chars: usize, strategy: SummarizationStrategy) -> Self {
        Self { max_chars, strategy }
    }

    /// Summarize content according to configured strategy
    pub fn summarize(&self, content: &str) -> String {
        match self.strategy {
            SummarizationStrategy::SimpleTruncate => self.simple_truncate(content),
            SummarizationStrategy::SmartTruncate => self.smart_truncate(content),
            SummarizationStrategy::KeySentences => self.extract_key_sentences(content),
        }
    }

    /// Simple truncation strategy
    ///
    /// Cuts content at max_chars and adds "..."
    fn simple_truncate(&self, content: &str) -> String {
        if content.len() <= self.max_chars {
            content.to_string()
        } else {
            // Ensure we don't split multi-byte characters
            let truncate_at = content
                .char_indices()
                .take_while(|(idx, _)| *idx < self.max_chars.saturating_sub(3))
                .last()
                .map(|(idx, ch)| idx + ch.len_utf8())
                .unwrap_or(0);

            format!("{}...", &content[..truncate_at])
        }
    }

    /// Smart truncation strategy
    ///
    /// Preserves beginning (2/3) and ending (1/3) of content
    /// with omission marker in the middle
    fn smart_truncate(&self, content: &str) -> String {
        if content.len() <= self.max_chars {
            return content.to_string();
        }

        let head_len = (self.max_chars * 2) / 3;
        let tail_len = self.max_chars / 3;

        // Calculate omission marker size
        let omitted_chars = content.len().saturating_sub(head_len + tail_len);
        let marker = format!("...[省略{}字符]...", omitted_chars);

        // Adjust head and tail to accommodate marker
        let marker_len = marker.len();
        let available = self.max_chars.saturating_sub(marker_len);
        let adjusted_head = (available * 2) / 3;
        let adjusted_tail = available / 3;

        // Extract head safely
        let head = content
            .char_indices()
            .take_while(|(idx, _)| *idx < adjusted_head)
            .last()
            .map(|(idx, ch)| &content[..idx + ch.len_utf8()])
            .unwrap_or("");

        // Extract tail safely
        let tail_start = content.len().saturating_sub(adjusted_tail);
        let tail = content
            .char_indices()
            .skip_while(|(idx, _)| *idx < tail_start)
            .next()
            .map(|(idx, _)| &content[idx..])
            .unwrap_or("");

        format!("{}{}{}", head, marker, tail)
    }

    /// Key sentences extraction strategy
    ///
    /// TODO: Phase 2 - Implement NLP-based key sentence extraction
    /// For now, falls back to smart truncation
    fn extract_key_sentences(&self, content: &str) -> String {
        // Phase 2: Implement TF-IDF or LLM-based extraction
        self.smart_truncate(content)
    }

    /// Get max characters limit
    pub fn max_chars(&self) -> usize {
        self.max_chars
    }

    /// Get current strategy
    pub fn strategy(&self) -> SummarizationStrategy {
        self.strategy
    }
}

impl Default for MemorySummarizer {
    /// Default summarizer with 200 char limit
    fn default() -> Self {
        Self::new(200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_truncate_short_content() {
        let summarizer = MemorySummarizer::with_strategy(50, SummarizationStrategy::SimpleTruncate);
        let short_text = "Short text";
        let result = summarizer.summarize(short_text);
        assert_eq!(result, short_text);
    }

    #[test]
    fn test_simple_truncate_long_content() {
        let summarizer = MemorySummarizer::with_strategy(50, SummarizationStrategy::SimpleTruncate);
        let long_text = "a".repeat(100);
        let result = summarizer.summarize(&long_text);
        assert!(result.len() <= 53); // 50 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_smart_truncate_short_content() {
        let summarizer = MemorySummarizer::new(200);
        let short_text = "Short text";
        let result = summarizer.summarize(short_text);
        assert_eq!(result, short_text);
    }

    #[test]
    fn test_smart_truncate_long_content() {
        let summarizer = MemorySummarizer::new(100);
        let text = format!("{}{}{}",
            "START",
            "x".repeat(200),
            "END"
        );
        let result = summarizer.summarize(&text);

        assert!(result.contains("START"), "Should preserve start");
        assert!(result.contains("END"), "Should preserve end");
        assert!(result.contains("省略"), "Should contain omission marker");
        assert!(result.len() <= 150, "Should be within reasonable limit");
    }

    #[test]
    fn test_smart_truncate_with_unicode() {
        let summarizer = MemorySummarizer::new(50);
        let text = format!("{}{}{}",
            "开始",
            "中".repeat(100),
            "结束"
        );
        let result = summarizer.summarize(&text);

        assert!(result.contains("开始"), "Should preserve Chinese start");
        assert!(result.contains("结束"), "Should preserve Chinese end");
        assert!(result.contains("省略"), "Should handle Unicode correctly");
    }

    #[test]
    fn test_default_strategy() {
        let summarizer = MemorySummarizer::new(100);
        assert_eq!(summarizer.strategy(), SummarizationStrategy::SmartTruncate);
    }

    #[test]
    fn test_key_sentences_fallback() {
        let summarizer = MemorySummarizer::with_strategy(100, SummarizationStrategy::KeySentences);
        let text = "a".repeat(200);
        let result = summarizer.summarize(&text);
        
        // Should fall back to smart truncate
        assert!(result.contains("省略"));
    }

    #[test]
    fn test_empty_content() {
        let summarizer = MemorySummarizer::new(100);
        let result = summarizer.summarize("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_exact_max_chars() {
        let summarizer = MemorySummarizer::new(50);
        let text = "a".repeat(50);
        let result = summarizer.summarize(&text);
        assert_eq!(result.len(), 50);
        assert!(!result.contains("..."));
    }
}

