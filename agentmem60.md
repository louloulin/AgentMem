# AgentMem 多维度记忆管理系统改造计划

**文档版本**: 60  
**日期**: 2025-11-07  
**状态**: 🔧 规划中

---

## 🎯 改造目标

基于对**Mem0**、**MIRIX**、**AgentMem**三大平台的全面对比分析，实现AgentMem的多维度、多场景、多租户记忆管理能力。

### 核心原则

1. **🔓 灵活可选**: user_id和agent_id都可选，支持多种组合
2. **🎭 多维度**: User/Agent/Run/Session/Organization多级隔离
3. **🏢 多租户**: 企业级安全和权限控制
4. **📦 最小改动**: 保持现有API兼容，渐进式增强
5. **🚀 高性能**: 不牺牲性能，优化存储和检索

---

## 📊 现状分析

### 当前架构概览

```
Memory (统一API)
    ↓
MemoryOrchestrator (编排器)
    ↓
8个专门Managers (CoreMemoryManager, EpisodicMemoryManager, etc.)
    ↓
Storage Layer (LibSQL, PostgreSQL)
```

### 当前限制

#### 1. **user_id和agent_id处理不一致**

```rust
// types.rs - 看似都可选
pub struct AddMemoryOptions {
    pub user_id: Option<String>,    // ❌ Option但实际使用中常常必需
    pub agent_id: Option<String>,   // ❌ Option但orchestrator要求必需
    pub run_id: Option<String>,     // ✅ 真正可选
}

// orchestrator.rs - 实际要求agent_id必需
pub async fn add_memory(
    &self,
    content: String,
    agent_id: String,              // ❌ 必需参数
    user_id: Option<String>,       // ⚠️ 可选但强烈建议
    ...
)
```

**问题**:
- 接口定义和实现不一致
- 用户困惑：看起来可选，实际运行时报错
- 不支持纯user级记忆（无Agent场景）

#### 2. **缺少明确的Scope概念**

当前没有统一的"记忆作用域"抽象，导致：
- 查询逻辑分散
- 过滤器重复构建
- 难以扩展新的隔离维度

#### 3. **多租户支持不完整**

- 没有Organization级别的隔离
- 缺少权限验证机制
- 审计日志不完善

---

## 🎨 Mem0多维度设计精髓

### Mem0的核心设计

```python
def _build_filters_and_metadata(
    *,
    user_id: Optional[str] = None,    # 🟢 可选
    agent_id: Optional[str] = None,   # 🟢 可选
    run_id: Optional[str] = None,     # 🟢 可选
    actor_id: Optional[str] = None,   # 🟢 可选
    input_metadata: Optional[Dict[str, Any]] = None,
    input_filters: Optional[Dict[str, Any]] = None,
) -> tuple[Dict[str, Any], Dict[str, Any]]:
    """
    动态构建metadata和filters：
    - 没有强制要求
    - 根据提供的标识符组合
    - 适应不同场景
    """
    base_metadata_template = {}
    effective_query_filters = {}
    
    # 动态添加
    if user_id:
        base_metadata_template["user_id"] = user_id
        effective_query_filters["user_id"] = user_id
    
    if agent_id:
        base_metadata_template["agent_id"] = agent_id
        effective_query_filters["agent_id"] = agent_id
    
    if run_id:
        base_metadata_template["run_id"] = run_id
        effective_query_filters["run_id"] = run_id
    
    return base_metadata_template, effective_query_filters
```

**关键洞察**:
1. ✅ 所有标识符都是可选的
2. ✅ 动态组合，适应场景
3. ✅ 分离metadata（存储）和filters（查询）
4. ✅ 支持任意标识符组合

---

## 🏗️ AgentMem改造方案

### Phase 1: 引入MemoryScope抽象（核心）

#### 1.1 定义MemoryScope枚举

**新增文件**: `agentmen/crates/agent-mem/src/scope.rs`

```rust
//! Memory Scope - 记忆作用域抽象
//!
//! 提供灵活的多维度记忆隔离机制

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆作用域
/// 
/// 定义记忆的隔离边界和访问范围
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryScope {
    /// 全局作用域（慎用，仅用于公共知识）
    /// 
    /// 使用场景: 公共知识库、系统级信息
    Global,
    
    /// 组织级作用域（企业多租户）
    /// 
    /// 使用场景: 企业内部共享知识、团队记忆
    Organization {
        org_id: String,
        /// 可选: 进一步限定到部门或团队
        department_id: Option<String>,
    },
    
    /// 用户级作用域（最常用）
    /// 
    /// 使用场景: 个人知识库、单用户AI助手
    User {
        user_id: String,
    },
    
    /// Agent级作用域（多Agent系统）
    /// 
    /// 使用场景: 用户有多个Agent，需要隔离记忆
    Agent {
        user_id: String,
        agent_id: String,
    },
    
    /// 运行级作用域（临时会话）
    /// 
    /// 使用场景: 临时对话、一次性任务、实验性Agent
    Run {
        user_id: String,
        agent_id: Option<String>,
        run_id: String,
    },
    
    /// 会话级作用域（对话隔离）
    /// 
    /// 使用场景: 多窗口对话、上下文切换
    Session {
        user_id: String,
        agent_id: Option<String>,
        session_id: String,
    },
    
    /// 自定义作用域（最大灵活性）
    /// 
    /// 使用场景: 特殊业务逻辑、自定义隔离维度
    Custom {
        /// 自定义标识符
        identifiers: HashMap<String, String>,
    },
}

impl MemoryScope {
    /// 从选项构建Scope（向后兼容）
    /// 
    /// 根据提供的user_id, agent_id, run_id自动选择最合适的Scope
    pub fn from_options(
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        // 优先级: Run > Session > Agent > User > Global
        
        if let Some(run_id) = run_id {
            return MemoryScope::Run {
                user_id: user_id.unwrap_or_else(|| "anonymous".to_string()),
                agent_id,
                run_id,
            };
        }
        
        if let Some(session_id) = session_id {
            return MemoryScope::Session {
                user_id: user_id.unwrap_or_else(|| "anonymous".to_string()),
                agent_id,
                session_id,
            };
        }
        
        if let (Some(user_id), Some(agent_id)) = (user_id.clone(), agent_id) {
            return MemoryScope::Agent { user_id, agent_id };
        }
        
        if let Some(user_id) = user_id {
            return MemoryScope::User { user_id };
        }
        
        // 默认: Global（但会发出警告）
        tracing::warn!("No scope identifiers provided, using Global scope. This is not recommended for production.");
        MemoryScope::Global
    }
    
    /// 构建存储metadata
    /// 
    /// 将Scope转换为存储时的metadata字段
    pub fn to_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        match self {
            MemoryScope::Global => {
                metadata.insert("scope_type".to_string(), "global".to_string());
            },
            MemoryScope::Organization { org_id, department_id } => {
                metadata.insert("scope_type".to_string(), "organization".to_string());
                metadata.insert("org_id".to_string(), org_id.clone());
                if let Some(dept_id) = department_id {
                    metadata.insert("department_id".to_string(), dept_id.clone());
                }
            },
            MemoryScope::User { user_id } => {
                metadata.insert("scope_type".to_string(), "user".to_string());
                metadata.insert("user_id".to_string(), user_id.clone());
            },
            MemoryScope::Agent { user_id, agent_id } => {
                metadata.insert("scope_type".to_string(), "agent".to_string());
                metadata.insert("user_id".to_string(), user_id.clone());
                metadata.insert("agent_id".to_string(), agent_id.clone());
            },
            MemoryScope::Run { user_id, agent_id, run_id } => {
                metadata.insert("scope_type".to_string(), "run".to_string());
                metadata.insert("user_id".to_string(), user_id.clone());
                if let Some(aid) = agent_id {
                    metadata.insert("agent_id".to_string(), aid.clone());
                }
                metadata.insert("run_id".to_string(), run_id.clone());
            },
            MemoryScope::Session { user_id, agent_id, session_id } => {
                metadata.insert("scope_type".to_string(), "session".to_string());
                metadata.insert("user_id".to_string(), user_id.clone());
                if let Some(aid) = agent_id {
                    metadata.insert("agent_id".to_string(), aid.clone());
                }
                metadata.insert("session_id".to_string(), session_id.clone());
            },
            MemoryScope::Custom { identifiers } => {
                metadata.insert("scope_type".to_string(), "custom".to_string());
                for (k, v) in identifiers {
                    metadata.insert(k.clone(), v.clone());
                }
            },
        }
        
        metadata
    }
    
    /// 构建查询filters
    /// 
    /// 将Scope转换为查询时的过滤条件
    pub fn to_filters(&self) -> HashMap<String, String> {
        // 与to_metadata相同，但可能在未来有不同的逻辑
        self.to_metadata()
    }
    
    /// 获取用户ID（如果存在）
    pub fn user_id(&self) -> Option<&str> {
        match self {
            MemoryScope::Global => None,
            MemoryScope::Organization { .. } => None,
            MemoryScope::User { user_id } => Some(user_id),
            MemoryScope::Agent { user_id, .. } => Some(user_id),
            MemoryScope::Run { user_id, .. } => Some(user_id),
            MemoryScope::Session { user_id, .. } => Some(user_id),
            MemoryScope::Custom { identifiers } => identifiers.get("user_id").map(|s| s.as_str()),
        }
    }
    
    /// 获取agent_id（如果存在）
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            MemoryScope::Agent { agent_id, .. } => Some(agent_id),
            MemoryScope::Run { agent_id, .. } => agent_id.as_deref(),
            MemoryScope::Session { agent_id, .. } => agent_id.as_deref(),
            MemoryScope::Custom { identifiers } => identifiers.get("agent_id").map(|s| s.as_str()),
            _ => None,
        }
    }
    
    /// 检查是否需要自动创建Agent
    /// 
    /// 如果Scope包含agent_id但Agent不存在，应该自动创建
    pub fn requires_agent_creation(&self) -> bool {
        match self {
            MemoryScope::Agent { .. } => true,
            MemoryScope::Run { agent_id: Some(_), .. } => true,
            MemoryScope::Session { agent_id: Some(_), .. } => true,
            _ => false,
        }
    }
    
    /// 获取Scope的显示名称（用于日志和调试）
    pub fn display_name(&self) -> String {
        match self {
            MemoryScope::Global => "Global".to_string(),
            MemoryScope::Organization { org_id, department_id } => {
                if let Some(dept) = department_id {
                    format!("Org({}/{})", org_id, dept)
                } else {
                    format!("Org({})", org_id)
                }
            },
            MemoryScope::User { user_id } => format!("User({})", user_id),
            MemoryScope::Agent { user_id, agent_id } => {
                format!("Agent({}/{})", user_id, agent_id)
            },
            MemoryScope::Run { user_id, agent_id, run_id } => {
                if let Some(aid) = agent_id {
                    format!("Run({}/{}/{})", user_id, aid, run_id)
                } else {
                    format!("Run({}/{})", user_id, run_id)
                }
            },
            MemoryScope::Session { user_id, agent_id, session_id } => {
                if let Some(aid) = agent_id {
                    format!("Session({}/{}/{})", user_id, aid, session_id)
                } else {
                    format!("Session({}/{})", user_id, session_id)
                }
            },
            MemoryScope::Custom { identifiers } => {
                format!("Custom({} identifiers)", identifiers.len())
            },
        }
    }
}

impl Default for MemoryScope {
    /// 默认使用Global scope（但会发出警告）
    fn default() -> Self {
        MemoryScope::Global
    }
}

/// Scope构建器（便捷创建）
pub struct ScopeBuilder {
    user_id: Option<String>,
    agent_id: Option<String>,
    run_id: Option<String>,
    session_id: Option<String>,
    org_id: Option<String>,
    department_id: Option<String>,
    custom_identifiers: HashMap<String, String>,
}

impl ScopeBuilder {
    pub fn new() -> Self {
        Self {
            user_id: None,
            agent_id: None,
            run_id: None,
            session_id: None,
            org_id: None,
            department_id: None,
            custom_identifiers: HashMap::new(),
        }
    }
    
    pub fn user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    
    pub fn agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }
    
    pub fn run(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }
    
    pub fn session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
    
    pub fn organization(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }
    
    pub fn department(mut self, department_id: impl Into<String>) -> Self {
        self.department_id = Some(department_id.into());
        self
    }
    
    pub fn custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_identifiers.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> MemoryScope {
        // 优先级: Organization > Run > Session > Agent > User
        
        if let Some(org_id) = self.org_id {
            return MemoryScope::Organization {
                org_id,
                department_id: self.department_id,
            };
        }
        
        if let Some(run_id) = self.run_id {
            return MemoryScope::Run {
                user_id: self.user_id.unwrap_or_else(|| "anonymous".to_string()),
                agent_id: self.agent_id,
                run_id,
            };
        }
        
        if let Some(session_id) = self.session_id {
            return MemoryScope::Session {
                user_id: self.user_id.unwrap_or_else(|| "anonymous".to_string()),
                agent_id: self.agent_id,
                session_id,
            };
        }
        
        if let (Some(user_id), Some(agent_id)) = (self.user_id.clone(), self.agent_id) {
            return MemoryScope::Agent { user_id, agent_id };
        }
        
        if let Some(user_id) = self.user_id {
            return MemoryScope::User { user_id };
        }
        
        if !self.custom_identifiers.is_empty() {
            return MemoryScope::Custom {
                identifiers: self.custom_identifiers,
            };
        }
        
        MemoryScope::Global
    }
}

impl Default for ScopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scope_from_options_user_only() {
        let scope = MemoryScope::from_options(
            Some("user123".to_string()),
            None,
            None,
            None,
        );
        assert!(matches!(scope, MemoryScope::User { .. }));
        assert_eq!(scope.user_id(), Some("user123"));
    }
    
    #[test]
    fn test_scope_from_options_agent() {
        let scope = MemoryScope::from_options(
            Some("user123".to_string()),
            Some("agent456".to_string()),
            None,
            None,
        );
        assert!(matches!(scope, MemoryScope::Agent { .. }));
        assert_eq!(scope.user_id(), Some("user123"));
        assert_eq!(scope.agent_id(), Some("agent456"));
    }
    
    #[test]
    fn test_scope_builder() {
        let scope = ScopeBuilder::new()
            .user("user123")
            .agent("agent456")
            .build();
        
        assert!(matches!(scope, MemoryScope::Agent { .. }));
        assert_eq!(scope.user_id(), Some("user123"));
        assert_eq!(scope.agent_id(), Some("agent456"));
    }
    
    #[test]
    fn test_scope_to_metadata() {
        let scope = MemoryScope::Agent {
            user_id: "user123".to_string(),
            agent_id: "agent456".to_string(),
        };
        
        let metadata = scope.to_metadata();
        assert_eq!(metadata.get("scope_type"), Some(&"agent".to_string()));
        assert_eq!(metadata.get("user_id"), Some(&"user123".to_string()));
        assert_eq!(metadata.get("agent_id"), Some(&"agent456".to_string()));
    }
}
```

#### 1.2 修改AddMemoryOptions

**修改文件**: `agentmen/crates/agent-mem/src/types.rs`

```rust
/// 添加记忆选项
#[derive(Debug, Clone, Default)]
pub struct AddMemoryOptions {
    // ========== 🆕 新增: Scope优先 ==========
    /// 记忆作用域（推荐使用）
    /// 
    /// 如果提供scope，将忽略下面的user_id/agent_id/run_id/session_id
    pub scope: Option<MemoryScope>,
    
    // ========== 向后兼容: 保留旧字段 ==========
    /// 用户 ID（向后兼容，建议使用scope）
    #[deprecated(since = "3.0.0", note = "use scope instead")]
    pub user_id: Option<String>,
    
    /// Agent ID（向后兼容，建议使用scope）
    #[deprecated(since = "3.0.0", note = "use scope instead")]
    pub agent_id: Option<String>,
    
    /// Run ID（向后兼容，建议使用scope）
    #[deprecated(since = "3.0.0", note = "use scope instead")]
    pub run_id: Option<String>,
    
    // ========== 🆕 新增: Session支持 ==========
    /// Session ID（新增）
    pub session_id: Option<String>,
    
    // ========== 🆕 新增: Organization支持 ==========
    /// Organization ID（新增，企业多租户）
    pub org_id: Option<String>,
    
    /// Department ID（新增，部门隔离）
    pub department_id: Option<String>,
    
    // ========== 现有字段 ==========
    /// 元数据（支持多种类型数据）
    pub metadata: HashMap<String, String>,
    
    /// 启用智能推理（事实提取、去重等）
    pub infer: bool,
    
    /// 记忆类型（可选，自动推断）
    pub memory_type: Option<String>,
}

impl AddMemoryOptions {
    /// 获取或构建Scope
    /// 
    /// 优先使用显式的scope字段，否则从传统字段构建
    pub fn get_scope(&self) -> MemoryScope {
        if let Some(ref scope) = self.scope {
            return scope.clone();
        }
        
        // 向后兼容: 从旧字段构建
        #[allow(deprecated)]
        MemoryScope::from_options(
            self.user_id.clone(),
            self.agent_id.clone(),
            self.run_id.clone(),
            self.session_id.clone(),
        )
    }
    
    /// 便捷构造器: User级
    pub fn user(user_id: impl Into<String>) -> Self {
        Self {
            scope: Some(MemoryScope::User {
                user_id: user_id.into(),
            }),
            ..Default::default()
        }
    }
    
    /// 便捷构造器: Agent级
    pub fn agent(user_id: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            scope: Some(MemoryScope::Agent {
                user_id: user_id.into(),
                agent_id: agent_id.into(),
            }),
            ..Default::default()
        }
    }
    
    /// 便捷构造器: Run级
    pub fn run(user_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            scope: Some(MemoryScope::Run {
                user_id: user_id.into(),
                agent_id: None,
                run_id: run_id.into(),
            }),
            ..Default::default()
        }
    }
    
    /// 便捷构造器: Organization级
    pub fn organization(org_id: impl Into<String>) -> Self {
        Self {
            scope: Some(MemoryScope::Organization {
                org_id: org_id.into(),
                department_id: None,
            }),
            ..Default::default()
        }
    }
}
```

---

### Phase 2: 重构Orchestrator支持Scope（核心逻辑）

#### 2.1 修改add_memory签名

**修改文件**: `agentmen/crates/agent-mem/src/orchestrator.rs`

```rust
/// 添加记忆（新版本 - Scope优先）
pub async fn add_memory_scoped(
    &self,
    content: String,
    scope: MemoryScope,                                    // 🆕 必需: Scope
    memory_type: Option<MemoryType>,
    metadata: Option<HashMap<String, serde_json::Value>>,
) -> Result<String> {
    info!("添加记忆 (Scope模式): content={}, scope={}", 
          content, scope.display_name());
    
    // 1. 将Scope信息合并到metadata
    let mut final_metadata = metadata.unwrap_or_default();
    for (k, v) in scope.to_metadata() {
        final_metadata.insert(k, serde_json::Value::String(v));
    }
    
    // 2. 如果Scope需要Agent且Agent不存在，自动创建
    if scope.requires_agent_creation() {
        if let (Some(user_id), Some(agent_id)) = (scope.user_id(), scope.agent_id()) {
            self.ensure_agent_exists(user_id, agent_id).await?;
        }
    }
    
    // 3. 调用原有逻辑（内部实现不变）
    let memory_id = uuid::Uuid::new_v4().to_string();
    
    // ... 现有的嵌入、存储逻辑 ...
    
    Ok(memory_id)
}

/// 添加记忆（兼容版本 - 保留向后兼容）
#[deprecated(since = "3.0.0", note = "use add_memory_scoped instead")]
pub async fn add_memory(
    &self,
    content: String,
    agent_id: String,
    user_id: Option<String>,
    memory_type: Option<MemoryType>,
    metadata: Option<HashMap<String, serde_json::Value>>,
) -> Result<String> {
    // 转换为Scope调用
    let scope = MemoryScope::Agent {
        user_id: user_id.unwrap_or_else(|| "default".to_string()),
        agent_id,
    };
    
    self.add_memory_scoped(content, scope, memory_type, metadata).await
}

/// 🆕 确保Agent存在（自动创建）
async fn ensure_agent_exists(&self, user_id: &str, agent_id: &str) -> Result<()> {
    // 实现自动Agent创建逻辑（已在前面实现）
    // ...
    Ok(())
}
```

#### 2.2 修改search_memories支持Scope

```rust
/// 搜索记忆（新版本 - Scope优先）
pub async fn search_memories_scoped(
    &self,
    query: String,
    scope: MemoryScope,          // 🆕 使用Scope过滤
    limit: Option<usize>,
    threshold: Option<f32>,
) -> Result<Vec<CoreMemory>> {
    info!("搜索记忆 (Scope模式): query={}, scope={}", 
          query, scope.display_name());
    
    // 1. 从Scope构建filters
    let filters = scope.to_filters();
    
    // 2. 调用底层搜索（传入filters）
    let results = self.search_with_filters(query, filters, limit, threshold).await?;
    
    Ok(results)
}

/// 搜索记忆（兼容版本）
#[deprecated(since = "3.0.0", note = "use search_memories_scoped instead")]
pub async fn search_memories(
    &self,
    query: String,
    user_id: Option<String>,
    agent_id: Option<String>,
    run_id: Option<String>,
    limit: Option<usize>,
    threshold: Option<f32>,
) -> Result<Vec<CoreMemory>> {
    // 转换为Scope调用
    let scope = MemoryScope::from_options(user_id, agent_id, run_id, None);
    
    self.search_memories_scoped(query, scope, limit, threshold).await
}
```

---

### Phase 3: 更新Memory API（用户界面）

#### 3.1 新增Scope友好的API

**修改文件**: `agentmen/crates/agent-mem/src/memory.rs`

```rust
impl Memory {
    /// 🆕 添加记忆（Scope模式 - 推荐）
    pub async fn add_scoped(
        &self,
        content: impl Into<String>,
        scope: MemoryScope,
    ) -> Result<AddResult> {
        let content = content.into();
        debug!("添加记忆 (Scope): {}, scope={}", content, scope.display_name());
        
        let orchestrator = self.orchestrator.read().await;
        
        orchestrator.add_memory_scoped(
            content,
            scope,
            None,  // memory_type自动推断
            None,  // metadata可选
        ).await.map(|memory_id| {
            AddResult {
                events: vec![MemoryEvent::Added {
                    memory_id,
                    content: "...".to_string(),
                }],
                relations: vec![],
            }
        })
    }
    
    /// 🆕 添加记忆（带完整选项）
    pub async fn add_with_options_v2(
        &self,
        content: impl Into<String>,
        options: AddMemoryOptions,
    ) -> Result<AddResult> {
        let content = content.into();
        let scope = options.get_scope();  // 🔑 从options获取scope
        
        debug!("添加记忆: {}, scope={}", content, scope.display_name());
        
        let orchestrator = self.orchestrator.read().await;
        
        // ... 调用orchestrator.add_memory_scoped ...
        
        Ok(AddResult {
            events: vec![],
            relations: vec![],
        })
    }
    
    /// 保留现有API（向后兼容）
    #[deprecated(since = "3.0.0", note = "use add_scoped or add_with_options_v2 instead")]
    pub async fn add_with_options(
        &self,
        content: impl Into<String>,
        options: AddMemoryOptions,
    ) -> Result<AddResult> {
        // 内部转发到新API
        self.add_with_options_v2(content, options).await
    }
    
    // ========== 便捷API ==========
    
    /// 添加用户级记忆（最简单）
    pub async fn add_user_memory(
        &self,
        content: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Result<AddResult> {
        let scope = MemoryScope::User {
            user_id: user_id.into(),
        };
        self.add_scoped(content, scope).await
    }
    
    /// 添加Agent级记忆
    pub async fn add_agent_memory(
        &self,
        content: impl Into<String>,
        user_id: impl Into<String>,
        agent_id: impl Into<String>,
    ) -> Result<AddResult> {
        let scope = MemoryScope::Agent {
            user_id: user_id.into(),
            agent_id: agent_id.into(),
        };
        self.add_scoped(content, scope).await
    }
    
    /// 添加组织级记忆（企业场景）
    pub async fn add_org_memory(
        &self,
        content: impl Into<String>,
        org_id: impl Into<String>,
    ) -> Result<AddResult> {
        let scope = MemoryScope::Organization {
            org_id: org_id.into(),
            department_id: None,
        };
        self.add_scoped(content, scope).await
    }
}
```

---

### Phase 4: 更新MCP Tools（集成到MCP）

#### 4.1 修改AddMemoryTool支持Scope

**修改文件**: `agentmen/crates/agent-mem-tools/src/agentmem_tools.rs`

```rust
impl Tool for AddMemoryTool {
    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        // ... 健康检查 ...
        
        let content = args["content"].as_str()
            .ok_or_else(|| ToolError::InvalidArgument("content is required".to_string()))?;
        
        // 🆕 从参数构建Scope
        let scope = build_scope_from_args(&args)?;
        
        tracing::info!("添加记忆: content={}, scope={}", content, scope.display_name());
        
        // 🆕 确保Agent存在（如果需要）
        if scope.requires_agent_creation() {
            if let (Some(user_id), Some(agent_id)) = (scope.user_id(), scope.agent_id()) {
                ensure_agent_exists(&api_url, agent_id, user_id).await?;
            }
        }
        
        // 构建请求体
        let mut metadata_map = HashMap::new();
        for (k, v) in scope.to_metadata() {
            metadata_map.insert(k, serde_json::Value::String(v));
        }
        
        if let Some(metadata_str) = args["metadata"].as_str() {
            if let Ok(user_metadata) = serde_json::from_str::<HashMap<String, serde_json::Value>>(metadata_str) {
                metadata_map.extend(user_metadata);
            }
        }
        
        let request_body = json!({
            "content": content,
            "scope": scope.to_metadata(),
            "metadata": metadata_map,
            "memory_type": args["memory_type"].as_str().unwrap_or("Episodic"),
        });
        
        // 调用API
        // ...
        
        Ok(json!({
            "success": true,
            "message": "Memory added successfully",
            "scope": scope.display_name(),
        }))
    }
}

/// 🆕 从MCP参数构建Scope
fn build_scope_from_args(args: &Value) -> ToolResult<MemoryScope> {
    use agent_mem::scope::{MemoryScope, ScopeBuilder};
    
    // 优先使用显式的scope参数（未来）
    if let Some(scope_type) = args["scope_type"].as_str() {
        match scope_type {
            "user" => {
                let user_id = args["user_id"].as_str()
                    .ok_or_else(|| ToolError::InvalidArgument("user_id required for user scope".to_string()))?;
                return Ok(MemoryScope::User {
                    user_id: user_id.to_string(),
                });
            },
            "agent" => {
                let user_id = args["user_id"].as_str()
                    .ok_or_else(|| ToolError::InvalidArgument("user_id required for agent scope".to_string()))?;
                let agent_id = args["agent_id"].as_str()
                    .ok_or_else(|| ToolError::InvalidArgument("agent_id required for agent scope".to_string()))?;
                return Ok(MemoryScope::Agent {
                    user_id: user_id.to_string(),
                    agent_id: agent_id.to_string(),
                });
            },
            "organization" => {
                let org_id = args["org_id"].as_str()
                    .ok_or_else(|| ToolError::InvalidArgument("org_id required for organization scope".to_string()))?;
                return Ok(MemoryScope::Organization {
                    org_id: org_id.to_string(),
                    department_id: args["department_id"].as_str().map(|s| s.to_string()),
                });
            },
            _ => return Err(ToolError::InvalidArgument(format!("Unknown scope_type: {}", scope_type))),
        }
    }
    
    // 向后兼容: 从传统参数构建
    let user_id = args["user_id"].as_str().map(|s| s.to_string());
    let agent_id = args["agent_id"].as_str().map(|s| s.to_string());
    let run_id = args["run_id"].as_str().map(|s| s.to_string());
    let session_id = args["session_id"].as_str().map(|s| s.to_string());
    let org_id = args["org_id"].as_str().map(|s| s.to_string());
    
    // 如果提供了org_id，使用Organization scope
    if let Some(org_id) = org_id {
        return Ok(MemoryScope::Organization {
            org_id,
            department_id: args["department_id"].as_str().map(|s| s.to_string()),
        });
    }
    
    // 否则使用from_options构建
    Ok(MemoryScope::from_options(user_id, agent_id, run_id, session_id))
}
```

#### 4.2 更新Tool Schema

```rust
impl Tool for AddMemoryTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(self.name(), self.description())
            .add_parameter(
                "content",
                PropertySchema::string("记忆内容"),
                true,
            )
            // ========== 🆕 新增: scope_type（推荐） ==========
            .add_parameter(
                "scope_type",
                PropertySchema::string("作用域类型（可选）：user, agent, run, session, organization。如不指定则根据其他参数自动判断"),
                false,
            )
            // ========== 传统参数（向后兼容） ==========
            .add_parameter(
                "user_id",
                PropertySchema::string("用户 ID（可选，根据scope_type决定）"),
                false,
            )
            .add_parameter(
                "agent_id",
                PropertySchema::string("Agent ID（可选，用于agent/run/session scope）"),
                false,
            )
            .add_parameter(
                "run_id",
                PropertySchema::string("Run ID（可选，用于run scope）"),
                false,
            )
            // ========== 🆕 新增参数 ==========
            .add_parameter(
                "session_id",
                PropertySchema::string("Session ID（可选，用于session scope）"),
                false,
            )
            .add_parameter(
                "org_id",
                PropertySchema::string("Organization ID（可选，用于organization scope）"),
                false,
            )
            .add_parameter(
                "department_id",
                PropertySchema::string("Department ID（可选，用于organization scope的进一步隔离）"),
                false,
            )
            // ========== 其他参数 ==========
            .add_parameter(
                "memory_type",
                PropertySchema::string("记忆类型（首字母必须大写）：Episodic, Semantic, Procedural, Factual, Core, Working, Resource, Knowledge, Contextual。默认：Episodic"),
                false,
            )
            .add_parameter(
                "metadata",
                PropertySchema::string("额外的元数据（JSON 字符串，可选）"),
                false,
            )
    }
}
```

---

### Phase 5: 存储层适配（最小改动）

#### 5.1 确保metadata字段支持

当前的LibSQL和PostgreSQL存储已经支持`metadata`字段（JSON类型），无需修改表结构。

只需确保在存储时将Scope信息序列化到metadata中：

```rust
// 在存储时
let metadata_json = serde_json::to_value(final_metadata)?;

sqlx::query(
    r#"
    INSERT INTO memories (id, content, user_id, agent_id, metadata, created_at)
    VALUES (?, ?, ?, ?, ?, ?)
    "#
)
.bind(&memory_id)
.bind(&content)
.bind(scope.user_id())          // 🔑 从Scope提取
.bind(scope.agent_id())         // 🔑 从Scope提取
.bind(&metadata_json)           // 🔑 Scope信息在metadata中
.bind(Utc::now().to_rfc3339())
.execute(&self.pool)
.await?;
```

#### 5.2 查询时的Scope过滤

```rust
// 在查询时
pub async fn search_with_scope(
    &self,
    query: &str,
    scope: &MemoryScope,
    limit: usize,
) -> Result<Vec<Memory>> {
    let filters = scope.to_filters();
    
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT * FROM memories WHERE 1=1"
    );
    
    // 动态添加过滤条件
    if let Some(user_id) = scope.user_id() {
        query_builder.push(" AND user_id = ");
        query_builder.push_bind(user_id);
    }
    
    if let Some(agent_id) = scope.agent_id() {
        query_builder.push(" AND agent_id = ");
        query_builder.push_bind(agent_id);
    }
    
    // 对于更复杂的Scope（如Run, Session），从metadata过滤
    match scope {
        MemoryScope::Run { run_id, .. } => {
            query_builder.push(" AND JSON_EXTRACT(metadata, '$.run_id') = ");
            query_builder.push_bind(run_id);
        },
        MemoryScope::Session { session_id, .. } => {
            query_builder.push(" AND JSON_EXTRACT(metadata, '$.session_id') = ");
            query_builder.push_bind(session_id);
        },
        _ => {},
    }
    
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit as i64);
    
    let query = query_builder.build_query_as::<Memory>();
    let results = query.fetch_all(&self.pool).await?;
    
    Ok(results)
}
```

---

## 🎯 实施计划

### Week 1: 核心Scope实现
- [x] 创建`scope.rs`，实现`MemoryScope`枚举
- [x] 添加`ScopeBuilder`
- [x] 编写单元测试
- [ ] 文档说明和示例

### Week 2: Orchestrator重构
- [ ] 实现`add_memory_scoped`
- [ ] 实现`search_memories_scoped`
- [ ] 实现`get_all_scoped`, `delete_all_scoped`
- [ ] 实现`ensure_agent_exists`逻辑
- [ ] 保留旧API（deprecated标记）

### Week 3: Memory API更新
- [ ] 实现`add_scoped`, `add_with_options_v2`
- [ ] 实现便捷API（`add_user_memory`, `add_agent_memory`, `add_org_memory`）
- [ ] 更新`search`相关API
- [ ] 更新文档和示例

### Week 4: MCP Tools集成
- [ ] 更新`AddMemoryTool`支持Scope
- [ ] 更新`SearchMemoriesTool`支持Scope
- [ ] 更新Tool Schema
- [ ] 实现`build_scope_from_args`
- [ ] 测试MCP集成

### Week 5: 存储层适配与测试
- [ ] 验证存储层兼容性
- [ ] 实现Scope过滤查询
- [ ] 性能测试和优化
- [ ] 端到端集成测试

### Week 6: 文档与发布
- [ ] 更新官方文档
- [ ] 编写迁移指南
- [ ] 录制演示视频
- [ ] 发布AgentMem 3.0-beta

---

## 📋 代码改动清单

### 新增文件（最小改动原则）

1. `agentmen/crates/agent-mem/src/scope.rs` (~500行)
   - `MemoryScope`枚举
   - `ScopeBuilder`
   - 单元测试

### 修改文件（核心改动）

1. `agentmen/crates/agent-mem/src/lib.rs` (+2行)
   ```rust
   pub mod scope;
   pub use scope::{MemoryScope, ScopeBuilder};
   ```

2. `agentmen/crates/agent-mem/src/types.rs` (~50行改动)
   - `AddMemoryOptions`: 新增`scope`, `session_id`, `org_id`, `department_id`
   - `SearchOptions`, `GetAllOptions`, `DeleteAllOptions`: 同样改动
   - 新增便捷构造器方法

3. `agentmen/crates/agent-mem/src/orchestrator.rs` (~200行改动)
   - 新增`add_memory_scoped`, `search_memories_scoped`等Scope版本
   - 保留旧API（deprecated）
   - 新增`ensure_agent_exists`

4. `agentmen/crates/agent-mem/src/memory.rs` (~150行改动)
   - 新增`add_scoped`, `add_with_options_v2`
   - 新增便捷API（`add_user_memory`, `add_agent_memory`, `add_org_memory`）
   - 更新`search`相关API

5. `agentmen/crates/agent-mem-tools/src/agentmem_tools.rs` (~100行改动)
   - `AddMemoryTool::execute`: 使用`build_scope_from_args`
   - `SearchMemoriesTool::execute`: 支持Scope过滤
   - 新增`build_scope_from_args`函数
   - 更新Tool Schema

### 总改动量估算

- **新增代码**: ~500行（scope.rs）
- **修改代码**: ~500行（分散在5个文件）
- **总计**: ~1000行（相对AgentMem整体代码量很小）

---

## 🧪 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_add_user_memory() {
        let mem = Memory::new().await.unwrap();
        let result = mem.add_user_memory("I love pizza", "user123").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_add_agent_memory() {
        let mem = Memory::new().await.unwrap();
        let result = mem.add_agent_memory(
            "I love pizza", 
            "user123", 
            "agent456"
        ).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_add_org_memory() {
        let mem = Memory::new().await.unwrap();
        let result = mem.add_org_memory(
            "Company policy: Work from home on Fridays", 
            "org789"
        ).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_scope_isolation() {
        let mem = Memory::new().await.unwrap();
        
        // 添加到User scope
        mem.add_user_memory("User secret", "user1").await.unwrap();
        
        // 添加到Agent scope
        mem.add_agent_memory("Agent secret", "user1", "agent1").await.unwrap();
        
        // 搜索User scope - 应该只返回User secret
        let user_results = mem.search_scoped(
            "secret",
            MemoryScope::User { user_id: "user1".to_string() }
        ).await.unwrap();
        
        assert_eq!(user_results.len(), 1);
        assert!(user_results[0].content.contains("User secret"));
        
        // 搜索Agent scope - 应该只返回Agent secret
        let agent_results = mem.search_scoped(
            "secret",
            MemoryScope::Agent { 
                user_id: "user1".to_string(), 
                agent_id: "agent1".to_string() 
            }
        ).await.unwrap();
        
        assert_eq!(agent_results.len(), 1);
        assert!(agent_results[0].content.contains("Agent secret"));
    }
}
```

### 集成测试

```bash
# 创建测试脚本
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cat > test_scope_functionality.sh <<'EOF'
#!/bin/bash

# 测试User scope
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"I love pizza","scope_type":"user","user_id":"test_user_1"}}}' | \
  ./target/release/agentmem-mcp-server

# 测试Agent scope
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"I love pasta","scope_type":"agent","user_id":"test_user_1","agent_id":"test_agent_1"}}}' | \
  ./target/release/agentmem-mcp-server

# 测试Organization scope
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"Company policy","scope_type":"organization","org_id":"test_org_1"}}}' | \
  ./target/release/agentmem-mcp-server

# 测试隔离性
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"pizza","scope_type":"user","user_id":"test_user_1","limit":10}}}' | \
  ./target/release/agentmem-mcp-server
EOF

chmod +x test_scope_functionality.sh
./test_scope_functionality.sh
```

---

## 📖 使用示例

### 示例1: 个人知识库（User scope）

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 添加用户级记忆（最简单）
    mem.add_user_memory("I love pizza", "alice").await?;
    mem.add_user_memory("My favorite color is blue", "alice").await?;
    
    // 搜索（自动限定在alice的记忆）
    let results = mem.search_scoped(
        "What do you know about me?",
        MemoryScope::User { user_id: "alice".to_string() }
    ).await?;
    
    for result in results {
        println!("- {}", result.content);
    }
    
    Ok(())
}
```

### 示例2: 多Agent系统（Agent scope）

```rust
use agent_mem::{Memory, MemoryScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // Alice有两个Agent: 工作助手 和 生活助手
    
    // 工作助手的记忆
    mem.add_agent_memory(
        "Meeting with Bob at 2pm",
        "alice",
        "work_assistant"
    ).await?;
    
    // 生活助手的记忆
    mem.add_agent_memory(
        "Buy groceries after work",
        "alice",
        "life_assistant"
    ).await?;
    
    // 查询工作助手 - 只看到工作相关
    let work_memories = mem.search_scoped(
        "What's on my schedule?",
        MemoryScope::Agent {
            user_id: "alice".to_string(),
            agent_id: "work_assistant".to_string(),
        }
    ).await?;
    
    // 查询生活助手 - 只看到生活相关
    let life_memories = mem.search_scoped(
        "What do I need to do?",
        MemoryScope::Agent {
            user_id: "alice".to_string(),
            agent_id: "life_assistant".to_string(),
        }
    ).await?;
    
    Ok(())
}
```

### 示例3: 企业多租户（Organization scope）

```rust
use agent_mem::{Memory, MemoryScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 公司级记忆
    mem.add_org_memory(
        "Company holiday: Dec 25",
        "acme_corp"
    ).await?;
    
    // 部门级记忆
    mem.add_scoped(
        "Engineering team standup at 9am daily",
        MemoryScope::Organization {
            org_id: "acme_corp".to_string(),
            department_id: Some("engineering".to_string()),
        }
    ).await?;
    
    // 查询部门记忆
    let dept_memories = mem.search_scoped(
        "team meetings",
        MemoryScope::Organization {
            org_id: "acme_corp".to_string(),
            department_id: Some("engineering".to_string()),
        }
    ).await?;
    
    Ok(())
}
```

### 示例4: 临时会话（Run scope）

```rust
use agent_mem::{Memory, MemoryScope};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mem = Memory::new().await?;
    
    // 创建临时运行ID
    let run_id = Uuid::new_v4().to_string();
    
    // 临时会话记忆（用完即删）
    mem.add_scoped(
        "This is a temporary note for this run",
        MemoryScope::Run {
            user_id: "alice".to_string(),
            agent_id: Some("experiment_agent".to_string()),
            run_id: run_id.clone(),
        }
    ).await?;
    
    // 查询临时会话
    let run_memories = mem.search_scoped(
        "temporary",
        MemoryScope::Run {
            user_id: "alice".to_string(),
            agent_id: Some("experiment_agent".to_string()),
            run_id: run_id.clone(),
        }
    ).await?;
    
    // 会话结束，删除临时记忆
    mem.delete_all_scoped(MemoryScope::Run {
        user_id: "alice".to_string(),
        agent_id: Some("experiment_agent".to_string()),
        run_id,
    }).await?;
    
    Ok(())
}
```

---

## 🚀 迁移指南

### 从旧API迁移到新API

#### Before (旧API)

```rust
let mem = Memory::new().await?;

// 旧方式: 使用AddMemoryOptions
let options = AddMemoryOptions {
    user_id: Some("alice".to_string()),
    agent_id: Some("assistant".to_string()),
    run_id: None,
    metadata: HashMap::new(),
    infer: true,
    memory_type: None,
};

mem.add_with_options("I love pizza", options).await?;
```

#### After (新API - 推荐)

```rust
let mem = Memory::new().await?;

// 🎉 新方式1: 使用便捷API
mem.add_agent_memory("I love pizza", "alice", "assistant").await?;

// 🎉 新方式2: 使用Scope（最灵活）
mem.add_scoped(
    "I love pizza",
    MemoryScope::Agent {
        user_id: "alice".to_string(),
        agent_id: "assistant".to_string(),
    }
).await?;

// 🎉 新方式3: 使用ScopeBuilder
use agent_mem::ScopeBuilder;

let scope = ScopeBuilder::new()
    .user("alice")
    .agent("assistant")
    .build();

mem.add_scoped("I love pizza", scope).await?;
```

---

## ⚡ 性能优化

### 索引策略

```sql
-- User scope查询优化
CREATE INDEX idx_memories_user_id ON memories(user_id);

-- Agent scope查询优化
CREATE INDEX idx_memories_user_agent ON memories(user_id, agent_id);

-- Organization scope查询优化
CREATE INDEX idx_memories_org ON memories(
    (JSON_EXTRACT(metadata, '$.org_id'))
);

-- Run/Session scope查询优化
CREATE INDEX idx_memories_run ON memories(
    (JSON_EXTRACT(metadata, '$.run_id'))
);
```

### 查询缓存

```rust
// 实现Scope级别的查询缓存
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ScopedCache {
    cache: Arc<RwLock<HashMap<String, Vec<Memory>>>>,
}

impl ScopedCache {
    pub async fn get(&self, scope: &MemoryScope, query: &str) -> Option<Vec<Memory>> {
        let cache_key = format!("{}-{}", scope.display_name(), query);
        let cache = self.cache.read().await;
        cache.get(&cache_key).cloned()
    }
    
    pub async fn set(&self, scope: &MemoryScope, query: &str, results: Vec<Memory>) {
        let cache_key = format!("{}-{}", scope.display_name(), query);
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, results);
    }
}
```

---

## 📊 监控与可观测性

### Scope使用统计

```rust
// 添加Scope使用跟踪
pub struct ScopeMetrics {
    pub global_count: AtomicU64,
    pub user_count: AtomicU64,
    pub agent_count: AtomicU64,
    pub run_count: AtomicU64,
    pub session_count: AtomicU64,
    pub org_count: AtomicU64,
}

impl MemoryOrchestrator {
    pub async fn add_memory_scoped(&self, ...) -> Result<String> {
        // 更新metrics
        match &scope {
            MemoryScope::Global => self.metrics.global_count.fetch_add(1, Ordering::Relaxed),
            MemoryScope::User { .. } => self.metrics.user_count.fetch_add(1, Ordering::Relaxed),
            MemoryScope::Agent { .. } => self.metrics.agent_count.fetch_add(1, Ordering::Relaxed),
            // ...
        };
        
        // ... 继续原有逻辑 ...
    }
    
    pub fn get_metrics(&self) -> HashMap<String, u64> {
        use std::sync::atomic::Ordering;
        
        HashMap::from([
            ("global".to_string(), self.metrics.global_count.load(Ordering::Relaxed)),
            ("user".to_string(), self.metrics.user_count.load(Ordering::Relaxed)),
            ("agent".to_string(), self.metrics.agent_count.load(Ordering::Relaxed)),
            ("run".to_string(), self.metrics.run_count.load(Ordering::Relaxed)),
            ("session".to_string(), self.metrics.session_count.load(Ordering::Relaxed)),
            ("org".to_string(), self.metrics.org_count.load(Ordering::Relaxed)),
        ])
    }
}
```

---

## 🔒 安全性增强

### 权限验证（企业场景）

```rust
//! 权限验证模块
//! 
//! 为Organization scope提供细粒度权限控制

use crate::scope::MemoryScope;

pub trait PermissionChecker: Send + Sync {
    /// 检查用户是否有权限访问指定Scope
    fn check_access(&self, user_id: &str, scope: &MemoryScope) -> bool;
}

pub struct DefaultPermissionChecker;

impl PermissionChecker for DefaultPermissionChecker {
    fn check_access(&self, user_id: &str, scope: &MemoryScope) -> bool {
        match scope {
            // Global: 需要管理员权限
            MemoryScope::Global => self.is_admin(user_id),
            
            // User: 只能访问自己的
            MemoryScope::User { user_id: scope_user_id } => user_id == scope_user_id,
            
            // Agent: 只能访问自己的
            MemoryScope::Agent { user_id: scope_user_id, .. } => user_id == scope_user_id,
            
            // Organization: 需要是组织成员
            MemoryScope::Organization { org_id, .. } => {
                self.is_org_member(user_id, org_id)
            },
            
            // Run/Session: 只能访问自己的
            MemoryScope::Run { user_id: scope_user_id, .. } => user_id == scope_user_id,
            MemoryScope::Session { user_id: scope_user_id, .. } => user_id == scope_user_id,
            
            // Custom: 需要自定义逻辑
            MemoryScope::Custom { .. } => self.check_custom_access(user_id, scope),
        }
    }
}
```

---

## 📚 总结

### 关键改进

1. ✅ **引入MemoryScope抽象** - 统一的作用域管理
2. ✅ **user_id和agent_id都可选** - 适应多种场景
3. ✅ **多维度隔离** - User/Agent/Run/Session/Organization
4. ✅ **最小改动** - 保持向后兼容，渐进式增强
5. ✅ **性能优化** - 索引策略和查询缓存
6. ✅ **企业级特性** - 权限验证和审计日志

### 预期效果

- **用户体验**: 从"必须理解Agent"到"根据场景选择"
- **灵活性**: 支持个人、团队、企业多种场景
- **兼容性**: 旧代码无需修改，新代码更简洁
- **性能**: 通过Scope优化查询，减少无关数据扫描
- **安全性**: 细粒度权限控制，多租户隔离

---

*状态: 📋 规划完成 | 下一步: 开始实施*

