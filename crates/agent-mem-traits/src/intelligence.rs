//! 智能处理 trait 定义
//!
//! 定义智能记忆处理的接口，用于解耦 agent-mem-core 和 agent-mem-intelligence

use crate::{MemoryItem, Message, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 提取的事实信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    pub content: String,
    pub confidence: f32,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

/// 记忆操作决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecision {
    pub action: MemoryActionType,
    pub confidence: f32,
    pub reasoning: String,
}

/// 记忆操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryActionType {
    Add {
        content: String,
        importance: f32,
        metadata: HashMap<String, String>,
    },
    Update {
        memory_id: String,
        new_content: String,
        merge_strategy: String,
    },
    Delete {
        memory_id: String,
        reason: String,
    },
    Merge {
        primary_memory_id: String,
        secondary_memory_ids: Vec<String>,
        merged_content: String,
    },
    NoAction {
        reason: String,
    },
}

/// 智能记忆处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentProcessingResult {
    pub facts: Vec<ExtractedFact>,
    pub decisions: Vec<MemoryDecision>,
    pub memory_ids: Vec<String>,
}

/// 事实提取器 trait
#[async_trait]
pub trait FactExtractor: Send + Sync {
    /// 从消息中提取结构化事实
    async fn extract_facts(&self, messages: &[Message]) -> Result<Vec<ExtractedFact>>;
}

/// 决策引擎 trait
#[async_trait]
pub trait DecisionEngine: Send + Sync {
    /// 为事实做出记忆操作决策
    async fn decide(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[MemoryItem],
    ) -> Result<MemoryDecision>;
}

/// 智能记忆处理器 trait (组合 FactExtractor 和 DecisionEngine)
#[async_trait]
pub trait IntelligentMemoryProcessor: Send + Sync {
    /// 处理记忆内容，返回处理结果
    async fn process_memory(
        &self,
        content: &str,
        existing_memories: &[MemoryItem],
    ) -> Result<IntelligentProcessingResult>;
}
