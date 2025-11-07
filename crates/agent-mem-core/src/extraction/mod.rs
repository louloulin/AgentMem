//! 实体和关系提取模块
//!
//! 提供从文本中提取实体和关系的功能，用于构建知识图谱。
//!
//! ## 功能特性
//!
//! - **实体识别**: 识别人物、组织、地点、时间等实体
//! - **关系提取**: 提取实体之间的关系
//! - **多种提取器**: 支持基于规则和基于 LLM 的提取
//! - **置信度评分**: 为每个提取结果提供置信度分数
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use agent_mem_core::extraction::{EntityExtractor, RuleBasedExtractor};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let extractor = RuleBasedExtractor::new();
//!     
//!     let text = "张三在北京的谷歌公司工作";
//!     let entities = extractor.extract_entities(text).await?;
//!     let relations = extractor.extract_relations(text, &entities).await?;
//!     
//!     println!("提取到 {} 个实体", entities.len());
//!     println!("提取到 {} 个关系", relations.len());
//!     
//!     Ok(())
//! }
//! ```

pub mod entity_extractor;
pub mod relation_extractor;
pub mod types;

pub use entity_extractor::{EntityExtractor, RuleBasedExtractor};
pub use relation_extractor::{RelationExtractor, RuleBasedRelationExtractor};
pub use types::{Entity, EntityType, ExtractionResult, Relation, RelationType};
