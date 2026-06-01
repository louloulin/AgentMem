//! AgentMem API Route Definitions
//! 
//! Extracted from routes/memory.rs for better modularity.

/// Memory API routes
pub mod routes {
    use serde::{Deserialize, Serialize};
    
    /// Memory operation response
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MemoryResponse {
        pub id: String,
        pub content: String,
        pub memory_type: String,
    }
    
    /// Add memory request
    #[derive(Debug, Deserialize)]
    pub struct AddMemoryRequest {
        pub content: String,
        pub memory_type: Option<String>,
        pub agent_id: Option<String>,
        pub user_id: Option<String>,
    }
    
    /// Search memory request
    #[derive(Debug, Deserialize)]
    pub struct SearchRequest {
        pub query: String,
        pub limit: Option<usize>,
    }
    
    /// Health check response
    #[derive(Debug, Serialize)]
    pub struct HealthResponse {
        pub status: String,
        pub version: String,
    }
}
