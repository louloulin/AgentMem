"""
Example 2: Tool Execution

This example demonstrates tool execution:
- Defining tool schemas
- Registering tools
- Executing tools
- Handling tool results
"""

import asyncio
from agentmem.tools import ToolExecutor, ToolSchema, ToolParameter


# Define tool functions
def calculator(operation: str, a: float, b: float) -> float:
    """Simple calculator tool."""
    operations = {
        "add": lambda x, y: x + y,
        "subtract": lambda x, y: x - y,
        "multiply": lambda x, y: x * y,
        "divide": lambda x, y: x / y if y != 0 else float('inf'),
    }
    
    if operation not in operations:
        raise ValueError(f"Unknown operation: {operation}")
    
    return operations[operation](a, b)


async def async_weather(city: str) -> dict:
    """Async weather lookup tool (simulated)."""
    # Simulate API call
    await asyncio.sleep(0.1)
    
    # Mock weather data
    weather_data = {
        "New York": {"temp": 72, "condition": "Sunny"},
        "London": {"temp": 65, "condition": "Cloudy"},
        "Tokyo": {"temp": 78, "condition": "Rainy"},
    }
    
    return weather_data.get(city, {"temp": 70, "condition": "Unknown"})


def string_processor(text: str, operation: str = "upper") -> str:
    """String processing tool."""
    operations = {
        "upper": str.upper,
        "lower": str.lower,
        "title": str.title,
        "reverse": lambda s: s[::-1],
    }
    
    if operation not in operations:
        raise ValueError(f"Unknown operation: {operation}")
    
    return operations[operation](text)


async def main():
    """Main example function."""
    print("🚀 AgentMem Python SDK - Tool Execution Example\n")
    
    # Create tool executor
    executor = ToolExecutor()
    
    # 1. Register calculator tool
    print("1️⃣  Registering calculator tool...")
    calculator_schema = ToolSchema(
        name="calculator",
        description="Perform basic arithmetic operations",
        parameters=[
            ToolParameter(
                name="operation",
                type="string",
                description="Operation: add, subtract, multiply, divide",
                required=True
            ),
            ToolParameter(
                name="a",
                type="number",
                description="First number",
                required=True
            ),
            ToolParameter(
                name="b",
                type="number",
                description="Second number",
                required=True
            ),
        ],
        returns="number"
    )
    executor.register_tool(calculator_schema, calculator)
    print("   ✅ Calculator tool registered\n")
    
    # 2. Register weather tool
    print("2️⃣  Registering weather tool...")
    weather_schema = ToolSchema(
        name="weather",
        description="Get weather information for a city",
        parameters=[
            ToolParameter(
                name="city",
                type="string",
                description="City name",
                required=True
            ),
        ],
        returns="object"
    )
    executor.register_tool(weather_schema, async_weather)
    print("   ✅ Weather tool registered\n")
    
    # 3. Register string processor tool
    print("3️⃣  Registering string processor tool...")
    string_schema = ToolSchema(
        name="string_processor",
        description="Process strings with various operations",
        parameters=[
            ToolParameter(
                name="text",
                type="string",
                description="Text to process",
                required=True
            ),
            ToolParameter(
                name="operation",
                type="string",
                description="Operation: upper, lower, title, reverse",
                required=False,
                default="upper"
            ),
        ],
        returns="string"
    )
    executor.register_tool(string_schema, string_processor)
    print("   ✅ String processor tool registered\n")
    
    # 4. List all tools
    print("4️⃣  Listing all registered tools...")
    tools = executor.list_tools()
    print(f"   📋 Registered tools: {len(tools)}")
    for tool in tools:
        print(f"      - {tool.name}: {tool.description}")
    print()
    
    # 5. Execute calculator tool
    print("5️⃣  Executing calculator tool...")
    result = await executor.execute(
        "calculator",
        {"operation": "add", "a": 15, "b": 27}
    )
    print(f"   ✅ Status: {result.status.value}")
    print(f"   📊 Result: 15 + 27 = {result.output}")
    print(f"   ⏱️  Duration: {result.duration_ms:.2f}ms\n")
    
    # 6. Execute weather tool (async)
    print("6️⃣  Executing weather tool (async)...")
    result = await executor.execute(
        "weather",
        {"city": "Tokyo"}
    )
    print(f"   ✅ Status: {result.status.value}")
    print(f"   🌤️  Weather: {result.output}")
    print(f"   ⏱️  Duration: {result.duration_ms:.2f}ms\n")
    
    # 7. Execute string processor with default parameter
    print("7️⃣  Executing string processor (default operation)...")
    result = await executor.execute(
        "string_processor",
        {"text": "hello world"}
    )
    print(f"   ✅ Status: {result.status.value}")
    print(f"   📝 Result: {result.output}")
    print(f"   ⏱️  Duration: {result.duration_ms:.2f}ms\n")
    
    # 8. Execute string processor with custom operation
    print("8️⃣  Executing string processor (reverse operation)...")
    result = await executor.execute(
        "string_processor",
        {"text": "hello world", "operation": "reverse"}
    )
    print(f"   ✅ Status: {result.status.value}")
    print(f"   📝 Result: {result.output}")
    print(f"   ⏱️  Duration: {result.duration_ms:.2f}ms\n")
    
    # 9. Execute with timeout
    print("9️⃣  Executing with timeout...")
    result = await executor.execute(
        "weather",
        {"city": "London"},
        timeout=0.05  # Very short timeout
    )
    if result.status.value == "failed":
        print(f"   ⚠️  Execution timed out (expected)")
        print(f"   ❌ Error: {result.error}\n")
    else:
        print(f"   ✅ Completed within timeout\n")
    
    # 10. Handle errors
    print("🔟 Handling errors...")
    
    # Missing required parameter
    result = await executor.execute(
        "calculator",
        {"operation": "add", "a": 5}  # Missing 'b'
    )
    print(f"   ❌ Missing parameter: {result.error}")
    
    # Invalid operation
    result = await executor.execute(
        "calculator",
        {"operation": "power", "a": 2, "b": 3}
    )
    print(f"   ❌ Invalid operation: {result.error}")
    
    # Non-existent tool
    result = await executor.execute(
        "non_existent_tool",
        {}
    )
    print(f"   ❌ Tool not found: {result.error}\n")
    
    # 11. Unregister a tool
    print("1️⃣1️⃣  Unregistering calculator tool...")
    success = executor.unregister_tool("calculator")
    if success:
        print(f"   ✅ Calculator tool unregistered")
        remaining_tools = executor.list_tools()
        print(f"   📋 Remaining tools: {len(remaining_tools)}\n")
    
    print("✨ Example completed successfully!")


if __name__ == "__main__":
    asyncio.run(main())

