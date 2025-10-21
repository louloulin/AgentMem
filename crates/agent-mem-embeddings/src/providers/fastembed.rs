// FastEmbed 提供商实现
// 使用 FastEmbed-rs 库提供高性能本地嵌入

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::{Arc, Mutex};
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
pub struct FastEmbedProvider {
    /// FastEmbed 模型实例（包装在 Mutex 中以支持异步）
    model: Arc<Mutex<TextEmbedding>>,
    
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
        info!("初始化 FastEmbed 提供商: {}", config.model);
        
        // 解析模型名称
        let embedding_model = Self::parse_model(&config.model)?;
        
        // 在阻塞线程中初始化模型
        let model_clone = embedding_model.clone();
        let model = tokio::task::spawn_blocking(move || {
            TextEmbedding::try_new(
                InitOptions::new(model_clone)
                    .with_show_download_progress(true)
            )
        })
        .await
        .map_err(|e| AgentMemError::embedding_error(format!("任务失败: {}", e)))?
        .map_err(|e| AgentMemError::embedding_error(format!("FastEmbed 初始化失败: {}", e)))?;
        
        let dimension = Self::get_dimension(&embedding_model);
        
        info!("FastEmbed 模型加载成功: {} (维度: {})", config.model, dimension);
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            config: config.clone(),
            model_name: config.model,
            dimension,
        })
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
            
            _ => Err(AgentMemError::config_error(&format!(
                "不支持的 FastEmbed 模型: {}. 支持的模型: bge-small-en-v1.5, bge-base-en-v1.5, bge-large-en-v1.5, all-MiniLM-L6-v2, all-MiniLM-L12-v2, mxbai-embed-large-v1, nomic-embed-text-v1, nomic-embed-text-v1.5, multilingual-e5-small, multilingual-e5-base, multilingual-e5-large",
                model
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
        let model = self.model.clone();
        
        // 在阻塞线程中执行同步操作
        let embedding = tokio::task::spawn_blocking(move || {
            let mut model = model.lock().expect("无法获取模型锁");
            model.embed(vec![text], None)
        })
        .await
        .map_err(|e| AgentMemError::embedding_error(format!("任务失败: {}", e)))?
        .map_err(|e| AgentMemError::embedding_error(format!("嵌入生成失败: {}", e)))?;
        
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
        let model = self.model.clone();
        let batch_size = self.config.batch_size;
        
        // 在阻塞线程中执行同步操作
        let embeddings = tokio::task::spawn_blocking(move || {
            let mut model = model.lock().expect("无法获取模型锁");
            model.embed(texts, Some(batch_size))
        })
        .await
        .map_err(|e| AgentMemError::embedding_error(format!("任务失败: {}", e)))?
        .map_err(|e| AgentMemError::embedding_error(format!("批量嵌入失败: {}", e)))?;
        
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
        let models = vec![
            ("bge-small-en-v1.5", 384),
            ("all-MiniLM-L6-v2", 384),
        ];
        
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
}

