//! Configuration Management for AgentMem Cognitive
//! 
//! Provides YAML configuration loading and environment variable overrides

use crate::unified::UnifiedConfig;
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Environment variable prefix
const ENV_PREFIX: &str = "AGENTMEM_";

/// Configuration manager with YAML and env support
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: UnifiedConfig,
    env_overrides: HashMap<String, String>,
}

impl ConfigManager {
    /// Create new config manager from UnifiedConfig
    pub fn new(config: UnifiedConfig) -> Self {
        Self {
            config,
            env_overrides: HashMap::new(),
        }
    }
    
    /// Create with defaults
    pub fn with_defaults() -> Self {
        Self::new(UnifiedConfig::default())
    }
    
    /// Load from YAML file
    pub async fn from_yaml_file(path: &Path) -> std::io::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }
    
    /// Load from YAML string
    pub fn from_yaml(yaml: &str) -> std::io::Result<Self> {
        // Try to parse as YAML first, fallback to JSON
        let config = serde_yaml::from_str::<UnifiedConfig>(yaml)
            .or_else(|_| serde_json::from_str(yaml))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let mut manager = Self::new(config);
        manager.apply_env_overrides();
        Ok(manager)
    }
    
    /// Load from JSON file
    pub async fn from_json_file(path: &Path) -> std::io::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_json(&content)
    }
    
    /// Load from JSON string
    pub fn from_json(json: &str) -> std::io::Result<Self> {
        let config = serde_json::from_str(json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        let mut manager = Self::new(config);
        manager.apply_env_overrides();
        Ok(manager)
    }
    
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        // Hierarchy overrides
        if let Ok(val) = env::var(format!("{}WORKING_CAPACITY", ENV_PREFIX)) {
            if let Ok(cap) = val.parse::<usize>() {
                self.config.hierarchy.working_capacity = cap;
                self.env_overrides.insert("working_capacity".to_string(), val);
            }
        }
        
        if let Ok(val) = env::var(format!("{}CORE_CAPACITY", ENV_PREFIX)) {
            if let Ok(cap) = val.parse::<usize>() {
                self.config.hierarchy.core_capacity = cap;
                self.env_overrides.insert("core_capacity".to_string(), val);
            }
        }
        
        // Tiering overrides
        if let Ok(val) = env::var(format!("{}PROMOTE_ACCESS_THRESHOLD", ENV_PREFIX)) {
            if let Ok(threshold) = val.parse::<u32>() {
                self.config.tiering.promote_access_threshold = threshold;
                self.env_overrides.insert("promote_access_threshold".to_string(), val);
            }
        }
        
        if let Ok(val) = env::var(format!("{}PROMOTE_IMPORTANCE_THRESHOLD", ENV_PREFIX)) {
            if let Ok(threshold) = val.parse::<f32>() {
                self.config.tiering.promote_importance_threshold = threshold;
                self.env_overrides.insert("promote_importance_threshold".to_string(), val);
            }
        }
        
        // Archive overrides
        if let Ok(val) = env::var(format!("{}ARCHIVE_MAX_ITEMS", ENV_PREFIX)) {
            if let Ok(max) = val.parse::<usize>() {
                self.config.archive.max_items = max;
                self.env_overrides.insert("archive_max_items".to_string(), val);
            }
        }
        
        if let Ok(val) = env::var(format!("{}ARCHIVE_AFTER_DAYS", ENV_PREFIX)) {
            if let Ok(days) = val.parse::<i64>() {
                self.config.archive.archive_after_days = days;
                self.env_overrides.insert("archive_after_days".to_string(), val);
            }
        }
        
        // Review overrides
        if let Ok(val) = env::var(format!("{}REVIEW_TRIGGER_THRESHOLD", ENV_PREFIX)) {
            if let Ok(threshold) = val.parse::<f32>() {
                self.config.review.trigger_threshold = threshold;
                self.env_overrides.insert("review_trigger_threshold".to_string(), val);
            }
        }
        
        if let Ok(val) = env::var(format!("{}REVIEW_MIN_INTERVAL", ENV_PREFIX)) {
            if let Ok(interval) = val.parse::<i64>() {
                self.config.review.min_review_interval = interval;
                self.env_overrides.insert("review_min_interval".to_string(), val);
            }
        }
    }
    
    /// Get the configuration
    pub fn config(&self) -> &UnifiedConfig {
        &self.config
    }
    
    /// Get mutable config reference
    pub fn config_mut(&mut self) -> &mut UnifiedConfig {
        &mut self.config
    }
    
    /// Get environment overrides
    pub fn env_overrides(&self) -> &HashMap<String, String> {
        &self.env_overrides
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        self.config.to_json()
    }
    
    /// Serialize to YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self.config)
    }
    
    /// Save to file (auto-detect format from extension)
    pub async fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("yaml") 
                  || path.extension().and_then(|s| s.to_str()) == Some("yml") {
            self.to_yaml().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        } else {
            self.to_json().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        };
        
        tokio::fs::write(path, content).await
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        let mut errors = Vec::new();
        
        // Validate hierarchy
        if self.config.hierarchy.working_capacity == 0 {
            errors.push("working_capacity must be > 0".to_string());
        }
        if self.config.hierarchy.core_capacity == 0 {
            errors.push("core_capacity must be > 0".to_string());
        }
        if self.config.hierarchy.working_capacity > self.config.hierarchy.core_capacity {
            errors.push("working_capacity should be <= core_capacity".to_string());
        }
        
        // Validate tiering
        if self.config.tiering.promote_access_threshold == 0 {
            errors.push("promote_access_threshold must be > 0".to_string());
        }
        if !(0.0..=1.0).contains(&self.config.tiering.promote_importance_threshold) {
            errors.push("promote_importance_threshold must be between 0.0 and 1.0".to_string());
        }
        
        // Validate archive
        if self.config.archive.archive_after_days < 0 {
            errors.push("archive_after_days must be >= 0".to_string());
        }
        
        // Validate review
        if !(0.0..=1.0).contains(&self.config.review.trigger_threshold) {
            errors.push("review.trigger_threshold must be between 0.0 and 1.0".to_string());
        }
        if self.config.review.min_review_interval < 60 {
            errors.push("min_review_interval should be >= 60 seconds".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigValidationError(errors))
        }
    }
}

/// Configuration validation error
#[derive(Debug)]
pub struct ConfigValidationError(pub Vec<String>);

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Configuration validation failed:\n{}", self.0.join("\n"))
    }
}

impl std::error::Error for ConfigValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let manager = ConfigManager::with_defaults();
        assert_eq!(manager.config().hierarchy.working_capacity, 100);
        assert_eq!(manager.config().hierarchy.core_capacity, 1000);
    }
    
    #[test]
    fn test_json_serialization() {
        let manager = ConfigManager::with_defaults();
        let json = manager.to_json().unwrap();
        assert!(json.contains("working_capacity"));
    }
    
    #[test]
    fn test_yaml_serialization() {
        let manager = ConfigManager::with_defaults();
        let yaml = manager.to_yaml().unwrap();
        assert!(yaml.contains("working_capacity"));
    }
    
    #[test]
    fn test_validation_success() {
        let manager = ConfigManager::with_defaults();
        assert!(manager.validate().is_ok());
    }
    
    #[test]
    fn test_validation_failure() {
        let mut manager = ConfigManager::with_defaults();
        manager.config_mut().hierarchy.working_capacity = 0;
        let result = manager.validate();
        assert!(result.is_err());
    }
}
