//! TopicExtractor - 基于 LLM 的主题提取器
//!
//! 参考 MIRIX 的主题提取机制，实现智能主题识别和层次化分类

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 主题类别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopicCategory {
    /// 技术相关
    Technical,
    /// 业务相关
    Business,
    /// 个人相关
    Personal,
    /// 学术相关
    Academic,
    /// 娱乐相关
    Entertainment,
    /// 健康相关
    Health,
    /// 财务相关
    Financial,
    /// 其他
    Other,
}

impl TopicCategory {
    /// 获取类别的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            TopicCategory::Technical => "技术、编程、软件开发相关主题",
            TopicCategory::Business => "商业、工作、项目管理相关主题",
            TopicCategory::Personal => "个人生活、兴趣爱好相关主题",
            TopicCategory::Academic => "学术研究、教育学习相关主题",
            TopicCategory::Entertainment => "娱乐、游戏、媒体相关主题",
            TopicCategory::Health => "健康、医疗、运动相关主题",
            TopicCategory::Financial => "财务、投资、经济相关主题",
            TopicCategory::Other => "其他未分类主题",
        }
    }
}

/// 提取的主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTopic {
    /// 主题名称
    pub name: String,
    /// 主题类别
    pub category: TopicCategory,
    /// 置信度分数 (0.0-1.0)
    pub confidence: f32,
    /// 关键词
    pub keywords: Vec<String>,
    /// 主题描述
    pub description: Option<String>,
    /// 层次级别 (0为顶级)
    pub hierarchy_level: u32,
    /// 父主题ID
    pub parent_topic_id: Option<String>,
    /// 相关性分数
    pub relevance_score: f32,
}

/// 主题层次结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicHierarchy {
    /// 根主题
    pub root_topics: Vec<ExtractedTopic>,
    /// 主题关系映射
    pub topic_relationships: HashMap<String, Vec<String>>,
    /// 主题相似度矩阵
    pub similarity_matrix: HashMap<(String, String), f32>,
}

/// 主题提取器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicExtractorConfig {
    /// 是否启用多语言支持
    pub enable_multilingual: bool,
    /// 支持的语言列表
    pub supported_languages: Vec<String>,
    /// 最大主题数量
    pub max_topics: usize,
    /// 最小置信度阈值
    pub min_confidence_threshold: f32,
    /// 是否启用层次化分类
    pub enable_hierarchical_classification: bool,
    /// 是否启用主题相似度计算
    pub enable_similarity_calculation: bool,
    /// LLM 模型名称
    pub llm_model: String,
    /// LLM API 密钥
    pub llm_api_key: Option<String>,
    /// 请求超时时间（秒）
    pub request_timeout_seconds: u64,
}

impl Default for TopicExtractorConfig {
    fn default() -> Self {
        Self {
            enable_multilingual: true,
            supported_languages: vec![
                "zh".to_string(),
                "en".to_string(),
                "ja".to_string(),
                "ko".to_string(),
            ],
            max_topics: 10,
            min_confidence_threshold: 0.3,
            enable_hierarchical_classification: true,
            enable_similarity_calculation: true,
            llm_model: "gpt-3.5-turbo".to_string(),
            llm_api_key: None,
            request_timeout_seconds: 30,
        }
    }
}

/// 主题提取器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicExtractorStats {
    /// 总提取次数
    pub total_extractions: u64,
    /// 成功提取次数
    pub successful_extractions: u64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
    /// 按类别统计的主题数量
    pub topics_by_category: HashMap<TopicCategory, u64>,
    /// 平均置信度
    pub avg_confidence: f32,
}

/// 主题提取器
///
/// 基于 LLM 的智能主题提取系统，支持多语言和层次化分类
pub struct TopicExtractor {
    /// 配置
    config: TopicExtractorConfig,
    /// 统计信息
    stats: Arc<RwLock<TopicExtractorStats>>,
    /// 主题缓存
    topic_cache: Arc<RwLock<HashMap<String, Vec<ExtractedTopic>>>>,
    /// 相似度缓存
    similarity_cache: Arc<RwLock<HashMap<(String, String), f32>>>,
}

impl TopicExtractor {
    /// 创建新的主题提取器
    pub async fn new(config: TopicExtractorConfig) -> Result<Self> {
        let stats = TopicExtractorStats {
            total_extractions: 0,
            successful_extractions: 0,
            avg_processing_time_ms: 0.0,
            topics_by_category: HashMap::new(),
            avg_confidence: 0.0,
        };

        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(stats)),
            topic_cache: Arc::new(RwLock::new(HashMap::new())),
            similarity_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 提取主题
    pub async fn extract_topics(
        &self,
        text: &str,
        context: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<ExtractedTopic>> {
        let start_time = std::time::Instant::now();

        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_extractions += 1;
        }

        // 检查缓存
        let cache_key = self.generate_cache_key(text, context);
        if let Some(cached_topics) = self.check_topic_cache(&cache_key).await {
            return Ok(cached_topics);
        }

        // 检测语言
        let detected_language = self.detect_language(text).await?;

        // 执行主题提取
        let topics = if self.config.enable_multilingual && detected_language != "en" {
            self.extract_multilingual_topics(text, &detected_language, context)
                .await?
        } else {
            self.extract_english_topics(text, context).await?
        };

        // 过滤低置信度主题
        let filtered_topics: Vec<ExtractedTopic> = topics
            .into_iter()
            .filter(|topic| topic.confidence >= self.config.min_confidence_threshold)
            .take(self.config.max_topics)
            .collect();

        // 层次化分类
        let hierarchical_topics = if self.config.enable_hierarchical_classification {
            self.build_topic_hierarchy(&filtered_topics).await?
        } else {
            filtered_topics
        };

        // 计算相似度
        let final_topics = if self.config.enable_similarity_calculation {
            self.calculate_topic_similarities(&hierarchical_topics)
                .await?
        } else {
            hierarchical_topics
        };

        // 缓存结果
        self.cache_topics(&cache_key, &final_topics).await;

        // 更新统计信息
        let processing_time = start_time.elapsed().as_millis() as f64;
        self.update_stats(&final_topics, processing_time).await;

        Ok(final_topics)
    }

    /// 检测文本语言
    async fn detect_language(&self, text: &str) -> Result<String> {
        // 简化的语言检测逻辑
        // 在实际实现中，可以使用更复杂的语言检测库

        let chinese_chars = text
            .chars()
            .filter(|c| {
                *c >= '\u{4e00}' && *c <= '\u{9fff}' // 中文字符范围
            })
            .count();

        let total_chars = text.chars().count();

        if total_chars > 0 && (chinese_chars as f32 / total_chars as f32) > 0.3 {
            Ok("zh".to_string())
        } else {
            Ok("en".to_string())
        }
    }

    /// 提取英文主题
    async fn extract_english_topics(
        &self,
        text: &str,
        _context: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<ExtractedTopic>> {
        // 简化的主题提取实现
        // 在实际实现中，这里会调用 LLM API

        let keywords = self.extract_keywords(text);
        let mut topics = Vec::new();

        // 基于关键词生成主题
        for (i, keyword) in keywords.iter().enumerate().take(self.config.max_topics) {
            let category = self.classify_topic_category(keyword);
            let topic = ExtractedTopic {
                name: keyword.clone(),
                category,
                confidence: 0.8 - (i as f32 * 0.1), // 简化的置信度计算
                keywords: vec![keyword.clone()],
                description: Some(format!("主题: {keyword}")),
                hierarchy_level: 0,
                parent_topic_id: None,
                relevance_score: 0.9 - (i as f32 * 0.1),
            };
            topics.push(topic);
        }

        Ok(topics)
    }

    /// 提取多语言主题
    async fn extract_multilingual_topics(
        &self,
        text: &str,
        language: &str,
        context: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<ExtractedTopic>> {
        // 对于非英语文本，先进行翻译再提取主题
        // 这里简化处理，直接调用英文提取
        self.extract_english_topics(text, context).await
    }

    /// 提取关键词
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        // 简化的关键词提取
        text.split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(10)
            .collect()
    }

    /// 分类主题类别
    fn classify_topic_category(&self, keyword: &str) -> TopicCategory {
        // 简化的分类逻辑
        let word = keyword.to_lowercase();
        if word.contains("code")
            || word.contains("program")
            || word.contains("tech")
            || word.contains("人工智能")
            || word.contains("机器学习")
            || word.contains("智能")
            || word.contains("技术")
            || word.contains("算法")
        {
            TopicCategory::Technical
        } else if word.contains("business")
            || word.contains("work")
            || word.contains("project")
            || word.contains("业务")
            || word.contains("工作")
            || word.contains("项目")
        {
            TopicCategory::Business
        } else if word.contains("personal")
            || word.contains("life")
            || word.contains("hobby")
            || word.contains("个人")
            || word.contains("生活")
            || word.contains("爱好")
        {
            TopicCategory::Personal
        } else if word.contains("study")
            || word.contains("research")
            || word.contains("academic")
            || word.contains("学习")
            || word.contains("研究")
            || word.contains("学术")
        {
            TopicCategory::Academic
        } else if word.contains("game")
            || word.contains("movie")
            || word.contains("music")
            || word.contains("游戏")
            || word.contains("电影")
            || word.contains("音乐")
        {
            TopicCategory::Entertainment
        } else if word.contains("health")
            || word.contains("medical")
            || word.contains("fitness")
            || word.contains("健康")
            || word.contains("医疗")
            || word.contains("运动")
        {
            TopicCategory::Health
        } else if word.contains("money")
            || word.contains("finance")
            || word.contains("investment")
            || word.contains("金钱")
            || word.contains("财务")
            || word.contains("投资")
        {
            TopicCategory::Financial
        } else {
            TopicCategory::Other
        }
    }

    /// 构建主题层次结构
    async fn build_topic_hierarchy(
        &self,
        topics: &[ExtractedTopic],
    ) -> Result<Vec<ExtractedTopic>> {
        // 简化的层次结构构建
        // 在实际实现中，这里会使用更复杂的算法
        Ok(topics.to_vec())
    }

    /// 计算主题相似度
    async fn calculate_topic_similarities(
        &self,
        topics: &[ExtractedTopic],
    ) -> Result<Vec<ExtractedTopic>> {
        // 简化的相似度计算
        // 在实际实现中，这里会计算主题间的相似度
        Ok(topics.to_vec())
    }

    /// 生成缓存键
    fn generate_cache_key(
        &self,
        text: &str,
        context: Option<&HashMap<String, serde_json::Value>>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        if let Some(ctx) = context {
            format!("{ctx:?}").hash(&mut hasher);
        }

        format!("topic_{}", hasher.finish())
    }

    /// 检查主题缓存
    async fn check_topic_cache(&self, cache_key: &str) -> Option<Vec<ExtractedTopic>> {
        let cache = self.topic_cache.read().await;
        cache.get(cache_key).cloned()
    }

    /// 缓存主题
    async fn cache_topics(&self, cache_key: &str, topics: &[ExtractedTopic]) {
        let mut cache = self.topic_cache.write().await;
        cache.insert(cache_key.to_string(), topics.to_vec());
    }

    /// 更新统计信息
    async fn update_stats(&self, topics: &[ExtractedTopic], processing_time_ms: f64) {
        let mut stats = self.stats.write().await;
        stats.successful_extractions += 1;

        // 更新平均处理时间
        let total_time = stats.avg_processing_time_ms * (stats.successful_extractions - 1) as f64
            + processing_time_ms;
        stats.avg_processing_time_ms = total_time / stats.successful_extractions as f64;

        // 更新类别统计
        for topic in topics {
            *stats
                .topics_by_category
                .entry(topic.category.clone())
                .or_insert(0) += 1;
        }

        // 更新平均置信度
        if !topics.is_empty() {
            let total_confidence: f32 = topics.iter().map(|t| t.confidence).sum();
            let avg_confidence = total_confidence / topics.len() as f32;

            let total_topics = stats.topics_by_category.values().sum::<u64>() as f32;
            stats.avg_confidence = (stats.avg_confidence * (total_topics - topics.len() as f32)
                + avg_confidence * topics.len() as f32)
                / total_topics;
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let stats = self.stats.read().await;
        serde_json::to_value(&*stats)
            .map_err(|e| AgentMemError::ProcessingError(format!("Failed to serialize stats: {e}")))
    }

    /// 清理缓存
    pub async fn clear_cache(&self) -> Result<()> {
        let mut topic_cache = self.topic_cache.write().await;
        let mut similarity_cache = self.similarity_cache.write().await;

        topic_cache.clear();
        similarity_cache.clear();

        Ok(())
    }
}
