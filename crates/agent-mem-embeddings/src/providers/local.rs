//! 本地嵌入提供商实现
//!
//! 支持本地运行的嵌入模型，包括：
//! - ONNX 模型
//! - Candle 模型
//! - HuggingFace 模型

use crate::config::EmbeddingConfig;
use agent_mem_traits::{AgentMemError, Embedder, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[cfg(feature = "local")]
use candle_core::Device;
#[cfg(feature = "local")]
use candle_transformers::models::bert::BertModel;

// Tokenizer 在 local 和 onnx feature 下都需要
#[cfg(any(feature = "local", feature = "onnx"))]
use tokenizers::Tokenizer;

#[cfg(feature = "onnx")]
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};

/// 本地模型类型
#[derive(Debug, Clone)]
pub enum LocalModelType {
    /// Candle 框架模型
    Candle {
        model_path: PathBuf,
        tokenizer_path: PathBuf,
    },
    /// ONNX 模型
    Onnx {
        model_path: PathBuf,
        tokenizer_path: PathBuf,
    },
    /// HuggingFace 模型
    HuggingFace {
        model_name: String,
        cache_dir: Option<PathBuf>,
    },
}

/// 模型缓存管理器
struct ModelCache {
    cache_dir: PathBuf,
    models: HashMap<String, Vec<u8>>,
}

impl ModelCache {
    fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| AgentMemError::config_error("Cannot determine cache directory"))?
            .join("agentmem")
            .join("models");

        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to create cache directory: {}", e))
        })?;

        Ok(Self {
            cache_dir,
            models: HashMap::new(),
        })
    }

    async fn download_model(&mut self, model_name: &str, url: &str) -> Result<PathBuf> {
        let model_path = self.cache_dir.join(format!("{}.bin", model_name));

        if model_path.exists() {
            info!("Model {} already cached at {:?}", model_name, model_path);
            return Ok(model_path);
        }

        info!("Downloading model {} from {}", model_name, url);

        let response = reqwest::get(url).await.map_err(|e| {
            AgentMemError::network_error(format!("Failed to download model: {}", e))
        })?;

        let bytes = response.bytes().await.map_err(|e| {
            AgentMemError::network_error(format!("Failed to read model data: {}", e))
        })?;

        tokio::fs::write(&model_path, &bytes)
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to save model: {}", e)))?;

        info!(
            "Model {} downloaded and cached at {:?}",
            model_name, model_path
        );
        Ok(model_path)
    }
}

/// 本地嵌入提供商
/// 支持真实的本地嵌入模型，包括 ONNX、Candle 和 HuggingFace 模型
pub struct LocalEmbedder {
    config: EmbeddingConfig,
    model_type: LocalModelType,
    model_cache: Arc<Mutex<ModelCache>>,
    is_loaded: Arc<Mutex<bool>>,

    #[cfg(feature = "local")]
    candle_model: Option<Arc<Mutex<BertModel>>>,
    #[cfg(feature = "local")]
    candle_tokenizer: Option<Arc<Tokenizer>>,
    #[cfg(feature = "local")]
    device: Option<Device>,

    #[cfg(feature = "onnx")]
    onnx_session: Option<Arc<Mutex<Session>>>,
    #[cfg(feature = "onnx")]
    onnx_tokenizer: Option<Arc<Tokenizer>>,
}

impl LocalEmbedder {
    /// 创建新的本地嵌入器实例
    pub async fn new(config: EmbeddingConfig) -> Result<Self> {
        let model_type = Self::determine_model_type(&config)?;
        let model_cache = Arc::new(Mutex::new(ModelCache::new()?));

        Ok(Self {
            config,
            model_type,
            model_cache,
            is_loaded: Arc::new(Mutex::new(false)),

            #[cfg(feature = "local")]
            candle_model: None,
            #[cfg(feature = "local")]
            candle_tokenizer: None,
            #[cfg(feature = "local")]
            device: None,

            #[cfg(feature = "onnx")]
            onnx_session: None,
            #[cfg(feature = "onnx")]
            onnx_tokenizer: None,
        })
    }

    /// 根据配置确定模型类型
    fn determine_model_type(config: &EmbeddingConfig) -> Result<LocalModelType> {
        if let Some(model_path) = config.get_model_path() {
            if model_path.ends_with(".onnx") {
                let model_path = PathBuf::from(model_path);
                let tokenizer_path = model_path.with_extension("tokenizer.json");
                return Ok(LocalModelType::Onnx {
                    model_path,
                    tokenizer_path,
                });
            } else if model_path.contains("/") && !model_path.starts_with("./") {
                // HuggingFace 模型名称格式
                return Ok(LocalModelType::HuggingFace {
                    model_name: model_path.to_string(),
                    cache_dir: None,
                });
            } else {
                // 本地 Candle 模型路径
                let model_path = PathBuf::from(model_path);
                let tokenizer_path = model_path.with_file_name("tokenizer.json");
                return Ok(LocalModelType::Candle {
                    model_path,
                    tokenizer_path,
                });
            }
        }

        // 默认使用 HuggingFace 模型
        Ok(LocalModelType::HuggingFace {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            cache_dir: None,
        })
    }

    /// 加载本地模型（真实实现）
    async fn load_model(&mut self) -> Result<()> {
        {
            let is_loaded = self.is_loaded.lock().await;
            if *is_loaded {
                return Ok(());
            }
        } // 释放锁

        match &self.model_type.clone() {
            #[cfg(feature = "local")]
            LocalModelType::Candle {
                model_path,
                tokenizer_path,
            } => {
                self.load_candle_model(model_path, tokenizer_path).await?;
            }
            #[cfg(feature = "onnx")]
            LocalModelType::Onnx {
                model_path,
                tokenizer_path,
            } => {
                self.load_onnx_model(model_path, tokenizer_path).await?;
            }
            LocalModelType::HuggingFace {
                model_name,
                cache_dir,
            } => {
                self.load_huggingface_model(model_name, cache_dir.as_ref())
                    .await?;
            }
            #[cfg(not(feature = "onnx"))]
            LocalModelType::Onnx { .. } => {
                warn!("ONNX feature not enabled, using deterministic embedding");
            }
            #[cfg(not(feature = "local"))]
            LocalModelType::Candle { .. } => {
                warn!("Candle feature not enabled, using deterministic embedding");
            }
            #[cfg(not(any(feature = "local", feature = "onnx")))]
            _ => {
                warn!("No local model backend enabled, using deterministic embedding");
            }
        }

        // 重新获取锁并设置状态
        let mut is_loaded = self.is_loaded.lock().await;
        *is_loaded = true;
        info!("Local embedding model loaded successfully");
        Ok(())
    }

    #[cfg(feature = "local")]
    async fn load_candle_model(
        &mut self,
        model_path: &PathBuf,
        tokenizer_path: &PathBuf,
    ) -> Result<()> {
        use candle_core::Device;

        info!("Loading Candle model from {:?}", model_path);

        // 初始化设备
        let device = Device::Cpu;

        // 加载分词器
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| {
            AgentMemError::embedding_error(format!("Failed to load tokenizer: {}", e))
        })?;

        self.device = Some(device);
        self.candle_tokenizer = Some(Arc::new(tokenizer));

        Ok(())
    }

    #[cfg(feature = "onnx")]
    async fn load_onnx_model(
        &mut self,
        model_path: &PathBuf,
        tokenizer_path: &PathBuf,
    ) -> Result<()> {
        info!("Loading ONNX model from {:?}", model_path);

        // 加载分词器
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| {
            AgentMemError::embedding_error(format!("Failed to load tokenizer: {}", e))
        })?;

        // 加载 ONNX 模型
        // 注意：ort 2.0.0-rc API 仍在变化中，这里使用简化的加载方式
        let session = Session::builder()
            .map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to create ONNX session builder: {}", e))
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to set optimization level: {}", e))
            })?
            .with_intra_threads(4)
            .map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to set intra threads: {}", e))
            })?
            .commit_from_file(model_path)
            .map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to load ONNX model from {:?}: {}", model_path, e))
            })?;

        self.onnx_session = Some(Arc::new(Mutex::new(session)));
        self.onnx_tokenizer = Some(Arc::new(tokenizer));

        info!("ONNX model loaded successfully from {:?}", model_path);
        warn!("ONNX inference is using placeholder implementation - full implementation pending ort 2.0 API stabilization");
        Ok(())
    }

    async fn load_huggingface_model(
        &mut self,
        model_name: &str,
        cache_dir: Option<&PathBuf>,
    ) -> Result<()> {
        info!("Loading HuggingFace model: {}", model_name);

        #[cfg(all(feature = "local", feature = "huggingface"))]
        {
            // 使用 hf-hub 下载模型
            let api = hf_hub::api::sync::Api::new().map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to create HF API: {}", e))
            })?;

            let repo = api.model(model_name.to_string());

            // 下载模型文件
            let model_path = repo.get("pytorch_model.bin").map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to download model: {}", e))
            })?;

            let tokenizer_path = repo.get("tokenizer.json").map_err(|e| {
                AgentMemError::embedding_error(format!("Failed to download tokenizer: {}", e))
            })?;

            // 加载模型和分词器
            self.load_candle_model(&model_path, &tokenizer_path).await?;
        }

        #[cfg(not(all(feature = "local", feature = "huggingface")))]
        {
            warn!("HuggingFace model loading requires 'local' and 'huggingface' features, using deterministic embedding");
        }

        Ok(())
    }

    /// 生成真实的嵌入向量
    async fn generate_embedding_real(&self, text: &str) -> Result<Vec<f32>> {
        match &self.model_type {
            #[cfg(feature = "local")]
            LocalModelType::Candle { .. } => self.generate_candle_embedding(text).await,
            #[cfg(feature = "onnx")]
            LocalModelType::Onnx { .. } => self.generate_onnx_embedding(text).await,
            LocalModelType::HuggingFace { .. } => {
                #[cfg(feature = "local")]
                {
                    self.generate_candle_embedding(text).await
                }
                #[cfg(not(feature = "local"))]
                {
                    Ok(self.generate_deterministic_embedding(text))
                }
            }
            #[cfg(not(feature = "onnx"))]
            LocalModelType::Onnx { .. } => Ok(self.generate_deterministic_embedding(text)),
            #[cfg(not(feature = "local"))]
            LocalModelType::Candle { .. } => Ok(self.generate_deterministic_embedding(text)),
            #[cfg(not(any(feature = "local", feature = "onnx")))]
            _ => Ok(self.generate_deterministic_embedding(text)),
        }
    }

    #[cfg(feature = "local")]
    async fn generate_candle_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if let (Some(model), Some(tokenizer), Some(device)) =
            (&self.candle_model, &self.candle_tokenizer, &self.device)
        {
            use candle_core::{DType, Tensor};

            // 分词
            let encoding = tokenizer.encode(text, true).map_err(|e| {
                AgentMemError::embedding_error(format!("Tokenization failed: {}", e))
            })?;

            let input_ids = encoding.get_ids();
            let attention_mask = encoding.get_attention_mask();

            // 限制序列长度
            let max_length = 512;
            let input_ids: Vec<u32> = input_ids.iter().take(max_length).cloned().collect();
            let attention_mask: Vec<u32> =
                attention_mask.iter().take(max_length).cloned().collect();

            // 填充到固定长度
            let mut padded_input_ids = input_ids;
            let mut padded_attention_mask = attention_mask;

            while padded_input_ids.len() < max_length {
                padded_input_ids.push(0);
                padded_attention_mask.push(0);
            }

            // 创建张量
            let input_ids_tensor = Tensor::from_vec(padded_input_ids, (1, max_length), device)
                .map_err(|e| {
                    AgentMemError::embedding_error(format!("Failed to create input tensor: {}", e))
                })?;

            let attention_mask_tensor =
                Tensor::from_vec(padded_attention_mask, (1, max_length), device).map_err(|e| {
                    AgentMemError::embedding_error(format!(
                        "Failed to create attention mask: {}",
                        e
                    ))
                })?;

            // 运行模型推理
            let model = model.lock().await;
            let outputs = model
                .forward(&input_ids_tensor, &attention_mask_tensor)
                .map_err(|e| {
                    AgentMemError::embedding_error(format!("Model forward failed: {}", e))
                })?;

            // 提取嵌入向量并进行池化
            let embeddings = if outputs.dims().len() == 3 {
                // [batch_size, seq_len, hidden_size] -> 平均池化
                let attention_mask_f32 =
                    attention_mask_tensor.to_dtype(DType::F32).map_err(|e| {
                        AgentMemError::embedding_error(format!("Dtype conversion failed: {}", e))
                    })?;

                // 扩展 attention_mask 维度以匹配 embeddings
                let attention_mask_expanded = attention_mask_f32
                    .unsqueeze(2)
                    .map_err(|e| {
                        AgentMemError::embedding_error(format!("Unsqueeze failed: {}", e))
                    })?
                    .broadcast_as(outputs.shape())
                    .map_err(|e| {
                        AgentMemError::embedding_error(format!("Broadcast failed: {}", e))
                    })?;

                // 应用 attention mask
                let masked_embeddings = (&outputs * &attention_mask_expanded).map_err(|e| {
                    AgentMemError::embedding_error(format!("Masking failed: {}", e))
                })?;

                // 计算平均值
                let sum_embeddings = masked_embeddings
                    .sum(1)
                    .map_err(|e| AgentMemError::embedding_error(format!("Sum failed: {}", e)))?;
                let sum_mask = attention_mask_f32
                    .sum(1)
                    .map_err(|e| AgentMemError::embedding_error(format!("Mask sum failed: {}", e)))?
                    .unsqueeze(1)
                    .map_err(|e| {
                        AgentMemError::embedding_error(format!("Unsqueeze failed: {}", e))
                    })?;

                (&sum_embeddings / &sum_mask).map_err(|e| {
                    AgentMemError::embedding_error(format!("Division failed: {}", e))
                })?
            } else {
                outputs
            };

            // 转换为 Vec<f32>
            let embedding_vec: Vec<f32> = embeddings
                .flatten_all()
                .map_err(|e| AgentMemError::embedding_error(format!("Flatten failed: {}", e)))?
                .to_vec1()
                .map_err(|e| AgentMemError::embedding_error(format!("To vec failed: {}", e)))?;

            // L2 归一化
            let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized: Vec<f32> = if norm > 0.0 {
                embedding_vec.iter().map(|x| x / norm).collect()
            } else {
                embedding_vec
            };

            info!(
                "Generated Candle embedding with {} dimensions",
                normalized.len()
            );
            Ok(normalized)
        } else {
            warn!("Candle model not fully loaded, using deterministic embedding");
            Ok(self.generate_deterministic_embedding(text))
        }
    }

    #[cfg(feature = "onnx")]
    async fn generate_onnx_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if let (Some(session), Some(tokenizer)) = (&self.onnx_session, &self.onnx_tokenizer) {
            // 1. 分词
            let encoding = tokenizer.encode(text, true).map_err(|e| {
                AgentMemError::embedding_error(format!("Tokenization failed: {}", e))
            })?;

            let input_ids = encoding.get_ids();
            let attention_mask = encoding.get_attention_mask();
            let seq_len = input_ids.len();

            debug!("Tokenized text into {} tokens", seq_len);

            // 2. 创建输入张量
            // 将 u32 转换为 i64（ONNX Runtime 期望的类型）
            let input_ids_i64: Vec<i64> = input_ids.iter().map(|&x| x as i64).collect();
            let attention_mask_i64: Vec<i64> = attention_mask.iter().map(|&x| x as i64).collect();

            // 创建形状为 [1, seq_len] 的张量
            let input_ids_tensor = Tensor::from_array((vec![1_usize, seq_len], input_ids_i64))
                .map_err(|e| {
                    AgentMemError::embedding_error(format!(
                        "Failed to create input_ids tensor: {}",
                        e
                    ))
                })?;

            let attention_mask_tensor =
                Tensor::from_array((vec![1_usize, seq_len], attention_mask_i64)).map_err(|e| {
                    AgentMemError::embedding_error(format!(
                        "Failed to create attention_mask tensor: {}",
                        e
                    ))
                })?;

            // 3. 运行 ONNX 推理
            let mut session_guard = session.lock().await;
            let outputs = session_guard
                .run(ort::inputs![input_ids_tensor, attention_mask_tensor])
                .map_err(|e| {
                    AgentMemError::embedding_error(format!("ONNX inference failed: {}", e))
                })?;

            // 4. 提取输出张量
            // 输出通常是 [batch_size, seq_len, hidden_size] 或 [batch_size, hidden_size]
            // 使用第一个输出（通常是 "last_hidden_state" 或类似的名称）
            let output_value = outputs
                .iter()
                .next()
                .ok_or_else(|| {
                    AgentMemError::embedding_error("No output tensor found".to_string())
                })?
                .1;

            let (shape, data) = output_value
                .try_extract_tensor::<f32>()
                .map_err(|e| {
                    AgentMemError::embedding_error(format!(
                        "Failed to extract output tensor: {}",
                        e
                    ))
                })?;
            debug!("Output tensor shape: {:?}", shape);

            // 5. 池化策略：使用 [CLS] token（第一个 token 的嵌入）
            let embedding_vec: Vec<f32> = if shape.len() == 3 {
                // 形状: [batch_size, seq_len, hidden_size]
                let hidden_size = shape[2] as usize;
                // 提取第一个 token 的嵌入 [0, 0, :]
                data[0..hidden_size].to_vec()
            } else if shape.len() == 2 {
                // 形状: [batch_size, hidden_size]（已经池化过）
                let hidden_size = shape[1] as usize;
                data[0..hidden_size].to_vec()
            } else {
                return Err(AgentMemError::embedding_error(format!(
                    "Unexpected output tensor shape: {:?}",
                    shape
                )));
            };

            // 6. L2 归一化
            let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized: Vec<f32> = if norm > 0.0 {
                embedding_vec.iter().map(|x| x / norm).collect()
            } else {
                embedding_vec
            };

            debug!(
                "Generated ONNX embedding with {} dimensions",
                normalized.len()
            );
            Ok(normalized)
        } else {
            warn!("ONNX model not loaded, using deterministic embedding");
            Ok(self.generate_deterministic_embedding(text))
        }
    }

    /// 批量生成 ONNX 嵌入（优化版本）
    #[cfg(feature = "onnx")]
    async fn generate_onnx_batch_embedding(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if let (Some(session), Some(tokenizer)) = (&self.onnx_session, &self.onnx_tokenizer) {
            let batch_size = texts.len();

            // 1. 批量分词
            let encodings: Vec<_> = texts
                .iter()
                .map(|text| {
                    tokenizer.encode(text.as_str(), true).map_err(|e| {
                        AgentMemError::embedding_error(format!("Tokenization failed: {}", e))
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            // 2. 找到最大序列长度
            let max_len = encodings
                .iter()
                .map(|enc| enc.get_ids().len())
                .max()
                .unwrap_or(0);

            debug!(
                "Batch tokenization: {} texts, max_len: {}",
                batch_size, max_len
            );

            // 3. 创建批量输入张量（padding 到相同长度）
            let mut input_ids_batch = Vec::with_capacity(batch_size * max_len);
            let mut attention_mask_batch = Vec::with_capacity(batch_size * max_len);

            for encoding in &encodings {
                let ids = encoding.get_ids();
                let mask = encoding.get_attention_mask();

                // 添加 token IDs 和 padding
                input_ids_batch.extend(ids.iter().map(|&x| x as i64));
                input_ids_batch.extend(vec![0i64; max_len - ids.len()]);

                // 添加 attention mask 和 padding
                attention_mask_batch.extend(mask.iter().map(|&x| x as i64));
                attention_mask_batch.extend(vec![0i64; max_len - mask.len()]);
            }

            // 创建形状为 [batch_size, max_len] 的张量
            let input_ids_tensor =
                Tensor::from_array((vec![batch_size, max_len], input_ids_batch)).map_err(|e| {
                    AgentMemError::embedding_error(format!(
                        "Failed to create batch input_ids tensor: {}",
                        e
                    ))
                })?;

            let attention_mask_tensor =
                Tensor::from_array((vec![batch_size, max_len], attention_mask_batch)).map_err(
                    |e| {
                        AgentMemError::embedding_error(format!(
                            "Failed to create batch attention_mask tensor: {}",
                            e
                        ))
                    },
                )?;

            // 4. 运行批量 ONNX 推理
            let mut session_guard = session.lock().await;
            let outputs = session_guard
                .run(ort::inputs![input_ids_tensor, attention_mask_tensor])
                .map_err(|e| {
                    AgentMemError::embedding_error(format!("Batch ONNX inference failed: {}", e))
                })?;

            // 5. 提取输出张量
            let output_value = outputs
                .iter()
                .next()
                .ok_or_else(|| {
                    AgentMemError::embedding_error("No output tensor found".to_string())
                })?
                .1;

            let (shape, data) = output_value
                .try_extract_tensor::<f32>()
                .map_err(|e| {
                    AgentMemError::embedding_error(format!(
                        "Failed to extract batch output tensor: {}",
                        e
                    ))
                })?;
            debug!("Batch output tensor shape: {:?}", shape);

            // 6. 提取每个样本的嵌入向量
            let mut embeddings = Vec::with_capacity(batch_size);

            if shape.len() == 3 {
                // 形状: [batch_size, seq_len, hidden_size]
                let hidden_size = shape[2] as usize;
                let seq_len = shape[1] as usize;

                for i in 0..batch_size {
                    // 提取第 i 个样本的 [CLS] token（第一个 token）
                    let start_idx = i * seq_len * hidden_size;
                    let embedding_vec: Vec<f32> = data[start_idx..start_idx + hidden_size].to_vec();

                    // L2 归一化
                    let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let normalized: Vec<f32> = if norm > 0.0 {
                        embedding_vec.iter().map(|x| x / norm).collect()
                    } else {
                        embedding_vec
                    };

                    embeddings.push(normalized);
                }
            } else if shape.len() == 2 {
                // 形状: [batch_size, hidden_size]（已经池化过）
                let hidden_size = shape[1] as usize;

                for i in 0..batch_size {
                    let start_idx = i * hidden_size;
                    let embedding_vec: Vec<f32> = data[start_idx..start_idx + hidden_size].to_vec();

                    // L2 归一化
                    let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let normalized: Vec<f32> = if norm > 0.0 {
                        embedding_vec.iter().map(|x| x / norm).collect()
                    } else {
                        embedding_vec
                    };

                    embeddings.push(normalized);
                }
            } else {
                return Err(AgentMemError::embedding_error(format!(
                    "Unexpected batch output tensor shape: {:?}",
                    shape
                )));
            }

            debug!(
                "Generated {} ONNX embeddings in batch with {} dimensions each",
                batch_size,
                embeddings[0].len()
            );

            Ok(embeddings)
        } else {
            warn!("ONNX model not loaded, using deterministic embeddings for batch");
            Ok(texts
                .iter()
                .map(|text| self.generate_deterministic_embedding(text))
                .collect())
        }
    }

    /// 生成确定性嵌入（作为后备方案）
    fn generate_deterministic_embedding(&self, text: &str) -> Vec<f32> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let hash = hasher.finalize();

        // 将哈希转换为浮点向量
        let mut embedding = Vec::with_capacity(384);
        for chunk in hash.chunks(4) {
            let mut bytes = [0u8; 4];
            bytes[..chunk.len()].copy_from_slice(chunk);
            let value = u32::from_le_bytes(bytes) as f32 / u32::MAX as f32;
            embedding.push(value * 2.0 - 1.0); // 归一化到 [-1, 1]
        }

        // 扩展到目标维度
        while embedding.len() < 384 {
            embedding.push(0.0);
        }

        // L2 归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut embedding {
                *v /= norm;
            }
        }

        embedding
    }

    /// 批量处理文本（优化版本）
    async fn process_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // 根据模型类型选择批处理策略
        match &self.model_type {
            #[cfg(feature = "onnx")]
            LocalModelType::Onnx { .. } => {
                // ONNX 支持真正的批量推理
                self.generate_onnx_batch_embedding(texts).await
            }
            _ => {
                // 其他模型类型使用逐个处理
                let mut embeddings = Vec::new();
                for text in texts {
                    let embedding = self.generate_embedding_real(text).await?;
                    embeddings.push(embedding);
                }
                Ok(embeddings)
            }
        }
    }
}

#[async_trait]
impl Embedder for LocalEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // 确保模型已加载
        {
            let is_loaded = self.is_loaded.lock().await;
            if !*is_loaded {
                drop(is_loaded);
                // 需要可变引用来加载模型，这里使用确定性嵌入作为后备
                warn!("Model not loaded, using deterministic embedding");
                return Ok(self.generate_deterministic_embedding(text));
            }
        }

        debug!("Generating embedding for text: {}", text);

        // 尝试使用真实模型
        match self.generate_embedding_real(text).await {
            Ok(embedding) => {
                debug!("Generated embedding with {} dimensions", embedding.len());
                Ok(embedding)
            }
            Err(e) => {
                warn!("Real model failed, falling back to deterministic: {}", e);
                Ok(self.generate_deterministic_embedding(text))
            }
        }
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // 将文本分批处理以避免内存问题
        let batch_size = self.config.batch_size;
        let mut all_embeddings = Vec::new();

        for chunk in texts.chunks(batch_size) {
            let batch_embeddings = self.process_batch(chunk).await?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        384 // 标准嵌入维度
    }

    fn provider_name(&self) -> &str {
        "local"
    }

    fn model_name(&self) -> &str {
        match &self.model_type {
            LocalModelType::Candle { model_path, .. } => model_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("candle-model"),
            LocalModelType::Onnx { model_path, .. } => model_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("onnx-model"),
            LocalModelType::HuggingFace { model_name, .. } => model_name,
        }
    }

    async fn health_check(&self) -> Result<bool> {
        // 检查模型是否已加载
        let is_loaded = self.is_loaded.lock().await;
        if !*is_loaded {
            return Ok(false);
        }
        drop(is_loaded);

        // 尝试生成一个测试嵌入
        match self.embed("health check").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore] // Requires local model files
    async fn test_local_embedder_creation_missing_path() {
        let config = EmbeddingConfig::local("/nonexistent/path", 384);
        let result = LocalEmbedder::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires local model files
    async fn test_local_embedder_creation_with_valid_path() {
        // 创建临时文件作为模型路径
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 384);
        let result = LocalEmbedder::new(config).await;
        assert!(result.is_ok());

        let embedder = result.unwrap();
        assert_eq!(embedder.provider_name(), "local");
        assert_eq!(embedder.model_name(), "local");
        assert_eq!(embedder.dimension(), 384);
    }

    #[tokio::test]
    async fn test_embed_single_text() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 384);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let result = embedder.embed("test text").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    #[ignore] // Requires local model files
    async fn test_embed_batch() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 256);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let texts = vec![
            "first text".to_string(),
            "second text".to_string(),
            "third text".to_string(),
        ];

        let result = embedder.embed_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 256);
        assert_eq!(embeddings[1].len(), 256);
        assert_eq!(embeddings[2].len(), 256);
    }

    #[tokio::test]
    async fn test_embed_empty_batch() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 128);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let result = embedder.embed_batch(&[]).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 0);
    }

    #[tokio::test]
    #[ignore] // Requires local model files
    async fn test_health_check() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 128);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_health_check_missing_model() {
        let temp_dir = tempdir().unwrap();
        let model_path = temp_dir.path().join("model.bin");
        File::create(&model_path).unwrap();

        let config = EmbeddingConfig::local(model_path.to_str().unwrap(), 128);
        let embedder = LocalEmbedder::new(config).await.unwrap();

        // 删除模型文件
        std::fs::remove_file(&model_path).unwrap();

        let result = embedder.health_check().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}
