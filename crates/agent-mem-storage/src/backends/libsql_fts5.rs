//! LibSQL FTS5 Full-Text Search Support
//!
//! 增强LibSQL存储，添加FTS5全文搜索功能，支持BM25算法

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

#[cfg(feature = "libsql")]
use libsql::{Builder, Connection, Database};

// Helper function to convert libsql errors to AgentMemError
#[cfg(feature = "libsql")]
fn convert_libsql_error(e: libsql::Error) -> AgentMemError {
    AgentMemError::StorageError(format!("LibSQL error: {}", e))
}

/// FTS5搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FTS5SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub agent_id: String,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// BM25参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Params {
    /// 词频饱和度参数 (通常1.2-2.0)
    pub k1: f32,
    /// 文档长度归一化参数 (通常0.75)
    pub b: f32,
}

impl Default for BM25Params {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
        }
    }
}

/// LibSQL FTS5存储
#[cfg(feature = "libsql")]
pub struct LibSQLFTS5Store {
    db: Database,
    conn: Connection,
    bm25_params: BM25Params,
}

#[cfg(feature = "libsql")]
impl LibSQLFTS5Store {
    /// 创建新的FTS5存储
    pub async fn new(path: &str) -> Result<Self> {
        Self::new_with_params(path, BM25Params::default()).await
    }
    
    /// 使用自定义BM25参数创建
    pub async fn new_with_params(path: &str, bm25_params: BM25Params) -> Result<Self> {
        info!("Initializing LibSQL FTS5 store at: {}", path);
        
        // 展开路径
        let expanded_path = if path.starts_with("~/") {
            let home = std::env::var("HOME").map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get HOME directory: {e}"))
            })?;
            path.replace("~", &home)
        } else {
            path.to_string()
        };
        
        // 创建父目录
        if expanded_path != ":memory:" {
            if let Some(parent) = Path::new(&expanded_path).parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to create directory: {e}"))
                })?;
            }
        }
        
        // 打开数据库
        let db = Builder::new_local(&expanded_path)
            .build()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to open database: {e}")))?;
        
        let conn = db.connect().map_err(|e| {
            AgentMemError::StorageError(format!("Failed to connect to database: {e}"))
        })?;
        
        let mut store = Self {
            db,
            conn,
            bm25_params,
        };
        
        // 初始化schema
        store.init_schema().await?;
        
        info!("LibSQL FTS5 store initialized successfully");
        Ok(store)
    }
    
    /// 初始化数据库schema
    async fn init_schema(&mut self) -> Result<()> {
        debug!("Initializing FTS5 database schema");
        
        // 1. 创建主表
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS memories (
                    id TEXT PRIMARY KEY,
                    agent_id TEXT NOT NULL,
                    user_id TEXT,
                    content TEXT NOT NULL,
                    memory_type TEXT NOT NULL,
                    importance REAL NOT NULL DEFAULT 0.5,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    metadata TEXT NOT NULL DEFAULT '{}',
                    is_deleted INTEGER NOT NULL DEFAULT 0
                )",
                (),
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to create memories table: {e}"))
            })?;
        
        // 2. 创建FTS5虚拟表
        self.conn
            .execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                    content,
                    content_rowid=id,
                    tokenize='unicode61 remove_diacritics 2'
                )",
                (),
            )
            .await
            .map_err(|e| {
                AgentMemError::StorageError(format!("Failed to create FTS5 table: {e}"))
            })?;
        
        // 3. 创建触发器 - 插入时同步
        self.conn
            .execute(
                "CREATE TRIGGER IF NOT EXISTS memories_ai 
                AFTER INSERT ON memories 
                BEGIN
                    INSERT INTO memories_fts(rowid, content) 
                    VALUES (NEW.rowid, NEW.content);
                END",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create insert trigger: {e}")))?;
        
        // 4. 创建触发器 - 更新时同步
        self.conn
            .execute(
                "CREATE TRIGGER IF NOT EXISTS memories_au 
                AFTER UPDATE ON memories 
                BEGIN
                    UPDATE memories_fts 
                    SET content = NEW.content 
                    WHERE rowid = NEW.rowid;
                END",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create update trigger: {e}")))?;
        
        // 5. 创建触发器 - 删除时同步
        self.conn
            .execute(
                "CREATE TRIGGER IF NOT EXISTS memories_ad 
                AFTER DELETE ON memories 
                BEGIN
                    DELETE FROM memories_fts WHERE rowid = OLD.rowid;
                END",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create delete trigger: {e}")))?;
        
        // 6. 创建索引
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
        
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
        
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_memories_created_at ON memories(created_at DESC)",
                (),
            )
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create index: {e}")))?;
        
        debug!("FTS5 database schema initialized");
        Ok(())
    }
    
    /// BM25全文搜索
    ///
    /// 使用FTS5的内置BM25函数进行搜索
    pub async fn search_bm25(
        &self,
        query: &str,
        limit: usize,
        filters: Option<&HashMap<String, String>>,
    ) -> Result<Vec<FTS5SearchResult>> {
        debug!("FTS5 BM25 search: query='{}', limit={}", query, limit);
        
        // 构建SQL查询
        let mut sql = String::from(
            "SELECT 
                m.id,
                m.content,
                bm25(memories_fts) as score,
                m.agent_id,
                m.user_id,
                m.created_at,
                m.metadata
            FROM memories_fts
            JOIN memories m ON memories_fts.rowid = m.rowid
            WHERE memories_fts MATCH ?"
        );
        
        // 添加过滤条件
        if let Some(filters) = filters {
            if let Some(agent_id) = filters.get("agent_id") {
                sql.push_str(&format!(" AND m.agent_id = '{}'", agent_id));
            }
            if let Some(user_id) = filters.get("user_id") {
                sql.push_str(&format!(" AND m.user_id = '{}'", user_id));
            }
        }
        
        sql.push_str(" AND m.is_deleted = 0");
        sql.push_str(" ORDER BY score");
        sql.push_str(&format!(" LIMIT {}", limit));
        
        debug!("SQL: {}", sql);
        
        // 执行查询
        let mut rows = self.conn
            .query(&sql, libsql::params![query])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("FTS5 search failed: {e}")))?;
        
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            let id: String = row.get(0).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get id: {e}"))
            })?;
            
            let content: String = row.get(1).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get content: {e}"))
            })?;
            
            // Get as f64 first, then convert to f32
            let score_f64: f64 = row.get(2).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get score: {e}"))
            })?;
            let score = score_f64 as f32;
            
            let agent_id: String = row.get(3).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get agent_id: {e}"))
            })?;
            
            let user_id: Option<String> = row.get(4).ok();
            
            let created_at_ts: i64 = row.get(5).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
            })?;
            
            let created_at = DateTime::from_timestamp(created_at_ts, 0)
                .unwrap_or_else(|| Utc::now());
            
            let metadata_json: String = row.get(6).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get metadata: {e}"))
            })?;
            
            let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json)
                .unwrap_or_default();
            
            results.push(FTS5SearchResult {
                id,
                content,
                score,
                agent_id,
                user_id,
                created_at,
                metadata,
            });
        }
        
        debug!("FTS5 search returned {} results", results.len());
        Ok(results)
    }
    
    /// 精确匹配搜索（用于ID等）
    pub async fn exact_match(
        &self,
        query: &str,
        limit: usize,
        filters: Option<&HashMap<String, String>>,
    ) -> Result<Vec<FTS5SearchResult>> {
        debug!("Exact match search: query='{}', limit={}", query, limit);
        
        let mut sql = String::from(
            "SELECT 
                id, content, 1.0 as score, agent_id, user_id, created_at, metadata
            FROM memories
            WHERE (id = ? OR content LIKE ?)
            AND is_deleted = 0"
        );
        
        // 添加过滤条件
        if let Some(filters) = filters {
            if let Some(agent_id) = filters.get("agent_id") {
                sql.push_str(&format!(" AND agent_id = '{}'", agent_id));
            }
            if let Some(user_id) = filters.get("user_id") {
                sql.push_str(&format!(" AND user_id = '{}'", user_id));
            }
        }
        
        sql.push_str(&format!(" LIMIT {}", limit));
        
        let like_pattern = format!("%{}%", query);
        let mut rows = self.conn
            .query(&sql, libsql::params![query, like_pattern])
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Exact match failed: {e}")))?;
        
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to fetch row: {e}"))
        })? {
            let id: String = row.get(0).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get id: {e}"))
            })?;
            let content: String = row.get(1).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get content: {e}"))
            })?;
            // Get as f64 first, then convert to f32
            let score_f64: f64 = row.get(2).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get score: {e}"))
            })?;
            let score = score_f64 as f32;
            let agent_id: String = row.get(3).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get agent_id: {e}"))
            })?;
            let user_id: Option<String> = row.get(4).ok();
            let created_at_ts: i64 = row.get(5).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get created_at: {e}"))
            })?;
            let created_at = DateTime::from_timestamp(created_at_ts, 0)
                .unwrap_or_else(|| Utc::now());
            let metadata_json: String = row.get(6).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get metadata: {e}"))
            })?;
            let metadata: HashMap<String, String> = serde_json::from_str(&metadata_json)
                .unwrap_or_default();
            
            results.push(FTS5SearchResult {
                id,
                content,
                score,
                agent_id,
                user_id,
                created_at,
                metadata,
            });
        }
        
        debug!("Exact match returned {} results", results.len());
        Ok(results)
    }
    
    /// 混合搜索（结合FTS5和精确匹配）
    pub async fn hybrid_search(
        &self,
        query: &str,
        limit: usize,
        filters: Option<&HashMap<String, String>>,
    ) -> Result<Vec<FTS5SearchResult>> {
        // 先尝试精确匹配
        let exact_results = self.exact_match(query, limit, filters).await?;
        if !exact_results.is_empty() {
            return Ok(exact_results);
        }
        
        // 如果没有精确匹配，使用FTS5搜索
        self.search_bm25(query, limit, filters).await
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<FTS5Stats> {
        let mut rows = self.conn
            .query("SELECT COUNT(*) FROM memories WHERE is_deleted = 0", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get stats: {e}")))?;
        
        let total_memories = if let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::StorageError(format!("Failed to get stats row: {e}"))
        })? {
            row.get::<i64>(0).unwrap_or(0) as usize
        } else {
            0
        };
        
        let mut rows = self.conn
            .query("SELECT COUNT(*) FROM memories_fts", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to get FTS stats: {e}")))?;
        
        let indexed_memories = if let Some(row) = rows.next().await.map_err(convert_libsql_error)? {
            row.get::<i64>(0).unwrap_or(0) as usize
        } else {
            0
        };
        
        Ok(FTS5Stats {
            total_memories,
            indexed_memories,
            bm25_params: self.bm25_params.clone(),
        })
    }
    
    /// 重建FTS5索引
    pub async fn rebuild_index(&self) -> Result<()> {
        info!("Rebuilding FTS5 index...");
        
        self.conn
            .execute("INSERT INTO memories_fts(memories_fts) VALUES('rebuild')", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to rebuild index: {e}")))?;
        
        info!("FTS5 index rebuilt successfully");
        Ok(())
    }
    
    /// 优化FTS5索引
    pub async fn optimize_index(&self) -> Result<()> {
        info!("Optimizing FTS5 index...");
        
        self.conn
            .execute("INSERT INTO memories_fts(memories_fts) VALUES('optimize')", ())
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to optimize index: {e}")))?;
        
        info!("FTS5 index optimized successfully");
        Ok(())
    }
}

/// FTS5统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FTS5Stats {
    pub total_memories: usize,
    pub indexed_memories: usize,
    pub bm25_params: BM25Params,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fts5_store_creation() {
        let store = LibSQLFTS5Store::new(":memory:").await;
        assert!(store.is_ok());
    }
    
    #[tokio::test]
    async fn test_bm25_params() {
        let params = BM25Params::default();
        assert_eq!(params.k1, 1.2);
        assert_eq!(params.b, 0.75);
    }
    
    #[tokio::test]
    async fn test_fts5_search_empty() {
        let store = LibSQLFTS5Store::new(":memory:").await.unwrap();
        let results = store.search_bm25("test", 10, None).await.unwrap();
        assert_eq!(results.len(), 0);
    }
    
    #[tokio::test]
    async fn test_exact_match_empty() {
        let store = LibSQLFTS5Store::new(":memory:").await.unwrap();
        let results = store.exact_match("test", 10, None).await.unwrap();
        assert_eq!(results.len(), 0);
    }
}

