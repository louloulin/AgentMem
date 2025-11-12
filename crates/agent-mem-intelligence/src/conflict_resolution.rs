//! 冲突解决系统
//!
//! 提供智能记忆冲突检测和解决功能，包括：
//! - 语义冲突检测
//! - 时间冲突检测
//! - 智能合并策略
//! - 置信度评估

use crate::similarity::SemanticSimilarity;
use agent_mem_traits::{MemoryV4 as Memory, Message, Result};
use agent_mem_llm::LLMProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

// P0 优化: 超时控制
use crate::timeout::{with_timeout, TimeoutConfig};

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// 语义冲突 - 内容相互矛盾
    Semantic,
    /// 时间冲突 - 时间信息不一致
    Temporal,
    /// 实体冲突 - 同一实体的不同描述
    Entity,
    /// 关系冲突 - 关系信息矛盾
    Relation,
    /// 重复冲突 - 内容重复
    Duplicate,
}

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetection {
    /// 冲突ID
    pub id: String,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 涉及的记忆ID
    pub memory_ids: Vec<String>,
    /// 冲突描述
    pub description: String,
    /// 冲突严重程度 (0.0-1.0)
    pub severity: f32,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 建议的解决方案
    pub suggested_resolution: ResolutionStrategy,
    /// 检测时间
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// 解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// 保留最新的记忆
    KeepLatest,
    /// 保留置信度最高的记忆
    KeepHighestConfidence,
    /// 合并记忆内容
    Merge {
        strategy: MergeStrategy,
        merged_content: String,
    },
    /// 标记为冲突，需要人工处理
    MarkForManualReview,
    /// 删除重复项
    RemoveDuplicates { keep_memory_id: String },
}

/// 合并策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// 简单连接
    Concatenate,
    /// 智能合并
    Intelligent,
    /// 保留关键信息
    KeepKeyInfo,
    /// 时间序列合并
    Temporal,
}

/// 冲突解决结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// 解决的冲突ID
    pub conflict_id: String,
    /// 应用的解决策略
    pub applied_strategy: ResolutionStrategy,
    /// 解决结果
    pub resolution_result: ResolutionResult,
    /// 解决时间
    pub resolved_at: chrono::DateTime<chrono::Utc>,
    /// 解决置信度
    pub resolution_confidence: f32,
}

/// 解决结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionResult {
    /// 成功解决
    Success {
        updated_memories: Vec<String>,
        deleted_memories: Vec<String>,
    },
    /// 需要人工干预
    RequiresManualIntervention { reason: String },
    /// 解决失败
    Failed { error: String },
}

/// 冲突解决器
pub struct ConflictResolver {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    similarity: SemanticSimilarity,
    config: ConflictResolverConfig,
    // P0 优化: 超时配置
    timeout_config: TimeoutConfig,
}

/// 冲突解决器配置
#[derive(Debug, Clone)]
pub struct ConflictResolverConfig {
    /// 语义相似度阈值
    pub semantic_similarity_threshold: f32,
    /// 时间冲突检测窗口（小时）
    pub temporal_conflict_window_hours: i64,
    /// 自动解决阈值
    pub auto_resolution_threshold: f32,
    /// 最大合并记忆数量
    pub max_merge_memories: usize,
    /// P0 优化 #10: 最大考虑记忆数量（防止prompt过长）
    pub max_consideration_memories: usize,
}

impl Default for ConflictResolverConfig {
    fn default() -> Self {
        Self {
            semantic_similarity_threshold: 0.85,
            temporal_conflict_window_hours: 24,
            auto_resolution_threshold: 0.8,
            max_merge_memories: 5,
            // P0 优化 #10: 限制为20个记忆，防止prompt过长
            max_consideration_memories: 20,
        }
    }
}

impl ConflictResolver {
    /// 创建新的冲突解决器
    pub fn new(llm: Arc<dyn LLMProvider + Send + Sync>, config: ConflictResolverConfig) -> Self {
        let similarity = SemanticSimilarity::default();

        Self {
            llm,
            similarity,
            config,
            timeout_config: TimeoutConfig::default(),
        }
    }

    /// 创建带超时配置的冲突解决器
    pub fn with_timeout_config(
        llm: Arc<dyn LLMProvider + Send + Sync>,
        config: ConflictResolverConfig,
        timeout_config: TimeoutConfig,
    ) -> Self {
        let similarity = SemanticSimilarity::default();

        Self {
            llm,
            similarity,
            config,
            timeout_config,
        }
    }

    /// 检测记忆冲突
    pub async fn detect_conflicts(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        info!(
            "开始检测冲突，新记忆: {}, 现有记忆: {}",
            new_memories.len(),
            existing_memories.len()
        );

        // P0 优化 #10: 限制记忆数量，防止prompt过长
        let limited_existing_memories: Vec<Memory> =
            if existing_memories.len() > self.config.max_consideration_memories {
                warn!(
                    "现有记忆数量 {} 超过限制 {}，仅取最相关的前 {} 个",
                    existing_memories.len(),
                    self.config.max_consideration_memories,
                    self.config.max_consideration_memories
                );
                // 取最新的记忆
                let mut sorted_memories = existing_memories.to_vec();
                sorted_memories.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
                sorted_memories[..self.config.max_consideration_memories].to_vec()
            } else {
                existing_memories.to_vec()
            };

        let mut conflicts = Vec::new();

        // 1. 检测语义冲突
        let semantic_conflicts = self
            .detect_semantic_conflicts(new_memories, &limited_existing_memories)
            .await?;
        conflicts.extend(semantic_conflicts);

        // 2. 检测时间冲突
        let temporal_conflicts = self
            .detect_temporal_conflicts(new_memories, &limited_existing_memories)
            .await?;
        conflicts.extend(temporal_conflicts);

        // 3. 检测重复内容
        let duplicate_conflicts = self
            .detect_duplicates(new_memories, &limited_existing_memories)
            .await?;
        conflicts.extend(duplicate_conflicts);

        info!("检测到 {} 个冲突", conflicts.len());
        Ok(conflicts)
    }

    /// 解决记忆冲突
    pub async fn resolve_memory_conflicts(
        &self,
        conflicts: &[ConflictDetection],
        memories: &[Memory],
    ) -> Result<Vec<ConflictResolution>> {
        info!("开始解决 {} 个冲突", conflicts.len());

        let mut resolutions = Vec::new();

        for conflict in conflicts {
            let resolution = self.resolve_single_conflict(conflict, memories).await?;
            resolutions.push(resolution);
        }

        info!("完成冲突解决，生成 {} 个解决方案", resolutions.len());
        Ok(resolutions)
    }

    /// 检测语义冲突
    async fn detect_semantic_conflicts(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        let mut conflicts = Vec::new();

        for new_memory in new_memories {
            for existing_memory in existing_memories {
                // 计算语义相似度
                let new_content_str = match &new_memory.content {
                    agent_mem_traits::Content::Text(t) => t.as_str(),
                    agent_mem_traits::Content::Structured(v) => &v.to_string(),
                    _ => "",
                };
                let existing_content_str = match &existing_memory.content {
                    agent_mem_traits::Content::Text(t) => t.as_str(),
                    agent_mem_traits::Content::Structured(v) => &v.to_string(),
                    _ => "",
                };
                
                let similarity = self
                    .similarity
                    .calculate_similarity(new_content_str, existing_content_str)
                    .await?;

                // 如果相似度高但内容不同，可能存在冲突
                if similarity > self.config.semantic_similarity_threshold {
                    let conflict_severity = self
                        .analyze_semantic_conflict(new_memory, existing_memory)
                        .await?;

                    if conflict_severity > 0.5 {
                        let conflict = ConflictDetection {
                            id: format!("semantic_conflict_{}", conflicts.len()),
                            conflict_type: ConflictType::Semantic,
                            memory_ids: vec![new_memory.id.as_str().to_string(), existing_memory.id.as_str().to_string()],
                            description: format!(
                                "语义冲突：新记忆 '{}' 与现有记忆 '{}' 存在矛盾",
                                new_content_str.chars().take(50).collect::<String>(),
                                existing_content_str.chars().take(50).collect::<String>()
                            ),
                            severity: conflict_severity,
                            confidence: similarity,
                            suggested_resolution: self
                                .suggest_resolution_strategy(
                                    &ConflictType::Semantic,
                                    &[new_memory.clone(), existing_memory.clone()],
                                )
                                .await?,
                            detected_at: chrono::Utc::now(),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 检测时间冲突
    async fn detect_temporal_conflicts(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        let mut conflicts = Vec::new();

        // 简化的时间冲突检测逻辑
        for new_memory in new_memories {
            for existing_memory in existing_memories {
                let time_diff = (new_memory.metadata.created_at - existing_memory.metadata.created_at)
                    .num_hours()
                    .abs();

                if time_diff <= self.config.temporal_conflict_window_hours {
                    let new_content_str = match &new_memory.content {
                        agent_mem_traits::Content::Text(t) => t.as_str(),
                        agent_mem_traits::Content::Structured(v) => &v.to_string(),
                        _ => "",
                    };
                    let existing_content_str = match &existing_memory.content {
                        agent_mem_traits::Content::Text(t) => t.as_str(),
                        agent_mem_traits::Content::Structured(v) => &v.to_string(),
                        _ => "",
                    };
                    
                    // 检查是否存在时间相关的冲突
                    if self.has_temporal_conflict(new_content_str, existing_content_str) {
                        let conflict = ConflictDetection {
                            id: format!("temporal_conflict_{}", conflicts.len()),
                            conflict_type: ConflictType::Temporal,
                            memory_ids: vec![new_memory.id.as_str().to_string(), existing_memory.id.as_str().to_string()],
                            description: "时间冲突：记忆中的时间信息不一致".to_string(),
                            severity: 0.7,
                            confidence: 0.8,
                            suggested_resolution: ResolutionStrategy::KeepLatest,
                            detected_at: chrono::Utc::now(),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 检测重复内容
    async fn detect_duplicates(
        &self,
        new_memories: &[Memory],
        existing_memories: &[Memory],
    ) -> Result<Vec<ConflictDetection>> {
        let mut conflicts = Vec::new();

        for new_memory in new_memories {
            for existing_memory in existing_memories {
                let new_text = match &new_memory.content {
                    agent_mem_traits::Content::Text(t) => t.as_str(),
                    agent_mem_traits::Content::Structured(v) => &v.to_string(),
                    _ => "",
                };
                let existing_text = match &existing_memory.content {
                    agent_mem_traits::Content::Text(t) => t.as_str(),
                    agent_mem_traits::Content::Structured(v) => &v.to_string(),
                    _ => "",
                };
                let similarity = self
                    .similarity
                    .calculate_similarity(new_text, existing_text)
                    .await?;

                // 高相似度且内容长度相近，可能是重复
                if similarity > 0.95 {
                    let new_len = match &new_memory.content {
                        agent_mem_traits::Content::Text(t) => t.len(),
                        agent_mem_traits::Content::Structured(v) => v.to_string().len(),
                        _ => 0,
                    };
                    let existing_len = match &existing_memory.content {
                        agent_mem_traits::Content::Text(t) => t.len(),
                        agent_mem_traits::Content::Structured(v) => v.to_string().len(),
                        _ => 0,
                    };
                    let length_ratio = (new_len as f32) / (existing_len.max(1) as f32);
                    if length_ratio > 0.8 && length_ratio < 1.2 {
                        let conflict = ConflictDetection {
                            id: format!("duplicate_conflict_{}", conflicts.len()),
                            conflict_type: ConflictType::Duplicate,
                            memory_ids: vec![new_memory.id.as_str().to_string(), existing_memory.id.as_str().to_string()],
                            description: "重复内容：发现相似的记忆内容".to_string(),
                            severity: 0.6,
                            confidence: similarity,
                            suggested_resolution: ResolutionStrategy::RemoveDuplicates {
                                keep_memory_id: if new_memory.metadata.created_at
                                    > existing_memory.metadata.created_at
                                {
                                    new_memory.id.as_str().to_string()
                                } else {
                                    existing_memory.id.as_str().to_string()
                                },
                            },
                            detected_at: chrono::Utc::now(),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 分析语义冲突严重程度
    async fn analyze_semantic_conflict(&self, memory1: &Memory, memory2: &Memory) -> Result<f32> {
        let content1_str = match &memory1.content {
            agent_mem_traits::Content::Text(t) => t.clone(),
            agent_mem_traits::Content::Structured(v) => v.to_string(),
            _ => String::new(),
        };
        let content2_str = match &memory2.content {
            agent_mem_traits::Content::Text(t) => t.clone(),
            agent_mem_traits::Content::Structured(v) => v.to_string(),
            _ => String::new(),
        };

        let prompt = format!(
            r#"请分析以下两段记忆内容是否存在语义冲突，并评估冲突严重程度（0.0-1.0）：

记忆1: {}
记忆2: {}

请返回JSON格式：
{{
  "has_conflict": true/false,
  "severity": 0.0-1.0,
  "explanation": "冲突分析说明"
}}

只返回JSON，不要其他解释："#,
            content1_str, content2_str
        );

        // P0 优化: 添加超时控制
        let llm = self.llm.clone();
        let response = with_timeout(
            async move { llm.generate(&[Message::user(&prompt)]).await },
            self.timeout_config.conflict_detection_timeout_secs,
            "conflict_detection_llm",
        )
        .await?;

        // 解析响应
        #[derive(Deserialize)]
        struct ConflictAnalysis {
            has_conflict: bool,
            severity: f32,
            explanation: String,
        }

        // P1 优化 #11: 解析失败时使用规则检测降级
        match serde_json::from_str::<ConflictAnalysis>(&response) {
            Ok(analysis) => {
                if analysis.has_conflict {
                    debug!("✅ LLM检测到语义冲突，严重程度: {}", analysis.severity);
                    Ok(analysis.severity.clamp(0.0, 1.0))
                } else {
                    Ok(0.0)
                }
            }
            Err(e) => {
                warn!("LLM冲突分析失败: {}, 降级到规则检测", e);
                // P1 优化 #11: 使用规则检测降级
                Ok(self.rule_based_conflict_detection(memory1, memory2))
            }
        }
    }

    /// 检查是否存在时间冲突
    fn has_temporal_conflict(&self, content1: &str, content2: &str) -> bool {
        // 简化的时间冲突检测
        // 实际实现中可以使用更复杂的时间实体识别
        let time_keywords = ["昨天", "今天", "明天", "上周", "下周", "去年", "今年"];

        let has_time1 = time_keywords
            .iter()
            .any(|&keyword| content1.contains(keyword));
        let has_time2 = time_keywords
            .iter()
            .any(|&keyword| content2.contains(keyword));

        has_time1 && has_time2
    }

    /// 建议解决策略
    async fn suggest_resolution_strategy(
        &self,
        conflict_type: &ConflictType,
        memories: &[Memory],
    ) -> Result<ResolutionStrategy> {
        match conflict_type {
            ConflictType::Semantic => {
                if memories.len() == 2 {
                    let newer = &memories[0];
                    let older = &memories[1];

                    if newer.metadata.created_at > older.metadata.created_at {
                        Ok(ResolutionStrategy::KeepLatest)
                    } else {
                        Ok(ResolutionStrategy::KeepHighestConfidence)
                    }
                } else {
                    Ok(ResolutionStrategy::MarkForManualReview)
                }
            }
            ConflictType::Temporal => Ok(ResolutionStrategy::KeepLatest),
            ConflictType::Duplicate => {
                if let Some(latest) = memories.iter().max_by_key(|m| m.metadata.created_at) {
                    Ok(ResolutionStrategy::RemoveDuplicates {
                        keep_memory_id: latest.id.as_str().to_string(),
                    })
                } else {
                    Ok(ResolutionStrategy::MarkForManualReview)
                }
            }
            _ => Ok(ResolutionStrategy::MarkForManualReview),
        }
    }

    /// 解决单个冲突
    async fn resolve_single_conflict(
        &self,
        conflict: &ConflictDetection,
        memories: &[Memory],
    ) -> Result<ConflictResolution> {
        let resolution_result = match &conflict.suggested_resolution {
            ResolutionStrategy::KeepLatest => {
                // 找到最新的记忆，删除其他的
                let conflict_memories: Vec<&Memory> = memories
                    .iter()
                    .filter(|m| {
                        let id_str = m.id.as_str().to_string();
                        conflict.memory_ids.contains(&id_str)
                    })
                    .collect();

                if let Some(latest) = conflict_memories.iter().max_by_key(|m| m.metadata.created_at) {
                    let to_delete: Vec<String> = conflict_memories
                        .iter()
                        .filter(|m| m.id.as_str() != latest.id.as_str())
                        .map(|m| m.id.as_str().to_string())
                        .collect();

                    ResolutionResult::Success {
                        updated_memories: vec![latest.id.as_str().to_string()],
                        deleted_memories: to_delete,
                    }
                } else {
                    ResolutionResult::Failed {
                        error: "无法找到最新记忆".to_string(),
                    }
                }
            }
            ResolutionStrategy::RemoveDuplicates { keep_memory_id } => {
                let to_delete: Vec<String> = conflict
                    .memory_ids
                    .iter()
                    .filter(|id| *id != keep_memory_id)
                    .cloned()
                    .collect();

                ResolutionResult::Success {
                    updated_memories: vec![keep_memory_id.clone()],
                    deleted_memories: to_delete,
                }
            }
            _ => ResolutionResult::RequiresManualIntervention {
                reason: "复杂冲突需要人工处理".to_string(),
            },
        };

        Ok(ConflictResolution {
            conflict_id: conflict.id.clone(),
            applied_strategy: conflict.suggested_resolution.clone(),
            resolution_result,
            resolved_at: chrono::Utc::now(),
            resolution_confidence: conflict.confidence,
        })
    }

    /// P1 优化 #11: 基于规则的冲突检测（降级方案）
    ///
    /// 当 LLM 解析失败时使用此方法作为后备方案
    fn rule_based_conflict_detection(&self, memory1: &Memory, memory2: &Memory) -> f32 {
        info!("使用基于规则的冲突检测（降级方案）");

        let content1 = match &memory1.content {
            agent_mem_traits::Content::Text(t) => t.to_lowercase(),
            agent_mem_traits::Content::Structured(v) => v.to_string().to_lowercase(),
            _ => String::new(),
        };
        let content2 = match &memory2.content {
            agent_mem_traits::Content::Text(t) => t.to_lowercase(),
            agent_mem_traits::Content::Structured(v) => v.to_string().to_lowercase(),
            _ => String::new(),
        };

        // 规则1: 检测否定词冲突（"不"、"没有" vs "是"、"有"）
        let has_negation1 =
            content1.contains("不") || content1.contains("没有") || content1.contains("never");
        let has_negation2 =
            content2.contains("不") || content2.contains("没有") || content2.contains("never");

        let has_affirmation1 =
            content1.contains("是") || content1.contains("有") || content1.contains("yes");
        let has_affirmation2 =
            content2.contains("是") || content2.contains("有") || content2.contains("yes");

        // 否定与肯定矛盾
        if (has_negation1 && has_affirmation2) || (has_affirmation1 && has_negation2) {
            debug!("规则检测：否定-肯定冲突");
            return 0.7; // 中等-高冲突
        }

        // 规则2: 检测数字冲突
        let conflict_score = self.detect_number_conflicts(&content1, &content2);
        if conflict_score > 0.0 {
            debug!("规则检测：数字冲突，严重程度: {}", conflict_score);
            return conflict_score;
        }

        // 规则3: 检测时间冲突
        if self.has_temporal_conflict(&content1, &content2) {
            debug!("规则检测：时间冲突");
            return 0.6; // 中等冲突
        }

        // 规则4: 检测反义词冲突
        let antonym_pairs = vec![
            ("喜欢", "讨厌"),
            ("爱", "恨"),
            ("快", "慢"),
            ("大", "小"),
            ("多", "少"),
            ("新", "旧"),
        ];

        for (word1, word2) in antonym_pairs {
            if (content1.contains(word1) && content2.contains(word2))
                || (content1.contains(word2) && content2.contains(word1))
            {
                debug!("规则检测：反义词冲突 ({}-{})", word1, word2);
                return 0.8; // 高冲突
            }
        }

        // 无明显冲突
        debug!("规则检测：无明显冲突");
        0.0
    }

    /// 检测数字冲突
    fn detect_number_conflicts(&self, content1: &str, content2: &str) -> f32 {
        // 简单的数字提取和比较
        let numbers1: Vec<i32> = content1
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        let numbers2: Vec<i32> = content2
            .split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        // 如果两个内容都包含数字且数字不同，可能存在冲突
        if !numbers1.is_empty() && !numbers2.is_empty() {
            let has_different_numbers = numbers1
                .iter()
                .any(|n1| numbers2.iter().any(|n2| (n1 - n2).abs() > 0));

            if has_different_numbers {
                return 0.5; // 中等冲突
            }
        }

        0.0
    }
}
