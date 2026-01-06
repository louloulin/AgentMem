//! Embedder trait definitions

use crate::Result;
use async_trait::async_trait;

/// Core trait for embedding providers
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Generate embeddings for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get the dimension of embeddings produced by this embedder
    fn dimension(&self) -> usize;

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get the model name being used
    fn model_name(&self) -> &str;

    /// Check if the embedder is available/healthy
    async fn health_check(&self) -> Result<bool>;
}
