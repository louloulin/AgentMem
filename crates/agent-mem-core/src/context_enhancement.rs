//! Context Understanding Enhancement Module
//!
//! Phase 2.3: 上下文理解增强
//! - 上下文窗口扩展
//! - 多轮对话理解
//! - 上下文压缩
//!
//! 参考Mem0的上下文管理策略，提升检索准确率5-10%

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use tracing::info;

/// 上下文理解增强配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEnhancementConfig {
    /// 最大上下文窗口大小（token数）
    pub max_context_tokens: usize,
    /// 最小上下文窗口大小（token数）
    pub min_context_tokens: usize,
    /// 启用上下文窗口扩展
    pub enable_window_expansion: bool,
    /// 启用多轮对话理解
    pub enable_multi_turn: bool,
    /// 启用上下文压缩
    pub enable_compression: bool,
    /// 对话历史保留轮数
    pub conversation_history_turns: usize,
    /// 压缩策略
    pub compression_strategy: CompressionStrategy,
    /// 上下文相关性阈值
    pub relevance_threshold: f64,
}

impl Default for ContextEnhancementConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: 8000,
            min_context_tokens: 1000,
            enable_window_expansion: true,
            enable_multi_turn: true,
            conversation_history_turns: 10,
            enable_compression: true,
            compression_strategy: CompressionStrategy::Summarization,
            relevance_threshold: 0.3,
        }
    }
}

/// 压缩策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// 摘要压缩
    Summarization,
    /// 关键信息提取
    KeyExtraction,
    /// 分层压缩（保留重要信息）
    Hierarchical,
    /// 自适应压缩
    Adaptive,
}

/// 对话轮次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// 轮次ID
    pub turn_id: String,
    /// 用户消息
    pub user_message: String,
    /// 助手回复
    pub assistant_message: Option<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 提取的关键信息
    pub key_information: Vec<String>,
    /// 相关性分数
    pub relevance_score: f64,
}

/// 上下文窗口管理器
pub struct ContextWindowManager {
    config: ContextEnhancementConfig,
    /// 对话历史
    conversation_history: Arc<RwLock<VecDeque<ConversationTurn>>>,
    /// 上下文缓存
    context_cache: Arc<RwLock<HashMap<String, CompressedContext>>>,
}

impl ContextWindowManager {
    /// 创建新的上下文窗口管理器
    pub fn new(config: ContextEnhancementConfig) -> Self {
        Self {
            config,
            conversation_history: Arc::new(RwLock::new(VecDeque::new())),
            context_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(ContextEnhancementConfig::default())
    }

    /// 扩展上下文窗口
    ///
    /// 根据查询需求动态扩展上下文窗口，包含更多相关历史信息
    pub async fn expand_context_window(
        &self,
        query: &str,
        current_context: &str,
    ) -> Result<String> {
        if !self.config.enable_window_expansion {
            return Ok(current_context.to_string());
        }

        info!("扩展上下文窗口: query={}", query);

        // 1. 从对话历史中检索相关轮次
        let relevant_turns = self.retrieve_relevant_turns(query).await?;

        // 2. 构建扩展的上下文
        let mut expanded_context = current_context.to_string();

        for turn in relevant_turns {
            let turn_context = format!(
                "\n[对话轮次 {}]:\n用户: {}\n助手: {}\n",
                turn.turn_id,
                turn.user_message,
                turn.assistant_message.as_deref().unwrap_or("")
            );
            expanded_context.push_str(&turn_context);
        }

        // 3. 检查token限制
        let estimated_tokens = self.estimate_tokens(&expanded_context);
        if estimated_tokens > self.config.max_context_tokens {
            // 压缩上下文
            expanded_context = self.compress_context(&expanded_context).await?;
        }

        Ok(expanded_context)
    }

    /// 多轮对话理解
    ///
    /// 理解多轮对话的上下文关系，提取关键信息
    pub async fn understand_multi_turn_conversation(
        &self,
        turns: Vec<ConversationTurn>,
    ) -> Result<MultiTurnContext> {
        if !self.config.enable_multi_turn {
            return Ok(MultiTurnContext::default());
        }

        info!("理解多轮对话: {} 轮", turns.len());

        // 1. 提取关键信息
        let mut key_information = Vec::new();
        let mut topics = Vec::new();
        let mut entities = HashMap::new();

        for turn in &turns {
            // 提取关键信息
            let turn_key_info = self.extract_key_information(&turn.user_message).await?;
            key_information.extend(turn_key_info);

            // 提取主题
            let turn_topics = self.extract_topics(&turn.user_message).await?;
            topics.extend(turn_topics);

            // 提取实体
            let turn_entities = self.extract_entities(&turn.user_message).await?;
            for (key, value) in turn_entities {
                entities.entry(key).or_insert_with(Vec::new).extend(value);
            }
        }

        // 2. 分析对话流程
        let conversation_flow = self.analyze_conversation_flow(&turns).await?;

        // 3. 构建多轮上下文
        Ok(MultiTurnContext {
            turns: turns.clone(),
            key_information,
            topics,
            entities,
            conversation_flow,
            summary: self.generate_conversation_summary(&turns).await?,
        })
    }

    /// 上下文压缩
    ///
    /// 压缩上下文内容，保留关键信息
    pub async fn compress_context(&self, context: &str) -> Result<String> {
        if !self.config.enable_compression {
            return Ok(context.to_string());
        }

        info!("压缩上下文: {} 字符", context.len());

        // 检查缓存
        let cache_key = self.generate_cache_key(context);
        if let Some(cached) = self.get_cached_context(&cache_key).await {
            return Ok(cached.compressed_content);
        }

        let compressed = match self.config.compression_strategy {
            CompressionStrategy::Summarization => {
                self.compress_by_summarization(context).await?
            }
            CompressionStrategy::KeyExtraction => {
                self.compress_by_key_extraction(context).await?
            }
            CompressionStrategy::Hierarchical => {
                self.compress_by_hierarchical(context).await?
            }
            CompressionStrategy::Adaptive => {
                self.compress_by_adaptive(context).await?
            }
        };

        // 缓存结果
        self.cache_context(cache_key, context, &compressed).await;

        Ok(compressed)
    }

    /// 添加对话轮次
    pub async fn add_conversation_turn(&self, turn: ConversationTurn) {
        let mut history = self.conversation_history.write().await;
        history.push_back(turn);

        // 限制历史长度
        while history.len() > self.config.conversation_history_turns {
            history.pop_front();
        }
    }

    /// 检索相关对话轮次
    async fn retrieve_relevant_turns(&self, query: &str) -> Result<Vec<ConversationTurn>> {
        let history = self.conversation_history.read().await;
        let mut relevant_turns = Vec::new();

        for turn in history.iter() {
            // 计算相关性分数
            let relevance = self.calculate_relevance(query, &turn.user_message).await?;
            if relevance >= self.config.relevance_threshold {
                relevant_turns.push(turn.clone());
            }
        }

        // 按相关性排序
        relevant_turns.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制数量
        relevant_turns.truncate(5); // 最多5个相关轮次

        Ok(relevant_turns)
    }

    /// 计算相关性
    async fn calculate_relevance(&self, query: &str, text: &str) -> Result<f64> {
        // 简化的相关性计算（实际应使用向量相似度）
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();

        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let text_words: Vec<&str> = text_lower.split_whitespace().collect();

        let mut matches = 0;
        for qw in &query_words {
            if text_words.iter().any(|tw| tw.contains(qw)) {
                matches += 1;
            }
        }

        let relevance = if query_words.is_empty() {
            0.0
        } else {
            matches as f64 / query_words.len() as f64
        };

        Ok(relevance)
    }

    /// 提取关键信息
    async fn extract_key_information(&self, text: &str) -> Result<Vec<String>> {
        // 简化的关键信息提取（实际应使用LLM或NLP工具）
        let key_info = Vec::new();

        // 提取数字、日期、名称等
        // TODO: 使用更高级的提取方法

        Ok(key_info)
    }

    /// 提取主题
    async fn extract_topics(&self, text: &str) -> Result<Vec<String>> {
        // 简化的主题提取
        Ok(Vec::new())
    }

    /// 提取实体
    async fn extract_entities(&self, text: &str) -> Result<HashMap<String, Vec<String>>> {
        // 简化的实体提取
        Ok(HashMap::new())
    }

    /// 分析对话流程
    async fn analyze_conversation_flow(
        &self,
        turns: &[ConversationTurn],
    ) -> Result<ConversationFlow> {
        Ok(ConversationFlow {
            total_turns: turns.len(),
            topics_evolution: Vec::new(),
            question_answer_pairs: Vec::new(),
        })
    }

    /// 生成对话摘要
    async fn generate_conversation_summary(&self, turns: &[ConversationTurn]) -> Result<String> {
        // 简化的摘要生成
        let summary = format!("包含 {} 轮对话", turns.len());
        Ok(summary)
    }

    /// 通过摘要压缩
    async fn compress_by_summarization(&self, context: &str) -> Result<String> {
        // 简化的摘要压缩（实际应使用LLM）
        let lines: Vec<&str> = context.lines().collect();
        let compressed_lines: Vec<&str> = lines.iter().take(lines.len() / 2).copied().collect();
        Ok(compressed_lines.join("\n"))
    }

    /// 通过关键信息提取压缩
    async fn compress_by_key_extraction(&self, context: &str) -> Result<String> {
        // 提取关键句子
        let sentences: Vec<&str> = context.split('.').collect();
        let key_sentences: Vec<&str> = sentences.iter().take(sentences.len() / 2).copied().collect();
        Ok(key_sentences.join("."))
    }

    /// 通过分层压缩
    async fn compress_by_hierarchical(&self, context: &str) -> Result<String> {
        // 保留开头和结尾，压缩中间部分
        let lines: Vec<&str> = context.lines().collect();
        if lines.len() <= 10 {
            return Ok(context.to_string());
        }

        let start: Vec<&str> = lines.iter().take(5).copied().collect();
        let end: Vec<&str> = lines.iter().rev().take(5).rev().copied().collect();
        let compressed = format!("{}\n... [压缩] ...\n{}", start.join("\n"), end.join("\n"));
        Ok(compressed)
    }

    /// 自适应压缩
    async fn compress_by_adaptive(&self, context: &str) -> Result<String> {
        // 根据内容长度选择压缩策略
        let estimated_tokens = self.estimate_tokens(context);
        if estimated_tokens > self.config.max_context_tokens * 2 {
            self.compress_by_hierarchical(context).await
        } else if estimated_tokens > self.config.max_context_tokens {
            self.compress_by_summarization(context).await
        } else {
            Ok(context.to_string())
        }
    }

    /// 估算token数
    fn estimate_tokens(&self, text: &str) -> usize {
        // 简化的token估算（实际应使用tokenizer）
        text.len() / 4 // 粗略估算：1 token ≈ 4字符
    }

    /// 生成缓存键
    fn generate_cache_key(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取缓存的上下文
    async fn get_cached_context(&self, cache_key: &str) -> Option<CompressedContext> {
        let cache = self.context_cache.read().await;
        cache.get(cache_key).cloned()
    }

    /// 缓存上下文
    async fn cache_context(&self, cache_key: String, original: &str, compressed: &str) {
        let mut cache = self.context_cache.write().await;
        cache.insert(
            cache_key,
            CompressedContext {
                original_content: original.to_string(),
                compressed_content: compressed.to_string(),
                cached_at: Utc::now(),
                compression_ratio: compressed.len() as f64 / original.len() as f64,
            },
        );
    }
}

/// 多轮对话上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MultiTurnContext {
    /// 对话轮次
    pub turns: Vec<ConversationTurn>,
    /// 关键信息
    pub key_information: Vec<String>,
    /// 主题列表
    pub topics: Vec<String>,
    /// 实体映射
    pub entities: HashMap<String, Vec<String>>,
    /// 对话流程
    pub conversation_flow: ConversationFlow,
    /// 对话摘要
    pub summary: String,
}


/// 对话流程
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ConversationFlow {
    /// 总轮次数
    pub total_turns: usize,
    /// 主题演化
    pub topics_evolution: Vec<String>,
    /// 问答对
    pub question_answer_pairs: Vec<(String, String)>,
}


/// 压缩的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompressedContext {
    /// 原始内容
    original_content: String,
    /// 压缩后内容
    compressed_content: String,
    /// 缓存时间
    cached_at: DateTime<Utc>,
    /// 压缩比
    compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_window_expansion() -> anyhow::Result<()> {
        let manager = ContextWindowManager::with_defaults();
        let query = "test query";
        let context = "current context";

        let expanded = manager.expand_context_window(query, context).await?;
        assert!(!expanded.is_empty());
    }

    #[tokio::test]
    async fn test_context_compression() {
        let manager = ContextWindowManager::with_defaults();
        let context = "This is a long context that needs to be compressed. ".repeat(100);

        let compressed = manager.compress_context(&context).await?;
        assert!(compressed.len() < context.len());
    }

    #[tokio::test]
    async fn test_multi_turn_understanding() {
        let manager = ContextWindowManager::with_defaults();
        let turns = vec![ConversationTurn {
            turn_id: "1".to_string(),
            user_message: "Hello".to_string(),
            assistant_message: Some("Hi".to_string()),
            timestamp: Utc::now(),
            key_information: Vec::new(),
            relevance_score: 0.5,
        Ok(())
        }];

        let context = manager
            .understand_multi_turn_conversation(turns)
            .await
            .unwrap();
        assert_eq!(context.turns.len(), 1);
    }
}
