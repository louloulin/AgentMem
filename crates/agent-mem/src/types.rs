//! 类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use agent_mem_traits::{MemoryItem, MemoryType};

/// 添加记忆的选项
#[derive(Debug, Clone, Default)]
pub struct AddMemoryOptions {
    /// 记忆类型（如果不指定，将自动推断）
    pub memory_type: Option<MemoryType>,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 搜索记忆的选项
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// 返回结果数量限制
    pub limit: usize,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 记忆类型过滤
    pub memory_type: Option<MemoryType>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            user_id: None,
            memory_type: None,
        }
    }
}

/// 对话选项
#[derive(Debug, Clone)]
pub struct ChatOptions {
    /// 用户 ID
    pub user_id: Option<String>,
    /// 是否保存对话历史
    pub save_history: bool,
    /// 检索记忆数量
    pub retrieval_limit: usize,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            user_id: None,
            save_history: true,
            retrieval_limit: 5,
        }
    }
}

/// 记忆统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// 总记忆数
    pub total_memories: usize,
    /// 按类型分组的记忆数
    pub memories_by_type: HashMap<String, usize>,
    /// 平均重要性分数
    pub average_importance: f32,
    /// 存储大小（字节）
    pub storage_size_bytes: u64,
    /// 最后更新时间
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memories: 0,
            memories_by_type: HashMap::new(),
            average_importance: 0.0,
            storage_size_bytes: 0,
            last_updated: None,
        }
    }
}

/// 记忆可视化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVisualization {
    /// 总记忆数
    pub total_count: usize,
    /// 核心记忆
    pub core_memories: Vec<MemoryItem>,
    /// 情景记忆
    pub episodic_memories: Vec<MemoryItem>,
    /// 语义记忆
    pub semantic_memories: Vec<MemoryItem>,
    /// 程序记忆
    pub procedural_memories: Vec<MemoryItem>,
    /// 资源记忆
    pub resource_memories: Vec<MemoryItem>,
    /// 统计信息
    pub stats: MemoryStats,
}

impl Default for MemoryVisualization {
    fn default() -> Self {
        Self {
            total_count: 0,
            core_memories: Vec::new(),
            episodic_memories: Vec::new(),
            semantic_memories: Vec::new(),
            procedural_memories: Vec::new(),
            resource_memories: Vec::new(),
            stats: MemoryStats::default(),
        }
    }
}

/// 备份选项
#[derive(Debug, Clone)]
pub struct BackupOptions {
    /// 是否包含配置
    pub include_config: bool,
    /// 是否压缩
    pub compress: bool,
}

impl Default for BackupOptions {
    fn default() -> Self {
        Self {
            include_config: true,
            compress: true,
        }
    }
}

/// 恢复选项
#[derive(Debug, Clone)]
pub struct RestoreOptions {
    /// 是否覆盖现有数据
    pub overwrite: bool,
    /// 是否验证数据完整性
    pub verify: bool,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            verify: true,
        }
    }
}

