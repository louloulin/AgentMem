//! AgentMem 部署模块
//!
//! 提供单二进制打包和嵌入式存储支持

pub mod embedded;
pub mod packaging;
pub mod config_embed;

pub use embedded::{EmbeddedDatabase, EmbeddedVectorStore, EmbeddedConfig};
pub use packaging::{PackageBuilder, PackageConfig, BinaryOptimizer};
pub use config_embed::{EmbeddedConfigManager, ConfigTemplate};

