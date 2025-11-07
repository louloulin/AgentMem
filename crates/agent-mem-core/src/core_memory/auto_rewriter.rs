//! Auto Rewriter - LLM 驱动的自动重写器
//!
//! 当 Block 内容超过限制时，使用 LLM 自动压缩和重写内容

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};

/// 重写策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewriteStrategy {
    /// 保留最重要的信息
    PreserveImportant,
    /// 摘要压缩
    Summarize,
    /// 保留最近的信息
    PreserveRecent,
    /// 自定义策略
    Custom(String),
}

/// Auto Rewriter 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRewriterConfig {
    /// 重写策略
    pub strategy: RewriteStrategy,
    /// 目标保留率 (0.0-1.0)
    pub retention_ratio: f32,
    /// 缓冲区比例 (0.0-1.0)
    pub buffer_ratio: f32,
    /// LLM 模型名称
    pub llm_model: String,
    /// LLM 温度
    pub llm_temperature: f32,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for AutoRewriterConfig {
    fn default() -> Self {
        Self {
            strategy: RewriteStrategy::PreserveImportant,
            retention_ratio: 0.8, // 保留 80% 的内容
            buffer_ratio: 0.1,    // 10% 缓冲区
            llm_model: "gpt-4".to_string(),
            llm_temperature: 0.3,
            max_retries: 3,
        }
    }
}

/// Auto Rewriter - 自动重写器
pub struct AutoRewriter {
    config: AutoRewriterConfig,
}

impl AutoRewriter {
    /// 创建新的 Auto Rewriter
    pub fn new(config: AutoRewriterConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(AutoRewriterConfig::default())
    }

    /// 重写内容
    pub async fn rewrite(
        &self,
        content: &str,
        target_length: usize,
        context: Option<&str>,
    ) -> Result<String> {
        // 如果内容已经在目标长度内，直接返回
        if content.len() <= target_length {
            return Ok(content.to_string());
        }

        // 根据策略选择重写方法
        match &self.config.strategy {
            RewriteStrategy::PreserveImportant => {
                self.rewrite_preserve_important(content, target_length)
                    .await
            }
            RewriteStrategy::Summarize => {
                self.rewrite_summarize(content, target_length, context)
                    .await
            }
            RewriteStrategy::PreserveRecent => {
                self.rewrite_preserve_recent(content, target_length).await
            }
            RewriteStrategy::Custom(prompt) => {
                self.rewrite_custom(content, target_length, prompt, context)
                    .await
            }
        }
    }

    /// 保留最重要的信息
    async fn rewrite_preserve_important(
        &self,
        content: &str,
        target_length: usize,
    ) -> Result<String> {
        // 简单实现：按行分割，保留最长的行（假设更长的行包含更多信息）
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(String::new());
        }

        // 按长度排序
        let mut sorted_lines: Vec<&str> = lines.clone();
        sorted_lines.sort_by(|a, b| b.len().cmp(&a.len()));

        // 选择最重要的行
        let mut result = String::new();
        let mut current_length = 0;

        for line in sorted_lines {
            let line_length = line.len() + 1; // +1 for newline
            if current_length + line_length <= target_length {
                if !result.is_empty() {
                    result.push('\n');
                    current_length += 1;
                }
                result.push_str(line);
                current_length += line.len();
            } else {
                break;
            }
        }

        Ok(result)
    }

    /// 摘要压缩（使用 LLM）
    async fn rewrite_summarize(
        &self,
        content: &str,
        target_length: usize,
        context: Option<&str>,
    ) -> Result<String> {
        // 构建 LLM 提示词
        let prompt = self.build_summarize_prompt(content, target_length, context);

        // 调用 LLM（这里是占位符，实际需要集成 LLM 服务）
        self.call_llm(&prompt).await
    }

    /// 保留最近的信息
    async fn rewrite_preserve_recent(&self, content: &str, target_length: usize) -> Result<String> {
        // 简单实现：从末尾截取
        if content.len() <= target_length {
            return Ok(content.to_string());
        }

        // 尝试在单词边界截取
        let truncated = &content[content.len() - target_length..];

        // 找到第一个空格
        if let Some(space_pos) = truncated.find(' ') {
            Ok(truncated[space_pos + 1..].to_string())
        } else {
            Ok(truncated.to_string())
        }
    }

    /// 自定义策略重写
    async fn rewrite_custom(
        &self,
        content: &str,
        target_length: usize,
        custom_prompt: &str,
        context: Option<&str>,
    ) -> Result<String> {
        // 构建自定义提示词
        let prompt = format!(
            "{}\n\nContent to rewrite:\n{}\n\nTarget length: {} characters\n{}",
            custom_prompt,
            content,
            target_length,
            context.map(|c| format!("Context: {c}")).unwrap_or_default()
        );

        // 调用 LLM
        self.call_llm(&prompt).await
    }

    /// 构建摘要提示词
    fn build_summarize_prompt(
        &self,
        content: &str,
        target_length: usize,
        context: Option<&str>,
    ) -> String {
        let context_str = context
            .map(|c| format!("\n\nContext: {c}"))
            .unwrap_or_default();

        format!(
            "Please summarize the following content to approximately {target_length} characters while preserving the most important information.{context_str}\n\nContent:\n{content}"
        )
    }

    /// 调用 LLM（占位符实现）
    async fn call_llm(&self, prompt: &str) -> Result<String> {
        // TODO: 集成实际的 LLM 服务
        // 这里返回一个简单的截断作为占位符

        let target_length = (prompt.len() as f32 * self.config.retention_ratio) as usize;

        if prompt.len() <= target_length {
            return Ok(prompt.to_string());
        }

        // 简单截断
        let truncated = &prompt[..target_length];

        // 尝试在句子边界截取
        if let Some(period_pos) = truncated.rfind('.') {
            Ok(truncated[..period_pos + 1].to_string())
        } else if let Some(space_pos) = truncated.rfind(' ') {
            Ok(truncated[..space_pos].to_string())
        } else {
            Ok(truncated.to_string())
        }
    }

    /// 验证重写结果
    pub fn validate_rewrite(
        &self,
        original: &str,
        rewritten: &str,
        target_length: usize,
    ) -> Result<()> {
        // 检查长度
        if rewritten.len() > target_length {
            return Err(AgentMemError::validation_error(format!(
                "Rewritten content exceeds target length: {} > {}",
                rewritten.len(),
                target_length
            )));
        }

        // 检查是否为空
        if rewritten.is_empty() && !original.is_empty() {
            return Err(AgentMemError::validation_error(
                "Rewritten content is empty",
            ));
        }

        Ok(())
    }

    /// 计算重写质量分数 (0.0-1.0)
    pub fn calculate_quality_score(&self, original: &str, rewritten: &str) -> f32 {
        if original.is_empty() {
            return 1.0;
        }

        // 简单的质量评估：基于长度比例和内容保留
        let length_ratio = rewritten.len() as f32 / original.len() as f32;

        // 计算内容保留率（简单的单词重叠）
        let original_words: std::collections::HashSet<&str> = original.split_whitespace().collect();
        let rewritten_words: std::collections::HashSet<&str> =
            rewritten.split_whitespace().collect();

        let overlap = original_words.intersection(&rewritten_words).count();
        let retention_rate = if !original_words.is_empty() {
            overlap as f32 / original_words.len() as f32
        } else {
            0.0
        };

        // 综合评分
        (length_ratio * 0.3 + retention_rate * 0.7).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rewrite_preserve_important() {
        let rewriter = AutoRewriter::with_default_config();
        let content =
            "Short line\nThis is a much longer line with more information\nMedium line here";
        let target_length = 50;

        let result = rewriter
            .rewrite(content, target_length, None)
            .await
            .unwrap();

        assert!(result.len() <= target_length);
        assert!(result.contains("much longer line"));
    }

    #[tokio::test]
    async fn test_rewrite_preserve_recent() {
        let mut config = AutoRewriterConfig::default();
        config.strategy = RewriteStrategy::PreserveRecent;

        let rewriter = AutoRewriter::new(config);
        let content = "Old information at the start. Recent information at the end.";
        let target_length = 30;

        let result = rewriter
            .rewrite(content, target_length, None)
            .await
            .unwrap();

        assert!(result.len() <= target_length);
        assert!(result.contains("end"));
    }

    #[test]
    fn test_validate_rewrite() {
        let rewriter = AutoRewriter::with_default_config();
        let original = "This is the original content";
        let rewritten = "This is rewritten";
        let target_length = 50;

        let result = rewriter.validate_rewrite(original, rewritten, target_length);
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_quality_score() {
        let rewriter = AutoRewriter::with_default_config();
        let original = "The quick brown fox jumps over the lazy dog";
        let rewritten = "The quick brown fox jumps";

        let score = rewriter.calculate_quality_score(original, rewritten);
        assert!(score > 0.0 && score <= 1.0);
    }
}
