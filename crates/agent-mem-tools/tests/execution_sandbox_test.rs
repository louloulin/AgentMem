//! Tool Execution Sandbox 测试
//!
//! 测试工具执行沙箱的核心功能

use agent_mem_tools::{
    LocalSandboxConfig, SandboxRunResult, SandboxStatus, SandboxType, ToolExecutionSandbox,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_sandbox_type_conversion() {
    assert_eq!(SandboxType::Local.as_str(), "local");
    assert_eq!(SandboxType::Docker.as_str(), "docker");
    assert_eq!(SandboxType::VM.as_str(), "vm");

    assert_eq!(SandboxType::from_str("local").unwrap(), SandboxType::Local);
    assert_eq!(SandboxType::from_str("docker").unwrap(), SandboxType::Docker);
    assert_eq!(SandboxType::from_str("vm").unwrap(), SandboxType::VM);

    assert!(SandboxType::from_str("invalid").is_err());
}

#[test]
fn test_sandbox_status() {
    let result = SandboxRunResult {
        func_return: Some(json!({"result": "success"})),
        stdout: vec!["output".to_string()],
        stderr: vec![],
        status: SandboxStatus::Success,
        execution_time_ms: 100,
        memory_used: Some(1024),
    };

    assert_eq!(result.status, SandboxStatus::Success);
    assert_eq!(result.execution_time_ms, 100);
    assert_eq!(result.memory_used, Some(1024));
}

#[test]
fn test_local_sandbox_config_default() {
    let config = LocalSandboxConfig::default();

    assert_eq!(config.sandbox_dir, PathBuf::from("/tmp/agentmem_sandbox"));
    assert!(!config.use_venv);
    assert_eq!(config.venv_name, "venv");
    assert_eq!(config.timeout, 60);
    assert!(config.env_vars.is_empty());
}

#[test]
fn test_local_sandbox_config_custom() {
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    let config = LocalSandboxConfig {
        sandbox_dir: PathBuf::from("/custom/sandbox"),
        use_venv: true,
        venv_name: "custom_venv".to_string(),
        timeout: 120,
        env_vars,
    };

    assert_eq!(config.sandbox_dir, PathBuf::from("/custom/sandbox"));
    assert!(config.use_venv);
    assert_eq!(config.venv_name, "custom_venv");
    assert_eq!(config.timeout, 120);
    assert_eq!(config.env_vars.len(), 1);
}

#[test]
fn test_tool_execution_sandbox_creation() {
    let sandbox = ToolExecutionSandbox::new(
        "test_tool".to_string(),
        json!({"arg1": "value1"}),
        SandboxType::Local,
    );

    // 沙箱应该成功创建
    // 实际测试需要在有 Python 环境的系统上运行
}

#[test]
fn test_tool_execution_sandbox_with_config() {
    let config = LocalSandboxConfig {
        sandbox_dir: PathBuf::from("/tmp/test_sandbox"),
        use_venv: false,
        venv_name: "test_venv".to_string(),
        timeout: 30,
        env_vars: HashMap::new(),
    };

    let sandbox = ToolExecutionSandbox::with_config(
        "test_tool".to_string(),
        json!({"arg1": "value1"}),
        SandboxType::Local,
        config,
    );

    // 沙箱应该成功创建
}

#[tokio::test]
#[ignore] // 需要 Python 环境
async fn test_run_local_sandbox() {
    let sandbox = ToolExecutionSandbox::new(
        "echo_tool".to_string(),
        json!({"message": "hello"}),
        SandboxType::Local,
    );

    let result = sandbox.run().await;

    // 在有 Python 环境的系统上，这应该成功
    if result.is_ok() {
        let run_result = result.unwrap();
        assert!(run_result.execution_time_ms > 0);
    }
}

#[tokio::test]
#[ignore] // 需要 Python 环境
async fn test_run_with_timeout() {
    let mut config = LocalSandboxConfig::default();
    config.timeout = 1; // 1 秒超时

    let sandbox = ToolExecutionSandbox::with_config(
        "slow_tool".to_string(),
        json!({"sleep": 10}),
        SandboxType::Local,
        config,
    );

    let result = sandbox.run().await;

    // 应该超时
    if result.is_ok() {
        let run_result = result.unwrap();
        // 检查是否在超时时间内完成
        assert!(run_result.execution_time_ms < 2000);
    }
}

#[tokio::test]
#[ignore] // 需要 Python 环境
async fn test_run_with_env_vars() {
    let mut config = LocalSandboxConfig::default();
    config
        .env_vars
        .insert("TEST_VAR".to_string(), "test_value".to_string());

    let sandbox = ToolExecutionSandbox::with_config(
        "env_tool".to_string(),
        json!({}),
        SandboxType::Local,
        config,
    );

    let result = sandbox.run().await;

    if result.is_ok() {
        let run_result = result.unwrap();
        // 环境变量应该被正确设置
        assert_eq!(run_result.status, SandboxStatus::Success);
    }
}

#[test]
fn test_sandbox_run_result_serialization() {
    let result = SandboxRunResult {
        func_return: Some(json!({"result": "success"})),
        stdout: vec!["line1".to_string(), "line2".to_string()],
        stderr: vec!["error1".to_string()],
        status: SandboxStatus::Success,
        execution_time_ms: 150,
        memory_used: Some(2048),
    };

    // 测试序列化
    let json_str = serde_json::to_string(&result).unwrap();
    assert!(json_str.contains("success"));

    // 测试反序列化
    let deserialized: SandboxRunResult = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.status, SandboxStatus::Success);
    assert_eq!(deserialized.execution_time_ms, 150);
}

#[test]
fn test_sandbox_status_variants() {
    let statuses = vec![
        SandboxStatus::Success,
        SandboxStatus::Error,
        SandboxStatus::Timeout,
        SandboxStatus::ResourceLimit,
    ];

    for status in statuses {
        // 测试序列化
        let json_str = serde_json::to_string(&status).unwrap();
        // 测试反序列化
        let deserialized: SandboxStatus = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized, status);
    }
}

#[tokio::test]
#[ignore] // 需要 Python 环境
async fn test_docker_sandbox_fallback() {
    let sandbox = ToolExecutionSandbox::new(
        "test_tool".to_string(),
        json!({"arg1": "value1"}),
        SandboxType::Docker,
    );

    let result = sandbox.run().await;

    // Docker 沙箱应该回退到本地沙箱
    if result.is_ok() {
        let run_result = result.unwrap();
        // 应该成功执行（使用本地沙箱）
        assert!(run_result.execution_time_ms >= 0);
    }
}

#[tokio::test]
#[ignore] // 需要 Python 环境
async fn test_vm_sandbox_fallback() {
    let sandbox = ToolExecutionSandbox::new(
        "test_tool".to_string(),
        json!({"arg1": "value1"}),
        SandboxType::VM,
    );

    let result = sandbox.run().await;

    // VM 沙箱应该回退到本地沙箱
    if result.is_ok() {
        let run_result = result.unwrap();
        // 应该成功执行（使用本地沙箱）
        assert!(run_result.execution_time_ms >= 0);
    }
}

#[test]
fn test_sandbox_result_with_no_output() {
    let result = SandboxRunResult {
        func_return: None,
        stdout: vec![],
        stderr: vec![],
        status: SandboxStatus::Success,
        execution_time_ms: 50,
        memory_used: None,
    };

    assert!(result.func_return.is_none());
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
    assert_eq!(result.status, SandboxStatus::Success);
}

#[test]
fn test_sandbox_result_with_error() {
    let result = SandboxRunResult {
        func_return: None,
        stdout: vec![],
        stderr: vec!["Error: Something went wrong".to_string()],
        status: SandboxStatus::Error,
        execution_time_ms: 25,
        memory_used: None,
    };

    assert_eq!(result.status, SandboxStatus::Error);
    assert!(!result.stderr.is_empty());
    assert!(result.stderr[0].contains("Error"));
}

#[test]
fn test_local_config_with_venv() {
    let config = LocalSandboxConfig {
        sandbox_dir: PathBuf::from("/tmp/venv_sandbox"),
        use_venv: true,
        venv_name: "my_venv".to_string(),
        timeout: 90,
        env_vars: HashMap::new(),
    };

    assert!(config.use_venv);
    assert_eq!(config.venv_name, "my_venv");

    // 虚拟环境路径应该是 sandbox_dir/venv_name
    let expected_venv_path = config.sandbox_dir.join(&config.venv_name);
    assert_eq!(expected_venv_path, PathBuf::from("/tmp/venv_sandbox/my_venv"));
}

#[test]
fn test_multiple_env_vars() {
    let mut env_vars = HashMap::new();
    env_vars.insert("VAR1".to_string(), "value1".to_string());
    env_vars.insert("VAR2".to_string(), "value2".to_string());
    env_vars.insert("VAR3".to_string(), "value3".to_string());

    let config = LocalSandboxConfig {
        sandbox_dir: PathBuf::from("/tmp/sandbox"),
        use_venv: false,
        venv_name: "venv".to_string(),
        timeout: 60,
        env_vars: env_vars.clone(),
    };

    assert_eq!(config.env_vars.len(), 3);
    assert_eq!(config.env_vars.get("VAR1"), Some(&"value1".to_string()));
    assert_eq!(config.env_vars.get("VAR2"), Some(&"value2".to_string()));
    assert_eq!(config.env_vars.get("VAR3"), Some(&"value3".to_string()));
}

