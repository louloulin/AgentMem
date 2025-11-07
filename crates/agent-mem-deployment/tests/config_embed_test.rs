//! 配置嵌入测试

use agent_mem_deployment::config_embed::templates::ConfigVariables;
use agent_mem_deployment::config_embed::{ConfigTemplate, EmbeddedConfigManager};

#[test]
fn test_config_template_all() {
    let templates = ConfigTemplate::all();
    assert_eq!(templates.len(), 5);
}

#[test]
fn test_config_template_name() {
    assert_eq!(ConfigTemplate::Development.name(), "development");
    assert_eq!(ConfigTemplate::Production.name(), "production");
    assert_eq!(ConfigTemplate::Testing.name(), "testing");
    assert_eq!(ConfigTemplate::Minimal.name(), "minimal");
    assert_eq!(ConfigTemplate::Full.name(), "full");
}

#[test]
fn test_config_template_description() {
    let desc = ConfigTemplate::Development.description();
    assert!(!desc.is_empty());
}

#[test]
fn test_config_template_to_toml() {
    let toml = ConfigTemplate::Development.to_toml();
    assert!(!toml.is_empty());
    assert!(toml.contains("data_root"));
}

#[test]
fn test_config_template_from_name() {
    assert_eq!(
        ConfigTemplate::from_name("dev"),
        Some(ConfigTemplate::Development)
    );
    assert_eq!(
        ConfigTemplate::from_name("development"),
        Some(ConfigTemplate::Development)
    );
    assert_eq!(
        ConfigTemplate::from_name("prod"),
        Some(ConfigTemplate::Production)
    );
    assert_eq!(
        ConfigTemplate::from_name("production"),
        Some(ConfigTemplate::Production)
    );
    assert_eq!(
        ConfigTemplate::from_name("test"),
        Some(ConfigTemplate::Testing)
    );
    assert_eq!(
        ConfigTemplate::from_name("min"),
        Some(ConfigTemplate::Minimal)
    );
    assert_eq!(
        ConfigTemplate::from_name("full"),
        Some(ConfigTemplate::Full)
    );
    assert_eq!(ConfigTemplate::from_name("invalid"), None);
}

#[test]
fn test_config_variables_creation() {
    let vars = ConfigVariables::new();
    let template = "test";
    let result = vars.replace(template);
    assert_eq!(result, "test");
}

#[test]
fn test_config_variables_add() {
    let mut vars = ConfigVariables::new();
    vars.add("KEY".to_string(), "value".to_string());

    let template = "${KEY}";
    let result = vars.replace(template);
    assert_eq!(result, "value");
}

#[test]
fn test_config_variables_replace() {
    let mut vars = ConfigVariables::new();
    vars.add("DATABASE_URL".to_string(), "test.db".to_string());
    vars.add("PORT".to_string(), "8080".to_string());

    let template = "url = \"${DATABASE_URL}\"\nport = ${PORT}";
    let result = vars.replace(template);
    assert_eq!(result, "url = \"test.db\"\nport = 8080");
}

#[test]
fn test_embedded_config_manager_creation() {
    let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
    let config = manager.get_config();
    assert!(!config.is_empty());
}

#[test]
fn test_embedded_config_manager_default() {
    let manager = EmbeddedConfigManager::default();
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
fn test_embedded_config_manager_add_variable() {
    let manager = EmbeddedConfigManager::new(ConfigTemplate::Development)
        .add_variable("KEY".to_string(), "value".to_string());

    let config = manager.get_config();
    assert!(!config.is_empty());
}

#[test]
fn test_embedded_config_manager_list_templates() {
    let templates = EmbeddedConfigManager::list_templates();
    assert_eq!(templates.len(), 5);

    for (template, name, desc) in templates {
        assert_eq!(template.name(), name);
        assert_eq!(template.description(), desc);
    }
}

#[test]
fn test_embedded_config_manager_validate() {
    let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
    let result = manager.validate();
    assert!(result.is_ok());
}

#[test]
fn test_embedded_config_manager_export_to_file() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config.toml");

    let manager = EmbeddedConfigManager::new(ConfigTemplate::Development);
    let result = manager.export_to_file(&file_path);
    assert!(result.is_ok());

    assert!(file_path.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("data_root"));
}

#[test]
fn test_all_templates_valid() {
    for template in ConfigTemplate::all() {
        let manager = EmbeddedConfigManager::new(template);
        let result = manager.validate();
        assert!(result.is_ok(), "模板 {} 验证失败", template.name());
    }
}
