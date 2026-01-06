"""
Example 4: Complete Application

This example demonstrates a complete application integrating:
- Memory management
- Tool execution
- Observability
- Error handling
"""

import asyncio
import os
from datetime import datetime
from agentmem import (
    AgentMemClient,
    Config,
    MemoryType,
    SearchQuery,
    AgentMemError,
)
from agentmem.tools import ToolExecutor, ToolSchema, ToolParameter
from agentmem.observability import MetricsCollector, PerformanceTracker


class SmartAssistant:
    """
    A smart assistant that uses AgentMem for memory management,
    tools for actions, and observability for monitoring.
    """
    
    def __init__(self, agent_id: str, api_key: str, base_url: str):
        """Initialize the smart assistant."""
        self.agent_id = agent_id
        
        # Initialize AgentMem client
        config = Config(
            api_key=api_key,
            api_base_url=base_url,
            timeout=30,
            enable_caching=True,
        )
        self.client = AgentMemClient(config)
        
        # Initialize tool executor
        self.tools = ToolExecutor()
        self._register_tools()
        
        # Initialize observability
        self.metrics = MetricsCollector()
        self.tracker = PerformanceTracker()
    
    def _register_tools(self):
        """Register available tools."""
        # Calculator tool
        calculator_schema = ToolSchema(
            name="calculator",
            description="Perform arithmetic operations",
            parameters=[
                ToolParameter("operation", "string", "Operation: add, subtract, multiply, divide"),
                ToolParameter("a", "number", "First number"),
                ToolParameter("b", "number", "Second number"),
            ],
            returns="number"
        )
        self.tools.register_tool(calculator_schema, self._calculator)
        
        # Note-taking tool
        note_schema = ToolSchema(
            name="take_note",
            description="Take a note and store it in memory",
            parameters=[
                ToolParameter("content", "string", "Note content"),
                ToolParameter("importance", "number", "Importance (0-1)", required=False, default=0.5),
            ],
            returns="string"
        )
        self.tools.register_tool(note_schema, self._take_note)
    
    def _calculator(self, operation: str, a: float, b: float) -> float:
        """Calculator tool implementation."""
        operations = {
            "add": lambda x, y: x + y,
            "subtract": lambda x, y: x - y,
            "multiply": lambda x, y: x * y,
            "divide": lambda x, y: x / y if y != 0 else float('inf'),
        }
        return operations[operation](a, b)
    
    async def _take_note(self, content: str, importance: float = 0.5) -> str:
        """Note-taking tool implementation."""
        memory_id = await self.client.add_memory(
            content=content,
            agent_id=self.agent_id,
            memory_type=MemoryType.SEMANTIC,
            importance=importance,
            metadata={"source": "note_tool", "timestamp": datetime.now().isoformat()}
        )
        return f"Note saved with ID: {memory_id}"
    
    async def process_command(self, command: str) -> dict:
        """
        Process a user command.
        
        Args:
            command: User command
            
        Returns:
            Result dictionary
        """
        with self.tracker.track("process_command"):
            self.metrics.increment("commands_total")
            
            try:
                # Parse command (simplified)
                if command.startswith("calculate"):
                    return await self._handle_calculate(command)
                elif command.startswith("note"):
                    return await self._handle_note(command)
                elif command.startswith("remember"):
                    return await self._handle_remember(command)
                elif command.startswith("recall"):
                    return await self._handle_recall(command)
                else:
                    return {"status": "error", "message": "Unknown command"}
            
            except Exception as e:
                self.metrics.increment("errors_total", labels={"type": "command_error"})
                return {"status": "error", "message": str(e)}
    
    async def _handle_calculate(self, command: str) -> dict:
        """Handle calculate command."""
        # Parse: "calculate add 5 3"
        parts = command.split()
        if len(parts) != 4:
            return {"status": "error", "message": "Invalid calculate command"}
        
        _, operation, a, b = parts
        
        result = await self.tools.execute(
            "calculator",
            {"operation": operation, "a": float(a), "b": float(b)}
        )
        
        if result.status.value == "success":
            self.metrics.increment("tool_executions_total", labels={"tool": "calculator", "status": "success"})
            return {"status": "success", "result": result.output}
        else:
            self.metrics.increment("tool_executions_total", labels={"tool": "calculator", "status": "failed"})
            return {"status": "error", "message": result.error}
    
    async def _handle_note(self, command: str) -> dict:
        """Handle note command."""
        # Parse: "note This is my note"
        content = command[5:].strip()  # Remove "note "
        
        result = await self.tools.execute(
            "take_note",
            {"content": content, "importance": 0.7}
        )
        
        if result.status.value == "success":
            self.metrics.increment("tool_executions_total", labels={"tool": "take_note", "status": "success"})
            return {"status": "success", "message": result.output}
        else:
            self.metrics.increment("tool_executions_total", labels={"tool": "take_note", "status": "failed"})
            return {"status": "error", "message": result.error}
    
    async def _handle_remember(self, command: str) -> dict:
        """Handle remember command."""
        # Parse: "remember User prefers dark mode"
        content = command[9:].strip()  # Remove "remember "
        
        memory_id = await self.client.add_memory(
            content=content,
            agent_id=self.agent_id,
            memory_type=MemoryType.SEMANTIC,
            importance=0.8
        )
        
        self.metrics.increment("memories_added_total")
        return {"status": "success", "memory_id": memory_id}
    
    async def _handle_recall(self, command: str) -> dict:
        """Handle recall command."""
        # Parse: "recall dark mode"
        query = command[7:].strip()  # Remove "recall "
        
        results = await self.client.search_memories(
            SearchQuery(
                agent_id=self.agent_id,
                text_query=query,
                limit=5
            )
        )
        
        self.metrics.increment("searches_total")
        
        return {
            "status": "success",
            "results": [
                {"content": r.memory.content, "score": r.score}
                for r in results
            ]
        }
    
    async def get_stats(self) -> dict:
        """Get assistant statistics."""
        return {
            "metrics": self.metrics.get_metrics(),
            "performance": {
                "process_command": self.tracker.get_stats("process_command"),
            },
            "memory_stats": await self.client.get_memory_stats(self.agent_id),
        }
    
    async def close(self):
        """Close the assistant."""
        await self.client.close()


async def main():
    """Main example function."""
    print("🚀 AgentMem Python SDK - Complete Application Example\n")
    
    # Initialize assistant
    assistant = SmartAssistant(
        agent_id="smart_assistant_1",
        api_key=os.getenv("AGENTMEM_API_KEY", "demo-api-key"),
        base_url=os.getenv("AGENTMEM_BASE_URL", "http://localhost:8080")
    )
    
    try:
        # Process various commands
        commands = [
            "calculate add 15 27",
            "calculate multiply 8 9",
            "note Meeting with team at 3pm",
            "remember User prefers dark mode",
            "remember User's favorite color is blue",
            "recall user preferences",
            "calculate divide 100 5",
            "note Review project documentation",
        ]
        
        print("📝 Processing commands...\n")
        for i, command in enumerate(commands, 1):
            print(f"{i}. Command: '{command}'")
            result = await assistant.process_command(command)
            
            if result["status"] == "success":
                if "result" in result:
                    print(f"   ✅ Result: {result['result']}")
                elif "message" in result:
                    print(f"   ✅ {result['message']}")
                elif "memory_id" in result:
                    print(f"   ✅ Memory saved: {result['memory_id']}")
                elif "results" in result:
                    print(f"   ✅ Found {len(result['results'])} results:")
                    for r in result["results"][:3]:
                        print(f"      - {r['content'][:50]}... (score: {r['score']:.3f})")
            else:
                print(f"   ❌ Error: {result['message']}")
            
            print()
        
        # Get and display statistics
        print("📊 Assistant Statistics:\n")
        stats = await assistant.get_stats()
        
        print("   Metrics:")
        metrics = stats["metrics"]
        for name, value in metrics["counters"].items():
            print(f"      {name}: {value}")
        
        print("\n   Performance:")
        perf = stats["performance"]["process_command"]
        print(f"      Commands processed: {perf['count']}")
        print(f"      Average duration: {perf['avg']:.2f}ms")
        print(f"      P95 duration: {perf['p95']:.2f}ms")
        
        print("\n   Memory:")
        mem_stats = stats["memory_stats"]
        print(f"      Total memories: {mem_stats.total_memories}")
        print(f"      Average importance: {mem_stats.average_importance:.2f}")
        
        print("\n✨ Example completed successfully!")
    
    finally:
        await assistant.close()


if __name__ == "__main__":
    asyncio.run(main())

