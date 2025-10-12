//! 配置模板
//!
//! 提供默认配置模板，嵌入到二进制文件中

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 配置模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigTemplate {
    /// 开发环境配置
    Development,
    /// 生产环境配置
    Production,
    /// 测试环境配置
    Testing,
    /// 最小化配置
    Minimal,
    /// 完整功能配置
    Full,
}

impl ConfigTemplate {
    /// 获取所有模板
    pub fn all() -> Vec<ConfigTemplate> {
        vec![
            ConfigTemplate::Development,
            ConfigTemplate::Production,
            ConfigTemplate::Testing,
            ConfigTemplate::Minimal,
            ConfigTemplate::Full,
        ]
    }
    
    /// 获取模板名称
    pub fn name(&self) -> &'static str {
        match self {
            ConfigTemplate::Development => "development",
            ConfigTemplate::Production => "production",
            ConfigTemplate::Testing => "testing",
            ConfigTemplate::Minimal => "minimal",
            ConfigTemplate::Full => "full",
        }
    }
    
    /// 获取模板描述
    pub fn description(&self) -> &'static str {
        match self {
            ConfigTemplate::Development => "开发环境配置，启用调试功能",
            ConfigTemplate::Production => "生产环境配置，优化性能和安全性",
            ConfigTemplate::Testing => "测试环境配置，使用内存存储",
            ConfigTemplate::Minimal => "最小化配置，仅启用核心功能",
            ConfigTemplate::Full => "完整功能配置，启用所有功能",
        }
    }
    
    /// 获取模板内容（TOML 格式）
    pub fn to_toml(&self) -> &'static str {
        match self {
            ConfigTemplate::Development => include_str!("../../templates/config.dev.toml"),
            ConfigTemplate::Production => include_str!("../../templates/config.prod.toml"),
            ConfigTemplate::Testing => include_str!("../../templates/config.test.toml"),
            ConfigTemplate::Minimal => include_str!("../../templates/config.minimal.toml"),
            ConfigTemplate::Full => include_str!("../../templates/config.full.toml"),
        }
    }
    
    /// 从名称解析模板
    pub fn from_name(name: &str) -> Option<ConfigTemplate> {
        match name.to_lowercase().as_str() {
            "development" | "dev" => Some(ConfigTemplate::Development),
            "production" | "prod" => Some(ConfigTemplate::Production),
            "testing" | "test" => Some(ConfigTemplate::Testing),
            "minimal" | "min" => Some(ConfigTemplate::Minimal),
            "full" => Some(ConfigTemplate::Full),
            _ => None,
        }
    }
}

/// 配置变量替换
pub struct ConfigVariables {
    variables: HashMap<String, String>,
}

impl ConfigVariables {
    /// 创建新的配置变量
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    
    /// 添加变量
    pub fn add(&mut self, key: String, value: String) -> &mut Self {
        self.variables.insert(key, value);
        self
    }
    
    /// 从环境变量加载
    pub fn from_env() -> Self {
        let mut variables = HashMap::new();
        
        // 常用环境变量
        if let Ok(val) = std::env::var("DATABASE_URL") {
            variables.insert("DATABASE_URL".to_string(), val);
        }
        if let Ok(val) = std::env::var("VECTOR_DIMENSION") {
            variables.insert("VECTOR_DIMENSION".to_string(), val);
        }
        if let Ok(val) = std::env::var("DATA_ROOT") {
            variables.insert("DATA_ROOT".to_string(), val);
        }
        if let Ok(val) = std::env::var("LOG_LEVEL") {
            variables.insert("LOG_LEVEL".to_string(), val);
        }
        
        Self { variables }
    }
    
    /// 替换模板中的变量
    pub fn replace(&self, template: &str) -> String {
        let mut result = template.to_string();
        
        for (key, value) in &self.variables {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
}

impl Default for ConfigVariables {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_template_all() {
        let templates = ConfigTemplate::all();
        assert_eq!(templates.len(), 5);
    }
    
    #[test]
    fn test_config_template_name() {
        assert_eq!(ConfigTemplate::Development.name(), "development");
        assert_eq!(ConfigTemplate::Production.name(), "production");
    }
    
    #[test]
    fn test_config_template_from_name() {
        assert_eq!(
            ConfigTemplate::from_name("dev"),
            Some(ConfigTemplate::Development)
        );
        assert_eq!(
            ConfigTemplate::from_name("production"),
            Some(ConfigTemplate::Production)
        );
        assert_eq!(ConfigTemplate::from_name("invalid"), None);
    }
    
    #[test]
    fn test_config_variables() {
        let mut vars = ConfigVariables::new();
        vars.add("DATABASE_URL".to_string(), "test.db".to_string());
        
        let template = "url = \"${DATABASE_URL}\"";
        let result = vars.replace(template);
        assert_eq!(result, "url = \"test.db\"");
    }
}

