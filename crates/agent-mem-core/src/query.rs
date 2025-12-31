//! Query Abstraction - 替代String查询
//!
//! V4.0 Query系统：将原始字符串查询抽象为结构化的Query对象

use crate::types::{AttributeKey, AttributeValue};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Query ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryId(String);

impl QueryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询抽象（替代String查询）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub id: QueryId,
    pub intent: QueryIntent,
    pub constraints: Vec<Constraint>,
    pub preferences: Vec<Preference>,
    pub context: QueryContext,
}

/// 查询意图（自动推断）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryIntent {
    /// ID精确查找
    Lookup { entity_id: String },
    /// 语义搜索
    SemanticSearch {
        text: String,
        semantic_vector: Option<Vec<f32>>,
    },
    /// 关系查询
    RelationQuery { source: String, relation: String },
    /// 聚合查询
    Aggregation { op: AggregationOp },
    /// 全文搜索
    FullTextSearch { keywords: Vec<String> },
}

/// 聚合操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationOp {
    Count,
    Sum(String), // field name
    Avg(String),
    Max(String),
    Min(String),
}

/// 约束条件（替代固定Scope）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// 属性精确匹配
    AttributeMatch {
        key: AttributeKey,
        value: AttributeValue,
    },
    /// 属性范围查询
    AttributeRange {
        key: AttributeKey,
        min: f64,
        max: f64,
    },
    /// 时间范围
    TimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    /// 结果数量限制
    Limit(usize),
    /// 最小分数
    MinScore(f32),
    /// 包含文本
    ContainsText(String),
    /// 正则匹配
    Regex { field: String, pattern: String },
}

/// 偏好（软约束）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Preference {
    PreferRecent { weight: f32 },
    PreferImportant { weight: f32 },
    PreferType { memory_type: String, weight: f32 },
    PreferAttribute { key: AttributeKey, weight: f32 },
}

/// 查询上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct QueryContext {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
}


/// 查询特征（用于意图推断）
#[derive(Debug, Clone)]
pub struct QueryFeatures {
    pub has_id_pattern: bool,
    pub has_attribute_filter: bool,
    pub has_relation_query: bool,
    pub has_keywords: bool,
    pub complexity: QueryComplexity,
    pub language: Option<String>,
}

/// 查询复杂度
#[derive(Debug, Clone, PartialEq)]
pub enum QueryComplexity {
    Simple,  // 单个关键词
    Medium,  // 多个关键词或简单条件
    Complex, // 复杂逻辑或多约束
}

impl Query {
    /// 构建器模式
    pub fn builder() -> QueryBuilder {
        QueryBuilder::new()
    }

    /// 从字符串自动构建Query
    pub fn from_string(s: &str) -> Self {
        let features = Self::extract_features(s);

        Query {
            id: QueryId::new(),
            intent: Self::infer_intent(s, &features),
            constraints: Self::extract_constraints(s, &features),
            preferences: vec![],
            context: QueryContext::default(),
        }
    }

    /// 提取查询特征
    fn extract_features(s: &str) -> QueryFeatures {
        // ID模式检测 (如: U123456, M789012)
        let has_id_pattern =
            s.contains(char::is_uppercase) && s.chars().filter(|c| c.is_numeric()).count() >= 6;

        // 属性过滤检测 (如: user::name=john)
        let has_attribute_filter = s.contains("::") || s.contains('=');

        // 关系查询检测 (如: memory1->related_to->memory2)
        let has_relation_query = s.contains("->") || s.contains("related");

        // 关键词检测
        let has_keywords = s.split_whitespace().count() > 1;

        // 复杂度估算
        let complexity = if s.len() < 10 && s.split_whitespace().count() == 1 {
            QueryComplexity::Simple
        } else if s.contains('&') || s.contains('|') || has_attribute_filter {
            QueryComplexity::Complex
        } else {
            QueryComplexity::Medium
        };

        QueryFeatures {
            has_id_pattern,
            has_attribute_filter,
            has_relation_query,
            has_keywords,
            complexity,
            language: None, // 可扩展语言检测
        }
    }

    /// 推断查询意图
    fn infer_intent(s: &str, features: &QueryFeatures) -> QueryIntent {
        if features.has_id_pattern {
            // ID查找
            let entity_id = s
                .split_whitespace()
                .find(|w| w.len() > 6 && w.chars().any(|c| c.is_uppercase()))
                .unwrap_or(s)
                .to_string();
            QueryIntent::Lookup { entity_id }
        } else if features.has_relation_query {
            // 关系查询
            let parts: Vec<&str> = s.split("->").collect();
            QueryIntent::RelationQuery {
                source: parts.first().unwrap_or(&"").to_string(),
                relation: parts.get(1).unwrap_or(&"").to_string(),
            }
        } else if s.starts_with("count") || s.starts_with("sum") {
            // 聚合查询
            QueryIntent::Aggregation {
                op: AggregationOp::Count,
            }
        } else {
            // 默认语义搜索
            QueryIntent::SemanticSearch {
                text: s.to_string(),
                semantic_vector: None,
            }
        }
    }

    /// 提取约束条件
    fn extract_constraints(s: &str, features: &QueryFeatures) -> Vec<Constraint> {
        let mut constraints = vec![];

        // 提取属性匹配
        if features.has_attribute_filter {
            // 简单解析 key::value 或 key=value
            for part in s.split_whitespace() {
                if part.contains("::") {
                    let kv: Vec<&str> = part.split("::").collect();
                    if kv.len() == 2 {
                        constraints.push(Constraint::AttributeMatch {
                            key: AttributeKey::system(kv[0]),
                            value: AttributeValue::String(kv[1].to_string()),
                        });
                    }
                } else if part.contains('=') {
                    let kv: Vec<&str> = part.split('=').collect();
                    if kv.len() == 2 {
                        constraints.push(Constraint::AttributeMatch {
                            key: AttributeKey::system(kv[0]),
                            value: AttributeValue::String(kv[1].to_string()),
                        });
                    }
                }
            }
        }

        // 默认限制100条结果
        if !constraints
            .iter()
            .any(|c| matches!(c, Constraint::Limit(_)))
        {
            constraints.push(Constraint::Limit(100));
        }

        constraints
    }

    /// 添加约束
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// 添加偏好
    pub fn with_preference(mut self, preference: Preference) -> Self {
        self.preferences.push(preference);
        self
    }

    /// 设置上下文
    pub fn with_context(mut self, context: QueryContext) -> Self {
        self.context = context;
        self
    }
}

/// Query构建器
pub struct QueryBuilder {
    intent: Option<QueryIntent>,
    constraints: Vec<Constraint>,
    preferences: Vec<Preference>,
    context: QueryContext,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            intent: None,
            constraints: vec![],
            preferences: vec![],
            context: QueryContext::default(),
        }
    }

    pub fn intent(mut self, intent: QueryIntent) -> Self {
        self.intent = Some(intent);
        self
    }

    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn preference(mut self, preference: Preference) -> Self {
        self.preferences.push(preference);
        self
    }

    pub fn context(mut self, context: QueryContext) -> Self {
        self.context = context;
        self
    }

    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.context.user_id = Some(user_id.into());
        self
    }

    pub fn agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.context.agent_id = Some(agent_id.into());
        self
    }

    pub fn build(self) -> Query {
        Query {
            id: QueryId::new(),
            intent: self.intent.unwrap_or(QueryIntent::SemanticSearch {
                text: String::new(),
                semantic_vector: None,
            }),
            constraints: self.constraints,
            preferences: self.preferences,
            context: self.context,
        }
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_from_string_simple() {
        let query = Query::from_string("hello world");
        assert!(matches!(query.intent, QueryIntent::SemanticSearch { .. }));
    }

    #[test]
    fn test_query_from_string_with_id() {
        let query = Query::from_string("find U123456");
        assert!(matches!(query.intent, QueryIntent::Lookup { .. }));
    }

    #[test]
    fn test_query_builder() {
        let query = Query::builder()
            .intent(QueryIntent::SemanticSearch {
                text: "test".to_string(),
                semantic_vector: None,
            })
            .constraint(Constraint::Limit(10))
            .user_id("user1")
            .build();

        assert_eq!(query.constraints.len(), 1);
        assert_eq!(query.context.user_id, Some("user1".to_string()));
    }

    #[test]
    fn test_query_with_attribute_filter() {
        let query = Query::from_string("search user::name=john");
        assert!(!query.constraints.is_empty());
    }
}
