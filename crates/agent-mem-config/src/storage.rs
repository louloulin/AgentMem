//! Storage configuration extensions

use serde::{Deserialize, Serialize};
use agent_mem_traits::VectorStoreConfig as BaseVectorStoreConfig;
use std::path::{Path, PathBuf};

/// Extended storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct StorageConfig {
    /// Vector store configuration
    pub vector_store: VectorStoreConfig,
    
    /// Graph store configuration (optional)
    pub graph_store: Option<GraphStoreConfig>,
    
    /// Key-value store configuration
    pub kv_store: KeyValueStoreConfig,
    
    /// History store configuration
    pub history_store: HistoryStoreConfig,
}


/// Extended vector store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    #[serde(flatten)]
    pub base: BaseVectorStoreConfig,
    
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    
    /// Maximum connections in pool
    pub max_connections: u32,
    
    /// Enable connection pooling
    pub enable_pooling: bool,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            base: BaseVectorStoreConfig::default(),
            timeout_seconds: 30,
            max_connections: 10,
            enable_pooling: true,
            batch_size: 100,
        }
    }
}

impl VectorStoreConfig {
    pub fn lancedb() -> Self {
        Self {
            base: BaseVectorStoreConfig {
                provider: "lancedb".to_string(),
                path: "./data/vectors".to_string(),
                table_name: "memories".to_string(),
                dimension: Some(1536),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn pinecone(index_name: &str) -> Self {
        Self {
            base: BaseVectorStoreConfig {
                provider: "pinecone".to_string(),
                index_name: Some(index_name.to_string()),
                dimension: Some(1536),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn qdrant() -> Self {
        Self {
            base: BaseVectorStoreConfig {
                provider: "qdrant".to_string(),
                path: "http://localhost:6333".to_string(),
                table_name: "memories".to_string(),
                dimension: Some(1536),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Graph store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStoreConfig {
    pub provider: String,
    pub uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub timeout_seconds: u64,
    pub max_connections: u32,
}

impl Default for GraphStoreConfig {
    fn default() -> Self {
        Self {
            provider: "neo4j".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: None,
            database: Some("neo4j".to_string()),
            timeout_seconds: 30,
            max_connections: 10,
        }
    }
}

/// Key-value store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueStoreConfig {
    pub provider: String,
    pub uri: String,
    pub password: Option<String>,
    pub database: u32,
    pub timeout_seconds: u64,
    pub max_connections: u32,
}

impl Default for KeyValueStoreConfig {
    fn default() -> Self {
        Self {
            provider: "redis".to_string(),
            uri: "redis://localhost:6379".to_string(),
            password: None,
            database: 0,
            timeout_seconds: 30,
            max_connections: 10,
        }
    }
}

/// History store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStoreConfig {
    pub provider: String,
    pub path: String,
    pub max_entries_per_memory: usize,
    pub cleanup_interval_hours: u64,
}

impl Default for HistoryStoreConfig {
    fn default() -> Self {
        Self {
            provider: "sqlite".to_string(),
            path: "./data/history.db".to_string(),
            max_entries_per_memory: 100,
            cleanup_interval_hours: 24,
        }
    }
}

// ============================================================================
// 部署模式配置（任务 3.1）
// ============================================================================

/// 部署模式
///
/// AgentMem 支持两种部署模式：
/// - **嵌入式模式**: 使用 LibSQL + LanceDB，适合单机部署和开发环境
/// - **服务器模式**: 使用 PostgreSQL + 多种向量服务，适合生产环境和分布式部署
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum DeploymentMode {
    /// 嵌入式模式（LibSQL + LanceDB）
    ///
    /// 特点：
    /// - 零配置，开箱即用
    /// - 单文件数据库
    /// - 本地向量存储
    /// - 适合开发和小规模部署
    Embedded(EmbeddedModeConfig),

    /// 服务器模式（PostgreSQL/LibSQL + 多向量服务）
    ///
    /// 特点：
    /// - 支持 13+ 向量服务
    /// - 高可用性
    /// - 水平扩展
    /// - 适合生产环境
    Server(ServerModeConfig),
}

/// 向量服务类型
///
/// 支持 14 种向量服务，涵盖云托管、自托管、嵌入式等多种场景
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorServiceType {
    /// PostgreSQL + pgvector（统一存储）
    ///
    /// 特点：
    /// - 数据和向量统一存储
    /// - 支持 ACID 事务
    /// - 成熟稳定
    /// - 适合中小规模
    #[serde(rename = "pgvector")]
    PgVector,

    /// LanceDB（嵌入式或服务器）
    ///
    /// 特点：
    /// - 基于 Apache Arrow
    /// - 高性能列式存储
    /// - 支持嵌入式和服务器模式
    /// - 适合大规模数据
    #[serde(rename = "lancedb")]
    LanceDB,

    /// Pinecone（云托管）
    ///
    /// 特点：
    /// - 完全托管
    /// - 自动扩展
    /// - 低延迟
    /// - 适合生产环境
    #[serde(rename = "pinecone")]
    Pinecone,

    /// Qdrant（专业向量库）
    ///
    /// 特点：
    /// - 高性能
    /// - 丰富的过滤功能
    /// - 支持云和自托管
    /// - 适合复杂查询
    #[serde(rename = "qdrant")]
    Qdrant,

    /// Milvus（超大规模）
    ///
    /// 特点：
    /// - 分布式架构
    /// - 支持十亿级向量
    /// - 多种索引类型
    /// - 适合超大规模
    #[serde(rename = "milvus")]
    Milvus,

    /// Weaviate（多模态）
    ///
    /// 特点：
    /// - 支持多模态数据
    /// - 内置 ML 模型
    /// - GraphQL API
    /// - 适合知识图谱
    #[serde(rename = "weaviate")]
    Weaviate,

    /// Chroma（开源）
    ///
    /// 特点：
    /// - 开源免费
    /// - 简单易用
    /// - Python 友好
    /// - 适合快速原型
    #[serde(rename = "chroma")]
    Chroma,

    /// Elasticsearch（混合搜索）
    ///
    /// 特点：
    /// - 全文 + 向量搜索
    /// - 成熟生态
    /// - 强大的分析功能
    /// - 适合混合搜索
    #[serde(rename = "elasticsearch")]
    Elasticsearch,

    /// Redis（实时）
    ///
    /// 特点：
    /// - 极低延迟
    /// - 内存存储
    /// - 丰富的数据结构
    /// - 适合实时场景
    #[serde(rename = "redis")]
    Redis,

    /// MongoDB（文档）
    ///
    /// 特点：
    /// - 文档数据库
    /// - 灵活的 Schema
    /// - 向量搜索支持
    /// - 适合文档存储
    #[serde(rename = "mongodb")]
    MongoDB,

    /// Supabase（云）
    ///
    /// 特点：
    /// - PostgreSQL + pgvector
    /// - 完全托管
    /// - 实时订阅
    /// - 适合全栈应用
    #[serde(rename = "supabase")]
    Supabase,

    /// FAISS（本地）
    ///
    /// 特点：
    /// - Facebook 开源
    /// - 高性能
    /// - 多种索引算法
    /// - 适合本地部署
    #[serde(rename = "faiss")]
    FAISS,

    /// Azure AI Search（企业）
    ///
    /// 特点：
    /// - 微软云服务
    /// - 企业级安全
    /// - 多语言支持
    /// - 适合企业应用
    #[serde(rename = "azure_ai_search")]
    AzureAISearch,

    /// Memory（测试）
    ///
    /// 特点：
    /// - 纯内存存储
    /// - 无持久化
    /// - 快速启动
    /// - 仅用于测试
    #[serde(rename = "memory")]
    Memory,
}

impl VectorServiceType {
    /// 获取服务名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::PgVector => "pgvector",
            Self::LanceDB => "lancedb",
            Self::Pinecone => "pinecone",
            Self::Qdrant => "qdrant",
            Self::Milvus => "milvus",
            Self::Weaviate => "weaviate",
            Self::Chroma => "chroma",
            Self::Elasticsearch => "elasticsearch",
            Self::Redis => "redis",
            Self::MongoDB => "mongodb",
            Self::Supabase => "supabase",
            Self::FAISS => "faiss",
            Self::AzureAISearch => "azure_ai_search",
            Self::Memory => "memory",
        }
    }

    /// 是否为云托管服务
    pub fn is_cloud_hosted(&self) -> bool {
        matches!(
            self,
            Self::Pinecone | Self::Supabase | Self::AzureAISearch
        )
    }

    /// 是否支持嵌入式部署
    pub fn supports_embedded(&self) -> bool {
        matches!(self, Self::LanceDB | Self::FAISS | Self::Memory)
    }
}

/// 嵌入式模式配置
///
/// 使用 LibSQL + LanceDB 的嵌入式部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedModeConfig {
    /// LibSQL 数据库路径
    ///
    /// 默认: `./data/agentmem.db`
    pub database_path: PathBuf,

    /// LanceDB 向量存储路径
    ///
    /// 默认: `./data/vectors`
    pub vector_path: PathBuf,

    /// 向量维度
    ///
    /// 默认: 1536 (OpenAI text-embedding-ada-002)
    pub vector_dimension: usize,

    /// 是否启用 WAL (Write-Ahead Logging)
    ///
    /// 默认: true
    pub enable_wal: bool,

    /// 缓存大小（KB）
    ///
    /// 默认: 10240 (10 MB)
    pub cache_size_kb: usize,
}

impl Default for EmbeddedModeConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("./data/agentmem.db"),
            vector_path: PathBuf::from("./data/vectors"),
            vector_dimension: 1536,
            enable_wal: true,
            cache_size_kb: 10240,
        }
    }
}

/// 服务器模式配置
///
/// 使用 PostgreSQL/LibSQL + 多种向量服务的服务器部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerModeConfig {
    /// PostgreSQL 连接 URL
    ///
    /// 格式: `postgresql://user:password@host:port/database`
    pub database_url: String,

    /// 连接池配置
    pub pool_config: PoolConfig,

    /// 向量服务类型
    pub vector_service: VectorServiceType,

    /// 向量服务配置
    pub vector_config: VectorStoreConfig,

    /// 向量维度
    ///
    /// 默认: 1536 (OpenAI text-embedding-ada-002)
    pub vector_dimension: usize,
}

impl Default for ServerModeConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/agentmem".to_string(),
            pool_config: PoolConfig::default(),
            vector_service: VectorServiceType::PgVector,
            vector_config: VectorStoreConfig::default(),
            vector_dimension: 1536,
        }
    }
}

/// 连接池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// 最小连接数
    pub min_connections: u32,

    /// 最大连接数
    pub max_connections: u32,

    /// 连接超时（秒）
    pub connect_timeout_seconds: u64,

    /// 空闲超时（秒）
    pub idle_timeout_seconds: u64,

    /// 最大生命周期（秒）
    pub max_lifetime_seconds: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 1800,
        }
    }
}

impl DeploymentMode {
    /// 创建嵌入式模式配置
    ///
    /// # 参数
    ///
    /// * `data_root` - 数据根目录路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::DeploymentMode;
    ///
    /// let mode = DeploymentMode::embedded("./data");
    /// ```
    pub fn embedded<P: AsRef<Path>>(data_root: P) -> Self {
        let data_root = data_root.as_ref();
        Self::Embedded(EmbeddedModeConfig {
            database_path: data_root.join("agentmem.db"),
            vector_path: data_root.join("vectors"),
            vector_dimension: 1536,
            enable_wal: true,
            cache_size_kb: 10240,
        })
    }

    /// 创建服务器模式配置（PostgreSQL + pgvector）
    ///
    /// # 参数
    ///
    /// * `database_url` - PostgreSQL 连接 URL
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::DeploymentMode;
    ///
    /// let mode = DeploymentMode::server_with_pgvector(
    ///     "postgresql://localhost:5432/agentmem".to_string()
    /// );
    /// ```
    pub fn server_with_pgvector(database_url: String) -> Self {
        Self::Server(ServerModeConfig {
            database_url: database_url.clone(),
            pool_config: PoolConfig::default(),
            vector_service: VectorServiceType::PgVector,
            vector_config: VectorStoreConfig {
                base: BaseVectorStoreConfig {
                    provider: "pgvector".to_string(),
                    url: Some(database_url),
                    ..Default::default()
                },
                ..Default::default()
            },
            vector_dimension: 1536,
        })
    }

    /// 创建服务器模式配置（PostgreSQL + LanceDB）
    ///
    /// # 参数
    ///
    /// * `database_url` - PostgreSQL 连接 URL
    /// * `vector_path` - LanceDB 向量存储路径
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::DeploymentMode;
    ///
    /// let mode = DeploymentMode::server_with_lancedb(
    ///     "postgresql://localhost:5432/agentmem".to_string(),
    ///     "./data/vectors".to_string(),
    /// );
    /// ```
    pub fn server_with_lancedb(database_url: String, vector_path: String) -> Self {
        Self::Server(ServerModeConfig {
            database_url,
            pool_config: PoolConfig::default(),
            vector_service: VectorServiceType::LanceDB,
            vector_config: VectorStoreConfig {
                base: BaseVectorStoreConfig {
                    provider: "lancedb".to_string(),
                    url: Some(vector_path),
                    ..Default::default()
                },
                ..Default::default()
            },
            vector_dimension: 1536,
        })
    }

    /// 创建服务器模式配置（PostgreSQL + Qdrant）
    ///
    /// # 参数
    ///
    /// * `database_url` - PostgreSQL 连接 URL
    /// * `qdrant_url` - Qdrant 服务 URL
    /// * `collection` - Qdrant 集合名称
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::DeploymentMode;
    ///
    /// let mode = DeploymentMode::server_with_qdrant(
    ///     "postgresql://localhost:5432/agentmem".to_string(),
    ///     "http://localhost:6333".to_string(),
    ///     "memories".to_string(),
    /// );
    /// ```
    pub fn server_with_qdrant(database_url: String, qdrant_url: String, collection: String) -> Self {
        Self::Server(ServerModeConfig {
            database_url,
            pool_config: PoolConfig::default(),
            vector_service: VectorServiceType::Qdrant,
            vector_config: VectorStoreConfig {
                base: BaseVectorStoreConfig {
                    provider: "qdrant".to_string(),
                    url: Some(qdrant_url),
                    collection_name: Some(collection),
                    ..Default::default()
                },
                ..Default::default()
            },
            vector_dimension: 1536,
        })
    }

    /// 创建服务器模式配置（PostgreSQL + Pinecone）
    ///
    /// # 参数
    ///
    /// * `database_url` - PostgreSQL 连接 URL
    /// * `api_key` - Pinecone API 密钥
    /// * `index` - Pinecone 索引名称
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::DeploymentMode;
    ///
    /// let mode = DeploymentMode::server_with_pinecone(
    ///     "postgresql://localhost:5432/agentmem".to_string(),
    ///     "your-api-key".to_string(),
    ///     "memories".to_string(),
    /// );
    /// ```
    pub fn server_with_pinecone(database_url: String, api_key: String, index: String) -> Self {
        Self::Server(ServerModeConfig {
            database_url,
            pool_config: PoolConfig::default(),
            vector_service: VectorServiceType::Pinecone,
            vector_config: VectorStoreConfig {
                base: BaseVectorStoreConfig {
                    provider: "pinecone".to_string(),
                    api_key: Some(api_key),
                    index_name: Some(index),
                    ..Default::default()
                },
                ..Default::default()
            },
            vector_dimension: 1536,
        })
    }

    /// 创建服务器模式配置（PostgreSQL + Milvus）
    ///
    /// # 参数
    ///
    /// * `database_url` - PostgreSQL 连接 URL
    /// * `milvus_url` - Milvus 服务 URL
    /// * `collection` - Milvus 集合名称
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::DeploymentMode;
    ///
    /// let mode = DeploymentMode::server_with_milvus(
    ///     "postgresql://localhost:5432/agentmem".to_string(),
    ///     "localhost:19530".to_string(),
    ///     "memories".to_string(),
    /// );
    /// ```
    pub fn server_with_milvus(database_url: String, milvus_url: String, collection: String) -> Self {
        Self::Server(ServerModeConfig {
            database_url,
            pool_config: PoolConfig::default(),
            vector_service: VectorServiceType::Milvus,
            vector_config: VectorStoreConfig {
                base: BaseVectorStoreConfig {
                    provider: "milvus".to_string(),
                    url: Some(milvus_url),
                    collection_name: Some(collection),
                    ..Default::default()
                },
                ..Default::default()
            },
            vector_dimension: 1536,
        })
    }

    /// 创建服务器模式配置（通用）
    ///
    /// # 参数
    ///
    /// * `database_url` - PostgreSQL 连接 URL
    /// * `vector_service` - 向量服务类型
    /// * `vector_config` - 向量服务配置
    ///
    /// # 示例
    ///
    /// ```rust
    /// use agent_mem_config::storage::{DeploymentMode, VectorServiceType, VectorStoreConfig};
    ///
    /// let mode = DeploymentMode::server_with_vector_service(
    ///     "postgresql://localhost:5432/agentmem".to_string(),
    ///     VectorServiceType::Chroma,
    ///     VectorStoreConfig::default(),
    /// );
    /// ```
    pub fn server_with_vector_service(
        database_url: String,
        vector_service: VectorServiceType,
        vector_config: VectorStoreConfig,
    ) -> Self {
        Self::Server(ServerModeConfig {
            database_url,
            pool_config: PoolConfig::default(),
            vector_service,
            vector_config,
            vector_dimension: 1536,
        })
    }
}