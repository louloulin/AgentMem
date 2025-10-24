//! 打包构建器
//!
//! 负责构建单二进制包

use super::{BinaryOptimizer, PackageConfig};
use agent_mem_traits::{AgentMemError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info};

/// 打包构建器
pub struct PackageBuilder {
    config: PackageConfig,
    workspace_root: PathBuf,
}

impl PackageBuilder {
    /// 创建新的打包构建器
    pub fn new(config: PackageConfig) -> Self {
        Self {
            config,
            workspace_root: PathBuf::from("."),
        }
    }
    
    /// 设置工作空间根目录
    pub fn with_workspace_root<P: AsRef<Path>>(mut self, root: P) -> Self {
        self.workspace_root = root.as_ref().to_path_buf();
        self
    }
    
    /// 构建二进制包
    pub async fn build(&self) -> Result<PathBuf> {
        info!("开始构建单二进制包...");
        info!("目标平台: {}", self.config.target.name());
        info!("优化级别: {:?}", self.config.optimization_level);
        
        // 1. 清理输出目录
        self.clean_output_dir().await?;
        
        // 2. 构建二进制文件
        let binary_path = self.build_binary().await?;
        
        // 3. 优化二进制文件
        let optimized_path = if self.config.enable_strip || self.config.enable_compression {
            self.optimize_binary(&binary_path).await?
        } else {
            binary_path
        };
        
        // 4. 复制到输出目录
        let output_path = self.copy_to_output(&optimized_path).await?;
        
        // 5. 包含配置文件（如果需要）
        if self.config.include_config {
            self.include_config_files().await?;
        }
        
        // 6. 包含文档（如果需要）
        if self.config.include_docs {
            self.include_docs().await?;
        }
        
        // 7. 生成元数据
        self.generate_metadata(&output_path).await?;
        
        info!("单二进制包构建完成: {:?}", output_path);
        Ok(output_path)
    }
    
    /// 清理输出目录
    async fn clean_output_dir(&self) -> Result<()> {
        debug!("清理输出目录: {:?}", self.config.output_dir);
        
        if self.config.output_dir.exists() {
            tokio::fs::remove_dir_all(&self.config.output_dir)
                .await
                .map_err(|e| {
                    AgentMemError::internal_error(format!("清理输出目录失败: {e}"))
                })?;
        }
        
        tokio::fs::create_dir_all(&self.config.output_dir)
            .await
            .map_err(|e| {
                AgentMemError::internal_error(format!("创建输出目录失败: {e}"))
            })?;
        
        Ok(())
    }
    
    /// 构建二进制文件
    async fn build_binary(&self) -> Result<PathBuf> {
        info!("构建二进制文件...");
        
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        cmd.arg("--release");
        cmd.arg("--target");
        cmd.arg(self.config.target.rust_target());
        
        // 设置优化级别
        if self.config.enable_lto {
            cmd.env("CARGO_PROFILE_RELEASE_LTO", "true");
        }
        
        cmd.env(
            "CARGO_PROFILE_RELEASE_OPT_LEVEL",
            self.config.optimization_level.opt_level(),
        );
        
        // 设置工作目录
        cmd.current_dir(&self.workspace_root);
        
        debug!("执行命令: {:?}", cmd);
        
        let output = cmd.output().map_err(|e| {
            AgentMemError::internal_error(format!("执行 cargo build 失败: {e}"))
        })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AgentMemError::internal_error(format!(
                "构建失败: {stderr}"
            )));
        }
        
        // 查找生成的二进制文件
        let binary_path = self
            .workspace_root
            .join("target")
            .join(self.config.target.rust_target())
            .join("release")
            .join(self.config.full_binary_name());
        
        if !binary_path.exists() {
            return Err(AgentMemError::internal_error(format!(
                "找不到构建的二进制文件: {binary_path:?}"
            )));
        }
        
        info!("二进制文件构建完成: {:?}", binary_path);
        Ok(binary_path)
    }
    
    /// 优化二进制文件
    async fn optimize_binary(&self, binary_path: &Path) -> Result<PathBuf> {
        info!("优化二进制文件...");
        
        let optimizer = BinaryOptimizer::new(self.config.clone());
        optimizer.optimize(binary_path).await
    }
    
    /// 复制到输出目录
    async fn copy_to_output(&self, binary_path: &Path) -> Result<PathBuf> {
        let output_path = self.config.output_path();
        
        debug!("复制二进制文件到: {:?}", output_path);
        
        tokio::fs::copy(binary_path, &output_path)
            .await
            .map_err(|e| {
                AgentMemError::internal_error(format!("复制二进制文件失败: {e}"))
            })?;
        
        Ok(output_path)
    }
    
    /// 包含配置文件
    async fn include_config_files(&self) -> Result<()> {
        info!("包含配置文件...");
        
        let config_dir = self.config.output_dir.join("config");
        tokio::fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| {
                AgentMemError::internal_error(format!("创建配置目录失败: {e}"))
            })?;
        
        // 导出所有配置模板
        use crate::config_embed::{ConfigTemplate, EmbeddedConfigManager};
        
        for template in ConfigTemplate::all() {
            let manager = EmbeddedConfigManager::new(template);
            let filename = format!("config.{}.toml", template.name());
            let path = config_dir.join(filename);
            manager.export_to_file(&path)?;
        }
        
        info!("配置文件已包含");
        Ok(())
    }
    
    /// 包含文档
    async fn include_docs(&self) -> Result<()> {
        info!("包含文档...");
        
        let docs_dir = self.config.output_dir.join("docs");
        tokio::fs::create_dir_all(&docs_dir)
            .await
            .map_err(|e| {
                AgentMemError::internal_error(format!("创建文档目录失败: {e}"))
            })?;
        
        // 复制 README
        let readme_src = self.workspace_root.join("README.md");
        if readme_src.exists() {
            let readme_dst = docs_dir.join("README.md");
            tokio::fs::copy(&readme_src, &readme_dst)
                .await
                .map_err(|e| {
                    AgentMemError::internal_error(format!("复制 README 失败: {e}"))
                })?;
        }
        
        info!("文档已包含");
        Ok(())
    }
    
    /// 生成元数据
    async fn generate_metadata(&self, binary_path: &Path) -> Result<()> {
        info!("生成元数据...");
        
        let metadata = PackageMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            target: self.config.target.name().to_string(),
            optimization: format!("{:?}", self.config.optimization_level),
            lto_enabled: self.config.enable_lto,
            stripped: self.config.enable_strip,
            compressed: self.config.enable_compression,
            binary_size: tokio::fs::metadata(binary_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0),
            build_time: chrono::Utc::now().to_rfc3339(),
        };
        
        let metadata_path = self.config.output_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&metadata).map_err(|e| {
            AgentMemError::internal_error(format!("序列化元数据失败: {e}"))
        })?;
        
        tokio::fs::write(&metadata_path, content)
            .await
            .map_err(|e| {
                AgentMemError::internal_error(format!("写入元数据失败: {e}"))
            })?;
        
        info!("元数据已生成: {:?}", metadata_path);
        Ok(())
    }
}

/// 打包元数据
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PackageMetadata {
    version: String,
    target: String,
    optimization: String,
    lto_enabled: bool,
    stripped: bool,
    compressed: bool,
    binary_size: u64,
    build_time: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_package_builder_creation() {
        let config = PackageConfig::default();
        let _builder = PackageBuilder::new(config);
    }
}

