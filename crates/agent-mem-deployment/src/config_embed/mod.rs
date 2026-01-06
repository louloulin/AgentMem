//! 配置文件嵌入模块
//!
//! 将配置文件嵌入到二进制文件中，支持单二进制部署

pub mod manager;
pub mod templates;

pub use manager::EmbeddedConfigManager;
pub use templates::ConfigTemplate;
