//! Python bindings for AgentMem
//!
//! This crate provides Python bindings for AgentMem using PyO3.

use agent_mem::{AddResult, DeleteAllOptions, GetAllOptions, Memory as RustMemory};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::collections::HashMap;

/// Python wrapper for Memory
#[pyclass(name = "Memory")]
struct PyMemory {
    inner: RustMemory, // Memory already implements Clone
}

#[pymethods]
impl PyMemory {
    /// Create a new Memory instance
    #[new]
    fn new() -> PyResult<Self> {
        // Create a new Memory instance
        let inner = pyo3_asyncio::tokio::get_runtime()
            .block_on(async { RustMemory::new().await })
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create Memory: {e}")))?;

        Ok(Self { inner })
    }

    /// Add a memory
    ///
    /// Args:
    ///     content (str): The memory content
    ///
    /// Returns:
    ///     str: Memory ID
    fn add<'py>(&self, py: Python<'py>, content: String) -> PyResult<&'py PyAny> {
        let memory = self.inner.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let result: AddResult = memory
                .add(&content)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add memory: {e}")))?;

            // Return the first memory ID from results
            if let Some(first) = result.results.first() {
                Ok(first.id.clone())
            } else {
                Err(PyRuntimeError::new_err("No memory ID returned"))
            }
        })
    }

    /// Search for memories
    ///
    /// Args:
    ///     query (str): Search query
    ///
    /// Returns:
    ///     list: List of matching memories
    fn search<'py>(&self, py: Python<'py>, query: String) -> PyResult<&'py PyAny> {
        let memory = self.inner.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let results = memory
                .search(&query)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to search: {e}")))?;

            // Convert to Python-friendly format
            let py_results: Vec<HashMap<String, String>> = results
                .into_iter()
                .map(|item| {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), item.id);
                    map.insert("content".to_string(), item.content);
                    map.insert("created_at".to_string(), item.created_at.to_rfc3339());
                    map
                })
                .collect();

            Ok(py_results)
        })
    }

    /// Get all memories
    ///
    /// Returns:
    ///     list: List of all memories
    fn get_all<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let memory = self.inner.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let results = memory
                .get_all(GetAllOptions::default())
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get all: {e}")))?;

            let py_results: Vec<HashMap<String, String>> = results
                .into_iter()
                .map(|item| {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), item.id);
                    map.insert("content".to_string(), item.content);
                    map.insert("created_at".to_string(), item.created_at.to_rfc3339());
                    map
                })
                .collect();

            Ok(py_results)
        })
    }

    /// Delete a memory
    ///
    /// Args:
    ///     memory_id (str): Memory identifier
    ///
    /// Returns:
    ///     bool: Success status
    fn delete<'py>(&self, py: Python<'py>, memory_id: String) -> PyResult<&'py PyAny> {
        let memory = self.inner.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            memory
                .delete(&memory_id)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to delete: {e}")))?;

            Ok(true)
        })
    }

    /// Clear all memories
    ///
    /// Returns:
    ///     int: Number of deleted memories
    fn clear<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let memory = self.inner.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let count = memory
                .delete_all(DeleteAllOptions::default())
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to clear: {e}")))?;

            Ok(count)
        })
    }
}

/// AgentMem Python module
#[pymodule]
fn agentmem_native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMemory>()?;
    Ok(())
}
