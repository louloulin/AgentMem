//! Async Storage Backend - 异步后台存储
//!
//! 实现异步后台存储，消除存储操作的阻塞延迟

use agent_mem::Memory as AgentMemApi;
use async_trait::async_trait;
use lumosai_core::llm::Message as LumosMessage;
use lumosai_core::memory::{Memory as LumosMemory, MemoryConfig};
use lumosai_core::Result as LumosResult;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::memory_adapter::AgentMemBackend;

/// 异步存储配置
#[derive(Debug, Clone)]
pub struct AsyncStorageConfig {
    /// 启用异步存储
    pub enable_async: bool,
    /// 批处理大小
    pub batch_size: usize,
    /// 批处理间隔（毫秒）
    pub batch_interval_ms: u64,
    /// 最大队列大小
    pub max_queue_size: usize,
}

impl Default for AsyncStorageConfig {
    fn default() -> Self {
        Self {
            enable_async: true,
            batch_size: 10,
            batch_interval_ms: 100,
            max_queue_size: 1000,
        }
    }
}

/// 存储任务
#[derive(Debug, Clone)]
struct StorageTask {
    message: LumosMessage,
    agent_id: String,
    user_id: String,
}

/// 异步存储Backend
pub struct AsyncStorageBackend {
    /// 底层Memory Backend
    backend: Arc<AgentMemBackend>,
    /// 存储队列
    storage_queue: Option<mpsc::UnboundedSender<StorageTask>>,
    /// 配置
    config: AsyncStorageConfig,
}

impl AsyncStorageBackend {
    /// 创建新的异步存储Backend
    pub fn new(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
        config: AsyncStorageConfig,
    ) -> Self {
        let backend = Arc::new(AgentMemBackend::new(
            memory_api.clone(),
            agent_id.clone(),
            user_id.clone(),
        ));

        let storage_queue = if config.enable_async {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let backend_clone = backend.clone();
            let batch_size = config.batch_size;
            let batch_interval = std::time::Duration::from_millis(config.batch_interval_ms);

            // 后台批处理任务
            tokio::spawn(async move {
                let mut batch = Vec::new();
                let mut last_flush = std::time::Instant::now();

                loop {
                    tokio::select! {
                        task = rx.recv() => {
                            match task {
                                Some(task) => {
                                    batch.push(task);

                                    // 达到批次大小，立即刷新
                                    if batch.len() >= batch_size {
                                        Self::flush_batch(&backend_clone, &mut batch).await;
                                        last_flush = std::time::Instant::now();
                                    }
                                }
                                None => {
                                    // 通道关闭，处理剩余任务后退出
                                    if !batch.is_empty() {
                                        Self::flush_batch(&backend_clone, &mut batch).await;
                                    }
                                    break;
                                }
                            }
                        }
                        _ = tokio::time::sleep(batch_interval) => {
                            // 定期刷新
                            if !batch.is_empty() && last_flush.elapsed() >= batch_interval {
                                Self::flush_batch(&backend_clone, &mut batch).await;
                                last_flush = std::time::Instant::now();
                            }
                        }
                    }
                }
            });

            Some(tx)
        } else {
            None
        };

        Self {
            backend,
            storage_queue,
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(
        memory_api: Arc<AgentMemApi>,
        agent_id: String,
        user_id: String,
    ) -> Self {
        Self::new(memory_api, agent_id, user_id, AsyncStorageConfig::default())
    }

    /// 刷新批次
    async fn flush_batch(backend: &Arc<AgentMemBackend>, batch: &mut Vec<StorageTask>) {
        if batch.is_empty() {
            return;
        }

        let batch_size = batch.len();
        let flush_start = std::time::Instant::now();

        // 批量存储
        for task in batch.drain(..) {
            if let Err(e) = backend.store(&task.message).await {
                warn!("   ⚠️  Failed to store message in batch: {}", e);
            }
        }

        let flush_duration = flush_start.elapsed();
        debug!(
            "   ✅ Batch flushed: {} messages in {:?}",
            batch_size,
            flush_duration
        );
    }
}

#[async_trait]
impl LumosMemory for AsyncStorageBackend {
    async fn store(&self, message: &LumosMessage) -> LumosResult<()> {
        let store_start = std::time::Instant::now();

        if let Some(ref queue) = self.storage_queue {
            // 异步存储：立即返回，后台处理
            let task = StorageTask {
                message: message.clone(),
                agent_id: self.backend.agent_id().to_string(),
                user_id: self.backend.user_id().to_string(),
            };

            queue.send(task).map_err(|_| {
                warn!("   ⚠️  Storage queue full, falling back to sync storage");
                lumosai_core::Error::Other("Storage queue full".to_string())
            })?;

            let queue_duration = store_start.elapsed();
            debug!("   ✅ Queued for async storage in {:?}", queue_duration);
            Ok(())
        } else {
            // 同步存储：直接调用
            let sync_duration = store_start.elapsed();
            let result = self.backend.store(message).await;
            info!("   ⏱️  Sync storage: {:?}", sync_duration);
            result
        }
    }

    async fn retrieve(&self, config: &MemoryConfig) -> LumosResult<Vec<LumosMessage>> {
        // 检索操作不使用异步，直接调用底层
        self.backend.retrieve(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_storage() {
        // 需要mock AgentMemApi
        // 实际测试应该在集成测试中完成
    }
}

