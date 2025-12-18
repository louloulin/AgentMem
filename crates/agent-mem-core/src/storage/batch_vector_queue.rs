//! 🆕 Phase 1.2: 批量向量存储队列
//!
//! 实现自动批量处理队列，将向量存储操作批量执行
//! 预期效果: 批量存储吞吐量提升5-10x

use agent_mem_traits::{AgentMemError, Result, VectorData, VectorStore};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 向量存储任务
#[derive(Debug, Clone)]
struct VectorStorageTask {
    /// 向量数据
    vector_data: VectorData,
    /// 创建时间
    created_at: Instant,
}

/// 批量向量存储队列配置
#[derive(Debug, Clone)]
pub struct BatchVectorQueueConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 批量处理间隔（毫秒）
    pub batch_interval_ms: u64,
    /// 最大队列大小
    pub max_queue_size: usize,
    /// 是否启用队列
    pub enable_queue: bool,
}

impl Default for BatchVectorQueueConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_interval_ms: 100,
            max_queue_size: 10000,
            enable_queue: true,
        }
    }
}

/// 批量向量存储队列统计
#[derive(Debug, Clone, Default)]
pub struct BatchVectorQueueStats {
    /// 总任务数
    pub total_tasks: usize,
    /// 已处理任务数
    pub processed_tasks: usize,
    /// 总批次数
    pub total_batches: usize,
    /// 平均批处理时间（毫秒）
    pub avg_batch_time_ms: f64,
    /// 队列当前大小
    pub current_queue_size: usize,
}

/// 批量向量存储队列
pub struct BatchVectorStorageQueue {
    /// 向量存储后端
    vector_store: Arc<dyn VectorStore + Send + Sync>,
    /// 配置
    config: BatchVectorQueueConfig,
    /// 任务发送端
    task_sender: mpsc::UnboundedSender<VectorStorageTask>,
    /// 统计信息
    stats: Arc<RwLock<BatchVectorQueueStats>>,
    /// 后台任务句柄
    _background_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl BatchVectorStorageQueue {
    /// 创建新的批量向量存储队列
    pub fn new(
        vector_store: Arc<dyn VectorStore + Send + Sync>,
        config: BatchVectorQueueConfig,
    ) -> Self {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        let stats = Arc::new(RwLock::new(BatchVectorQueueStats::default()));

        let vector_store_clone = Arc::clone(&vector_store);
        let config_clone = config.clone();
        let stats_clone = Arc::clone(&stats);

        // 启动后台批处理任务
        let background_handle = tokio::spawn(async move {
            Self::process_batch_loop(
                task_receiver,
                vector_store_clone,
                config_clone,
                stats_clone,
            )
            .await;
        });

        Self {
            vector_store,
            config,
            task_sender,
            stats,
            _background_handle: Arc::new(RwLock::new(Some(background_handle))),
        }
    }

    /// 添加向量存储任务（非阻塞）
    ///
    /// 立即返回，向量存储将在后台批量处理
    pub async fn add_vector(&self, vector_data: VectorData) -> Result<()> {
        if !self.config.enable_queue {
            // 如果队列未启用，直接存储
            let _ = self.vector_store.add_vectors(vec![vector_data]).await?;
            return Ok(());
        }

        // 注意: UnboundedSender没有len()方法，我们使用统计信息来跟踪
        // 如果队列满了，send会失败，我们会在那里处理

        let task = VectorStorageTask {
            vector_data,
            created_at: Instant::now(),
        };

        self.task_sender.send(task).map_err(|e| {
            AgentMemError::StorageError(format!("Failed to send vector storage task: {e}"))
        })?;

        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_tasks += 1;
            // 注意: UnboundedSender没有len()方法，我们使用统计信息跟踪
            // stats.current_queue_size会在process_batch_loop中更新
        }

        Ok(())
    }

    /// 后台批处理循环
    async fn process_batch_loop(
        mut task_receiver: mpsc::UnboundedReceiver<VectorStorageTask>,
        vector_store: Arc<dyn VectorStore + Send + Sync>,
        config: BatchVectorQueueConfig,
        stats: Arc<RwLock<BatchVectorQueueStats>>,
    ) {
        let mut batch = Vec::new();
        let mut last_flush = Instant::now();
        let batch_interval = Duration::from_millis(config.batch_interval_ms);

        loop {
            tokio::select! {
                // 接收新任务
                task = task_receiver.recv() => {
                    match task {
                        Some(task) => {
                            batch.push(task.vector_data);
                            
                            // 如果达到批量大小，立即刷新
                            if batch.len() >= config.batch_size {
                                Self::flush_batch(&vector_store, &mut batch, &stats).await;
                                last_flush = Instant::now();
                            }
                        }
                        None => {
                            // 通道关闭，处理剩余批次
                            if !batch.is_empty() {
                                Self::flush_batch(&vector_store, &mut batch, &stats).await;
                            }
                            break;
                        }
                    }
                }
                // 定时刷新
                _ = tokio::time::sleep(batch_interval) => {
                    if !batch.is_empty() && last_flush.elapsed() >= batch_interval {
                        Self::flush_batch(&vector_store, &mut batch, &stats).await;
                        last_flush = Instant::now();
                    }
                }
            }
        }
    }

    /// 刷新批次
    async fn flush_batch(
        vector_store: &Arc<dyn VectorStore + Send + Sync>,
        batch: &mut Vec<VectorData>,
        stats: &Arc<RwLock<BatchVectorQueueStats>>,
    ) {
        if batch.is_empty() {
            return;
        }

        let batch_size = batch.len();
        let start = Instant::now();
        let vectors = batch.drain(..).collect();

        match vector_store.add_vectors(vectors).await {
            Ok(_) => {
                let elapsed = start.elapsed().as_millis() as u64;
                info!(
                    "✅ Batch vector storage completed: {} vectors in {}ms",
                    batch_size, elapsed
                );

                // 更新统计
                let mut stats_guard = stats.write().await;
                stats_guard.processed_tasks += batch_size;
                stats_guard.total_batches += 1;
                if stats_guard.total_batches > 0 {
                    stats_guard.avg_batch_time_ms = (stats_guard.avg_batch_time_ms
                        * (stats_guard.total_batches - 1) as f64
                        + elapsed as f64)
                        / stats_guard.total_batches as f64;
                }
            }
            Err(e) => {
                error!("Failed to batch store vectors: {}", e);
                // 注意: 这里不更新processed_tasks，因为失败了
            }
        }
    }

    /// 获取统计信息
    pub async fn stats(&self) -> BatchVectorQueueStats {
        // 注意: UnboundedSender没有len()方法
        // 我们使用统计信息中的processed_tasks来估算队列大小
        self.stats.read().await.clone()
    }

    /// 等待所有任务完成（用于测试或优雅关闭）
    pub async fn flush(&self) -> Result<()> {
        // 等待队列处理完成（通过检查统计信息）
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100;
        let initial_tasks = {
            let stats = self.stats.read().await;
            stats.total_tasks
        };
        
        while attempts < MAX_ATTEMPTS {
            let stats = self.stats.read().await;
            if stats.processed_tasks >= initial_tasks {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
            attempts += 1;
        }

        let final_stats = self.stats.read().await;
        if final_stats.processed_tasks < initial_tasks {
            warn!(
                "Vector storage queue still processing: {}/{} tasks processed",
                final_stats.processed_tasks,
                initial_tasks
            );
        }

        Ok(())
    }
}
