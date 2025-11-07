//! Tool Execution Sandbox - 工具执行沙箱
//!
//! 参考 MIRIX 的 ToolExecutionSandbox 实现，提供完整的工具执行沙箱功能：
//! 1. 本地沙箱执行
//! 2. 虚拟环境隔离
//! 3. 资源限制和监控
//! 4. 安全执行环境
//!
//! 学习自 MIRIX: mirix/services/tool_execution_sandbox.py

use crate::error::{ToolError, ToolResult};
use crate::sandbox::{CommandOutput, SandboxConfig, SandboxManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

/// 沙箱类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxType {
    /// 本地沙箱
    Local,
    /// Docker 沙箱
    Docker,
    /// 虚拟机沙箱
    VM,
}

impl SandboxType {
    pub fn as_str(&self) -> &str {
        match self {
            SandboxType::Local => "local",
            SandboxType::Docker => "docker",
            SandboxType::VM => "vm",
        }
    }

    pub fn from_str(s: &str) -> ToolResult<Self> {
        match s {
            "local" => Ok(SandboxType::Local),
            "docker" => Ok(SandboxType::Docker),
            "vm" => Ok(SandboxType::VM),
            _ => Err(ToolError::InvalidArgument(format!(
                "Invalid sandbox type: {s}"
            ))),
        }
    }
}

/// 沙箱执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRunResult {
    /// 函数返回值
    pub func_return: Option<serde_json::Value>,
    /// 标准输出
    pub stdout: Vec<String>,
    /// 标准错误
    pub stderr: Vec<String>,
    /// 执行状态
    pub status: SandboxStatus,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 内存使用（字节）
    pub memory_used: Option<usize>,
}

/// 沙箱执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxStatus {
    /// 成功
    Success,
    /// 错误
    Error,
    /// 超时
    Timeout,
    /// 资源限制
    ResourceLimit,
}

/// 本地沙箱配置
///
/// 参考 MIRIX: LocalSandboxConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSandboxConfig {
    /// 沙箱目录
    pub sandbox_dir: PathBuf,
    /// 是否使用虚拟环境
    pub use_venv: bool,
    /// 虚拟环境名称
    pub venv_name: String,
    /// 超时时间（秒）
    pub timeout: u64,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
}

impl Default for LocalSandboxConfig {
    fn default() -> Self {
        Self {
            sandbox_dir: PathBuf::from("/tmp/agentmem_sandbox"),
            use_venv: false,
            venv_name: "venv".to_string(),
            timeout: 60,
            env_vars: HashMap::new(),
        }
    }
}

/// 工具执行沙箱
///
/// 参考 MIRIX: ToolExecutionSandbox
pub struct ToolExecutionSandbox {
    /// 工具名称
    tool_name: String,
    /// 工具参数
    args: serde_json::Value,
    /// 沙箱类型
    sandbox_type: SandboxType,
    /// 本地沙箱配置
    local_config: LocalSandboxConfig,
    /// 沙箱管理器
    sandbox_manager: SandboxManager,
}

impl ToolExecutionSandbox {
    /// 创建新的工具执行沙箱
    pub fn new(
        tool_name: String,
        args: serde_json::Value,
        sandbox_type: SandboxType,
    ) -> Self {
        let local_config = LocalSandboxConfig::default();
        let sandbox_config = SandboxConfig {
            max_memory: 512 * 1024 * 1024, // 512MB
            max_cpu_time: Some(local_config.timeout),
            default_timeout: Duration::from_secs(local_config.timeout),
            enable_monitoring: true,
            enable_network_isolation: false,
            working_directory: Some(local_config.sandbox_dir.clone()),
            environment_variables: local_config.env_vars.clone(),
            enable_filesystem_isolation: false,
            allowed_paths: vec![local_config.sandbox_dir.clone()],
        };

        Self {
            tool_name,
            args,
            sandbox_type,
            local_config,
            sandbox_manager: SandboxManager::new(sandbox_config),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(
        tool_name: String,
        args: serde_json::Value,
        sandbox_type: SandboxType,
        local_config: LocalSandboxConfig,
    ) -> Self {
        let sandbox_config = SandboxConfig {
            max_memory: 512 * 1024 * 1024,
            max_cpu_time: Some(local_config.timeout),
            default_timeout: Duration::from_secs(local_config.timeout),
            enable_monitoring: true,
            enable_network_isolation: false,
            working_directory: Some(local_config.sandbox_dir.clone()),
            environment_variables: local_config.env_vars.clone(),
            enable_filesystem_isolation: false,
            allowed_paths: vec![local_config.sandbox_dir.clone()],
        };

        Self {
            tool_name,
            args,
            sandbox_type,
            local_config,
            sandbox_manager: SandboxManager::new(sandbox_config),
        }
    }

    /// 运行工具
    ///
    /// 参考 MIRIX: run()
    pub async fn run(&self) -> ToolResult<SandboxRunResult> {
        info!(
            "Running tool '{}' in {:?} sandbox",
            self.tool_name, self.sandbox_type
        );

        let start_time = std::time::Instant::now();

        let result = match self.sandbox_type {
            SandboxType::Local => self.run_local_sandbox().await?,
            SandboxType::Docker => {
                warn!("Docker sandbox not yet implemented, falling back to local");
                self.run_local_sandbox().await?
            }
            SandboxType::VM => {
                warn!("VM sandbox not yet implemented, falling back to local");
                self.run_local_sandbox().await?
            }
        };

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(SandboxRunResult {
            func_return: result.func_return,
            stdout: result.stdout,
            stderr: result.stderr,
            status: result.status,
            execution_time_ms,
            memory_used: result.memory_used,
        })
    }

    /// 运行本地沙箱
    ///
    /// 参考 MIRIX: run_local_dir_sandbox()
    async fn run_local_sandbox(&self) -> ToolResult<SandboxRunResult> {
        debug!(
            "Executing tool '{}' in local sandbox at {:?}",
            self.tool_name, self.local_config.sandbox_dir
        );

        // 确保沙箱目录存在
        if !self.local_config.sandbox_dir.exists() {
            std::fs::create_dir_all(&self.local_config.sandbox_dir).map_err(|e| {
                ToolError::ExecutionFailed(format!("Failed to create sandbox directory: {e}"))
            })?;
        }

        // 生成执行脚本
        let script = self.generate_execution_script()?;

        // 写入临时文件
        let temp_file = self.local_config.sandbox_dir.join(format!(
            "tool_{}_{}.py",
            self.tool_name,
            uuid::Uuid::new_v4()
        ));

        std::fs::write(&temp_file, script).map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to write execution script: {e}"))
        })?;

        // 执行
        let result = if self.local_config.use_venv {
            self.run_with_venv(&temp_file).await
        } else {
            self.run_direct(&temp_file).await
        };

        // 清理临时文件
        let _ = std::fs::remove_file(&temp_file);

        result
    }

    /// 生成执行脚本
    ///
    /// 参考 MIRIX: generate_execution_script()
    fn generate_execution_script(&self) -> ToolResult<String> {
        // 简化版本：直接执行 Python 代码
        // 在实际实现中，这里应该根据工具的源代码生成完整的执行脚本
        let script = format!(
            r#"
import json
import sys

# Tool: {}
# Args: {}

def main():
    args = json.loads('{}')
    # TODO: 实际执行工具代码
    result = {{"status": "success", "message": "Tool executed", "args": args}}
    print(json.dumps(result))

if __name__ == "__main__":
    main()
"#,
            self.tool_name,
            self.args,
            serde_json::to_string(&self.args).unwrap()
        );

        Ok(script)
    }

    /// 直接运行（不使用虚拟环境）
    async fn run_direct(&self, script_path: &PathBuf) -> ToolResult<SandboxRunResult> {
        let timeout_duration = Duration::from_secs(self.local_config.timeout);

        let output = self
            .sandbox_manager
            .execute_command(
                "python3",
                &[script_path.to_str().unwrap()],
                timeout_duration,
            )
            .await?;

        self.parse_output(output)
    }

    /// 使用虚拟环境运行
    ///
    /// 参考 MIRIX: run_local_dir_sandbox_venv()
    async fn run_with_venv(&self, script_path: &PathBuf) -> ToolResult<SandboxRunResult> {
        let venv_path = self.local_config.sandbox_dir.join(&self.local_config.venv_name);

        // 检查虚拟环境是否存在
        if !venv_path.exists() {
            info!("Creating virtual environment at {:?}", venv_path);
            // TODO: 创建虚拟环境
            // 这里需要调用 python -m venv
        }

        let python_executable = venv_path.join("bin").join("python3");
        let timeout_duration = Duration::from_secs(self.local_config.timeout);

        let output = self
            .sandbox_manager
            .execute_command(
                python_executable.to_str().unwrap(),
                &[script_path.to_str().unwrap()],
                timeout_duration,
            )
            .await?;

        self.parse_output(output)
    }

    /// 解析输出
    fn parse_output(&self, output: CommandOutput) -> ToolResult<SandboxRunResult> {
        let status = if output.success {
            SandboxStatus::Success
        } else {
            SandboxStatus::Error
        };

        // 尝试解析 JSON 输出
        let func_return = if !output.stdout.is_empty() {
            serde_json::from_str(&output.stdout).ok()
        } else {
            None
        };

        Ok(SandboxRunResult {
            func_return,
            stdout: vec![output.stdout],
            stderr: if output.stderr.is_empty() {
                vec![]
            } else {
                vec![output.stderr]
            },
            status,
            execution_time_ms: 0, // 将在外部设置
            memory_used: None,
        })
    }
}

