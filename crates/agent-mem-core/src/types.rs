//! Core memory types and data structures

use agent_mem_traits::{AgentMemError, MemoryItem, Result, Vector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;

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
    Image { url: String, caption: Option<String> },
    /// 音频内容（URL + 可选转录文本）
    Audio { url: String, transcript: Option<String> },
    /// 视频内容（URL + 可选摘要）
    Video { url: String, summary: Option<String> },
    /// 结构化数据（JSON）
    Structured(serde_json::Value),
    /// 混合内容（多种类型组合）
    Mixed(Vec<Content>),
}

impl Content {
    /// 获取文本表示（用于向后兼容）
    pub fn as_text(&self) -> String {
        match self {
            Content::Text(s) => s.clone(),
            Content::Image { url, caption } => {
                format!("[Image: {}{}]", url, caption.as_ref().map(|c| format!(" - {}", c)).unwrap_or_default())
            }
            Content::Audio { url, transcript } => {
                format!("[Audio: {}{}]", url, transcript.as_ref().map(|t| format!(" - {}", t)).unwrap_or_default())
            }
            Content::Video { url, summary } => {
                format!("[Video: {}{}]", url, summary.as_ref().map(|s| format!(" - {}", s)).unwrap_or_default())
            }
            Content::Structured(v) => serde_json::to_string(v).unwrap_or_else(|_| "[Structured Data]".to_string()),
            Content::Mixed(contents) => contents.iter().map(|c| c.as_text()).collect::<Vec<_>>().join("\n"),
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
    
    /// 领域属性（domain命名空间）
    pub fn domain(name: impl Into<String>) -> Self {
        Self::new("domain", name)
    }
    
    /// 旧版属性（legacy命名空间，用于迁移）
    pub fn legacy(name: impl Into<String>) -> Self {
        Self::new("legacy", name)
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
            serde_json::Value::Object(obj) => {
                AttributeValue::Object(
                    obj.into_iter()
                        .map(|(k, v)| (k, Self::from_json(v)))
                        .collect(),
                )
            }
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
            AttributeValue::Object(obj) => {
                serde_json::Value::Object(
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.to_json()))
                        .collect(),
                )
            }
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
    Range { key: AttributeKey, min: f64, max: f64 },
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
    pub fn query(&self, pattern: &AttributePattern) -> Vec<(&AttributeKey, &AttributeValue)> {
        match pattern {
            AttributePattern::Exact { key } => {
                self.get(key).map(|v| vec![(key, v)]).unwrap_or_default()
            }
            AttributePattern::Prefix { namespace, prefix } => {
                self.attributes.iter()
                    .filter(|(k, _)| k.namespace == *namespace && k.name.starts_with(prefix))
                    .collect()
            }
            AttributePattern::Regex { namespace, pattern } => {
                if let Ok(re) = Regex::new(pattern) {
                    self.attributes.iter()
                        .filter(|(k, _)| k.namespace == *namespace && re.is_match(&k.name))
                        .collect()
                } else {
                    vec![]
                }
            }
            AttributePattern::Range { key, min, max } => {
                self.get(key)
                    .and_then(|v| v.as_number())
                    .filter(|&n| n >= *min && n <= *max)
                    .map(|_| vec![(key, self.get(key).unwrap())])
                    .unwrap_or_default()
            }
        }
    }
    
    /// 按命名空间查询
    pub fn query_by_namespace(&self, namespace: &str) -> Vec<(&AttributeKey, &AttributeValue)> {
        self.attributes.iter()
            .filter(|(k, _)| k.namespace == namespace)
            .collect()
    }
}

impl Default for AttributeSet {
    fn default() -> Self {
        Self::new()
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
        self.relations.iter()
            .filter(|r| std::mem::discriminant(&r.relation_type) == std::mem::discriminant(relation_type))
            .collect()
    }
    
    /// 查找目标记忆的所有关系
    pub fn find_by_target(&self, target_id: &str) -> Vec<&Relation> {
        self.relations.iter()
            .filter(|r| r.target_id == target_id)
            .collect()
    }
}

/// 系统元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub accessed_count: u64,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            accessed_count: 0,
            last_accessed: None,
        }
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
            AttributeKey::system("memory_type"),
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
            attributes.set(
                AttributeKey::legacy(key),
                AttributeValue::String(value),
            );
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
                accessed_count: old.access_count as u64,
                last_accessed: Some(
                    chrono::DateTime::from_timestamp(old.last_accessed_at, 0)
                        .unwrap_or_else(chrono::Utc::now),
                ),
            },
        }
    }
    
    /// 记录访问
    pub fn access(&mut self) {
        self.metadata.accessed_count += 1;
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
    
    pub fn attribute(mut self, key: impl Into<AttributeKey>, value: impl Into<AttributeValue>) -> Self {
        self.attributes.set(key.into(), value.into());
        self
    }
    
    pub fn relation(mut self, target_id: String, relation_type: RelationType, strength: f32) -> Self {
        self.relations.add_relation(Relation { target_id, relation_type, strength });
        self
    }
    
    pub fn build(self) -> Memory {
        Memory {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            content: self.content.expect("content is required"),
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
        let agent_id = memory.attributes.get(&AttributeKey::system("agent_id"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default".to_string());
        
        let user_id = memory.attributes.get(&AttributeKey::system("user_id"))
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());
        
        let memory_type_str = memory.attributes.get(&AttributeKey::system("memory_type"))
            .and_then(|v| v.as_string())
            .unwrap_or("semantic");
        
        let importance = memory.attributes.get(&AttributeKey::system("importance"))
            .and_then(|v| v.as_number())
            .unwrap_or(0.5) as f32;

        // Convert all attributes to metadata
        let metadata: HashMap<String, serde_json::Value> = memory.attributes
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
            access_count: memory.metadata.accessed_count as u32,
            expires_at: memory.attributes.get(&AttributeKey::system("expires_at"))
                .and_then(|v| v.as_number())
                .map(|ts| DateTime::from_timestamp(ts as i64, 0).unwrap_or_else(Utc::now)),
            version: memory.attributes.get(&AttributeKey::system("version"))
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
            AttributeKey::system("memory_type"),
            AttributeValue::String(item.memory_type.as_str().to_string()),
        );
        
        attributes.set(
            AttributeKey::system("importance"),
            AttributeValue::Number(item.importance as f64),
        );
        
        // Convert metadata to attributes
        for (k, v) in item.metadata {
            attributes.set(
                AttributeKey::user(k),
                AttributeValue::from_json(v),
            );
        }

        Ok(Memory {
            id: item.id,
            content: Content::Text(item.content),
            attributes,
            relations: RelationGraph::new(),
            metadata: Metadata {
                created_at: item.created_at,
                updated_at: item.updated_at.unwrap_or(item.created_at),
                accessed_count: item.access_count as u64,
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
    pub memory: Memory,
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
            Content::Image { url: "http://example.com/img.jpg".to_string(), caption: Some("Image".to_string()) },
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
        attrs.set(AttributeKey::system("key1"), AttributeValue::String("val1".to_string()));
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
        attrs.set(AttributeKey::domain("product_id"), AttributeValue::String("P000257".to_string()));
        attrs.set(AttributeKey::domain("product_name"), AttributeValue::String("Widget".to_string()));
        attrs.set(AttributeKey::domain("category"), AttributeValue::String("Electronics".to_string()));
        
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
        attrs.set(AttributeKey::system("importance"), AttributeValue::Number(0.75));
        
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
            .attribute(AttributeKey::system("user_id"), AttributeValue::String("user123".to_string()))
            .attribute(AttributeKey::system("importance"), AttributeValue::Number(0.8))
            .relation("mem-999".to_string(), RelationType::References, 0.95)
            .build();
        
        assert!(memory.id.len() > 0);
        assert_eq!(memory.content.as_text(), "Test content");
        assert_eq!(
            memory.attributes.get(&AttributeKey::system("user_id")).unwrap().as_string(),
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
            memory.attributes.get(&AttributeKey::system("agent_id")).unwrap().as_string(),
            Some("agent-1")
        );
        assert_eq!(
            memory.attributes.get(&AttributeKey::system("user_id")).unwrap().as_string(),
            Some("user-1")
        );
        assert_eq!(
            memory.attributes.get(&AttributeKey::system("memory_type")).unwrap().as_string(),
            Some("semantic")
        );
        assert_eq!(
            memory.attributes.get(&AttributeKey::system("importance")).unwrap().as_number(),
            Some(0.75)
        );
        
        // Verify legacy metadata migration
        assert_eq!(
            memory.attributes.get(&AttributeKey::legacy("key1")).unwrap().as_string(),
            Some("value1")
        );
        
        // Verify metadata
        assert_eq!(memory.metadata.accessed_count, 5);
    }
    
    #[test]
    fn test_memory_access() {
        let mut memory = Memory::builder()
            .text("Test")
            .build();
        
        let initial_count = memory.metadata.accessed_count;
        memory.access();
        
        assert_eq!(memory.metadata.accessed_count, initial_count + 1);
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
}
