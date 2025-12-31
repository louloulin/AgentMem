//! Performance benchmarks for the plugin system

use agent_mem_plugin_sdk::{Memory, PluginConfig, PluginMetadata, PluginType};
use agent_mem_plugins::*;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Get the path to compiled WASM plugins
fn get_wasm_plugin_path(plugin_name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    workspace_root.join(format!("target/wasm32-wasip1/release/{plugin_name}.wasm"))
}

/// Create a test memory object
fn create_test_memory(id: &str, content: &str) -> Memory {
    let now = Utc::now().to_rfc3339();
    Memory {
        id: id.to_string(),
        content: content.to_string(),
        memory_type: "test".to_string(),
        user_id: "bench_user".to_string(),
        agent_id: None,
        metadata: HashMap::new(),
        created_at: now.clone(),
        updated_at: now,
    }
}

#[tokio::main]
async fn main() {
    println!("\n🚀 AgentMem Plugin System Performance Benchmarks\n");
    println!("================================================\n");

    // Check if WASM files exist
    let hello_path = get_wasm_plugin_path("hello_plugin");
    let memory_processor_path = get_wasm_plugin_path("memory_processor_plugin");
    let code_analyzer_path = get_wasm_plugin_path("code_analyzer_plugin");

    if !hello_path.exists() || !memory_processor_path.exists() || !code_analyzer_path.exists() {
        eprintln!("⚠️  WASM plugins not found. Run: ./build_plugins.sh");
        return;
    }

    // Benchmark 1: Plugin Loading
    println!("📊 Benchmark 1: Plugin Loading Performance");
    benchmark_plugin_loading().await;
    println!();

    // Benchmark 2: Plugin Execution
    println!("📊 Benchmark 2: Plugin Execution Performance");
    benchmark_plugin_execution().await;
    println!();

    // Benchmark 3: Cache Performance
    println!("📊 Benchmark 3: Cache Performance");
    benchmark_cache_performance().await;
    println!();

    // Benchmark 4: Concurrent Execution
    println!("📊 Benchmark 4: Concurrent Execution");
    benchmark_concurrent_execution().await;
    println!();

    // Benchmark 5: Memory Processing
    println!("📊 Benchmark 5: Memory Processing Throughput");
    benchmark_memory_processing().await;
    println!();

    println!("✅ All benchmarks completed!\n");
}

async fn benchmark_plugin_loading() {
    let manager = PluginManager::new(10);
    let wasm_path = get_wasm_plugin_path("hello_plugin");

    let plugin = RegisteredPlugin {
        id: "bench-hello".to_string(),
        metadata: PluginMetadata {
            name: "Benchmark Hello".to_string(),
            version: "0.1.0".to_string(),
            description: "Benchmark plugin".to_string(),
            author: "Bench".to_string(),
            plugin_type: PluginType::Custom("test".to_string()),
            required_capabilities: vec![],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    // Measure registration time
    let start = Instant::now();
    manager.register(plugin).await.unwrap();
    let register_time = start.elapsed();

    // Measure first load time
    let start = Instant::now();
    let _ = manager.get_plugin("bench-hello").await.unwrap();
    let first_load_time = start.elapsed();

    // Measure cached load time
    let start = Instant::now();
    let _ = manager.get_plugin("bench-hello").await.unwrap();
    let cached_load_time = start.elapsed();

    println!("  Registration time:     {register_time:?}");
    println!("  First load time:       {first_load_time:?}");
    println!("  Cached load time:      {cached_load_time:?}");
    println!(
        "  Cache speedup:         {:.2}x",
        first_load_time.as_micros() as f64 / cached_load_time.as_micros() as f64
    );
}

async fn benchmark_plugin_execution() {
    let manager = PluginManager::new(10);
    let wasm_path = get_wasm_plugin_path("hello_plugin");

    let plugin = RegisteredPlugin {
        id: "bench-exec".to_string(),
        metadata: PluginMetadata {
            name: "Benchmark Exec".to_string(),
            version: "0.1.0".to_string(),
            description: "Benchmark plugin".to_string(),
            author: "Bench".to_string(),
            plugin_type: PluginType::Custom("test".to_string()),
            required_capabilities: vec![],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    manager.register(plugin).await.unwrap();

    // Warm up
    let input = serde_json::json!({"message": "warmup"});
    let _ = manager
        .call_plugin("bench-exec", "hello", &input.to_string())
        .await;

    // Run multiple iterations
    let iterations = 1000;
    let start = Instant::now();

    for i in 0..iterations {
        let input = serde_json::json!({"message": format!("test-{}", i)});
        let _ = manager
            .call_plugin("bench-exec", "hello", &input.to_string())
            .await
            .unwrap();
    }

    let total_time = start.elapsed();
    let avg_time = total_time / iterations;
    let throughput = iterations as f64 / total_time.as_secs_f64();

    println!("  Iterations:            {iterations}");
    println!("  Total time:            {total_time:?}");
    println!("  Average time:          {avg_time:?}");
    println!("  Throughput:            {throughput:.2} calls/sec");
}

async fn benchmark_cache_performance() {
    let cache_sizes = vec![1, 5, 10, 50];

    for cache_size in cache_sizes {
        let manager = PluginManager::new(cache_size);
        let wasm_path = get_wasm_plugin_path("hello_plugin");

        // Register multiple plugins
        for i in 0..10 {
            let plugin = RegisteredPlugin {
                id: format!("cache-test-{i}"),
                metadata: PluginMetadata {
                    name: format!("Cache Test {i}"),
                    version: "0.1.0".to_string(),
                    description: "Test".to_string(),
                    author: "Test".to_string(),
                    plugin_type: PluginType::Custom("test".to_string()),
                    required_capabilities: vec![],
                    config_schema: None,
                },
                path: wasm_path.to_string_lossy().to_string(),
                status: PluginStatus::Registered,
                config: PluginConfig::default(),
                registered_at: Utc::now(),
                last_loaded_at: None,
            };
            manager.register(plugin).await.unwrap();
        }

        // Test cache hit rate
        let iterations = 100;
        let start = Instant::now();

        for i in 0..iterations {
            let plugin_id = format!("cache-test-{}", i % 10);
            let input = serde_json::json!({"message": "test"});
            let _ = manager
                .call_plugin(&plugin_id, "hello", &input.to_string())
                .await;
        }

        let total_time = start.elapsed();
        let avg_time = total_time / iterations;

        println!("  Cache size {cache_size:<2}:  {avg_time:?}/call");
    }
}

async fn benchmark_concurrent_execution() {
    let manager = PluginManager::new(20);
    let wasm_path = get_wasm_plugin_path("hello_plugin");

    let plugin = RegisteredPlugin {
        id: "concurrent-test".to_string(),
        metadata: PluginMetadata {
            name: "Concurrent Test".to_string(),
            version: "0.1.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            plugin_type: PluginType::Custom("test".to_string()),
            required_capabilities: vec![],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    manager.register(plugin).await.unwrap();

    let concurrency_levels = vec![1, 10, 50, 100];

    for concurrency in concurrency_levels {
        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..concurrency {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let input = serde_json::json!({"message": format!("concurrent-{}", i)});
                mgr.call_plugin("concurrent-test", "hello", &input.to_string())
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await.unwrap();
        }

        let total_time = start.elapsed();
        let avg_time = total_time / concurrency;

        println!(
            "  Concurrency {concurrency:<3}:  {total_time:?} total, {avg_time:?} avg"
        );
    }
}

async fn benchmark_memory_processing() {
    let manager = PluginManager::new(10);
    let wasm_path = get_wasm_plugin_path("memory_processor_plugin");

    let plugin = RegisteredPlugin {
        id: "memory-bench".to_string(),
        metadata: PluginMetadata {
            name: "Memory Benchmark".to_string(),
            version: "0.1.0".to_string(),
            description: "Benchmark".to_string(),
            author: "Bench".to_string(),
            plugin_type: PluginType::MemoryProcessor,
            required_capabilities: vec![],
            config_schema: None,
        },
        path: wasm_path.to_string_lossy().to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };

    manager.register(plugin).await.unwrap();

    // Test with different content sizes
    let sizes = vec![100, 500, 1000, 5000];

    for size in sizes {
        let content = "test ".repeat(size / 5); // Approximate size
        let memory = create_test_memory("bench", &content);
        let input = serde_json::to_string(&memory).unwrap();

        let iterations = 100usize;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = manager
                .call_plugin("memory-bench", "process_memory", &input)
                .await
                .unwrap();
        }

        let total_time = start.elapsed();
        let avg_time = total_time / iterations as u32;
        let throughput =
            (content.len() * iterations) as f64 / total_time.as_secs_f64() / 1024.0 / 1024.0;

        println!(
            "  Content size {:<4} chars:  {:?}/call, {:.2} MB/s",
            content.len(),
            avg_time,
            throughput
        );
    }
}
