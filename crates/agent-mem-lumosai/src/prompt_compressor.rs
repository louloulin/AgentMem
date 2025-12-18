//! Prompt Compressor - Prompt压缩器
//!
//! 智能压缩Prompt，减少LLM延迟和成本

use lumosai_core::llm::Message as LumosMessage;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// 压缩策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// 截断最旧的消息
    TruncateOldest,
    /// 摘要旧消息（需要LLM，暂不实现）
    SummarizeOld,
    /// 选择最重要的消息（基于简单启发式）
    SelectImportant,
}

/// Prompt压缩器配置
#[derive(Debug, Clone)]
pub struct PromptCompressorConfig {
    /// 最大token数
    pub max_tokens: usize,
    /// 压缩策略
    pub strategy: CompressionStrategy,
    /// 启用压缩
    pub enable_compression: bool,
}

impl Default for PromptCompressorConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2000, // 目标: < 2000 tokens
            strategy: CompressionStrategy::TruncateOldest,
            enable_compression: true,
        }
    }
}

/// Prompt压缩器
pub struct PromptCompressor {
    config: PromptCompressorConfig,
}

impl PromptCompressor {
    /// 创建新的压缩器
    pub fn new(config: PromptCompressorConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(PromptCompressorConfig::default())
    }

    /// 压缩消息列表
    pub fn compress_messages(&self, messages: Vec<LumosMessage>) -> Vec<LumosMessage> {
        if !self.config.enable_compression {
            return messages;
        }

        let total_tokens = self.estimate_tokens(&messages);
        info!(
            "📊 [PROMPT-COMPRESS] Original: {} messages, ~{} tokens",
            messages.len(),
            total_tokens
        );

        if total_tokens <= self.config.max_tokens {
            debug!("   ✅ Within token limit, no compression needed");
            return messages;
        }

        let compressed = match self.config.strategy {
            CompressionStrategy::TruncateOldest => {
                self.compress_truncate_oldest(messages, total_tokens)
            }
            CompressionStrategy::SelectImportant => {
                self.compress_select_important(messages, total_tokens)
            }
            CompressionStrategy::SummarizeOld => {
                // 暂不实现，需要LLM调用
                warn!("   ⚠️  SummarizeOld strategy not implemented, using TruncateOldest");
                self.compress_truncate_oldest(messages, total_tokens)
            }
        };

        let compressed_tokens = self.estimate_tokens(&compressed);
        info!(
            "   ✅ Compressed: {} messages, ~{} tokens (reduced {}%)",
            compressed.len(),
            compressed_tokens,
            ((total_tokens - compressed_tokens) as f64 / total_tokens as f64) * 100.0
        );

        compressed
    }

    /// 截断最旧的消息
    fn compress_truncate_oldest(
        &self,
        messages: Vec<LumosMessage>,
        _total_tokens: usize,
    ) -> Vec<LumosMessage> {
        if messages.is_empty() {
            return messages;
        }

        // 保留系统消息和最后一条用户消息
        let mut result = Vec::new();
        let mut tokens_used = 0;

        // 1. 保留所有系统消息
        for msg in messages.iter() {
            if matches!(msg.role, lumosai_core::llm::Role::System) {
                let tokens = self.estimate_message_tokens(msg);
                if tokens_used + tokens <= self.config.max_tokens {
                    result.push(msg.clone());
                    tokens_used += tokens;
                }
            }
        }

        // 2. 从后往前保留消息，直到达到token限制
        for msg in messages.iter().rev() {
            if matches!(msg.role, lumosai_core::llm::Role::System) {
                continue; // 已处理
            }

            let tokens = self.estimate_message_tokens(msg);
            if tokens_used + tokens <= self.config.max_tokens {
                result.insert(result.len() - result.iter().rev().take_while(|m| matches!(m.role, lumosai_core::llm::Role::System)).count(), msg.clone());
                tokens_used += tokens;
            } else {
                break;
            }
        }

        // 反转以保持时间顺序
        result.reverse();
        result
    }

    /// 选择最重要的消息
    fn compress_select_important(
        &self,
        messages: Vec<LumosMessage>,
        total_tokens: usize,
    ) -> Vec<LumosMessage> {
        // 简化实现：保留系统消息 + 最后N条消息
        // 实际应该基于重要性评分
        self.compress_truncate_oldest(messages, total_tokens)
    }

    /// 估算消息的token数（简化实现：4字符 ≈ 1 token）
    fn estimate_message_tokens(&self, message: &LumosMessage) -> usize {
        // 简化：4字符 ≈ 1 token
        // 实际应该使用tiktoken等库
        message.content.chars().count() / 4 + 10 // +10 for role and formatting
    }

    /// 估算总token数
    fn estimate_tokens(&self, messages: &[LumosMessage]) -> usize {
        messages
            .iter()
            .map(|msg| self.estimate_message_tokens(msg))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumosai_core::llm::Role;

    #[test]
    fn test_estimate_tokens() {
        let compressor = PromptCompressor::with_defaults();
        let msg = LumosMessage {
            role: Role::User,
            content: "Hello, world!".to_string(),
            metadata: None,
            name: None,
        };
        let tokens = compressor.estimate_message_tokens(&msg);
        assert!(tokens > 0);
    }

    #[test]
    fn test_compress_no_compression_needed() {
        let compressor = PromptCompressor::with_defaults();
        let messages = vec![
            LumosMessage {
                role: Role::User,
                content: "Short message".to_string(),
                metadata: None,
                name: None,
            },
        ];
        let compressed = compressor.compress_messages(messages.clone());
        assert_eq!(compressed.len(), messages.len());
    }
}

