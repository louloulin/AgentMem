//! 记忆提取模块 - 从对话中自动提取记忆

use crate::{Memory, CoreResult};
use tracing::{debug, info};

/// 记忆提取器
pub struct MemoryExtractor {
    // TODO: 添加字段
}

impl MemoryExtractor {
    pub fn new() -> Self {
        Self {}
    }

    /// 从对话中提取记忆
    pub async fn extract_from_conversation(
        &self,
        messages: &[String], // TODO: 使用正确的消息类型
        agent_id: &str,
    ) -> CoreResult<Vec<Memory>> {
        // TODO: 使用 LLM 提取记忆
        debug!("Extracting memories from {} messages", messages.len());
        Ok(Vec::new())
    }

    /// 分类记忆类型
    pub fn classify_memory_type(&self, content: &str) -> String {
        // TODO: 实现记忆分类
        "episodic".to_string()
    }
}

