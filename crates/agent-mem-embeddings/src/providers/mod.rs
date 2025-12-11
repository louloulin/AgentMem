//! 嵌入提供商实现模块

pub mod cohere;
pub mod huggingface;
pub mod local;
pub mod openai;

#[cfg(feature = "fastembed")]
pub mod fastembed;

// 嵌入队列和队列化包装器
pub mod embedding_queue;
pub mod queued_embedder;

// Local tests require local or onnx features
#[cfg(all(test, any(feature = "local", feature = "onnx")))]
mod local_test;

pub use cohere::CohereEmbedder;
pub use huggingface::HuggingFaceEmbedder;
pub use local::LocalEmbedder;
pub use openai::OpenAIEmbedder;

#[cfg(feature = "fastembed")]
pub use fastembed::FastEmbedProvider;

pub use embedding_queue::EmbeddingQueue;
pub use queued_embedder::QueuedEmbedder;
