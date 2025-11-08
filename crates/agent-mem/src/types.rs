//! 类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use agent_mem_traits::{MemoryItem, MemoryType};

/// 添加记忆的选项（Mem0 兼容）
///
/// # 默认行为
///
/// - `infer`: **默认为 `true`**，启用智能功能（事实提取、去重、冲突解决）
/// - 如果智能组件未初始化（如未配置 LLM API Key），会自动降级到简单模式
/// - 与 Mem0 的默认行为一致
///
/// # 示例
///
/// ## 使用默认值（推荐）
///
/// ```rust
/// use agent_mem::Memory;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mem = Memory::new().await?;
///
/// // 使用默认值（智能模式）
/// mem.add("I love pizza").await?;
/// # Ok(())
/// # }
/// ```
///
/// ## 显式禁用智能功能
///
/// ```rust
/// use agent_mem::{Memory, AddMemoryOptions};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mem = Memory::new().await?;
///
/// // 禁用智能功能（直接存储原始内容）
/// let options = AddMemoryOptions {
///     infer: false,
///     ..Default::default()
/// };
/// mem.add_with_options("Raw content".to_string(), options).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## 使用 Session 管理
///
/// ```rust
/// use agent_mem::{Memory, AddMemoryOptions};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mem = Memory::new().await?;
///
/// let options = AddMemoryOptions {
///     user_id: Some("alice".to_string()),
///     agent_id: Some("assistant".to_string()),
///     run_id: Some("session-123".to_string()),
///     ..Default::default()  // infer: true
/// };
/// mem.add_with_options("I love pizza".to_string(), options).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemoryOptions {
    /// 用户 ID
    pub user_id: Option<String>,
    /// Agent ID
    pub agent_id: Option<String>,
    /// Run ID
    pub run_id: Option<String>,
    /// 元数据（支持多种类型数据）
    pub metadata: HashMap<String, String>,
    /// 启用智能推理（事实提取、去重等）
    ///
    /// **默认值**: `true`（与 Mem0 一致）
    ///
    /// - 如果为 `true`，使用 LLM 提取事实并决策 ADD/UPDATE/DELETE
    /// - 如果为 `false`，直接添加原始消息作为记忆
    /// - 如果智能组件未初始化，自动降级到简单模式
    pub infer: bool,
    /// 记忆类型（如 "procedural_memory"）
    pub memory_type: Option<String>,
    /// 自定义提示词
    pub prompt: Option<String>,
}

impl Default for AddMemoryOptions {
    fn default() -> Self {
        Self {
            user_id: None,
            agent_id: None,
            run_id: None,
            metadata: HashMap::new(),
            infer: true,  // ✅ 修改为 true，对标 Mem0，默认启用智能功能
            memory_type: None,
            prompt: None,
        }
    }
}

impl AddMemoryOptions {
    /// 🆕 Phase 1: 从options推断scope类型（不修改结构）
    /// 
    /// 根据提供的user_id, agent_id, run_id自动判断记忆作用域
    pub fn infer_scope_type(&self) -> String {
        // 优先级: Run > Agent > User > Global
        if self.run_id.is_some() {
            return "run".to_string();
        }
        if self.agent_id.is_some() && self.user_id.is_some() {
            return "agent".to_string();
        }
        if self.user_id.is_some() {
            return "user".to_string();
        }
        "global".to_string()
    }
    
    /// 🆕 Phase 1: 构建带scope的metadata（复用现有逻辑）
    /// 
    /// 将options中的信息转换为metadata，包含scope_type
    pub fn build_full_metadata(&self) -> HashMap<String, String> {
        let mut full_metadata = self.metadata.clone();
        
        // 自动添加scope信息到metadata
        full_metadata.insert("scope_type".to_string(), self.infer_scope_type());
        
        if let Some(ref user_id) = self.user_id {
            full_metadata.insert("user_id".to_string(), user_id.clone());
        }
        if let Some(ref agent_id) = self.agent_id {
            full_metadata.insert("agent_id".to_string(), agent_id.clone());
        }
        if let Some(ref run_id) = self.run_id {
            full_metadata.insert("run_id".to_string(), run_id.clone());
        }
        
        full_metadata
    }
}

/// 添加操作的结果（mem0 兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddResult {
    /// 受影响的记忆项列表（添加、更新、删除）
    pub results: Vec<MemoryEvent>,
    /// 提取的关系（如果启用了图存储）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<RelationEvent>>,
}

/// 记忆事件（ADD, UPDATE, DELETE）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// 记忆 ID
    pub id: String,
    /// 记忆内容
    pub memory: String,
    /// 事件类型：ADD, UPDATE, DELETE
    pub event: String,
    /// Actor ID（如果可用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    /// 角色（user, assistant, system）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// 关系事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEvent {
    /// 源实体
    pub source: String,
    /// 关系类型
    pub relation: String,
    /// 目标实体
    pub target: String,
}

/// 搜索记忆的选项（mem0 兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// 用户 ID 过滤
    pub user_id: Option<String>,
    /// Agent ID 过滤
    pub agent_id: Option<String>,
    /// Run ID 过滤
    pub run_id: Option<String>,
    /// 返回结果数量限制
    pub limit: Option<usize>,
    /// 最小相似度阈值 (0.0 - 1.0)
    pub threshold: Option<f32>,
    /// 额外过滤条件
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            user_id: None,
            agent_id: None,
            run_id: None,
            limit: Some(10),
            threshold: None,
            filters: None,
        }
    }
}

/// 获取所有记忆的选项（mem0 兼容）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetAllOptions {
    /// 用户 ID 过滤
    pub user_id: Option<String>,
    /// Agent ID 过滤
    pub agent_id: Option<String>,
    /// Run ID 过滤
    pub run_id: Option<String>,
    /// 返回结果数量限制
    pub limit: Option<usize>,
}

/// 删除所有记忆的选项（mem0 兼容）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeleteAllOptions {
    /// 用户 ID 过滤
    pub user_id: Option<String>,
    /// Agent ID 过滤
    pub agent_id: Option<String>,
    /// Run ID 过滤
    pub run_id: Option<String>,
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
