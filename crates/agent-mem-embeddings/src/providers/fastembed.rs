// FastEmbed 提供商实现
// 使用 FastEmbed-rs 库提供高性能本地嵌入

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};

/// FastEmbed 提供商
///
/// 使用 FastEmbed-rs 库提供本地嵌入功能，支持多种预训练模型。
///
/// # 特性
/// - 完全本地运行，无需 API 调用
/// - 支持 19+ 预训练模型
/// - 自动下载和缓存模型
/// - 高性能（延迟 < 10ms）
/// - 批处理优化
/// - 多模型实例池（解决 Mutex 锁竞争）
pub struct FastEmbedProvider {
    /// FastEmbed 模型实例池（多个实例避免锁竞争）
    /// 优化：使用模型池，每个请求使用不同的模型实例，避免 Mutex 锁竞争
    /// 参考 Mem0 的实现，使用多个模型实例来提升并发性能
    model_pool: Vec<Arc<Mutex<TextEmbedding>>>,

    /// 轮询计数器（用于选择模型实例）
    counter: Arc<AtomicUsize>,

    /// 配置
    config: EmbeddingConfig,

    /// 模型名称
    model_name: String,

    /// 嵌入维度
    dimension: usize,
}

impl FastEmbedProvider {
    /// 创建新的 FastEmbed 提供商
    ///
    /// # 参数
    /// - `config`: 嵌入配置
    ///
    /// # 返回
    /// - `Ok(FastEmbedProvider)`: 成功创建的提供商
    /// - `Err(AgentMemError)`: 创建失败
    ///
    /// # 示例
    /// ```no_run
    /// use agent_mem_embeddings::{EmbeddingConfig, providers::FastEmbedProvider};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = EmbeddingConfig {
    ///     provider: "fastembed".to_string(),
    ///     model: "bge-small-en-v1.5".to_string(),
    ///     dimension: 384,
    ///     ..Default::default()
    /// };
    ///
    /// let provider = FastEmbedProvider::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        // 默认使用 CPU 核心数作为模型池大小
        // 这样可以充分利用多核 CPU，避免 Mutex 锁竞争
        let pool_size = num_cpus::get().max(1); // 至少1个实例
        Self::new_with_pool_size(config, pool_size).await
    }

    /// 创建新的 FastEmbed 提供商（指定模型池大小）
    ///
    /// # 参数
    /// - `config`: 嵌入配置
    /// - `pool_size`: 模型池大小（建议等于 CPU 核心数）
    ///
    /// # 返回
    /// - `Ok(FastEmbedProvider)`: 成功创建的提供商
    /// - `Err(AgentMemError)`: 创建失败
    ///
    /// # 性能优化说明
    /// 使用多个模型实例可以避免 Mutex 锁竞争，提升并发性能。
    /// 参考 Mem0 的实现，每个 CPU 核心使用一个模型实例。
    pub async fn new_with_pool_size(config: EmbeddingConfig, pool_size: usize) -> Result<Self> {
        info!("初始化 FastEmbed 提供商: {} (池大小: {})", config.model, pool_size);

        // 解析模型名称
        let embedding_model = Self::parse_model(&config.model)?;
        let dimension = Self::get_dimension(&embedding_model);

        // 创建模型池：每个 CPU 核心一个模型实例
        // 这样可以避免 Mutex 锁竞争，多个并发请求可以使用不同的模型实例
        let mut model_pool = Vec::with_capacity(pool_size);
        
        info!("正在初始化 {} 个模型实例...", pool_size);
        
        // 并行初始化模型实例（使用 tokio::join! 并行执行）
        let init_tasks: Vec<_> = (0..pool_size)
            .map(|i| {
                let model_clone = embedding_model.clone();
                tokio::task::spawn_blocking(move || {
                    info!("初始化模型实例 {} / {}", i + 1, pool_size);
                    TextEmbedding::try_new(
                        InitOptions::new(model_clone)
                            .with_show_download_progress(i == 0) // 只显示第一个的进度
                    )
                })
            })
            .collect();

        // 等待所有模型实例初始化完成
        for (i, task) in init_tasks.into_iter().enumerate() {
            let model = task
                .await
                .map_err(|e| AgentMemError::embedding_error(format!("任务失败: {e}")))?
                .map_err(|e| AgentMemError::embedding_error(format!("FastEmbed 初始化失败 (实例 {}): {e}", i + 1)))?;
            model_pool.push(Arc::new(Mutex::new(model)));
        }

        info!(
            "FastEmbed 模型池初始化成功: {} (维度: {}, 池大小: {})",
            config.model, dimension, pool_size
        );

        Ok(Self {
            model_pool,
            counter: Arc::new(AtomicUsize::new(0)),
            config: config.clone(),
            model_name: config.model,
            dimension,
        })
    }

    /// 获取下一个模型实例（轮询方式）
    fn get_model(&self) -> Arc<Mutex<TextEmbedding>> {
        let index = self.counter.fetch_add(1, Ordering::Relaxed) % self.model_pool.len();
        self.model_pool[index].clone()
    }

    /// 解析模型名称到 FastEmbed 模型枚举
    fn parse_model(model: &str) -> Result<EmbeddingModel> {
        match model {
            // BGE 系列（推荐）
            "bge-small-en-v1.5" | "BAAI/bge-small-en-v1.5" => Ok(EmbeddingModel::BGESmallENV15),
            "bge-base-en-v1.5" | "BAAI/bge-base-en-v1.5" => Ok(EmbeddingModel::BGEBaseENV15),
            "bge-large-en-v1.5" | "BAAI/bge-large-en-v1.5" => Ok(EmbeddingModel::BGELargeENV15),
            
            // MiniLM 系列（轻量级）
            "all-MiniLM-L6-v2" | "sentence-transformers/all-MiniLM-L6-v2" => {
                Ok(EmbeddingModel::AllMiniLML6V2)
            }
            "all-MiniLM-L12-v2" | "sentence-transformers/all-MiniLM-L12-v2" => {
                Ok(EmbeddingModel::AllMiniLML12V2)
            }
            
            // MixedBread 系列
            "mxbai-embed-large-v1" | "mixedbread-ai/mxbai-embed-large-v1" => {
                Ok(EmbeddingModel::MxbaiEmbedLargeV1)
            }
            
            // Nomic 系列
            "nomic-embed-text-v1" | "nomic-ai/nomic-embed-text-v1" => {
                Ok(EmbeddingModel::NomicEmbedTextV1)
            }
            "nomic-embed-text-v1.5" | "nomic-ai/nomic-embed-text-v1.5" => {
                Ok(EmbeddingModel::NomicEmbedTextV15)
            }
            
            // E5 多语言系列
            "multilingual-e5-small" | "intfloat/multilingual-e5-small" => {
                Ok(EmbeddingModel::MultilingualE5Small)
            }
            "multilingual-e5-base" | "intfloat/multilingual-e5-base" => {
                Ok(EmbeddingModel::MultilingualE5Base)
            }
            "multilingual-e5-large" | "intfloat/multilingual-e5-large" => {
                Ok(EmbeddingModel::MultilingualE5Large)
            }
            
            _ => Err(AgentMemError::config_error(format!(
                "不支持的 FastEmbed 模型: {model}. 支持的模型: bge-small-en-v1.5, bge-base-en-v1.5, bge-large-en-v1.5, all-MiniLM-L6-v2, all-MiniLM-L12-v2, mxbai-embed-large-v1, nomic-embed-text-v1, nomic-embed-text-v1.5, multilingual-e5-small, multilingual-e5-base, multilingual-e5-large"
            ))),
        }
    }

    /// 获取模型维度
    fn get_dimension(model: &EmbeddingModel) -> usize {
        match model {
            EmbeddingModel::BGESmallENV15 => 384,
            EmbeddingModel::BGEBaseENV15 => 768,
            EmbeddingModel::BGELargeENV15 => 1024,
            EmbeddingModel::AllMiniLML6V2 => 384,
            EmbeddingModel::AllMiniLML12V2 => 384,
            EmbeddingModel::MxbaiEmbedLargeV1 => 1024,
            EmbeddingModel::NomicEmbedTextV1 => 768,
            EmbeddingModel::NomicEmbedTextV15 => 768,
            EmbeddingModel::MultilingualE5Small => 384,
            EmbeddingModel::MultilingualE5Base => 768,
            EmbeddingModel::MultilingualE5Large => 1024,
            _ => 768, // 默认维度
        }
    }
}

#[async_trait]
impl Embedder for FastEmbedProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("FastEmbed 生成嵌入: {} 字符", text.len());

        let text = text.to_string();
        // 优化：使用模型池，轮询选择模型实例，避免 Mutex 锁竞争
        // 多个并发请求可以使用不同的模型实例，实现真正的并行处理
        let model = self.get_model();

        // 在阻塞线程中获取锁和执行嵌入生成
        // 由于使用了模型池，不同请求使用不同的模型实例，锁竞争大大减少
        let embedding_result = tokio::task::spawn_blocking(move || {
            let mut model_guard = match model.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    return Err(AgentMemError::embedding_error(format!("无法获取模型锁: {e}")));
                }
            };
            model_guard.embed(vec![text], None)
                .map_err(|e| AgentMemError::embedding_error(format!("嵌入生成失败: {e}")))
        })
        .await
        .map_err(|e| AgentMemError::embedding_error(format!("任务失败: {e}")))?
        .map_err(|e| AgentMemError::embedding_error(format!("嵌入生成失败: {e}")))?;
        
        let embedding = embedding_result;

        embedding
            .into_iter()
            .next()
            .ok_or_else(|| AgentMemError::embedding_error("未返回嵌入"))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        debug!("FastEmbed 批量生成嵌入: {} 个文本", texts.len());

        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let texts = texts.to_vec();
        // 优化：批量处理使用第一个模型实例（批量处理本身已经高效）
        // 如果需要更高的并发，可以考虑将批量任务拆分到多个模型实例
        let model = self.model_pool[0].clone();
        let batch_size = self.config.batch_size;

        // 在阻塞线程中获取锁和执行批量嵌入生成
        // 批量处理可以显著减少锁竞争（一次处理多个文本）
        let embeddings_result = tokio::task::spawn_blocking(move || {
            let mut model_guard = match model.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    return Err(AgentMemError::embedding_error(format!("无法获取模型锁: {e}")));
                }
            };
            model_guard.embed(texts, Some(batch_size))
                .map_err(|e| AgentMemError::embedding_error(format!("批量嵌入失败: {e}")))
        })
        .await
        .map_err(|e| AgentMemError::embedding_error(format!("任务失败: {e}")))?
        .map_err(|e| AgentMemError::embedding_error(format!("批量嵌入失败: {e}")))?;
        
        let embeddings = embeddings_result;

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn health_check(&self) -> Result<bool> {
        // 测试嵌入生成
        match self.embed("health check").await {
            Ok(embedding) => {
                if embedding.len() == self.dimension {
                    debug!("FastEmbed 健康检查通过");
                    Ok(true)
                } else {
                    warn!(
                        "FastEmbed 健康检查失败: 维度不匹配 (期望: {}, 实际: {})",
                        self.dimension,
                        embedding.len()
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("FastEmbed 健康检查失败: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    #[ignore] // 需要下载模型，默认跳过
    async fn test_fastembed_provider_basic() {
        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            batch_size: 256,
            ..Default::default()
        };

        let provider = FastEmbedProvider::new(config).await.unwrap();

        // 测试单个嵌入
        let embedding = provider.embed("Hello, world!").await.unwrap();
        assert_eq!(embedding.len(), 384);

        // 测试批量嵌入
        let texts = vec!["Hello".to_string(), "World".to_string()];
        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);

        // 测试健康检查
        assert!(provider.health_check().await.unwrap());
    }

    #[tokio::test]
    #[ignore] // 需要下载模型，默认跳过
    async fn test_fastembed_models() {
        let models = vec![("bge-small-en-v1.5", 384), ("all-MiniLM-L6-v2", 384)];

        for (model, dim) in models {
            let config = EmbeddingConfig {
                provider: "fastembed".to_string(),
                model: model.to_string(),
                dimension: dim,
                ..Default::default()
            };

            let provider = FastEmbedProvider::new(config).await.unwrap();
            assert_eq!(provider.dimension(), dim);

            let embedding = provider.embed("test").await.unwrap();
            assert_eq!(embedding.len(), dim);
        }
    }

    #[tokio::test]
    #[ignore] // 需要下载模型，默认跳过
    async fn test_fastembed_model_pool() {
        // 测试模型池功能：多个并发请求应该使用不同的模型实例
        let config = EmbeddingConfig {
            provider: "fastembed".to_string(),
            model: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            ..Default::default()
        };

        // 创建一个小池（2个实例）用于测试
        let provider = Arc::new(FastEmbedProvider::new_with_pool_size(config, 2).await.unwrap());
        assert_eq!(provider.dimension(), 384);

        // 并发测试：多个请求应该能够并行处理
        let tasks: Vec<_> = (0..4)
            .map(|i| {
                let provider = provider.clone();
                tokio::spawn(async move {
                    let text = format!("test text {}", i);
                    provider.embed(&text).await
                })
            })
            .collect();

        // 等待所有任务完成
        for task in tasks {
            let result = task.await.unwrap();
            let embedding = result.unwrap();
            assert_eq!(embedding.len(), 384);
        }
    }
}
