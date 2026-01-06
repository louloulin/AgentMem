//! Memory routes utility functions
//!
//! 提供路由处理中使用的辅助函数，包括：
//! - 字符串处理
//! - 评分计算
//! - 查询检测
//! - 数据转换

use agent_mem_traits::MemoryItem;
use regex::Regex;

/// 安全地截取字符串到指定字符数（使用字符边界，避免UTF-8 panic）
///
/// 这个函数确保在字符边界处截取，而不是字节边界，避免在多字节UTF-8字符中间切片
///
/// # 性能说明
/// 使用 `chars().take()` 直接截取，只遍历需要的字符，对长字符串高效
pub fn truncate_string_at_char_boundary(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// 检测文本是否包含中文字符
pub fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c as u32,
            0x4E00..=0x9FFF |  // CJK统一汉字
            0x3400..=0x4DBF |  // CJK扩展A
            0x20000..=0x2A6DF | // CJK扩展B
            0x2A700..=0x2B73F | // CJK扩展C
            0x2B740..=0x2B81F | // CJK扩展D
            0x2B820..=0x2CEAF   // CJK扩展E
        )
    })
}

/// 计算Recency评分（基于最后访问时间的指数衰减）
/// 
/// 使用指数衰减模型：recency = exp(-decay * hours_since_access)
/// - 最近访问的记忆得分接近1.0
/// - 随着时间推移，得分指数级衰减
/// 
/// # 参数
/// - `last_accessed_at`: 最后访问时间（ISO 8601字符串）
/// - `recency_decay`: 衰减系数（默认0.1，表示每小时衰减约10%）
/// 
/// # 返回
/// Recency评分（0.0到1.0之间）
pub fn calculate_recency_score(last_accessed_at: &str, recency_decay: f64) -> f64 {
    use chrono::{DateTime, Utc};
    
    // 解析最后访问时间
    let last_accessed = if let Ok(dt) = DateTime::parse_from_rfc3339(last_accessed_at) {
        dt.with_timezone(&Utc)
    } else if let Ok(dt) = last_accessed_at.parse::<DateTime<Utc>>() {
        dt
    } else {
        // 如果解析失败，返回默认值（假设是最近访问的）
        return 1.0;
    };
    
    // 计算距离现在的小时数
    let now = Utc::now();
    let hours_since_access = (now - last_accessed).num_seconds() as f64 / 3600.0;
    
    // 指数衰减：exp(-decay * hours)
    let recency = (-recency_decay * hours_since_access.max(0.0)).exp();
    
    // 确保结果在[0.0, 1.0]范围内
    recency.max(0.0).min(1.0)
}

/// 计算三维检索综合评分（Recency × Importance × Relevance）
/// 
/// 基于Generative Agents论文的三维检索模型
pub fn calculate_3d_score(
    relevance: f32,
    importance: f32,
    last_accessed_at: &str,
    recency_decay: f64,
) -> f64 {
    let recency = calculate_recency_score(last_accessed_at, recency_decay);
    let importance_clamped = importance.max(0.0).min(1.0) as f64;
    let relevance_clamped = relevance.max(0.0).min(1.0) as f64;
    
    let composite_score = recency * importance_clamped * relevance_clamped;
    composite_score.max(0.0).min(1.0)
}

/// 计算搜索结果质量评分
/// 
/// 基于内容质量、完整性和元数据丰富度评估搜索结果的质量
pub fn calculate_quality_score(item: &MemoryItem) -> f64 {
    let mut quality_score = 0.0;
    let mut weight_sum = 0.0;
    
    // 1. 内容长度评分（理想长度：50-500字符）
    let content_len = item.content.len();
    let length_score = if content_len < 10 {
        0.2
    } else if content_len < 50 {
        0.5
    } else if content_len <= 500 {
        1.0
    } else if content_len <= 2000 {
        0.8
    } else {
        0.6
    };
    quality_score += length_score * 0.3;
    weight_sum += 0.3;
    
    // 2. 元数据丰富度评分
    let metadata_score = if item.metadata.is_empty() {
        0.3
    } else if item.metadata.len() < 3 {
        0.6
    } else if item.metadata.len() <= 10 {
        1.0
    } else {
        0.9
    };
    quality_score += metadata_score * 0.2;
    weight_sum += 0.2;
    
    // 3. 内容完整性评分（是否有hash）
    let completeness_score = if let Some(hash) = &item.hash {
        if hash.is_empty() {
            0.5
        } else {
            1.0
        }
    } else {
        0.5
    };
    quality_score += completeness_score * 0.2;
    weight_sum += 0.2;
    
    // 4. 访问历史评分
    let access_score = if item.access_count > 0 {
        (item.access_count.min(100) as f64 / 100.0).min(1.0)
    } else {
        0.5
    };
    quality_score += access_score * 0.15;
    weight_sum += 0.15;
    
    // 5. 重要性评分
    let importance_score = item.importance.max(0.0).min(1.0) as f64;
    quality_score += importance_score * 0.15;
    weight_sum += 0.15;
    
    // 归一化
    if weight_sum > 0.0 {
        quality_score / weight_sum
    } else {
        0.5
    }
}

/// 智能阈值计算：根据查询类型动态调整阈值
/// 
/// 增强：添加中文检测，为中文查询降低阈值以提高召回率
pub fn get_adaptive_threshold(query: &str) -> f32 {
    // 检测中文查询，降低阈值
    let has_chinese = contains_chinese(query);
    let chinese_adjustment = if has_chinese { -0.2 } else { 0.0 };

    // 检测商品ID格式: P + 6位数字
    if let Ok(pattern) = Regex::new(r"^P\d{6}$") {
        if pattern.is_match(query) {
            return 0.1;
        }
    }

    // 检测UUID格式
    if query.len() == 36 && query.matches('-').count() == 4 {
        return 0.1;
    }

    // 检测其他精确ID格式
    if query.len() < 20
        && !query.contains(' ')
        && query.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return 0.2;
    }

    // 短查询（< 5字符）
    if query.len() < 5 {
        return (0.3f32 + chinese_adjustment).max(0.1f32);
    }

    // 包含商品相关关键词
    let lower_query = query.to_lowercase();
    if lower_query.contains("商品")
        || lower_query.contains("订单")
        || lower_query.contains("id")
        || lower_query.contains("product")
    {
        return (0.4f32 + chinese_adjustment).max(0.2f32);
    }

    // 根据长度调整
    let query_len = query.len();
    let base_threshold = if query_len < 20 {
        0.3f32
    } else if query_len < 50 {
        0.5f32
    } else {
        0.7f32
    };
    
    (base_threshold + chinese_adjustment).max(0.1f32).min(0.9f32)
}

/// 检测是否是精确查询（商品ID、SKU等）
pub fn detect_exact_query(query: &str) -> bool {
    // 商品ID格式：P + 6位数字
    if let Ok(pattern) = Regex::new(r"^P\d{6}$") {
        if pattern.is_match(query) {
            return true;
        }
    }

    // 其他精确ID格式（全字母数字，无空格，长度< 20）
    query.len() < 20
        && !query.contains(' ')
        && query.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// 转换MemoryItem为JSON
pub fn convert_memory_to_json(item: MemoryItem) -> serde_json::Value {
    serde_json::json!({
        "id": item.id,
        "agent_id": item.agent_id,
        "user_id": item.user_id,
        "content": item.content,
        "memory_type": item.memory_type,
        "importance": item.importance,
        "created_at": item.created_at,
        "last_accessed_at": item.last_accessed_at,
        "access_count": item.access_count,
        "metadata": item.metadata,
        "score": item.score,
        "hash": item.hash,
    })
}

/// 计算访问模式评分（用于预取候选选择）
pub fn calculate_access_pattern_score(access_count: i64, last_accessed_ts: Option<i64>) -> f64 {
    use chrono::Utc;
    
    let count_score = (access_count.min(100) as f64 / 100.0).min(1.0);
    
    let recency_score = if let Some(ts) = last_accessed_ts {
        let last_accessed = chrono::DateTime::from_timestamp(ts, 0)
            .unwrap_or_else(|| Utc::now());
        let hours_ago = (Utc::now() - last_accessed).num_hours() as f64;
        (-0.1 * hours_ago.max(0.0)).exp()
    } else {
        0.5
    };
    
    // 综合评分：访问次数权重0.6，时间权重0.4
    count_score * 0.6 + recency_score * 0.4
}

/// 计算自动重要性（基于访问模式）
/// 
/// 根据访问频率和最近访问时间自动调整importance
/// 公式：new_importance = base_importance + access_bonus + recency_bonus
pub fn calculate_auto_importance(
    current_importance: f64,
    access_count: i64,
    last_accessed_ts: Option<i64>,
) -> f32 {
    use chrono::Utc;
    
    let base_importance = current_importance.max(0.0).min(1.0) as f32;
    
    // 访问频率奖励（对数增长，避免过度增长）
    let access_bonus = if access_count > 0 {
        (access_count as f32).ln() / 10.0 // 对数增长，最大约0.7
    } else {
        0.0
    };
    
    // 最近访问奖励（指数衰减）
    let recency_bonus = if let Some(ts) = last_accessed_ts {
        let hours_since_access = (Utc::now().timestamp() - ts) as f64 / 3600.0;
        // 最近24小时内访问，给予奖励
        if hours_since_access < 24.0 {
            (1.0 - hours_since_access / 24.0) * 0.1 // 最多0.1的奖励
        } else {
            0.0
        }
    } else {
        0.0
    };
    
    // 计算新的importance（限制在[0.0, 1.0]范围内）
    let new_importance = (base_importance + access_bonus + recency_bonus as f32)
        .max(0.0)
        .min(1.0);
    
    new_importance
}

/// 应用分层排序（基于scope和level）
/// 
/// 基于scope字段对搜索结果进行层次排序，优先返回最具体scope的结果
/// 层次顺序（从最具体到最抽象）：run -> session -> agent -> user -> organization -> global
pub fn apply_hierarchical_sorting(mut items: Vec<MemoryItem>) -> Vec<MemoryItem> {
    // Scope层次映射（数字越小越具体，优先级越高）
    let scope_level = |scope: &str| -> usize {
        match scope {
            "run" => 0,
            "session" => 1,
            "agent" => 2,
            "user" => 3,
            "organization" => 4,
            "global" => 5,
            _ => 6, // 未知scope放在最后
        }
    };
    
    // 按scope层次和重要性排序
    items.sort_by(|a, b| {
        let scope_a = a.metadata
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("global");
        let scope_b = b.metadata
            .get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("global");
        
        let level_a = scope_level(scope_a);
        let level_b = scope_level(scope_b);
        
        // 首先按scope层次排序（level越小越具体，优先级越高）
        match level_a.cmp(&level_b) {
            std::cmp::Ordering::Equal => {
                // 相同层次时，按重要性排序（重要性高的在前）
                b.importance.partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
            other => other,
        }
    });
    
    items
}

/// 应用智能过滤（基于时间范围和重要性阈值）
/// 
/// 基于时间范围和重要性阈值对搜索结果进行过滤
/// - min_importance: 最小重要性阈值（默认0.0，不过滤）
/// - max_age_days: 最大年龄（天数，默认不过滤）
/// - min_access_count: 最小访问次数（默认0，不过滤）
pub fn apply_intelligent_filtering(
    items: Vec<MemoryItem>,
    min_importance: Option<f32>,
    max_age_days: Option<u64>,
    min_access_count: Option<i64>,
) -> Vec<MemoryItem> {
    use chrono::Utc;
    
    let min_importance = min_importance.unwrap_or(0.0);
    let min_access_count = min_access_count.unwrap_or(0) as u32;
    let now = Utc::now();
    
    items
        .into_iter()
        .filter(|item| {
            // 重要性过滤
            if item.importance < min_importance {
                return false;
            }
            
            // 访问次数过滤
            if item.access_count < min_access_count {
                return false;
            }
            
            // 年龄过滤
            if let Some(max_age) = max_age_days {
                let age_days = (now - item.created_at).num_days() as u64;
                if age_days > max_age {
                    return false;
                }
            }
            
            true
        })
        .collect()
}

/// 计算预取候选（基于访问模式评分）
pub fn compute_prefetch_candidates(
    rows: Vec<(String, i64, Option<i64>)>,
    limit: usize,
) -> Vec<String> {
    let mut scored: Vec<(String, f64)> = rows
        .into_iter()
        .map(|(id, count, ts)| (id, calculate_access_pattern_score(count, ts)))
        .collect();

    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    scored
        .into_iter()
        .take(limit)
        .map(|(id, _)| id)
        .collect()
}
