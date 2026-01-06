"""
Tests for AgentMem Tools Module
"""

import pytest
import asyncio
from agentmem.tools import (
    ToolExecutor,
    ToolSchema,
    ToolParameter,
    ToolExecution,
    ToolStatus,
)


# Test fixtures
@pytest.fixture
def tool_executor():
    """Create a tool executor instance."""
    return ToolExecutor()


@pytest.fixture
def calculator_schema():
    """Create a calculator tool schema."""
    return ToolSchema(
        name="calculator",
        description="Perform arithmetic operations",
        parameters=[
            ToolParameter("operation", "string", "Operation type"),
            ToolParameter("a", "number", "First number"),
            ToolParameter("b", "number", "Second number"),
        ],
        returns="number"
    )


def calculator_handler(operation: str, a: float, b: float) -> float:
    """Calculator tool handler."""
    operations = {
        "add": lambda x, y: x + y,
        "subtract": lambda x, y: x - y,
        "multiply": lambda x, y: x * y,
        "divide": lambda x, y: x / y if y != 0 else float('inf'),
    }
    return operations[operation](a, b)


async def async_handler(value: str) -> str:
    """Async tool handler."""
    await asyncio.sleep(0.01)
    return value.upper()


# Tests
class TestToolParameter:
    """Tests for ToolParameter."""
    
    def test_create_parameter(self):
        """Test creating a tool parameter."""
        param = ToolParameter(
            name="test_param",
            type="string",
            description="Test parameter",
            required=True,
            default=None
        )
        
        assert param.name == "test_param"
        assert param.type == "string"
        assert param.description == "Test parameter"
        assert param.required is True
        assert param.default is None
    
    def test_parameter_to_dict(self):
        """Test converting parameter to dictionary."""
        param = ToolParameter("name", "string", "desc", required=False, default="default")
        data = param.to_dict()
        
        assert data["name"] == "name"
        assert data["type"] == "string"
        assert data["description"] == "desc"
        assert data["required"] is False
        assert data["default"] == "default"


class TestToolSchema:
    """Tests for ToolSchema."""
    
    def test_create_schema(self, calculator_schema):
        """Test creating a tool schema."""
        assert calculator_schema.name == "calculator"
        assert calculator_schema.description == "Perform arithmetic operations"
        assert len(calculator_schema.parameters) == 3
        assert calculator_schema.returns == "number"
    
    def test_schema_to_dict(self, calculator_schema):
        """Test converting schema to dictionary."""
        data = calculator_schema.to_dict()
        
        assert data["name"] == "calculator"
        assert data["description"] == "Perform arithmetic operations"
        assert len(data["parameters"]) == 3
        assert data["returns"] == "number"
    
    def test_schema_from_dict(self):
        """Test creating schema from dictionary."""
        data = {
            "name": "test_tool",
            "description": "Test tool",
            "parameters": [
                {
                    "name": "param1",
                    "type": "string",
                    "description": "First param",
                    "required": True,
                    "default": None
                }
            ],
            "returns": "string"
        }
        
        schema = ToolSchema.from_dict(data)
        assert schema.name == "test_tool"
        assert len(schema.parameters) == 1
        assert schema.parameters[0].name == "param1"


class TestToolExecution:
    """Tests for ToolExecution."""
    
    def test_create_execution(self):
        """Test creating a tool execution."""
        execution = ToolExecution(
            id="exec_123",
            tool_name="calculator",
            status=ToolStatus.SUCCESS,
            input={"a": 1, "b": 2},
            output=3,
            duration_ms=10.5
        )
        
        assert execution.id == "exec_123"
        assert execution.tool_name == "calculator"
        assert execution.status == ToolStatus.SUCCESS
        assert execution.output == 3
        assert execution.duration_ms == 10.5
    
    def test_execution_to_dict(self):
        """Test converting execution to dictionary."""
        execution = ToolExecution(
            id="exec_123",
            tool_name="test",
            status=ToolStatus.FAILED,
            input={"x": 1},
            error="Test error"
        )
        
        data = execution.to_dict()
        assert data["id"] == "exec_123"
        assert data["status"] == "failed"
        assert data["error"] == "Test error"
    
    def test_execution_from_dict(self):
        """Test creating execution from dictionary."""
        data = {
            "id": "exec_456",
            "tool_name": "test",
            "status": "success",
            "input": {"x": 1},
            "output": 2,
            "duration_ms": 5.0
        }
        
        execution = ToolExecution.from_dict(data)
        assert execution.id == "exec_456"
        assert execution.status == ToolStatus.SUCCESS
        assert execution.output == 2


class TestToolExecutor:
    """Tests for ToolExecutor."""
    
    def test_create_executor(self, tool_executor):
        """Test creating a tool executor."""
        assert tool_executor is not None
        assert len(tool_executor.list_tools()) == 0
    
    def test_register_tool(self, tool_executor, calculator_schema):
        """Test registering a tool."""
        tool_executor.register_tool(calculator_schema, calculator_handler)
        
        tools = tool_executor.list_tools()
        assert len(tools) == 1
        assert tools[0].name == "calculator"
    
    def test_unregister_tool(self, tool_executor, calculator_schema):
        """Test unregistering a tool."""
        tool_executor.register_tool(calculator_schema, calculator_handler)
        assert len(tool_executor.list_tools()) == 1
        
        success = tool_executor.unregister_tool("calculator")
        assert success is True
        assert len(tool_executor.list_tools()) == 0
    
    def test_unregister_nonexistent_tool(self, tool_executor):
        """Test unregistering a non-existent tool."""
        success = tool_executor.unregister_tool("nonexistent")
        assert success is False
    
    def test_get_tool(self, tool_executor, calculator_schema):
        """Test getting a tool schema."""
        tool_executor.register_tool(calculator_schema, calculator_handler)
        
        schema = tool_executor.get_tool("calculator")
        assert schema is not None
        assert schema.name == "calculator"
        
        schema = tool_executor.get_tool("nonexistent")
        assert schema is None
    
    @pytest.mark.asyncio
    async def test_execute_sync_tool(self, tool_executor, calculator_schema):
        """Test executing a synchronous tool."""
        tool_executor.register_tool(calculator_schema, calculator_handler)
        
        result = await tool_executor.execute(
            "calculator",
            {"operation": "add", "a": 5, "b": 3}
        )
        
        assert result.status == ToolStatus.SUCCESS
        assert result.output == 8
        assert result.error is None
        assert result.duration_ms is not None
    
    @pytest.mark.asyncio
    async def test_execute_async_tool(self, tool_executor):
        """Test executing an asynchronous tool."""
        schema = ToolSchema(
            name="async_tool",
            description="Async tool",
            parameters=[ToolParameter("value", "string", "Input value")],
            returns="string"
        )
        tool_executor.register_tool(schema, async_handler)
        
        result = await tool_executor.execute("async_tool", {"value": "hello"})
        
        assert result.status == ToolStatus.SUCCESS
        assert result.output == "HELLO"
    
    @pytest.mark.asyncio
    async def test_execute_nonexistent_tool(self, tool_executor):
        """Test executing a non-existent tool."""
        result = await tool_executor.execute("nonexistent", {})
        
        assert result.status == ToolStatus.FAILED
        assert "not found" in result.error.lower()
    
    @pytest.mark.asyncio
    async def test_execute_missing_parameter(self, tool_executor, calculator_schema):
        """Test executing with missing required parameter."""
        tool_executor.register_tool(calculator_schema, calculator_handler)
        
        result = await tool_executor.execute(
            "calculator",
            {"operation": "add", "a": 5}  # Missing 'b'
        )
        
        assert result.status == ToolStatus.FAILED
        assert "missing" in result.error.lower()
    
    @pytest.mark.asyncio
    async def test_execute_with_error(self, tool_executor, calculator_schema):
        """Test executing a tool that raises an error."""
        tool_executor.register_tool(calculator_schema, calculator_handler)
        
        result = await tool_executor.execute(
            "calculator",
            {"operation": "invalid", "a": 5, "b": 3}
        )
        
        assert result.status == ToolStatus.FAILED
        assert result.error is not None
    
    @pytest.mark.asyncio
    async def test_execute_with_timeout(self, tool_executor):
        """Test executing a tool with timeout."""
        async def slow_handler(value: str) -> str:
            await asyncio.sleep(1.0)  # Slow operation
            return value

        schema = ToolSchema(
            name="slow_tool",
            description="Slow tool",
            parameters=[ToolParameter("value", "string", "Input")],
            returns="string"
        )
        tool_executor.register_tool(schema, slow_handler)

        result = await tool_executor.execute("slow_tool", {"value": "test"}, timeout=0.1)

        assert result.status == ToolStatus.FAILED
        assert ("timeout" in result.error.lower() or "timed out" in result.error.lower())


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

