//! 嵌入式配置管理器
//!
//! 管理嵌入到二进制文件中的配置模板

use super::templates::{ConfigTemplate, ConfigVariables};
use agent_mem_traits::{AgentMemError, Result};
use std::path::Path;
use tracing::{debug, info};

/// 嵌入式配置管理器
pub struct EmbeddedConfigManager {
    template: ConfigTemplate,
    variables: ConfigVariables,
}

impl EmbeddedConfigManager {
    /// 创建新的配置管理器
    pub fn new(template: ConfigTemplate) -> Self {
        Self {
            template,
            variables: ConfigVariables::new(),
        }
    }

    /// 使用默认模板（开发环境）
    pub fn default() -> Self {
        Self::new(ConfigTemplate::Development)
    }

    /// 设置配置变量
    pub fn with_variables(mut self, variables: ConfigVariables) -> Self {
        self.variables = variables;
        self
    }

    /// 添加变量
    pub fn add_variable(mut self, key: String, value: String) -> Self {
        self.variables.add(key, value);
        self
    }

    /// 从环境变量加载
    pub fn from_env() -> Self {
        let template = std::env::var("AGENTMEM_ENV")
            .ok()
            .and_then(|env| ConfigTemplate::from_name(&env))
            .unwrap_or(ConfigTemplate::Development);

        Self {
            template,
            variables: ConfigVariables::from_env(),
        }
    }

    /// 获取配置内容
    pub fn get_config(&self) -> String {
        let template_content = self.template.to_toml();
        self.variables.replace(template_content)
    }

    /// 导出配置到文件
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config = self.get_config();

        // 创建父目录
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AgentMemError::internal_error(format!("创建目录失败: {e}")))?;
        }

        std::fs::write(path.as_ref(), config)
            .map_err(|e| AgentMemError::internal_error(format!("写入配置文件失败: {e}")))?;

        info!("配置已导出到: {:?}", path.as_ref());
        Ok(())
    }

    /// 列出所有可用模板
    pub fn list_templates() -> Vec<(ConfigTemplate, &'static str, &'static str)> {
        ConfigTemplate::all()
            .into_iter()
            .map(|t| (t, t.name(), t.description()))
            .collect()
    }

    /// 打印模板信息
    pub fn print_template_info(&self) {
        println!("配置模板: {}", self.template.name());
        println!("描述: {}", self.template.description());
        println!("\n配置内容:");
        println!("{}", self.get_config());
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        let config = self.get_config();

        // 尝试解析 TOML
        toml::from_str::<toml::Value>(&config)
            .map_err(|e| AgentMemError::internal_error(format!("配置验证失败: {e}")))?;

        debug!("配置验证通过");
        Ok(())
    }
}

impl Default for EmbeddedConfigManager {
    fn default() -> Self {
        Self::new(ConfigTemplate::Development)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_config_manager_creation() {
        let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
        let config = manager.get_config();
        assert!(!config.is_empty());
    }

    #[test]
    fn test_embedded_config_manager_with_variables() {
        let mut vars = ConfigVariables::new();
        vars.add("DATABASE_URL".to_string(), "custom.db".to_string());

        let manager = EmbeddedConfigManager::new(ConfigTemplate::Development).with_variables(vars);

        let config = manager.get_config();
        assert!(!config.is_empty());
    }

    #[test]
    fn test_list_templates() {
        let templates = EmbeddedConfigManager::list_templates();
        assert_eq!(templates.len(), 5);
    }

    #[test]
    fn test_validate_config() {
        let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
        let result = manager.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_to_file() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.toml");

        let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
        let result = manager.export_to_file(&file_path);
        assert!(result.is_ok());

        assert!(file_path.exists());
    }
}
