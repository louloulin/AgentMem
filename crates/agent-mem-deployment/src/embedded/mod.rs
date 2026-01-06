//! 嵌入式存储模块
//!
//! 提供嵌入式数据库和向量存储支持，用于单二进制部署

pub mod config;
pub mod database;
pub mod vector_store;

pub use config::EmbeddedConfig;
pub use database::EmbeddedDatabase;
pub use vector_store::EmbeddedVectorStore;
