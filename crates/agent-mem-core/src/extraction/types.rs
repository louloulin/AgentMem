//! 实体和关系提取的类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// 人物（姓名、称呼等）
    Person,
    /// 组织机构（公司、学校、政府等）
    Organization,
    /// 地点（城市、国家、地址等）
    Location,
    /// 日期（年月日）
    Date,
    /// 时间（时分秒）
    Time,
    /// 事件（会议、活动等）
    Event,
    /// 概念（抽象概念、术语等）
    Concept,
    /// 产品（商品、服务等）
    Product,
    /// 数字（数量、编号等）
    Number,
    /// 金额（货币金额）
    Money,
    /// 百分比
    Percentage,
    /// 邮箱地址
    Email,
    /// 电话号码
    Phone,
    /// 网址
    Url,
    /// 技能（编程语言、专业技能等）
    Skill,
    /// 语言（自然语言）
    Language,
    /// 技术（技术栈、工具等）
    Technology,
    /// 其他类型
    Other(String),
}

impl EntityType {
    /// 转换为字符串表示
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Person => "Person",
            EntityType::Organization => "Organization",
            EntityType::Location => "Location",
            EntityType::Date => "Date",
            EntityType::Time => "Time",
            EntityType::Event => "Event",
            EntityType::Concept => "Concept",
            EntityType::Product => "Product",
            EntityType::Number => "Number",
            EntityType::Money => "Money",
            EntityType::Percentage => "Percentage",
            EntityType::Email => "Email",
            EntityType::Phone => "Phone",
            EntityType::Url => "Url",
            EntityType::Skill => "Skill",
            EntityType::Language => "Language",
            EntityType::Technology => "Technology",
            EntityType::Other(s) => s,
        }
    }
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体唯一标识符
    pub id: String,
    /// 实体名称
    pub name: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 置信度（0.0-1.0）
    pub confidence: f32,
    /// 实体属性（键值对）
    pub attributes: HashMap<String, String>,
    /// 在文本中的位置（起始位置，结束位置）
    pub span: Option<(usize, usize)>,
}

impl Entity {
    /// 创建新实体
    pub fn new(id: String, name: String, entity_type: EntityType, confidence: f32) -> Self {
        Self {
            id,
            name,
            entity_type,
            confidence,
            attributes: HashMap::new(),
            span: None,
        }
    }

    /// 添加属性
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// 设置文本位置
    pub fn with_span(mut self, start: usize, end: usize) -> Self {
        self.span = Some((start, end));
        self
    }
}

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// 工作于（人物-组织）
    WorksAt,
    /// 位于（实体-地点）
    LocatedAt,
    /// 家庭关系（人物-人物）
    FamilyOf,
    /// 朋友关系（人物-人物）
    FriendOf,
    /// 喜欢（人物-事物）
    Likes,
    /// 不喜欢（人物-事物）
    Dislikes,
    /// 拥有（人物-事物）
    Owns,
    /// 参与（人物-事件）
    ParticipatesIn,
    /// 发生于（事件-时间/地点）
    OccursAt,
    /// 导致（事件-事件）
    Causes,
    /// 属于（实体-类别）
    BelongsTo,
    /// 使用（人物-技术/工具）
    Uses,
    /// 创建（人物-产品/内容）
    Creates,
    /// 学习（人物-技能/知识）
    Learns,
    /// 教授（人物-人物-技能）
    Teaches,
    /// 其他关系
    Other(String),
}

impl RelationType {
    /// 转换为字符串表示
    pub fn as_str(&self) -> &str {
        match self {
            RelationType::WorksAt => "WorksAt",
            RelationType::LocatedAt => "LocatedAt",
            RelationType::FamilyOf => "FamilyOf",
            RelationType::FriendOf => "FriendOf",
            RelationType::Likes => "Likes",
            RelationType::Dislikes => "Dislikes",
            RelationType::Owns => "Owns",
            RelationType::ParticipatesIn => "ParticipatesIn",
            RelationType::OccursAt => "OccursAt",
            RelationType::Causes => "Causes",
            RelationType::BelongsTo => "BelongsTo",
            RelationType::Uses => "Uses",
            RelationType::Creates => "Creates",
            RelationType::Learns => "Learns",
            RelationType::Teaches => "Teaches",
            RelationType::Other(s) => s,
        }
    }
}

/// 关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// 关系唯一标识符
    pub id: String,
    /// 主体实体 ID
    pub subject_id: String,
    /// 主体实体名称
    pub subject: String,
    /// 谓词（关系描述）
    pub predicate: String,
    /// 客体实体 ID
    pub object_id: String,
    /// 客体实体名称
    pub object: String,
    /// 关系类型
    pub relation_type: RelationType,
    /// 置信度（0.0-1.0）
    pub confidence: f32,
    /// 关系属性（键值对）
    pub attributes: HashMap<String, String>,
}

impl Relation {
    /// 创建新关系
    pub fn new(
        id: String,
        subject_id: String,
        subject: String,
        predicate: String,
        object_id: String,
        object: String,
        relation_type: RelationType,
        confidence: f32,
    ) -> Self {
        Self {
            id,
            subject_id,
            subject,
            predicate,
            object_id,
            object,
            relation_type,
            confidence,
            attributes: HashMap::new(),
        }
    }

    /// 添加属性
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// 提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// 提取到的实体列表
    pub entities: Vec<Entity>,
    /// 提取到的关系列表
    pub relations: Vec<Relation>,
    /// 提取耗时（毫秒）
    pub duration_ms: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl ExtractionResult {
    /// 创建新的提取结果
    pub fn new(entities: Vec<Entity>, relations: Vec<Relation>, duration_ms: u64) -> Self {
        Self {
            entities,
            relations,
            duration_ms,
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 获取关系数量
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }
}
