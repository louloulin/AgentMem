//! # 程序性记忆演示
//!
//! 展示 AgentMem 程序性记忆功能，包括：
//! - 工作流创建和执行
//! - 任务链管理
//! - 步骤序列处理
//! - 过程记忆存储

use agent_mem_compat::{
    Mem0Client, StepStatus, StepType, Task, TaskPriority, WorkflowStep,
};
use agent_mem_traits::Session;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,procedural_memory_demo=debug,agent_mem_compat=debug")
        .init();

    info!("🚀 启动程序性记忆演示");

    // 创建 Mem0Client
    let client = Mem0Client::new().await?;
    info!("✅ Mem0Client 初始化成功");

    // 创建会话
    let session = Session::new()
        .with_user_id(Some("demo_user".to_string()))
        .with_agent_id(Some("demo_agent".to_string()));

    println!("\n🎯 演示 1: 创建和执行工作流");
    demo_workflow_creation_and_execution(&client, &session).await?;

    println!("\n🎯 演示 2: 任务链管理");
    demo_task_chain_management(&client).await?;

    println!("\n🎯 演示 3: 复杂工作流执行");
    demo_complex_workflow(&client, &session).await?;

    println!("\n🎯 演示 4: 工作流列表和管理");
    demo_workflow_listing(&client).await?;

    println!("\n✅ 所有程序性记忆演示完成！");
    Ok(())
}

/// 演示工作流创建和执行
async fn demo_workflow_creation_and_execution(
    client: &Mem0Client,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("创建简单工作流");

    // 创建工作流步骤
    let steps = vec![
        WorkflowStep {
            id: "step_1".to_string(),
            name: "初始化".to_string(),
            description: "初始化工作流环境".to_string(),
            step_type: StepType::Action,
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("action".to_string(), json!("initialize"));
                inputs
            },
            outputs: HashMap::new(),
            prerequisites: vec![],
            next_steps: vec!["step_2".to_string()],
            status: StepStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_time_ms: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        },
        WorkflowStep {
            id: "step_2".to_string(),
            name: "数据处理".to_string(),
            description: "处理输入数据".to_string(),
            step_type: StepType::Custom("data_processing".to_string()),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("processing_type".to_string(), json!("transform"));
                inputs.insert("data_source".to_string(), json!("user_input"));
                inputs
            },
            outputs: HashMap::new(),
            prerequisites: vec!["step_1".to_string()],
            next_steps: vec!["step_3".to_string()],
            status: StepStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_time_ms: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        },
        WorkflowStep {
            id: "step_3".to_string(),
            name: "结果通知".to_string(),
            description: "发送处理结果通知".to_string(),
            step_type: StepType::Custom("notification".to_string()),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("message".to_string(), json!("数据处理完成"));
                inputs.insert("recipient".to_string(), json!("demo_user"));
                inputs
            },
            outputs: HashMap::new(),
            prerequisites: vec!["step_2".to_string()],
            next_steps: vec![],
            status: StepStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_time_ms: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        },
    ];

    // 创建工作流
    let workflow_id = client
        .create_workflow(
            "数据处理工作流".to_string(),
            "演示数据处理的完整工作流程".to_string(),
            steps,
            "demo_user".to_string(),
            vec!["demo".to_string(), "data_processing".to_string()],
        )
        .await?;

    println!("✅ 工作流创建成功: {workflow_id}");

    // 开始执行工作流
    let execution_id = client
        .start_workflow_execution(
            workflow_id.clone(),
            "demo_executor".to_string(),
            session.clone(),
            Some({
                let mut context = HashMap::new();
                context.insert("user_id".to_string(), json!("demo_user"));
                context.insert("start_time".to_string(), json!(Utc::now().to_rfc3339()));
                context
            }),
        )
        .await?;

    println!("✅ 工作流执行开始: {execution_id}");

    // 执行工作流步骤
    for i in 1..=3 {
        println!("\n📋 执行第 {i} 步");

        let result = client.execute_next_step(&execution_id).await?;

        if result.success {
            println!("  ✅ 步骤 {} 执行成功: {}", result.step_id, result.message);
            println!("  ⏱️ 执行时间: {}ms", result.execution_time_ms);
        } else {
            println!("  ❌ 步骤 {} 执行失败: {}", result.step_id, result.message);
        }

        // 获取执行状态
        if let Some(execution) = client.get_execution_status(&execution_id).await? {
            println!("  📊 执行状态: {:?}", execution.status);
            println!("  📈 已完成步骤: {}", execution.completed_steps.len());
        }
    }

    // 获取最终执行状态
    if let Some(execution) = client.get_execution_status(&execution_id).await? {
        println!("\n🎉 工作流执行完成!");
        println!("  状态: {:?}", execution.status);
        println!("  已完成步骤: {:?}", execution.completed_steps);
        println!("  失败步骤: {:?}", execution.failed_steps);
    }

    Ok(())
}

/// 演示任务链管理
async fn demo_task_chain_management(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("创建和管理任务链");

    // 创建任务列表
    let tasks = vec![
        Task {
            id: Uuid::new_v4().to_string(),
            name: "数据收集".to_string(),
            description: "从多个数据源收集信息".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("type".to_string(), json!("data_processing"));
                params.insert("processing_type".to_string(), json!("collect"));
                params.insert("sources".to_string(), json!(["api", "database", "files"]));
                params
            },
            status: agent_mem_compat::TaskStatus::Pending,
            estimated_duration: Some(30),
            actual_duration: None,
            priority: TaskPriority::High,
        },
        Task {
            id: Uuid::new_v4().to_string(),
            name: "数据清洗".to_string(),
            description: "清洗和标准化收集的数据".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("type".to_string(), json!("data_processing"));
                params.insert("processing_type".to_string(), json!("clean"));
                params.insert(
                    "rules".to_string(),
                    json!(["remove_duplicates", "validate_format"]),
                );
                params
            },
            status: agent_mem_compat::TaskStatus::Pending,
            estimated_duration: Some(45),
            actual_duration: None,
            priority: TaskPriority::High,
        },
        Task {
            id: Uuid::new_v4().to_string(),
            name: "数据分析".to_string(),
            description: "对清洗后的数据进行分析".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("type".to_string(), json!("data_processing"));
                params.insert("processing_type".to_string(), json!("analyze"));
                params.insert("algorithms".to_string(), json!(["statistical", "ml"]));
                params
            },
            status: agent_mem_compat::TaskStatus::Pending,
            estimated_duration: Some(60),
            actual_duration: None,
            priority: TaskPriority::Medium,
        },
        Task {
            id: Uuid::new_v4().to_string(),
            name: "生成报告".to_string(),
            description: "生成分析结果报告".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("type".to_string(), json!("default"));
                params.insert("format".to_string(), json!("pdf"));
                params.insert("template".to_string(), json!("standard"));
                params
            },
            status: agent_mem_compat::TaskStatus::Pending,
            estimated_duration: Some(20),
            actual_duration: None,
            priority: TaskPriority::Low,
        },
    ];

    // 创建任务链
    let chain_id = client
        .create_task_chain("数据处理任务链".to_string(), tasks)
        .await?;

    println!("✅ 任务链创建成功: {chain_id}");

    // 获取任务链信息
    if let Some(task_chain) = client.get_task_chain(&chain_id).await? {
        println!("📋 任务链信息:");
        println!("  名称: {}", task_chain.name);
        println!("  任务数量: {}", task_chain.tasks.len());
        println!("  状态: {:?}", task_chain.status);
        println!("  当前任务索引: {}", task_chain.current_task_index);
    }

    // 执行任务链中的任务
    for i in 1..=4 {
        println!("\n🔄 执行任务 {i}");

        let result = client.execute_next_task(&chain_id).await?;

        if result.success {
            println!("  ✅ 任务 {} 执行成功: {}", result.task_id, result.message);
            println!("  ⏱️ 执行时间: {}秒", result.duration);
        } else {
            println!("  ❌ 任务 {} 执行失败: {}", result.task_id, result.message);
            break;
        }

        // 获取更新后的任务链状态
        if let Some(task_chain) = client.get_task_chain(&chain_id).await? {
            println!("  📊 任务链状态: {:?}", task_chain.status);
            println!("  📈 当前任务索引: {}", task_chain.current_task_index);
        }
    }

    // 获取最终任务链状态
    if let Some(task_chain) = client.get_task_chain(&chain_id).await? {
        println!("\n🎉 任务链执行完成!");
        println!("  最终状态: {:?}", task_chain.status);
        println!("  总任务数: {}", task_chain.tasks.len());

        // 统计任务状态
        let completed_count = task_chain
            .tasks
            .iter()
            .filter(|t| t.status == agent_mem_compat::TaskStatus::Completed)
            .count();
        let failed_count = task_chain
            .tasks
            .iter()
            .filter(|t| t.status == agent_mem_compat::TaskStatus::Failed)
            .count();

        println!("  已完成任务: {completed_count}");
        println!("  失败任务: {failed_count}");
    }

    Ok(())
}

/// 演示复杂工作流执行
async fn demo_complex_workflow(
    client: &Mem0Client,
    session: &Session,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("创建复杂工作流，包含决策和循环");

    // 创建复杂工作流步骤
    let steps = vec![
        WorkflowStep {
            id: "init".to_string(),
            name: "初始化".to_string(),
            description: "初始化工作流参数".to_string(),
            step_type: StepType::Action,
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("counter".to_string(), json!(0));
                inputs
            },
            outputs: HashMap::new(),
            prerequisites: vec![],
            next_steps: vec!["decision".to_string()],
            status: StepStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_time_ms: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        },
        WorkflowStep {
            id: "decision".to_string(),
            name: "决策步骤".to_string(),
            description: "根据条件决定下一步操作".to_string(),
            step_type: StepType::Decision,
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("condition".to_string(), json!("true"));
                inputs
            },
            outputs: HashMap::new(),
            prerequisites: vec!["init".to_string()],
            next_steps: vec!["memory_op".to_string()],
            status: StepStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_time_ms: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        },
        WorkflowStep {
            id: "memory_op".to_string(),
            name: "记忆操作".to_string(),
            description: "执行记忆相关操作".to_string(),
            step_type: StepType::Custom("memory_operation".to_string()),
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert("operation".to_string(), json!("create"));
                inputs.insert("content".to_string(), json!("工作流执行记录"));
                inputs
            },
            outputs: HashMap::new(),
            prerequisites: vec!["decision".to_string()],
            next_steps: vec![],
            status: StepStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            execution_time_ms: None,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        },
    ];

    // 创建复杂工作流
    let workflow_id = client
        .create_workflow(
            "复杂处理工作流".to_string(),
            "包含决策和记忆操作的工作流".to_string(),
            steps,
            "demo_user".to_string(),
            vec!["demo".to_string(), "complex".to_string()],
        )
        .await?;

    println!("✅ 复杂工作流创建成功: {workflow_id}");

    // 开始执行工作流
    let execution_id = client
        .start_workflow_execution(
            workflow_id.clone(),
            "advanced_executor".to_string(),
            session.clone(),
            Some({
                let mut context = HashMap::new();
                context.insert("workflow_type".to_string(), json!("complex"));
                context
            }),
        )
        .await?;

    println!("✅ 复杂工作流执行开始: {execution_id}");

    // 执行所有步骤
    let step_names = ["初始化", "决策步骤", "记忆操作"];

    for (i, step_name) in step_names.iter().enumerate() {
        println!("\n🔄 执行步骤 {}: {}", i + 1, step_name);

        let result = client.execute_next_step(&execution_id).await?;

        if result.success {
            println!("  ✅ 步骤执行成功: {}", result.message);
            println!("  ⏱️ 执行时间: {}ms", result.execution_time_ms);
        } else {
            println!("  ❌ 步骤执行失败: {}", result.message);
        }
    }

    println!("\n🎉 复杂工作流执行完成!");
    Ok(())
}

/// 演示工作流列表和管理
async fn demo_workflow_listing(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    info!("演示工作流列表和管理功能");

    // 列出所有工作流
    println!("📋 列出所有工作流:");
    let all_workflows = client.list_workflows(None).await?;

    for (i, workflow) in all_workflows.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, workflow.name, workflow.id);
        println!("     描述: {}", workflow.description);
        println!("     步骤数: {}", workflow.steps.len());
        println!("     标签: {:?}", workflow.tags);
        println!();
    }

    // 按标签过滤工作流
    println!("🏷️ 按标签 'demo' 过滤工作流:");
    let demo_workflows = client
        .list_workflows(Some(vec!["demo".to_string()]))
        .await?;

    for workflow in demo_workflows {
        println!("  - {} (标签: {:?})", workflow.name, workflow.tags);
    }

    // 列出所有任务链
    println!("\n📋 列出所有任务链:");
    let task_chains = client.list_task_chains().await?;

    for (i, chain) in task_chains.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, chain.name, chain.id);
        println!("     状态: {:?}", chain.status);
        println!("     任务数: {}", chain.tasks.len());
        println!();
    }

    Ok(())
}
