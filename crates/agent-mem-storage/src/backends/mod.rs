//! 存储后端实现模块

// 嵌入式存储
pub mod libsql_store;
pub mod lancedb_store;

pub mod azure_ai_search;
#[cfg(test)]
mod azure_ai_search_test;
pub mod chroma;
pub mod elasticsearch;
pub mod faiss;
#[cfg(test)]
mod faiss_test;
pub mod lancedb;
pub mod memory;
pub mod milvus;
pub mod mongodb;
#[cfg(test)]
mod mongodb_test;
pub mod pinecone;
pub mod qdrant;
pub mod redis;
#[cfg(test)]
mod redis_test;
pub mod supabase;
#[cfg(test)]
mod supabase_test;
pub mod weaviate;

// 嵌入式存储导出
pub use libsql_store::LibSQLStore;
pub use lancedb_store::LanceDBStore as LanceDBVectorStore;

pub use azure_ai_search::AzureAISearchStore;
pub use chroma::ChromaStore;
pub use elasticsearch::ElasticsearchStore;
pub use faiss::FaissStore;
pub use lancedb::LanceDBStore;
pub use memory::MemoryVectorStore;
pub use milvus::MilvusStore;
pub use mongodb::MongoDBStore;
pub use pinecone::PineconeStore;
pub use qdrant::QdrantStore;
pub use redis::RedisStore;
pub use supabase::SupabaseStore;
pub use weaviate::WeaviateStore;
