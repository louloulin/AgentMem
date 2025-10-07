//! Tool Manager - 工具管理器
//!
//! 参考 MIRIX 的 ToolManager 实现，提供完整的工具管理功能：
//! 1. 工具注册和发现
//! 2. 工具权限管理
//! 3. 工具执行和监控
//! 4. 工具版本管理
//!
//! 学习自 MIRIX: mirix/services/tool_manager.py

use crate::storage::models::Tool;
use crate::storage::tool_repository::ToolRepository;
use crate::{CoreError, CoreResult};
use agent_mem_tools::{ToolExecutor, ToolSchema};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 工具类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// 核心工具（内置）
    Core,
    /// 记忆工具
    Memory,
    /// 自定义工具
    Custom,
    /// MCP 工具
    Mcp,
    /// 外部工具
    External,
}

impl ToolType {
    pub fn as_str(&self) -> &str {
        match self {
            ToolType::Core => "core",
            ToolType::Memory => "memory",
            ToolType::Custom => "custom",
            ToolType::Mcp => "mcp",
            ToolType::External => "external",
        }
    }

    pub fn from_str(s: &str) -> CoreResult<Self> {
        match s {
            "core" => Ok(ToolType::Core),
            "memory" => Ok(ToolType::Memory),
            "custom" => Ok(ToolType::Custom),
            "mcp" => Ok(ToolType::Mcp),
            "external" => Ok(ToolType::External),
            _ => Err(CoreError::InvalidInput(format!("Invalid tool type: {}", s))),
        }
    }
}

/// 工具管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManagerConfig {
    /// 是否启用工具缓存
    pub enable_cache: bool,
    /// 缓存过期时间（秒）
    pub cache_ttl_seconds: u64,
    /// 是否自动注册内置工具
    pub auto_register_builtin: bool,
    /// 最大并发工具执行数
    pub max_concurrent_executions: usize,
}

impl Default for ToolManagerConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl_seconds: 3600,
            auto_register_builtin: true,
            max_concurrent_executions: 10,
        }
    }
}

/// 工具创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateToolRequest {
    pub name: String,
    pub description: Option<String>,
    pub tool_type: String,
    pub source_code: Option<String>,
    pub source_type: Option<String>,
    pub json_schema: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

/// 工具更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateToolRequest {
    pub description: Option<String>,
    pub source_code: Option<String>,
    pub json_schema: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub is_enabled: Option<bool>,
}

/// 工具统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStats {
    pub total_tools: usize,
    pub by_type: HashMap<String, usize>,
    pub enabled_count: usize,
    pub disabled_count: usize,
}

/// 工具管理器
///
/// 参考 MIRIX 的 ToolManager 实现
pub struct ToolManager {
    config: ToolManagerConfig,
    repository: Arc<ToolRepository>,
    executor: Arc<ToolExecutor>,
    /// 工具缓存
    cache: Arc<tokio::sync::RwLock<HashMap<String, Tool>>>,
}

impl ToolManager {
    /// 创建新的工具管理器
    pub fn new(
        config: ToolManagerConfig,
        pool: Arc<PgPool>,
        executor: Arc<ToolExecutor>,
    ) -> Self {
        let repository = Arc::new(ToolRepository::new((*pool).clone()));
        Self {
            config,
            repository,
            executor,
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(pool: Arc<PgPool>, executor: Arc<ToolExecutor>) -> Self {
        Self::new(ToolManagerConfig::default(), pool, executor)
    }

    /// 创建或更新工具
    ///
    /// 参考 MIRIX: create_or_update_tool
    pub async fn create_or_update_tool(
        &self,
        organization_id: &str,
        user_id: &str,
        request: CreateToolRequest,
    ) -> CoreResult<Tool> {
        // 检查工具是否已存在
        if let Some(existing) = self
            .repository
            .find_by_name(organization_id, &request.name)
            .await?
        {
            // 更新现有工具
            let update_req = UpdateToolRequest {
                description: request.description,
                source_code: request.source_code,
                json_schema: request.json_schema,
                tags: request.tags,
                is_enabled: None,
            };
            self.update_tool(&existing.id, user_id, update_req).await
        } else {
            // 创建新工具
            self.create_tool(organization_id, user_id, request).await
        }
    }

    /// 创建工具
    ///
    /// 参考 MIRIX: create_tool
    pub async fn create_tool(
        &self,
        organization_id: &str,
        user_id: &str,
        request: CreateToolRequest,
    ) -> CoreResult<Tool> {
        info!(
            "Creating tool '{}' for organization {}",
            request.name, organization_id
        );

        // 验证工具类型
        let tool_type = ToolType::from_str(&request.tool_type)?;

        // 自动生成描述（如果未提供）
        let description = request.description.or_else(|| {
            request
                .json_schema
                .as_ref()
                .and_then(|schema| schema.get("description"))
                .and_then(|d| d.as_str())
                .map(|s| s.to_string())
        });

        // 创建工具
        let tool = self
            .repository
            .create_tool(
                organization_id,
                user_id,
                &request.name,
                description.as_deref(),
                request.source_code.as_deref(),
                request.source_type.as_deref(),
                request.json_schema.as_ref(),
                request.tags.as_deref(),
            )
            .await?;

        // 更新缓存
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.insert(tool.id.clone(), tool.clone());
        }

        info!("Tool '{}' created successfully with ID {}", request.name, tool.id);
        Ok(tool)
    }

    /// 获取工具
    pub async fn get_tool(&self, tool_id: &str, user_id: &str) -> CoreResult<Tool> {
        // 先检查缓存
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some(tool) = cache.get(tool_id) {
                debug!("Tool {} found in cache", tool_id);
                return Ok(tool.clone());
            }
        }

        // 从数据库获取
        let tool = self.repository.get(tool_id).await?;

        // 更新缓存
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.insert(tool_id.to_string(), tool.clone());
        }

        Ok(tool)
    }

    /// 根据名称获取工具
    ///
    /// 参考 MIRIX: get_tool_by_name
    pub async fn get_tool_by_name(
        &self,
        organization_id: &str,
        name: &str,
    ) -> CoreResult<Option<Tool>> {
        self.repository.find_by_name(organization_id, name).await
    }

    /// 列出工具
    ///
    /// 参考 MIRIX: list_tools
    pub async fn list_tools(
        &self,
        organization_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<Tool>> {
        self.repository
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    /// 更新工具
    ///
    /// 参考 MIRIX: update_tool_by_id
    pub async fn update_tool(
        &self,
        tool_id: &str,
        user_id: &str,
        request: UpdateToolRequest,
    ) -> CoreResult<Tool> {
        info!("Updating tool {}", tool_id);

        let tool = self.repository.update_tool(tool_id, user_id, request).await?;

        // 清除缓存
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(tool_id);
        }

        Ok(tool)
    }

    /// 删除工具
    ///
    /// 参考 MIRIX: delete_tool_by_id
    pub async fn delete_tool(&self, tool_id: &str, user_id: &str) -> CoreResult<()> {
        info!("Deleting tool {}", tool_id);

        self.repository.delete_tool(tool_id, user_id).await?;

        // 清除缓存
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(tool_id);
        }

        Ok(())
    }

    /// 获取工具统计信息
    pub async fn get_stats(&self, organization_id: &str) -> CoreResult<ToolStats> {
        let tools = self.list_tools(organization_id, None, None).await?;

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut enabled_count = 0;
        let mut disabled_count = 0;

        for tool in &tools {
            // 统计类型
            if let Some(tags) = &tool.tags {
                for tag in tags {
                    *by_type.entry(tag.clone()).or_insert(0) += 1;
                }
            }

            // 统计启用/禁用
            if !tool.is_deleted {
                enabled_count += 1;
            } else {
                disabled_count += 1;
            }
        }

        Ok(ToolStats {
            total_tools: tools.len(),
            by_type,
            enabled_count,
            disabled_count,
        })
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Tool cache cleared");
    }
}

