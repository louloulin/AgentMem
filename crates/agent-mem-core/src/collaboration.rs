//! 多智能体协作记忆系统
//!
//! 实现智能体间的记忆共享、协作学习和知识传播功能。
//! 基于最新的多智能体协作研究，提供企业级的协作记忆管理。

use agent_mem_traits::{AgentMemError, Result};
use agent_mem_traits::{MemoryItem, MemoryType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 协作记忆系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// 最大共享记忆池大小
    pub max_shared_pool_size: usize,
    /// 知识传播延迟（毫秒）
    pub propagation_delay_ms: u64,
    /// 冲突解决策略
    pub conflict_resolution_strategy: ConflictResolutionStrategy,
    /// 权限检查启用
    pub enable_permission_check: bool,
    /// 协作学习启用
    pub enable_collaborative_learning: bool,
    /// 知识衰减因子
    pub knowledge_decay_factor: f32,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            max_shared_pool_size: 10000,
            propagation_delay_ms: 100,
            conflict_resolution_strategy: ConflictResolutionStrategy::LastWriterWins,
            enable_permission_check: true,
            enable_collaborative_learning: true,
            knowledge_decay_factor: 0.95,
        }
    }
}

/// 冲突解决策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// 最后写入者获胜
    LastWriterWins,
    /// 基于重要性评分
    ImportanceBased,
    /// 基于访问频率
    AccessFrequencyBased,
    /// 投票机制
    VotingBased,
    /// 合并策略
    MergeBased,
}

/// 智能体权限级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentPermissionLevel {
    /// 只读权限
    ReadOnly = 1,
    /// 读写权限
    ReadWrite = 2,
    /// 管理员权限
    Admin = 3,
    /// 超级管理员权限
    SuperAdmin = 4,
}

/// 协作操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationOperation {
    /// 共享记忆
    ShareMemory {
        /// Memory ID to be shared
        memory_id: String,
        /// Target agent IDs to share with
        target_agents: Vec<String>,
        /// Permission level for target agents
        permission_level: AgentPermissionLevel,
    },
    /// 请求记忆访问
    RequestAccess {
        /// Memory ID being requested
        memory_id: String,
        /// Agent ID making the request
        requesting_agent: String,
        /// Type of access requested
        access_type: AccessType,
    },
    /// 传播知识
    PropagateKnowledge {
        /// Knowledge item to propagate
        knowledge: KnowledgeItem,
        /// Target agent IDs for propagation
        target_agents: Vec<String>,
    },
    /// 解决冲突
    ResolveConflict {
        /// Conflict ID to resolve
        conflict_id: String,
        /// Resolution strategy to apply
        resolution: ConflictResolution,
    },
}

/// 访问类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessType {
    /// Read access to memory
    Read,
    /// Write access to memory
    Write,
    /// Delete access to memory
    Delete,
    /// Share access to memory
    Share,
}

/// 知识项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// Unique identifier for the knowledge item
    pub id: String,
    /// Content of the knowledge item
    pub content: String,
    /// Type of knowledge
    pub knowledge_type: KnowledgeType,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f32,
    /// ID of the agent that created this knowledge
    pub source_agent: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// 知识类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnowledgeType {
    /// 事实性知识
    Factual,
    /// 程序性知识
    Procedural,
    /// 经验性知识
    Experiential,
    /// 元知识
    Meta,
}

/// 冲突解决方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// 冲突ID
    pub conflict_id: String,
    /// 解决方案类型
    pub resolution_type: ResolutionType,
    /// 解决后的内容
    pub resolved_content: String,
    /// 解决者代理ID
    pub resolver_agent: String,
    /// 解决时间
    pub resolution_time: DateTime<Utc>,
    /// 解决方案的置信度
    pub confidence: f32,
}

/// 解决方案类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResolutionType {
    /// 接受某个版本
    Accept,
    /// 拒绝所有版本
    Reject,
    /// 合并多个版本
    Merge,
    /// 延迟处理
    Defer,
}

/// 共享记忆池
#[derive(Debug)]
pub struct SharedMemoryPool {
    /// 共享记忆存储
    memories: Arc<RwLock<HashMap<String, SharedMemory>>>,
    /// 访问权限映射
    permissions: Arc<RwLock<HashMap<String, HashMap<String, AgentPermissionLevel>>>>,
    /// 访问历史
    access_history: Arc<RwLock<VecDeque<AccessRecord>>>,
    /// 配置
    config: CollaborationConfig,
}

/// 共享记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemory {
    /// 内存项内容
    pub memory: MemoryItem,
    /// 拥有者代理ID
    pub owner_agent: String,
    /// 共享给的代理及其权限级别
    pub shared_with: HashMap<String, AgentPermissionLevel>,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
    /// 共享元数据
    pub sharing_metadata: HashMap<String, String>,
}

/// 访问记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRecord {
    /// 代理ID
    pub agent_id: String,
    /// 内存ID
    pub memory_id: String,
    /// 访问类型
    pub access_type: AccessType,
    /// 访问时间戳
    pub timestamp: DateTime<Utc>,
    /// 访问是否成功
    pub success: bool,
    /// 访问元数据
    pub metadata: HashMap<String, String>,
}

impl SharedMemoryPool {
    /// 创建新的共享记忆池
    pub fn new(config: CollaborationConfig) -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            access_history: Arc::new(RwLock::new(VecDeque::new())),
            config,
        }
    }

    /// 添加共享记忆
    pub async fn add_shared_memory(
        &self,
        memory: MemoryItem,
        owner_agent: String,
        initial_permissions: HashMap<String, AgentPermissionLevel>,
    ) -> Result<String> {
        let mut memories = self.memories.write().await;
        let mut permissions = self.permissions.write().await;

        // 检查池大小限制
        if memories.len() >= self.config.max_shared_pool_size {
            return Err(AgentMemError::validation_error(
                "Shared memory pool is full",
            ));
        }

        let memory_id = memory.id.clone();
        let shared_memory = SharedMemory {
            memory,
            owner_agent: owner_agent.clone(),
            shared_with: initial_permissions.clone(),
            access_count: 0,
            last_accessed: Utc::now(),
            sharing_metadata: HashMap::new(),
        };

        memories.insert(memory_id.clone(), shared_memory);
        permissions.insert(memory_id.clone(), initial_permissions);

        // 记录访问历史
        self.record_access(
            owner_agent,
            memory_id.clone(),
            AccessType::Write,
            true,
            HashMap::new(),
        )
        .await;

        Ok(memory_id)
    }

    /// 获取共享记忆
    pub async fn get_shared_memory(
        &self,
        memory_id: &str,
        requesting_agent: &str,
    ) -> Result<Option<MemoryItem>> {
        let mut memories = self.memories.write().await;

        // 检查权限
        if !self
            .check_permission(memory_id, requesting_agent, &AccessType::Read)
            .await?
        {
            self.record_access(
                requesting_agent.to_string(),
                memory_id.to_string(),
                AccessType::Read,
                false,
                HashMap::from([("error".to_string(), "permission_denied".to_string())]),
            )
            .await;
            return Err(AgentMemError::validation_error("Permission denied"));
        }

        if let Some(shared_memory) = memories.get_mut(memory_id) {
            shared_memory.access_count += 1;
            shared_memory.last_accessed = Utc::now();

            // 记录成功访问
            self.record_access(
                requesting_agent.to_string(),
                memory_id.to_string(),
                AccessType::Read,
                true,
                HashMap::new(),
            )
            .await;

            Ok(Some(shared_memory.memory.clone()))
        } else {
            Ok(None)
        }
    }

    /// 检查权限
    pub async fn check_permission(
        &self,
        memory_id: &str,
        agent_id: &str,
        access_type: &AccessType,
    ) -> Result<bool> {
        if !self.config.enable_permission_check {
            return Ok(true);
        }

        let permissions = self.permissions.read().await;
        let memories = self.memories.read().await;

        // 检查是否是所有者
        if let Some(shared_memory) = memories.get(memory_id) {
            if shared_memory.owner_agent == agent_id {
                return Ok(true);
            }
        }

        // 检查共享权限
        if let Some(memory_permissions) = permissions.get(memory_id) {
            if let Some(permission_level) = memory_permissions.get(agent_id) {
                return Ok(self.check_access_permission(permission_level, access_type));
            }
        }

        Ok(false)
    }

    /// 检查访问权限
    fn check_access_permission(
        &self,
        permission_level: &AgentPermissionLevel,
        access_type: &AccessType,
    ) -> bool {
        match access_type {
            AccessType::Read => *permission_level >= AgentPermissionLevel::ReadOnly,
            AccessType::Write => *permission_level >= AgentPermissionLevel::ReadWrite,
            AccessType::Delete => *permission_level >= AgentPermissionLevel::Admin,
            AccessType::Share => *permission_level >= AgentPermissionLevel::Admin,
        }
    }

    /// 记录访问历史
    async fn record_access(
        &self,
        agent_id: String,
        memory_id: String,
        access_type: AccessType,
        success: bool,
        metadata: HashMap<String, String>,
    ) {
        let mut history = self.access_history.write().await;

        let record = AccessRecord {
            agent_id,
            memory_id,
            access_type,
            timestamp: Utc::now(),
            success,
            metadata,
        };

        history.push_back(record);

        // 保持历史记录大小限制
        while history.len() > 1000 {
            history.pop_front();
        }
    }

    /// 获取访问统计
    pub async fn get_access_statistics(&self) -> Result<AccessStatistics> {
        let history = self.access_history.read().await;
        let memories = self.memories.read().await;

        let total_accesses = history.len();
        let successful_accesses = history.iter().filter(|r| r.success).count();
        let unique_agents = history
            .iter()
            .map(|r| &r.agent_id)
            .collect::<HashSet<_>>()
            .len();
        let total_shared_memories = memories.len();

        // 按智能体统计访问次数
        let mut access_by_agent = HashMap::new();
        for record in history.iter() {
            *access_by_agent.entry(record.agent_id.clone()).or_insert(0) += 1;
        }

        // 按记忆统计访问次数
        let mut access_by_memory = HashMap::new();
        for record in history.iter() {
            *access_by_memory
                .entry(record.memory_id.clone())
                .or_insert(0) += 1;
        }

        Ok(AccessStatistics {
            total_accesses,
            successful_accesses,
            failed_accesses: total_accesses - successful_accesses,
            unique_agents,
            total_shared_memories,
            access_by_agent,
            access_by_memory,
        })
    }
}

/// 访问统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessStatistics {
    pub total_accesses: usize,
    pub successful_accesses: usize,
    pub failed_accesses: usize,
    pub unique_agents: usize,
    pub total_shared_memories: usize,
    pub access_by_agent: HashMap<String, u64>,
    pub access_by_memory: HashMap<String, u64>,
}

/// 权限管理器
#[derive(Debug)]
pub struct PermissionManager {
    /// 智能体权限映射
    agent_permissions: Arc<RwLock<HashMap<String, AgentPermissionLevel>>>,
    /// 角色定义
    roles: Arc<RwLock<HashMap<String, Role>>>,
    /// 权限策略
    policies: Arc<RwLock<Vec<PermissionPolicy>>>,
}

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>,
    pub inherits_from: Vec<String>,
}

/// 权限策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub id: String,
    pub name: String,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub priority: u32,
}

/// 策略条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    AgentId(String),
    AgentRole(String),
    MemoryType(MemoryType),
    TimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    AccessCount {
        min: u64,
        max: u64,
    },
    And(Vec<PolicyCondition>),
    Or(Vec<PolicyCondition>),
}

/// 策略动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireApproval,
    Log,
    Throttle {
        max_requests: u32,
        window_seconds: u32,
    },
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionManager {
    /// 创建新的权限管理器
    pub fn new() -> Self {
        Self {
            agent_permissions: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 设置智能体权限
    pub async fn set_agent_permission(
        &self,
        agent_id: String,
        permission_level: AgentPermissionLevel,
    ) -> Result<()> {
        let mut permissions = self.agent_permissions.write().await;
        permissions.insert(agent_id, permission_level);
        Ok(())
    }

    /// 获取智能体权限
    pub async fn get_agent_permission(&self, agent_id: &str) -> Result<AgentPermissionLevel> {
        let permissions = self.agent_permissions.read().await;
        Ok(permissions
            .get(agent_id)
            .cloned()
            .unwrap_or(AgentPermissionLevel::ReadOnly))
    }

    /// 添加角色
    pub async fn add_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// 添加权限策略
    pub async fn add_policy(&self, policy: PermissionPolicy) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.push(policy);
        // 按优先级排序
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }

    /// 评估权限请求
    pub async fn evaluate_permission(
        &self,
        agent_id: &str,
        memory_id: &str,
        access_type: &AccessType,
        context: &PermissionContext,
    ) -> Result<PermissionDecision> {
        let policies = self.policies.read().await;

        for policy in policies.iter() {
            if self.evaluate_condition(
                &policy.condition,
                agent_id,
                memory_id,
                access_type,
                context,
            )? {
                return Ok(PermissionDecision {
                    allowed: matches!(policy.action, PolicyAction::Allow),
                    policy_id: Some(policy.id.clone()),
                    reason: format!("Matched policy: {}", policy.name),
                    requires_approval: matches!(policy.action, PolicyAction::RequireApproval),
                });
            }
        }

        // 默认基于智能体权限级别
        let permission_level = self.get_agent_permission(agent_id).await?;
        let allowed = match access_type {
            AccessType::Read => permission_level >= AgentPermissionLevel::ReadOnly,
            AccessType::Write => permission_level >= AgentPermissionLevel::ReadWrite,
            AccessType::Delete => permission_level >= AgentPermissionLevel::Admin,
            AccessType::Share => permission_level >= AgentPermissionLevel::Admin,
        };

        Ok(PermissionDecision {
            allowed,
            policy_id: None,
            reason: format!("Default permission level: {permission_level:?}"),
            requires_approval: false,
        })
    }

    /// 评估策略条件
    fn evaluate_condition(
        &self,
        condition: &PolicyCondition,
        agent_id: &str,
        _memory_id: &str,
        _access_type: &AccessType,
        context: &PermissionContext,
    ) -> Result<bool> {
        match condition {
            PolicyCondition::AgentId(id) => Ok(agent_id == id),
            PolicyCondition::AgentRole(role) => {
                // 简化实现，实际应该检查智能体的角色
                Ok(context.agent_roles.contains(role))
            }
            PolicyCondition::MemoryType(memory_type) => {
                Ok(context.memory_type.as_ref() == Some(memory_type))
            }
            PolicyCondition::TimeRange { start, end } => {
                let now = Utc::now();
                Ok(now >= *start && now <= *end)
            }
            PolicyCondition::AccessCount { min, max } => {
                Ok(context.access_count >= *min && context.access_count <= *max)
            }
            PolicyCondition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(
                        cond,
                        agent_id,
                        _memory_id,
                        _access_type,
                        context,
                    )? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            PolicyCondition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, agent_id, _memory_id, _access_type, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

/// 权限上下文
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub agent_roles: HashSet<String>,
    pub memory_type: Option<MemoryType>,
    pub access_count: u64,
    pub metadata: HashMap<String, String>,
}

/// 权限决策
#[derive(Debug, Clone)]
pub struct PermissionDecision {
    pub allowed: bool,
    pub policy_id: Option<String>,
    pub reason: String,
    pub requires_approval: bool,
}

/// 冲突解决器
#[derive(Debug)]
pub struct ConflictResolver {
    /// 冲突记录
    conflicts: Arc<RwLock<HashMap<String, MemoryConflict>>>,
    /// 解决策略
    strategy: ConflictResolutionStrategy,
    /// 投票系统
    voting_system: Arc<VotingSystem>,
}

/// 记忆冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConflict {
    pub conflict_id: String,
    pub memory_id: String,
    pub conflicting_versions: Vec<ConflictingVersion>,
    pub detected_at: DateTime<Utc>,
    pub status: ConflictStatus,
    pub resolution: Option<ConflictResolution>,
}

/// 冲突版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictingVersion {
    pub version_id: String,
    pub content: String,
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    pub importance_score: f32,
    pub access_count: u64,
}

/// 冲突状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictStatus {
    Detected,
    UnderReview,
    Resolved,
    Escalated,
}

/// 投票系统
#[derive(Debug)]
pub struct VotingSystem {
    /// 投票记录
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>,
    /// 投票权重
    voter_weights: Arc<RwLock<HashMap<String, f32>>>,
}

/// 投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub conflict_id: String,
    pub preferred_version: String,
    pub confidence: f32,
    pub reasoning: String,
    pub timestamp: DateTime<Utc>,
}

impl ConflictResolver {
    /// 创建新的冲突解决器
    pub fn new(strategy: ConflictResolutionStrategy) -> Self {
        Self {
            conflicts: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            voting_system: Arc::new(VotingSystem::new()),
        }
    }

    /// 检测冲突
    pub async fn detect_conflict(
        &self,
        memory_id: String,
        versions: Vec<ConflictingVersion>,
    ) -> Result<String> {
        let conflict_id = format!("conflict_{}", uuid::Uuid::new_v4());

        let conflict = MemoryConflict {
            conflict_id: conflict_id.clone(),
            memory_id,
            conflicting_versions: versions,
            detected_at: Utc::now(),
            status: ConflictStatus::Detected,
            resolution: None,
        };

        let mut conflicts = self.conflicts.write().await;
        conflicts.insert(conflict_id.clone(), conflict);

        Ok(conflict_id)
    }

    /// 解决冲突
    pub async fn resolve_conflict(&self, conflict_id: &str) -> Result<ConflictResolution> {
        let mut conflicts = self.conflicts.write().await;

        let conflict = conflicts
            .get_mut(conflict_id)
            .ok_or_else(|| AgentMemError::validation_error("Conflict not found"))?;

        let resolution = match &self.strategy {
            ConflictResolutionStrategy::LastWriterWins => {
                self.resolve_last_writer_wins(conflict).await?
            }
            ConflictResolutionStrategy::ImportanceBased => {
                self.resolve_importance_based(conflict).await?
            }
            ConflictResolutionStrategy::AccessFrequencyBased => {
                self.resolve_access_frequency_based(conflict).await?
            }
            ConflictResolutionStrategy::VotingBased => self.resolve_voting_based(conflict).await?,
            ConflictResolutionStrategy::MergeBased => self.resolve_merge_based(conflict).await?,
        };

        conflict.resolution = Some(resolution.clone());
        conflict.status = ConflictStatus::Resolved;

        Ok(resolution)
    }

    /// 最后写入者获胜解决策略
    async fn resolve_last_writer_wins(
        &self,
        conflict: &MemoryConflict,
    ) -> Result<ConflictResolution> {
        let latest_version = conflict
            .conflicting_versions
            .iter()
            .max_by_key(|v| v.timestamp)
            .ok_or_else(|| AgentMemError::validation_error("No versions found"))?;

        Ok(ConflictResolution {
            conflict_id: conflict.conflict_id.clone(),
            resolution_type: ResolutionType::Accept,
            resolved_content: latest_version.content.clone(),
            resolver_agent: "system".to_string(),
            resolution_time: Utc::now(),
            confidence: 0.8,
        })
    }

    /// 基于重要性解决策略
    async fn resolve_importance_based(
        &self,
        conflict: &MemoryConflict,
    ) -> Result<ConflictResolution> {
        let most_important = conflict
            .conflicting_versions
            .iter()
            .max_by(|a, b| a.importance_score.partial_cmp(&b.importance_score).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| AgentMemError::validation_error("No versions found"))?;

        Ok(ConflictResolution {
            conflict_id: conflict.conflict_id.clone(),
            resolution_type: ResolutionType::Accept,
            resolved_content: most_important.content.clone(),
            resolver_agent: "system".to_string(),
            resolution_time: Utc::now(),
            confidence: most_important.importance_score,
        })
    }

    /// 基于访问频率解决策略
    async fn resolve_access_frequency_based(
        &self,
        conflict: &MemoryConflict,
    ) -> Result<ConflictResolution> {
        let most_accessed = conflict
            .conflicting_versions
            .iter()
            .max_by_key(|v| v.access_count)
            .ok_or_else(|| AgentMemError::validation_error("No versions found"))?;

        Ok(ConflictResolution {
            conflict_id: conflict.conflict_id.clone(),
            resolution_type: ResolutionType::Accept,
            resolved_content: most_accessed.content.clone(),
            resolver_agent: "system".to_string(),
            resolution_time: Utc::now(),
            confidence: (most_accessed.access_count as f32 / 100.0).min(1.0),
        })
    }

    /// 基于投票解决策略
    async fn resolve_voting_based(&self, conflict: &MemoryConflict) -> Result<ConflictResolution> {
        let votes = self.voting_system.get_votes(&conflict.conflict_id).await?;

        if votes.is_empty() {
            // 如果没有投票，回退到重要性策略
            return self.resolve_importance_based(conflict).await;
        }

        // 计算加权投票结果
        let mut vote_scores: HashMap<String, f32> = HashMap::new();
        for vote in &votes {
            let weight = self.voting_system.get_voter_weight(&vote.voter_id).await?;
            *vote_scores
                .entry(vote.preferred_version.clone())
                .or_insert(0.0) += weight * vote.confidence;
        }

        let winning_version = vote_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(version_id, _)| version_id)
            .ok_or_else(|| AgentMemError::validation_error("No winning version"))?;

        let winning_content = conflict
            .conflicting_versions
            .iter()
            .find(|v| &v.version_id == winning_version)
            .map(|v| v.content.clone())
            .ok_or_else(|| AgentMemError::validation_error("Winning version not found"))?;

        Ok(ConflictResolution {
            conflict_id: conflict.conflict_id.clone(),
            resolution_type: ResolutionType::Accept,
            resolved_content: winning_content,
            resolver_agent: "voting_system".to_string(),
            resolution_time: Utc::now(),
            confidence: *vote_scores.get(winning_version).unwrap_or(&0.0),
        })
    }

    /// 基于合并解决策略
    async fn resolve_merge_based(&self, conflict: &MemoryConflict) -> Result<ConflictResolution> {
        // 简化的合并策略：连接所有版本的内容
        let merged_content = conflict
            .conflicting_versions
            .iter()
            .map(|v| v.content.as_str())
            .collect::<Vec<_>>()
            .join(" | ");

        Ok(ConflictResolution {
            conflict_id: conflict.conflict_id.clone(),
            resolution_type: ResolutionType::Merge,
            resolved_content: merged_content,
            resolver_agent: "merge_system".to_string(),
            resolution_time: Utc::now(),
            confidence: 0.6,
        })
    }

    /// 获取冲突统计
    pub async fn get_conflict_statistics(&self) -> Result<ConflictStatistics> {
        let conflicts = self.conflicts.read().await;

        let total_conflicts = conflicts.len();
        let resolved_conflicts = conflicts
            .values()
            .filter(|c| matches!(c.status, ConflictStatus::Resolved))
            .count();
        let pending_conflicts = total_conflicts - resolved_conflicts;

        // 按解决类型统计
        let mut resolution_by_type = HashMap::new();
        for conflict in conflicts.values() {
            if let Some(resolution) = &conflict.resolution {
                *resolution_by_type
                    .entry(resolution.resolution_type.clone())
                    .or_insert(0) += 1;
            }
        }

        Ok(ConflictStatistics {
            total_conflicts,
            resolved_conflicts,
            pending_conflicts,
            resolution_by_type,
            average_resolution_time_seconds: self
                .calculate_average_resolution_time(&conflicts)
                .await,
        })
    }

    /// 计算平均解决时间
    async fn calculate_average_resolution_time(
        &self,
        conflicts: &HashMap<String, MemoryConflict>,
    ) -> f64 {
        let resolved_conflicts: Vec<_> = conflicts
            .values()
            .filter(|c| matches!(c.status, ConflictStatus::Resolved))
            .collect();

        if resolved_conflicts.is_empty() {
            return 0.0;
        }

        let total_time: i64 = resolved_conflicts
            .iter()
            .filter_map(|c| {
                c.resolution
                    .as_ref()
                    .map(|r| (r.resolution_time - c.detected_at).num_seconds())
            })
            .sum();

        total_time as f64 / resolved_conflicts.len() as f64
    }
}

impl Default for VotingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VotingSystem {
    /// 创建新的投票系统
    pub fn new() -> Self {
        Self {
            votes: Arc::new(RwLock::new(HashMap::new())),
            voter_weights: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加投票
    pub async fn add_vote(&self, vote: Vote) -> Result<()> {
        let mut votes = self.votes.write().await;
        votes
            .entry(vote.conflict_id.clone())
            .or_insert_with(Vec::new)
            .push(vote);
        Ok(())
    }

    /// 获取投票
    pub async fn get_votes(&self, conflict_id: &str) -> Result<Vec<Vote>> {
        let votes = self.votes.read().await;
        Ok(votes.get(conflict_id).cloned().unwrap_or_default())
    }

    /// 设置投票者权重
    pub async fn set_voter_weight(&self, voter_id: String, weight: f32) -> Result<()> {
        let mut weights = self.voter_weights.write().await;
        weights.insert(voter_id, weight);
        Ok(())
    }

    /// 获取投票者权重
    pub async fn get_voter_weight(&self, voter_id: &str) -> Result<f32> {
        let weights = self.voter_weights.read().await;
        Ok(weights.get(voter_id).cloned().unwrap_or(1.0))
    }
}

/// 冲突统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictStatistics {
    pub total_conflicts: usize,
    pub resolved_conflicts: usize,
    pub pending_conflicts: usize,
    pub resolution_by_type: HashMap<ResolutionType, u32>,
    pub average_resolution_time_seconds: f64,
}

/// 知识传播器
#[derive(Debug)]
pub struct KnowledgePropagator {
    /// 知识库
    knowledge_base: Arc<RwLock<HashMap<String, KnowledgeItem>>>,
    /// 传播历史
    propagation_history: Arc<RwLock<VecDeque<PropagationRecord>>>,
    /// 订阅关系
    subscriptions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// 传播配置
    config: PropagationConfig,
}

/// 传播配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationConfig {
    /// 最大传播延迟（毫秒）
    pub max_propagation_delay_ms: u64,
    /// 批量传播大小
    pub batch_size: usize,
    /// 知识衰减启用
    pub enable_knowledge_decay: bool,
    /// 衰减因子
    pub decay_factor: f32,
    /// 最小置信度阈值
    pub min_confidence_threshold: f32,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            max_propagation_delay_ms: 1000,
            batch_size: 100,
            enable_knowledge_decay: true,
            decay_factor: 0.95,
            min_confidence_threshold: 0.3,
        }
    }
}

/// 传播记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationRecord {
    pub knowledge_id: String,
    pub source_agent: String,
    pub target_agents: Vec<String>,
    pub propagated_at: DateTime<Utc>,
    pub success_count: u32,
    pub failure_count: u32,
    pub metadata: HashMap<String, String>,
}

impl KnowledgePropagator {
    /// 创建新的知识传播器
    pub fn new(config: PropagationConfig) -> Self {
        Self {
            knowledge_base: Arc::new(RwLock::new(HashMap::new())),
            propagation_history: Arc::new(RwLock::new(VecDeque::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 添加知识
    pub async fn add_knowledge(&self, knowledge: KnowledgeItem) -> Result<()> {
        let mut knowledge_base = self.knowledge_base.write().await;
        knowledge_base.insert(knowledge.id.clone(), knowledge);
        Ok(())
    }

    /// 传播知识
    pub async fn propagate_knowledge(
        &self,
        knowledge_id: &str,
        target_agents: Vec<String>,
    ) -> Result<PropagationResult> {
        let knowledge_base = self.knowledge_base.read().await;
        let knowledge = knowledge_base
            .get(knowledge_id)
            .ok_or_else(|| AgentMemError::validation_error("Knowledge not found"))?;

        // 检查置信度阈值
        if knowledge.confidence_score < self.config.min_confidence_threshold {
            let total_targets = target_agents.len();
            return Ok(PropagationResult {
                knowledge_id: knowledge_id.to_string(),
                successful_targets: Vec::new(),
                failed_targets: target_agents,
                total_targets,
                propagation_time_ms: 0,
            });
        }

        let start_time = std::time::Instant::now();
        let mut successful_targets = Vec::new();
        let mut failed_targets = Vec::new();

        // 模拟传播过程
        for target_agent in &target_agents {
            // 检查订阅关系
            if self
                .is_subscribed(target_agent, &knowledge.knowledge_type)
                .await?
            {
                // 应用知识衰减
                let _decayed_knowledge = if self.config.enable_knowledge_decay {
                    self.apply_knowledge_decay(knowledge.clone()).await?
                } else {
                    knowledge.clone()
                };

                // 模拟传播延迟
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    self.config.max_propagation_delay_ms / target_agents.len() as u64,
                ))
                .await;

                // 传播成功（简化实现）
                successful_targets.push(target_agent.clone());
            } else {
                failed_targets.push(target_agent.clone());
            }
        }

        let propagation_time_ms = start_time.elapsed().as_millis() as u64;

        // 记录传播历史
        self.record_propagation(
            knowledge_id.to_string(),
            knowledge.source_agent.clone(),
            target_agents.clone(),
            successful_targets.len() as u32,
            failed_targets.len() as u32,
        )
        .await;

        Ok(PropagationResult {
            knowledge_id: knowledge_id.to_string(),
            successful_targets,
            failed_targets,
            total_targets: target_agents.len(),
            propagation_time_ms,
        })
    }

    /// 订阅知识类型
    pub async fn subscribe(&self, agent_id: String, knowledge_type: KnowledgeType) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        let knowledge_type_str = format!("{knowledge_type:?}");
        subscriptions
            .entry(agent_id)
            .or_insert_with(HashSet::new)
            .insert(knowledge_type_str);
        Ok(())
    }

    /// 检查订阅关系
    async fn is_subscribed(&self, agent_id: &str, knowledge_type: &KnowledgeType) -> Result<bool> {
        let subscriptions = self.subscriptions.read().await;
        let knowledge_type_str = format!("{knowledge_type:?}");
        Ok(subscriptions
            .get(agent_id)
            .map(|subs| subs.contains(&knowledge_type_str))
            .unwrap_or(false))
    }

    /// 应用知识衰减
    async fn apply_knowledge_decay(&self, mut knowledge: KnowledgeItem) -> Result<KnowledgeItem> {
        let age_days = (Utc::now() - knowledge.created_at).num_days() as f32;
        let decay_factor = self.config.decay_factor.powf(age_days);
        knowledge.confidence_score *= decay_factor;
        Ok(knowledge)
    }

    /// 记录传播历史
    async fn record_propagation(
        &self,
        knowledge_id: String,
        source_agent: String,
        target_agents: Vec<String>,
        success_count: u32,
        failure_count: u32,
    ) {
        let mut history = self.propagation_history.write().await;

        let record = PropagationRecord {
            knowledge_id,
            source_agent,
            target_agents,
            propagated_at: Utc::now(),
            success_count,
            failure_count,
            metadata: HashMap::new(),
        };

        history.push_back(record);

        // 保持历史记录大小限制
        while history.len() > 1000 {
            history.pop_front();
        }
    }

    /// 获取传播统计
    pub async fn get_propagation_statistics(&self) -> Result<PropagationStatistics> {
        let history = self.propagation_history.read().await;
        let knowledge_base = self.knowledge_base.read().await;

        let total_propagations = history.len();
        let total_successes: u32 = history.iter().map(|r| r.success_count).sum();
        let total_failures: u32 = history.iter().map(|r| r.failure_count).sum();
        let total_knowledge_items = knowledge_base.len();

        // 按知识类型统计
        let mut knowledge_by_type = HashMap::new();
        for knowledge in knowledge_base.values() {
            *knowledge_by_type
                .entry(knowledge.knowledge_type.clone())
                .or_insert(0) += 1;
        }

        // 按智能体统计传播次数
        let mut propagations_by_agent = HashMap::new();
        for record in history.iter() {
            *propagations_by_agent
                .entry(record.source_agent.clone())
                .or_insert(0) += 1;
        }

        Ok(PropagationStatistics {
            total_propagations,
            total_successes,
            total_failures,
            success_rate: if total_successes + total_failures > 0 {
                total_successes as f32 / (total_successes + total_failures) as f32
            } else {
                0.0
            },
            total_knowledge_items,
            knowledge_by_type,
            propagations_by_agent,
        })
    }
}

/// 传播结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub knowledge_id: String,
    pub successful_targets: Vec<String>,
    pub failed_targets: Vec<String>,
    pub total_targets: usize,
    pub propagation_time_ms: u64,
}

/// 传播统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationStatistics {
    pub total_propagations: usize,
    pub total_successes: u32,
    pub total_failures: u32,
    pub success_rate: f32,
    pub total_knowledge_items: usize,
    pub knowledge_by_type: HashMap<KnowledgeType, u32>,
    pub propagations_by_agent: HashMap<String, u32>,
}

/// 协作记忆系统
#[derive(Debug)]
pub struct CollaborativeMemorySystem {
    /// 共享记忆池
    shared_memory_pool: Arc<SharedMemoryPool>,
    /// 权限管理器
    permission_manager: Arc<PermissionManager>,
    /// 冲突解决器
    conflict_resolver: Arc<ConflictResolver>,
    /// 知识传播器
    knowledge_propagator: Arc<KnowledgePropagator>,
    /// 系统配置
    config: CollaborationConfig,
}

impl CollaborativeMemorySystem {
    /// 创建新的协作记忆系统
    pub fn new(config: CollaborationConfig) -> Self {
        let propagation_config = PropagationConfig {
            max_propagation_delay_ms: config.propagation_delay_ms,
            decay_factor: config.knowledge_decay_factor,
            ..Default::default()
        };

        Self {
            shared_memory_pool: Arc::new(SharedMemoryPool::new(config.clone())),
            permission_manager: Arc::new(PermissionManager::new()),
            conflict_resolver: Arc::new(ConflictResolver::new(
                config.conflict_resolution_strategy.clone(),
            )),
            knowledge_propagator: Arc::new(KnowledgePropagator::new(propagation_config)),
            config,
        }
    }

    /// 执行协作操作
    pub async fn execute_operation(
        &self,
        operation: CollaborationOperation,
    ) -> Result<OperationResult> {
        match operation {
            CollaborationOperation::ShareMemory {
                memory_id,
                target_agents,
                permission_level,
            } => {
                self.share_memory(memory_id, target_agents, permission_level)
                    .await
            }
            CollaborationOperation::RequestAccess {
                memory_id,
                requesting_agent,
                access_type,
            } => {
                self.request_access(memory_id, requesting_agent, access_type)
                    .await
            }
            CollaborationOperation::PropagateKnowledge {
                knowledge,
                target_agents,
            } => self.propagate_knowledge(knowledge, target_agents).await,
            CollaborationOperation::ResolveConflict {
                conflict_id,
                resolution,
            } => self.resolve_conflict(conflict_id, resolution).await,
        }
    }

    /// 共享记忆
    async fn share_memory(
        &self,
        memory_id: String,
        target_agents: Vec<String>,
        permission_level: AgentPermissionLevel,
    ) -> Result<OperationResult> {
        // 检查记忆是否存在
        let memory = self
            .shared_memory_pool
            .get_shared_memory(&memory_id, "system")
            .await?;
        if memory.is_none() {
            return Ok(OperationResult::Error {
                message: "Memory not found".to_string(),
            });
        }

        // 为每个目标智能体设置权限
        for agent_id in &target_agents {
            self.permission_manager
                .set_agent_permission(agent_id.clone(), permission_level.clone())
                .await?;
        }

        Ok(OperationResult::ShareMemory {
            memory_id,
            shared_with: target_agents,
            permission_level,
        })
    }

    /// 请求访问
    async fn request_access(
        &self,
        memory_id: String,
        requesting_agent: String,
        access_type: AccessType,
    ) -> Result<OperationResult> {
        let context = PermissionContext {
            agent_roles: HashSet::new(),
            memory_type: None,
            access_count: 0,
            metadata: HashMap::new(),
        };

        let decision = self
            .permission_manager
            .evaluate_permission(&requesting_agent, &memory_id, &access_type, &context)
            .await?;

        if decision.allowed {
            // 执行实际访问
            let memory = self
                .shared_memory_pool
                .get_shared_memory(&memory_id, &requesting_agent)
                .await?;
            Ok(OperationResult::AccessGranted {
                memory_id,
                requesting_agent,
                access_type,
                memory: memory.map(Box::new),
            })
        } else {
            Ok(OperationResult::AccessDenied {
                memory_id,
                requesting_agent,
                reason: decision.reason,
            })
        }
    }

    /// 传播知识
    async fn propagate_knowledge(
        &self,
        knowledge: KnowledgeItem,
        target_agents: Vec<String>,
    ) -> Result<OperationResult> {
        // 添加知识到传播器
        self.knowledge_propagator
            .add_knowledge(knowledge.clone())
            .await?;

        // 执行传播
        let result = self
            .knowledge_propagator
            .propagate_knowledge(&knowledge.id, target_agents)
            .await?;

        Ok(OperationResult::KnowledgePropagated {
            knowledge_id: knowledge.id,
            propagation_result: result,
        })
    }

    /// 解决冲突
    async fn resolve_conflict(
        &self,
        conflict_id: String,
        _resolution: ConflictResolution,
    ) -> Result<OperationResult> {
        let resolution = self
            .conflict_resolver
            .resolve_conflict(&conflict_id)
            .await?;

        Ok(OperationResult::ConflictResolved {
            conflict_id,
            resolution,
        })
    }

    /// 获取系统统计
    pub async fn get_system_statistics(&self) -> Result<CollaborationStatistics> {
        let access_stats = self.shared_memory_pool.get_access_statistics().await?;
        let conflict_stats = self.conflict_resolver.get_conflict_statistics().await?;
        let propagation_stats = self
            .knowledge_propagator
            .get_propagation_statistics()
            .await?;

        Ok(CollaborationStatistics {
            access_statistics: access_stats,
            conflict_statistics: conflict_stats,
            propagation_statistics: propagation_stats,
            system_uptime_seconds: 0,      // 简化实现
            active_agents: HashSet::new(), // 简化实现
        })
    }

    /// 启动协作学习
    pub async fn start_collaborative_learning(&self) -> Result<()> {
        if !self.config.enable_collaborative_learning {
            return Ok(());
        }

        // 启动后台任务进行协作学习
        // 这里是简化实现，实际应该启动独立的任务
        tokio::spawn(async move {
            // 协作学习逻辑
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                // 执行协作学习算法
            }
        });

        Ok(())
    }

    /// 获取共享记忆池的引用
    pub fn shared_memory_pool(&self) -> &SharedMemoryPool {
        &self.shared_memory_pool
    }

    /// 获取权限管理器的引用
    pub fn permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }

    /// 获取冲突解决器的引用
    pub fn conflict_resolver(&self) -> &ConflictResolver {
        &self.conflict_resolver
    }

    /// 获取知识传播器的引用
    pub fn knowledge_propagator(&self) -> &KnowledgePropagator {
        &self.knowledge_propagator
    }
}

/// 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    ShareMemory {
        memory_id: String,
        shared_with: Vec<String>,
        permission_level: AgentPermissionLevel,
    },
    AccessGranted {
        memory_id: String,
        requesting_agent: String,
        access_type: AccessType,
        memory: Option<Box<MemoryItem>>,
    },
    AccessDenied {
        memory_id: String,
        requesting_agent: String,
        reason: String,
    },
    KnowledgePropagated {
        knowledge_id: String,
        propagation_result: PropagationResult,
    },
    ConflictResolved {
        conflict_id: String,
        resolution: ConflictResolution,
    },
    Error {
        message: String,
    },
}

/// 协作统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationStatistics {
    pub access_statistics: AccessStatistics,
    pub conflict_statistics: ConflictStatistics,
    pub propagation_statistics: PropagationStatistics,
    pub system_uptime_seconds: u64,
    /// Set of currently active agent IDs
    pub active_agents: HashSet<String>,
}
