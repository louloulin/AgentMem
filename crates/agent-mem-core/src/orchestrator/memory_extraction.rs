//! 记忆提取模块 - 从对话中自动提取记忆
//!
//! 参考 MIRIX 的 absorb_content_into_memory 逻辑

use crate::{engine::MemoryEngine, hierarchy::MemoryScope, Memory};
use agent_mem_llm::LLMClient;
use agent_mem_traits::{
    AttributeSet, Content, MemoryId, MemoryType, Message, MetadataV4 as Metadata, RelationGraph,
    Result,
};
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
            min_turns_for_extraction: 1,  // ✅ 修改为1，允许首轮对话就提取记忆
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
        let prompt = self
            .config
            .extraction_prompt
            .replace("{conversation}", &conversation);

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
                        let _memory_type = item
                            .get("type")
                            .and_then(|v| v.as_str())
                            .and_then(|s| self.parse_memory_type(s));

                        let importance = item
                            .get("importance")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.5) as f32;

                        let now = Utc::now();
                        let mut memory = Memory {
                            id: MemoryId::from_string(Uuid::new_v4().to_string()),
                            content: Content::Text(content.to_string()),
                            attributes: AttributeSet::new(),
                            relations: RelationGraph::new(),
                            metadata: Metadata {
                                created_at: now,
                                updated_at: now,
                                accessed_at: now,
                                access_count: 0,
                                version: 1,
                                hash: None,
                            },
                        };
                        memory.set_agent_id(agent_id);
                        memory.set_user_id(user_id);
                        memory.set_importance(importance as f64);
                        memory.set_score(importance as f64);
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
            "resource" => Some(MemoryType::Resource),
            "knowledge" => Some(MemoryType::Knowledge),
            "contextual" => Some(MemoryType::Contextual),
            "factual" => Some(MemoryType::Factual),
            _ => None,
        }
    }

    /// 提取简单记忆（降级方案）
    fn extract_simple_memories(&self, text: &str, agent_id: &str, user_id: &str) -> Vec<Memory> {
        // 简单的基于行的提取
        let now = Utc::now();
        text.lines()
            .filter(|line| !line.trim().is_empty() && line.len() > 10)
            .take(5)
            .map(|line| {
                let mut memory = Memory {
                    id: MemoryId::from_string(Uuid::new_v4().to_string()),
                    content: Content::Text(line.trim().to_string()),
                    attributes: AttributeSet::new(),
                    relations: RelationGraph::new(),
                    metadata: Metadata {
                        created_at: now,
                        updated_at: now,
                        accessed_at: now,
                        access_count: 0,
                        version: 1,
                        hash: None,
                    },
                };
                memory.set_agent_id(agent_id);
                memory.set_user_id(user_id);
                memory.set_importance(0.5);
                memory.set_score(0.5);
                memory
            })
            .collect()
    }

    /// 保存提取的记忆（带去重逻辑）
    ///
    /// 参考 mem0 的去重机制：
    /// 1. 对每条新记忆，搜索现有相似记忆
    /// 2. 如果相似度 > 0.85，跳过（认为是重复）
    /// 3. 否则保存新记忆
    pub async fn save_memories(&self, memories: Vec<Memory>) -> Result<usize> {
        let count = memories.len();

        if count == 0 {
            debug!("No memories to save");
            return Ok(0);
        }

        info!("Saving {} extracted memories (with deduplication)", count);

        let mut saved_count = 0;
        let mut skipped_count = 0;

        for memory in memories {
            // 去重检查：搜索相似记忆
            let is_duplicate = self.check_duplicate(&memory).await?;

            if is_duplicate {
                debug!("Skipping duplicate memory: {:?}", memory.content);
                skipped_count += 1;
                continue;
            }

            // 使用 MemoryEngine 保存记忆
            match self.memory_engine.add_memory(memory.clone()).await {
                Ok(_) => {
                    debug!("Successfully saved memory: {:?}", memory.content);
                    saved_count += 1;
                }
                Err(e) => {
                    warn!("Failed to save memory '{:?}': {}", memory.content, e);
                    // 继续保存其他记忆，不中断整个流程
                }
            }
        }

        info!(
            "Memory save complete: {} saved, {} skipped (duplicates)",
            saved_count, skipped_count
        );
        Ok(saved_count)
    }

    /// 检查记忆是否重复
    ///
    /// 使用向量相似度检测重复记忆
    /// 相似度阈值: 0.85 (参考 mem0)
    async fn check_duplicate(&self, memory: &Memory) -> Result<bool> {
        // 获取记忆的文本内容
        let query = match &memory.content {
            Content::Text(text) => text.clone(),
            _ => {
                // 非文本内容不做去重检查
                return Ok(false);
            }
        };

        // 获取 agent_id 和 user_id 用于构建 scope
        let agent_id = memory.agent_id().map(|s| s.to_string());
        let user_id = memory.user_id().map(|s| s.to_string());

        // 构建 MemoryScope
        let scope = match (agent_id, user_id) {
            (Some(aid), Some(uid)) => Some(MemoryScope::User {
                agent_id: aid,
                user_id: uid,
            }),
            (Some(aid), None) => Some(MemoryScope::Agent(aid)),
            (None, Some(uid)) => Some(MemoryScope::User {
                agent_id: "default".to_string(),
                user_id: uid,
            }),
            (None, None) => None,
        };

        // 搜索相似记忆（限制5条）
        let similar_memories = self
            .memory_engine
            .search_memories(&query, scope, Some(5))
            .await
            .map_err(|e| agent_mem_traits::AgentMemError::storage_error(e.to_string()))?;

        // 检查是否有高度相似的记忆
        const SIMILARITY_THRESHOLD: f64 = 0.85;

        for similar in similar_memories {
            let score = similar.score().unwrap_or(0.0);
            if score >= SIMILARITY_THRESHOLD {
                debug!(
                    "Found duplicate memory with similarity {:.2}: {:?}",
                    score, similar.content
                );
                return Ok(true);
            }
        }

        Ok(false)
    }
}
