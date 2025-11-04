# AgentMem Plugin SDK

Plugin SDK for developing AgentMem WebAssembly plugins.

## Features

- 🎯 Type-safe plugin development
- 🎯 Standard plugin lifecycle management
- 🎯 Host function bindings for AgentMem functionality
- 🎯 Convenience macros for common tasks

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
agent-mem-plugin-sdk = "0.1"
extism-pdk = "1.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Example

See `examples/` directory for complete examples.

```rust
use agent_mem_plugin_sdk::*;
use extism_pdk::*;

#[plugin_fn]
pub fn process(input: String) -> FnResult<String> {
    let memory: Memory = serde_json::from_str(&input)?;
    
    // Process memory...
    
    Ok(serde_json::to_string(&memory)?)
}

#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = plugin_metadata!(
        name: "my-plugin",
        version: "0.1.0",
        description: "My custom plugin",
        author: "Your Name",
        plugin_type: PluginType::MemoryProcessor,
        capabilities: [Capability::MemoryAccess]
    );
    
    Ok(serde_json::to_string(&metadata)?)
}
```

## License

MIT OR Apache-2.0

