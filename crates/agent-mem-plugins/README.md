# AgentMem Plugin Manager

Plugin management system for AgentMem - handles loading, lifecycle, and security of WebAssembly plugins.

## Features

- 🎯 Plugin registration and discovery
- 🎯 Dynamic loading and unloading
- 🎯 LRU-based plugin caching
- 🎯 Permission-based security
- 🎯 Sandbox configuration
- 🎯 Host capability system

## Usage

```rust
use agent_mem_plugins::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create plugin manager
    let manager = PluginManager::new(10); // cache size: 10
    
    // Register a plugin
    let plugin = RegisteredPlugin {
        id: "my-plugin".to_string(),
        metadata: /* ... */,
        path: "./plugins/my_plugin.wasm".to_string(),
        status: PluginStatus::Registered,
        config: PluginConfig::default(),
        registered_at: Utc::now(),
        last_loaded_at: None,
    };
    
    manager.register(plugin).await?;
    
    // Call plugin function
    let result = manager
        .call_plugin("my-plugin", "process", r#"{"message": "hello"}"#)
        .await?;
    
    println!("Result: {}", result);
    
    Ok(())
}
```

## Architecture

```
PluginManager
  ├── PluginRegistry (stores plugin metadata)
  ├── PluginLoader (loads WASM modules)
  ├── Plugin Cache (LRU cache of loaded plugins)
  └── Capabilities (host functions for plugins)
      ├── MemoryCapability
      ├── LoggingCapability
      └── ... (more capabilities)
```

## Security

- **Sandbox Isolation**: Plugins run in WebAssembly sandbox
- **Permission System**: Capability-based permissions
- **Resource Limits**: Memory, CPU, I/O limits
- **Safe Host Functions**: Controlled access to AgentMem functionality

## Testing

```bash
# Run tests
cargo test -p agent-mem-plugins

# Run integration tests
cargo test -p agent-mem-plugins --test integration_test
```

## License

MIT OR Apache-2.0

