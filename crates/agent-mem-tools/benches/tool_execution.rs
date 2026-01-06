//! Benchmarks for tool execution

use agent_mem_tools::{builtin::CalculatorTool, ExecutionContext, ToolExecutor};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

fn bench_tool_registration(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("tool_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let executor = ToolExecutor::new();
                let tool = Arc::new(CalculatorTool);
                executor.register_tool(tool).await.unwrap();
            });
        });
    });
}

fn bench_tool_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let executor = rt.block_on(async {
        let executor = ToolExecutor::new();
        let tool = Arc::new(CalculatorTool);
        executor.register_tool(tool).await.unwrap();
        executor
            .permissions()
            .assign_role("test_user", "admin")
            .await;
        executor
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    c.bench_function("tool_execution_simple", |b| {
        b.iter(|| {
            rt.block_on(async {
                executor
                    .execute_tool(
                        black_box("calculator"),
                        black_box(json!({
                            "operation": "add",
                            "a": 10.0,
                            "b": 20.0
                        })),
                        &context,
                    )
                    .await
                    .unwrap();
            });
        });
    });
}

fn bench_schema_validation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let schema = rt.block_on(async {
        let executor = ToolExecutor::new();
        let tool = Arc::new(CalculatorTool);
        executor.register_tool(tool).await.unwrap();
        executor.get_schema("calculator").await.unwrap()
    });

    let args = json!({
        "operation": "add",
        "a": 10.0,
        "b": 20.0
    });

    c.bench_function("schema_validation", |b| {
        b.iter(|| {
            schema.validate(black_box(&args)).unwrap();
        });
    });
}

fn bench_permission_check(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let executor = rt.block_on(async {
        let executor = ToolExecutor::new();
        executor
            .permissions()
            .assign_role("test_user", "admin")
            .await;
        executor
    });

    c.bench_function("permission_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                executor
                    .permissions()
                    .check_permission(black_box("calculator"), black_box("test_user"))
                    .await
                    .unwrap();
            });
        });
    });
}

criterion_group!(
    benches,
    bench_tool_registration,
    bench_tool_execution,
    bench_schema_validation,
    bench_permission_check
);
criterion_main!(benches);
