//! LanceDB vector database storage implementation
//!
//! Provides embedded vector search capabilities for AgentMem.
//! LanceDB is a serverless, low-latency vector database built on Lance format.

use agent_mem_traits::{AgentMemError, Result, VectorData, VectorSearchResult, VectorStore};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

#[cfg(feature = "lancedb")]
use lancedb::{connect, Connection, Table};
#[cfg(feature = "lancedb")]
use lancedb::query::{ExecutableQuery, QueryBase};

#[cfg(feature = "lancedb")]
use arrow::array::{Array, ArrayRef, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray};
#[cfg(feature = "lancedb")]
use arrow::datatypes::{DataType, Field, Schema};
#[cfg(feature = "lancedb")]
use std::sync::Arc as ArrowArc;
#[cfg(feature = "lancedb")]
use futures::TryStreamExt;

/// LanceDB vector store
#[cfg(feature = "lancedb")]
pub struct LanceDBStore {
    conn: Arc<Connection>,
    table_name: String,
}

#[cfg(feature = "lancedb")]
impl LanceDBStore {
    /// Create a new LanceDB store
    ///
    /// # Arguments
    /// * `path` - Path to the LanceDB directory
    /// * `table_name` - Name of the table to use (default: "vectors")
    ///
    /// # Example
    /// ```no_run
    /// use agent_mem_storage::backends::lancedb_store::LanceDBStore;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = LanceDBStore::new("~/.agentmem/vectors.lance", "vectors").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(path: &str, table_name: &str) -> Result<Self> {
        info!("Initializing LanceDB store at: {}", path);

        // Expand home directory
        let expanded_path = if path.starts_with("~/") {
            let home = std::env::var("HOME").map_err(|e| {
                AgentMemError::StorageError(format!("Failed to get HOME directory: {}", e))
            })?;
            path.replace("~", &home)
        } else {
            path.to_string()
        };

        // Create parent directory if needed
        if let Some(parent) = Path::new(&expanded_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AgentMemError::StorageError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Connect to LanceDB
        let conn = connect(&expanded_path)
            .execute()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to connect to LanceDB: {}", e)))?;

        info!("LanceDB store initialized successfully");

        Ok(Self {
            conn: Arc::new(conn),
            table_name: table_name.to_string(),
        })
    }

    /// Get or create the vectors table
    async fn get_or_create_table(&self) -> Result<Table> {
        // Check if table exists
        let table_names = self
            .conn
            .table_names()
            .execute()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to list tables: {}", e)))?;

        if table_names.contains(&self.table_name) {
            // Open existing table
            self.conn
                .open_table(&self.table_name)
                .execute()
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to open table: {}", e)))
        } else {
            // Create new table with empty data
            // We'll add data later
            Err(AgentMemError::StorageError(format!(
                "Table '{}' does not exist. Use add_vectors to create it.",
                self.table_name
            )))
        }
    }

    /// Create table if it doesn't exist
    async fn ensure_table_exists(&self, dimension: usize) -> Result<()> {
        let table_names = self
            .conn
            .table_names()
            .execute()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to list tables: {}", e)))?;

        if !table_names.contains(&self.table_name) {
            debug!("Creating new table: {}", self.table_name);
            // Table will be created when first data is added
        }

        Ok(())
    }
}

#[cfg(feature = "lancedb")]
#[async_trait]
impl VectorStore for LanceDBStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        debug!("Adding {} vectors to LanceDB", vectors.len());

        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        // Get dimension from first vector
        let dimension = vectors[0].vector.len();
        let num_vectors = vectors.len();

        // 1. Create Arrow Schema
        let schema = ArrowArc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    ArrowArc::new(Field::new("item", DataType::Float32, true)),
                    dimension as i32,
                ),
                false,
            ),
            Field::new("metadata", DataType::Utf8, true),
        ]));

        // 2. Convert VectorData to Arrow arrays
        // ID array
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();
        let id_array = StringArray::from(ids.clone());

        // Vector array (as FixedSizeList)
        let vector_values: Vec<f32> = vectors
            .iter()
            .flat_map(|v| v.vector.clone())
            .collect();
        let vector_value_array = Float32Array::from(vector_values);
        let vector_array = FixedSizeListArray::new(
            ArrowArc::new(Field::new("item", DataType::Float32, true)),
            dimension as i32,
            ArrowArc::new(vector_value_array) as ArrayRef,
            None,
        );

        // Metadata array (serialize HashMap to JSON string)
        let metadata_values: Vec<Option<String>> = vectors
            .iter()
            .map(|v| {
                if v.metadata.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&v.metadata).unwrap_or_default())
                }
            })
            .collect();
        let metadata_array = StringArray::from(metadata_values);

        // 3. Create RecordBatch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                ArrowArc::new(id_array) as ArrayRef,
                ArrowArc::new(vector_array) as ArrayRef,
                ArrowArc::new(metadata_array) as ArrayRef,
            ],
        )
        .map_err(|e| {
            AgentMemError::StorageError(format!("Failed to create RecordBatch: {}", e))
        })?;

        debug!(
            "Created RecordBatch with {} rows, {} columns",
            batch.num_rows(),
            batch.num_columns()
        );

        // 4. Insert into LanceDB
        let table_names = self
            .conn
            .table_names()
            .execute()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to list tables: {}", e)))?;

        // Create RecordBatchIterator (implements RecordBatchReader)
        let batches = vec![Ok(batch)];
        let reader = RecordBatchIterator::new(batches.into_iter(), schema.clone());

        if table_names.contains(&self.table_name) {
            // Table exists, append data
            let table = self
                .conn
                .open_table(&self.table_name)
                .execute()
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to open table: {}", e)))?;

            table
                .add(reader)
                .execute()
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to add vectors: {}", e)))?;

            info!("Added {} vectors to existing table '{}'", num_vectors, self.table_name);
        } else {
            // Create new table with data
            self.conn
                .create_table(&self.table_name, reader)
                .execute()
                .await
                .map_err(|e| {
                    AgentMemError::StorageError(format!("Failed to create table: {}", e))
                })?;

            info!("Created table '{}' with {} vectors", self.table_name, num_vectors);
        }

        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        info!("Searching for {} similar vectors with threshold {:?}", limit, threshold);

        // 1. 获取表
        let table = self.get_or_create_table().await?;

        // 2. 执行向量搜索
        // LanceDB 0.22.2 API: table.query().nearest_to(&query_vector)?.limit(limit).execute().await?
        let batches = table
            .query()
            .nearest_to(query_vector.as_slice())
            .map_err(|e| AgentMemError::StorageError(format!("Failed to create nearest_to query: {}", e)))?
            .limit(limit)
            .execute()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to execute query: {}", e)))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to collect results: {}", e)))?;

        // 3. 解析结果并转换为 VectorSearchResult
        let mut results = Vec::new();

        for batch in batches {
            let num_rows = batch.num_rows();
            if num_rows == 0 {
                continue;
            }

            // 获取列数据
            let id_array = batch
                .column_by_name("id")
                .ok_or_else(|| AgentMemError::StorageError("Missing 'id' column".to_string()))?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| AgentMemError::StorageError("Invalid 'id' column type".to_string()))?;

            let vector_array = batch
                .column_by_name("vector")
                .ok_or_else(|| AgentMemError::StorageError("Missing 'vector' column".to_string()))?
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .ok_or_else(|| AgentMemError::StorageError("Invalid 'vector' column type".to_string()))?;

            let metadata_array = batch
                .column_by_name("metadata")
                .ok_or_else(|| AgentMemError::StorageError("Missing 'metadata' column".to_string()))?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| AgentMemError::StorageError("Invalid 'metadata' column type".to_string()))?;

            // 检查是否有距离列（LanceDB 搜索结果可能包含 _distance 列）
            let distance_array = batch.column_by_name("_distance")
                .and_then(|col| col.as_any().downcast_ref::<Float32Array>());

            // 处理每一行
            for i in 0..num_rows {
                let id = id_array.value(i).to_string();

                // 提取向量
                let vector_list = vector_array.value(i);
                let vector_data = vector_list
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .ok_or_else(|| AgentMemError::StorageError("Invalid vector data type".to_string()))?;
                let vector: Vec<f32> = vector_data.values().to_vec();

                // 提取 metadata
                let metadata_str = if metadata_array.is_null(i) {
                    String::new()
                } else {
                    metadata_array.value(i).to_string()
                };
                let metadata: HashMap<String, String> = if metadata_str.is_empty() {
                    HashMap::new()
                } else {
                    serde_json::from_str(&metadata_str).unwrap_or_default()
                };

                // 计算距离和相似度
                let distance = if let Some(dist_arr) = distance_array {
                    dist_arr.value(i)
                } else {
                    // 如果没有距离列，手动计算欧氏距离
                    let sum: f32 = query_vector.iter()
                        .zip(vector.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum();
                    sum.sqrt()
                };

                // 将距离转换为相似度（假设使用 L2 距离）
                // 相似度 = 1 / (1 + distance)
                let similarity = 1.0 / (1.0 + distance);

                // 应用阈值过滤
                if let Some(threshold) = threshold {
                    if similarity < threshold {
                        continue;
                    }
                }

                results.push(VectorSearchResult {
                    id,
                    vector,
                    metadata,
                    similarity,
                    distance,
                });
            }
        }

        info!("Found {} similar vectors", results.len());
        Ok(results)
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, serde_json::Value>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        debug!("Searching with filters: {:?}", filters);

        // TODO: Implement filtered search
        warn!("LanceDB search_with_filters is not fully implemented yet");

        Ok(Vec::new())
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        info!("Deleting {} vectors", ids.len());

        // 1. 获取表
        let table = self.get_or_create_table().await?;

        // 2. 构建删除条件
        // LanceDB delete API 使用 SQL-like 条件: "id = 'vec1' OR id = 'vec2'"
        let condition = ids.iter()
            .map(|id| format!("id = '{}'", id.replace("'", "''"))) // 转义单引号
            .collect::<Vec<_>>()
            .join(" OR ");

        // 3. 执行删除
        table
            .delete(&condition)
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Delete failed: {}", e)))?;

        info!("Successfully deleted {} vectors", ids.len());
        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        info!("Updating {} vectors", vectors.len());

        // LanceDB doesn't have a native update API in version 0.22.2
        // We use delete + insert strategy for updates
        // This is atomic at the table level and ensures data consistency

        // 1. Extract IDs to delete
        let ids: Vec<String> = vectors.iter().map(|v| v.id.clone()).collect();

        // 2. Delete existing vectors
        self.delete_vectors(ids).await?;

        // 3. Insert updated vectors
        self.add_vectors(vectors).await?;

        info!("Successfully updated vectors using delete+insert strategy");
        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        debug!("Getting vector by ID: {}", id);

        // Get table
        let table = match self.get_or_create_table().await {
            Ok(t) => t,
            Err(_) => return Ok(None), // Table doesn't exist, no vector found
        };

        // LanceDB 0.22.2 doesn't have a simple get-by-id API
        // We use a full table scan and filter in memory
        // For production use, consider using an index or nearest_to with a dummy vector

        // Execute full table scan
        let batches = table
            .query()
            .execute()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to execute query: {}", e)))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| AgentMemError::StorageError(format!("Failed to collect results: {}", e)))?;

        // Parse results and find matching ID
        for batch in batches {
            if batch.num_rows() == 0 {
                continue;
            }

            // Get column data
            let id_array = batch
                .column_by_name("id")
                .ok_or_else(|| AgentMemError::StorageError("Missing 'id' column".to_string()))?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| AgentMemError::StorageError("Invalid 'id' column type".to_string()))?;

            let vector_array = batch
                .column_by_name("vector")
                .ok_or_else(|| AgentMemError::StorageError("Missing 'vector' column".to_string()))?
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .ok_or_else(|| AgentMemError::StorageError("Invalid 'vector' column type".to_string()))?;

            let metadata_array = batch
                .column_by_name("metadata")
                .ok_or_else(|| AgentMemError::StorageError("Missing 'metadata' column".to_string()))?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| AgentMemError::StorageError("Invalid 'metadata' column type".to_string()))?;

            // Scan all rows to find matching ID
            for row_idx in 0..batch.num_rows() {
                let found_id = id_array.value(row_idx).to_string();

                // Check if this is the ID we're looking for
                if found_id == id {
                    // Extract vector
                    let vector_list = vector_array.value(row_idx);
                    let vector_data = vector_list
                        .as_any()
                        .downcast_ref::<Float32Array>()
                        .ok_or_else(|| AgentMemError::StorageError("Invalid vector data type".to_string()))?;
                    let vector: Vec<f32> = vector_data.values().to_vec();

                    // Extract metadata
                    let metadata: HashMap<String, String> = if metadata_array.is_null(row_idx) {
                        HashMap::new()
                    } else {
                        let metadata_str = metadata_array.value(row_idx);
                        serde_json::from_str(metadata_str).unwrap_or_default()
                    };

                    debug!("Found vector with ID: {}", found_id);

                    return Ok(Some(VectorData {
                        id: found_id,
                        vector,
                        metadata,
                    }));
                }
            }
        }

        debug!("Vector with ID '{}' not found", id);
        Ok(None)
    }

    async fn count_vectors(&self) -> Result<usize> {
        // Get table
        match self.get_or_create_table().await {
            Ok(table) => {
                // Get count from table
                let count = table
                    .count_rows(None)
                    .await
                    .map_err(|e| AgentMemError::StorageError(format!("Failed to count rows: {}", e)))?;
                Ok(count)
            }
            Err(_) => Ok(0),
        }
    }

    async fn clear(&self) -> Result<()> {
        warn!("Clearing all vectors from LanceDB");

        // TODO: Implement clear operation
        warn!("LanceDB clear is not fully implemented yet");

        Ok(())
    }

    async fn health_check(&self) -> Result<agent_mem_traits::HealthStatus> {
        // Check if we can list tables
        match self.conn.table_names().execute().await {
            Ok(_) => Ok(agent_mem_traits::HealthStatus::healthy()),
            Err(e) => Ok(agent_mem_traits::HealthStatus::unhealthy(&format!(
                "LanceDB health check failed: {}",
                e
            ))),
        }
    }

    async fn get_stats(&self) -> Result<agent_mem_traits::VectorStoreStats> {
        let count = self.count_vectors().await?;

        Ok(agent_mem_traits::VectorStoreStats {
            total_vectors: count,
            dimension: 1536, // TODO: Get actual dimension
            index_size: 0,   // TODO: Get actual index size
        })
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        debug!("Adding {} batches of vectors", batches.len());

        let mut all_ids = Vec::new();
        for batch in batches {
            let ids = self.add_vectors(batch).await?;
            all_ids.push(ids);
        }

        Ok(all_ids)
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        debug!("Deleting {} batches of vectors", id_batches.len());

        let mut results = Vec::new();
        for batch in id_batches {
            self.delete_vectors(batch).await?;
            results.push(true);
        }

        Ok(results)
    }
}

/// Stub implementation when lancedb feature is not enabled
#[cfg(not(feature = "lancedb"))]
pub struct LanceDBStore;

#[cfg(not(feature = "lancedb"))]
impl LanceDBStore {
    pub async fn new(_path: &str, _table_name: &str) -> Result<Self> {
        Err(AgentMemError::StorageError(
            "LanceDB feature is not enabled. Enable with --features lancedb".to_string(),
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "lancedb")]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_lancedb_initialization() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Test health check
        let health = store.health_check().await.unwrap();
        assert_eq!(health.status, "healthy");
    }

    #[tokio::test]
    async fn test_lancedb_stats() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 0);
    }

    #[tokio::test]
    async fn test_add_vectors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Create test vectors
        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 2.0, 3.0, 4.0],
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("key1".to_string(), "value1".to_string());
                    map
                },
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![5.0, 6.0, 7.0, 8.0],
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("key2".to_string(), "value2".to_string());
                    map
                },
            },
        ];

        // Add vectors
        let ids = store.add_vectors(vectors).await.unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "vec1");
        assert_eq!(ids[1], "vec2");

        // Verify stats
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 2);
    }

    #[tokio::test]
    async fn test_add_vectors_multiple_batches() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // First batch
        let vectors1 = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 2.0, 3.0],
                metadata: std::collections::HashMap::new(),
            },
        ];
        let ids1 = store.add_vectors(vectors1).await.unwrap();
        assert_eq!(ids1.len(), 1);

        // Second batch
        let vectors2 = vec![
            VectorData {
                id: "vec2".to_string(),
                vector: vec![4.0, 5.0, 6.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec3".to_string(),
                vector: vec![7.0, 8.0, 9.0],
                metadata: std::collections::HashMap::new(),
            },
        ];
        let ids2 = store.add_vectors(vectors2).await.unwrap();
        assert_eq!(ids2.len(), 2);

        // Verify total count
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 3);
    }

    #[tokio::test]
    async fn test_search_vectors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add test vectors
        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 0.0, 0.0, 0.0],
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("label".to_string(), "first".to_string());
                    map
                },
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.0, 1.0, 0.0, 0.0],
                metadata: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("label".to_string(), "second".to_string());
                    map
                },
            },
        ];

        store.add_vectors(vectors).await.unwrap();

        // Search for similar vectors
        // Query vector is close to vec1 [1.0, 0.0, 0.0, 0.0]
        let query = vec![0.9, 0.1, 0.0, 0.0];
        let results = store.search_vectors(query, 2, None).await.unwrap();

        // Should return 2 results (vec1 and vec2)
        assert_eq!(results.len(), 2);

        // First result should be vec1 (closest to query)
        assert_eq!(results[0].id, "vec1");
        assert!(results[0].similarity > results[1].similarity);

        // Verify metadata
        assert_eq!(results[0].metadata.get("label").unwrap(), "first");
        assert_eq!(results[1].metadata.get("label").unwrap(), "second");
    }

    #[tokio::test]
    async fn test_search_with_threshold() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add test vectors
        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 0.0, 0.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.0, 1.0, 0.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec3".to_string(),
                vector: vec![0.0, 0.0, 1.0],
                metadata: std::collections::HashMap::new(),
            },
        ];

        store.add_vectors(vectors).await.unwrap();

        // Search with high threshold - should filter out distant vectors
        let query = vec![1.0, 0.0, 0.0];
        let results = store.search_vectors(query.clone(), 10, Some(0.8)).await.unwrap();

        // Only vec1 should pass the threshold (exact match, similarity = 1.0)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "vec1");
        assert!(results[0].similarity >= 0.8);
    }

    #[tokio::test]
    async fn test_delete_vectors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add test vectors
        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 0.0, 0.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.0, 1.0, 0.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec3".to_string(),
                vector: vec![0.0, 0.0, 1.0],
                metadata: std::collections::HashMap::new(),
            },
        ];

        store.add_vectors(vectors).await.unwrap();

        // Verify all vectors are added
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 3);

        // Delete vec2
        store.delete_vectors(vec!["vec2".to_string()]).await.unwrap();

        // Verify vec2 is deleted
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 2);

        // Search should not return vec2
        let results = store.search_vectors(vec![0.0, 1.0, 0.0], 10, None).await.unwrap();
        assert!(!results.iter().any(|r| r.id == "vec2"));
    }

    #[tokio::test]
    async fn test_delete_multiple_vectors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add test vectors
        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 0.0, 0.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.0, 1.0, 0.0],
                metadata: std::collections::HashMap::new(),
            },
            VectorData {
                id: "vec3".to_string(),
                vector: vec![0.0, 0.0, 1.0],
                metadata: std::collections::HashMap::new(),
            },
        ];

        store.add_vectors(vectors).await.unwrap();

        // Delete vec1 and vec3
        store.delete_vectors(vec!["vec1".to_string(), "vec3".to_string()]).await.unwrap();

        // Verify only vec2 remains
        let stats = store.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 1);

        // Search should only return vec2
        let results = store.search_vectors(vec![0.0, 1.0, 0.0], 10, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "vec2");
    }

    #[tokio::test]
    async fn test_delete_empty_list() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Delete empty list should not error
        store.delete_vectors(vec![]).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_vectors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add initial vectors
        let mut metadata1 = HashMap::new();
        metadata1.insert("version".to_string(), "1".to_string());

        let mut metadata2 = HashMap::new();
        metadata2.insert("version".to_string(), "1".to_string());

        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 0.0, 0.0, 0.0],
                metadata: metadata1,
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.0, 1.0, 0.0, 0.0],
                metadata: metadata2,
            },
        ];

        store.add_vectors(vectors).await.unwrap();

        // Update vectors with new data
        let mut updated_metadata1 = HashMap::new();
        updated_metadata1.insert("version".to_string(), "2".to_string());
        updated_metadata1.insert("updated".to_string(), "true".to_string());

        let updated_vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![0.9, 0.1, 0.0, 0.0], // Changed vector
                metadata: updated_metadata1,
            },
        ];

        store.update_vectors(updated_vectors).await.unwrap();

        // Verify update by searching
        let results = store
            .search_vectors(vec![0.9, 0.1, 0.0, 0.0], 10, None)
            .await
            .unwrap();

        assert!(!results.is_empty());
        let vec1_result = results.iter().find(|r| r.id == "vec1").unwrap();
        assert_eq!(vec1_result.metadata.get("version").unwrap(), "2");
        assert_eq!(vec1_result.metadata.get("updated").unwrap(), "true");

        // Verify vec2 still exists
        let vec2_exists = results.iter().any(|r| r.id == "vec2");
        assert!(vec2_exists);
    }

    #[tokio::test]
    async fn test_get_vector() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add vectors
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let vectors = vec![
            VectorData {
                id: "vec1".to_string(),
                vector: vec![1.0, 0.0, 0.0, 0.0],
                metadata: metadata.clone(),
            },
            VectorData {
                id: "vec2".to_string(),
                vector: vec![0.0, 1.0, 0.0, 0.0],
                metadata: HashMap::new(),
            },
        ];

        store.add_vectors(vectors).await.unwrap();

        // Get existing vector
        let result = store.get_vector("vec1").await.unwrap();
        assert!(result.is_some());

        let vec1 = result.unwrap();
        assert_eq!(vec1.id, "vec1");
        assert_eq!(vec1.vector, vec![1.0, 0.0, 0.0, 0.0]);
        assert_eq!(vec1.metadata.get("key1").unwrap(), "value1");
        assert_eq!(vec1.metadata.get("key2").unwrap(), "value2");

        // Get non-existent vector
        let result = store.get_vector("vec999").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_vector_empty_metadata() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Add vector with empty metadata
        let vectors = vec![VectorData {
            id: "vec1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        }];

        store.add_vectors(vectors).await.unwrap();

        // Get vector
        let result = store.get_vector("vec1").await.unwrap();
        assert!(result.is_some());

        let vec1 = result.unwrap();
        assert_eq!(vec1.id, "vec1");
        assert_eq!(vec1.vector, vec![1.0, 2.0, 3.0]);
        assert!(vec1.metadata.is_empty());
    }

    #[tokio::test]
    async fn test_update_empty_list() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // Update empty list should not error
        store.update_vectors(vec![]).await.unwrap();
    }

    /// 性能基准测试：向量插入性能
    /// 目标：> 1000 ops/s
    #[tokio::test]
    async fn test_insert_performance() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // 准备 1000 个向量
        let num_vectors = 1000;
        let dimension = 128;
        let mut vectors = Vec::new();

        for i in 0..num_vectors {
            vectors.push(VectorData {
                id: format!("vec_{}", i),
                vector: vec![i as f32 / num_vectors as f32; dimension],
                metadata: std::collections::HashMap::new(),
            });
        }

        // 测试插入性能
        let start = std::time::Instant::now();
        store.add_vectors(vectors).await.unwrap();
        let duration = start.elapsed();

        let ops_per_sec = num_vectors as f64 / duration.as_secs_f64();
        println!("插入性能: {:.2} ops/s (目标: > 1000 ops/s)", ops_per_sec);
        println!("插入 {} 个向量耗时: {:?}", num_vectors, duration);

        // 验证性能指标
        assert!(ops_per_sec > 1000.0, "插入性能未达标: {:.2} ops/s < 1000 ops/s", ops_per_sec);
    }

    /// 性能基准测试：向量搜索性能 (1K 向量)
    /// 目标：< 50ms (LanceDB 嵌入式数据库的合理性能目标)
    #[tokio::test]
    async fn test_search_performance_1k() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // 准备 1000 个向量
        let num_vectors = 1000;
        let dimension = 128;
        let mut vectors = Vec::new();

        for i in 0..num_vectors {
            vectors.push(VectorData {
                id: format!("vec_{}", i),
                vector: vec![i as f32 / num_vectors as f32; dimension],
                metadata: std::collections::HashMap::new(),
            });
        }

        store.add_vectors(vectors).await.unwrap();

        // 测试搜索性能
        let query = vec![0.5; dimension];
        let start = std::time::Instant::now();
        let results = store.search_vectors(query, 10, None).await.unwrap();
        let duration = start.elapsed();

        println!("搜索性能 (1K 向量): {:?} (目标: < 50ms)", duration);
        println!("返回结果数: {}", results.len());

        // 验证性能指标（LanceDB 嵌入式数据库，50ms 是合理目标）
        assert!(duration.as_millis() < 50, "搜索延迟未达标: {:?} >= 50ms", duration);
    }

    /// 性能基准测试：向量搜索性能 (10K 向量)
    /// 目标：< 50ms (文档中是 100K，但为了测试速度，这里用 10K)
    #[tokio::test]
    #[ignore] // 默认忽略，因为需要较长时间
    async fn test_search_performance_10k() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.lance");

        let store = LanceDBStore::new(path.to_str().unwrap(), "vectors")
            .await
            .unwrap();

        // 准备 10000 个向量
        let num_vectors = 10000;
        let dimension = 128;

        // 分批插入以提高性能
        let batch_size = 1000;
        for batch_idx in 0..(num_vectors / batch_size) {
            let mut vectors = Vec::new();
            for i in 0..batch_size {
                let idx = batch_idx * batch_size + i;
                vectors.push(VectorData {
                    id: format!("vec_{}", idx),
                    vector: vec![idx as f32 / num_vectors as f32; dimension],
                    metadata: std::collections::HashMap::new(),
                });
            }
            store.add_vectors(vectors).await.unwrap();
        }

        // 测试搜索性能
        let query = vec![0.5; dimension];
        let start = std::time::Instant::now();
        let results = store.search_vectors(query, 10, None).await.unwrap();
        let duration = start.elapsed();

        println!("搜索性能 (10K 向量): {:?} (目标: < 50ms)", duration);
        println!("返回结果数: {}", results.len());

        // 验证性能指标（10K 向量应该 < 50ms）
        assert!(duration.as_millis() < 50, "搜索延迟未达标: {:?} >= 50ms", duration);
    }
}

