//! Python bindings for AgentMem
//!
//! This crate provides Python bindings for AgentMem using PyO3.

use agent_mem_core::SimpleMemory as RustSimpleMemory;
use agent_mem_traits::MemoryItem;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Python wrapper for SimpleMemory
/// 
/// Uses Arc<RwLock<>> to solve lifetime and Clone issues
#[pyclass(name = "Memory")]
#[derive(Clone)]
struct PyMemory {
    inner: Arc<RwLock<RustSimpleMemory>>,
}

#[pymethods]
impl PyMemory {
    /// Create a new Memory instance
    #[new]
    fn new() -> PyResult<Self> {
        // Create a new SimpleMemory instance
        let inner = pyo3_asyncio::tokio::get_runtime()
            .block_on(async { RustSimpleMemory::new().await })
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Memory: {}", e)))?;

        Ok(Self { 
            inner: Arc::new(RwLock::new(inner))
        })
    }

    /// Add a memory
    ///
    /// Args:
    ///     content (str): The memory content
    ///     agent_id (str, optional): Agent identifier
    ///     user_id (str, optional): User identifier
    ///     metadata (dict, optional): Additional metadata
    ///
    /// Returns:
    ///     str: Memory ID
    fn add<'py>(
        &self,
        py: Python<'py>,
        content: String,
        agent_id: Option<String>,
        user_id: Option<String>,
        metadata: Option<&PyDict>,
    ) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);
        
        // Convert Python dict to Rust HashMap
        let metadata_map = if let Some(meta) = metadata {
            let mut map = HashMap::new();
            for (key, value) in meta.iter() {
                let key_str: String = key.extract()?;
                let value_str: String = value.extract()?;
                map.insert(key_str, value_str);
            }
            Some(map)
        } else {
            None
        };

        pyo3_asyncio::tokio::future_into_py(py, async move {
            // Get read lock to access inner memory
            let memory = {
                let guard = inner.read();
                guard.clone()  // Clone the SimpleMemory for async usage
            };

            // Set user_id and agent_id if provided
            let memory = if let Some(user_id) = user_id {
                memory.with_user_id(user_id)
            } else {
                memory
            };

            let memory = if let Some(agent_id) = agent_id {
                memory.with_agent_id(agent_id)
            } else {
                memory
            };

            // Add memory
            let id = if let Some(meta) = metadata_map {
                memory.add_with_metadata(content, Some(meta)).await
            } else {
                memory.add(content).await
            }
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to add memory: {}", e)))?;

            Ok(id)
        })
    }

    /// Search memories
    ///
    /// Args:
    ///     query (str): Search query
    ///     limit (int, optional): Maximum number of results (default: 10)
    ///
    /// Returns:
    ///     list: List of memory dictionaries
    fn search<'py>(
        &self,
        py: Python<'py>,
        query: String,
        limit: Option<usize>,
    ) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);
        let limit = limit.unwrap_or(10);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let memory = {
                let guard = inner.read();
                guard.clone()
            };
            let results = memory
                .search_with_limit(query, limit)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to search: {}", e)))?;

            // Convert results to Python list of dicts
            Python::with_gil(|py| {
                let list = PyList::empty(py);
                for item in results {
                    let dict = memory_item_to_dict(py, &item)?;
                    list.append(dict)?;
                }
                Ok(list.into())
            })
        })
    }

    /// Get a memory by ID
    ///
    /// Args:
    ///     memory_id (str): Memory identifier
    ///
    /// Returns:
    ///     dict or None: Memory dictionary or None if not found
    fn get<'py>(&self, py: Python<'py>, memory_id: String) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mem = {
                let guard = inner.read();
                guard.clone()
            };
            let memory = mem
                .get(&memory_id)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get memory: {}", e)))?;

            Python::with_gil(|py| {
                if let Some(item) = memory {
                    let dict = memory_item_to_dict(py, &item)?;
                    Ok(dict.into())
                } else {
                    Ok(py.None())
                }
            })
        })
    }

    /// Get all memories
    ///
    /// Args:
    ///     limit (int, optional): Maximum number of results
    ///
    /// Returns:
    ///     list: List of memory dictionaries
    fn get_all<'py>(&self, py: Python<'py>, limit: Option<usize>) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let memory = {
                let guard = inner.read();
                guard.clone()
            };
            let results = memory
                .get_all(limit)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get all: {}", e)))?;

            Python::with_gil(|py| {
                let list = PyList::empty(py);
                for item in results {
                    let dict = memory_item_to_dict(py, &item)?;
                    list.append(dict)?;
                }
                Ok(list.into())
            })
        })
    }

    /// Update a memory
    ///
    /// Args:
    ///     memory_id (str): Memory identifier
    ///     content (str, optional): New content
    ///     importance (float, optional): New importance score
    ///
    /// Returns:
    ///     dict: Updated memory dictionary
    fn update<'py>(
        &self,
        py: Python<'py>,
        memory_id: String,
        content: Option<String>,
        importance: Option<f32>,
    ) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mem = {
                let guard = inner.read();
                guard.clone()
            };
            let memory = mem
                .update(&memory_id, content, importance)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to update: {}", e)))?;

            Python::with_gil(|py| {
                let dict = memory_item_to_dict(py, &memory)?;
                Ok(dict.into())
            })
        })
    }

    /// Delete a memory
    ///
    /// Args:
    ///     memory_id (str): Memory identifier
    ///
    /// Returns:
    ///     bool: True if deleted successfully
    fn delete<'py>(&self, py: Python<'py>, memory_id: String) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let memory = {
                let guard = inner.read();
                guard.clone()
            };
            let success = memory
                .delete(&memory_id)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to delete: {}", e)))?;

            Ok(success)
        })
    }

    /// Clear all memories
    ///
    /// Returns:
    ///     int: Number of memories deleted
    fn clear<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let inner = Arc::clone(&self.inner);

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let memory = {
                let guard = inner.read();
                guard.clone()
            };
            let count = memory
                .clear()
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to clear: {}", e)))?;

            Ok(count)
        })
    }
}

/// Convert MemoryItem to Python dict
fn memory_item_to_dict(py: Python, item: &MemoryItem) -> PyResult<&PyDict> {
    let dict = PyDict::new(py);
    dict.set_item("id", &item.id)?;
    dict.set_item("content", &item.content)?;
    dict.set_item("agent_id", &item.agent_id)?;
    dict.set_item("user_id", &item.user_id)?;
    dict.set_item("importance", item.importance)?;
    dict.set_item("memory_type", format!("{:?}", item.memory_type))?;
    dict.set_item("created_at", item.created_at.to_rfc3339())?;
    
    // Convert metadata to Python dict
    let metadata_dict = PyDict::new(py);
    for (key, value) in &item.metadata {
        metadata_dict.set_item(key, value.to_string())?;
    }
    dict.set_item("metadata", metadata_dict)?;

    Ok(dict)
}

/// Python module definition
#[pymodule]
fn agentmem_native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMemory>()?;
    Ok(())
}

