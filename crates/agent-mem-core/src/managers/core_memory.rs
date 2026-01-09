//! Core Memory Manager - 核心记忆管理器
//!
//! 实现 persona 和 human 块管理，支持自动重写机制
//! 基于 AgentMem 7.0 认知记忆架构

use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Core Memory 块类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreMemoryBlockType {
    /// Persona 块 - 智能体的身份、性格、偏好
    Persona,
    /// Human 块 - 用户的信息、偏好、历史
    Human,
}

impl CoreMemoryBlockType {
    /// 获取块类型的描述
    pub fn description(&self) -> &'static str {
        match self {
            CoreMemoryBlockType::Persona => "智能体的身份、性格、偏好和行为模式",
            CoreMemoryBlockType::Human => "用户的信息、偏好、历史和交互模式",
        }
    }
}

/// Core Memory 块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMemoryBlock {
    /// 块ID
    pub id: String,
    /// 块类型
    pub block_type: CoreMemoryBlockType,
    /// 块内容
    pub content: String,
    /// 重要性评分 (0.0-1.0)
    pub importance: f32,
    /// 最大容量 (字符数)
    pub max_capacity: usize,
    /// 当前使用量 (字符数)
    pub current_size: usize,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
    /// 访问次数
    pub access_count: u64,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl CoreMemoryBlock {
    /// 创建新的 Core Memory 块
    pub fn new(block_type: CoreMemoryBlockType, content: String, max_capacity: usize) -> Self {
        let now = Utc::now();
        let current_size = content.len();

        Self {
            id: Uuid::new_v4().to_string(),
            block_type,
            content,
            importance: 0.5, // 默认中等重要性
            max_capacity,
            current_size,
            created_at: now,
            updated_at: now,
            last_accessed: now,
            access_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// 更新块内容
    pub fn update_content(&mut self, new_content: String) -> CoreResult<()> {
        let new_size = new_content.len();

        if new_size > self.max_capacity {
            return Err(CoreError::InvalidInput(format!(
                "Content size {} exceeds max capacity {}",
                new_size, self.max_capacity
            )));
        }

        self.content = new_content;
        self.current_size = new_size;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 追加内容到块
    pub fn append_content(&mut self, additional_content: &str) -> CoreResult<()> {
        let new_content = format!("{}\n{}", self.content, additional_content);
        self.update_content(new_content)
    }

    /// 检查是否需要重写 (容量使用率 >= 90%)
    pub fn needs_rewrite(&self) -> bool {
        let usage_ratio = self.current_size as f32 / self.max_capacity as f32;
        usage_ratio >= 0.9
    }

    /// 获取容量使用率
    pub fn capacity_usage(&self) -> f32 {
        self.current_size as f32 / self.max_capacity as f32
    }

    /// 记录访问
    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

/// Core Memory 管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreMemoryConfig {
    /// Persona 块默认容量
    pub persona_default_capacity: usize,
    /// Human 块默认容量
    pub human_default_capacity: usize,
    /// 自动重写阈值 (0.0-1.0)
    pub auto_rewrite_threshold: f32,
    /// 是否启用自动重写
    pub enable_auto_rewrite: bool,
    /// 重写时保留的重要内容比例
    pub rewrite_retention_ratio: f32,
}

impl Default for CoreMemoryConfig {
    fn default() -> Self {
        Self {
            persona_default_capacity: 2000, // 2KB
            human_default_capacity: 4000,   // 4KB
            auto_rewrite_threshold: 0.9,    // 90%
            enable_auto_rewrite: true,
            rewrite_retention_ratio: 0.7, // 保留70%重要内容
        }
    }
}

/// Core Memory 管理器
#[derive(Debug)]
pub struct CoreMemoryManager {
    /// 配置
    config: CoreMemoryConfig,
    /// Persona 块存储
    persona_blocks: Arc<RwLock<HashMap<String, CoreMemoryBlock>>>,
    /// Human 块存储
    human_blocks: Arc<RwLock<HashMap<String, CoreMemoryBlock>>>,
    /// 统计信息
    stats: Arc<RwLock<CoreMemoryStats>>,
}

/// Core Memory 统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreMemoryStats {
    /// Persona 块数量
    pub persona_blocks_count: usize,
    /// Human 块数量
    pub human_blocks_count: usize,
    /// 总访问次数
    pub total_accesses: u64,
    /// 自动重写次数
    pub auto_rewrites: u64,
    /// 平均容量使用率
    pub average_capacity_usage: f32,
}

impl CoreMemoryManager {
    /// 创建新的 Core Memory 管理器
    pub fn new() -> Self {
        Self::with_config(CoreMemoryConfig::default())
    }

    /// 使用自定义配置创建 Core Memory 管理器
    pub fn with_config(config: CoreMemoryConfig) -> Self {
        Self {
            config,
            persona_blocks: Arc::new(RwLock::new(HashMap::new())),
            human_blocks: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CoreMemoryStats::default())),
        }
    }

    /// 创建 Persona 块
    pub async fn create_persona_block(
        &self,
        content: String,
        max_capacity: Option<usize>,
    ) -> CoreResult<String> {
        let capacity = max_capacity.unwrap_or(self.config.persona_default_capacity);
        let block = CoreMemoryBlock::new(CoreMemoryBlockType::Persona, content, capacity);
        let block_id = block.id.clone();

        let mut persona_blocks = self.persona_blocks.write().await;
        persona_blocks.insert(block_id.clone(), block);

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.persona_blocks_count = persona_blocks.len();

        Ok(block_id)
    }

    /// 创建 Human 块
    pub async fn create_human_block(
        &self,
        content: String,
        max_capacity: Option<usize>,
    ) -> CoreResult<String> {
        let capacity = max_capacity.unwrap_or(self.config.human_default_capacity);
        let block = CoreMemoryBlock::new(CoreMemoryBlockType::Human, content, capacity);
        let block_id = block.id.clone();

        let mut human_blocks = self.human_blocks.write().await;
        human_blocks.insert(block_id.clone(), block);

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.human_blocks_count = human_blocks.len();

        Ok(block_id)
    }

    /// 获取 Persona 块
    pub async fn get_persona_block(&self, block_id: &str) -> CoreResult<Option<CoreMemoryBlock>> {
        let mut persona_blocks = self.persona_blocks.write().await;

        if let Some(block) = persona_blocks.get_mut(block_id) {
            block.record_access();

            // 更新统计
            let mut stats = self.stats.write().await;
            stats.total_accesses += 1;

            Ok(Some(block.clone()))
        } else {
            Ok(None)
        }
    }

    /// 获取 Human 块
    pub async fn get_human_block(&self, block_id: &str) -> CoreResult<Option<CoreMemoryBlock>> {
        let mut human_blocks = self.human_blocks.write().await;

        if let Some(block) = human_blocks.get_mut(block_id) {
            block.record_access();

            // 更新统计
            let mut stats = self.stats.write().await;
            stats.total_accesses += 1;

            Ok(Some(block.clone()))
        } else {
            Ok(None)
        }
    }

    /// 更新 Persona 块内容
    pub async fn update_persona_block(
        &self,
        block_id: &str,
        new_content: String,
    ) -> CoreResult<()> {
        let mut persona_blocks = self.persona_blocks.write().await;

        if let Some(block) = persona_blocks.get_mut(block_id) {
            block.update_content(new_content)?;

            // 检查是否需要自动重写
            if self.config.enable_auto_rewrite && block.needs_rewrite() {
                self.auto_rewrite_block(block).await?;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.auto_rewrites += 1;
            }

            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Persona block {block_id} not found"
            )))
        }
    }

    /// 更新 Human 块内容
    pub async fn update_human_block(&self, block_id: &str, new_content: String) -> CoreResult<()> {
        let mut human_blocks = self.human_blocks.write().await;

        if let Some(block) = human_blocks.get_mut(block_id) {
            block.update_content(new_content)?;

            // 检查是否需要自动重写
            if self.config.enable_auto_rewrite && block.needs_rewrite() {
                self.auto_rewrite_block(block).await?;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.auto_rewrites += 1;
            }

            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Human block {block_id} not found"
            )))
        }
    }

    /// 追加内容到 Persona 块
    pub async fn append_to_persona_block(
        &self,
        block_id: &str,
        additional_content: &str,
    ) -> CoreResult<()> {
        let mut persona_blocks = self.persona_blocks.write().await;

        if let Some(block) = persona_blocks.get_mut(block_id) {
            block.append_content(additional_content)?;

            // 检查是否需要自动重写
            if self.config.enable_auto_rewrite && block.needs_rewrite() {
                self.auto_rewrite_block(block).await?;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.auto_rewrites += 1;
            }

            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Persona block {block_id} not found"
            )))
        }
    }

    /// 追加内容到 Human 块
    pub async fn append_to_human_block(
        &self,
        block_id: &str,
        additional_content: &str,
    ) -> CoreResult<()> {
        let mut human_blocks = self.human_blocks.write().await;

        if let Some(block) = human_blocks.get_mut(block_id) {
            block.append_content(additional_content)?;

            // 检查是否需要自动重写
            if self.config.enable_auto_rewrite && block.needs_rewrite() {
                self.auto_rewrite_block(block).await?;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.auto_rewrites += 1;
            }

            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Human block {block_id} not found"
            )))
        }
    }

    /// 删除 Persona 块
    pub async fn delete_persona_block(&self, block_id: &str) -> CoreResult<()> {
        let mut persona_blocks = self.persona_blocks.write().await;

        if persona_blocks.remove(block_id).is_some() {
            // 更新统计
            let mut stats = self.stats.write().await;
            stats.persona_blocks_count = persona_blocks.len();
            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Persona block {block_id} not found"
            )))
        }
    }

    /// 删除 Human 块
    pub async fn delete_human_block(&self, block_id: &str) -> CoreResult<()> {
        let mut human_blocks = self.human_blocks.write().await;

        if human_blocks.remove(block_id).is_some() {
            // 更新统计
            let mut stats = self.stats.write().await;
            stats.human_blocks_count = human_blocks.len();
            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Human block {block_id} not found"
            )))
        }
    }

    /// 列出所有 Persona 块
    pub async fn list_persona_blocks(&self) -> CoreResult<Vec<CoreMemoryBlock>> {
        let persona_blocks = self.persona_blocks.read().await;
        Ok(persona_blocks.values().cloned().collect())
    }

    /// 列出所有 Human 块
    pub async fn list_human_blocks(&self) -> CoreResult<Vec<CoreMemoryBlock>> {
        let human_blocks = self.human_blocks.read().await;
        Ok(human_blocks.values().cloned().collect())
    }

    /// 自动重写块内容 (90% 容量触发)
    async fn auto_rewrite_block(&self, block: &mut CoreMemoryBlock) -> CoreResult<()> {
        // 简单的重写策略：保留最重要的内容
        let lines: Vec<&str> = block.content.lines().collect();
        let target_size =
            (block.max_capacity as f32 * self.config.rewrite_retention_ratio) as usize;

        // 按重要性排序（这里简化为按长度，实际应该使用更复杂的重要性评估）
        let mut important_lines: Vec<&str> = lines.clone();
        important_lines.sort_by(|a, b| b.len().cmp(&a.len()));

        let mut new_content = String::new();
        let mut current_size = 0;

        for line in important_lines {
            if current_size + line.len() < target_size {
                if !new_content.is_empty() {
                    new_content.push('\n');
                    current_size += 1;
                }
                new_content.push_str(line);
                current_size += line.len();
            } else {
                break;
            }
        }

        // 添加重写标记
        new_content.push_str("\n[Auto-rewritten to manage capacity]");

        block.content = new_content;
        block.current_size = block.content.len();
        block.updated_at = Utc::now();

        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CoreResult<CoreMemoryStats> {
        let mut stats = self.stats.write().await;

        // 更新实时统计
        let persona_blocks = self.persona_blocks.read().await;
        let human_blocks = self.human_blocks.read().await;

        stats.persona_blocks_count = persona_blocks.len();
        stats.human_blocks_count = human_blocks.len();

        // 计算平均容量使用率
        let mut total_usage = 0.0;
        let mut total_blocks = 0;

        for block in persona_blocks.values() {
            total_usage += block.capacity_usage();
            total_blocks += 1;
        }

        for block in human_blocks.values() {
            total_usage += block.capacity_usage();
            total_blocks += 1;
        }

        if total_blocks > 0 {
            stats.average_capacity_usage = total_usage / total_blocks as f32;
        }

        Ok(stats.clone())
    }

    /// 检查所有块的容量状态
    pub async fn check_capacity_status(
        &self,
    ) -> CoreResult<Vec<(String, CoreMemoryBlockType, f32)>> {
        let mut status = Vec::new();

        let persona_blocks = self.persona_blocks.read().await;
        for (id, block) in persona_blocks.iter() {
            status.push((id.clone(), block.block_type.clone(), block.capacity_usage()));
        }

        let human_blocks = self.human_blocks.read().await;
        for (id, block) in human_blocks.iter() {
            status.push((id.clone(), block.block_type.clone(), block.capacity_usage()));
        }

        Ok(status)
    }

    /// 获取需要重写的块列表
    pub async fn get_blocks_needing_rewrite(&self) -> CoreResult<Vec<String>> {
        let mut blocks_needing_rewrite = Vec::new();

        let persona_blocks = self.persona_blocks.read().await;
        for (id, block) in persona_blocks.iter() {
            if block.needs_rewrite() {
                blocks_needing_rewrite.push(id.clone());
            }
        }

        let human_blocks = self.human_blocks.read().await;
        for (id, block) in human_blocks.iter() {
            if block.needs_rewrite() {
                blocks_needing_rewrite.push(id.clone());
            }
        }

        Ok(blocks_needing_rewrite)
    }

    /// 手动触发重写指定块
    pub async fn manual_rewrite_block(&self, block_id: &str) -> CoreResult<()> {
        // 尝试在 persona 块中查找
        {
            let mut persona_blocks = self.persona_blocks.write().await;
            if let Some(block) = persona_blocks.get_mut(block_id) {
                self.auto_rewrite_block(block).await?;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.auto_rewrites += 1;

                return Ok(());
            }
        }

        // 尝试在 human 块中查找
        {
            let mut human_blocks = self.human_blocks.write().await;
            if let Some(block) = human_blocks.get_mut(block_id) {
                self.auto_rewrite_block(block).await?;

                // 更新统计
                let mut stats = self.stats.write().await;
                stats.auto_rewrites += 1;

                return Ok(());
            }
        }

        Err(CoreError::NotFound(format!("Block {block_id} not found")))
    }

    /// 清空所有块
    pub async fn clear_all(&self) -> CoreResult<()> {
        let mut persona_blocks = self.persona_blocks.write().await;
        let mut human_blocks = self.human_blocks.write().await;
        let mut stats = self.stats.write().await;

        persona_blocks.clear();
        human_blocks.clear();

        stats.persona_blocks_count = 0;
        stats.human_blocks_count = 0;

        Ok(())
    }
}

impl Default for CoreMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_core_memory_manager_creation() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();
        let stats = manager.get_stats().await?;

        assert_eq!(stats.persona_blocks_count, 0);
        assert_eq!(stats.human_blocks_count, 0);
        assert_eq!(stats.total_accesses, 0);
        assert_eq!(stats.auto_rewrites, 0);
    }

    #[tokio::test]
    async fn test_persona_block_creation_and_retrieval() {
        let manager = CoreMemoryManager::new();

        let content = "I am a helpful AI assistant with a friendly personality.".to_string();
        let block_id = manager
            .create_persona_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Persona);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_human_block_creation_and_retrieval() {
        let manager = CoreMemoryManager::new();

        let content = "User prefers concise responses and technical details.".to_string();
        let block_id = manager
            .create_human_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_human_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Human);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_block_content_update() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content, None)
            .await
            .unwrap();

        let new_content = "Updated content with more information".to_string();
        manager
            .update_persona_block(&block_id, new_content.clone())
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(updated_block.content, new_content);
        assert!(updated_block.updated_at > updated_block.created_at);
    }

    #[tokio::test]
    async fn test_block_content_append() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content.clone(), None)
            .await
            .unwrap();

        let additional_content = "Additional information";
        manager
            .append_to_persona_block(&block_id, additional_content)
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(updated_block.content.contains(&initial_content));
        assert!(updated_block.content.contains(additional_content));
    }

    #[tokio::test]
    async fn test_capacity_management() {
        let manager = CoreMemoryManager::new();

        // 创建一个小容量的块
        let small_capacity = 50;
        let content = "Short content".to_string();
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(block.max_capacity, small_capacity);
        assert!(block.capacity_usage() < 1.0);

        // 测试容量超限
        let large_content = "x".repeat(100);
        let result = manager.update_persona_block(&block_id, large_content).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_rewrite_trigger() {
        let mut config = CoreMemoryConfig::default();
        config.enable_auto_rewrite = true;
        config.auto_rewrite_threshold = 0.8; // 80% 触发重写

        let manager = CoreMemoryManager::with_config(config);

        // 创建一个小容量的块
        let small_capacity = 100;
        let content = "x".repeat(85); // 85% 容量使用
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        // 添加更多内容触发重写
        manager
            .append_to_persona_block(&block_id, "more content")
            .await
            .unwrap();

        let stats = manager.get_stats().await?;
        assert!(stats.auto_rewrites > 0);
    }

    #[tokio::test]
    async fn test_block_deletion() {
        let manager = CoreMemoryManager::new();

        let content = "Content to be deleted".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        // 确认块存在
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_some());

        // 删除块
        manager.delete_persona_block(&block_id).await?;

        // 确认块已删除
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_none());

        let stats = manager.get_stats().await?;
        assert_eq!(stats.persona_blocks_count, 0);
    }

    #[tokio::test]
    async fn test_list_blocks() {
        let manager = CoreMemoryManager::new();

        // 创建多个块
        manager
            .create_persona_block("Persona 1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_persona_block("Persona 2".to_string(), None)
            .await
            .unwrap();
        manager
            .create_human_block("Human 1".to_string(), None)
            .await
            .unwrap();

        let persona_blocks = manager.list_persona_blocks().await?;
        let human_blocks = manager.list_human_blocks().await?;

        assert_eq!(persona_blocks.len(), 2);
        assert_eq!(human_blocks.len(), 1);
    }

    #[tokio::test]
    async fn test_capacity_status_check() {
        let manager = CoreMemoryManager::new();

        let block_id = manager
            .create_persona_block("Test content".to_string(), Some(100))
            .await
            .unwrap();

        let status = manager.check_capacity_status().await?;
        assert_eq!(status.len(), 1);

        let (id, block_type, usage) = &status[0];
        assert_eq!(id, &block_id);
        assert_eq!(*block_type, CoreMemoryBlockType::Persona);
        assert!(usage > &0.0 && usage < &1.0);
    async fn test_core_memory_manager_creation() {
        let manager = CoreMemoryManager::new();
        let stats = manager.get_stats().await?;

        assert_eq!(stats.persona_blocks_count, 0);
        assert_eq!(stats.human_blocks_count, 0);
        assert_eq!(stats.total_accesses, 0);
        assert_eq!(stats.auto_rewrites, 0);
    }

    #[tokio::test]
    async fn test_persona_block_creation_and_retrieval() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();

        let content = "I am a helpful AI assistant with a friendly personality.".to_string();
        let block_id = manager
            .create_persona_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Persona);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_human_block_creation_and_retrieval() {
        let manager = CoreMemoryManager::new();

        let content = "User prefers concise responses and technical details.".to_string();
        let block_id = manager
            .create_human_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_human_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Human);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_block_content_update() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content, None)
            .await
            .unwrap();

        let new_content = "Updated content with more information".to_string();
        manager
            .update_persona_block(&block_id, new_content.clone())
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(updated_block.content, new_content);
        assert!(updated_block.updated_at > updated_block.created_at);
    }

    #[tokio::test]
    async fn test_block_content_append() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content.clone(), None)
            .await
            .unwrap();

        let additional_content = "Additional information";
        manager
            .append_to_persona_block(&block_id, additional_content)
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(updated_block.content.contains(&initial_content));
        assert!(updated_block.content.contains(additional_content));
    }

    #[tokio::test]
    async fn test_capacity_management() {
        let manager = CoreMemoryManager::new();

        // 创建一个小容量的块
        let small_capacity = 50;
        let content = "Short content".to_string();
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(block.max_capacity, small_capacity);
        assert!(block.capacity_usage() < 1.0);

        // 测试容量超限
        let large_content = "x".repeat(100);
        let result = manager.update_persona_block(&block_id, large_content).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_rewrite_trigger() {
        let mut config = CoreMemoryConfig::default();
        config.enable_auto_rewrite = true;
        config.auto_rewrite_threshold = 0.8; // 80% 触发重写

        let manager = CoreMemoryManager::with_config(config);

        // 创建一个小容量的块
        let small_capacity = 100;
        let content = "x".repeat(85); // 85% 容量使用
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        // 添加更多内容触发重写
        manager
            .append_to_persona_block(&block_id, "more content")
            .await
            .unwrap();

        let stats = manager.get_stats().await?;
        assert!(stats.auto_rewrites > 0);
    }

    #[tokio::test]
    async fn test_block_deletion() {
        let manager = CoreMemoryManager::new();

        let content = "Content to be deleted".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        // 确认块存在
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_some());

        // 删除块
        manager.delete_persona_block(&block_id).await?;

        // 确认块已删除
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_none());

        let stats = manager.get_stats().await?;
        assert_eq!(stats.persona_blocks_count, 0);
    }

    #[tokio::test]
    async fn test_list_blocks() {
        let manager = CoreMemoryManager::new();

        // 创建多个块
        manager
            .create_persona_block("Persona 1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_persona_block("Persona 2".to_string(), None)
            .await
            .unwrap();
        manager
            .create_human_block("Human 1".to_string(), None)
            .await
            .unwrap();

        let persona_blocks = manager.list_persona_blocks().await?;
        let human_blocks = manager.list_human_blocks().await?;

        assert_eq!(persona_blocks.len(), 2);
        assert_eq!(human_blocks.len(), 1);
    }

    #[tokio::test]
    async fn test_capacity_status_check() {
        let manager = CoreMemoryManager::new();

        let block_id = manager
            .create_persona_block("Test content".to_string(), Some(100))
            .await
            .unwrap();

        let status = manager.check_capacity_status().await?;
        assert_eq!(status.len(), 1);

        let (id, block_type, usage) = &status[0];
        assert_eq!(id, &block_id);
        assert_eq!(*block_type, CoreMemoryBlockType::Persona);
        assert!(usage > &0.0 && usage < &1.0);
    }

    #[tokio::test]
    async fn test_manual_rewrite() {
        let manager = CoreMemoryManager::new();

        let content = "Content that will be rewritten manually".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        manager.manual_rewrite_block(&block_id).await?;

        let stats = manager.get_stats().await?;
        assert_eq!(stats.auto_rewrites, 1);

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(block
            .content
            .contains("[Auto-rewritten to manage capacity]"));
    }

    #[tokio::test]
    async fn test_persona_block_creation_and_retrieval() {
        let manager = CoreMemoryManager::new();
        let content = "I am a helpful AI assistant with a friendly personality.".to_string();
        let block_id = manager
            .create_persona_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Persona);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_human_block_creation_and_retrieval() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();

        let content = "User prefers concise responses and technical details.".to_string();
        let block_id = manager
            .create_human_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_human_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Human);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_block_content_update() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content, None)
            .await
            .unwrap();

        let new_content = "Updated content with more information".to_string();
        manager
            .update_persona_block(&block_id, new_content.clone())
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(updated_block.content, new_content);
        assert!(updated_block.updated_at > updated_block.created_at);
    }

    #[tokio::test]
    async fn test_block_content_append() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content.clone(), None)
            .await
            .unwrap();

        let additional_content = "Additional information";
        manager
            .append_to_persona_block(&block_id, additional_content)
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(updated_block.content.contains(&initial_content));
        assert!(updated_block.content.contains(additional_content));
    }

    #[tokio::test]
    async fn test_capacity_management() {
        let manager = CoreMemoryManager::new();

        // 创建一个小容量的块
        let small_capacity = 50;
        let content = "Short content".to_string();
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(block.max_capacity, small_capacity);
        assert!(block.capacity_usage() < 1.0);

        // 测试容量超限
        let large_content = "x".repeat(100);
        let result = manager.update_persona_block(&block_id, large_content).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_rewrite_trigger() {
        let mut config = CoreMemoryConfig::default();
        config.enable_auto_rewrite = true;
        config.auto_rewrite_threshold = 0.8; // 80% 触发重写

        let manager = CoreMemoryManager::with_config(config);

        // 创建一个小容量的块
        let small_capacity = 100;
        let content = "x".repeat(85); // 85% 容量使用
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        // 添加更多内容触发重写
        manager
            .append_to_persona_block(&block_id, "more content")
            .await
            .unwrap();

        let stats = manager.get_stats().await?;
        assert!(stats.auto_rewrites > 0);
    }

    #[tokio::test]
    async fn test_block_deletion() {
        let manager = CoreMemoryManager::new();

        let content = "Content to be deleted".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        // 确认块存在
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_some());

        // 删除块
        manager.delete_persona_block(&block_id).await?;

        // 确认块已删除
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_none());

        let stats = manager.get_stats().await?;
        assert_eq!(stats.persona_blocks_count, 0);
    }

    #[tokio::test]
    async fn test_list_blocks() {
        let manager = CoreMemoryManager::new();

        // 创建多个块
        manager
            .create_persona_block("Persona 1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_persona_block("Persona 2".to_string(), None)
            .await
            .unwrap();
        manager
            .create_human_block("Human 1".to_string(), None)
            .await
            .unwrap();

        let persona_blocks = manager.list_persona_blocks().await?;
        let human_blocks = manager.list_human_blocks().await?;

        assert_eq!(persona_blocks.len(), 2);
        assert_eq!(human_blocks.len(), 1);
    }

    #[tokio::test]
    async fn test_capacity_status_check() {
        let manager = CoreMemoryManager::new();

        let block_id = manager
            .create_persona_block("Test content".to_string(), Some(100))
            .await
            .unwrap();

        let status = manager.check_capacity_status().await?;
        assert_eq!(status.len(), 1);

        let (id, block_type, usage) = &status[0];
        assert_eq!(id, &block_id);
        assert_eq!(*block_type, CoreMemoryBlockType::Persona);
        assert!(usage > &0.0 && usage < &1.0);
    }

    #[tokio::test]
    async fn test_manual_rewrite() {
        let manager = CoreMemoryManager::new();

        let content = "Content that will be rewritten manually".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        manager.manual_rewrite_block(&block_id).await?;

        let stats = manager.get_stats().await?;
        assert_eq!(stats.auto_rewrites, 1);

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(block
            .content
            .contains("[Auto-rewritten to manage capacity]"));
    }
}

    async fn test_human_block_creation_and_retrieval() {
        let manager = CoreMemoryManager::new();

        let content = "User prefers concise responses and technical details.".to_string();
        let block_id = manager
            .create_human_block(content.clone(), None)
            .await
            .unwrap();

        let retrieved_block = manager.get_human_block(&block_id).await?.unwrap();
        assert_eq!(retrieved_block.content, content);
        assert_eq!(retrieved_block.block_type, CoreMemoryBlockType::Human);
        assert_eq!(retrieved_block.access_count, 1);
    }

    #[tokio::test]
    async fn test_block_content_update() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content, None)
            .await
            .unwrap();

        let new_content = "Updated content with more information".to_string();
        manager
            .update_persona_block(&block_id, new_content.clone())
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(updated_block.content, new_content);
        assert!(updated_block.updated_at > updated_block.created_at);
    }

    #[tokio::test]
    async fn test_block_content_append() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content.clone(), None)
            .await
            .unwrap();

        let additional_content = "Additional information";
        manager
            .append_to_persona_block(&block_id, additional_content)
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(updated_block.content.contains(&initial_content));
        assert!(updated_block.content.contains(additional_content));
    }

    #[tokio::test]
    async fn test_capacity_management() {
        let manager = CoreMemoryManager::new();

        // 创建一个小容量的块
        let small_capacity = 50;
        let content = "Short content".to_string();
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(block.max_capacity, small_capacity);
        assert!(block.capacity_usage() < 1.0);

        // 测试容量超限
        let large_content = "x".repeat(100);
        let result = manager.update_persona_block(&block_id, large_content).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_rewrite_trigger() {
        let mut config = CoreMemoryConfig::default();
        config.enable_auto_rewrite = true;
        config.auto_rewrite_threshold = 0.8; // 80% 触发重写

        let manager = CoreMemoryManager::with_config(config);

        // 创建一个小容量的块
        let small_capacity = 100;
        let content = "x".repeat(85); // 85% 容量使用
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        // 添加更多内容触发重写
        manager
            .append_to_persona_block(&block_id, "more content")
            .await
            .unwrap();

        let stats = manager.get_stats().await?;
        assert!(stats.auto_rewrites > 0);
    }

    #[tokio::test]
    async fn test_block_deletion() {
        let manager = CoreMemoryManager::new();

        let content = "Content to be deleted".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        // 确认块存在
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_some());

        // 删除块
        manager.delete_persona_block(&block_id).await?;

        // 确认块已删除
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_none());

        let stats = manager.get_stats().await?;
        assert_eq!(stats.persona_blocks_count, 0);
    }

    #[tokio::test]
    async fn test_list_blocks() {
        let manager = CoreMemoryManager::new();

        // 创建多个块
        manager
            .create_persona_block("Persona 1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_persona_block("Persona 2".to_string(), None)
            .await
            .unwrap();
        manager
            .create_human_block("Human 1".to_string(), None)
            .await
            .unwrap();

        let persona_blocks = manager.list_persona_blocks().await?;
        let human_blocks = manager.list_human_blocks().await?;

        assert_eq!(persona_blocks.len(), 2);
        assert_eq!(human_blocks.len(), 1);
    }

    #[tokio::test]
    async fn test_capacity_status_check() {
        let manager = CoreMemoryManager::new();

        let block_id = manager
            .create_persona_block("Test content".to_string(), Some(100))
            .await
            .unwrap();

        let status = manager.check_capacity_status().await?;
        assert_eq!(status.len(), 1);

        let (id, block_type, usage) = &status[0];
        assert_eq!(id, &block_id);
        assert_eq!(*block_type, CoreMemoryBlockType::Persona);
        assert!(usage > &0.0 && usage < &1.0);
    }

    #[tokio::test]
    async fn test_manual_rewrite() {
        let manager = CoreMemoryManager::new();

        let content = "Content that will be rewritten manually".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        manager.manual_rewrite_block(&block_id).await?;

        let stats = manager.get_stats().await?;
        assert_eq!(stats.auto_rewrites, 1);

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(block
            .content
            .contains("[Auto-rewritten to manage capacity]"));
    }
}

    async fn test_block_content_update() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content, None)
            .await
            .unwrap();

        let new_content = "Updated content with more information".to_string();
        manager
            .update_persona_block(&block_id, new_content.clone())
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(updated_block.content, new_content);
        assert!(updated_block.updated_at > updated_block.created_at);
        Ok(())
    }

    #[tokio::test]
    async fn test_block_content_append() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content.clone(), None)
            .await
            .unwrap();

        let additional_content = "Additional information";
        manager
            .append_to_persona_block(&block_id, additional_content)
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(updated_block.content.contains(&initial_content));
        assert!(updated_block.content.contains(additional_content));
    }

    #[tokio::test]
    async fn test_capacity_management() {
        let manager = CoreMemoryManager::new();

        // 创建一个小容量的块
        let small_capacity = 50;
        let content = "Short content".to_string();
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(block.max_capacity, small_capacity);
        assert!(block.capacity_usage() < 1.0);

        // 测试容量超限
        let large_content = "x".repeat(100);
        let result = manager.update_persona_block(&block_id, large_content).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_rewrite_trigger() {
        let mut config = CoreMemoryConfig::default();
        config.enable_auto_rewrite = true;
        config.auto_rewrite_threshold = 0.8; // 80% 触发重写

        let manager = CoreMemoryManager::with_config(config);

        // 创建一个小容量的块
        let small_capacity = 100;
        let content = "x".repeat(85); // 85% 容量使用
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        // 添加更多内容触发重写
        manager
            .append_to_persona_block(&block_id, "more content")
            .await
            .unwrap();

        let stats = manager.get_stats().await?;
        assert!(stats.auto_rewrites > 0);
    }

    #[tokio::test]
    async fn test_block_deletion() {
        let manager = CoreMemoryManager::new();

        let content = "Content to be deleted".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        // 确认块存在
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_some());

        // 删除块
        manager.delete_persona_block(&block_id).await?;

        // 确认块已删除
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_none());

        let stats = manager.get_stats().await?;
        assert_eq!(stats.persona_blocks_count, 0);
    }

    #[tokio::test]
    async fn test_list_blocks() {
        let manager = CoreMemoryManager::new();

        // 创建多个块
        manager
            .create_persona_block("Persona 1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_persona_block("Persona 2".to_string(), None)
            .await
            .unwrap();
        manager
            .create_human_block("Human 1".to_string(), None)
            .await
            .unwrap();

        let persona_blocks = manager.list_persona_blocks().await?;
        let human_blocks = manager.list_human_blocks().await?;

        assert_eq!(persona_blocks.len(), 2);
        assert_eq!(human_blocks.len(), 1);
    }

    #[tokio::test]
    async fn test_capacity_status_check() {
        let manager = CoreMemoryManager::new();

        let block_id = manager
            .create_persona_block("Test content".to_string(), Some(100))
            .await
            .unwrap();

        let status = manager.check_capacity_status().await?;
        assert_eq!(status.len(), 1);

        let (id, block_type, usage) = &status[0];
        assert_eq!(id, &block_id);
        assert_eq!(*block_type, CoreMemoryBlockType::Persona);
        assert!(usage > &0.0 && usage < &1.0);
    }

    #[tokio::test]

    async fn test_block_content_append() {
        let manager = CoreMemoryManager::new();

        let initial_content = "Initial content".to_string();
        let block_id = manager
            .create_persona_block(initial_content.clone(), None)
            .await
            .unwrap();

        let additional_content = "Additional information";
        manager
            .append_to_persona_block(&block_id, additional_content)
            .await
            .unwrap();

        let updated_block = manager.get_persona_block(&block_id).await?.unwrap();
        assert!(updated_block.content.contains(&initial_content));
        assert!(updated_block.content.contains(additional_content));
    }

    #[tokio::test]
    async fn test_capacity_management() -> anyhow::Result<()> {
        let manager = CoreMemoryManager::new();

        // 创建一个小容量的块
        let small_capacity = 50;
        let content = "Short content".to_string();
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        let block = manager.get_persona_block(&block_id).await?.unwrap();
        assert_eq!(block.max_capacity, small_capacity);
        assert!(block.capacity_usage() < 1.0);

        // 测试容量超限
        let large_content = "x".repeat(100);
        let result = manager.update_persona_block(&block_id, large_content).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auto_rewrite_trigger() {
        let mut config = CoreMemoryConfig::default();
        config.enable_auto_rewrite = true;
        config.auto_rewrite_threshold = 0.8; // 80% 触发重写

        let manager = CoreMemoryManager::with_config(config);

        // 创建一个小容量的块
        let small_capacity = 100;
        let content = "x".repeat(85); // 85% 容量使用
        let block_id = manager
            .create_persona_block(content, Some(small_capacity))
            .await
            .unwrap();

        // 添加更多内容触发重写
        manager
            .append_to_persona_block(&block_id, "more content")
            .await
            .unwrap();

        let stats = manager.get_stats().await?;
        assert!(stats.auto_rewrites > 0);
    }

    #[tokio::test]
    async fn test_block_deletion() {
        let manager = CoreMemoryManager::new();

        let content = "Content to be deleted".to_string();
        let block_id = manager.create_persona_block(content, None).await?;

        // 确认块存在
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_some());

        // 删除块
        manager.delete_persona_block(&block_id).await?;

        // 确认块已删除
        assert!(manager
            .get_persona_block(&block_id)
            .await
            .unwrap()
            .is_none());

        let stats = manager.get_stats().await?;
        assert_eq!(stats.persona_blocks_count, 0);
    }

    #[tokio::test]
    async fn test_list_blocks() {
        let manager = CoreMemoryManager::new();

        // 创建多个块
        manager
            .create_persona_block("Persona 1".to_string(), None)
            .await
            .unwrap();
        manager
            .create_persona_block("Persona 2".to_string(), None)
            .await
            .unwrap();
        manager
            .create_human_block("Human 1".to_string(), None)
            .await
            .unwrap();

        let persona_blocks = manager.list_persona_blocks().await?;
        let human_blocks = manager.list_human_blocks().await?;

        assert_eq!(persona_blocks.len(), 2);
        assert_eq!(human_blocks.len(), 1);
    }

    #[tokio::test]
    async fn test_capacity_status_check() {
        let manager = CoreMemoryManager::new();

        let block_id = manager
            .create_persona_block("Test content".to_string(), Some(100))
            .await
            .unwrap();

        let status = manager.check_capacity_status().await?;
        assert_eq!(status.len(), 1);

        let (id, block_type, usage) = &status[0];
        assert_eq!(id, &block_id);
        assert_eq!(*block_type, CoreMemoryBlockType::Persona);
        assert!(usage > &0.0 && usage < &1.0);
    }

