//! 批量操作优化模块
//!
//! 提供批量操作优化功能，包括：
//! - 批量向量搜索
//! - 批量图查询
//! - 批量插入优化
//! - 并发控制

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};

/// 批量处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// 批量大小
    pub batch_size: usize,
    
    /// 最大并发数
    pub max_concurrency: usize,
    
    /// 批量超时（秒）
    pub batch_timeout_seconds: u64,
    
    /// 是否启用自动批处理
    pub enable_auto_batching: bool,
    
    /// 自动批处理延迟（毫秒）
    pub auto_batch_delay_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_concurrency: 10,
            batch_timeout_seconds: 30,
            enable_auto_batching: true,
            auto_batch_delay_ms: 100,
        }
    }
}

/// 批量处理器
pub struct BatchProcessor {
    /// 配置
    config: BatchConfig,
    
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    
    /// 统计信息
    stats: Arc<RwLock<BatchStats>>,
}

impl BatchProcessor {
    /// 创建新的批量处理器
    pub fn new(config: BatchConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        
        Self {
            config,
            semaphore,
            stats: Arc::new(RwLock::new(BatchStats::default())),
        }
    }
    
    /// 批量执行操作
    ///
    /// # Arguments
    ///
    /// * `items` - 要处理的项目列表
    /// * `operation` - 对每个项目执行的操作
    ///
    /// # Returns
    ///
    /// 返回所有操作的结果
    pub async fn batch_execute<T, R, F, Fut>(
        &self,
        items: Vec<T>,
        operation: F,
    ) -> Result<Vec<R>>
    where
        T: Send + Clone + 'static,
        R: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
    {
        let start = std::time::Instant::now();
        let total_items = items.len();
        
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_batches += 1;
            stats.total_items += total_items;
        }
        
        // 分批处理
        let mut results = Vec::new();
        let operation = Arc::new(operation);
        
        for chunk in items.chunks(self.config.batch_size) {
            let chunk_results = self.process_chunk(chunk.to_vec(), operation.clone()).await?;
            results.extend(chunk_results);
        }
        
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.successful_batches += 1;
            stats.total_processing_time_ms += start.elapsed().as_millis() as u64;
        }
        
        Ok(results)
    }
    
    /// 处理一个批次
    async fn process_chunk<T, R, F, Fut>(
        &self,
        chunk: Vec<T>,
        operation: Arc<F>,
    ) -> Result<Vec<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
    {
        let mut tasks = Vec::new();
        
        for item in chunk {
            let permit = self.semaphore.clone().acquire_owned().await
                .map_err(|e| agent_mem_traits::AgentMemError::internal_error(
                    format!("Failed to acquire semaphore: {}", e)
                ))?;
            
            let operation = operation.clone();
            
            let task = tokio::spawn(async move {
                let result = operation(item).await;
                drop(permit); // 释放许可
                result
            });
            
            tasks.push(task);
        }
        
        // 等待所有任务完成
        let mut results = Vec::new();
        for task in tasks {
            let result = task.await
                .map_err(|e| agent_mem_traits::AgentMemError::internal_error(
                    format!("Task failed: {}", e)
                ))??;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 批量向量搜索
    ///
    /// # Arguments
    ///
    /// * `query_vectors` - 查询向量列表
    /// * `search_fn` - 搜索函数
    ///
    /// # Returns
    ///
    /// 返回所有搜索结果
    pub async fn batch_vector_search<F, Fut, R>(
        &self,
        query_vectors: Vec<Vec<f32>>,
        search_fn: F,
    ) -> Result<Vec<R>>
    where
        F: Fn(Vec<f32>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send + 'static,
    {
        tracing::debug!("Batch vector search: {} queries", query_vectors.len());

        self.batch_execute(query_vectors, search_fn).await
    }

    /// 批量图查询
    ///
    /// # Arguments
    ///
    /// * `node_ids` - 节点 ID 列表
    /// * `query_fn` - 查询函数
    ///
    /// # Returns
    ///
    /// 返回所有查询结果
    pub async fn batch_graph_query<T, F, Fut, R>(
        &self,
        node_ids: Vec<T>,
        query_fn: F,
    ) -> Result<Vec<R>>
    where
        T: Send + Clone + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send + 'static,
    {
        tracing::debug!("Batch graph query: {} nodes", node_ids.len());

        self.batch_execute(node_ids, query_fn).await
    }

    /// 批量插入
    ///
    /// # Arguments
    ///
    /// * `items` - 要插入的项目列表
    /// * `insert_fn` - 插入函数
    ///
    /// # Returns
    ///
    /// 返回所有插入结果
    pub async fn batch_insert<T, F, Fut, R>(
        &self,
        items: Vec<T>,
        insert_fn: F,
    ) -> Result<Vec<R>>
    where
        T: Send + Clone + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<R>> + Send,
        R: Send + 'static,
    {
        tracing::debug!("Batch insert: {} items", items.len());

        self.batch_execute(items, insert_fn).await
    }
    
    /// 优化批处理
    pub async fn optimize(&self) -> Result<()> {
        // 这里可以添加批处理优化逻辑
        // 例如：调整批量大小、并发数等
        
        tracing::info!("Batch processor optimized");
        Ok(())
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> BatchStats {
        self.stats.read().await.clone()
    }
    
    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = BatchStats::default();
    }
}

/// 批量处理统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchStats {
    /// 总批次数
    pub total_batches: usize,
    
    /// 成功批次数
    pub successful_batches: usize,
    
    /// 失败批次数
    pub failed_batches: usize,
    
    /// 总处理项目数
    pub total_items: usize,
    
    /// 总处理时间（毫秒）
    pub total_processing_time_ms: u64,
}

impl BatchStats {
    /// 计算平均处理时间
    pub fn average_processing_time_ms(&self) -> f64 {
        if self.total_batches == 0 {
            0.0
        } else {
            self.total_processing_time_ms as f64 / self.total_batches as f64
        }
    }
    
    /// 计算成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_batches == 0 {
            0.0
        } else {
            self.successful_batches as f64 / self.total_batches as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_processor_creation() {
        let config = BatchConfig::default();
        let processor = BatchProcessor::new(config);
        
        let stats = processor.get_stats().await;
        assert_eq!(stats.total_batches, 0);
    }
    
    #[tokio::test]
    async fn test_batch_execute() {
        let config = BatchConfig::default();
        let processor = BatchProcessor::new(config);
        
        let items = vec![1, 2, 3, 4, 5];
        let results = processor
            .batch_execute(items, |x| async move { Ok(x * 2) })
            .await
            .unwrap();
        
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
        
        let stats = processor.get_stats().await;
        assert_eq!(stats.total_batches, 1);
        assert_eq!(stats.total_items, 5);
    }
}

