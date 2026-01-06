//! AgentMem HTTP Client SDK
//!
//! Enterprise-grade HTTP client for AgentMem memory management platform.
//! Provides both synchronous and asynchronous interfaces with connection pooling,
//! retry logic, and comprehensive error handling.

pub mod client;
pub mod config;
pub mod error;
pub mod mem5_client;
pub mod models;
pub mod retry;

pub use client::{AgentMemClient, AsyncAgentMemClient};
pub use config::ClientConfig;
pub use error::{ClientError, ClientResult};
pub use mem5_client::Mem5Client;
pub use models::*;

/// Re-export commonly used types
pub use agent_mem_core::MemoryType;
pub use agent_mem_traits::AgentMemError;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = ClientConfig::new("http://localhost:8080");
        let client = AsyncAgentMemClient::new(config);
        assert!(client.is_ok());
    }
}
