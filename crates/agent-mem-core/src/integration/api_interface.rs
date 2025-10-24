use serde::{Deserialize, Serialize};
/// 统一API接口
///
/// 提供统一的记忆操作API，简化客户端使用
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::SystemIntegrationManager;
use agent_mem_traits::{AgentMemError, MemoryItem as Memory, Result};

/// API请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// 请求ID
    pub request_id: Uuid,
    /// 操作类型
    pub operation: ApiOperation,
    /// 请求时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 客户端信息
    pub client_info: Option<ClientInfo>,
}

/// API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 请求ID
    pub request_id: Uuid,
    /// 响应状态
    pub status: ApiStatus,
    /// 响应数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
    /// 响应时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 处理时间 (ms)
    pub processing_time_ms: u64,
}

/// API操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiOperation {
    /// 存储记忆
    StoreMemory { memory: Memory },
    /// 检索记忆
    RetrieveMemory { memory_id: Uuid },
    /// 搜索记忆
    SearchMemories { query: String, limit: Option<usize> },
    /// 删除记忆
    DeleteMemory { memory_id: Uuid },
    /// 获取系统状态
    GetSystemStatus,
    /// 获取健康状态
    GetHealthStatus,
    /// 获取统计信息
    GetStatistics,
    /// 获取配置
    GetConfiguration,
    /// 更新配置
    UpdateConfiguration { config: super::SystemConfig },
}

/// API状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiStatus {
    /// 成功
    Success,
    /// 失败
    Error,
    /// 部分成功
    PartialSuccess,
    /// 处理中
    Processing,
}

/// 客户端信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// 客户端ID
    pub client_id: String,
    /// 客户端版本
    pub version: String,
    /// 用户代理
    pub user_agent: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
}

/// 批量操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// 批量请求ID
    pub batch_id: Uuid,
    /// 操作列表
    pub operations: Vec<ApiOperation>,
    /// 是否事务性
    pub transactional: bool,
    /// 客户端信息
    pub client_info: Option<ClientInfo>,
}

/// 批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// 批量请求ID
    pub batch_id: Uuid,
    /// 响应列表
    pub responses: Vec<ApiResponse<serde_json::Value>>,
    /// 整体状态
    pub overall_status: ApiStatus,
    /// 成功数量
    pub success_count: usize,
    /// 失败数量
    pub error_count: usize,
}

/// 统一API接口
pub struct UnifiedApiInterface {
    /// 系统集成管理器
    system_manager: Arc<SystemIntegrationManager>,
}

impl UnifiedApiInterface {
    /// 创建新的API接口
    pub fn new(system_manager: Arc<SystemIntegrationManager>) -> Self {
        Self { system_manager }
    }

    /// 处理API请求
    pub async fn handle_request(&self, request: ApiRequest) -> ApiResponse<serde_json::Value> {
        let start_time = std::time::Instant::now();
        let request_id = request.request_id;

        let (status, data, error) = match request.operation {
            ApiOperation::StoreMemory { memory } => {
                match self.system_manager.store_memory(memory).await {
                    Ok(memory_id) => (
                        ApiStatus::Success,
                        Some(serde_json::to_value(memory_id).unwrap()),
                        None,
                    ),
                    Err(e) => (ApiStatus::Error, None, Some(e.to_string())),
                }
            }

            ApiOperation::RetrieveMemory { memory_id } => {
                match self.system_manager.retrieve_memory(memory_id).await {
                    Ok(memory) => (
                        ApiStatus::Success,
                        Some(serde_json::to_value(memory).unwrap()),
                        None,
                    ),
                    Err(e) => (ApiStatus::Error, None, Some(e.to_string())),
                }
            }

            ApiOperation::SearchMemories { query, limit } => {
                match self.system_manager.search_memories(&query, limit).await {
                    Ok(memories) => (
                        ApiStatus::Success,
                        Some(serde_json::to_value(memories).unwrap()),
                        None,
                    ),
                    Err(e) => (ApiStatus::Error, None, Some(e.to_string())),
                }
            }

            ApiOperation::DeleteMemory { memory_id } => {
                match self.system_manager.delete_memory(memory_id).await {
                    Ok(deleted) => (
                        ApiStatus::Success,
                        Some(serde_json::to_value(deleted).unwrap()),
                        None,
                    ),
                    Err(e) => (ApiStatus::Error, None, Some(e.to_string())),
                }
            }

            ApiOperation::GetSystemStatus => {
                let status = self.system_manager.get_status().await;
                (
                    ApiStatus::Success,
                    Some(serde_json::to_value(status).unwrap()),
                    None,
                )
            }

            ApiOperation::GetHealthStatus => {
                match self.system_manager.perform_health_check().await {
                    Ok(health) => (
                        ApiStatus::Success,
                        Some(serde_json::to_value(health).unwrap()),
                        None,
                    ),
                    Err(e) => (ApiStatus::Error, None, Some(e.to_string())),
                }
            }

            ApiOperation::GetStatistics => {
                match self.system_manager.get_system_statistics().await {
                    Ok(stats) => (
                        ApiStatus::Success,
                        Some(serde_json::to_value(stats).unwrap()),
                        None,
                    ),
                    Err(e) => (ApiStatus::Error, None, Some(e.to_string())),
                }
            }

            ApiOperation::GetConfiguration => {
                let config = self.system_manager.get_config();
                (
                    ApiStatus::Success,
                    Some(serde_json::to_value(config).unwrap()),
                    None,
                )
            }

            ApiOperation::UpdateConfiguration { config } => {
                // 注意：这里需要可变引用，实际实现中可能需要调整架构
                (
                    ApiStatus::Error,
                    None,
                    Some("配置更新需要可变引用".to_string()),
                )
            }
        };

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        ApiResponse {
            request_id,
            status,
            data,
            error,
            timestamp: chrono::Utc::now(),
            processing_time_ms,
        }
    }

    /// 处理批量请求
    pub async fn handle_batch_request(&self, batch_request: BatchRequest) -> BatchResponse {
        let batch_id = batch_request.batch_id;
        let mut responses = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        for operation in batch_request.operations {
            let request = ApiRequest {
                request_id: Uuid::new_v4(),
                operation,
                timestamp: chrono::Utc::now(),
                client_info: batch_request.client_info.clone(),
            };

            let response = self.handle_request(request).await;

            if response.status == ApiStatus::Success {
                success_count += 1;
            } else {
                error_count += 1;
            }

            responses.push(response);

            // 如果是事务性操作且有失败，则回滚
            if batch_request.transactional && error_count > 0 {
                // 这里应该实现回滚逻辑
                break;
            }
        }

        let overall_status = if error_count == 0 {
            ApiStatus::Success
        } else if success_count == 0 {
            ApiStatus::Error
        } else {
            ApiStatus::PartialSuccess
        };

        BatchResponse {
            batch_id,
            responses,
            overall_status,
            success_count,
            error_count,
        }
    }

    /// 验证请求
    pub fn validate_request(&self, request: &ApiRequest) -> Result<()> {
        // 基本验证
        match &request.operation {
            ApiOperation::StoreMemory { memory } => {
                if memory.content.is_empty() {
                    return Err(AgentMemError::InvalidInput("记忆内容不能为空".to_string()));
                }
            }
            ApiOperation::SearchMemories { query, .. } => {
                if query.is_empty() {
                    return Err(AgentMemError::InvalidInput("搜索查询不能为空".to_string()));
                }
            }
            _ => {} // 其他操作暂不验证
        }

        Ok(())
    }

    /// 获取API使用统计
    pub async fn get_api_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // 获取系统统计
        if let Ok(system_stats) = self.system_manager.get_system_statistics().await {
            stats.extend(system_stats);
        }

        // 添加API特定统计
        stats.insert(
            "api_version".to_string(),
            serde_json::Value::String("1.0.0".to_string()),
        );
        stats.insert(
            "supported_operations".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("store_memory".to_string()),
                serde_json::Value::String("retrieve_memory".to_string()),
                serde_json::Value::String("search_memories".to_string()),
                serde_json::Value::String("delete_memory".to_string()),
                serde_json::Value::String("get_system_status".to_string()),
                serde_json::Value::String("get_health_status".to_string()),
                serde_json::Value::String("get_statistics".to_string()),
            ]),
        );

        stats
    }

    /// 创建标准化错误响应
    pub fn create_error_response(
        request_id: Uuid,
        error: AgentMemError,
    ) -> ApiResponse<serde_json::Value> {
        ApiResponse {
            request_id,
            status: ApiStatus::Error,
            data: None,
            error: Some(error.to_string()),
            timestamp: chrono::Utc::now(),
            processing_time_ms: 0,
        }
    }

    /// 创建成功响应
    pub fn create_success_response<T: Serialize>(
        request_id: Uuid,
        data: T,
        processing_time_ms: u64,
    ) -> ApiResponse<serde_json::Value> {
        ApiResponse {
            request_id,
            status: ApiStatus::Success,
            data: Some(serde_json::to_value(data).unwrap()),
            error: None,
            timestamp: chrono::Utc::now(),
            processing_time_ms,
        }
    }
}

/// API辅助函数
impl UnifiedApiInterface {
    /// 记录API调用
    pub async fn log_api_call(
        &self,
        request: &ApiRequest,
        response: &ApiResponse<serde_json::Value>,
    ) {
        // 记录API调用日志
        println!(
            "API调用: {} - {} - {}ms - {}",
            request.request_id,
            match &request.operation {
                ApiOperation::StoreMemory { .. } => "store_memory",
                ApiOperation::RetrieveMemory { .. } => "retrieve_memory",
                ApiOperation::SearchMemories { .. } => "search_memories",
                ApiOperation::DeleteMemory { .. } => "delete_memory",
                ApiOperation::GetSystemStatus => "get_system_status",
                ApiOperation::GetHealthStatus => "get_health_status",
                ApiOperation::GetStatistics => "get_statistics",
                ApiOperation::GetConfiguration => "get_configuration",
                ApiOperation::UpdateConfiguration { .. } => "update_configuration",
            },
            response.processing_time_ms,
            if response.status == ApiStatus::Success {
                "成功"
            } else {
                "失败"
            }
        );
    }

    /// 获取支持的操作列表
    pub fn get_supported_operations(&self) -> Vec<String> {
        vec![
            "store_memory".to_string(),
            "retrieve_memory".to_string(),
            "search_memories".to_string(),
            "delete_memory".to_string(),
            "get_system_status".to_string(),
            "get_health_status".to_string(),
            "get_statistics".to_string(),
            "get_configuration".to_string(),
            "update_configuration".to_string(),
        ]
    }
}
