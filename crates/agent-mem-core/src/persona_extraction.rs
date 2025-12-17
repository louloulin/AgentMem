//! Persona Dynamic Extraction Module
//!
//! Phase 2.4: Persona动态提取
//! - Persona提取引擎
//! - 动态更新机制
//! - Persona检索优化
//!
//! 参考Mem0和O-Mem的Persona提取策略，提升个性化准确率15-20%

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};

/// Persona提取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaExtractionConfig {
    /// 启用Persona提取
    pub enable_extraction: bool,
    /// 启用动态更新
    pub enable_dynamic_update: bool,
    /// Persona更新阈值（变化超过此值才更新）
    pub update_threshold: f64,
    /// 提取的Persona属性数量
    pub max_persona_attributes: usize,
    /// 启用Persona检索优化
    pub enable_retrieval_optimization: bool,
    /// Persona权重（在检索中的权重）
    pub persona_weight: f64,
    /// 更新频率（小时）
    pub update_frequency_hours: u64,
}

impl Default for PersonaExtractionConfig {
    fn default() -> Self {
        Self {
            enable_extraction: true,
            enable_dynamic_update: true,
            update_threshold: 0.1, // 10%变化
            max_persona_attributes: 20,
            enable_retrieval_optimization: true,
            persona_weight: 0.3, // 30%权重
            update_frequency_hours: 24, // 24小时更新一次
        }
    }
}

/// Persona属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaAttribute {
    /// 属性名称
    pub name: String,
    /// 属性值
    pub value: String,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 来源（提取自哪条记忆）
    pub source_memory_id: Option<String>,
    /// 提取时间
    pub extracted_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 更新次数
    pub update_count: u64,
}

/// Persona档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaProfile {
    /// 用户ID
    pub user_id: String,
    /// Persona属性
    pub attributes: HashMap<String, PersonaAttribute>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 版本号
    pub version: u64,
    /// 统计信息
    pub stats: PersonaStats,
}

/// Persona统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaStats {
    /// 总属性数
    pub total_attributes: usize,
    /// 高置信度属性数
    pub high_confidence_attributes: usize,
    /// 更新次数
    pub update_count: u64,
    /// 最后提取时间
    pub last_extraction_at: Option<DateTime<Utc>>,
}

impl Default for PersonaStats {
    fn default() -> Self {
        Self {
            total_attributes: 0,
            high_confidence_attributes: 0,
            update_count: 0,
            last_extraction_at: None,
        }
    }
}

/// Persona提取引擎
pub struct PersonaExtractionEngine {
    config: PersonaExtractionConfig,
    /// Persona档案存储 (user_id -> PersonaProfile)
    persona_profiles: Arc<RwLock<HashMap<String, PersonaProfile>>>,
    /// 提取历史（用于跟踪变化）
    extraction_history: Arc<RwLock<HashMap<String, Vec<PersonaProfile>>>>,
}

impl PersonaExtractionEngine {
    /// 创建新的Persona提取引擎
    pub fn new(config: PersonaExtractionConfig) -> Self {
        Self {
            config,
            persona_profiles: Arc::new(RwLock::new(HashMap::new())),
            extraction_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(PersonaExtractionConfig::default())
    }

    /// 从记忆内容中提取Persona
    ///
    /// 分析用户的记忆内容，提取Persona属性（偏好、习惯、特征等）
    pub async fn extract_persona_from_memories(
        &self,
        user_id: &str,
        memories: Vec<MemoryContent>,
    ) -> Result<PersonaProfile> {
        if !self.config.enable_extraction {
            return self.get_or_create_persona(user_id).await;
        }

        info!("提取Persona: user_id={}, memories={}", user_id, memories.len());

        // 1. 提取Persona属性
        let mut attributes = HashMap::new();

        for memory in &memories {
            let extracted = self.extract_attributes_from_memory(memory).await?;
            for attr in extracted {
                // 合并或更新属性
                self.merge_attribute(&mut attributes, attr);
            }
        }

        // 2. 过滤和排序属性
        let filtered_attributes = self.filter_and_rank_attributes(attributes).await?;

        // 3. 创建或更新Persona档案
        let mut profile = self.get_or_create_persona(user_id).await?;
        let old_version = profile.version;

        // 4. 检查是否需要更新
        if self.should_update_persona(&profile, &filtered_attributes).await? {
            profile.attributes = filtered_attributes;
            profile.version += 1;
            profile.updated_at = Utc::now();
            profile.stats.total_attributes = profile.attributes.len();
            profile.stats.high_confidence_attributes = profile
                .attributes
                .values()
                .filter(|a| a.confidence >= 0.7)
                .count();
            profile.stats.update_count += 1;
            profile.stats.last_extraction_at = Some(Utc::now());

            // 保存历史
            self.save_extraction_history(user_id, &profile).await;

            info!(
                "Persona已更新: user_id={}, version={} -> {}",
                user_id, old_version, profile.version
            );
        }

        // 5. 保存Persona档案
        self.save_persona_profile(&profile).await;

        Ok(profile)
    }

    /// 动态更新Persona
    ///
    /// 根据新的记忆内容动态更新Persona属性
    pub async fn update_persona_dynamically(
        &self,
        user_id: &str,
        new_memory: &MemoryContent,
    ) -> Result<PersonaProfile> {
        if !self.config.enable_dynamic_update {
            return self.get_or_create_persona(user_id).await;
        }

        info!("动态更新Persona: user_id={}", user_id);

        // 1. 获取当前Persona
        let mut profile = self.get_or_create_persona(user_id).await?;

        // 2. 从新记忆中提取属性
        let new_attributes = self.extract_attributes_from_memory(new_memory).await?;

        // 3. 更新Persona属性
        let mut updated = false;
        for attr in new_attributes {
            if self.should_update_attribute(&profile, &attr).await? {
                profile.attributes.insert(attr.name.clone(), attr);
                updated = true;
            }
        }

        // 4. 如果更新了，保存新版本
        if updated {
            profile.version += 1;
            profile.updated_at = Utc::now();
            profile.stats.update_count += 1;
            self.save_persona_profile(&profile).await;
        }

        Ok(profile)
    }

    /// Persona检索优化
    ///
    /// 使用Persona信息优化记忆检索结果
    pub async fn optimize_retrieval_with_persona(
        &self,
        user_id: &str,
        query: &str,
        search_results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>> {
        if !self.config.enable_retrieval_optimization {
            return Ok(search_results);
        }

        info!("使用Persona优化检索: user_id={}", user_id);

        // 1. 获取Persona档案
        let profile = self.get_or_create_persona(user_id).await?;

        // 2. 计算Persona相关性分数
        let mut scored_results = Vec::new();
        for result in search_results {
            let persona_score = self.calculate_persona_relevance(&profile, &result).await?;
            
            // 结合原始分数和Persona分数
            let original_score = result.score;
            let final_score = original_score * (1.0 - self.config.persona_weight)
                + persona_score * self.config.persona_weight;

            let mut scored_result = result;
            scored_result.score = final_score;
            scored_results.push(scored_result);
        }

        // 3. 按新分数重新排序
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scored_results)
    }

    /// 获取Persona档案
    pub async fn get_persona_profile(&self, user_id: &str) -> Result<Option<PersonaProfile>> {
        let profiles = self.persona_profiles.read().await;
        Ok(profiles.get(user_id).cloned())
    }

    /// 从记忆中提取属性
    async fn extract_attributes_from_memory(
        &self,
        memory: &MemoryContent,
    ) -> Result<Vec<PersonaAttribute>> {
        // 简化的属性提取（实际应使用LLM或NLP工具）
        let mut attributes = Vec::new();

        // 提取偏好、习惯、特征等
        // TODO: 使用更高级的提取方法（LLM、NER等）

        Ok(attributes)
    }

    /// 合并属性
    fn merge_attribute(&self, attributes: &mut HashMap<String, PersonaAttribute>, attr: PersonaAttribute) {
        if let Some(existing) = attributes.get_mut(&attr.name) {
            // 如果新属性置信度更高，更新
            if attr.confidence > existing.confidence {
                *existing = attr;
            } else {
                // 增加更新次数
                existing.update_count += 1;
                existing.updated_at = Utc::now();
            }
        } else {
            attributes.insert(attr.name.clone(), attr);
        }
    }

    /// 过滤和排序属性
    async fn filter_and_rank_attributes(
        &self,
        attributes: HashMap<String, PersonaAttribute>,
    ) -> Result<HashMap<String, PersonaAttribute>> {
        let mut filtered: Vec<PersonaAttribute> = attributes.into_values().collect();

        // 按置信度排序
        filtered.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制数量
        filtered.truncate(self.config.max_persona_attributes);

        // 转换回HashMap
        let mut result = HashMap::new();
        for attr in filtered {
            result.insert(attr.name.clone(), attr);
        }

        Ok(result)
    }

    /// 检查是否应该更新Persona
    async fn should_update_persona(
        &self,
        current: &PersonaProfile,
        new_attributes: &HashMap<String, PersonaAttribute>,
    ) -> Result<bool> {
        if !self.config.enable_dynamic_update {
            return Ok(false);
        }

        // 检查更新频率
        let hours_since_update = (Utc::now() - current.updated_at).num_hours() as u64;
        if hours_since_update < self.config.update_frequency_hours {
            return Ok(false);
        }

        // 计算变化量
        let change_ratio = self.calculate_change_ratio(current, new_attributes).await?;
        Ok(change_ratio >= self.config.update_threshold)
    }

    /// 计算变化比例
    async fn calculate_change_ratio(
        &self,
        current: &PersonaProfile,
        new: &HashMap<String, PersonaAttribute>,
    ) -> Result<f64> {
        let mut changes = 0;
        let mut total = current.attributes.len().max(new.len());

        for (key, new_attr) in new {
            if let Some(old_attr) = current.attributes.get(key) {
                if (new_attr.confidence - old_attr.confidence).abs() > 0.1 {
                    changes += 1;
                }
            } else {
                changes += 1; // 新属性
            }
        }

        // 检查删除的属性
        for key in current.attributes.keys() {
            if !new.contains_key(key) {
                changes += 1;
            }
        }

        if total == 0 {
            return Ok(1.0); // 如果没有现有属性，认为完全变化
        }

        Ok(changes as f64 / total as f64)
    }

    /// 检查是否应该更新单个属性
    async fn should_update_attribute(
        &self,
        profile: &PersonaProfile,
        new_attr: &PersonaAttribute,
    ) -> Result<bool> {
        if let Some(existing) = profile.attributes.get(&new_attr.name) {
            // 如果新属性置信度明显更高，更新
            if new_attr.confidence > existing.confidence + 0.1 {
                return Ok(true);
            }
        } else {
            // 新属性，添加
            return Ok(true);
        }

        Ok(false)
    }

    /// 计算Persona相关性分数
    async fn calculate_persona_relevance(
        &self,
        profile: &PersonaProfile,
        result: &SearchResult,
    ) -> Result<f64> {
        let mut relevance = 0.0;
        let mut count = 0;

        // 检查结果内容是否匹配Persona属性
        for attr in profile.attributes.values() {
            if result.content.to_lowercase().contains(&attr.value.to_lowercase()) {
                relevance += attr.confidence;
                count += 1;
            }
        }

        if count == 0 {
            return Ok(0.5); // 默认中等相关性
        }

        Ok(relevance / count as f64)
    }

    /// 获取或创建Persona档案
    async fn get_or_create_persona(&self, user_id: &str) -> Result<PersonaProfile> {
        let profiles = self.persona_profiles.read().await;
        if let Some(profile) = profiles.get(user_id) {
            return Ok(profile.clone());
        }

        // 创建新Persona
        drop(profiles);
        let new_profile = PersonaProfile {
            user_id: user_id.to_string(),
            attributes: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            stats: PersonaStats::default(),
        };

        let mut profiles = self.persona_profiles.write().await;
        profiles.insert(user_id.to_string(), new_profile.clone());
        Ok(new_profile)
    }

    /// 保存Persona档案
    async fn save_persona_profile(&self, profile: &PersonaProfile) {
        let mut profiles = self.persona_profiles.write().await;
        profiles.insert(profile.user_id.clone(), profile.clone());
    }

    /// 保存提取历史
    async fn save_extraction_history(&self, user_id: &str, profile: &PersonaProfile) {
        let mut history = self.extraction_history.write().await;
        let user_history = history.entry(user_id.to_string()).or_insert_with(Vec::new);
        user_history.push(profile.clone());

        // 限制历史长度
        if user_history.len() > 10 {
            user_history.remove(0);
        }
    }
}

/// 记忆内容（简化版）
#[derive(Debug, Clone)]
pub struct MemoryContent {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

/// 搜索结果（简化版）
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persona_extraction() {
        let engine = PersonaExtractionEngine::with_defaults();
        let memories = vec![MemoryContent {
            id: "1".to_string(),
            content: "I like coffee".to_string(),
            metadata: HashMap::new(),
        }];

        let profile = engine
            .extract_persona_from_memories("user1", memories)
            .await
            .unwrap();
        assert_eq!(profile.user_id, "user1");
    }

    #[tokio::test]
    async fn test_persona_retrieval_optimization() {
        let engine = PersonaExtractionEngine::with_defaults();
        let results = vec![SearchResult {
            id: "1".to_string(),
            content: "coffee shop".to_string(),
            score: 0.7,
        }];

        let optimized = engine
            .optimize_retrieval_with_persona("user1", "coffee", results)
            .await
            .unwrap();
        assert!(!optimized.is_empty());
    }
}
