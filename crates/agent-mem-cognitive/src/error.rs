//! Error Types for AgentMem Cognitive
//! 
//! Provides unified error handling with context

use thiserror::Error;

/// Unified error type for AgentMem Cognitive
#[derive(Error, Debug)]
pub enum MemoryError {
    /// Item not found
    #[error("Memory item not found: {id}")]
    NotFound { id: String },
    
    /// Invalid input
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    
    /// Storage error
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    /// Capacity exceeded
    #[error("Capacity exceeded: {tier} tier has {current}/{max} items")]
    CapacityExceeded {
        tier: String,
        current: usize,
        max: usize,
    },
    
    /// Tier invalid transition
    #[error("Invalid tier transition from {from} to {to}")]
    InvalidTierTransition { from: String, to: String },
    
    /// Archive error
    #[error("Archive error: {message}")]
    ArchiveError { message: String },
    
    /// Review error
    #[error("Review error: {message}")]
    ReviewError { message: String },
    
    /// Concurrent access error
    #[error("Concurrent access error")]
    ConcurrentAccess,
    
    /// Internal error
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl MemoryError {
    /// Create a not found error
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound { id: id.into() }
    }
    
    /// Create an invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput { message: msg.into() }
    }
    
    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::StorageError { message: msg.into() }
    }
    
    /// Create a capacity exceeded error
    pub fn capacity_exceeded(tier: impl Into<String>, current: usize, max: usize) -> Self {
        Self::CapacityExceeded {
            tier: tier.into(),
            current,
            max,
        }
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, MemoryError>;
