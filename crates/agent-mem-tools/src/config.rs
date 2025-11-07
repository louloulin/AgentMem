//! AgentMem Tools配置管理
//! 
//! 支持环境变量配置，避免硬编码

use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<ToolsConfig> = OnceLock::new();

/// 工具配置
#[derive(Debug, Clone)]
pub struct ToolsConfig {
    /// AgentMem后端API URL
    pub api_url: String,
    
    /// API超时时间（秒）
    pub timeout: u64,
    
    /// 重试次数
    pub max_retries: u32,
    
    /// 默认Agent ID
    pub default_agent_id: String,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            api_url: std::env::var("AGENTMEM_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()),
            timeout: std::env::var("AGENTMEM_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_retries: std::env::var("AGENTMEM_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            default_agent_id: std::env::var("AGENTMEM_DEFAULT_AGENT_ID")
                .unwrap_or_else(|_| "agent-default".to_string()),
        }
    }
}

impl ToolsConfig {
    /// 获取全局配置（懒加载）
    pub fn global() -> &'static Self {
        GLOBAL_CONFIG.get_or_init(Self::default)
    }
}

/// 获取配置（快捷函数）
pub fn get_config() -> &'static ToolsConfig {
    ToolsConfig::global()
}

/// 获取API URL（快捷函数）
pub fn get_api_url() -> String {
    get_config().api_url.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ToolsConfig::default();
        assert!(!config.api_url.is_empty());
        assert!(config.timeout > 0);
    }
    
    #[test]
    fn test_global_config() {
        let config1 = ToolsConfig::global();
        let config2 = ToolsConfig::global();
        assert_eq!(config1.api_url, config2.api_url);
    }
}

