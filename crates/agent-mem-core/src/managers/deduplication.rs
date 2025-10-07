//! 记忆去重模块
//!
//! 提供记忆去重和合并功能，包括：
//! - 相似度检测
//! - 重复记忆识别
//! - 智能合并策略
//! - 冲突解决

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// 去重配置
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    /// 相似度阈值（0.0-1.0）
    pub similarity_threshold: f32,
    /// 时间窗口（秒）- 在此窗口内的记忆才会比较
    pub time_window_seconds: i64,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否启用智能合并
    pub enable_intelligent_merge: bool,
    /// 是否保留历史
    pub preserve_history: bool,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            time_window_seconds: 30 * 60, // 30 分钟
            batch_size: 100,
            enable_intelligent_merge: true,
            preserve_history: true,
        }
    }
}

/// 合并策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// 保留最新的
    KeepLatest,
    /// 保留最重要的
    KeepMostImportant,
    /// 保留最完整的
    KeepMostComplete,
    /// 智能合并
    IntelligentMerge,
    /// 用户自定义
    UserDefined,
}

/// 去重结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationResult {
    /// 原始记忆数量
    pub original_count: usize,
    /// 去重后数量
    pub deduplicated_count: usize,
    /// 重复记忆数量
    pub duplicate_count: usize,
    /// 合并的记忆数量
    pub merged_count: usize,
    /// 去重率（百分比）
    pub deduplication_rate: f32,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 被移除的记忆 ID
    pub removed_ids: Vec<String>,
    /// 合并的记忆组数
    pub merged_groups: usize,
}

/// 记忆项（简化版本）
#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// 重复记忆组
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub memories: Vec<MemoryItem>,
    pub similarity_score: f32,
}

/// 记忆去重器
pub struct MemoryDeduplicator {
    config: DeduplicationConfig,
    total_processed: usize,
    total_duplicates_found: usize,
}

impl MemoryDeduplicator {
    /// 创建新的去重器
    pub fn new(config: DeduplicationConfig) -> Self {
        Self {
            config,
            total_processed: 0,
            total_duplicates_found: 0,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(DeduplicationConfig::default())
    }

    /// 去重记忆列表
    pub fn deduplicate(&mut self, memories: Vec<MemoryItem>) -> Result<(DeduplicationResult, Vec<MemoryItem>)> {
        let start_time = std::time::Instant::now();
        let original_count = memories.len();

        info!("开始去重处理，记忆数量: {}", original_count);

        if memories.is_empty() {
            return Ok((
                DeduplicationResult {
                    original_count: 0,
                    deduplicated_count: 0,
                    duplicate_count: 0,
                    merged_count: 0,
                    deduplication_rate: 0.0,
                    processing_time_ms: 0,
                    removed_ids: vec![],
                    merged_groups: 0,
                },
                vec![],
            ));
        }

        // 检测重复记忆组
        let duplicate_groups = self.detect_duplicate_groups(&memories)?;
        info!("发现 {} 个重复记忆组", duplicate_groups.len());

        // 处理重复记忆
        let merged_groups_count = duplicate_groups.len();
        let (processed_memories, removed_ids, merged_count) =
            self.process_duplicates(memories, duplicate_groups)?;

        let deduplicated_count = processed_memories.len();
        let duplicate_count = original_count - deduplicated_count;
        let deduplication_rate = if original_count > 0 {
            (duplicate_count as f32 / original_count as f32) * 100.0
        } else {
            0.0
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        self.total_processed += original_count;
        self.total_duplicates_found += duplicate_count;

        let result = DeduplicationResult {
            original_count,
            deduplicated_count,
            duplicate_count,
            merged_count,
            deduplication_rate,
            processing_time_ms,
            removed_ids,
            merged_groups: merged_groups_count,
        };

        info!(
            "去重完成: 原始 {} -> 去重后 {} (去重率 {:.2}%)",
            original_count, deduplicated_count, deduplication_rate
        );

        Ok((result, processed_memories))
    }

    /// 检测重复记忆组
    fn detect_duplicate_groups(&self, memories: &[MemoryItem]) -> Result<Vec<DuplicateGroup>> {
        let mut duplicate_groups = Vec::new();
        let mut processed = HashMap::new();

        for (i, memory) in memories.iter().enumerate() {
            if processed.contains_key(&memory.id) {
                continue;
            }

            let mut group = vec![memory.clone()];
            processed.insert(memory.id.clone(), true);

            // 查找相似的记忆
            for (j, candidate) in memories.iter().enumerate() {
                if i == j || processed.contains_key(&candidate.id) {
                    continue;
                }

                // 检查时间窗口
                let time_diff = (memory.created_at.timestamp() - candidate.created_at.timestamp()).abs();
                if time_diff > self.config.time_window_seconds {
                    continue;
                }

                // 检查内容相似度
                let similarity = self.calculate_similarity(&memory.content, &candidate.content);
                if similarity >= self.config.similarity_threshold {
                    group.push(candidate.clone());
                    processed.insert(candidate.id.clone(), true);
                }
            }

            // 如果找到重复项，添加到结果中
            if group.len() > 1 {
                let avg_similarity = self.config.similarity_threshold; // 简化实现
                duplicate_groups.push(DuplicateGroup {
                    memories: group,
                    similarity_score: avg_similarity,
                });
            }
        }

        Ok(duplicate_groups)
    }

    /// 处理重复记忆
    fn process_duplicates(
        &self,
        memories: Vec<MemoryItem>,
        duplicate_groups: Vec<DuplicateGroup>,
    ) -> Result<(Vec<MemoryItem>, Vec<String>, usize)> {
        let mut processed_ids = HashMap::new();
        let mut result_memories = Vec::new();
        let mut removed_ids = Vec::new();
        let mut merged_count = 0;

        // 处理重复组
        for group in duplicate_groups {
            let merged = if self.config.enable_intelligent_merge {
                self.intelligent_merge(&group.memories)?
            } else {
                self.keep_latest(&group.memories)?
            };

            result_memories.push(merged.clone());
            processed_ids.insert(merged.id.clone(), true);

            // 记录被移除的记忆
            for memory in &group.memories {
                if memory.id != merged.id {
                    removed_ids.push(memory.id.clone());
                }
            }

            merged_count += group.memories.len() - 1;
        }

        // 添加未处理的记忆
        for memory in memories {
            if !processed_ids.contains_key(&memory.id) && !removed_ids.contains(&memory.id) {
                result_memories.push(memory);
            }
        }

        Ok((result_memories, removed_ids, merged_count))
    }

    /// 计算内容相似度（简化实现）
    fn calculate_similarity(&self, content1: &str, content2: &str) -> f32 {
        // 完全相同
        if content1 == content2 {
            return 1.0;
        }

        // 长度差异太大
        let len_diff = (content1.len() as i32 - content2.len() as i32).abs();
        if len_diff > 100 {
            return 0.0;
        }

        // 简单的字符匹配率
        let min_len = content1.len().min(content2.len());
        if min_len == 0 {
            return 0.0;
        }

        let check_len = min_len.min(100);
        let mut match_count = 0;

        for i in 0..check_len {
            if content1.chars().nth(i) == content2.chars().nth(i) {
                match_count += 1;
            }
        }

        match_count as f32 / check_len as f32
    }

    /// 保留最新的记忆
    fn keep_latest(&self, memories: &[MemoryItem]) -> Result<MemoryItem> {
        memories
            .iter()
            .max_by_key(|m| m.created_at)
            .cloned()
            .ok_or_else(|| AgentMemError::validation_error("Empty memory group"))
    }

    /// 智能合并记忆
    fn intelligent_merge(&self, memories: &[MemoryItem]) -> Result<MemoryItem> {
        if memories.is_empty() {
            return Err(AgentMemError::validation_error("Empty memory group"));
        }

        // 选择最重要的作为基础
        let base = memories
            .iter()
            .max_by(|a, b| a.importance.partial_cmp(&b.importance).unwrap())
            .unwrap();

        // 合并内容（选择最长的）
        let merged_content = memories
            .iter()
            .max_by_key(|m| m.content.len())
            .map(|m| m.content.clone())
            .unwrap_or_else(|| base.content.clone());

        // 合并元数据
        let mut merged_metadata = base.metadata.clone();
        for memory in memories {
            for (key, value) in &memory.metadata {
                merged_metadata.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }

        Ok(MemoryItem {
            id: base.id.clone(),
            content: merged_content,
            importance: base.importance,
            created_at: base.created_at,
            metadata: merged_metadata,
        })
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_processed".to_string(), self.total_processed);
        stats.insert("total_duplicates_found".to_string(), self.total_duplicates_found);
        stats
    }
}

