//! 个性化记忆管理模块
//!
//! 提供用户个性化记忆策略，包括：
//! - 个性化搜索和推荐
//! - 用户偏好学习和适应
//! - 个性化记忆排序和过滤
//! - 用户行为分析和建模

use crate::types::Memory;
use agent_mem_traits::{Result, Session};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// 简单的记忆搜索结果（用于个性化处理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// 记忆
    pub memory: Memory,
    /// 搜索分数
    pub score: f32,
}

/// 个性化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationConfig {
    /// 用户偏好学习率
    pub learning_rate: f32,
    /// 行为历史保留天数
    pub behavior_history_days: u32,
    /// 推荐系统启用
    pub enable_recommendations: bool,
    /// 个性化搜索启用
    pub enable_personalized_search: bool,
    /// 最大用户偏好数量
    pub max_user_preferences: usize,
    /// 行为权重衰减因子
    pub behavior_decay_factor: f32,
}

impl Default for PersonalizationConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            behavior_history_days: 30,
            enable_recommendations: true,
            enable_personalized_search: true,
            max_user_preferences: 100,
            behavior_decay_factor: 0.95,
        }
    }
}

/// 用户偏好类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PreferenceType {
    /// 主题偏好
    Topic,
    /// 内容类型偏好
    ContentType,
    /// 时间偏好
    Temporal,
    /// 重要性偏好
    Importance,
    /// 搜索模式偏好
    SearchPattern,
    /// 交互方式偏好
    InteractionStyle,
}

/// 用户偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    /// 偏好ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 偏好类型
    pub preference_type: PreferenceType,
    /// 偏好值
    pub value: String,
    /// 偏好权重 (0.0-1.0)
    pub weight: f32,
    /// 置信度 (0.0-1.0)
    pub confidence: f32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 使用频率
    pub usage_frequency: u32,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 用户行为类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BehaviorType {
    /// 搜索行为
    Search,
    /// 访问行为
    Access,
    /// 创建行为
    Create,
    /// 更新行为
    Update,
    /// 删除行为
    Delete,
    /// 分享行为
    Share,
    /// 收藏行为
    Favorite,
    /// 评分行为
    Rating,
}

/// 用户行为记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehavior {
    /// 行为ID
    pub id: String,
    /// 用户ID
    pub user_id: String,
    /// 行为类型
    pub behavior_type: BehaviorType,
    /// 目标记忆ID
    pub memory_id: Option<String>,
    /// 搜索查询
    pub search_query: Option<String>,
    /// 行为上下文
    pub context: HashMap<String, String>,
    /// 行为时间
    pub timestamp: DateTime<Utc>,
    /// 会话ID
    pub session_id: String,
    /// 持续时间（秒）
    pub duration: Option<u32>,
    /// 行为结果
    pub result: Option<String>,
}

/// 个性化搜索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedSearchRequest {
    /// 搜索查询
    pub query: String,
    /// 用户ID
    pub user_id: String,
    /// 会话信息
    pub session: Session,
    /// 搜索限制
    pub limit: Option<usize>,
    /// 基础过滤器
    pub filters: Option<HashMap<String, String>>,
    /// 启用个性化
    pub enable_personalization: bool,
    /// 个性化权重
    pub personalization_weight: f32,
}

/// 个性化搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedSearchResult {
    /// 记忆
    pub memory: Memory,
    /// 基础相关性分数
    pub base_score: f32,
    /// 个性化分数
    pub personalization_score: f32,
    /// 最终分数
    pub final_score: f32,
    /// 匹配的偏好
    pub matched_preferences: Vec<UserPreference>,
    /// 推荐原因
    pub recommendation_reasons: Vec<String>,
}

/// 记忆推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecommendation {
    /// 记忆
    pub memory: Memory,
    /// 推荐分数
    pub score: f32,
    /// 推荐类型
    pub recommendation_type: String,
    /// 推荐原因
    pub reasons: Vec<String>,
    /// 相关偏好
    pub related_preferences: Vec<UserPreference>,
    /// 推荐时间
    pub recommended_at: DateTime<Utc>,
}

/// 用户档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// 用户ID
    pub user_id: String,
    /// 用户偏好
    pub preferences: Vec<UserPreference>,
    /// 行为历史
    pub behavior_history: VecDeque<UserBehavior>,
    /// 兴趣标签
    pub interest_tags: HashMap<String, f32>,
    /// 活跃时间段
    pub active_hours: Vec<u8>,
    /// 搜索模式
    pub search_patterns: HashMap<String, u32>,
    /// 档案创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 档案统计
    pub stats: UserProfileStats,
}

/// 用户档案统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileStats {
    /// 总搜索次数
    pub total_searches: u32,
    /// 总访问次数
    pub total_accesses: u32,
    /// 平均会话时长
    pub avg_session_duration: f32,
    /// 最常用的搜索词
    pub top_search_terms: Vec<String>,
    /// 最活跃时间段
    pub most_active_hour: u8,
    /// 偏好多样性分数
    pub preference_diversity: f32,
}

/// 个性化学习结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationLearningResult {
    /// 新发现的偏好
    pub new_preferences: Vec<UserPreference>,
    /// 更新的偏好
    pub updated_preferences: Vec<UserPreference>,
    /// 学习洞察
    pub insights: Vec<String>,
    /// 置信度
    pub confidence: f32,
}

/// 个性化记忆管理器
pub struct PersonalizationManager {
    /// 配置
    config: PersonalizationConfig,
    /// 用户档案存储
    user_profiles: Arc<RwLock<HashMap<String, UserProfile>>>,
    /// 用户偏好存储
    user_preferences: Arc<RwLock<HashMap<String, Vec<UserPreference>>>>,
    /// 行为历史存储
    behavior_history: Arc<RwLock<HashMap<String, VecDeque<UserBehavior>>>>,
    /// 推荐缓存
    recommendation_cache: Arc<RwLock<HashMap<String, Vec<MemoryRecommendation>>>>,
}

impl PersonalizationManager {
    /// 创建新的个性化管理器
    pub fn new(config: PersonalizationConfig) -> Self {
        info!(
            "Initializing PersonalizationManager with config: {:?}",
            config
        );

        Self {
            config,
            user_profiles: Arc::new(RwLock::new(HashMap::new())),
            user_preferences: Arc::new(RwLock::new(HashMap::new())),
            behavior_history: Arc::new(RwLock::new(HashMap::new())),
            recommendation_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录用户行为
    pub async fn record_behavior(&self, behavior: UserBehavior) -> Result<()> {
        debug!("Recording user behavior: {:?}", behavior);

        let mut history = self.behavior_history.write().await;
        let user_history = history
            .entry(behavior.user_id.clone())
            .or_insert_with(VecDeque::new);

        // 添加新行为
        user_history.push_back(behavior.clone());

        // 限制历史记录长度
        let max_history = (self.config.behavior_history_days * 24 * 10) as usize; // 假设每小时10个行为
        while user_history.len() > max_history {
            user_history.pop_front();
        }

        // 异步学习用户偏好
        self.learn_from_behavior(&behavior).await?;

        Ok(())
    }

    /// 从行为中学习用户偏好
    async fn learn_from_behavior(&self, behavior: &UserBehavior) -> Result<()> {
        debug!("Learning from behavior: {:?}", behavior.behavior_type);

        let mut preferences = self.user_preferences.write().await;
        let user_prefs = preferences
            .entry(behavior.user_id.clone())
            .or_insert_with(Vec::new);

        // 根据行为类型学习偏好
        match behavior.behavior_type {
            BehaviorType::Search => {
                if let Some(query) = &behavior.search_query {
                    self.learn_search_preference(user_prefs, query, &behavior.user_id)
                        .await?;
                }
            }
            BehaviorType::Access => {
                if let Some(memory_id) = &behavior.memory_id {
                    self.learn_access_preference(user_prefs, memory_id, &behavior.user_id)
                        .await?;
                }
            }
            BehaviorType::Favorite => {
                if let Some(memory_id) = &behavior.memory_id {
                    self.learn_favorite_preference(user_prefs, memory_id, &behavior.user_id)
                        .await?;
                }
            }
            _ => {
                // 其他行为类型的学习逻辑
                debug!("Learning from behavior type: {:?}", behavior.behavior_type);
            }
        }

        Ok(())
    }

    /// 学习搜索偏好
    async fn learn_search_preference(
        &self,
        user_prefs: &mut Vec<UserPreference>,
        query: &str,
        user_id: &str,
    ) -> Result<()> {
        // 提取搜索关键词
        let keywords = self.extract_keywords(query);

        for keyword in keywords {
            // 查找现有偏好或创建新偏好
            if let Some(pref) = user_prefs
                .iter_mut()
                .find(|p| p.preference_type == PreferenceType::SearchPattern && p.value == keyword)
            {
                // 更新现有偏好
                pref.weight = (pref.weight + self.config.learning_rate).min(1.0);
                pref.usage_frequency += 1;
                pref.updated_at = Utc::now();
                pref.confidence = (pref.confidence + 0.05).min(1.0);
            } else {
                // 创建新偏好
                let new_pref = UserPreference {
                    id: Uuid::new_v4().to_string(),
                    user_id: user_id.to_string(),
                    preference_type: PreferenceType::SearchPattern,
                    value: keyword,
                    weight: self.config.learning_rate,
                    confidence: 0.6,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    usage_frequency: 1,
                    metadata: HashMap::new(),
                };
                user_prefs.push(new_pref);
            }
        }

        // 限制偏好数量
        if user_prefs.len() > self.config.max_user_preferences {
            user_prefs.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
            user_prefs.truncate(self.config.max_user_preferences);
        }

        Ok(())
    }

    /// 学习访问偏好
    async fn learn_access_preference(
        &self,
        _user_prefs: &mut Vec<UserPreference>,
        _memory_id: &str,
        _user_id: &str,
    ) -> Result<()> {
        // TODO: 实现访问偏好学习
        // 分析访问的记忆内容，学习用户对内容类型、主题等的偏好
        Ok(())
    }

    /// 学习收藏偏好
    async fn learn_favorite_preference(
        &self,
        _user_prefs: &mut Vec<UserPreference>,
        _memory_id: &str,
        _user_id: &str,
    ) -> Result<()> {
        // TODO: 实现收藏偏好学习
        // 收藏行为表明强烈偏好，应该给予更高权重
        Ok(())
    }

    /// 个性化搜索
    pub async fn personalized_search(
        &self,
        request: PersonalizedSearchRequest,
        base_results: Vec<MemorySearchResult>,
    ) -> Result<Vec<PersonalizedSearchResult>> {
        debug!(
            "Performing personalized search for user: {}",
            request.user_id
        );

        if !request.enable_personalization || !self.config.enable_personalized_search {
            // 如果未启用个性化，返回基础结果
            return Ok(base_results
                .into_iter()
                .map(|result| PersonalizedSearchResult {
                    memory: result.memory,
                    base_score: result.score,
                    personalization_score: 0.0,
                    final_score: result.score,
                    matched_preferences: Vec::new(),
                    recommendation_reasons: Vec::new(),
                })
                .collect());
        }

        let preferences = self.user_preferences.read().await;
        let user_prefs = preferences
            .get(&request.user_id)
            .cloned()
            .unwrap_or_default();

        let mut personalized_results = Vec::new();

        for result in base_results {
            let personalization_score = self
                .calculate_personalization_score(&result.memory, &user_prefs, &request.query)
                .await?;
            let final_score = result.score * (1.0 - request.personalization_weight)
                + personalization_score * request.personalization_weight;

            let matched_preferences = self
                .find_matched_preferences(&result.memory, &user_prefs, &request.query)
                .await?;
            let recommendation_reasons = self
                .generate_recommendation_reasons(&matched_preferences)
                .await?;

            personalized_results.push(PersonalizedSearchResult {
                memory: result.memory,
                base_score: result.score,
                personalization_score,
                final_score,
                matched_preferences,
                recommendation_reasons,
            });
        }

        // 按最终分数排序
        personalized_results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());

        Ok(personalized_results)
    }

    /// 计算个性化分数
    async fn calculate_personalization_score(
        &self,
        memory: &Memory,
        user_prefs: &[UserPreference],
        query: &str,
    ) -> Result<f32> {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // 基于搜索模式偏好
        let query_keywords = self.extract_keywords(query);
        for pref in user_prefs
            .iter()
            .filter(|p| p.preference_type == PreferenceType::SearchPattern)
        {
            if query_keywords.contains(&pref.value) {
                score += pref.weight * pref.confidence;
                total_weight += pref.weight;
            }
        }

        // 基于内容匹配偏好
        let content_keywords = self.extract_keywords(&memory.memory);
        for pref in user_prefs
            .iter()
            .filter(|p| p.preference_type == PreferenceType::Topic)
        {
            if content_keywords.contains(&pref.value) {
                score += pref.weight * pref.confidence * 0.8; // 内容匹配权重稍低
                total_weight += pref.weight * 0.8;
            }
        }

        // 归一化分数
        if total_weight > 0.0 {
            Ok(score / total_weight)
        } else {
            Ok(0.0)
        }
    }

    /// 查找匹配的偏好
    async fn find_matched_preferences(
        &self,
        memory: &Memory,
        user_prefs: &[UserPreference],
        query: &str,
    ) -> Result<Vec<UserPreference>> {
        let mut matched = Vec::new();

        let query_keywords = self.extract_keywords(query);
        let content_keywords = self.extract_keywords(&memory.memory);

        for pref in user_prefs {
            let is_matched = match pref.preference_type {
                PreferenceType::SearchPattern => query_keywords.contains(&pref.value),
                PreferenceType::Topic => content_keywords.contains(&pref.value),
                PreferenceType::ContentType => {
                    // 基于记忆元数据匹配内容类型
                    memory
                        .metadata
                        .get("content_type")
                        .is_some_and(|ct| ct == &pref.value)
                }
                _ => false,
            };

            if is_matched {
                matched.push(pref.clone());
            }
        }

        Ok(matched)
    }

    /// 生成推荐原因
    async fn generate_recommendation_reasons(
        &self,
        matched_preferences: &[UserPreference],
    ) -> Result<Vec<String>> {
        let mut reasons = Vec::new();

        for pref in matched_preferences {
            let reason = match pref.preference_type {
                PreferenceType::SearchPattern => format!("您经常搜索 '{}'", pref.value),
                PreferenceType::Topic => format!("您对 '{}' 主题感兴趣", pref.value),
                PreferenceType::ContentType => format!("您偏好 '{}' 类型的内容", pref.value),
                PreferenceType::Importance => "符合您的重要性偏好".to_string(),
                PreferenceType::Temporal => "符合您的时间偏好".to_string(),
                PreferenceType::InteractionStyle => "符合您的交互方式偏好".to_string(),
            };
            reasons.push(reason);
        }

        Ok(reasons)
    }

    /// 生成记忆推荐
    pub async fn generate_recommendations(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<MemoryRecommendation>> {
        debug!("Generating recommendations for user: {}", user_id);

        if !self.config.enable_recommendations {
            return Ok(Vec::new());
        }

        // 检查缓存
        let cache = self.recommendation_cache.read().await;
        if let Some(cached_recommendations) = cache.get(user_id) {
            return Ok(cached_recommendations.iter().take(limit).cloned().collect());
        }
        drop(cache);

        let preferences = self.user_preferences.read().await;
        let user_prefs = preferences.get(user_id).cloned().unwrap_or_default();
        drop(preferences);

        if user_prefs.is_empty() {
            return Ok(Vec::new());
        }

        // TODO: 实现基于偏好的推荐算法
        // 这里需要访问记忆存储来查找相关记忆
        let recommendations = Vec::new();

        // 缓存推荐结果
        let mut cache = self.recommendation_cache.write().await;
        cache.insert(user_id.to_string(), recommendations.clone());

        Ok(recommendations.into_iter().take(limit).collect())
    }

    /// 获取用户偏好
    pub async fn get_user_preferences(&self, user_id: &str) -> Result<Vec<UserPreference>> {
        let preferences = self.user_preferences.read().await;
        Ok(preferences.get(user_id).cloned().unwrap_or_default())
    }

    /// 更新用户偏好
    pub async fn update_user_preference(&self, preference: UserPreference) -> Result<()> {
        let mut preferences = self.user_preferences.write().await;
        let user_prefs = preferences
            .entry(preference.user_id.clone())
            .or_insert_with(Vec::new);

        // 查找现有偏好并更新，或添加新偏好
        if let Some(existing) = user_prefs.iter_mut().find(|p| p.id == preference.id) {
            *existing = preference;
        } else {
            user_prefs.push(preference);
        }

        Ok(())
    }

    /// 删除用户偏好
    pub async fn delete_user_preference(&self, user_id: &str, preference_id: &str) -> Result<bool> {
        let mut preferences = self.user_preferences.write().await;
        if let Some(user_prefs) = preferences.get_mut(user_id) {
            let original_len = user_prefs.len();
            user_prefs.retain(|p| p.id != preference_id);
            Ok(user_prefs.len() < original_len)
        } else {
            Ok(false)
        }
    }

    /// 获取用户档案
    pub async fn get_user_profile(&self, user_id: &str) -> Result<Option<UserProfile>> {
        let profiles = self.user_profiles.read().await;
        Ok(profiles.get(user_id).cloned())
    }

    /// 更新用户档案
    pub async fn update_user_profile(&self, user_id: &str) -> Result<UserProfile> {
        let preferences = self.get_user_preferences(user_id).await?;
        let behavior_history = self.behavior_history.read().await;
        let user_behaviors = behavior_history.get(user_id).cloned().unwrap_or_default();

        // 计算统计信息
        let stats = self.calculate_user_stats(&user_behaviors).await?;

        let profile = UserProfile {
            user_id: user_id.to_string(),
            preferences,
            behavior_history: user_behaviors,
            interest_tags: self.extract_interest_tags(user_id).await?,
            active_hours: self.calculate_active_hours(user_id).await?,
            search_patterns: self.analyze_search_patterns(user_id).await?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            stats,
        };

        // 更新档案存储
        let mut profiles = self.user_profiles.write().await;
        profiles.insert(user_id.to_string(), profile.clone());

        Ok(profile)
    }

    /// 计算用户统计信息
    async fn calculate_user_stats(
        &self,
        behaviors: &VecDeque<UserBehavior>,
    ) -> Result<UserProfileStats> {
        let total_searches = behaviors
            .iter()
            .filter(|b| b.behavior_type == BehaviorType::Search)
            .count() as u32;
        let total_accesses = behaviors
            .iter()
            .filter(|b| b.behavior_type == BehaviorType::Access)
            .count() as u32;

        let avg_session_duration = behaviors
            .iter()
            .filter_map(|b| b.duration)
            .map(|d| d as f32)
            .sum::<f32>()
            / behaviors.len().max(1) as f32;

        let mut search_terms = HashMap::new();
        for behavior in behaviors
            .iter()
            .filter(|b| b.behavior_type == BehaviorType::Search)
        {
            if let Some(query) = &behavior.search_query {
                let keywords = self.extract_keywords(query);
                for keyword in keywords {
                    *search_terms.entry(keyword).or_insert(0) += 1;
                }
            }
        }

        let mut top_search_terms: Vec<_> = search_terms.into_iter().collect();
        top_search_terms.sort_by(|a, b| b.1.cmp(&a.1));
        let top_search_terms = top_search_terms
            .into_iter()
            .take(10)
            .map(|(term, _)| term)
            .collect();

        // 计算最活跃时间段
        let mut hour_counts = [0u32; 24];
        for behavior in behaviors {
            let hour = behavior.timestamp.hour() as usize;
            hour_counts[hour] += 1;
        }
        let most_active_hour = hour_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| hour as u8)
            .unwrap_or(12);

        Ok(UserProfileStats {
            total_searches,
            total_accesses,
            avg_session_duration,
            top_search_terms,
            most_active_hour,
            preference_diversity: 0.8, // TODO: 实现偏好多样性计算
        })
    }

    /// 提取兴趣标签
    async fn extract_interest_tags(&self, user_id: &str) -> Result<HashMap<String, f32>> {
        let preferences = self.get_user_preferences(user_id).await?;
        let mut tags = HashMap::new();

        for pref in preferences {
            if pref.preference_type == PreferenceType::Topic {
                tags.insert(pref.value, pref.weight * pref.confidence);
            }
        }

        Ok(tags)
    }

    /// 计算活跃时间段
    async fn calculate_active_hours(&self, user_id: &str) -> Result<Vec<u8>> {
        let behavior_history = self.behavior_history.read().await;
        let user_behaviors = behavior_history.get(user_id).cloned().unwrap_or_default();

        let mut hour_counts = [0u32; 24];
        for behavior in user_behaviors.iter() {
            let hour = behavior.timestamp.hour() as usize;
            hour_counts[hour] += 1;
        }

        // 找出活跃度超过平均值的时间段
        let avg_activity = hour_counts.iter().sum::<u32>() as f32 / 24.0;
        let active_hours = hour_counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count as f32 > avg_activity)
            .map(|(hour, _)| hour as u8)
            .collect();

        Ok(active_hours)
    }

    /// 分析搜索模式
    async fn analyze_search_patterns(&self, user_id: &str) -> Result<HashMap<String, u32>> {
        let behavior_history = self.behavior_history.read().await;
        let user_behaviors = behavior_history.get(user_id).cloned().unwrap_or_default();

        let mut patterns = HashMap::new();

        for behavior in user_behaviors
            .iter()
            .filter(|b| b.behavior_type == BehaviorType::Search)
        {
            if let Some(query) = &behavior.search_query {
                let keywords = self.extract_keywords(query);
                for keyword in keywords {
                    *patterns.entry(keyword).or_insert(0) += 1;
                }
            }
        }

        Ok(patterns)
    }

    /// 提取关键词
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // 简单的关键词提取
        text.split_whitespace()
            .filter(|word| word.len() > 2)
            .map(|word| {
                word.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_personalization_manager_creation() {
        let config = PersonalizationConfig::default();
        let manager = PersonalizationManager::new(config);

        // 验证管理器创建成功
        assert!(manager.user_profiles.read().await.is_empty());
        assert!(manager.user_preferences.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_behavior_recording() {
        let config = PersonalizationConfig::default();
        let manager = PersonalizationManager::new(config);

        let behavior = UserBehavior {
            id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            behavior_type: BehaviorType::Search,
            memory_id: None,
            search_query: Some("rust programming".to_string()),
            context: HashMap::new(),
            timestamp: Utc::now(),
            session_id: "test_session".to_string(),
            duration: Some(30),
            result: Some("success".to_string()),
        };

        let result = manager.record_behavior(behavior).await;
        assert!(result.is_ok());

        // 验证行为已记录
        let history = manager.behavior_history.read().await;
        assert!(history.contains_key("test_user"));
        assert_eq!(history.get("test_user").unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_keyword_extraction() {
        let config = PersonalizationConfig::default();
        let manager = PersonalizationManager::new(config);

        let keywords = manager.extract_keywords("rust programming language tutorial");
        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"programming".to_string()));
        assert!(keywords.contains(&"language".to_string()));
        assert!(keywords.contains(&"tutorial".to_string()));
    }

    #[tokio::test]
    async fn test_user_preference_management() {
        let config = PersonalizationConfig::default();
        let manager = PersonalizationManager::new(config);

        let preference = UserPreference {
            id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            preference_type: PreferenceType::Topic,
            value: "rust".to_string(),
            weight: 0.8,
            confidence: 0.9,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_frequency: 5,
            metadata: HashMap::new(),
        };

        // 更新偏好
        let result = manager.update_user_preference(preference.clone()).await;
        assert!(result.is_ok());

        // 获取偏好
        let preferences = manager.get_user_preferences("test_user").await.unwrap();
        assert_eq!(preferences.len(), 1);
        assert_eq!(preferences[0].value, "rust");

        // 删除偏好
        let deleted = manager
            .delete_user_preference("test_user", &preference.id)
            .await
            .unwrap();
        assert!(deleted);

        // 验证删除
        let preferences = manager.get_user_preferences("test_user").await.unwrap();
        assert_eq!(preferences.len(), 0);
    }
}
