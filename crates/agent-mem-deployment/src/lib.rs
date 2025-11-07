//! AgentMem 部署模块
//!
//! 提供单二进制打包和嵌入式存储支持

pub mod config_embed;
pub mod config_loader;
pub mod embedded;
pub mod packaging;

pub use config_embed::{ConfigTemplate, EmbeddedConfigManager};
pub use config_loader::{ConfigFile, ConfigFormat, ConfigLoader};
pub use embedded::{EmbeddedConfig, EmbeddedDatabase, EmbeddedVectorStore};
pub use packaging::{BinaryOptimizer, PackageBuilder, PackageConfig};
