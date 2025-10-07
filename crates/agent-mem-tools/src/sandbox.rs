//! Sandboxed execution environment for tools
//!
//! This module provides safe tool execution with timeout and resource limits,
//! inspired by MIRIX's tool_execution_sandbox.py.
//!
//! 参考 MIRIX 实现:
//! - mirix/services/tool_execution_sandbox.py
//! - 本地沙箱执行
//! - 虚拟环境隔离
//! - 资源限制和监控

use crate::error::{ToolError, ToolResult};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory usage (bytes)
    pub max_memory: usize,
    /// Maximum CPU time (seconds)
    pub max_cpu_time: Option<u64>,
    /// Default timeout duration
    pub default_timeout: Duration,
    /// Enable resource monitoring
    pub enable_monitoring: bool,
    /// Enable network isolation
    pub enable_network_isolation: bool,
    /// Working directory for sandboxed execution
    pub working_directory: Option<PathBuf>,
    /// Environment variables for sandboxed execution
    pub environment_variables: HashMap<String, String>,
    /// Enable file system isolation
    pub enable_filesystem_isolation: bool,
    /// Allowed file system paths (when isolation is enabled)
    pub allowed_paths: Vec<PathBuf>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: 512 * 1024 * 1024, // 512MB
            max_cpu_time: Some(30),
            default_timeout: Duration::from_secs(30),
            enable_monitoring: true,
            enable_network_isolation: false,
            working_directory: None,
            environment_variables: HashMap::new(),
            enable_filesystem_isolation: false,
            allowed_paths: Vec::new(),
        }
    }
}

/// Sandbox manager for safe tool execution
#[derive(Default)]
pub struct SandboxManager {
    config: SandboxConfig,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Execute a function in the sandbox with timeout control
    pub async fn execute<F, T>(&self, func: F, timeout_duration: Duration) -> ToolResult<T>
    where
        F: Future<Output = ToolResult<T>> + Send,
        T: Send,
    {
        debug!("Executing in sandbox with timeout: {:?}", timeout_duration);

        // Check resource usage before execution
        if self.config.enable_monitoring {
            self.check_resource_usage()?;
        }

        // Execute with timeout
        let result = timeout(timeout_duration, func).await;

        match result {
            Ok(Ok(value)) => {
                debug!("Sandbox execution completed successfully");
                Ok(value)
            }
            Ok(Err(e)) => {
                warn!("Sandbox execution failed: {}", e);
                Err(e)
            }
            Err(_) => {
                warn!("Sandbox execution timeout");
                Err(ToolError::Timeout)
            }
        }
    }

    /// Execute with default timeout
    pub async fn execute_default<F, T>(&self, func: F) -> ToolResult<T>
    where
        F: Future<Output = ToolResult<T>> + Send,
        T: Send,
    {
        self.execute(func, self.config.default_timeout).await
    }

    /// Check resource usage
    fn check_resource_usage(&self) -> ToolResult<()> {
        let memory_usage = self.get_memory_usage();

        if memory_usage > self.config.max_memory {
            warn!(
                "Memory limit exceeded: {} > {}",
                memory_usage, self.config.max_memory
            );
            return Err(ToolError::ResourceLimitExceeded(format!(
                "Memory usage {} exceeds limit {}",
                memory_usage, self.config.max_memory
            )));
        }

        Ok(())
    }

    /// Get current memory usage (platform-specific)
    fn get_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<usize>() {
                                return kb_val * 1024;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, we can use task_info
            // For now, return 0 as a placeholder
            // In production, use proper system calls
            0
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, use GetProcessMemoryInfo
            // For now, return 0 as a placeholder
            0
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            0
        }
    }

    /// Get sandbox statistics
    pub fn get_stats(&self) -> SandboxStats {
        SandboxStats {
            max_memory: self.config.max_memory,
            current_memory: self.get_memory_usage(),
            default_timeout: self.config.default_timeout,
        }
    }

    /// Execute a command in a sandboxed subprocess
    ///
    /// This provides process-level isolation similar to MIRIX's subprocess execution
    pub async fn execute_command(
        &self,
        command: &str,
        args: &[&str],
        timeout_duration: Duration,
    ) -> ToolResult<CommandOutput> {
        use tokio::process::Command;

        debug!(
            "Executing command '{}' with args {:?} in sandbox",
            command, args
        );

        // Build command with environment variables
        let mut cmd = Command::new(command);
        cmd.args(args);

        // Set working directory if specified
        if let Some(ref working_dir) = self.config.working_directory {
            cmd.current_dir(working_dir);
        }

        // Set environment variables
        if !self.config.environment_variables.is_empty() {
            cmd.env_clear(); // Clear existing environment for isolation
            for (key, value) in &self.config.environment_variables {
                cmd.env(key, value);
            }
        }

        // Execute with timeout
        let child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn process: {e}")))?;

        let output = timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| ToolError::Timeout)?
            .map_err(|e| ToolError::ExecutionFailed(format!("Process execution failed: {e}")))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        })
    }

    /// Validate file system access
    ///
    /// Checks if a path is allowed when filesystem isolation is enabled
    pub fn validate_path_access(&self, path: &PathBuf) -> ToolResult<()> {
        if !self.config.enable_filesystem_isolation {
            return Ok(());
        }

        // Check if path is in allowed paths
        for allowed_path in &self.config.allowed_paths {
            if path.starts_with(allowed_path) {
                return Ok(());
            }
        }

        Err(ToolError::PermissionDenied(format!(
            "Access to path {path:?} is not allowed"
        )))
    }
}

/// Sandbox statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxStats {
    /// Maximum memory limit
    pub max_memory: usize,
    /// Current memory usage
    pub current_memory: usize,
    /// Default timeout
    pub default_timeout: Duration,
}

/// Command execution output
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandOutput {
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Whether the command succeeded
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_sandbox_success() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute(async { Ok::<i32, ToolError>(42) }, Duration::from_secs(1))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_sandbox_timeout() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute(
                async {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    Ok::<i32, ToolError>(42)
                },
                Duration::from_millis(100),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::Timeout));
    }

    #[tokio::test]
    async fn test_sandbox_error() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute(
                async {
                    Err::<i32, ToolError>(ToolError::ExecutionFailed("test error".to_string()))
                },
                Duration::from_secs(1),
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::ExecutionFailed(_)));
    }

    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig {
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_cpu_time: Some(60),
            default_timeout: Duration::from_secs(60),
            enable_monitoring: true,
            enable_network_isolation: false,
            working_directory: None,
            environment_variables: HashMap::new(),
            enable_filesystem_isolation: false,
            allowed_paths: Vec::new(),
        };

        let sandbox = SandboxManager::new(config.clone());
        let stats = sandbox.get_stats();

        assert_eq!(stats.max_memory, config.max_memory);
        assert_eq!(stats.default_timeout, config.default_timeout);
    }

    #[tokio::test]
    async fn test_execute_default() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute_default(async { Ok::<String, ToolError>("success".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_command_execution() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute_command("echo", &["hello"], Duration::from_secs(5))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_command_timeout() {
        let sandbox = SandboxManager::default();

        let result = sandbox
            .execute_command("sleep", &["10"], Duration::from_millis(100))
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::Timeout));
    }

    #[test]
    fn test_filesystem_isolation() {
        let mut config = SandboxConfig::default();
        config.enable_filesystem_isolation = true;
        config.allowed_paths = vec![PathBuf::from("/tmp")];

        let sandbox = SandboxManager::new(config);

        // Allowed path
        assert!(sandbox
            .validate_path_access(&PathBuf::from("/tmp/test.txt"))
            .is_ok());

        // Disallowed path
        assert!(sandbox
            .validate_path_access(&PathBuf::from("/etc/passwd"))
            .is_err());
    }

    #[tokio::test]
    async fn test_environment_variables() {
        let mut config = SandboxConfig::default();
        config
            .environment_variables
            .insert("TEST_VAR".to_string(), "test_value".to_string());

        let sandbox = SandboxManager::new(config);

        let result = sandbox
            .execute_command("sh", &["-c", "echo $TEST_VAR"], Duration::from_secs(5))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("test_value"));
    }
}
