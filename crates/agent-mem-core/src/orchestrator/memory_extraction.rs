//! 记忆提取模块 - 从对话中自动提取记忆
//!
//! 参考 MIRIX 的 absorb_content_into_memory 逻辑

use crate::{Memory, engine::MemoryEngine};
use agent_mem_llm::LLMClient;
use agent_mem_traits::{Result, Message, MemoryType};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 记忆提取器配置
#[derive(Debug, Clone)]
pub struct MemoryExtractorConfig {
    /// 是否启用自动提取
    pub auto_extract: bool,
    /// 最小对话轮数触发提取
    pub min_turns_for_extraction: usize,
    /// 提取提示词模板
    pub extraction_prompt: String,
}

impl Default for MemoryExtractorConfig {
    fn default() -> Self {
        Self {
            auto_extract: true,
            min_turns_for_extraction: 3,
            extraction_prompt: r#"Analyze the following conversation and extract important information that should be remembered for future interactions.

Focus on:
1. User preferences and interests
2. Important facts mentioned by the user
3. Goals or tasks the user wants to accomplish
4. Personal information shared by the user
5. Context that would be useful in future conversations

Format each memory as a clear, concise statement.
Return a JSON array of memories, each with:
- "content": the memory text
- "type": one of [episodic, semantic, procedural, working]
- "importance": a score from 0.0 to 1.0

Example:
[
  {"content": "User prefers Python over JavaScript", "type": "semantic", "importance": 0.7},
  {"content": "User is working on a machine learning project", "type": "episodic", "importance": 0.8}
]

Conversation:
{conversation}

Extracted memories (JSON array):"#.to_string(),
        }
    }
}

/// 记忆提取器
pub struct MemoryExtractor {
    llm_client: Arc<LLMClient>,
    memory_engine: Arc<MemoryEngine>,
    config: MemoryExtractorConfig,
}

impl MemoryExtractor {
    /// 创建新的记忆提取器
    pub fn new(
        llm_client: Arc<LLMClient>,
        memory_engine: Arc<MemoryEngine>,
        config: MemoryExtractorConfig,
    ) -> Self {
        Self {
            llm_client,
            memory_engine,
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(
        llm_client: Arc<LLMClient>,
        memory_engine: Arc<MemoryEngine>,
    ) -> Self {
        Self::new(llm_client, memory_engine, MemoryExtractorConfig::default())
    }

    /// 从对话中提取记忆
    ///
    /// 参考 MIRIX 的 absorb_content_into_memory 方法
    pub async fn extract_from_conversation(
        &self,
        messages: &[Message],
        agent_id: &str,
        user_id: &str,
    ) -> Result<Vec<Memory>> {
        if messages.len() < self.config.min_turns_for_extraction {
            debug!("Not enough messages for extraction: {}", messages.len());
            return Ok(Vec::new());
        }

        debug!("Extracting memories from {} messages", messages.len());

        // 构建对话文本
        let conversation = self.format_conversation(messages);

        // 构建提取提示词
        let prompt = self.config.extraction_prompt.replace("{conversation}", &conversation);

        // 调用 LLM 提取记忆
        let extraction_messages = vec![Message::user(&prompt)];

        let response = match self.llm_client.generate(&extraction_messages).await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Failed to extract memories: {}", e);
                return Ok(Vec::new());
            }
        };

        // 解析 LLM 响应
        let memories = self.parse_extraction_response(&response, agent_id, user_id)?;

        info!("Extracted {} memories from conversation", memories.len());
        Ok(memories)
    }

    /// 格式化对话为文本
    fn format_conversation(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    agent_mem_traits::MessageRole::System => "System",
                    agent_mem_traits::MessageRole::User => "User",
                    agent_mem_traits::MessageRole::Assistant => "Assistant",
                };
                format!("{}: {}", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 解析 LLM 提取响应
    fn parse_extraction_response(
        &self,
        response: &str,
        agent_id: &str,
        user_id: &str,
    ) -> Result<Vec<Memory>> {
        // 尝试解析 JSON
        let json_str = self.extract_json_from_response(response);

        match serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
            Ok(items) => {
                let mut memories = Vec::new();
                for item in items {
                    if let Some(content) = item.get("content").and_then(|v| v.as_str()) {
                        let memory_type = item
                            .get("type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| self.parse_memory_type(s));

                        let importance = item
                            .get("importance")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.5) as f32;

                        let memory = Memory::new(
                            agent_id.to_string(),
                            Some(user_id.to_string()),
                            memory_type.unwrap_or(MemoryType::Episodic),
                            content.to_string(),
                            importance,
                        );
                        memories.push(memory);
                    }
                }
                Ok(memories)
            }
            Err(e) => {
                warn!("Failed to parse extraction response as JSON: {}", e);
                // 降级：从文本中提取简单记忆
                Ok(self.extract_simple_memories(response, agent_id, user_id))
            }
        }
    }

    /// 从响应中提取 JSON
    fn extract_json_from_response(&self, response: &str) -> String {
        // 尝试找到 JSON 数组
        if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                return response[start..=end].to_string();
            }
        }
        response.to_string()
    }

    /// 解析记忆类型
    fn parse_memory_type(&self, type_str: &str) -> Option<MemoryType> {
        match type_str.to_lowercase().as_str() {
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            "working" => Some(MemoryType::Working),
            "core" => Some(MemoryType::Core),
            "declarative" => Some(MemoryType::Declarative),
            "implicit" => Some(MemoryType::Implicit),
            "explicit" => Some(MemoryType::Explicit),
            "autobiographical" => Some(MemoryType::Autobiographical),
            _ => None,
        }
    }

    /// 提取简单记忆（降级方案）
    fn extract_simple_memories(&self, text: &str, agent_id: &str, user_id: &str) -> Vec<Memory> {
        // 简单的基于行的提取
        text.lines()
            .filter(|line| !line.trim().is_empty() && line.len() > 10)
            .take(5) // 最多提取 5 条
            .map(|line| {
                Memory::new(
                    agent_id.to_string(),
                    Some(user_id.to_string()),
                    MemoryType::Episodic,
                    line.trim().to_string(),
                    0.5,
                )
            })
            .collect()
    }

    /// 保存提取的记忆
    pub async fn save_memories(&self, memories: Vec<Memory>) -> Result<usize> {
        let count = memories.len();
        for memory in memories {
            // TODO: 使用 MemoryEngine 保存记忆
            // self.memory_engine.add_memory(memory).await?;
            debug!("Would save memory: {}", memory.content);
        }
        Ok(count)
    }
}

