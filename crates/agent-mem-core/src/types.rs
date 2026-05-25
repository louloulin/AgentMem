//! Core memory types and data structures

use agent_mem_traits::{AgentMemError, MemoryItem, Result, Vector};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Cognitive memory type classification (8 types for AgentMem 7.0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryType {
    // Basic cognitive memories (existing)
    /// Episodic memories - specific events and experiences with temporal context
    Episodic,
    /// Semantic memories - facts, concepts, and general knowledge
    Semantic,
    /// Procedural memories - skills, procedures, and how-to knowledge
    Procedural,
    /// Working memories - temporary information processing and active context
    Working,

    // Advanced cognitive memories (new in AgentMem 7.0)
    /// Core memories - persistent identity, preferences, and fundamental beliefs
    Core,
    /// Resource memories - multimedia content, documents, and external resources
    Resource,
    /// Knowledge memories - structured knowledge graphs and domain expertise
    Knowledge,
    /// Contextual memories - environment-aware and situation-specific information
    Contextual,
}

impl MemoryType {
    /// Convert memory type to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            // Basic cognitive memories
            MemoryType::Episodic => "episodic",
            MemoryType::Semantic => "semantic",
            MemoryType::Procedural => "procedural",
            MemoryType::Working => "working",
            // Advanced cognitive memories (AgentMem 7.0)
            MemoryType::Core => "core",
            MemoryType::Resource => "resource",
            MemoryType::Knowledge => "knowledge",
            MemoryType::Contextual => "contextual",
        }
    }

    /// Parse memory type from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // Basic cognitive memories
            "episodic" => Some(MemoryType::Episodic),
            "semantic" => Some(MemoryType::Semantic),
            "procedural" => Some(MemoryType::Procedural),
            "working" => Some(MemoryType::Working),
            // Advanced cognitive memories (AgentMem 7.0)
            "core" => Some(MemoryType::Core),
            "resource" => Some(MemoryType::Resource),
            "knowledge" => Some(MemoryType::Knowledge),
            "contextual" => Some(MemoryType::Contextual),
            _ => None,
        }
    }

    /// Get all available memory types
    pub fn all_types() -> Vec<Self> {
        vec![
            MemoryType::Episodic,
            MemoryType::Semantic,
            MemoryType::Procedural,
            MemoryType::Working,
            MemoryType::Core,
            MemoryType::Resource,
            MemoryType::Knowledge,
            MemoryType::Contextual,
        ]
    }

    /// Check if this is a basic cognitive memory type
    pub fn is_basic_type(&self) -> bool {
        matches!(
            self,
            MemoryType::Episodic
                | MemoryType::Semantic
                | MemoryType::Procedural
                | MemoryType::Working
        )
    }

    /// Check if this is an advanced cognitive memory type (AgentMem 7.0)
    pub fn is_advanced_type(&self) -> bool {
        matches!(
            self,
            MemoryType::Core
                | MemoryType::Resource
                | MemoryType::Knowledge
                | MemoryType::Contextual
        )
    }

    /// Get the description of the memory type
    pub fn description(&self) -> &'static str {
        match self {
            MemoryType::Episodic => "Specific events and experiences with temporal context",
            MemoryType::Semantic => "Facts, concepts, and general knowledge",
            MemoryType::Procedural => "Skills, procedures, and how-to knowledge",
            MemoryType::Working => "Temporary information processing and active context",
            MemoryType::Core => "Persistent identity, preferences, and fundamental beliefs",
            MemoryType::Resource => "Multimedia content, documents, and external resources",
            MemoryType::Knowledge => "Structured knowledge graphs and domain expertise",
            MemoryType::Contextual => "Environment-aware and situation-specific information",
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MemoryType {
    type Err = AgentMemError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "episodic" => Ok(MemoryType::Episodic),
            "semantic" => Ok(MemoryType::Semantic),
            "procedural" => Ok(MemoryType::Procedural),
            "working" => Ok(MemoryType::Working),
            "core" => Ok(MemoryType::Core),
            "resource" => Ok(MemoryType::Resource),
            "knowledge" => Ok(MemoryType::Knowledge),
            "contextual" => Ok(MemoryType::Contextual),
            _ => Err(AgentMemError::validation_error(format!(
                "Unknown memory type: {s}"
            ))),
        }
    }
}

/// Memory importance level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum ImportanceLevel {
    /// Low importance (score < 0.4)
    Low = 1,
    /// Medium importance (0.4 <= score < 0.6)
    Medium = 2,
    /// High importance (0.6 <= score < 0.8)
    High = 3,
    /// Critical importance (score >= 0.8)
    Critical = 4,
}

impl ImportanceLevel {
    /// Convert a numeric score to an importance level
    pub fn from_score(score: f32) -> Self {
        if score >= 0.8 {
            ImportanceLevel::Critical
        } else if score >= 0.6 {
            ImportanceLevel::High
        } else if score >= 0.4 {
            ImportanceLevel::Medium
        } else {
            ImportanceLevel::Low
        }
    }

    /// Convert importance level to a numeric score
    pub fn to_score(&self) -> f32 {
        match self {
            ImportanceLevel::Low => 0.25,
            ImportanceLevel::Medium => 0.5,
            ImportanceLevel::High => 0.75,
            ImportanceLevel::Critical => 1.0,
        }
    }
}

// ========== 🆕 V4.0 新架构 ==========

/// 多模态内容类型（支持文本、图像、音频、视频等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    /// 文本内容
    Text(String),
    /// 图像内容（URL + 可选描述）
    Image {
        url: String,
        caption: Option<String>,
    },
    /// 音频内容（URL + 可选转录文本）
    Audio {
        url: String,
        transcript: Option<String>,
    },
    /// 视频内容（URL + 可选摘要）
    Video {
        url: String,
        summary: Option<String>,
    },
    /// 结构化数据（JSON）
    Structured(serde_json::Value),
    /// 混合内容（多种类型组合）
    Mixed(Vec<Content>),
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_text())
    }
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Content::Text(a), Content::Text(b)) => a == b,
            (
                Content::Image {
                    url: u1,
                    caption: c1,
                },
                Content::Image {
                    url: u2,
                    caption: c2,
                },
            ) => u1 == u2 && c1 == c2,
            (
                Content::Audio {
                    url: u1,
                    transcript: t1,
                },
                Content::Audio {
                    url: u2,
                    transcript: t2,
                },
            ) => u1 == u2 && t1 == t2,
            (
                Content::Video {
                    url: u1,
                    summary: s1,
                },
                Content::Video {
                    url: u2,
                    summary: s2,
                },
            ) => u1 == u2 && s1 == s2,
            (Content::Structured(v1), Content::Structured(v2)) => v1 == v2,
            (Content::Mixed(m1), Content::Mixed(m2)) => m1 == m2,
            _ => false,
        }
    }
}

impl Content {
    /// 获取文本表示（用于向后兼容）
    pub fn as_text(&self) -> String {
        match self {
            Content::Text(s) => s.clone(),
            Content::Image { url, caption } => {
                format!(
                    "[Image: {}{}]",
                    url,
                    caption
                        .as_ref()
                        .map(|c| format!(" - {c}"))
                        .unwrap_or_default()
                )
            }
            Content::Audio { url, transcript } => {
                format!(
                    "[Audio: {}{}]",
                    url,
                    transcript
                        .as_ref()
                        .map(|t| format!(" - {t}"))
                        .unwrap_or_default()
                )
            }
            Content::Video { url, summary } => {
                format!(
                    "[Video: {}{}]",
                    url,
                    summary
                        .as_ref()
                        .map(|s| format!(" - {s}"))
                        .unwrap_or_default()
                )
            }
            Content::Structured(v) => {
                serde_json::to_string(v).unwrap_or_else(|_| "[Structured Data]".to_string())
            }
            Content::Mixed(contents) => contents
                .iter()
                .map(|c| c.as_text())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// 属性键（命名空间化，避免冲突）
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeKey {
    /// 命名空间（如：system, user, domain, legacy）
    pub namespace: String,
    /// 属性名
    pub name: String,
}

impl AttributeKey {
    /// 创建新的属性键
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    /// 系统属性（system命名空间）
    pub fn system(name: impl Into<String>) -> Self {
        Self::new("system", name)
    }

    /// 用户属性（user命名空间）
    pub fn user(name: impl Into<String>) -> Self {
        Self::new("user", name)
    }

    /// 核心属性（core命名空间）- 用于agent_id, user_id等核心字段
    pub fn core(name: impl Into<String>) -> Self {
        Self::new("core", name)
    }

    /// 领域属性（domain命名空间）
    pub fn domain(name: impl Into<String>) -> Self {
        Self::new("domain", name)
    }

    /// 旧版属性（legacy命名空间，用于迁移）
    pub fn legacy(name: impl Into<String>) -> Self {
        Self::new("legacy", name)
    }

    // ========== 🆕 标准Scope属性键（替代MemoryScope enum） ==========

    /// 是否为全局scope (system::scope_global = true)
    pub fn scope_global() -> Self {
        Self::system("scope_global")
    }

    /// Agent ID (system::agent_id)
    pub fn agent_id() -> Self {
        Self::system("agent_id")
    }

    /// User ID (system::user_id)
    pub fn user_id() -> Self {
        Self::system("user_id")
    }

    /// Session ID (system::session_id)
    pub fn session_id() -> Self {
        Self::system("session_id")
    }
}

/// 属性值（类型安全）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Array(Vec<AttributeValue>),
    Object(HashMap<String, AttributeValue>),
}

impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String(s) => write!(f, "{s}"),
            AttributeValue::Number(n) => write!(f, "{n}"),
            AttributeValue::Boolean(b) => write!(f, "{b}"),
            AttributeValue::Timestamp(t) => write!(f, "{}", t.to_rfc3339()),
            AttributeValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            AttributeValue::Object(obj) => {
                let items: Vec<String> = obj.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

impl AttributeValue {
    /// 从JSON转换
    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::String(s) => AttributeValue::String(s),
            serde_json::Value::Number(n) => AttributeValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => AttributeValue::Boolean(b),
            serde_json::Value::Array(arr) => {
                AttributeValue::Array(arr.into_iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(obj) => AttributeValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, Self::from_json(v)))
                    .collect(),
            ),
            serde_json::Value::Null => AttributeValue::String("null".to_string()),
        }
    }

    /// 转换为JSON
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            AttributeValue::String(s) => serde_json::Value::String(s.clone()),
            AttributeValue::Number(n) => serde_json::json!(*n),
            AttributeValue::Boolean(b) => serde_json::Value::Bool(*b),
            AttributeValue::Timestamp(dt) => serde_json::Value::String(dt.to_rfc3339()),
            AttributeValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect())
            }
            AttributeValue::Object(obj) => serde_json::Value::Object(
                obj.iter().map(|(k, v)| (k.clone(), v.to_json())).collect(),
            ),
        }
    }

    /// 获取字符串值
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 获取数字值
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// 获取数组
    pub fn as_array(&self) -> Option<&Vec<AttributeValue>> {
        match self {
            AttributeValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// 获取布尔值
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AttributeValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<f64> for AttributeValue {
    fn from(n: f64) -> Self {
        AttributeValue::Number(n)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Boolean(b)
    }
}

/// 属性模式查询（支持通配符、正则、范围）
#[derive(Debug, Clone)]
pub enum AttributePattern {
    /// 精确匹配
    Exact { key: AttributeKey },
    /// 前缀匹配
    Prefix { namespace: String, prefix: String },
    /// 正则匹配
    Regex { namespace: String, pattern: String },
    /// 范围匹配（数值）
    Range {
        key: AttributeKey,
        min: f64,
        max: f64,
    },
}

/// 属性集（完全开放的属性系统）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSet {
    attributes: HashMap<AttributeKey, AttributeValue>,
}

impl AttributeSet {
    /// 创建空的属性集
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// 设置属性
    pub fn set(&mut self, key: AttributeKey, value: AttributeValue) -> Option<AttributeValue> {
        self.attributes.insert(key, value)
    }

    /// 获取属性
    pub fn get(&self, key: &AttributeKey) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }

    /// 删除属性
    pub fn remove(&mut self, key: &AttributeKey) -> Option<AttributeValue> {
        self.attributes.remove(key)
    }

    /// 检查是否包含属性
    pub fn contains(&self, key: &AttributeKey) -> bool {
        self.attributes.contains_key(key)
    }

    /// 获取所有属性
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, AttributeKey, AttributeValue> {
        self.attributes.iter()
    }

    /// 模式查询（支持通配符、正则、范围）
    pub fn query<'a>(
        &'a self,
        pattern: &'a AttributePattern,
    ) -> Vec<(&'a AttributeKey, &'a AttributeValue)> {
        match pattern {
            AttributePattern::Exact { key } => {
                if let Some(value) = self.get(key) {
                    vec![(key, value)]
                } else {
                    vec![]
                }
            }
            AttributePattern::Prefix { namespace, prefix } => self
                .attributes
                .iter()
                .filter(|(k, _)| k.namespace == *namespace && k.name.starts_with(prefix))
                .collect(),
            AttributePattern::Regex { namespace, pattern } => {
                if let Ok(re) = Regex::new(pattern) {
                    self.attributes
                        .iter()
                        .filter(|(k, _)| k.namespace == *namespace && re.is_match(&k.name))
                        .collect()
                } else {
                    vec![]
                }
            }
            AttributePattern::Range { key, min, max } => {
                if let Some(value) = self.get(key) {
                    if let Some(n) = value.as_number() {
                        if n >= *min && n <= *max {
                            return vec![(key, value)];
                        }
                    }
                }
                vec![]
            }
        }
    }

    /// 按命名空间查询
    pub fn query_by_namespace(&self, namespace: &str) -> Vec<(&AttributeKey, &AttributeValue)> {
        self.attributes
            .iter()
            .filter(|(k, _)| k.namespace == namespace)
            .collect()
    }

    // ========== 🆕 Scope辅助方法（替代MemoryScope） ==========

    /// 设置为全局scope
    pub fn set_global_scope(&mut self) {
        self.set(AttributeKey::scope_global(), AttributeValue::Boolean(true));
    }

    /// 设置Agent scope
    pub fn set_agent_scope(&mut self, agent_id: impl Into<String>) {
        self.set(
            AttributeKey::agent_id(),
            AttributeValue::String(agent_id.into()),
        );
    }

    /// 设置User scope
    pub fn set_user_scope(&mut self, agent_id: impl Into<String>, user_id: impl Into<String>) {
        self.set(
            AttributeKey::agent_id(),
            AttributeValue::String(agent_id.into()),
        );
        self.set(
            AttributeKey::user_id(),
            AttributeValue::String(user_id.into()),
        );
    }

    /// 设置Session scope
    pub fn set_session_scope(
        &mut self,
        agent_id: impl Into<String>,
        user_id: impl Into<String>,
        session_id: impl Into<String>,
    ) {
        self.set(
            AttributeKey::agent_id(),
            AttributeValue::String(agent_id.into()),
        );
        self.set(
            AttributeKey::user_id(),
            AttributeValue::String(user_id.into()),
        );
        self.set(
            AttributeKey::session_id(),
            AttributeValue::String(session_id.into()),
        );
    }

    /// 判断是否为全局scope
    pub fn is_global_scope(&self) -> bool {
        self.get(&AttributeKey::scope_global())
            .and_then(|v| v.as_boolean())
            .unwrap_or(false)
    }

    /// 获取Agent ID
    pub fn get_agent_id(&self) -> Option<String> {
        self.get(&AttributeKey::agent_id())
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// 获取User ID
    pub fn get_user_id(&self) -> Option<String> {
        self.get(&AttributeKey::user_id())
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// 获取Session ID
    pub fn get_session_id(&self) -> Option<String> {
        self.get(&AttributeKey::session_id())
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// 推断scope层级（0=Global, 1=Agent, 2=User, 3=Session）
    pub fn infer_scope_level(&self) -> u8 {
        if self.is_global_scope() {
            return 0;
        }

        let has_agent = self.get_agent_id().is_some();
        let has_user = self.get_user_id().is_some();
        let has_session = self.get_session_id().is_some();

        match (has_agent, has_user, has_session) {
            (false, false, false) => 0, // 默认Global
            (true, false, false) => 1,  // Agent
            (true, true, false) => 2,   // User
            (true, true, true) => 3,    // Session
            _ => 0,                     // 其他情况默认Global
        }
    }

    /// 检查是否可以访问另一个AttributeSet的scope
    pub fn can_access(&self, other: &AttributeSet) -> bool {
        let self_level = self.infer_scope_level();
        let other_level = other.infer_scope_level();

        // 更高权限可以访问更低权限
        if self_level < other_level {
            return true;
        }

        // 同级别需要匹配ID
        if self_level == other_level {
            match self_level {
                0 => true, // Global总是可以访问Global
                1 => self.get_agent_id() == other.get_agent_id(),
                2 => {
                    self.get_agent_id() == other.get_agent_id()
                        && self.get_user_id() == other.get_user_id()
                }
                3 => {
                    self.get_agent_id() == other.get_agent_id()
                        && self.get_user_id() == other.get_user_id()
                        && self.get_session_id() == other.get_session_id()
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Default for AttributeSet {
    fn default() -> Self {
        Self::new()
    }
}

// ========== 🆕 从MemoryScope转换（向后兼容） ==========

use crate::hierarchy::MemoryScope;

impl From<MemoryScope> for AttributeSet {
    fn from(scope: MemoryScope) -> Self {
        let mut attrs = AttributeSet::new();

        match scope {
            MemoryScope::Global => {
                attrs.set_global_scope();
            }
            MemoryScope::Agent(agent_id) => {
                attrs.set_agent_scope(agent_id);
            }
            MemoryScope::User { agent_id, user_id } => {
                attrs.set_user_scope(agent_id, user_id);
            }
            MemoryScope::Session {
                agent_id,
                user_id,
                session_id,
            } => {
                attrs.set_session_scope(agent_id, user_id, session_id);
            }
        }

        attrs
    }
}

impl From<&AttributeSet> for MemoryScope {
    fn from(attrs: &AttributeSet) -> Self {
        if attrs.is_global_scope() {
            return MemoryScope::Global;
        }

        let agent_id = attrs.get_agent_id();
        let user_id = attrs.get_user_id();
        let session_id = attrs.get_session_id();

        match (agent_id, user_id, session_id) {
            (Some(aid), Some(uid), Some(sid)) => MemoryScope::Session {
                agent_id: aid,
                user_id: uid,
                session_id: sid,
            },
            (Some(aid), Some(uid), None) => MemoryScope::User {
                agent_id: aid,
                user_id: uid,
            },
            (Some(aid), None, None) => MemoryScope::Agent(aid),
            _ => MemoryScope::Global,
        }
    }
}

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    /// 引用关系
    References,
    /// 替代关系
    Supersedes,
    /// 部分关系
    PartOf,
    /// 相似关系
    SimilarTo,
    /// 因果关系
    CausedBy,
    /// 自定义关系
    Custom(String),
}

/// 关系（记忆间的关系）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// 目标记忆ID
    pub target_id: String,
    /// 关系类型
    pub relation_type: RelationType,
    /// 关系强度（0.0-1.0）
    pub strength: f32,
}

/// 关系图（记忆间的关系网络）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationGraph {
    relations: Vec<Relation>,
}

impl RelationGraph {
    /// 创建空的关系图
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加关系
    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    /// 获取所有关系
    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    /// 根据类型查找关系
    pub fn find_by_type(&self, relation_type: &RelationType) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| {
                std::mem::discriminant(&r.relation_type) == std::mem::discriminant(relation_type)
            })
            .collect()
    }

    /// 查找目标记忆的所有关系
    pub fn find_by_target(&self, target_id: &str) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| r.target_id == target_id)
            .collect()
    }
}

/// 系统元信息 (保持与V4 Metadata兼容)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64, // 统一使用access_count，与V4一致
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            access_count: 0,
            last_accessed: None,
        }
    }
}

impl Metadata {
    /// 转换为HashMap用于向后兼容
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("created_at".to_string(), self.created_at.to_rfc3339());
        map.insert("updated_at".to_string(), self.updated_at.to_rfc3339());
        if let Some(last_accessed) = self.last_accessed {
            map.insert("last_accessed".to_string(), last_accessed.to_rfc3339());
        }
        map.insert("access_count".to_string(), self.access_count.to_string());
        map
    }
}

/// 🆕 V4.0 Memory结构（完全抽象化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// 记忆ID
    pub id: String,
    /// 内容（多模态）
    pub content: Content,
    /// 属性（完全开放）
    pub attributes: AttributeSet,
    /// 关系网络
    pub relations: RelationGraph,
    /// 系统元信息
    pub metadata: Metadata,
}

impl Memory {
    /// 创建构建器
    pub fn builder() -> MemoryBuilder {
        MemoryBuilder::new()
    }

    /// 便捷方法：创建新的Memory（向后兼容）
    pub fn new(
        agent_id: String,
        user_id: Option<String>,
        memory_type: MemoryType,
        content: String,
        importance: f32,
    ) -> Self {
        let mut builder = MemoryBuilder::new().content(Content::Text(content));

        // 🔑 关键修复: 使用core属性以匹配memory_to_db的读取逻辑
        builder.attributes.set(
            AttributeKey::core("agent_id"),
            AttributeValue::String(agent_id),
        );

        if let Some(uid) = user_id {
            builder
                .attributes
                .set(AttributeKey::core("user_id"), AttributeValue::String(uid));
        }

        builder.attributes.set(
            AttributeKey::core("memory_type"),
            AttributeValue::String(memory_type.as_str().to_string()),
        );

        builder.attributes.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(importance as f64),
        );

        builder.build()
    }

    /// 获取importance（向后兼容）
    pub fn importance(&self) -> f32 {
        self.attributes
            .get(&AttributeKey::system("importance"))
            .and_then(|v| v.as_number())
            .unwrap_or(0.0) as f32
    }

    /// 获取score（检索相关性分数，向后兼容）
    pub fn score(&self) -> Option<f64> {
        // 优先从score属性获取
        if let Some(score) = self
            .attributes
            .get(&AttributeKey::system("score"))
            .and_then(|v| v.as_number())
        {
            return Some(score);
        }
        // 如果没有score，使用importance作为fallback
        self.attributes
            .get(&AttributeKey::system("importance"))
            .and_then(|v| v.as_number())
    }

    /// 获取agent_id（向后兼容）
    pub fn agent_id(&self) -> String {
        self.attributes
            .get(&AttributeKey::system("agent_id"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    /// 获取user_id（向后兼容）
    pub fn user_id(&self) -> Option<String> {
        self.attributes
            .get(&AttributeKey::system("user_id"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
    }

    /// 获取version（向后兼容）
    pub fn version(&self) -> u32 {
        self.attributes
            .get(&AttributeKey::system("version"))
            .and_then(|v| v.as_number())
            .unwrap_or(1.0) as u32
    }

    /// 获取memory_type（向后兼容）
    pub fn memory_type(&self) -> MemoryType {
        self.attributes
            .get(&AttributeKey::core("memory_type"))
            .and_then(|v| v.as_string())
            .and_then(|s| s.parse::<MemoryType>().ok())
            .unwrap_or(MemoryType::Semantic)
    }

    /// 获取created_at（向后兼容）
    pub fn created_at(&self) -> i64 {
        self.metadata.created_at.timestamp()
    }

    /// 获取last_accessed_at（向后兼容）
    pub fn last_accessed_at(&self) -> i64 {
        self.metadata
            .last_accessed
            .unwrap_or(self.metadata.updated_at)
            .timestamp()
    }

    /// 更新内容（向后兼容）
    pub fn update_content(&mut self, new_content: String) {
        self.content = Content::Text(new_content);
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// 添加元数据（向后兼容）
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.attributes
            .set(AttributeKey::user(key), AttributeValue::String(value));
    }

    /// 从旧格式迁移（用于数据迁移）
    pub fn from_legacy(old: LegacyMemory) -> Self {
        let mut attributes = AttributeSet::new();

        // 迁移固定字段到属性
        attributes.set(
            AttributeKey::system("agent_id"),
            AttributeValue::String(old.agent_id),
        );

        if let Some(user_id) = old.user_id {
            attributes.set(
                AttributeKey::system("user_id"),
                AttributeValue::String(user_id),
            );
        }

        attributes.set(
            AttributeKey::core("memory_type"),
            AttributeValue::String(old.memory_type.as_str().to_string()),
        );

        attributes.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(old.importance as f64),
        );

        // 迁移嵌入向量
        if let Some(embedding) = old.embedding {
            attributes.set(
                AttributeKey::system("embedding_dimension"),
                AttributeValue::Number(embedding.values.len() as f64),
            );
        }

        // 迁移metadata到legacy命名空间
        for (key, value) in old.metadata {
            attributes.set(AttributeKey::legacy(key), AttributeValue::String(value));
        }

        // 迁移访问信息
        attributes.set(
            AttributeKey::system("access_count"),
            AttributeValue::Number(old.access_count as f64),
        );

        if let Some(expires_at) = old.expires_at {
            attributes.set(
                AttributeKey::system("expires_at"),
                AttributeValue::Number(expires_at as f64),
            );
        }

        attributes.set(
            AttributeKey::system("version"),
            AttributeValue::Number(old.version as f64),
        );

        Self {
            id: old.id,
            content: Content::Text(old.content),
            attributes,
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: chrono::DateTime::from_timestamp(old.created_at, 0)
                    .unwrap_or_else(chrono::Utc::now),
                updated_at: chrono::DateTime::from_timestamp(old.last_accessed_at, 0)
                    .unwrap_or_else(chrono::Utc::now),
                access_count: old.access_count as u64,
                last_accessed: Some(
                    chrono::DateTime::from_timestamp(old.last_accessed_at, 0)
                        .unwrap_or_else(chrono::Utc::now),
                ),
            },
        }
    }

    /// 记录访问
    pub fn access(&mut self) {
        self.metadata.access_count += 1;
        self.metadata.last_accessed = Some(chrono::Utc::now());
        self.metadata.updated_at = chrono::Utc::now();
    }
}

/// Memory构建器
pub struct MemoryBuilder {
    id: Option<String>,
    content: Option<Content>,
    attributes: AttributeSet,
    relations: RelationGraph,
}

impl MemoryBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            content: None,
            attributes: AttributeSet::new(),
            relations: RelationGraph::new(),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn content(mut self, content: impl Into<Content>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content = Some(Content::Text(text.into()));
        self
    }

    pub fn attribute(
        mut self,
        key: impl Into<AttributeKey>,
        value: impl Into<AttributeValue>,
    ) -> Self {
        self.attributes.set(key.into(), value.into());
        self
    }

    pub fn relation(
        mut self,
        target_id: String,
        relation_type: RelationType,
        strength: f32,
    ) -> Self {
        self.relations.add_relation(Relation {
            target_id,
            relation_type,
            strength,
        });
        self
    }

    pub fn build(self) -> Memory {
        Memory {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            content: self.content.unwrap_or_else(|| {
                tracing::warn!("MemoryBuilder: content is required but not set, using empty string");
                Content::Text(String::new())
            }),
            attributes: self.attributes,
            relations: self.relations,
            metadata: Metadata::default(),
        }
    }
}

impl Default for MemoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Content::Text(s)
    }
}

impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Content::Text(s.to_string())
    }
}

impl From<AttributeKey> for String {
    fn from(key: AttributeKey) -> Self {
        format!("{}::{}", key.namespace, key.name)
    }
}

// ========== 🆕 V4.0 Query抽象 ==========

/// 查询意图（自动推断）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryIntent {
    /// ID查询（精确匹配）
    Lookup { entity_id: String },
    /// 语义搜索
    SemanticSearch {
        text: String,
        semantic_vector: Option<Vec<f32>>,
    },
    /// 关系查询
    RelationQuery { source: String, relation: String },
    /// 聚合查询
    Aggregation { operation: AggregationOp },
}

/// 聚合操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationOp {
    Count,
    Sum(String),
    Average(String),
    Max(String),
    Min(String),
}

/// 查询约束（硬性条件，必须满足）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// 属性匹配
    AttributeMatch {
        key: AttributeKey,
        operator: ComparisonOperator,
        value: AttributeValue,
    },
    /// 属性范围
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
    /// 关系约束
    RelationConstraint {
        relation_type: String,
        target: Option<String>,
    },
    /// 结果数量限制
    Limit(usize),
    /// 最小分数
    MinScore(f32),
    /// 逻辑组合
    And(Vec<Constraint>),
    Or(Vec<Constraint>),
    Not(Box<Constraint>),
}

/// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    Matches, // Regex
}

/// 查询偏好（软性要求，影响排序）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub preference_type: PreferenceType,
    pub weight: f32,
}

/// 偏好类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreferenceType {
    /// 时间偏好（新鲜度）
    Temporal(TemporalPreference),
    /// 相关性偏好
    Relevance(RelevancePreference),
    /// 多样性偏好
    Diversity(DiversityPreference),
    /// 重要性偏好
    Importance { min_importance: f32 },
}

/// 时间偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalPreference {
    /// 偏好最近的记忆
    Recent { within_days: u32 },
    /// 偏好特定时间段
    TimeWindow {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    /// 偏好访问频繁的
    FrequentlyAccessed,
}

/// 相关性偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelevancePreference {
    /// 语义相关性
    Semantic { threshold: f32 },
    /// 关系相关性
    Relational { max_hops: usize },
}

/// 多样性偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiversityPreference {
    /// 类型多样性
    TypeDiversity,
    /// 来源多样性
    SourceDiversity,
    /// 最大最小相关性
    MaxMarginalRelevance { lambda: f32 },
}

/// 查询上下文
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryContext {
    /// 当前会话信息
    pub session_info: Option<HashMap<String, String>>,
    /// 用户上下文
    pub user_context: Option<HashMap<String, String>>,
    /// 历史查询
    pub query_history: Vec<String>,
    /// 额外元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 🆕 V4.0 Query结构（完全抽象化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// 查询ID
    pub id: String,
    /// 查询意图
    pub intent: QueryIntent,
    /// 约束条件（必须满足）
    pub constraints: Vec<Constraint>,
    /// 偏好（影响排序）
    pub preferences: Vec<Preference>,
    /// 查询上下文
    pub context: QueryContext,
}

impl Query {
    /// 创建构建器
    pub fn builder() -> QueryBuilder {
        QueryBuilder::new()
    }

    /// 从字符串自动构建Query（智能推断）
    pub fn from_string(s: &str) -> Self {
        let features = QueryFeatures::extract(s);

        Query {
            id: Uuid::new_v4().to_string(),
            intent: features.infer_intent(s),
            constraints: features.extract_constraints(),
            preferences: vec![],
            context: QueryContext::default(),
        }
    }
}

/// 查询特征提取
struct QueryFeatures {
    has_id_pattern: bool,
    has_attribute_filter: bool,
    has_relation_query: bool,
    complexity: QueryComplexity,
}

#[derive(Debug, Clone)]
enum QueryComplexity {
    Simple,  // 单一条件
    Medium,  // 2-3个条件
    Complex, // 4+个条件
}

impl QueryFeatures {
    fn extract(s: &str) -> Self {
        // Safe: regex pattern is a compile-time constant
        let has_id_pattern = Regex::new(r"[A-Z]\d{6}")
            .unwrap_or_else(|e| {
                tracing::error!("Failed to compile ID pattern regex: {e}, using fallback pattern");
                Regex::new(r"\d{6}").unwrap_or_else(|_| {
                    // Empty regex pattern is always valid, but if it somehow fails, use a simple pattern
                    Regex::new(r"^$").unwrap_or_else(|_| {
                        tracing::error!("Failed to create even empty regex pattern, using simple fallback");
                        // This should never fail, but if it does, we'll use a simple pattern
                        Regex::new(r".").unwrap_or_else(|_| {
                            // Last resort: create a regex that matches nothing
                            tracing::error!("Critical: All regex patterns failed, using match-nothing pattern");
                            Regex::new(r"(?!)").unwrap_or_else(|_| {
                                // If even this fails, we have a serious problem
                                tracing::error!("Fatal: Cannot create any regex pattern, aborting");
                                std::process::abort();
                            })
                        })
                    })
                })
            })
            .is_match(s);
        let has_attribute_filter = s.contains("::");
        let has_relation_query = s.contains("->");

        let word_count = s.split_whitespace().count();
        let complexity = if word_count <= 3 {
            QueryComplexity::Simple
        } else if word_count <= 10 {
            QueryComplexity::Medium
        } else {
            QueryComplexity::Complex
        };

        Self {
            has_id_pattern,
            has_attribute_filter,
            has_relation_query,
            complexity,
        }
    }

    fn infer_intent(&self, s: &str) -> QueryIntent {
        if self.has_id_pattern {
            // Extract ID pattern
            // Safe: regex pattern is a compile-time constant
            if let Some(captures) = Regex::new(r"([A-Z]\d{6})")
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to compile ID capture pattern regex: {e}, using fallback pattern");
                    Regex::new(r"(\d{6})").unwrap_or_else(|_| {
                        // Empty regex pattern is always valid, but if it somehow fails, use a simple pattern
                        Regex::new(r"^$").unwrap_or_else(|_| {
                            tracing::error!("Failed to create even empty regex pattern, using simple fallback");
                            // This should never fail, but if it does, we'll use a simple pattern
                            Regex::new(r".").unwrap_or_else(|_| {
                                // Last resort: create a regex that matches nothing
                                tracing::error!("Critical: All regex patterns failed, using match-nothing pattern");
                                Regex::new(r"(?!)").unwrap_or_else(|_| {
                                    // If even this fails, we have a serious problem
                                    tracing::error!("Fatal: Cannot create any regex pattern, aborting");
                                    std::process::abort();
                                })
                            })
                        })
                    })
                })
                .captures(s)
            {
                // Safe: we know the pattern has one capture group
                if let Some(id_match) = captures.get(1) {
                    return QueryIntent::Lookup {
                        entity_id: id_match.as_str().to_string(),
                    };
                }
            }
        }

        if self.has_relation_query {
            let parts: Vec<&str> = s.split("->").collect();
            if parts.len() == 2 {
                return QueryIntent::RelationQuery {
                    source: parts[0].trim().to_string(),
                    relation: parts[1].trim().to_string(),
                };
            }
        }

        // Default: Semantic search
        QueryIntent::SemanticSearch {
            text: s.to_string(),
            semantic_vector: None,
        }
    }

    fn extract_constraints(&self) -> Vec<Constraint> {
        let mut constraints = vec![];

        // Default limit
        constraints.push(Constraint::Limit(100));

        constraints
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

    /// 设置文本查询
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.intent = Some(QueryIntent::SemanticSearch {
            text: text.into(),
            semantic_vector: None,
        });
        self
    }

    /// 设置ID查询
    pub fn lookup(mut self, entity_id: impl Into<String>) -> Self {
        self.intent = Some(QueryIntent::Lookup {
            entity_id: entity_id.into(),
        });
        self
    }

    /// 添加属性约束
    pub fn with_attribute(
        mut self,
        key: AttributeKey,
        operator: ComparisonOperator,
        value: AttributeValue,
    ) -> Self {
        self.constraints.push(Constraint::AttributeMatch {
            key,
            operator,
            value,
        });
        self
    }

    /// 添加时间范围约束
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.constraints.push(Constraint::TimeRange { start, end });
        self
    }

    /// 设置结果限制
    pub fn limit(mut self, limit: usize) -> Self {
        self.constraints.push(Constraint::Limit(limit));
        self
    }

    /// 添加偏好
    pub fn prefer(mut self, preference_type: PreferenceType, weight: f32) -> Self {
        self.preferences.push(Preference {
            preference_type,
            weight,
        });
        self
    }

    /// 构建Query
    pub fn build(self) -> Query {
        Query {
            id: Uuid::new_v4().to_string(),
            intent: self.intent.unwrap_or_else(|| {
                tracing::warn!("QueryBuilder: intent is required but not set, using default SemanticSearch intent");
                QueryIntent::SemanticSearch {
                    text: String::new(),
                    semantic_vector: None,
                }
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

// ========== 🆕 V4.0 Pipeline框架 ==========

/// Pipeline上下文（在各stage间传递）
#[derive(Debug, Clone, Default)]
pub struct PipelineContext {
    /// 键值对存储
    data: HashMap<String, serde_json::Value>,
}

impl PipelineContext {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> std::result::Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.data.insert(key.into(), json_value);
        Ok(())
    }

    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }
}

/// Pipeline Stage结果
#[derive(Debug)]
pub enum StageResult<T> {
    /// 成功，继续下一个stage
    Continue(T),
    /// 成功，跳过后续stage
    Skip(T),
    /// 失败，中止pipeline
    Abort(String),
}

/// Pipeline Stage trait
#[async_trait::async_trait]
pub trait PipelineStage: Send + Sync {
    type Input: Send;
    type Output: Send;

    /// Stage名称
    fn name(&self) -> &str;

    /// 执行stage
    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>>;

    /// 是否可选（可选stage失败不会中止pipeline）
    fn is_optional(&self) -> bool {
        false
    }
}

/// Pipeline构建器（线性执行）
pub struct Pipeline<I, O> {
    name: String,
    stages: Vec<Box<dyn PipelineStage<Input = I, Output = O>>>,
    error_handler: Option<Box<dyn Fn(&str, &str) + Send + Sync>>,
}

// ========== 🆕 DAG Pipeline支持 ==========

/// DAG节点（Stage包装器）
pub struct DagNode<I, O> {
    pub id: String,
    pub stage: std::sync::Arc<dyn PipelineStage<Input = I, Output = O>>,
    pub dependencies: Vec<String>, // 依赖的节点ID
}

/// 条件函数（用于分支决策）
pub type ConditionFn = Box<dyn Fn(&PipelineContext) -> bool + Send + Sync>;

/// DAG边（带条件）
#[derive(Clone)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>, // context中的条件键
}

/// DAG Pipeline构建器
pub struct DagPipeline<I, O> {
    name: String,
    nodes: HashMap<String, DagNode<I, O>>,
    edges: Vec<DagEdge>,
    conditions: HashMap<String, ConditionFn>,
    entry_nodes: Vec<String>, // 入口节点（无依赖）
    max_parallelism: usize,
    error_handler: Option<std::sync::Arc<dyn Fn(&str, &str) + Send + Sync>>,
}

impl<I: Send + Clone + 'static, O: Send + Clone + 'static> DagPipeline<I, O> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: HashMap::new(),
            edges: vec![],
            conditions: HashMap::new(),
            entry_nodes: vec![],
            max_parallelism: 10,
            error_handler: None,
        }
    }

    /// 添加节点
    pub fn add_node<S>(mut self, id: impl Into<String>, stage: S, dependencies: Vec<String>) -> Self
    where
        S: PipelineStage<Input = I, Output = O> + 'static,
    {
        let id = id.into();
        let node = DagNode {
            id: id.clone(),
            stage: std::sync::Arc::new(stage),
            dependencies: dependencies.clone(),
        };

        self.nodes.insert(id.clone(), node);

        // 如果没有依赖，是入口节点
        if dependencies.is_empty() {
            self.entry_nodes.push(id);
        }

        self
    }

    /// 添加边（带条件）
    pub fn add_edge(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        condition: Option<String>,
    ) -> Self {
        self.edges.push(DagEdge {
            from: from.into(),
            to: to.into(),
            condition,
        });
        self
    }

    /// 添加条件函数
    pub fn add_condition<F>(mut self, name: impl Into<String>, condition: F) -> Self
    where
        F: Fn(&PipelineContext) -> bool + Send + Sync + 'static,
    {
        self.conditions.insert(name.into(), Box::new(condition));
        self
    }

    /// 设置最大并行度
    pub fn with_max_parallelism(mut self, max: usize) -> Self {
        self.max_parallelism = max;
        self
    }

    pub fn with_error_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.error_handler = Some(std::sync::Arc::new(handler));
        self
    }

    /// 执行DAG Pipeline
    pub async fn execute(
        &self,
        input: I,
        context: &mut PipelineContext,
    ) -> anyhow::Result<HashMap<String, O>> {
        use std::sync::Arc;
        use tokio::sync::Mutex;

        // 拓扑排序，检测循环依赖
        let execution_order = self.topological_sort()?;

        // 存储每个节点的执行结果
        let results: Arc<Mutex<HashMap<String, O>>> = Arc::new(Mutex::new(HashMap::new()));
        let context_shared = Arc::new(Mutex::new(context.clone()));

        // 按层级执行（同层级可以并行）
        for level in execution_order {
            let mut handles = vec![];

            for node_id in level {
                let node = self
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| anyhow::anyhow!("Node '{node_id}' not found"))?;

                // 检查边条件
                let should_execute = self
                    .check_edge_conditions(&node_id, &context_shared)
                    .await?;
                if !should_execute {
                    continue;
                }

                let input_clone = input.clone();
                let results_clone = results.clone();
                let context_clone = context_shared.clone();
                let node_name = node.stage.name().to_string();
                let error_handler = self.error_handler.clone();
                let stage_clone = node.stage.clone();
                let is_optional = node.stage.is_optional();

                // 执行节点（并行）
                let handle = tokio::spawn(async move {
                    let mut ctx = context_clone.lock().await;

                    match stage_clone.execute(input_clone, &mut ctx).await {
                        Ok(StageResult::Continue(output)) => {
                            results_clone.lock().await.insert(node_id.clone(), output);
                            Ok(())
                        }
                        Ok(StageResult::Skip(output)) => {
                            results_clone.lock().await.insert(node_id.clone(), output);
                            Ok(())
                        }
                        Ok(StageResult::Abort(reason)) => {
                            if let Some(ref handler) = error_handler {
                                handler(&node_name, &reason);
                            }
                            Err(anyhow::anyhow!("Node '{node_id}' aborted: {reason}"))
                        }
                        Err(e) => {
                            if is_optional {
                                if let Some(ref handler) = error_handler {
                                    handler(&node_name, &e.to_string());
                                }
                                Ok(())
                            } else {
                                Err(anyhow::anyhow!("Node '{node_id}' failed: {e}"))
                            }
                        }
                    }
                });

                handles.push(handle);

                // 控制并行度
                if handles.len() >= self.max_parallelism {
                    for handle in handles.drain(..) {
                        handle.await??;
                    }
                }
            }

            // 等待当前层级所有任务完成
            for handle in handles {
                handle.await??;
            }
        }

        // 更新context
        *context = context_shared.lock().await.clone();

        let final_results = results.lock().await.clone();
        Ok(final_results)
    }

    /// 拓扑排序（Kahn算法）
    fn topological_sort(&self) -> anyhow::Result<Vec<Vec<String>>> {
        use std::collections::{HashMap, HashSet, VecDeque};

        // 计算每个节点的入度
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            adjacency.insert(node_id.clone(), vec![]);
        }

        // 构建邻接表和入度
        for edge in &self.edges {
            if let Some(degree) = in_degree.get_mut(&edge.to) {
                *degree += 1;
            }
            adjacency
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }

        // 也从节点的dependencies构建
        for (node_id, node) in &self.nodes {
            for dep in &node.dependencies {
                if let Some(degree) = in_degree.get_mut(node_id) {
                    *degree += 1;
                }
                adjacency
                    .entry(dep.clone())
                    .or_default()
                    .push(node_id.clone());
            }
        }

        // Kahn算法
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result: Vec<Vec<String>> = vec![];
        let mut visited = HashSet::new();

        while !queue.is_empty() {
            let level_size = queue.len();
            let mut current_level = vec![];

            for _ in 0..level_size {
                if let Some(node_id) = queue.pop_front() {
                    if visited.contains(&node_id) {
                        continue;
                    }

                    visited.insert(node_id.clone());
                    current_level.push(node_id.clone());

                    // 减少后继节点的入度
                    if let Some(neighbors) = adjacency.get(&node_id) {
                        for neighbor in neighbors {
                            if let Some(degree) = in_degree.get_mut(neighbor) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(neighbor.clone());
                                }
                            }
                        }
                    }
                }
            }

            if !current_level.is_empty() {
                result.push(current_level);
            }
        }

        // 检测循环依赖
        if visited.len() != self.nodes.len() {
            return Err(anyhow::anyhow!(
                "Cycle detected in DAG: visited {} nodes, expected {}",
                visited.len(),
                self.nodes.len()
            ));
        }

        Ok(result)
    }

    /// 检查边条件
    async fn check_edge_conditions(
        &self,
        node_id: &str,
        context: &std::sync::Arc<tokio::sync::Mutex<PipelineContext>>,
    ) -> anyhow::Result<bool> {
        // 找到所有指向该节点的边
        let incoming_edges: Vec<&DagEdge> = self.edges.iter().filter(|e| e.to == node_id).collect();

        if incoming_edges.is_empty() {
            return Ok(true); // 无入边，直接执行
        }

        // 检查所有入边的条件
        let ctx = context.lock().await;
        for edge in incoming_edges {
            if let Some(ref condition_name) = edge.condition {
                if let Some(condition_fn) = self.conditions.get(condition_name) {
                    if !condition_fn(&ctx) {
                        return Ok(false); // 条件不满足
                    }
                }
            }
        }

        Ok(true)
    }
}

impl<I: Send + 'static, O: Send + 'static> Pipeline<I, O> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stages: vec![],
            error_handler: None,
        }
    }

    pub fn add_stage<S>(mut self, stage: S) -> Self
    where
        S: PipelineStage<Input = I, Output = O> + 'static,
    {
        self.stages.push(Box::new(stage));
        self
    }

    pub fn with_error_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.error_handler = Some(Box::new(handler));
        self
    }

    pub async fn execute(&self, input: I, context: &mut PipelineContext) -> anyhow::Result<O>
    where
        I: Clone,
        O: Clone + Into<I>,
    {
        let mut current_input = input;
        let mut current_output: Option<O> = None;

        for stage in &self.stages {
            let stage_name = stage.name();

            match stage.execute(current_input.clone(), context).await {
                Ok(StageResult::Continue(output)) => {
                    current_input = output.clone().into();
                    current_output = Some(output);
                }
                Ok(StageResult::Skip(output)) => {
                    current_input = output.clone().into();
                    current_output = Some(output);
                    break;
                }
                Ok(StageResult::Abort(reason)) => {
                    if let Some(ref handler) = self.error_handler {
                        handler(stage_name, &reason);
                    }
                    return Err(anyhow::anyhow!(
                        "Pipeline aborted at stage '{stage_name}': {reason}"
                    ));
                }
                Err(e) => {
                    if stage.is_optional() {
                        if let Some(ref handler) = self.error_handler {
                            handler(stage_name, &e.to_string());
                        }
                        continue;
                    } else {
                        return Err(anyhow::anyhow!(
                            "Pipeline failed at stage '{stage_name}': {e}"
                        ));
                    }
                }
            }
        }

        current_output
            .ok_or_else(|| anyhow::anyhow!("Pipeline completed but no output was produced"))
    }
}

// ========== 🔄 向后兼容：保留旧Memory结构用于迁移 ==========

/// 旧版Memory结构（保留用于数据迁移）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMemory {
    /// Unique memory identifier
    pub id: String,
    /// Agent identifier
    pub agent_id: String,
    /// User identifier (optional)
    pub user_id: Option<String>,
    /// Memory type
    pub memory_type: MemoryType,
    /// Memory content
    pub content: String,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Vector embedding (optional)
    pub embedding: Option<Vector>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last access timestamp
    pub last_accessed_at: i64,
    /// Access count
    pub access_count: u32,
    /// Expiration timestamp (optional)
    pub expires_at: Option<i64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Memory version for conflict resolution
    pub version: u32,
}

impl LegacyMemory {
    /// Create a new memory (旧版API)
    pub fn new(
        agent_id: String,
        user_id: Option<String>,
        memory_type: MemoryType,
        content: String,
        importance: f32,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            user_id,
            memory_type,
            content,
            importance: importance.clamp(0.0, 1.0),
            embedding: None,
            created_at: now,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            metadata: HashMap::new(),
            version: 1,
        }
    }

    /// Record access to this memory
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed_at = chrono::Utc::now().timestamp();
    }

    /// Calculate current importance with decay
    pub fn calculate_current_importance(&self) -> f32 {
        let current_time = chrono::Utc::now().timestamp();
        let time_decay = (current_time - self.created_at) as f32 / (24.0 * 3600.0); // days
        let access_factor = (self.access_count as f32 + 1.0).ln();

        // Apply time decay and access boost
        self.importance * (-time_decay * 0.01).exp() * (1.0 + access_factor * 0.1)
    }

    /// Check if memory is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now().timestamp() > expires_at
        } else {
            false
        }
    }

    /// Set expiration time
    pub fn set_expiration(&mut self, expires_at: i64) {
        self.expires_at = Some(expires_at);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Update content and increment version
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.version += 1;
        self.last_accessed_at = chrono::Utc::now().timestamp();
    }
}

// ========== 🔄 向后兼容From实现 ==========

/// 新Memory → MemoryItem（用于API兼容）
impl From<Memory> for MemoryItem {
    fn from(memory: Memory) -> Self {
        use agent_mem_traits::{MemoryType as TraitMemoryType, Session};

        // Extract system attributes
        let agent_id = memory
            .attributes
            .get(&AttributeKey::system("agent_id"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default".to_string());

        let user_id = memory
            .attributes
            .get(&AttributeKey::system("user_id"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        let memory_type_str = memory
            .attributes
            .get(&AttributeKey::core("memory_type"))
            .and_then(|v| v.as_string())
            .unwrap_or("semantic");

        let importance = memory
            .attributes
            .get(&AttributeKey::system("importance"))
            .and_then(|v| v.as_number())
            .unwrap_or(0.5) as f32;

        // Convert all attributes to metadata
        let metadata: HashMap<String, serde_json::Value> = memory
            .attributes
            .iter()
            .map(|(k, v)| (format!("{}::{}", k.namespace, k.name), v.to_json()))
            .collect();

        // Create session
        let session = Session::new()
            .with_user_id(user_id.clone())
            .with_agent_id(Some(agent_id.clone()));

        // Parse memory type
        let memory_type = match memory_type_str {
            "episodic" => TraitMemoryType::Episodic,
            "semantic" => TraitMemoryType::Semantic,
            "procedural" => TraitMemoryType::Procedural,
            "working" => TraitMemoryType::Working,
            "core" => TraitMemoryType::Core,
            "resource" => TraitMemoryType::Resource,
            "knowledge" => TraitMemoryType::Knowledge,
            "contextual" => TraitMemoryType::Contextual,
            _ => TraitMemoryType::Semantic,
        };

        MemoryItem {
            id: memory.id,
            content: memory.content.as_text(),
            hash: None,
            metadata,
            score: Some(importance),
            created_at: memory.metadata.created_at,
            updated_at: Some(memory.metadata.updated_at),
            session,
            memory_type,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id,
            user_id,
            importance,
            embedding: None,
            last_accessed_at: memory.metadata.last_accessed.unwrap_or_else(Utc::now),
            access_count: memory.metadata.access_count as u32,
            expires_at: memory
                .attributes
                .get(&AttributeKey::system("expires_at"))
                .and_then(|v| v.as_number())
                .map(|ts| DateTime::from_timestamp(ts as i64, 0).unwrap_or_else(Utc::now)),
            version: memory
                .attributes
                .get(&AttributeKey::system("version"))
                .and_then(|v| v.as_number())
                .unwrap_or(1.0) as u32,
        }
    }
}

/// LegacyMemory → MemoryItem（原有实现）
impl From<LegacyMemory> for MemoryItem {
    fn from(memory: LegacyMemory) -> Self {
        use agent_mem_traits::{MemoryType as TraitMemoryType, Session};

        // Convert metadata from String to serde_json::Value
        let metadata: std::collections::HashMap<String, serde_json::Value> = memory
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Create a session from memory data
        let session = Session::new()
            .with_user_id(memory.user_id.clone())
            .with_agent_id(Some(memory.agent_id.clone()));

        MemoryItem {
            id: memory.id,
            content: memory.content,
            hash: None,
            metadata,
            score: Some(memory.importance),
            created_at: DateTime::from_timestamp(memory.created_at, 0).unwrap_or_else(Utc::now),
            updated_at: Some(
                DateTime::from_timestamp(memory.last_accessed_at, 0).unwrap_or_else(Utc::now),
            ),
            session,
            memory_type: match memory.memory_type {
                MemoryType::Episodic => TraitMemoryType::Episodic,
                MemoryType::Semantic => TraitMemoryType::Semantic,
                MemoryType::Procedural => TraitMemoryType::Procedural,
                MemoryType::Working => TraitMemoryType::Working,
                MemoryType::Core => TraitMemoryType::Core,
                MemoryType::Resource => TraitMemoryType::Resource,
                MemoryType::Knowledge => TraitMemoryType::Knowledge,
                MemoryType::Contextual => TraitMemoryType::Contextual,
            },
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: memory.agent_id,
            user_id: memory.user_id,
            importance: memory.importance,
            embedding: memory.embedding.map(|v| v.values),
            last_accessed_at: DateTime::from_timestamp(memory.last_accessed_at, 0)
                .unwrap_or_else(Utc::now),
            access_count: memory.access_count,
            expires_at: memory
                .expires_at
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
            version: memory.version,
        }
    }
}

/// MemoryItem → Memory（用于API兼容）
impl TryFrom<MemoryItem> for Memory {
    type Error = AgentMemError;

    fn try_from(item: MemoryItem) -> Result<Self> {
        let mut attributes = AttributeSet::new();

        // Extract system attributes
        attributes.set(
            AttributeKey::system("agent_id"),
            AttributeValue::String(item.agent_id.clone()),
        );

        if let Some(user_id) = item.user_id.clone() {
            attributes.set(
                AttributeKey::system("user_id"),
                AttributeValue::String(user_id),
            );
        }

        attributes.set(
            AttributeKey::core("memory_type"),
            AttributeValue::String(item.memory_type.as_str().to_string()),
        );

        attributes.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(item.importance as f64),
        );

        // Convert metadata to attributes
        for (k, v) in item.metadata {
            attributes.set(AttributeKey::user(k), AttributeValue::from_json(v));
        }

        Ok(Memory {
            id: item.id,
            content: Content::Text(item.content),
            attributes,
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: item.created_at,
                updated_at: item.updated_at.unwrap_or(item.created_at),
                access_count: item.access_count as u64,
                last_accessed: Some(item.last_accessed_at),
            },
        })
    }
}

/// Memory search query
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    /// Agent ID to search within
    pub agent_id: String,
    /// User ID filter (optional)
    pub user_id: Option<String>,
    /// Memory type filter (optional)
    pub memory_type: Option<MemoryType>,
    /// Text query for content search
    pub text_query: Option<String>,
    /// Vector query for semantic search
    pub vector_query: Option<Vector>,
    /// Minimum importance threshold
    pub min_importance: Option<f32>,
    /// Maximum age in seconds
    pub max_age_seconds: Option<i64>,
    /// Maximum number of results
    pub limit: usize,
}

impl MemoryQuery {
    /// Create a new memory query for the specified agent
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            user_id: None,
            memory_type: None,
            text_query: None,
            vector_query: None,
            min_importance: None,
            max_age_seconds: None,
            limit: 10,
        }
    }

    /// Set the user ID for the query
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the memory type filter
    pub fn with_memory_type(mut self, memory_type: MemoryType) -> Self {
        self.memory_type = Some(memory_type);
        self
    }

    /// Set the text query for searching
    pub fn with_text_query(mut self, query: String) -> Self {
        self.text_query = Some(query);
        self
    }

    /// Set the vector query for semantic search
    pub fn with_vector_query(mut self, vector: Vector) -> Self {
        self.vector_query = Some(vector);
        self
    }

    /// Set the minimum importance threshold
    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }

    /// Set the maximum age filter in seconds
    pub fn with_max_age_seconds(mut self, seconds: i64) -> Self {
        self.max_age_seconds = Some(seconds);
        self
    }

    /// Set the maximum number of results to return
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Memory search result
#[derive(Debug, Clone)]
pub struct MemorySearchResult {
    /// The matched memory
    pub memory: agent_mem_traits::MemoryV4,
    /// Relevance score (0.0 to 1.0)
    pub score: f32,
    /// Type of match found
    pub match_type: MatchType,
}

/// Type of match found
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    /// Exact text match
    ExactText,
    /// Partial text match
    PartialText,
    /// Semantic similarity match
    Semantic,
    /// Metadata field match
    Metadata,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of memories
    pub total_memories: usize,
    /// Count of memories by type
    pub memories_by_type: HashMap<MemoryType, usize>,
    /// Count of memories by agent
    pub memories_by_agent: HashMap<String, usize>,
    /// Average importance score across all memories
    pub average_importance: f32,
    /// Age of the oldest memory in days
    pub oldest_memory_age_days: f32,
    /// ID of the most frequently accessed memory
    pub most_accessed_memory_id: Option<String>,
    /// Total number of memory accesses
    pub total_access_count: u64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memories: 0,
            memories_by_type: HashMap::new(),
            memories_by_agent: HashMap::new(),
            average_importance: 0.0,
            oldest_memory_age_days: 0.0,
            most_accessed_memory_id: None,
            total_access_count: 0,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    // ========== V4.0 新架构测试 ==========

    #[test]
    fn test_content_text() {
        let content = Content::Text("Hello World".to_string());
        assert_eq!(content.as_text(), "Hello World");
    }

    #[test]
    fn test_content_mixed() {
        let content = Content::Mixed(vec![
            Content::Text("Part 1".to_string()),
            Content::Image {
                url: "http://example.com/img.jpg".to_string(),
                caption: Some("Image".to_string()),
            },
        ]);
        let text = content.as_text();
        assert!(text.contains("Part 1"));
        assert!(text.contains("[Image:"));
    }

    #[test]
    fn test_attribute_set_basic() {
        let mut attrs = AttributeSet::new();

        // Set attribute
        attrs.set(
            AttributeKey::system("user_id"),
            AttributeValue::String("user123".to_string()),
        );

        // Get attribute
        let value = attrs.get(&AttributeKey::system("user_id"));
        assert!(value.is_some());
        assert_eq!(value.unwrap().as_string(), Some("user123"));

        // Contains check
        assert!(attrs.contains(&AttributeKey::system("user_id")));
        assert!(!attrs.contains(&AttributeKey::system("nonexistent")));
    }

    #[test]
    fn test_attribute_set_query_by_namespace() {
        let mut attrs = AttributeSet::new();
        attrs.set(
            AttributeKey::system("key1"),
            AttributeValue::String("val1".to_string()),
        );
        attrs.set(AttributeKey::system("key2"), AttributeValue::Number(42.0));
        attrs.set(AttributeKey::user("key3"), AttributeValue::Boolean(true));

        let system_attrs = attrs.query_by_namespace("system");
        assert_eq!(system_attrs.len(), 2);

        let user_attrs = attrs.query_by_namespace("user");
        assert_eq!(user_attrs.len(), 1);
    }

    #[test]
    fn test_attribute_set_query_prefix() {
        let mut attrs = AttributeSet::new();
        attrs.set(
            AttributeKey::domain("product_id"),
            AttributeValue::String("P000257".to_string()),
        );
        attrs.set(
            AttributeKey::domain("product_name"),
            AttributeValue::String("Widget".to_string()),
        );
        attrs.set(
            AttributeKey::domain("category"),
            AttributeValue::String("Electronics".to_string()),
        );

        let pattern = AttributePattern::Prefix {
            namespace: "domain".to_string(),
            prefix: "product".to_string(),
        };

        let results = attrs.query(&pattern);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_attribute_set_query_range() {
        let mut attrs = AttributeSet::new();
        attrs.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(0.75),
        );

        let pattern = AttributePattern::Range {
            key: AttributeKey::system("importance"),
            min: 0.5,
            max: 1.0,
        };

        let results = attrs.query(&pattern);
        assert_eq!(results.len(), 1);

        let pattern_out_of_range = AttributePattern::Range {
            key: AttributeKey::system("importance"),
            min: 0.0,
            max: 0.5,
        };

        let results_empty = attrs.query(&pattern_out_of_range);
        assert_eq!(results_empty.len(), 0);
    }

    #[test]
    fn test_relation_graph() {
        let mut graph = RelationGraph::new();

        graph.add_relation(Relation {
            target_id: "mem-123".to_string(),
            relation_type: RelationType::References,
            strength: 0.9,
        });

        graph.add_relation(Relation {
            target_id: "mem-456".to_string(),
            relation_type: RelationType::SimilarTo,
            strength: 0.7,
        });

        assert_eq!(graph.relations().len(), 2);

        let references = graph.find_by_type(&RelationType::References);
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].target_id, "mem-123");

        let target_relations = graph.find_by_target("mem-456");
        assert_eq!(target_relations.len(), 1);
    }

    #[test]
    fn test_memory_builder() {
        let memory = Memory::builder()
            .text("Test content")
            .attribute(
                AttributeKey::system("user_id"),
                AttributeValue::String("user123".to_string()),
            )
            .attribute(
                AttributeKey::system("importance"),
                AttributeValue::Number(0.8),
            )
            .relation("mem-999".to_string(), RelationType::References, 0.95)
            .build();

        assert!(memory.id.len() > 0);
        assert_eq!(memory.content.as_text(), "Test content");
        assert_eq!(
            memory
                .attributes
                .get(&AttributeKey::system("user_id"))
                .unwrap()
                .as_string(),
            Some("user123")
        );
        assert_eq!(memory.relations.relations().len(), 1);
    }

    #[test]
    fn test_memory_from_legacy() {
        let legacy = LegacyMemory {
            id: "mem-001".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: Some("user-1".to_string()),
            memory_type: MemoryType::Semantic,
            content: "Legacy content".to_string(),
            importance: 0.75,
            embedding: None,
            created_at: 1609459200,
            last_accessed_at: 1609459200,
            access_count: 5,
            expires_at: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("key1".to_string(), "value1".to_string());
                m
            },
            version: 1,
        };

        let memory = Memory::from_legacy(legacy.clone());

        // Verify ID and content
        assert_eq!(memory.id, "mem-001");
        assert_eq!(memory.content.as_text(), "Legacy content");

        // Verify system attributes
        assert_eq!(
            memory
                .attributes
                .get(&AttributeKey::system("agent_id"))
                .unwrap()
                .as_string(),
            Some("agent-1")
        );
        assert_eq!(
            memory
                .attributes
                .get(&AttributeKey::system("user_id"))
                .unwrap()
                .as_string(),
            Some("user-1")
        );
        assert_eq!(
            memory
                .attributes
                .get(&AttributeKey::core("memory_type"))
                .unwrap()
                .as_string(),
            Some("semantic")
        );
        assert_eq!(
            memory
                .attributes
                .get(&AttributeKey::system("importance"))
                .unwrap()
                .as_number(),
            Some(0.75)
        );

        // Verify legacy metadata migration
        assert_eq!(
            memory
                .attributes
                .get(&AttributeKey::legacy("key1"))
                .unwrap()
                .as_string(),
            Some("value1")
        );

        // Verify metadata
        assert_eq!(memory.metadata.access_count, 5);
    }

    #[test]
    fn test_memory_access() {
        let mut memory = Memory::builder().text("Test").build();

        let initial_count = memory.metadata.access_count;
        memory.access();

        assert_eq!(memory.metadata.access_count, initial_count + 1);
        assert!(memory.metadata.last_accessed.is_some());
    }

    // ========== 原有测试（保持向后兼容） ==========

    #[test]
    fn test_memory_type_string_conversion() {
        // Test basic cognitive memory types
        assert_eq!(MemoryType::Episodic.as_str(), "episodic");
        assert_eq!(MemoryType::Semantic.as_str(), "semantic");
        assert_eq!(MemoryType::Procedural.as_str(), "procedural");
        assert_eq!(MemoryType::Working.as_str(), "working");

        // Test advanced cognitive memory types (AgentMem 7.0)
        assert_eq!(MemoryType::Core.as_str(), "core");
        assert_eq!(MemoryType::Resource.as_str(), "resource");
        assert_eq!(MemoryType::Knowledge.as_str(), "knowledge");
        assert_eq!(MemoryType::Contextual.as_str(), "contextual");
    }

    #[test]
    fn test_memory_type_from_string() {
        // Test basic cognitive memory types
        assert_eq!(MemoryType::from_str("episodic"), Some(MemoryType::Episodic));
        assert_eq!(MemoryType::from_str("semantic"), Some(MemoryType::Semantic));
        assert_eq!(
            MemoryType::from_str("procedural"),
            Some(MemoryType::Procedural)
        );
        assert_eq!(MemoryType::from_str("working"), Some(MemoryType::Working));

        // Test advanced cognitive memory types (AgentMem 7.0)
        assert_eq!(MemoryType::from_str("core"), Some(MemoryType::Core));
        assert_eq!(MemoryType::from_str("resource"), Some(MemoryType::Resource));
        assert_eq!(
            MemoryType::from_str("knowledge"),
            Some(MemoryType::Knowledge)
        );
        assert_eq!(
            MemoryType::from_str("contextual"),
            Some(MemoryType::Contextual)
        );

        // Test invalid type
        assert_eq!(MemoryType::from_str("invalid"), None);
    }

    #[test]
    fn test_memory_type_classification() {
        // Test basic type classification
        assert!(MemoryType::Episodic.is_basic_type());
        assert!(MemoryType::Semantic.is_basic_type());
        assert!(MemoryType::Procedural.is_basic_type());
        assert!(MemoryType::Working.is_basic_type());

        assert!(!MemoryType::Episodic.is_advanced_type());
        assert!(!MemoryType::Semantic.is_advanced_type());
        assert!(!MemoryType::Procedural.is_advanced_type());
        assert!(!MemoryType::Working.is_advanced_type());

        // Test advanced type classification
        assert!(MemoryType::Core.is_advanced_type());
        assert!(MemoryType::Resource.is_advanced_type());
        assert!(MemoryType::Knowledge.is_advanced_type());
        assert!(MemoryType::Contextual.is_advanced_type());

        assert!(!MemoryType::Core.is_basic_type());
        assert!(!MemoryType::Resource.is_basic_type());
        assert!(!MemoryType::Knowledge.is_basic_type());
        assert!(!MemoryType::Contextual.is_basic_type());
    }

    #[test]
    fn test_memory_type_all_types() {
        let all_types = MemoryType::all_types();
        assert_eq!(all_types.len(), 8);

        // Verify all types are included
        assert!(all_types.contains(&MemoryType::Episodic));
        assert!(all_types.contains(&MemoryType::Semantic));
        assert!(all_types.contains(&MemoryType::Procedural));
        assert!(all_types.contains(&MemoryType::Working));
        assert!(all_types.contains(&MemoryType::Core));
        assert!(all_types.contains(&MemoryType::Resource));
        assert!(all_types.contains(&MemoryType::Knowledge));
        assert!(all_types.contains(&MemoryType::Contextual));
    }

    #[test]
    fn test_memory_type_descriptions() {
        // Test that all memory types have descriptions
        for memory_type in MemoryType::all_types() {
            let description = memory_type.description();
            assert!(
                !description.is_empty(),
                "Memory type {memory_type:?} should have a description"
            );
        }
    }

    #[test]
    fn test_legacy_memory_creation_with_new_types() {
        // Test creating memories with new cognitive types (using LegacyMemory)
        let core_memory = LegacyMemory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Core,
            "User prefers dark mode interface".to_string(),
            0.9,
        );
        assert_eq!(core_memory.memory_type, MemoryType::Core);
        assert_eq!(core_memory.importance, 0.9);

        let resource_memory = LegacyMemory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Resource,
            "Document: project_plan.pdf".to_string(),
            0.7,
        );
        assert_eq!(resource_memory.memory_type, MemoryType::Resource);

        let knowledge_memory = LegacyMemory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Knowledge,
            "Python is a programming language".to_string(),
            0.8,
        );
        assert_eq!(knowledge_memory.memory_type, MemoryType::Knowledge);

        let contextual_memory = LegacyMemory::new(
            "agent_1".to_string(),
            Some("user_1".to_string()),
            MemoryType::Contextual,
            "Currently in meeting room A".to_string(),
            0.6,
        );
        assert_eq!(contextual_memory.memory_type, MemoryType::Contextual);
    }

    // ========== Query抽象测试 ==========

    #[test]
    fn test_query_builder_basic() {
        let query = Query::builder().text("测试查询").limit(10).build();

        assert!(!query.id.is_empty());
        assert!(matches!(query.intent, QueryIntent::SemanticSearch { .. }));
        assert_eq!(query.constraints.len(), 1);
    }

    #[test]
    fn test_query_from_string_id_pattern() {
        let query = Query::from_string("P000257商品详情");

        if let QueryIntent::Lookup { entity_id } = query.intent {
            assert_eq!(entity_id, "P000257");
        } else {
            panic!("Expected Lookup intent");
        }
    }

    #[test]
    fn test_query_from_string_semantic() {
        let query = Query::from_string("查询所有电子产品");

        if let QueryIntent::SemanticSearch { text, .. } = query.intent {
            assert_eq!(text, "查询所有电子产品");
        } else {
            panic!("Expected SemanticSearch intent");
        }
    }

    #[test]
    fn test_query_builder_with_constraints() {
        let query = Query::builder()
            .text("测试")
            .with_attribute(
                AttributeKey::domain("product_id"),
                ComparisonOperator::Equal,
                AttributeValue::String("P000257".to_string()),
            )
            .limit(5)
            .build();

        assert_eq!(query.constraints.len(), 2); // attribute + limit
    }

    #[test]
    fn test_query_builder_with_preferences() {
        let query = Query::builder()
            .text("测试")
            .prefer(
                PreferenceType::Temporal(TemporalPreference::Recent { within_days: 7 }),
                0.8,
            )
            .prefer(
                PreferenceType::Importance {
                    min_importance: 0.5,
                },
                0.6,
            )
            .build();

        assert_eq!(query.preferences.len(), 2);
        assert_eq!(query.preferences[0].weight, 0.8);
        assert_eq!(query.preferences[1].weight, 0.6);
    }

    #[test]
    fn test_constraint_logic() {
        let constraint = Constraint::And(vec![
            Constraint::AttributeMatch {
                key: AttributeKey::domain("category"),
                operator: ComparisonOperator::Equal,
                value: AttributeValue::String("电子产品".to_string()),
            },
            Constraint::MinScore(0.7),
        ]);

        match constraint {
            Constraint::And(inner) => assert_eq!(inner.len(), 2),
            _ => panic!("Expected And constraint"),
        }
    }

    // ========== Scope消除测试（AttributeSet替代MemoryScope） ==========

    #[test]
    fn test_attributeset_global_scope() {
        let mut attrs = AttributeSet::new();
        attrs.set_global_scope();

        assert!(attrs.is_global_scope());
        assert_eq!(attrs.infer_scope_level(), 0);
    }

    #[test]
    fn test_attributeset_agent_scope() {
        let mut attrs = AttributeSet::new();
        attrs.set_agent_scope("agent-123");

        assert_eq!(attrs.get_agent_id(), Some("agent-123".to_string()));
        assert_eq!(attrs.infer_scope_level(), 1);
    }

    #[test]
    fn test_attributeset_user_scope() {
        let mut attrs = AttributeSet::new();
        attrs.set_user_scope("agent-123", "user-456");

        assert_eq!(attrs.get_agent_id(), Some("agent-123".to_string()));
        assert_eq!(attrs.get_user_id(), Some("user-456".to_string()));
        assert_eq!(attrs.infer_scope_level(), 2);
    }

    #[test]
    fn test_attributeset_session_scope() {
        let mut attrs = AttributeSet::new();
        attrs.set_session_scope("agent-123", "user-456", "session-789");

        assert_eq!(attrs.get_agent_id(), Some("agent-123".to_string()));
        assert_eq!(attrs.get_user_id(), Some("user-456".to_string()));
        assert_eq!(attrs.get_session_id(), Some("session-789".to_string()));
        assert_eq!(attrs.infer_scope_level(), 3);
    }

    #[test]
    fn test_attributeset_can_access() {
        let mut global = AttributeSet::new();
        global.set_global_scope();

        let mut agent = AttributeSet::new();
        agent.set_agent_scope("agent-123");

        let mut user = AttributeSet::new();
        user.set_user_scope("agent-123", "user-456");

        // Global可以访问所有
        assert!(global.can_access(&agent));
        assert!(global.can_access(&user));

        // Agent可以访问相同agent的user
        assert!(agent.can_access(&user));

        // User不能访问Agent
        assert!(!user.can_access(&agent));
    }

    #[test]
    fn test_memoryscope_to_attributeset() {
        use crate::hierarchy::MemoryScope;

        let scope = MemoryScope::User {
            agent_id: "agent-123".to_string(),
            user_id: "user-456".to_string(),
        };

        let attrs: AttributeSet = scope.into();

        assert_eq!(attrs.get_agent_id(), Some("agent-123".to_string()));
        assert_eq!(attrs.get_user_id(), Some("user-456".to_string()));
        assert_eq!(attrs.infer_scope_level(), 2);
    }

    #[test]
    fn test_attributeset_to_memoryscope() {
        use crate::hierarchy::MemoryScope;

        let mut attrs = AttributeSet::new();
        attrs.set_session_scope("agent-123", "user-456", "session-789");

        let scope: MemoryScope = (&attrs).into();

        match scope {
            MemoryScope::Session {
                agent_id,
                user_id,
                session_id,
            } => {
                assert_eq!(agent_id, "agent-123");
                assert_eq!(user_id, "user-456");
                assert_eq!(session_id, "session-789");
            }
            _ => panic!("Expected Session scope"),
        }
    }

    // ========== DAG Pipeline测试 ==========

    // 简单的测试Stage
    struct TestStage {
        name: String,
        delay_ms: u64,
    }

    impl TestStage {
        fn new(name: impl Into<String>, delay_ms: u64) -> Self {
            Self {
                name: name.into(),
                delay_ms,
            }
        }
    }

    #[async_trait::async_trait]
    impl PipelineStage for TestStage {
        type Input = i32;
        type Output = i32;

        fn name(&self) -> &str {
            &self.name
        }

        async fn execute(
            &self,
            input: Self::Input,
            context: &mut PipelineContext,
        ) -> anyhow::Result<StageResult<Self::Output>> {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            let _ = context.set(format!("{}_executed", self.name), true);
            Ok(StageResult::Continue(input + 1))
        }
    }

    #[tokio::test]
    async fn test_dag_pipeline_linear() -> anyhow::Result<()> {
        // 线性DAG: A -> B -> C
        let dag = DagPipeline::new("test_linear")
            .add_node("A", TestStage::new("A", 10), vec![])
            .add_node("B", TestStage::new("B", 10), vec!["A".to_string()])
            .add_node("C", TestStage::new("C", 10), vec!["B".to_string()]);

        let mut ctx = PipelineContext::new();
        let results = dag.execute(0, &mut ctx).await?;

        assert_eq!(results.len(), 3);
        assert_eq!(results.get("A"), Some(&1));
        assert_eq!(results.get("B"), Some(&1));
        assert_eq!(results.get("C"), Some(&1));
        Ok(())
    }

    #[tokio::test]
    async fn test_dag_pipeline_parallel() -> anyhow::Result<()> {
        // 并行DAG: A, B, C (无依赖)
        let dag = DagPipeline::new("test_parallel")
            .add_node("A", TestStage::new("A", 50), vec![])
            .add_node("B", TestStage::new("B", 50), vec![])
            .add_node("C", TestStage::new("C", 50), vec![]);

        let start = std::time::Instant::now();
        let mut ctx = PipelineContext::new();
        let results = dag.execute(0, &mut ctx).await?;
        let elapsed = start.elapsed().as_millis();

        assert_eq!(results.len(), 3);
        // 并行执行应该快于串行（3个50ms任务串行需要150ms，并行应该在100ms内，留50%余量）
        assert!(
            elapsed < 200,
            "Parallel execution took {}ms, expected < 200ms",
            elapsed
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_dag_pipeline_diamond() -> anyhow::Result<()> {
        // 菱形DAG: A -> B,C -> D
        let dag = DagPipeline::new("test_diamond")
            .add_node("A", TestStage::new("A", 10), vec![])
            .add_node("B", TestStage::new("B", 10), vec!["A".to_string()])
            .add_node("C", TestStage::new("C", 10), vec!["A".to_string()])
            .add_node(
                "D",
                TestStage::new("D", 10),
                vec!["B".to_string(), "C".to_string()],
            );

        let mut ctx = PipelineContext::new();
        let results = dag.execute(0, &mut ctx).await?;

        assert_eq!(results.len(), 4);
        assert!(ctx.get::<bool>("A_executed").unwrap_or(false));
        assert!(ctx.get::<bool>("B_executed").unwrap_or(false));
        assert!(ctx.get::<bool>("C_executed").unwrap_or(false));
        assert!(ctx.get::<bool>("D_executed").unwrap_or(false));
        Ok(())
    }

    #[tokio::test]
    async fn test_dag_pipeline_conditional() -> anyhow::Result<()> {
        // 条件分支: A -> B (if true) or C (if false)
        struct ConditionalStage;

        #[async_trait::async_trait]
        impl PipelineStage for ConditionalStage {
            type Input = i32;
            type Output = i32;

            fn name(&self) -> &str {
                "Conditional"
            }

            async fn execute(
                &self,
                input: Self::Input,
                context: &mut PipelineContext,
            ) -> anyhow::Result<StageResult<Self::Output>> {
                let _ = context.set("condition_value", input > 5);
                Ok(StageResult::Continue(input))
            }
        }

        let dag = DagPipeline::new("test_conditional")
            .add_node("A", ConditionalStage, vec![])
            .add_node("B", TestStage::new("B", 10), vec!["A".to_string()])
            .add_node("C", TestStage::new("C", 10), vec!["A".to_string()])
            .add_condition("is_high", |ctx| {
                ctx.get::<bool>("condition_value").unwrap_or(false)
            })
            .add_condition("is_low", |ctx| {
                !ctx.get::<bool>("condition_value").unwrap_or(true)
            })
            .add_edge("A", "B", Some("is_high".to_string()))
            .add_edge("A", "C", Some("is_low".to_string()));

        // Test with high value (should execute B)
        let mut ctx1 = PipelineContext::new();
        let results1 = dag.execute(10, &mut ctx1).await?;
        assert!(results1.contains_key("B"));
        assert!(!results1.contains_key("C"));

        // Test with low value (should execute C)
        let mut ctx2 = PipelineContext::new();
        let results2 = dag.execute(3, &mut ctx2).await?;
        assert!(!results2.contains_key("B"));
        assert!(results2.contains_key("C"));
        Ok(())
    }

    #[tokio::test]
    async fn test_dag_pipeline_cycle_detection() -> anyhow::Result<()> {
        // 创建循环依赖: A -> B -> C -> A
        let dag = DagPipeline::new("test_cycle")
            .add_node("A", TestStage::new("A", 10), vec!["C".to_string()])
            .add_node("B", TestStage::new("B", 10), vec!["A".to_string()])
            .add_node("C", TestStage::new("C", 10), vec!["B".to_string()]);

        let mut ctx = PipelineContext::new();
        let result = dag.execute(0, &mut ctx).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cycle detected"));
        Ok(())
    }

    #[tokio::test]
    async fn test_dag_pipeline_max_parallelism() -> anyhow::Result<()> {
        // 测试并行度控制
        let dag = DagPipeline::new("test_parallelism")
            .add_node("A", TestStage::new("A", 100), vec![])
            .add_node("B", TestStage::new("B", 100), vec![])
            .add_node("C", TestStage::new("C", 100), vec![])
            .add_node("D", TestStage::new("D", 100), vec![])
            .with_max_parallelism(2); // 最多同时执行2个

        let start = std::time::Instant::now();
        let mut ctx = PipelineContext::new();
        let results = dag.execute(0, &mut ctx).await?;
        let elapsed = start.elapsed().as_millis();

        assert_eq!(results.len(), 4);
        // 4个任务，并行度2，每批100ms，应该 >= 200ms
        assert!(
            elapsed >= 180,
            "Execution took {}ms, expected >= 180ms",
            elapsed
        );
        Ok(())
    }
}
