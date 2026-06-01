//! 打包模块
//!
//! 提供单二进制打包和优化功能

pub mod builder;
pub mod config;
pub mod optimizer;

pub use builder::PackageBuilder;
pub use config::PackageConfig;
pub use optimizer::BinaryOptimizer;
