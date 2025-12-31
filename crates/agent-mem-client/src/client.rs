//! HTTP client implementation

use crate::{
    config::ClientConfig,
    error::{ClientError, ClientResult},
    models::*,
    retry::{RetryExecutor, RetryPolicy},
};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;

use tracing::{debug, error};
use url::Url;

/// Asynchronous AgentMem client
pub struct AsyncAgentMemClient {
    client: Client,
    config: ClientConfig,
    retry_executor: RetryExecutor,
}

impl AsyncAgentMemClient {
    /// Create a new async client
    pub fn new(config: ClientConfig) -> ClientResult<Self> {
        // Validate configuration
        config.validate().map_err(ClientError::ConfigError)?;

        // Create HTTP client
        let client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(&config.user_agent)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .pool_idle_timeout(config.pool_idle_timeout)
            .build()
            .map_err(ClientError::HttpError)?;

        // Create retry policy
        let retry_policy = RetryPolicy::new(config.max_retries)
            .with_base_delay(config.retry_base_delay)
            .with_max_delay(config.retry_max_delay);

        let retry_executor = RetryExecutor::new(retry_policy);

        Ok(Self {
            client,
            config,
            retry_executor,
        })
    }

    /// Add a new memory
    pub async fn add_memory(&self, request: AddMemoryRequest) -> ClientResult<MemoryResponse> {
        let url = self.build_url("/api/v1/memories")?;

        self.retry_executor
            .execute(|| async {
                let response = self.client.post(&url).json(&request).send().await?;

                self.handle_response(response).await
            })
            .await
    }

    /// Get a memory by ID
    pub async fn get_memory(&self, memory_id: &str) -> ClientResult<Memory> {
        let url = self.build_url(&format!("/api/v1/memories/{memory_id}"))?;

        self.retry_executor
            .execute(|| async {
                let response = self.client.get(&url).send().await?;
                self.handle_response(response).await
            })
            .await
    }

    /// Search memories
    pub async fn search_memories(
        &self,
        request: SearchMemoriesRequest,
    ) -> ClientResult<SearchMemoriesResponse> {
        let url = self.build_url("/api/v1/memories/search")?;

        self.retry_executor
            .execute(|| async {
                let response = self.client.post(&url).json(&request).send().await?;

                self.handle_response(response).await
            })
            .await
    }

    /// Get health status
    pub async fn health_check(&self) -> ClientResult<HealthResponse> {
        let url = self.build_url("/health")?;

        self.retry_executor
            .execute(|| async {
                let response = self.client.get(&url).send().await?;
                self.handle_response(response).await
            })
            .await
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> ClientResult<MetricsResponse> {
        let url = self.build_url("/metrics")?;

        self.retry_executor
            .execute(|| async {
                let response = self.client.get(&url).send().await?;
                self.handle_response(response).await
            })
            .await
    }

    /// Build full URL from path
    fn build_url(&self, path: &str) -> ClientResult<String> {
        let base_url = Url::parse(&self.config.base_url)?;
        let full_url = base_url.join(path)?;
        Ok(full_url.to_string())
    }

    /// Handle HTTP response and deserialize JSON
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> ClientResult<T> {
        let status = response.status();

        if self.config.enable_logging {
            debug!("HTTP response: {} {}", status, response.url());
        }

        if status.is_success() {
            let body = response.text().await?;

            if self.config.enable_logging {
                debug!("Response body: {}", body);
            }

            serde_json::from_str(&body).map_err(|e| {
                error!("Failed to deserialize response: {}", e);
                ClientError::InvalidResponse(format!("JSON deserialization failed: {e}"))
            })
        } else {
            let body = response.text().await.unwrap_or_default();

            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&body) {
                error!(
                    "Server error: {} - {}",
                    error_response.code, error_response.message
                );
                Err(ClientError::ServerError {
                    status: status.as_u16(),
                    message: error_response.message,
                })
            } else {
                error!("HTTP error {}: {}", status, body);
                Err(ClientError::ServerError {
                    status: status.as_u16(),
                    message: body,
                })
            }
        }
    }
}

/// Synchronous AgentMem client (wrapper around async client)
pub struct AgentMemClient {
    async_client: AsyncAgentMemClient,
    runtime: tokio::runtime::Runtime,
}

impl AgentMemClient {
    /// Create a new sync client
    pub fn new(config: ClientConfig) -> ClientResult<Self> {
        let async_client = AsyncAgentMemClient::new(config)?;
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| ClientError::InternalError(format!("Failed to create runtime: {e}")))?;

        Ok(Self {
            async_client,
            runtime,
        })
    }

    /// Add a new memory (sync)
    pub fn add_memory(&self, request: AddMemoryRequest) -> ClientResult<MemoryResponse> {
        self.runtime.block_on(self.async_client.add_memory(request))
    }

    /// Get a memory by ID (sync)
    pub fn get_memory(&self, memory_id: &str) -> ClientResult<Memory> {
        self.runtime
            .block_on(self.async_client.get_memory(memory_id))
    }

    /// Search memories (sync)
    pub fn search_memories(
        &self,
        request: SearchMemoriesRequest,
    ) -> ClientResult<SearchMemoriesResponse> {
        self.runtime
            .block_on(self.async_client.search_memories(request))
    }

    /// Get health status (sync)
    pub fn health_check(&self) -> ClientResult<HealthResponse> {
        self.runtime.block_on(self.async_client.health_check())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_client_creation() {
        let config = ClientConfig::new("http://localhost:8080");
        let client = AsyncAgentMemClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_sync_client_creation() {
        let config = ClientConfig::new("http://localhost:8080");
        let client = AgentMemClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_url_building() {
        let config = ClientConfig::new("http://localhost:8080");
        let client = AsyncAgentMemClient::new(config).unwrap();

        let url = client.build_url("/api/v1/memories").unwrap();
        assert_eq!(url, "http://localhost:8080/api/v1/memories");

        let url = client.build_url("/health").unwrap();
        assert_eq!(url, "http://localhost:8080/health");
    }
}
