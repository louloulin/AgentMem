//! 嵌入模型配置

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 嵌入模型配置
/// 扩展了agent-mem-config中的EmbedderConfig，增加了更多配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// 提供商名称 (openai, huggingface, local)
    pub provider: String,

    /// 模型名称
    pub model: String,

    /// API密钥（用于远程提供商）
    pub api_key: Option<String>,

    /// 基础URL（用于自定义端点）
    pub base_url: Option<String>,

    /// 嵌入维度
    pub dimension: usize,

    /// 批处理大小
    pub batch_size: usize,

    /// 请求超时时间（秒）
    pub timeout_seconds: u64,

    /// 最大重试次数
    pub max_retries: u32,

    /// 额外配置参数
    pub extra_params: HashMap<String, String>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key: None,
            base_url: None,
            dimension: 1536,
            batch_size: 100,
            timeout_seconds: 30,
            max_retries: 3,
            extra_params: HashMap::new(),
        }
    }
}

impl EmbeddingConfig {
    /// 创建OpenAI配置
    pub fn openai(api_key: Option<String>) -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-ada-002".to_string(),
            api_key,
            base_url: Some("https://api.openai.com/v1".to_string()),
            dimension: 1536,
            batch_size: 100,
            timeout_seconds: 30,
            max_retries: 3,
            extra_params: HashMap::new(),
        }
    }

    /// 创建OpenAI 3-small配置
    pub fn openai_3_small(api_key: Option<String>) -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key,
            base_url: Some("https://api.openai.com/v1".to_string()),
            dimension: 1536,
            batch_size: 100,
            timeout_seconds: 30,
            max_retries: 3,
            extra_params: HashMap::new(),
        }
    }

    /// 创建OpenAI 3-large配置
    pub fn openai_3_large(api_key: Option<String>) -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-3-large".to_string(),
            api_key,
            base_url: Some("https://api.openai.com/v1".to_string()),
            dimension: 3072,
            batch_size: 100,
            timeout_seconds: 30,
            max_retries: 3,
            extra_params: HashMap::new(),
        }
    }

    /// 创建HuggingFace配置
    pub fn huggingface(model: &str) -> Self {
        Self {
            provider: "huggingface".to_string(),
            model: model.to_string(),
            api_key: None,
            base_url: None,
            dimension: 768, // 默认BERT维度
            batch_size: 32,
            timeout_seconds: 60,
            max_retries: 3,
            extra_params: HashMap::new(),
        }
    }

    /// 创建本地模型配置
    pub fn local(model_path: &str, dimension: usize) -> Self {
        let mut extra_params = HashMap::new();
        extra_params.insert("model_path".to_string(), model_path.to_string());

        Self {
            provider: "local".to_string(),
            model: "local".to_string(),
            api_key: None,
            base_url: None,
            dimension,
            batch_size: 16,
            timeout_seconds: 120,
            max_retries: 1,
            extra_params,
        }
    }

    /// 验证配置
    pub fn validate(&self) -> agent_mem_traits::Result<()> {
        if self.provider.is_empty() {
            return Err(agent_mem_traits::AgentMemError::config_error(
                "Provider cannot be empty",
            ));
        }

        if self.model.is_empty() {
            return Err(agent_mem_traits::AgentMemError::config_error(
                "Model cannot be empty",
            ));
        }

        if self.dimension == 0 {
            return Err(agent_mem_traits::AgentMemError::config_error(
                "Dimension must be greater than 0",
            ));
        }

        if self.batch_size == 0 {
            return Err(agent_mem_traits::AgentMemError::config_error(
                "Batch size must be greater than 0",
            ));
        }

        // 验证提供商特定的配置
        match self.provider.as_str() {
            "openai" => {
                if self.api_key.is_none() {
                    return Err(agent_mem_traits::AgentMemError::config_error(
                        "OpenAI provider requires an API key",
                    ));
                }
            }
            "local" => {
                if !self.extra_params.contains_key("model_path") {
                    return Err(agent_mem_traits::AgentMemError::config_error(
                        "Local provider requires model_path in extra_params",
                    ));
                }
            }
            _ => {} // 其他提供商暂时不需要特殊验证
        }

        Ok(())
    }

    /// 获取模型路径（用于本地模型）
    pub fn get_model_path(&self) -> Option<&str> {
        self.extra_params.get("model_path").map(|s| s.as_str())
    }

    /// 设置额外参数
    pub fn set_extra_param(&mut self, key: String, value: String) {
        self.extra_params.insert(key, value);
    }

    /// 获取额外参数
    pub fn get_extra_param(&self, key: &str) -> Option<&str> {
        self.extra_params.get(key).map(|s| s.as_str())
    }
}

// 注意：转换功能需要在使用时添加agent-mem-config依赖
// 这里暂时注释掉，避免循环依赖

// /// 从agent-mem-config的EmbedderConfig转换
// impl From<agent_mem_config::memory::EmbedderConfig> for EmbeddingConfig {
//     fn from(config: agent_mem_config::memory::EmbedderConfig) -> Self {
//         Self {
//             provider: config.provider,
//             model: config.model,
//             api_key: config.api_key,
//             base_url: config.base_url,
//             dimension: config.dimension,
//             batch_size: 100,
//             timeout_seconds: 30,
//             max_retries: 3,
//             extra_params: HashMap::new(),
//         }
//     }
// }

// /// 转换为agent-mem-config的EmbedderConfig
// impl From<EmbeddingConfig> for agent_mem_config::memory::EmbedderConfig {
//     fn from(config: EmbeddingConfig) -> Self {
//         Self {
//             provider: config.provider,
//             model: config.model,
//             api_key: config.api_key,
//             base_url: config.base_url,
//             dimension: config.dimension,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "text-embedding-ada-002");
        assert_eq!(config.dimension, 1536);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_openai_config() {
        let config = EmbeddingConfig::openai(Some("test-key".to_string()));
        assert_eq!(config.provider, "openai");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.dimension, 1536);
    }

    #[test]
    fn test_openai_3_large_config() {
        let config = EmbeddingConfig::openai_3_large(Some("test-key".to_string()));
        assert_eq!(config.model, "text-embedding-3-large");
        assert_eq!(config.dimension, 3072);
    }

    #[test]
    fn test_huggingface_config() {
        let config = EmbeddingConfig::huggingface("sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(config.provider, "huggingface");
        assert_eq!(config.model, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(config.dimension, 768);
    }

    #[test]
    fn test_local_config() {
        let config = EmbeddingConfig::local("/path/to/model", 384);
        assert_eq!(config.provider, "local");
        assert_eq!(config.dimension, 384);
        assert_eq!(config.get_model_path(), Some("/path/to/model"));
    }

    #[test]
    fn test_validate_config() {
        let mut config = EmbeddingConfig::openai(Some("test-key".to_string()));
        assert!(config.validate().is_ok());

        // 测试无效配置
        config.provider = "".to_string();
        assert!(config.validate().is_err());

        config.provider = "openai".to_string();
        config.api_key = None;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_extra_params() {
        let mut config = EmbeddingConfig::default();
        config.set_extra_param("custom_param".to_string(), "value".to_string());
        assert_eq!(config.get_extra_param("custom_param"), Some("value"));
    }
}
