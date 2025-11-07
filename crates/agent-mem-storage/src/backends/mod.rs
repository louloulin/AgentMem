//! 存储后端实现模块

// Memory store implementations (trait-based)
pub mod libsql_core;
pub mod libsql_episodic;
pub mod libsql_procedural;
pub mod libsql_semantic;
pub mod libsql_working;
#[cfg(feature = "postgres")]
pub mod postgres_core;
#[cfg(feature = "postgres")]
pub mod postgres_episodic;
#[cfg(feature = "postgres")]
pub mod postgres_procedural;
#[cfg(feature = "postgres")]
pub mod postgres_semantic;
#[cfg(feature = "postgres")]
pub mod postgres_vector;
#[cfg(feature = "postgres")]
pub mod postgres_working;

// 嵌入式存储
#[cfg(feature = "lancedb")]
pub mod lancedb_store;
pub mod libsql_store;

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

// Memory store exports (trait-based)
pub use libsql_core::LibSqlCoreStore;
pub use libsql_episodic::LibSqlEpisodicStore;
pub use libsql_procedural::LibSqlProceduralStore;
pub use libsql_semantic::LibSqlSemanticStore;
pub use libsql_working::LibSqlWorkingStore;
#[cfg(feature = "postgres")]
pub use postgres_core::PostgresCoreStore;
#[cfg(feature = "postgres")]
pub use postgres_episodic::PostgresEpisodicStore;
#[cfg(feature = "postgres")]
pub use postgres_procedural::PostgresProceduralStore;
#[cfg(feature = "postgres")]
pub use postgres_semantic::PostgresSemanticStore;
#[cfg(feature = "postgres")]
pub use postgres_vector::{PostgresVectorConfig, PostgresVectorStore, VectorDistanceOperator};
#[cfg(feature = "postgres")]
pub use postgres_working::PostgresWorkingStore;

// 嵌入式存储导出
#[cfg(feature = "lancedb")]
pub use lancedb_store::LanceDBStore as LanceDBVectorStore;
pub use libsql_store::LibSQLStore;

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
