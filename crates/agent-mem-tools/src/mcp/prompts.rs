//! MCP Prompts 支持
//!
//! 提供 MCP 协议的 Prompts 功能，用于管理和使用提示词模板。
//!
//! 主要功能：
//! - 提示词定义和管理
//! - 提示词参数系统
//! - 提示词模板引擎
//! - 提示词版本管理

use super::error::{McpError, McpResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP 提示词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    /// 提示词名称
    pub name: String,
    
    /// 提示词描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// 提示词参数
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub arguments: Vec<PromptArgument>,
    
    /// 提示词内容
    pub content: Vec<PromptContent>,
    
    /// 提示词版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// 提示词标签
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    
    /// 元数据
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl McpPrompt {
    /// 创建新的提示词
    pub fn new(name: impl Into<String>, content: Vec<PromptContent>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
            content,
            version: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 添加参数
    pub fn with_argument(mut self, argument: PromptArgument) -> Self {
        self.arguments.push(argument);
        self
    }
    
    /// 设置版本
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
    
    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// 提示词参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// 参数名称
    pub name: String,
    
    /// 参数描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    
    /// 参数类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arg_type: Option<String>,
    
    /// 默认值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

impl PromptArgument {
    /// 创建新的参数
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: false,
            arg_type: None,
            default: None,
        }
    }
    
    /// 设置为必需
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 设置类型
    pub fn with_type(mut self, arg_type: impl Into<String>) -> Self {
        self.arg_type = Some(arg_type.into());
        self
    }
}

/// 提示词内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    /// 文本内容
    #[serde(rename = "text")]
    Text {
        /// 文本内容
        text: String,
    },
    
    /// 图片内容
    #[serde(rename = "image")]
    Image {
        /// 图片 URL 或 base64 数据
        data: String,
        
        /// MIME 类型
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
    
    /// 资源引用
    #[serde(rename = "resource")]
    Resource {
        /// 资源 URI
        uri: String,
        
        /// MIME 类型
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
}

/// MCP 列出提示词响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpListPromptsResponse {
    /// 提示词列表
    pub prompts: Vec<McpPrompt>,
}

/// MCP 获取提示词请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpGetPromptRequest {
    /// 提示词名称
    pub name: String,
    
    /// 提示词参数
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

/// MCP 获取提示词响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpGetPromptResponse {
    /// 提示词内容
    pub content: Vec<PromptContent>,
    
    /// 提示词描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 提示词管理器
pub struct PromptManager {
    /// 提示词存储
    prompts: Arc<RwLock<HashMap<String, McpPrompt>>>,
    
    /// 模板引擎（使用 Handlebars）
    template_engine: Arc<RwLock<handlebars::Handlebars<'static>>>,
}

impl PromptManager {
    /// 创建新的提示词管理器
    pub fn new() -> Self {
        let mut hb = handlebars::Handlebars::new();
        hb.set_strict_mode(false);
        
        Self {
            prompts: Arc::new(RwLock::new(HashMap::new())),
            template_engine: Arc::new(RwLock::new(hb)),
        }
    }
    
    /// 注册提示词
    pub async fn register_prompt(&self, prompt: McpPrompt) -> McpResult<()> {
        let name = prompt.name.clone();
        info!("Registering prompt: {}", name);
        
        // 注册模板
        let mut engine = self.template_engine.write().await;
        for (idx, content) in prompt.content.iter().enumerate() {
            if let PromptContent::Text { text } = content {
                let template_name = format!("{name}_{idx}");
                engine
                    .register_template_string(&template_name, text)
                    .map_err(|e| McpError::Internal(format!("Failed to register template: {e}")))?;
            }
        }
        drop(engine);
        
        // 存储提示词
        let mut prompts = self.prompts.write().await;
        prompts.insert(name.clone(), prompt);
        
        debug!("Prompt registered: {}", name);
        Ok(())
    }
    
    /// 列出所有提示词
    pub async fn list_prompts(&self) -> McpResult<Vec<McpPrompt>> {
        info!("Listing all prompts");
        let prompts = self.prompts.read().await;
        let result: Vec<McpPrompt> = prompts.values().cloned().collect();
        debug!("Found {} prompts", result.len());
        Ok(result)
    }
    
    /// 获取提示词
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, serde_json::Value>,
    ) -> McpResult<McpGetPromptResponse> {
        info!("Getting prompt: {}", name);
        
        // 查找提示词
        let prompts = self.prompts.read().await;
        let prompt = prompts
            .get(name)
            .ok_or_else(|| McpError::Internal(format!("Prompt not found: {name}")))?
            .clone();
        drop(prompts);
        
        // 验证必需参数
        for arg in &prompt.arguments {
            if arg.required && !arguments.contains_key(&arg.name) {
                return Err(McpError::Internal(format!(
                    "Missing required argument: {}",
                    arg.name
                )));
            }
        }
        
        // 渲染模板
        let engine = self.template_engine.read().await;
        let mut rendered_content = Vec::new();
        
        for (idx, content) in prompt.content.iter().enumerate() {
            match content {
                PromptContent::Text { .. } => {
                    let template_name = format!("{name}_{idx}");
                    let rendered = engine
                        .render(&template_name, &arguments)
                        .map_err(|e| McpError::Internal(format!("Failed to render template: {e}")))?;
                    
                    rendered_content.push(PromptContent::Text { text: rendered });
                }
                other => {
                    rendered_content.push(other.clone());
                }
            }
        }
        
        Ok(McpGetPromptResponse {
            content: rendered_content,
            description: prompt.description,
        })
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_register_prompt() {
        let manager = PromptManager::new();

        let prompt = McpPrompt::new(
            "test_prompt",
            vec![PromptContent::Text {
                text: "Hello, {{name}}!".to_string(),
            }],
        )
        .with_description("A test prompt")
        .with_argument(
            PromptArgument::new("name")
                .required()
                .with_type("string")
                .with_description("User name"),
        );

        let result = manager.register_prompt(prompt).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_prompts() {
        let manager = PromptManager::new();

        // 注册多个提示词
        for i in 0..3 {
            let prompt = McpPrompt::new(
                format!("prompt_{i}"),
                vec![PromptContent::Text {
                    text: format!("Prompt {i}"),
                }],
            );
            manager.register_prompt(prompt).await.unwrap();
        }

        let prompts = manager.list_prompts().await.unwrap();
        assert_eq!(prompts.len(), 3);
    }

    #[tokio::test]
    async fn test_get_prompt_with_arguments() {
        let manager = PromptManager::new();

        let prompt = McpPrompt::new(
            "greeting",
            vec![PromptContent::Text {
                text: "Hello, {{name}}! You are {{age}} years old.".to_string(),
            }],
        )
        .with_argument(PromptArgument::new("name").required())
        .with_argument(PromptArgument::new("age").required());

        manager.register_prompt(prompt).await.unwrap();

        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("Alice"));
        args.insert("age".to_string(), json!(30));

        let response = manager.get_prompt("greeting", args).await.unwrap();

        assert_eq!(response.content.len(), 1);
        if let PromptContent::Text { text } = &response.content[0] {
            assert_eq!(text, "Hello, Alice! You are 30 years old.");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_get_prompt_missing_required_argument() {
        let manager = PromptManager::new();

        let prompt = McpPrompt::new(
            "test",
            vec![PromptContent::Text {
                text: "Hello, {{name}}!".to_string(),
            }],
        )
        .with_argument(PromptArgument::new("name").required());

        manager.register_prompt(prompt).await.unwrap();

        let args = HashMap::new();
        let result = manager.get_prompt("test", args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_nonexistent_prompt() {
        let manager = PromptManager::new();

        let args = HashMap::new();
        let result = manager.get_prompt("nonexistent", args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prompt_with_multiple_content() {
        let manager = PromptManager::new();

        let prompt = McpPrompt::new(
            "multi_content",
            vec![
                PromptContent::Text {
                    text: "Part 1: {{part1}}".to_string(),
                },
                PromptContent::Text {
                    text: "Part 2: {{part2}}".to_string(),
                },
                PromptContent::Resource {
                    uri: "agentmem://memory/core".to_string(),
                    mime_type: Some("application/json".to_string()),
                },
            ],
        )
        .with_argument(PromptArgument::new("part1"))
        .with_argument(PromptArgument::new("part2"));

        manager.register_prompt(prompt).await.unwrap();

        let mut args = HashMap::new();
        args.insert("part1".to_string(), json!("First"));
        args.insert("part2".to_string(), json!("Second"));

        let response = manager.get_prompt("multi_content", args).await.unwrap();

        assert_eq!(response.content.len(), 3);

        if let PromptContent::Text { text } = &response.content[0] {
            assert_eq!(text, "Part 1: First");
        } else {
            panic!("Expected text content");
        }

        if let PromptContent::Text { text } = &response.content[1] {
            assert_eq!(text, "Part 2: Second");
        } else {
            panic!("Expected text content");
        }

        if let PromptContent::Resource { uri, .. } = &response.content[2] {
            assert_eq!(uri, "agentmem://memory/core");
        } else {
            panic!("Expected resource content");
        }
    }

    #[tokio::test]
    async fn test_prompt_builder() {
        let prompt = McpPrompt::new(
            "builder_test",
            vec![PromptContent::Text {
                text: "Test".to_string(),
            }],
        )
        .with_description("Test description")
        .with_version("1.0.0")
        .with_tag("test")
        .with_tag("example")
        .with_argument(PromptArgument::new("arg1").required());

        assert_eq!(prompt.name, "builder_test");
        assert_eq!(prompt.description, Some("Test description".to_string()));
        assert_eq!(prompt.version, Some("1.0.0".to_string()));
        assert_eq!(prompt.tags.len(), 2);
        assert_eq!(prompt.arguments.len(), 1);
    }
}

