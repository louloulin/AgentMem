//! 打包功能测试

use agent_mem_deployment::packaging::{PackageConfig, BinaryOptimizer};
use agent_mem_deployment::packaging::config::{TargetPlatform, OptimizationLevel};

#[test]
fn test_package_config_creation() {
    let config = PackageConfig::default();
    assert_eq!(config.binary_name, "agentmem");
    assert_eq!(config.target, TargetPlatform::LinuxX64);
}

#[test]
fn test_package_config_production() {
    let config = PackageConfig::production();
    assert_eq!(config.optimization_level, OptimizationLevel::MaxSpeed);
    assert!(config.enable_lto);
    assert!(config.enable_strip);
    assert!(config.enable_compression);
}

#[test]
fn test_package_config_minimal() {
    let config = PackageConfig::minimal();
    assert_eq!(config.optimization_level, OptimizationLevel::MinSize);
    assert!(!config.include_docs);
}

#[test]
fn test_package_config_development() {
    let config = PackageConfig::development();
    assert_eq!(config.optimization_level, OptimizationLevel::Debug);
    assert!(!config.enable_lto);
}

#[test]
fn test_target_platform_rust_target() {
    assert_eq!(
        TargetPlatform::LinuxX64.rust_target(),
        "x86_64-unknown-linux-gnu"
    );
    assert_eq!(
        TargetPlatform::MacOSArm64.rust_target(),
        "aarch64-apple-darwin"
    );
}

#[test]
fn test_target_platform_binary_extension() {
    assert_eq!(TargetPlatform::LinuxX64.binary_extension(), "");
    assert_eq!(TargetPlatform::WindowsX64.binary_extension(), ".exe");
}

#[test]
fn test_optimization_level_cargo_profile() {
    assert_eq!(OptimizationLevel::Debug.cargo_profile(), "dev");
    assert_eq!(OptimizationLevel::Release.cargo_profile(), "release");
}

#[test]
fn test_optimization_level_opt_level() {
    assert_eq!(OptimizationLevel::Debug.opt_level(), "0");
    assert_eq!(OptimizationLevel::MinSize.opt_level(), "z");
    assert_eq!(OptimizationLevel::MaxSpeed.opt_level(), "3");
}

#[test]
fn test_package_config_with_target() {
    let config = PackageConfig::default()
        .with_target(TargetPlatform::MacOSArm64);
    assert_eq!(config.target, TargetPlatform::MacOSArm64);
}

#[test]
fn test_package_config_full_binary_name() {
    let config = PackageConfig::default();
    assert_eq!(config.full_binary_name(), "agentmem");
    
    let config = config.with_target(TargetPlatform::WindowsX64);
    assert_eq!(config.full_binary_name(), "agentmem.exe");
}

#[test]
fn test_binary_optimizer_creation() {
    let config = PackageConfig::default();
    let _optimizer = BinaryOptimizer::new(config);
}

#[test]
fn test_format_size() {
    use agent_mem_deployment::packaging::optimizer::format_size;
    
    assert_eq!(format_size(512), "512 bytes");
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
}

