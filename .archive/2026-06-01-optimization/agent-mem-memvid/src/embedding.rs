//! Embedding 生成和向量搜索支持
//!
//! 此模块提供文本嵌入（embedding）的生成接口和实现，
//! 用于支持语义搜索和向量相似度查询。

use crate::error::{MemvidError, Result};
use serde::{Deserialize, Serialize};

/// 文本嵌入向量
pub type EmbeddingVector = Vec<f32>;

/// OpenAI API 嵌入生成器
#[derive(Debug, Clone)]
pub struct OpenAIEmbedding {
    api_key: String,
    model: String,
    dimension: usize,
    client: reqwest::Client,
}

impl OpenAIEmbedding {
    /// 创建新的 OpenAI 嵌入生成器
    pub fn new(api_key: String, model: String) -> Self {
        let dimension = match model.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536,
        };

        Self {
            api_key,
            model,
            dimension,
            client: reqwest::Client::new(),
        }
    }
}

impl crate::vector_search::EmbeddingGenerator for OpenAIEmbedding {
    fn embed_sync(&self, text: &str) -> Result<EmbeddingVector> {
        // 在同步上下文中执行异步 HTTP 请求
        // 注意：这会阻塞当前线程，应该在 spawn_blocking 中使用
        use reqwest::header;
        use std::io;

        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            MemvidError::Io(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create runtime: {}", e),
            ))
        })?;

        runtime.block_on(async {
            let response = self
                .client
                .post("https://api.openai.com/v1/embeddings")
                .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
                .header(header::CONTENT_TYPE, "application/json")
                .json(&serde_json::json!({
                    "input": text,
                    "model": self.model
                }))
                .send()
                .await
                .map_err(|e| {
                    MemvidError::Io(io::Error::new(
                        io::ErrorKind::Other,
                        format!("OpenAI API error: {}", e),
                    ))
                })?;

            if !response.status().is_success() {
                return Err(MemvidError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!("OpenAI API returned status: {}", response.status()),
                )));
            }

            let json: serde_json::Value = response.json().await.map_err(|e| {
                MemvidError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to parse JSON: {}", e),
                ))
            })?;

            let embedding = json["data"][0]["embedding"].as_array().ok_or_else(|| {
                MemvidError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid embedding format",
                ))
            })?;

            let vector: EmbeddingVector = embedding
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();

            Ok(vector)
        })
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// 本地简单嵌入生成器（基于 TF-IDF 的简化实现）
#[derive(Debug, Clone)]
pub struct LocalEmbedding {
    dimension: usize,
}

impl LocalEmbedding {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl crate::vector_search::EmbeddingGenerator for LocalEmbedding {
    fn embed_sync(&self, text: &str) -> Result<EmbeddingVector> {
        // 简化的基于词频的嵌入
        // 实际应用中应使用专业模型
        let mut vector = vec![0.0f32; self.dimension];

        // 简单的哈希-based 嵌入
        let bytes = text.as_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            let idx = (i as usize + byte as usize) % self.dimension;
            vector[idx] += byte as f32 / 255.0;
        }

        // 归一化
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }

        Ok(vector)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "local-tfidf"
    }
}

/// 余弦相似度计算
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// 欧几里得距离计算
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum::<f32>()
        .sqrt()
}

/// 向量相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    /// 记忆 ID
    pub memory_id: String,
    /// 相似度分数 (0-1)
    pub score: f32,
    /// 相似度类型
    pub similarity_type: SimilarityType,
}

/// 相似度类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SimilarityType {
    /// 余弦相似度
    Cosine,
    /// 欧几里得距离
    Euclidean,
    /// 点积
    DotProduct,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_search::EmbeddingGenerator;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![2.0, 4.0, 6.0]; // 平行向量，相似度应为1

        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);

        // 正交向量
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let sim_orth = cosine_similarity(&c, &d);
        assert!((sim_orth - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_local_embedding() {
        let embedding = LocalEmbedding::new(128);

        // 创建简单测试（需要异步运行时）
        // 实际测试需要 tokio runtime
        assert_eq!(embedding.dimension(), 128);
    }
}
