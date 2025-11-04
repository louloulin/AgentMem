//! Plugin Integration Example
//!
//! This example demonstrates how to use the AgentMem plugin system.
//!
//! To run this example with plugin support:
//! ```bash
//! cargo run --example plugin_integration --features plugins
//! ```

#[cfg(feature = "plugins")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use agent_mem::plugins::{
        PluginManager, PluginMetadata, PluginRegistry, PluginStatus, RegisteredPlugin, PluginType,
        Capability, PluginConfig, ResourceLimits, ResourceMonitor, PluginMonitor,
    };
    use std::sync::Arc;

    println!("=== AgentMem Plugin System Integration Demo ===\n");

    // 1. Create plugin registry
    println!("1. Creating plugin registry...");
    let mut registry = PluginRegistry::new();

    // 2. Register a plugin
    println!("2. Registering memory processor plugin...");
    let metadata = PluginMetadata {
        name: "memory-processor".to_string(),
        version: "0.1.0".to_string(),
        description: "Process and enhance memory content".to_string(),
        author: "AgentMem Team".to_string(),
        plugin_type: PluginType::MemoryProcessor,
        required_capabilities: vec![
            Capability::MemoryAccess,
            Capability::LoggingAccess,
        ],
        config_schema: None,
    };

    let plugin = RegisteredPlugin {
        id: "memory-processor".to_string(),
        metadata: metadata.clone(),
        path: "target/wasm32-wasip1/release/memory_processor_plugin.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: chrono::Utc::now(),
        last_loaded_at: None,
    };

    registry.register(plugin)?;
    println!("   ✓ Plugin registered: {}", metadata.name);

    // 3. List registered plugins
    println!("\n3. Listing registered plugins:");
    let plugins = registry.list();
    for p in plugins {
        println!("   - {} v{} ({})", p.metadata.name, p.metadata.version, p.id);
        println!("     Type: {:?}", p.metadata.plugin_type);
        println!("     Status: {:?}", p.status);
        println!("     Capabilities: {:?}", p.metadata.required_capabilities);
    }

    // 4. Resource monitoring
    println!("\n4. Setting up resource monitoring...");
    let limits = ResourceLimits::default();
    let monitor = Arc::new(ResourceMonitor::new(limits.clone()));
    let usage = monitor.usage();

    println!("   Resource Limits:");
    println!("   - Memory: {} MB", limits.memory.max_heap_bytes / 1024 / 1024);
    println!("   - CPU Time: {} ms", limits.cpu.max_execution_time_ms);
    println!("   - Network Requests: {}", limits.io.max_network_requests);

    // Simulate some resource usage
    usage.start_timing();
    usage.record_allocation(1024 * 1024); // 1 MB
    usage.record_instructions(10000);
    usage.record_network_request();

    println!("\n   Current Usage:");
    println!("   - Memory: {} KB", usage.memory_used() / 1024);
    println!("   - Instructions: {}", usage.total_instructions());
    println!("   - Network Requests: {}", usage.network_requests());

    // Check limits
    match monitor.check_limits() {
        Ok(_) => println!("   ✓ All resource limits OK"),
        Err(e) => println!("   ✗ Resource limit exceeded: {}", e),
    }

    // 5. Plugin execution monitoring
    println!("\n5. Setting up plugin execution monitoring...");
    let plugin_monitor = PluginMonitor::new();

    // Simulate plugin executions
    let tracker1 = plugin_monitor.start_execution("memory-processor");
    std::thread::sleep(std::time::Duration::from_millis(10));
    tracker1.complete();

    let tracker2 = plugin_monitor.start_execution("memory-processor");
    std::thread::sleep(std::time::Duration::from_millis(5));
    tracker2.complete();

    let tracker3 = plugin_monitor.start_execution("memory-processor");
    tracker3.fail("Test error".to_string());

    // Get metrics
    if let Some(metrics) = plugin_monitor.get_metrics("memory-processor") {
        println!("   Plugin Execution Metrics:");
        println!("   - Total Calls: {}", metrics.total_calls);
        println!("   - Successful: {}", metrics.successful_calls);
        println!("   - Failed: {}", metrics.failed_calls);
        println!("   - Success Rate: {:.1}%", metrics.success_rate() * 100.0);
        println!("   - Avg Execution Time: {:?}", metrics.avg_execution_time);
        if let Some(err) = &metrics.last_error {
            println!("   - Last Error: {}", err);
        }
    }

    // 6. Network capability (mock mode)
    println!("\n6. Testing network capability...");
    use agent_mem::plugins::capabilities::{NetworkCapability, HttpRequest, HttpMethod};
    use std::collections::HashMap;

    let network = NetworkCapability::new(true, 100); // mock mode, 100 requests max

    let request = HttpRequest {
        url: "https://api.example.com/data".to_string(),
        method: HttpMethod::GET,
        headers: HashMap::new(),
        body: None,
        timeout_ms: Some(5000),
    };

    match network.http_request(request) {
        Ok(response) => {
            println!("   ✓ HTTP request successful");
            println!("   - Status: {}", response.status);
            println!("   - Body: {}", response.body);
        }
        Err(e) => {
            println!("   ✗ HTTP request failed: {}", e);
        }
    }

    // 7. Plugin Manager
    println!("\n7. Creating plugin manager with cache...");
    let manager = PluginManager::new(10); // LRU cache size 10
    println!("   ✓ Plugin manager created with cache size: 10");

    println!("\n=== Demo Complete ===");
    println!("\nTo use plugins in your AgentMem application:");
    println!("1. Enable the 'plugins' feature in Cargo.toml");
    println!("2. Use `agent_mem::plugins` to access plugin APIs");
    println!("3. Register and load WASM plugins");
    println!("4. Call plugin functions through the manager");

    Ok(())
}

#[cfg(not(feature = "plugins"))]
fn main() {
    println!("Plugin support is not enabled!");
    println!("Run with: cargo run --example plugin_integration --features plugins");
}

