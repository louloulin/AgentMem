//! 打包配置
//!
//! 定义单二进制打包的配置选项

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 打包配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// 目标平台
    pub target: TargetPlatform,
    
    /// 优化级别
    pub optimization_level: OptimizationLevel,
    
    /// 是否启用 LTO（链接时优化）
    pub enable_lto: bool,
    
    /// 是否启用 strip（移除符号）
    pub enable_strip: bool,
    
    /// 是否启用压缩
    pub enable_compression: bool,
    
    /// 压缩算法
    pub compression: CompressionAlgorithm,
    
    /// 输出目录
    pub output_dir: PathBuf,
    
    /// 二进制文件名
    pub binary_name: String,
    
    /// 是否包含配置文件
    pub include_config: bool,
    
    /// 是否包含文档
    pub include_docs: bool,
}

/// 目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPlatform {
    /// Linux x86_64
    LinuxX64,
    /// Linux ARM64
    LinuxArm64,
    /// macOS x86_64
    MacOSX64,
    /// macOS ARM64 (Apple Silicon)
    MacOSArm64,
    /// Windows x86_64
    WindowsX64,
}

impl TargetPlatform {
    /// 获取 Rust 目标三元组
    pub fn rust_target(&self) -> &'static str {
        match self {
            TargetPlatform::LinuxX64 => "x86_64-unknown-linux-gnu",
            TargetPlatform::LinuxArm64 => "aarch64-unknown-linux-gnu",
            TargetPlatform::MacOSX64 => "x86_64-apple-darwin",
            TargetPlatform::MacOSArm64 => "aarch64-apple-darwin",
            TargetPlatform::WindowsX64 => "x86_64-pc-windows-msvc",
        }
    }
    
    /// 获取平台名称
    pub fn name(&self) -> &'static str {
        match self {
            TargetPlatform::LinuxX64 => "linux-x64",
            TargetPlatform::LinuxArm64 => "linux-arm64",
            TargetPlatform::MacOSX64 => "macos-x64",
            TargetPlatform::MacOSArm64 => "macos-arm64",
            TargetPlatform::WindowsX64 => "windows-x64",
        }
    }
    
    /// 获取二进制文件扩展名
    pub fn binary_extension(&self) -> &'static str {
        match self {
            TargetPlatform::WindowsX64 => ".exe",
            _ => "",
        }
    }
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// 无优化（调试）
    Debug,
    /// 基本优化
    Release,
    /// 大小优化
    MinSize,
    /// 速度优化
    MaxSpeed,
}

impl OptimizationLevel {
    /// 获取 Cargo 配置
    pub fn cargo_profile(&self) -> &'static str {
        match self {
            OptimizationLevel::Debug => "dev",
            OptimizationLevel::Release => "release",
            OptimizationLevel::MinSize => "release",
            OptimizationLevel::MaxSpeed => "release",
        }
    }
    
    /// 获取优化标志
    pub fn opt_level(&self) -> &'static str {
        match self {
            OptimizationLevel::Debug => "0",
            OptimizationLevel::Release => "2",
            OptimizationLevel::MinSize => "z",
            OptimizationLevel::MaxSpeed => "3",
        }
    }
}

/// 压缩算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// 无压缩
    None,
    /// Gzip
    Gzip,
    /// Zstd
    Zstd,
    /// UPX
    Upx,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            target: TargetPlatform::LinuxX64,
            optimization_level: OptimizationLevel::Release,
            enable_lto: true,
            enable_strip: true,
            enable_compression: false,
            compression: CompressionAlgorithm::None,
            output_dir: PathBuf::from("./dist"),
            binary_name: "agentmem".to_string(),
            include_config: true,
            include_docs: false,
        }
    }
}

impl PackageConfig {
    /// 创建生产环境打包配置
    pub fn production() -> Self {
        Self {
            optimization_level: OptimizationLevel::MaxSpeed,
            enable_lto: true,
            enable_strip: true,
            enable_compression: true,
            compression: CompressionAlgorithm::Zstd,
            ..Default::default()
        }
    }
    
    /// 创建最小化打包配置
    pub fn minimal() -> Self {
        Self {
            optimization_level: OptimizationLevel::MinSize,
            enable_lto: true,
            enable_strip: true,
            enable_compression: true,
            compression: CompressionAlgorithm::Upx,
            include_config: false,
            include_docs: false,
            ..Default::default()
        }
    }
    
    /// 创建开发环境打包配置
    pub fn development() -> Self {
        Self {
            optimization_level: OptimizationLevel::Debug,
            enable_lto: false,
            enable_strip: false,
            enable_compression: false,
            compression: CompressionAlgorithm::None,
            include_config: true,
            include_docs: true,
            ..Default::default()
        }
    }
    
    /// 设置目标平台
    pub fn with_target(mut self, target: TargetPlatform) -> Self {
        self.target = target;
        self
    }
    
    /// 设置输出目录
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = dir;
        self
    }
    
    /// 设置二进制名称
    pub fn with_binary_name(mut self, name: String) -> Self {
        self.binary_name = name;
        self
    }
    
    /// 获取完整的二进制文件名
    pub fn full_binary_name(&self) -> String {
        format!(
            "{}{}",
            self.binary_name,
            self.target.binary_extension()
        )
    }
    
    /// 获取输出路径
    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(self.full_binary_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_target_platform() {
        assert_eq!(
            TargetPlatform::LinuxX64.rust_target(),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(TargetPlatform::LinuxX64.name(), "linux-x64");
        assert_eq!(TargetPlatform::LinuxX64.binary_extension(), "");
        assert_eq!(TargetPlatform::WindowsX64.binary_extension(), ".exe");
    }
    
    #[test]
    fn test_optimization_level() {
        assert_eq!(OptimizationLevel::Debug.cargo_profile(), "dev");
        assert_eq!(OptimizationLevel::Release.cargo_profile(), "release");
        assert_eq!(OptimizationLevel::MinSize.opt_level(), "z");
        assert_eq!(OptimizationLevel::MaxSpeed.opt_level(), "3");
    }
    
    #[test]
    fn test_package_config() {
        let config = PackageConfig::default();
        assert_eq!(config.binary_name, "agentmem");
        assert_eq!(config.full_binary_name(), "agentmem");
        
        let config = PackageConfig::default()
            .with_target(TargetPlatform::WindowsX64);
        assert_eq!(config.full_binary_name(), "agentmem.exe");
    }
    
    #[test]
    fn test_package_config_presets() {
        let prod = PackageConfig::production();
        assert!(prod.enable_lto);
        assert!(prod.enable_strip);
        assert!(prod.enable_compression);
        
        let minimal = PackageConfig::minimal();
        assert_eq!(minimal.optimization_level, OptimizationLevel::MinSize);
        assert!(!minimal.include_docs);
        
        let dev = PackageConfig::development();
        assert_eq!(dev.optimization_level, OptimizationLevel::Debug);
        assert!(!dev.enable_lto);
    }
}

