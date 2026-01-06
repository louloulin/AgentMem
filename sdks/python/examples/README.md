# AgentMem Python SDK - Examples

This directory contains comprehensive examples demonstrating the AgentMem Python SDK features.

## Prerequisites

```bash
# Install the SDK
pip install agentmem

# Or install from source
cd agentmen/sdks/python
pip install -e .
```

## Environment Setup

Set the following environment variables:

```bash
export AGENTMEM_API_KEY="your-api-key"
export AGENTMEM_BASE_URL="http://localhost:8080"  # Optional, defaults to production
```

## Examples

### 1. Basic Memory Management (`01_basic_memory.py`)

Demonstrates core memory operations:
- Adding memories (episodic, semantic, procedural)
- Retrieving memories by ID
- Searching memories with filters
- Updating memory properties
- Batch operations
- Getting statistics
- Deleting memories

**Run:**
```bash
python 01_basic_memory.py
```

**Key Features:**
- ✅ Memory CRUD operations
- ✅ Text and vector search
- ✅ Memory type filtering
- ✅ Importance-based filtering
- ✅ Batch operations
- ✅ Statistics and analytics

### 2. Tool Execution (`02_tool_execution.py`)

Demonstrates tool execution framework:
- Defining tool schemas
- Registering sync and async tools
- Executing tools with parameters
- Handling tool results and errors
- Tool timeout management

**Run:**
```bash
python 02_tool_execution.py
```

**Key Features:**
- ✅ Sync and async tool support
- ✅ Parameter validation
- ✅ Error handling
- ✅ Timeout control
- ✅ Tool discovery

**Tools Demonstrated:**
- Calculator (sync)
- Weather lookup (async)
- String processor (with defaults)

### 3. Observability (`03_observability.py`)

Demonstrates monitoring and metrics:
- Collecting metrics (counters, gauges, histograms)
- Performance tracking
- Operation statistics
- Context manager usage

**Run:**
```bash
python 03_observability.py
```

**Key Features:**
- ✅ Counter metrics
- ✅ Gauge metrics
- ✅ Histogram metrics with percentiles
- ✅ Performance tracking
- ✅ Context manager for automatic tracking
- ✅ Metric queries

**Metrics Collected:**
- Request counts by endpoint/method
- Error counts by type
- Active connections
- Memory usage
- Request duration (P50, P95, P99)
- Database query performance

### 4. Complete Application (`04_complete_application.py`)

Demonstrates a complete smart assistant application:
- Integrating memory, tools, and observability
- Command processing
- Error handling
- Statistics reporting

**Run:**
```bash
python 04_complete_application.py
```

**Key Features:**
- ✅ Complete application architecture
- ✅ Command parsing and routing
- ✅ Tool integration
- ✅ Memory management
- ✅ Comprehensive monitoring
- ✅ Error handling

**Commands Supported:**
- `calculate <operation> <a> <b>` - Perform calculations
- `note <content>` - Take a note
- `remember <content>` - Store a memory
- `recall <query>` - Search memories

## Running All Examples

```bash
# Run all examples in sequence
for example in 01_*.py 02_*.py 03_*.py 04_*.py; do
    echo "Running $example..."
    python "$example"
    echo ""
done
```

## Example Output

### Basic Memory Management
```
🚀 AgentMem Python SDK - Basic Memory Management Example

1️⃣  Adding memories...
   ✅ Added semantic memory: mem_abc123
   ✅ Added episodic memory: mem_def456
   ✅ Added procedural memory: mem_ghi789

2️⃣  Retrieving memory...
   📝 Content: The user prefers dark mode in the UI
   🏷️  Type: semantic
   ⭐ Importance: 0.8
   📊 Metadata: {'category': 'user_preferences', 'ui': 'theme'}

...
```

### Tool Execution
```
🚀 AgentMem Python SDK - Tool Execution Example

1️⃣  Registering calculator tool...
   ✅ Calculator tool registered

...

5️⃣  Executing calculator tool...
   ✅ Status: success
   📊 Result: 15 + 27 = 42
   ⏱️  Duration: 0.15ms

...
```

### Observability
```
🚀 AgentMem Python SDK - Observability Example

1️⃣  Collecting basic metrics...
   ✅ Metrics recorded

...

4️⃣  Displaying collected metrics...
   📊 Counters:
      requests_total|method=GET,endpoint=/api/memories: 2
      requests_total|method=POST,endpoint=/api/memories: 1

   📊 Histograms:
      request_duration_seconds:
         Count: 3
         Avg: 0.0250
         P95: 0.0320
         P99: 0.0320

...
```

## Customization

### Using Custom Configuration

```python
from agentmem import Config, AgentMemClient

config = Config(
    api_key="your-api-key",
    api_base_url="https://your-server.com",
    timeout=60,
    max_retries=5,
    enable_caching=True,
    cache_ttl=600,
)

async with AgentMemClient(config) as client:
    # Your code here
    pass
```

### Adding Custom Tools

```python
from agentmem.tools import ToolExecutor, ToolSchema, ToolParameter

def my_custom_tool(param1: str, param2: int) -> str:
    """Your custom tool implementation."""
    return f"Processed: {param1} with {param2}"

executor = ToolExecutor()
executor.register_tool(
    ToolSchema(
        name="my_tool",
        description="My custom tool",
        parameters=[
            ToolParameter("param1", "string", "First parameter"),
            ToolParameter("param2", "number", "Second parameter"),
        ],
        returns="string"
    ),
    my_custom_tool
)

result = await executor.execute("my_tool", {"param1": "test", "param2": 42})
```

### Custom Metrics

```python
from agentmem.observability import MetricsCollector

metrics = MetricsCollector()

# Record custom metrics
metrics.increment("my_counter", labels={"type": "custom"})
metrics.set_gauge("my_gauge", 100)
metrics.record_histogram("my_duration", 0.123)

# Query metrics
value = metrics.get_counter("my_counter", labels={"type": "custom"})
stats = metrics.get_histogram_stats("my_duration")
```

## Troubleshooting

### Connection Errors

If you get connection errors:
1. Check that AgentMem server is running
2. Verify the `AGENTMEM_BASE_URL` is correct
3. Check your API key is valid

### Import Errors

If you get import errors:
```bash
# Reinstall the SDK
pip uninstall agentmem
pip install agentmem

# Or install from source
cd agentmen/sdks/python
pip install -e .
```

### Type Checking

To enable type checking:
```bash
pip install mypy
mypy your_script.py
```

## Next Steps

- Read the [API Documentation](../README.md)
- Explore the [SDK Source Code](../agentmem/)
- Check out the [Test Suite](../tests/)
- Join our [Discord Community](https://discord.gg/agentmem)

## Support

- 📖 [Documentation](https://docs.agentmem.dev)
- 💬 [Discord](https://discord.gg/agentmem)
- 🐛 [Issues](https://github.com/agentmem/agentmem/issues)
- 📧 [Email](mailto:support@agentmem.dev)

