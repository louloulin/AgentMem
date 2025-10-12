//! 打包模块
//!
//! 提供单二进制打包和优化功能

pub mod builder;
pub mod optimizer;
pub mod config;

pub use builder::PackageBuilder;
pub use optimizer::BinaryOptimizer;
pub use config::PackageConfig;

