//! 嵌入批处理队列
//! 
//! 解决 Mutex 锁竞争问题：收集并发请求，批量处理嵌入生成

use agent_mem_traits::{AgentMemError, Embedder, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// 嵌入请求
struct EmbedRequest {
    text: String,
    responder: oneshot::Sender<Result<Vec<f32>>>,
}

/// 嵌入批处理队列
/// 
/// 收集并发请求，定期批量处理，减少 Mutex 锁竞争
pub struct EmbeddingQueue {
    /// 请求发送通道
    request_tx: mpsc::UnboundedSender<EmbedRequest>,
    
    /// 批处理大小
    batch_size: usize,
    
    /// 批处理间隔（毫秒）
    batch_interval_ms: u64,
}

impl EmbeddingQueue {
    /// 创建新的嵌入队列
    /// 
    /// # 参数
    /// - `embedder`: 底层嵌入器
    /// - `batch_size`: 批处理大小（默认 32）
    /// - `batch_interval_ms`: 批处理间隔（毫秒，默认 10ms）
    pub fn new(
        embedder: Arc<dyn Embedder + Send + Sync>,
        batch_size: usize,
        batch_interval_ms: u64,
    ) -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        
        // 启动批处理任务
        let embedder_clone = embedder.clone();
        tokio::spawn(Self::batch_processor(
            embedder_clone,
            request_rx,
            batch_size,
            batch_interval_ms,
        ));
        
        Self {
            request_tx,
            batch_size,
            batch_interval_ms,
        }
    }
    
    /// 批处理任务
    async fn batch_processor(
        embedder: Arc<dyn Embedder + Send + Sync>,
        mut request_rx: mpsc::UnboundedReceiver<EmbedRequest>,
        batch_size: usize,
        batch_interval_ms: u64,
    ) {
        let mut batch = Vec::new();
        let mut last_batch_time = Instant::now();
        let batch_interval = Duration::from_millis(batch_interval_ms);
        
        loop {
            // 收集批处理请求
            let timeout = tokio::time::sleep(batch_interval);
            tokio::pin!(timeout);
            
            loop {
                tokio::select! {
                    // 接收新请求
                    request = request_rx.recv() => {
                        match request {
                            Some(req) => {
                                batch.push(req);
                                
                                // 如果达到批处理大小，立即处理
                                if batch.len() >= batch_size {
                                    break;
                                }
                            }
                            None => {
                                // 通道关闭，处理剩余请求后退出
                                if !batch.is_empty() {
                                    Self::process_batch(&embedder, &mut batch).await;
                                }
                                return;
                            }
                        }
                    }
                    // 超时处理
                    _ = &mut timeout => {
                        break;
                    }
                }
            }
            
            // 处理批处理
            if !batch.is_empty() {
                let elapsed = last_batch_time.elapsed();
                if elapsed >= batch_interval || batch.len() >= batch_size {
                    Self::process_batch(&embedder, &mut batch).await;
                    last_batch_time = Instant::now();
                }
            }
        }
    }
    
    /// 处理一批请求
    async fn process_batch(
        embedder: &Arc<dyn Embedder + Send + Sync>,
        batch: &mut Vec<EmbedRequest>,
    ) {
        if batch.is_empty() {
            return;
        }
        
        let batch_size = batch.len();
        let start = Instant::now();
        
        // 提取所有文本
        let texts: Vec<String> = batch.iter().map(|req| req.text.clone()).collect();
        
        // 批量生成嵌入
        match embedder.embed_batch(&texts).await {
            Ok(embeddings) => {
                let elapsed = start.elapsed();
                debug!(
                    "批量嵌入生成成功: {} 个文本，耗时: {:?}，平均: {:?}",
                    batch_size,
                    elapsed,
                    elapsed / batch_size as u32
                );
                
                // 发送结果
                for (i, (req, embedding)) in batch.iter().zip(embeddings.iter()).enumerate() {
                    if let Err(_) = req.responder.send(Ok(embedding.clone())) {
                        warn!("无法发送嵌入结果（接收端已关闭）: {}", i);
                    }
                }
            }
            Err(e) => {
                warn!("批量嵌入生成失败: {}", e);
                
                // 发送错误
                for req in batch.iter() {
                    let _ = req.responder.send(Err(AgentMemError::embedding_error(format!(
                        "批量嵌入失败: {}",
                        e
                    ))));
                }
            }
        }
        
        batch.clear();
    }
    
    /// 提交嵌入请求
    /// 
    /// # 参数
    /// - `text`: 要嵌入的文本
    /// 
    /// # 返回
    /// - `Ok(Vec<f32>)`: 嵌入向量
    /// - `Err(AgentMemError)`: 嵌入失败
    pub async fn embed(&self, text: String) -> Result<Vec<f32>> {
        let (tx, rx) = oneshot::channel();
        
        let request = EmbedRequest {
            text,
            responder: tx,
        };
        
        // 发送请求
        self.request_tx
            .send(request)
            .map_err(|_| AgentMemError::embedding_error("嵌入队列已关闭"))?;
        
        // 等待结果
        rx.await
            .map_err(|_| AgentMemError::embedding_error("嵌入请求超时或队列关闭"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EmbeddingConfig;
    use crate::providers::FastEmbedProvider;
    
    #[tokio::test]
    #[ignore] // 需要下载模型，默认跳过
    async fn test_embedding_queue() {
        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            batch_size: 32,
            ..Default::default()
        };
        
        let embedder = Arc::new(FastEmbedProvider::new(config).await.unwrap());
        let queue = EmbeddingQueue::new(embedder, 10, 10);
        
        // 测试单个嵌入
        let embedding = queue.embed("Hello, world!".to_string()).await.unwrap();
        assert_eq!(embedding.len(), 384);
        
        // 测试并发嵌入
        let mut tasks = Vec::new();
        for i in 0..20 {
            let queue_clone = &queue;
            let task = tokio::spawn(async move {
                queue_clone
                    .embed(format!("Test text {}", i))
                    .await
            });
            tasks.push(task);
        }
        
        let mut success_count = 0;
        for task in tasks {
            if task.await.unwrap().is_ok() {
                success_count += 1;
            }
        }
        
        assert_eq!(success_count, 20);
    }
}
