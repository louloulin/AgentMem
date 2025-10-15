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
use arrow::array::{ArrayRef, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray};
#[cfg(feature = "lancedb")]
use arrow::datatypes::{DataType, Field, Schema};
#[cfg(feature = "lancedb")]
use std::sync::Arc as ArrowArc;

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
        debug!("Searching for {} similar vectors", limit);

        // Get table
        let table = self.get_or_create_table().await?;

        // TODO: Implement actual vector search using LanceDB
        // For now, return empty results
        warn!("LanceDB search_vectors is not fully implemented yet");

        Ok(Vec::new())
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
        debug!("Deleting {} vectors", ids.len());

        // TODO: Implement vector deletion
        warn!("LanceDB delete_vectors is not fully implemented yet");

        Ok(())
    }

    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        debug!("Updating {} vectors", vectors.len());

        // TODO: Implement vector update
        warn!("LanceDB update_vectors is not fully implemented yet");

        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        debug!("Getting vector: {}", id);

        // TODO: Implement get vector by ID
        warn!("LanceDB get_vector is not fully implemented yet");

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
}

