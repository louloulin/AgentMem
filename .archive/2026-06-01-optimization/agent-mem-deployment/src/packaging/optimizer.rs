//! 二进制优化器
//!
//! 提供二进制文件大小优化功能

use super::PackageConfig;
use agent_mem_traits::{AgentMemError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// 二进制优化器
pub struct BinaryOptimizer {
    config: PackageConfig,
}

impl BinaryOptimizer {
    /// 创建新的二进制优化器
    pub fn new(config: PackageConfig) -> Self {
        Self { config }
    }

    /// 优化二进制文件
    pub async fn optimize(&self, binary_path: &Path) -> Result<PathBuf> {
        info!("开始优化二进制文件: {:?}", binary_path);

        let mut current_path = binary_path.to_path_buf();

        // 1. Strip 符号
        if self.config.enable_strip {
            current_path = self.strip_binary(&current_path).await?;
        }

        // 2. 压缩
        if self.config.enable_compression {
            current_path = self.compress_binary(&current_path).await?;
        }

        // 3. 报告优化结果
        self.report_optimization(binary_path, &current_path).await?;

        info!("二进制文件优化完成: {:?}", current_path);
        Ok(current_path)
    }

    /// Strip 二进制文件（移除符号）
    async fn strip_binary(&self, binary_path: &Path) -> Result<PathBuf> {
        info!("移除二进制符号...");

        // 创建临时文件
        let stripped_path = binary_path.with_extension("stripped");

        // 复制文件
        tokio::fs::copy(binary_path, &stripped_path)
            .await
            .map_err(|e| AgentMemError::internal_error(format!("复制文件失败: {e}")))?;

        // 执行 strip 命令
        let strip_cmd = if cfg!(target_os = "macos") {
            "strip"
        } else if cfg!(target_os = "linux") {
            "strip"
        } else if cfg!(target_os = "windows") {
            // Windows 使用不同的工具
            warn!("Windows 平台暂不支持 strip");
            return Ok(stripped_path);
        } else {
            warn!("未知平台，跳过 strip");
            return Ok(stripped_path);
        };

        let output = Command::new(strip_cmd)
            .arg(&stripped_path)
            .output()
            .map_err(|e| AgentMemError::internal_error(format!("执行 strip 失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Strip 失败: {}", stderr);
            return Ok(stripped_path);
        }

        info!("符号移除完成");
        Ok(stripped_path)
    }

    /// 压缩二进制文件
    async fn compress_binary(&self, binary_path: &Path) -> Result<PathBuf> {
        use super::config::CompressionAlgorithm;

        match self.config.compression {
            CompressionAlgorithm::None => Ok(binary_path.to_path_buf()),
            CompressionAlgorithm::Gzip => self.compress_with_gzip(binary_path).await,
            CompressionAlgorithm::Zstd => self.compress_with_zstd(binary_path).await,
            CompressionAlgorithm::Upx => self.compress_with_upx(binary_path).await,
        }
    }

    /// 使用 Gzip 压缩
    async fn compress_with_gzip(&self, binary_path: &Path) -> Result<PathBuf> {
        info!("使用 Gzip 压缩...");

        let compressed_path = binary_path.with_extension("gz");

        let output = Command::new("gzip")
            .arg("-9")
            .arg("-c")
            .arg(binary_path)
            .output()
            .map_err(|e| AgentMemError::internal_error(format!("执行 gzip 失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AgentMemError::internal_error(format!(
                "Gzip 压缩失败: {stderr}"
            )));
        }

        tokio::fs::write(&compressed_path, output.stdout)
            .await
            .map_err(|e| AgentMemError::internal_error(format!("写入压缩文件失败: {e}")))?;

        info!("Gzip 压缩完成");
        Ok(compressed_path)
    }

    /// 使用 Zstd 压缩
    async fn compress_with_zstd(&self, binary_path: &Path) -> Result<PathBuf> {
        info!("使用 Zstd 压缩...");

        let compressed_path = binary_path.with_extension("zst");

        let output = Command::new("zstd")
            .arg("-19")
            .arg("-o")
            .arg(&compressed_path)
            .arg(binary_path)
            .output()
            .map_err(|e| AgentMemError::internal_error(format!("执行 zstd 失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AgentMemError::internal_error(format!(
                "Zstd 压缩失败: {stderr}"
            )));
        }

        info!("Zstd 压缩完成");
        Ok(compressed_path)
    }

    /// 使用 UPX 压缩
    async fn compress_with_upx(&self, binary_path: &Path) -> Result<PathBuf> {
        info!("使用 UPX 压缩...");

        let compressed_path = binary_path.with_extension("upx");

        // 复制文件
        tokio::fs::copy(binary_path, &compressed_path)
            .await
            .map_err(|e| AgentMemError::internal_error(format!("复制文件失败: {e}")))?;

        let output = Command::new("upx")
            .arg("--best")
            .arg("--lzma")
            .arg(&compressed_path)
            .output()
            .map_err(|e| AgentMemError::internal_error(format!("执行 upx 失败: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AgentMemError::internal_error(format!(
                "UPX 压缩失败: {stderr}"
            )));
        }

        info!("UPX 压缩完成");
        Ok(compressed_path)
    }

    /// 报告优化结果
    async fn report_optimization(&self, original: &Path, optimized: &Path) -> Result<()> {
        let original_size = tokio::fs::metadata(original)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        let optimized_size = tokio::fs::metadata(optimized)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        let reduction = if original_size > 0 {
            ((original_size - optimized_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        info!("优化结果:");
        info!(
            "  原始大小: {} bytes ({:.2} MB)",
            original_size,
            original_size as f64 / 1024.0 / 1024.0
        );
        info!(
            "  优化后大小: {} bytes ({:.2} MB)",
            optimized_size,
            optimized_size as f64 / 1024.0 / 1024.0
        );
        info!("  减少: {:.2}%", reduction);

        Ok(())
    }
}

/// 获取二进制文件大小
pub async fn get_binary_size(path: &Path) -> Result<u64> {
    tokio::fs::metadata(path)
        .await
        .map(|m| m.len())
        .map_err(|e| AgentMemError::internal_error(format!("获取文件大小失败: {e}")))
}

/// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
