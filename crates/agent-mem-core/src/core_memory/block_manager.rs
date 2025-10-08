//! Block Manager - 基于数据库的 Block 管理器
//!
//! 提供 Block 的 CRUD 操作，支持：
//! - 数据库持久化
//! - 字符限制验证
//! - 模板管理
//! - 自动重写触发

use crate::core_memory::{BlockMetadata, BlockStats, BlockType};
#[cfg(feature = "postgres")]
use crate::storage::block_repository::BlockRepository;
#[cfg(feature = "postgres")]
use crate::storage::models::Block;
#[cfg(feature = "postgres")]
use crate::storage::repository::Repository;
use agent_mem_traits::{AgentMemError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
#[cfg(feature = "postgres")]
use sqlx::PgPool;
use std::sync::Arc;

/// Block Manager 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockManagerConfig {
    /// Persona 块默认字符限制
    pub persona_default_limit: i64,
    /// Human 块默认字符限制
    pub human_default_limit: i64,
    /// System 块默认字符限制
    pub system_default_limit: i64,
    /// 自动重写阈值 (0.0-1.0)
    pub auto_rewrite_threshold: f32,
    /// 启用自动重写
    pub enable_auto_rewrite: bool,
}

impl Default for BlockManagerConfig {
    fn default() -> Self {
        Self {
            persona_default_limit: 2000,
            human_default_limit: 2000,
            system_default_limit: 1000,
            auto_rewrite_threshold: 0.9, // 90% 触发重写
            enable_auto_rewrite: true,
        }
    }
}

/// Block Manager - 管理 Core Memory Blocks
pub struct BlockManager {
    repository: Arc<BlockRepository>,
    config: BlockManagerConfig,
}

impl BlockManager {
    /// 创建新的 Block Manager
    pub fn new(pool: PgPool, config: BlockManagerConfig) -> Self {
        Self {
            repository: Arc::new(BlockRepository::new(pool)),
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(pool: PgPool) -> Self {
        Self::new(pool, BlockManagerConfig::default())
    }

    /// 创建 Block
    pub async fn create_block(
        &self,
        organization_id: String,
        user_id: String,
        block_type: BlockType,
        value: String,
        limit: Option<i64>,
        template_name: Option<String>,
        description: Option<String>,
    ) -> Result<Block> {
        // 确定字符限制
        let limit = limit.unwrap_or_else(|| self.get_default_limit(&block_type));

        // 验证内容长度
        if value.len() as i64 > limit {
            return Err(AgentMemError::validation_error(format!(
                "Block value exceeds limit: {} > {}",
                value.len(),
                limit
            )));
        }

        // 创建 Block
        let block = Block::new(
            organization_id,
            user_id,
            block_type.as_str().to_string(),
            value,
            limit,
        );

        let mut block = block;
        block.template_name = template_name;
        block.description = description;

        // 保存到数据库
        self.repository
            .create_validated(&block)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 更新 Block 内容
    pub async fn update_block_value(&self, block_id: &str, new_value: String) -> Result<Block> {
        // 获取现有 Block
        let mut block = self
            .repository
            .read(block_id)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))?
            .ok_or_else(|| AgentMemError::not_found("Block not found"))?;

        // 检查是否需要自动重写
        if self.config.enable_auto_rewrite {
            let utilization = new_value.len() as f32 / block.limit as f32;
            if utilization >= self.config.auto_rewrite_threshold {
                // 标记需要重写
                let mut metadata = self.parse_metadata(&block)?;
                metadata.needs_rewrite = true;
                block.metadata_ = Some(serde_json::to_value(&metadata)?);
            }
        }

        // 更新内容
        block.value = new_value;
        block.updated_at = Utc::now();

        // 保存到数据库
        self.repository
            .update_validated(&block)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 追加内容到 Block
    pub async fn append_to_block(&self, block_id: &str, additional_content: &str) -> Result<Block> {
        // 获取现有 Block
        let block = self
            .repository
            .read(block_id)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))?
            .ok_or_else(|| AgentMemError::not_found("Block not found"))?;

        // 追加内容
        let new_value = if block.value.is_empty() {
            additional_content.to_string()
        } else {
            format!("{}\n{}", block.value, additional_content)
        };

        // 更新
        self.update_block_value(block_id, new_value).await
    }

    /// 获取 Block
    pub async fn get_block(&self, block_id: &str) -> Result<Block> {
        let mut block = self
            .repository
            .read(block_id)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))?
            .ok_or_else(|| AgentMemError::not_found("Block not found"))?;

        // 更新访问统计
        let mut metadata = self.parse_metadata(&block)?;
        metadata.access_count += 1;
        metadata.last_accessed = Some(Utc::now());
        block.metadata_ = Some(serde_json::to_value(&metadata)?);

        // 保存更新的元数据
        self.repository
            .update(&block)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))?;

        Ok(block)
    }

    /// 删除 Block
    pub async fn delete_block(&self, block_id: &str) -> Result<()> {
        self.repository
            .delete(block_id)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))?;
        Ok(())
    }

    /// 列出用户的所有 Blocks
    pub async fn list_user_blocks(
        &self,
        user_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Block>> {
        self.repository
            .list_by_user(user_id, limit, offset)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 列出特定类型的 Blocks
    pub async fn list_blocks_by_type(
        &self,
        user_id: &str,
        block_type: BlockType,
    ) -> Result<Vec<Block>> {
        self.repository
            .list_by_label(user_id, block_type.as_str())
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 获取 Agent 的所有 Blocks
    pub async fn get_agent_blocks(&self, agent_id: &str) -> Result<Vec<Block>> {
        self.repository
            .list_by_agent(agent_id)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 创建 Block 模板
    pub async fn create_template(
        &self,
        organization_id: String,
        user_id: String,
        template_name: String,
        block_type: BlockType,
        template_value: String,
        description: Option<String>,
    ) -> Result<Block> {
        let limit = self.get_default_limit(&block_type);

        let mut block = Block::new(
            organization_id,
            user_id,
            block_type.as_str().to_string(),
            template_value,
            limit,
        );

        block.is_template = true;
        block.template_name = Some(template_name);
        block.description = description;

        self.repository
            .create(&block)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 从模板创建 Block
    pub async fn create_from_template(
        &self,
        template_id: &str,
        organization_id: String,
        user_id: String,
        variables: std::collections::HashMap<String, String>,
    ) -> Result<Block> {
        // 获取模板
        let template = self
            .repository
            .read(template_id)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))?
            .ok_or_else(|| AgentMemError::not_found("Template not found"))?;

        if !template.is_template {
            return Err(AgentMemError::validation_error("Block is not a template"));
        }

        // 渲染模板
        let rendered_value = self.render_template(&template.value, &variables)?;

        // 创建新 Block
        let mut block = Block::new(
            organization_id,
            user_id,
            template.label.clone(),
            rendered_value,
            template.limit,
        );

        block.template_name = template.template_name.clone();
        block.description = template.description.clone();

        self.repository
            .create_validated(&block)
            .await
            .map_err(|e| AgentMemError::storage_error(e.to_string()))
    }

    /// 获取 Block 统计信息
    pub async fn get_stats(&self, user_id: &str) -> Result<BlockStats> {
        let blocks = self.list_user_blocks(user_id, None, None).await?;

        let mut stats = BlockStats::default();
        stats.total_blocks = blocks.len();

        for block in &blocks {
            match block.label.as_str() {
                "persona" => stats.persona_blocks += 1,
                "human" => stats.human_blocks += 1,
                "system" => stats.system_blocks += 1,
                _ => {}
            }

            stats.total_characters += block.value.len();

            // 检查是否需要重写
            if let Ok(metadata) = self.parse_metadata(block) {
                if metadata.needs_rewrite {
                    stats.blocks_needing_rewrite += 1;
                }
            }
        }

        // 计算平均使用率
        if !blocks.is_empty() {
            let total_utilization: f32 = blocks
                .iter()
                .map(|b| b.value.len() as f32 / b.limit as f32)
                .sum();
            stats.average_utilization = total_utilization / blocks.len() as f32;
        }

        Ok(stats)
    }

    /// 获取默认字符限制
    fn get_default_limit(&self, block_type: &BlockType) -> i64 {
        match block_type {
            BlockType::Persona => self.config.persona_default_limit,
            BlockType::Human => self.config.human_default_limit,
            BlockType::System => self.config.system_default_limit,
        }
    }

    /// 解析 Block 元数据
    fn parse_metadata(&self, block: &Block) -> Result<BlockMetadata> {
        if let Some(ref metadata_value) = block.metadata_ {
            Ok(serde_json::from_value(metadata_value.clone()).unwrap_or_default())
        } else {
            Ok(BlockMetadata::default())
        }
    }

    /// 简单的模板渲染（替换 {{variable}} 占位符）
    fn render_template(
        &self,
        template: &str,
        variables: &std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let mut rendered = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        Ok(rendered)
    }
}
