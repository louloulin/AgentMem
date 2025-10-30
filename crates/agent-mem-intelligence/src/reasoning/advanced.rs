//! 高级推理功能模块
//!
//! 实现高级推理能力，包括：
//! - 多跳因果推理（Multi-hop Causal Reasoning）
//! - 反事实推理（Counterfactual Reasoning）
//! - 类比推理（Analogical Reasoning）

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use super::MemoryData;

/// 多跳因果推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiHopCausalResult {
    /// 推理链（从原因到结果的路径）
    pub reasoning_chain: Vec<CausalStep>,
    /// 总体置信度
    pub overall_confidence: f32,
    /// 推理深度（跳数）
    pub depth: usize,
    /// 推理解释
    pub explanation: String,
    /// 推理时间
    pub reasoned_at: DateTime<Utc>,
}

/// 因果推理步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalStep {
    /// 原因记忆ID
    pub cause_id: String,
    /// 结果记忆ID
    pub effect_id: String,
    /// 因果关系类型
    pub relation_type: CausalRelationType,
    /// 步骤置信度
    pub confidence: f32,
    /// 证据
    pub evidence: Vec<String>,
}

/// 因果关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CausalRelationType {
    /// 直接因果（A 直接导致 B）
    Direct,
    /// 间接因果（A 通过中间步骤导致 B）
    Indirect,
    /// 必要条件（A 是 B 的必要条件）
    Necessary,
    /// 充分条件（A 是 B 的充分条件）
    Sufficient,
    /// 促进因素（A 促进 B 发生）
    Facilitating,
    /// 抑制因素（A 抑制 B 发生）
    Inhibiting,
}

/// 反事实推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    /// 原始情况
    pub original_scenario: String,
    /// 反事实假设
    pub counterfactual_hypothesis: String,
    /// 预测结果
    pub predicted_outcome: String,
    /// 置信度
    pub confidence: f32,
    /// 影响的记忆ID列表
    pub affected_memory_ids: Vec<String>,
    /// 推理解释
    pub explanation: String,
    /// 推理时间
    pub reasoned_at: DateTime<Utc>,
}

/// 类比推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogyResult {
    /// 源领域（类比的来源）
    pub source_domain: AnalogicalDomain,
    /// 目标领域（类比的目标）
    pub target_domain: AnalogicalDomain,
    /// 映射关系
    pub mappings: Vec<AnalogicalMapping>,
    /// 类比强度
    pub analogy_strength: f32,
    /// 推理结论
    pub conclusion: String,
    /// 推理时间
    pub reasoned_at: DateTime<Utc>,
}

/// 类比领域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogicalDomain {
    /// 领域名称
    pub name: String,
    /// 涉及的记忆ID
    pub memory_ids: Vec<String>,
    /// 领域特征
    pub features: HashMap<String, String>,
    /// 领域关系
    pub relations: Vec<(String, String, String)>, // (from, relation, to)
}

/// 类比映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogicalMapping {
    /// 源元素
    pub source_element: String,
    /// 目标元素
    pub target_element: String,
    /// 映射类型
    pub mapping_type: MappingType,
    /// 映射置信度
    pub confidence: f32,
}

/// 映射类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MappingType {
    /// 实体映射
    Entity,
    /// 关系映射
    Relation,
    /// 属性映射
    Attribute,
    /// 结构映射
    Structural,
}

/// 高级推理引擎
pub struct AdvancedReasoner {
    /// 最大推理深度
    max_depth: usize,
    /// 最小置信度阈值
    min_confidence: f32,
    /// 最大推理链长度
    max_chain_length: usize,
}

impl AdvancedReasoner {
    /// 创建新的高级推理引擎
    pub fn new(max_depth: usize, min_confidence: f32, max_chain_length: usize) -> Self {
        Self {
            max_depth,
            min_confidence,
            max_chain_length,
        }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(5, 0.6, 10)
    }

    /// 多跳因果推理
    ///
    /// 从起始记忆推理到目标记忆，找出因果链
    pub fn multi_hop_causal_reasoning(
        &self,
        start_memory: &MemoryData,
        target_memory: &MemoryData,
        all_memories: &[MemoryData],
    ) -> Result<Vec<MultiHopCausalResult>> {
        let mut results = Vec::new();

        // 使用广度优先搜索找出所有可能的因果链
        let chains = self.find_causal_chains(start_memory, target_memory, all_memories)?;

        for chain in chains {
            if chain.len() > 1 && chain.len() <= self.max_chain_length {
                let reasoning_chain = self.build_causal_steps(&chain, all_memories)?;
                let overall_confidence = self.calculate_chain_confidence(&reasoning_chain);

                if overall_confidence >= self.min_confidence {
                    let explanation = self.generate_causal_explanation(&reasoning_chain);

                    results.push(MultiHopCausalResult {
                        reasoning_chain,
                        overall_confidence,
                        depth: chain.len() - 1,
                        explanation,
                        reasoned_at: Utc::now(),
                    });
                }
            }
        }

        // 按置信度排序
        results.sort_by(|a, b| {
            b.overall_confidence
                .partial_cmp(&a.overall_confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// 反事实推理
    ///
    /// 假设某个记忆不存在或改变，推理会产生什么影响
    pub fn counterfactual_reasoning(
        &self,
        target_memory: &MemoryData,
        hypothesis: &str,
        all_memories: &[MemoryData],
    ) -> Result<CounterfactualResult> {
        // 找出依赖于目标记忆的其他记忆
        let affected_memories = self.find_dependent_memories(target_memory, all_memories)?;

        // 分析假设情况下的影响
        let predicted_outcome = self.predict_counterfactual_outcome(
            target_memory,
            hypothesis,
            &affected_memories,
            all_memories,
        )?;

        // 计算置信度
        let confidence = self.calculate_counterfactual_confidence(
            target_memory,
            &affected_memories,
            all_memories,
        );

        // 生成解释
        let explanation = self.generate_counterfactual_explanation(
            target_memory,
            hypothesis,
            &affected_memories,
            &predicted_outcome,
        );

        Ok(CounterfactualResult {
            original_scenario: target_memory.content.clone(),
            counterfactual_hypothesis: hypothesis.to_string(),
            predicted_outcome,
            confidence,
            affected_memory_ids: affected_memories.iter().map(|m| m.id.clone()).collect(),
            explanation,
            reasoned_at: Utc::now(),
        })
    }

    /// 类比推理
    ///
    /// 在两个领域之间建立类比关系
    pub fn analogical_reasoning(
        &self,
        source_memories: &[MemoryData],
        target_memories: &[MemoryData],
    ) -> Result<AnalogyResult> {
        // 提取源领域和目标领域的结构
        let source_domain = self.extract_domain_structure("source", source_memories)?;
        let target_domain = self.extract_domain_structure("target", target_memories)?;

        // 建立映射关系
        let mappings = self.find_analogical_mappings(&source_domain, &target_domain)?;

        // 计算类比强度
        let analogy_strength = self.calculate_analogy_strength(&mappings, &source_domain, &target_domain);

        // 生成结论
        let conclusion = self.generate_analogy_conclusion(&source_domain, &target_domain, &mappings);

        Ok(AnalogyResult {
            source_domain,
            target_domain,
            mappings,
            analogy_strength,
            conclusion,
            reasoned_at: Utc::now(),
        })
    }

    /// 找出因果链
    fn find_causal_chains(
        &self,
        start: &MemoryData,
        target: &MemoryData,
        all_memories: &[MemoryData],
    ) -> Result<Vec<Vec<String>>> {
        let mut chains = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        // 初始化队列
        queue.push_back(vec![start.id.clone()]);

        while let Some(current_chain) = queue.pop_front() {
            let current_id = current_chain.last().unwrap();

            // 如果到达目标，保存链
            if current_id == &target.id {
                chains.push(current_chain.clone());
                continue;
            }

            // 如果链太长，跳过
            if current_chain.len() >= self.max_depth {
                continue;
            }

            // 标记为已访问
            visited.insert(current_id.clone());

            // 找出可能的下一步
            let next_memories = self.find_causally_related_memories(current_id, all_memories)?;

            for next_memory in next_memories {
                if !visited.contains(&next_memory.id) && !current_chain.contains(&next_memory.id) {
                    let mut new_chain = current_chain.clone();
                    new_chain.push(next_memory.id.clone());
                    queue.push_back(new_chain);
                }
            }
        }

        Ok(chains)
    }

    /// 找出因果相关的记忆
    fn find_causally_related_memories(
        &self,
        memory_id: &str,
        all_memories: &[MemoryData],
    ) -> Result<Vec<MemoryData>> {
        let current_memory = all_memories
            .iter()
            .find(|m| m.id == memory_id)
            .ok_or_else(|| AgentMemError::not_found(format!("Memory {memory_id} not found")))?;

        let mut related = Vec::new();

        for memory in all_memories {
            if memory.id == memory_id {
                continue;
            }

            // 时间顺序：原因必须在结果之前
            if memory.created_at > current_memory.created_at {
                // 检查内容相关性
                if self.has_causal_relationship(current_memory, memory)? {
                    related.push(memory.clone());
                }
            }
        }

        Ok(related)
    }

    /// 检查是否存在因果关系
    fn has_causal_relationship(&self, cause: &MemoryData, effect: &MemoryData) -> Result<bool> {
        // 基于内容相似度和时间关系判断
        if let (Some(cause_emb), Some(effect_emb)) = (&cause.embedding, &effect.embedding) {
            let similarity = self.cosine_similarity(cause_emb, effect_emb)?;
            Ok(similarity > 0.5) // 简化的判断标准
        } else {
            // 基于关键词重叠
            let cause_keywords = self.extract_keywords(&cause.content);
            let effect_keywords = self.extract_keywords(&effect.content);
            let overlap = self.calculate_keyword_overlap(&cause_keywords, &effect_keywords);
            Ok(overlap > 0.3)
        }
    }

    /// 构建因果步骤
    fn build_causal_steps(
        &self,
        chain: &[String],
        all_memories: &[MemoryData],
    ) -> Result<Vec<CausalStep>> {
        let mut steps = Vec::new();

        for i in 0..chain.len() - 1 {
            let cause_id = &chain[i];
            let effect_id = &chain[i + 1];

            let cause = all_memories
                .iter()
                .find(|m| &m.id == cause_id)
                .ok_or_else(|| AgentMemError::not_found(format!("Memory {cause_id} not found")))?;

            let effect = all_memories
                .iter()
                .find(|m| &m.id == effect_id)
                .ok_or_else(|| AgentMemError::not_found(format!("Memory {effect_id} not found")))?;

            let confidence = self.calculate_causal_confidence(cause, effect)?;
            let relation_type = self.determine_causal_type(cause, effect)?;
            let evidence = self.collect_causal_evidence(cause, effect)?;

            steps.push(CausalStep {
                cause_id: cause_id.clone(),
                effect_id: effect_id.clone(),
                relation_type,
                confidence,
                evidence,
            });
        }

        Ok(steps)
    }

    /// 计算因果置信度
    fn calculate_causal_confidence(&self, cause: &MemoryData, effect: &MemoryData) -> Result<f32> {
        let mut confidence = 0.0;

        // 时间因素（时间越近，置信度越高）
        let time_diff = effect.created_at.signed_duration_since(cause.created_at);
        let time_hours = time_diff.num_hours() as f32;
        let time_score = if time_hours > 0.0 {
            1.0 / (1.0 + (time_hours / 24.0).ln())
        } else {
            0.0
        };
        confidence += time_score * 0.3;

        // 内容相似度因素
        if let (Some(cause_emb), Some(effect_emb)) = (&cause.embedding, &effect.embedding) {
            let similarity = self.cosine_similarity(cause_emb, effect_emb)?;
            confidence += similarity * 0.7;
        } else {
            let cause_keywords = self.extract_keywords(&cause.content);
            let effect_keywords = self.extract_keywords(&effect.content);
            let overlap = self.calculate_keyword_overlap(&cause_keywords, &effect_keywords);
            confidence += overlap * 0.7;
        }

        Ok(confidence.min(1.0))
    }

    /// 确定因果类型
    fn determine_causal_type(&self, cause: &MemoryData, effect: &MemoryData) -> Result<CausalRelationType> {
        // 简化的因果类型判断
        let time_diff = effect.created_at.signed_duration_since(cause.created_at);
        let hours = time_diff.num_hours();

        if hours <= 1 {
            Ok(CausalRelationType::Direct)
        } else if hours <= 24 {
            Ok(CausalRelationType::Indirect)
        } else {
            Ok(CausalRelationType::Facilitating)
        }
    }

    /// 收集因果证据
    fn collect_causal_evidence(&self, cause: &MemoryData, effect: &MemoryData) -> Result<Vec<String>> {
        let mut evidence = Vec::new();

        // 时间证据
        let time_diff = effect.created_at.signed_duration_since(cause.created_at);
        evidence.push(format!(
            "时间顺序: 原因发生在结果之前 ({} 小时)",
            time_diff.num_hours()
        ));

        // 内容证据
        let cause_keywords = self.extract_keywords(&cause.content);
        let effect_keywords = self.extract_keywords(&effect.content);
        let shared: Vec<_> = cause_keywords.intersection(&effect_keywords).collect();
        if !shared.is_empty() {
            evidence.push(format!(
                "共享关键词: {}",
                shared.iter().take(3).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }

        Ok(evidence)
    }

    /// 计算推理链置信度
    fn calculate_chain_confidence(&self, chain: &[CausalStep]) -> f32 {
        if chain.is_empty() {
            return 0.0;
        }

        // 使用几何平均数
        let product: f32 = chain.iter().map(|step| step.confidence).product();
        product.powf(1.0 / chain.len() as f32)
    }

    /// 生成因果解释
    fn generate_causal_explanation(&self, chain: &[CausalStep]) -> String {
        if chain.is_empty() {
            return "无因果链".to_string();
        }

        let mut explanation = format!("发现 {} 步因果推理链:\n", chain.len());

        for (i, step) in chain.iter().enumerate() {
            explanation.push_str(&format!(
                "  步骤 {}: {} → {} (类型: {:?}, 置信度: {:.2})\n",
                i + 1,
                step.cause_id,
                step.effect_id,
                step.relation_type,
                step.confidence
            ));
        }

        explanation
    }

    /// 找出依赖记忆
    fn find_dependent_memories(
        &self,
        target: &MemoryData,
        all_memories: &[MemoryData],
    ) -> Result<Vec<MemoryData>> {
        let mut dependent = Vec::new();

        for memory in all_memories {
            if memory.id == target.id {
                continue;
            }

            // 如果记忆在目标之后创建，且内容相关，则可能依赖于目标
            if memory.created_at > target.created_at
                && self.has_causal_relationship(target, memory)? {
                    dependent.push(memory.clone());
                }
        }

        Ok(dependent)
    }

    /// 预测反事实结果
    fn predict_counterfactual_outcome(
        &self,
        target: &MemoryData,
        hypothesis: &str,
        affected: &[MemoryData],
        _all_memories: &[MemoryData],
    ) -> Result<String> {
        let mut outcome = format!("如果 '{}' 变为 '{}'，", target.content, hypothesis);

        if affected.is_empty() {
            outcome.push_str("则不会影响其他记忆。");
        } else {
            outcome.push_str(&format!(
                "则会影响 {} 个相关记忆: ",
                affected.len()
            ));

            let affected_ids: Vec<_> = affected.iter().take(3).map(|m| m.id.as_str()).collect();
            outcome.push_str(&affected_ids.join(", "));

            if affected.len() > 3 {
                outcome.push_str(&format!(" 等 {} 个记忆", affected.len()));
            }
        }

        Ok(outcome)
    }

    /// 计算反事实置信度
    fn calculate_counterfactual_confidence(
        &self,
        _target: &MemoryData,
        affected: &[MemoryData],
        _all_memories: &[MemoryData],
    ) -> f32 {
        // 基于影响范围计算置信度
        let base_confidence = 0.7;
        let impact_factor = (affected.len() as f32 / 10.0).min(1.0);
        base_confidence * (1.0 - impact_factor * 0.3)
    }

    /// 生成反事实解释
    fn generate_counterfactual_explanation(
        &self,
        target: &MemoryData,
        hypothesis: &str,
        affected: &[MemoryData],
        outcome: &str,
    ) -> String {
        format!(
            "反事实推理分析:\n\
             原始情况: {}\n\
             假设: {}\n\
             影响范围: {} 个记忆\n\
             预测结果: {}",
            target.content,
            hypothesis,
            affected.len(),
            outcome
        )
    }

    /// 提取领域结构
    fn extract_domain_structure(
        &self,
        name: &str,
        memories: &[MemoryData],
    ) -> Result<AnalogicalDomain> {
        let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();

        // 提取特征
        let mut features = HashMap::new();
        for (i, memory) in memories.iter().enumerate() {
            let keywords = self.extract_keywords(&memory.content);
            features.insert(
                format!("entity_{i}"),
                keywords.iter().take(3).cloned().collect::<Vec<_>>().join(", "),
            );
        }

        // 提取关系
        let mut relations = Vec::new();
        for i in 0..memories.len() {
            for j in i + 1..memories.len() {
                if self.has_causal_relationship(&memories[i], &memories[j])? {
                    relations.push((
                        format!("entity_{i}"),
                        "related_to".to_string(),
                        format!("entity_{j}"),
                    ));
                }
            }
        }

        Ok(AnalogicalDomain {
            name: name.to_string(),
            memory_ids,
            features,
            relations,
        })
    }

    /// 找出类比映射
    fn find_analogical_mappings(
        &self,
        source: &AnalogicalDomain,
        target: &AnalogicalDomain,
    ) -> Result<Vec<AnalogicalMapping>> {
        let mut mappings = Vec::new();

        // 实体映射（基于特征相似度）
        for (source_entity, source_features) in &source.features {
            for (target_entity, target_features) in &target.features {
                let similarity = self.calculate_string_similarity(source_features, target_features);

                if similarity > 0.5 {
                    mappings.push(AnalogicalMapping {
                        source_element: source_entity.clone(),
                        target_element: target_entity.clone(),
                        mapping_type: MappingType::Entity,
                        confidence: similarity,
                    });
                }
            }
        }

        // 关系映射
        for source_rel in &source.relations {
            for target_rel in &target.relations {
                if source_rel.1 == target_rel.1 {
                    // 相同类型的关系
                    mappings.push(AnalogicalMapping {
                        source_element: format!("{}-{}-{}", source_rel.0, source_rel.1, source_rel.2),
                        target_element: format!("{}-{}-{}", target_rel.0, target_rel.1, target_rel.2),
                        mapping_type: MappingType::Relation,
                        confidence: 0.8,
                    });
                }
            }
        }

        Ok(mappings)
    }

    /// 计算类比强度
    fn calculate_analogy_strength(
        &self,
        mappings: &[AnalogicalMapping],
        source: &AnalogicalDomain,
        target: &AnalogicalDomain,
    ) -> f32 {
        if mappings.is_empty() {
            return 0.0;
        }

        // 映射覆盖率
        let source_size = source.features.len() + source.relations.len();
        let target_size = target.features.len() + target.relations.len();
        let coverage = mappings.len() as f32 / source_size.max(target_size) as f32;

        // 平均映射置信度
        let avg_confidence: f32 = mappings.iter().map(|m| m.confidence).sum::<f32>() / mappings.len() as f32;

        // 综合强度
        (coverage * 0.4 + avg_confidence * 0.6).min(1.0)
    }

    /// 生成类比结论
    fn generate_analogy_conclusion(
        &self,
        source: &AnalogicalDomain,
        target: &AnalogicalDomain,
        mappings: &[AnalogicalMapping],
    ) -> String {
        format!(
            "在 '{}' 和 '{}' 之间发现类比关系:\n\
             - 建立了 {} 个映射\n\
             - 源领域包含 {} 个实体和 {} 个关系\n\
             - 目标领域包含 {} 个实体和 {} 个关系\n\
             - 类比可用于从源领域推理到目标领域",
            source.name,
            target.name,
            mappings.len(),
            source.features.len(),
            source.relations.len(),
            target.features.len(),
            target.relations.len()
        )
    }

    /// 辅助方法：余弦相似度
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error("Vector dimensions must match"));
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// 辅助方法：提取关键词
    fn extract_keywords(&self, content: &str) -> HashSet<String> {
        content
            .to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// 辅助方法：计算关键词重叠度
    fn calculate_keyword_overlap(&self, keywords1: &HashSet<String>, keywords2: &HashSet<String>) -> f32 {
        let intersection_size = keywords1.intersection(keywords2).count();
        let union_size = keywords1.union(keywords2).count();

        if union_size == 0 {
            0.0
        } else {
            intersection_size as f32 / union_size as f32
        }
    }

    /// 辅助方法：计算字符串相似度
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f32 {
        let words1: HashSet<String> = s1.split(',').map(|s| s.trim().to_string()).collect();
        let words2: HashSet<String> = s2.split(',').map(|s| s.trim().to_string()).collect();
        self.calculate_keyword_overlap(&words1, &words2)
    }
}

