//! Orchestrator Utils - 辅助方法和工具函数
//!
//! 提供各种辅助方法，包括查询预处理、阈值计算、类型转换等

use agent_mem_core::types::Memory as CoreMemory;
use agent_mem_core::types::MemoryType;
use agent_mem_intelligence::{ExistingMemory, StructuredFact};
use agent_mem_traits::{MemoryItem, Result};
use std::collections::HashMap;
use tracing::{debug, error};

/// 工具函数模块
pub struct UtilsModule;

impl UtilsModule {
    /// 计算动态阈值
    ///
    /// 根据查询特征动态调整相似度阈值
    pub fn calculate_dynamic_threshold(query: &str, base_threshold: Option<f32>) -> f32 {
        let base = base_threshold.unwrap_or(0.7);

        let query_len = query.len();
        let word_count = query.split_whitespace().count();

        // 检测精确查询模式（商品ID、SKU等）
        let is_exact_query =
            query.chars().all(|c| c.is_alphanumeric()) && query_len < 20 && word_count <= 1;

        // 规则1: 精确查询（如P000001）大幅降低阈值
        let len_adjustment = if is_exact_query {
            -0.4 // 精确查询降到0.3，因为向量相似度本来就低
        } else if query_len < 10 {
            -0.2 // 短查询也降低阈值
        } else if query_len > 100 {
            -0.05 // 长查询降低阈值到0.65，提高召回率
        } else {
            0.0
        };

        // 规则2: 单词数少降低阈值
        let word_adjustment = if is_exact_query {
            0.0 // 精确查询已经在规则1处理
        } else if word_count == 1 {
            -0.1 // 单词查询降低阈值
        } else if word_count > 10 {
            -0.05 // 多词查询更宽松
        } else {
            0.0
        };

        // 规则3: 精确查询不需要特殊字符检查
        let has_special = !is_exact_query
            && query
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace());
        let special_adjustment = if has_special { 0.05 } else { 0.0 };

        // 计算最终阈值
        let dynamic_threshold = base + len_adjustment + word_adjustment + special_adjustment;

        // 限制在合理范围内 [0.2, 0.9]
        let final_threshold = dynamic_threshold.max(0.2).min(0.9);

        if final_threshold != base {
            debug!(
                "动态阈值调整: {} -> {} (查询长度: {}, 词数: {}, 特殊字符: {})",
                base, final_threshold, query_len, word_count, has_special
            );
        }

        final_threshold
    }

    /// 查询预处理
    ///
    /// 清理和标准化查询文本
    pub async fn preprocess_query(query: &str) -> Result<String> {
        // Step 1: 基础清理
        let mut processed = query.trim().to_string();

        // Step 2: 移除常见停用词（中英文）
        let stopwords = [
            // 英文停用词
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
            "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has", "had",
            "do", "does", "did", "will", "would", "should", "could", "may", "might", "can",
            // 中文停用词
            "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上",
            "也", "很", "到", "说", "要", "去", "你", "会",
        ];

        let words: Vec<&str> = processed.split_whitespace().collect();
        let filtered_words: Vec<&str> = words
            .into_iter()
            .filter(|word| {
                let lower = word.to_lowercase();
                !stopwords.contains(&lower.as_str())
            })
            .collect();

        // Step 3: 重新组合（如果过滤后为空，保留原始查询）
        if !filtered_words.is_empty() {
            processed = filtered_words.join(" ");
        }

        // Step 4: 转小写
        processed = processed.to_lowercase();

        // Step 5: 移除多余空格
        processed = processed
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        debug!("查询预处理: '{}' -> '{}'", query, processed);

        Ok(processed)
    }

    /// 生成查询嵌入向量
    ///
    /// 使用 Embedder 生成查询的向量表示
    pub async fn generate_query_embedding(
        query: &str,
        embedder: &dyn agent_mem_traits::Embedder,
    ) -> Result<Vec<f32>> {
        match embedder.embed(query).await {
            Ok(embedding) => {
                debug!(
                    "成功生成查询嵌入向量: query={}, dimension={}",
                    query,
                    embedding.len()
                );
                Ok(embedding)
            }
            Err(e) => {
                error!("生成查询嵌入向量失败: {}", e);
                Err(agent_mem_traits::AgentMemError::EmbeddingError(format!(
                    "Failed to generate query embedding: {e}"
                )))
            }
        }
    }

    /// 转换搜索结果为 MemoryItem
    ///
    /// 将 SearchResult 转换为 MemoryItem 格式
    #[cfg(feature = "postgres")]
    pub fn convert_search_results_to_memory_items(results: Vec<SearchResult>) -> Vec<MemoryItem> {
        if results.is_empty() {
            return Vec::new();
        }

        debug!("批量转换 {} 个搜索结果", results.len());

        results
            .into_iter()
            .map(|result| {
                // 解析元数据
                let metadata = if let Some(meta) = result.metadata {
                    if let Ok(map) = serde_json::from_value::<HashMap<String, String>>(meta) {
                        map
                    } else {
                        HashMap::new()
                    }
                } else {
                    HashMap::new()
                };

                // 创建 MemoryItem
                MemoryItem {
                    id: result.id,
                    content: result.content,
                    hash: None,
                    metadata,
                    score: Some(result.score),
                    created_at: chrono::Utc::now(),
                    updated_at: None,
                    session: agent_mem_traits::Session::new(),
                    memory_type: agent_mem_traits::MemoryType::Semantic,
                    entities: Vec::new(),
                    relations: Vec::new(),
                    agent_id: "default".to_string(),
                    user_id: None,
                    importance: result.score,
                    embedding: None,
                    last_accessed_at: chrono::Utc::now(),
                    access_count: 0,
                    expires_at: None,
                    version: 1,
                }
            })
            .collect()
    }

    /// 将 StructuredFact 转换为 MemoryItem
    ///
    /// 用于调用 Intelligence 组件
    pub fn structured_fact_to_memory_item(
        fact: &StructuredFact,
        agent_id: String,
        user_id: Option<String>,
    ) -> MemoryItem {
        use agent_mem_traits::Session;
        use chrono::Utc;

        let now = Utc::now();

        // 将 StructuredFact 的 metadata 转换为 HashMap<String, serde_json::Value>
        let metadata: HashMap<String, serde_json::Value> = fact
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        MemoryItem {
            id: fact.id.clone(),
            content: fact.description.clone(),
            hash: None,
            metadata,
            score: Some(fact.importance),
            created_at: now,
            updated_at: None,
            session: Session {
                id: "default".to_string(),
                user_id: user_id.clone(),
                agent_id: Some(agent_id.clone()),
                run_id: None,
                actor_id: Some(agent_id.clone()),
                created_at: now,
                metadata: HashMap::new(),
            },
            memory_type: agent_mem_traits::MemoryType::Semantic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id,
            user_id,
            importance: fact.importance,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }

    /// 将 StructuredFact 转换为 CoreMemory
    ///
    /// 用于调用 Intelligence 组件
    pub fn structured_fact_to_core_memory(
        fact: &StructuredFact,
        agent_id: String,
        user_id: Option<String>,
    ) -> CoreMemory {
        use chrono::Utc;

        let _now = Utc::now().timestamp();

        // 将 StructuredFact 的 metadata 转换为 HashMap<String, String>
        let metadata: HashMap<String, String> = fact.metadata.clone();

        let mut mem = CoreMemory::new(
            agent_id,
            user_id,
            MemoryType::Semantic,
            fact.description.clone(),
            fact.importance,
        );
        mem.id = fact.id.clone();

        // Add metadata to attributes
        for (key, value) in metadata {
            mem.add_metadata(key, value);
        }

        mem
    }

    /// 将 ExistingMemory 转换为 MemoryItem
    ///
    /// 用于调用 Intelligence 组件
    pub fn existing_memory_to_memory_item(memory: &ExistingMemory) -> MemoryItem {
        use agent_mem_traits::Session;
        use chrono::Utc;

        let now = Utc::now();

        // 将 ExistingMemory 的 metadata 转换为 HashMap<String, serde_json::Value>
        let metadata: HashMap<String, serde_json::Value> = memory
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        // 解析 created_at 字符串为 DateTime
        let created_at = chrono::DateTime::parse_from_rfc3339(&memory.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now);

        // 解析 updated_at 字符串为 DateTime
        let updated_at = memory
            .updated_at
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        MemoryItem {
            id: memory.id.clone(),
            content: memory.content.clone(),
            hash: None,
            metadata,
            score: Some(memory.importance),
            created_at,
            updated_at,
            session: Session::new(),
            memory_type: agent_mem_traits::MemoryType::Semantic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "default_agent".to_string(),
            user_id: None,
            importance: memory.importance,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }

    /// 将 ExistingMemory 转换为 CoreMemory
    ///
    /// 用于调用 Intelligence 组件
    pub fn existing_memory_to_core_memory(memory: &ExistingMemory) -> CoreMemory {
        use chrono::Utc;

        let now = Utc::now().timestamp();

        // 将 ExistingMemory 的 metadata 转换为 HashMap<String, String>
        let metadata: HashMap<String, String> = memory.metadata.clone();

        // 解析 created_at 字符串为时间戳
        let _created_at = chrono::DateTime::parse_from_rfc3339(&memory.created_at)
            .map(|dt| dt.timestamp())
            .unwrap_or(now);

        let mut mem = CoreMemory::new(
            "default_agent".to_string(),
            None,
            MemoryType::Semantic,
            memory.content.clone(),
            memory.importance,
        );
        mem.id = memory.id.clone();

        // Add metadata to attributes
        for (key, value) in metadata {
            mem.add_metadata(key, value);
        }

        mem
    }

    /// 推断scope类型（Mem0风格）
    ///
    /// 根据user_id, agent_id和metadata中的信息自动推断记忆作用域
    pub fn infer_scope_type(
        user_id: &str,
        agent_id: &str,
        metadata: &Option<HashMap<String, serde_json::Value>>,
    ) -> String {
        // 检查metadata中是否有run_id或session_id
        if let Some(meta) = metadata {
            if meta.contains_key("run_id") {
                return "run".to_string();
            }
            if meta.contains_key("session_id") {
                return "session".to_string();
            }
            if meta.contains_key("org_id") {
                return "organization".to_string();
            }
        }

        // 默认逻辑
        if user_id != "default" && agent_id != "default" {
            "agent".to_string()
        } else if user_id != "default" {
            "user".to_string()
        } else {
            "global".to_string()
        }
    }

    /// 构建标准元数据
    ///
    /// 构建与mem0兼容的标准元数据格式
    pub fn build_standard_metadata(
        content: &str,
        hash: &str,
        user_id: &Option<String>,
        agent_id: &str,
        run_id: &Option<String>,
        actor_id: &Option<String>,
        role: &Option<String>,
        custom_metadata: &Option<HashMap<String, serde_json::Value>>,
    ) -> HashMap<String, serde_json::Value> {
        use chrono::Utc;
        let mut metadata = HashMap::new();

        // 标准字段（与 mem0 兼容）
        metadata.insert("data".to_string(), serde_json::json!(content));
        metadata.insert("hash".to_string(), serde_json::json!(hash));
        metadata.insert(
            "created_at".to_string(),
            serde_json::json!(Utc::now().to_rfc3339()),
        );

        if let Some(uid) = user_id {
            metadata.insert("user_id".to_string(), serde_json::json!(uid));
        }
        metadata.insert("agent_id".to_string(), serde_json::json!(agent_id));

        if let Some(rid) = run_id {
            metadata.insert("run_id".to_string(), serde_json::json!(rid));
        }
        if let Some(aid) = actor_id {
            metadata.insert("actor_id".to_string(), serde_json::json!(aid));
        }
        if let Some(r) = role {
            metadata.insert("role".to_string(), serde_json::json!(r));
        }

        // 合并自定义 metadata
        if let Some(custom) = custom_metadata {
            for (k, v) in custom {
                metadata.insert(k.clone(), v.clone());
            }
        }

        metadata
    }

    /// 去重记忆项
    ///
    /// 基于ID去重，保留第一次出现的项（通常相似度最高）
    pub fn deduplicate_memory_items(items: Vec<MemoryItem>) -> Vec<MemoryItem> {
        let mut seen_ids = std::collections::HashSet::new();
        items
            .into_iter()
            .filter(|item| seen_ids.insert(item.id.clone()))
            .collect()
    }

    /// 推断记忆类型
    ///
    /// 基于内容关键词推断记忆类型
    pub fn infer_memory_type(content: &str) -> MemoryType {
        let content_lower = content.to_lowercase();

        // 核心记忆关键词
        if content_lower.contains("i am")
            || content_lower.contains("my name is")
            || content_lower.contains("i'm")
            || content_lower.contains("我是")
            || content_lower.contains("我叫")
        {
            return MemoryType::Core;
        }

        // 情景记忆关键词
        if content_lower.contains("happened")
            || content_lower.contains("did")
            || content_lower.contains("went to")
            || content_lower.contains("发生")
            || content_lower.contains("去了")
        {
            return MemoryType::Episodic;
        }

        // 程序记忆关键词
        if content_lower.contains("how to")
            || content_lower.contains("步骤")
            || content_lower.contains("方法")
        {
            return MemoryType::Procedural;
        }

        // 默认为语义记忆
        MemoryType::Semantic
    }

    /// 构建重排序提示词
    pub fn build_rerank_prompt(query: &str, memory_items: &[MemoryItem], user_id: &str) -> String {
        let mut prompt = format!(
            "用户查询: {query}\n用户ID: {user_id}\n\n请根据查询意图和相关性对以下记忆进行重新排序。\n\n记忆列表:\n"
        );

        for (idx, item) in memory_items.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. [{}] {} (重要性: {:.2})\n",
                idx, item.id, item.content, item.importance
            ));
        }

        prompt.push_str(
            "\n请返回最相关记忆的索引列表（用逗号分隔），例如: 0,2,1,3\n仅返回索引，不要其他内容。",
        );
        prompt
    }

    /// 解析重排序响应
    pub fn parse_rerank_response(response: &str, max_len: usize) -> Result<Vec<usize>> {
        // 提取数字
        let numbers: Vec<usize> = response
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .filter(|&n| n < max_len)
            .collect();

        if numbers.is_empty() {
            // 尝试按行解析
            let numbers: Vec<usize> = response
                .lines()
                .filter_map(|line| {
                    line.split_whitespace().next().and_then(|s| {
                        s.trim_matches(|c: char| !c.is_numeric())
                            .parse::<usize>()
                            .ok()
                    })
                })
                .filter(|&n| n < max_len)
                .collect();

            if numbers.is_empty() {
                return Err(agent_mem_traits::AgentMemError::internal_error(
                    "无法解析重排序结果".to_string(),
                ));
            }

            Ok(numbers)
        } else {
            Ok(numbers)
        }
    }
}
