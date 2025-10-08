//! Trait implementations for intelligent memory processing
//!
//! 实现 agent-mem-traits 中定义的智能处理 trait

use agent_mem_traits::{
    DecisionEngine as DecisionEngineTrait, ExtractedFact, FactExtractor as FactExtractorTrait,
    MemoryActionType, MemoryDecision, MemoryItem, Message, Result,
};
use async_trait::async_trait;

use crate::{
    decision_engine::MemoryDecisionEngine,
    fact_extraction::FactExtractor,
};

/// FactExtractor trait 实现
#[async_trait]
impl FactExtractorTrait for FactExtractor {
    async fn extract_facts(&self, messages: &[Message]) -> Result<Vec<ExtractedFact>> {
        // 调用 FactExtractor 的内部实现
        let internal_facts = self.extract_facts_internal(messages).await?;

        // 转换为 trait 定义的 ExtractedFact 格式
        let converted_facts: Vec<ExtractedFact> = internal_facts.into_iter()
            .map(|fact| ExtractedFact {
                content: fact.content,
                confidence: fact.confidence,
                category: fact.category,
                metadata: fact.metadata,
            })
            .collect();

        Ok(converted_facts)
    }
}

/// DecisionEngine trait 实现
#[async_trait]
impl DecisionEngineTrait for MemoryDecisionEngine {
    async fn decide(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[MemoryItem],
    ) -> Result<MemoryDecision> {
        // 转换 MemoryItem 为 ExistingMemory
        let existing: Vec<crate::ExistingMemory> = existing_memories.iter()
            .map(|item| crate::ExistingMemory {
                id: item.id.clone(),
                content: item.content.clone(),
                importance: item.importance,
                created_at: item.created_at.to_rfc3339(),
                updated_at: item.updated_at.map(|dt| dt.to_rfc3339()),
                metadata: item.metadata.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
            })
            .collect();
        
        // 调用现有的 decide 方法
        let decision = self.decide(&fact.content, &existing).await?;
        
        // 转换决策结果
        let action = match decision.action {
            crate::decision_engine::MemoryAction::Add { content, importance, metadata } => {
                MemoryActionType::Add {
                    content,
                    importance,
                    metadata,
                }
            },
            crate::decision_engine::MemoryAction::Update { memory_id, new_content, merge_strategy, .. } => {
                MemoryActionType::Update {
                    memory_id,
                    new_content,
                    merge_strategy: format!("{:?}", merge_strategy).to_lowercase(),
                }
            },
            crate::decision_engine::MemoryAction::Delete { memory_id, deletion_reason } => {
                MemoryActionType::Delete {
                    memory_id,
                    reason: format!("{:?}", deletion_reason),
                }
            },
            crate::decision_engine::MemoryAction::Merge { primary_memory_id, secondary_memory_ids, merged_content } => {
                MemoryActionType::Merge {
                    primary_memory_id,
                    secondary_memory_ids,
                    merged_content,
                }
            },
            crate::decision_engine::MemoryAction::NoAction { reason } => {
                MemoryActionType::NoAction { reason }
            },
        };
        
        Ok(MemoryDecision {
            action,
            confidence: decision.confidence,
            reasoning: decision.reasoning,
        })
    }
}

