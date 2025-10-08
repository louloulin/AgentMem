//! Core Memory 系统
//!
//! 实现 MIRIX 风格的 Core Memory 系统，包括：
//! - Block 管理（Persona, Human, System）
//! - 模板系统（类似 Jinja2）
//! - 自动重写机制（LLM 驱动）
//! - Core Memory 编译器

pub mod auto_rewriter;
#[cfg(feature = "postgres")]
pub mod block_manager;
#[cfg(feature = "postgres")]
pub mod compiler;
pub mod template_engine;

pub use auto_rewriter::{AutoRewriter, AutoRewriterConfig, RewriteStrategy};
#[cfg(feature = "postgres")]
pub use block_manager::{BlockManager, BlockManagerConfig};
#[cfg(feature = "postgres")]
pub use compiler::{CompilerConfig, CoreMemoryCompiler};
pub use template_engine::{TemplateContext, TemplateEngine};

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};

/// Block 类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    /// Persona 块 - AI 的身份、性格、偏好
    Persona,
    /// Human 块 - 用户的信息、偏好、历史
    Human,
    /// System 块 - 系统指令、规则
    System,
}

impl BlockType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockType::Persona => "persona",
            BlockType::Human => "human",
            BlockType::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "persona" => Some(BlockType::Persona),
            "human" => Some(BlockType::Human),
            "system" => Some(BlockType::System),
            _ => None,
        }
    }
}

/// Block 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// 重要性评分 (0.0-1.0)
    pub importance: f32,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    /// 是否需要重写
    pub needs_rewrite: bool,
    /// 重写次数
    pub rewrite_count: u32,
    /// 自定义标签
    pub tags: Vec<String>,
}

impl Default for BlockMetadata {
    fn default() -> Self {
        Self {
            importance: 0.5,
            access_count: 0,
            last_accessed: None,
            needs_rewrite: false,
            rewrite_count: 0,
            tags: Vec::new(),
        }
    }
}

/// Block 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStats {
    /// 总 Block 数
    pub total_blocks: usize,
    /// Persona 块数
    pub persona_blocks: usize,
    /// Human 块数
    pub human_blocks: usize,
    /// System 块数
    pub system_blocks: usize,
    /// 总字符数
    pub total_characters: usize,
    /// 平均使用率
    pub average_utilization: f32,
    /// 需要重写的块数
    pub blocks_needing_rewrite: usize,
}

impl Default for BlockStats {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            persona_blocks: 0,
            human_blocks: 0,
            system_blocks: 0,
            total_characters: 0,
            average_utilization: 0.0,
            blocks_needing_rewrite: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_type_conversion() {
        assert_eq!(BlockType::Persona.as_str(), "persona");
        assert_eq!(BlockType::Human.as_str(), "human");
        assert_eq!(BlockType::System.as_str(), "system");

        assert_eq!(BlockType::from_str("persona"), Some(BlockType::Persona));
        assert_eq!(BlockType::from_str("HUMAN"), Some(BlockType::Human));
        assert_eq!(BlockType::from_str("System"), Some(BlockType::System));
        assert_eq!(BlockType::from_str("invalid"), None);
    }

    #[test]
    fn test_block_metadata_default() {
        let metadata = BlockMetadata::default();
        assert_eq!(metadata.importance, 0.5);
        assert_eq!(metadata.access_count, 0);
        assert!(metadata.last_accessed.is_none());
        assert!(!metadata.needs_rewrite);
        assert_eq!(metadata.rewrite_count, 0);
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_block_stats_default() {
        let stats = BlockStats::default();
        assert_eq!(stats.total_blocks, 0);
        assert_eq!(stats.persona_blocks, 0);
        assert_eq!(stats.human_blocks, 0);
        assert_eq!(stats.system_blocks, 0);
        assert_eq!(stats.total_characters, 0);
        assert_eq!(stats.average_utilization, 0.0);
        assert_eq!(stats.blocks_needing_rewrite, 0);
    }
}
