"""
AgentMem Python SDK - Tool Execution

Tool execution and management for AgentMem.
"""

from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass
from enum import Enum
import json


class ToolStatus(Enum):
    """Tool execution status."""
    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class ToolParameter:
    """Tool parameter definition."""
    name: str
    type: str
    description: str
    required: bool = True
    default: Optional[Any] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "name": self.name,
            "type": self.type,
            "description": self.description,
            "required": self.required,
            "default": self.default,
        }


@dataclass
class ToolSchema:
    """Tool schema definition."""
    name: str
    description: str
    parameters: List[ToolParameter]
    returns: str
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "name": self.name,
            "description": self.description,
            "parameters": [p.to_dict() for p in self.parameters],
            "returns": self.returns,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ToolSchema":
        """Create from dictionary."""
        return cls(
            name=data["name"],
            description=data["description"],
            parameters=[
                ToolParameter(
                    name=p["name"],
                    type=p["type"],
                    description=p["description"],
                    required=p.get("required", True),
                    default=p.get("default"),
                )
                for p in data["parameters"]
            ],
            returns=data["returns"],
        )


@dataclass
class ToolExecution:
    """Tool execution result."""
    id: str
    tool_name: str
    status: ToolStatus
    input: Dict[str, Any]
    output: Optional[Any] = None
    error: Optional[str] = None
    duration_ms: Optional[float] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "tool_name": self.tool_name,
            "status": self.status.value,
            "input": self.input,
            "output": self.output,
            "error": self.error,
            "duration_ms": self.duration_ms,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ToolExecution":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            tool_name=data["tool_name"],
            status=ToolStatus(data["status"]),
            input=data["input"],
            output=data.get("output"),
            error=data.get("error"),
            duration_ms=data.get("duration_ms"),
        )


class ToolExecutor:
    """
    Tool executor for AgentMem.
    
    Example:
        ```python
        from agentmem.tools import ToolExecutor, ToolSchema, ToolParameter
        
        # Define a tool
        calculator_schema = ToolSchema(
            name="calculator",
            description="Perform basic arithmetic operations",
            parameters=[
                ToolParameter(name="operation", type="string", description="Operation: add, subtract, multiply, divide"),
                ToolParameter(name="a", type="number", description="First number"),
                ToolParameter(name="b", type="number", description="Second number"),
            ],
            returns="number"
        )
        
        # Register tool
        executor = ToolExecutor()
        executor.register_tool(calculator_schema, calculator_function)
        
        # Execute tool
        result = await executor.execute("calculator", {"operation": "add", "a": 5, "b": 3})
        print(result.output)  # 8
        ```
    """
    
    def __init__(self):
        """Initialize tool executor."""
        self._tools: Dict[str, ToolSchema] = {}
        self._handlers: Dict[str, Callable] = {}
    
    def register_tool(self, schema: ToolSchema, handler: Callable) -> None:
        """
        Register a tool.
        
        Args:
            schema: Tool schema
            handler: Tool handler function
        """
        self._tools[schema.name] = schema
        self._handlers[schema.name] = handler
    
    def unregister_tool(self, name: str) -> bool:
        """
        Unregister a tool.
        
        Args:
            name: Tool name
            
        Returns:
            True if successful
        """
        if name in self._tools:
            del self._tools[name]
            del self._handlers[name]
            return True
        return False
    
    def list_tools(self) -> List[ToolSchema]:
        """
        List all registered tools.
        
        Returns:
            List of tool schemas
        """
        return list(self._tools.values())
    
    def get_tool(self, name: str) -> Optional[ToolSchema]:
        """
        Get tool schema by name.
        
        Args:
            name: Tool name
            
        Returns:
            Tool schema or None
        """
        return self._tools.get(name)
    
    async def execute(
        self,
        tool_name: str,
        input_data: Dict[str, Any],
        timeout: Optional[float] = None,
    ) -> ToolExecution:
        """
        Execute a tool.
        
        Args:
            tool_name: Tool name
            input_data: Input parameters
            timeout: Execution timeout in seconds
            
        Returns:
            Tool execution result
        """
        import time
        import asyncio
        import uuid
        
        execution_id = str(uuid.uuid4())
        start_time = time.time()
        
        # Check if tool exists
        if tool_name not in self._tools:
            return ToolExecution(
                id=execution_id,
                tool_name=tool_name,
                status=ToolStatus.FAILED,
                input=input_data,
                error=f"Tool '{tool_name}' not found",
                duration_ms=(time.time() - start_time) * 1000,
            )
        
        # Validate input
        schema = self._tools[tool_name]
        for param in schema.parameters:
            if param.required and param.name not in input_data:
                return ToolExecution(
                    id=execution_id,
                    tool_name=tool_name,
                    status=ToolStatus.FAILED,
                    input=input_data,
                    error=f"Missing required parameter: {param.name}",
                    duration_ms=(time.time() - start_time) * 1000,
                )
        
        # Execute tool
        try:
            handler = self._handlers[tool_name]
            
            # Handle both sync and async handlers
            if asyncio.iscoroutinefunction(handler):
                if timeout:
                    output = await asyncio.wait_for(handler(**input_data), timeout=timeout)
                else:
                    output = await handler(**input_data)
            else:
                output = handler(**input_data)
            
            duration_ms = (time.time() - start_time) * 1000
            
            return ToolExecution(
                id=execution_id,
                tool_name=tool_name,
                status=ToolStatus.SUCCESS,
                input=input_data,
                output=output,
                duration_ms=duration_ms,
            )
        
        except asyncio.TimeoutError:
            return ToolExecution(
                id=execution_id,
                tool_name=tool_name,
                status=ToolStatus.FAILED,
                input=input_data,
                error=f"Tool execution timed out after {timeout}s",
                duration_ms=(time.time() - start_time) * 1000,
            )
        
        except Exception as e:
            return ToolExecution(
                id=execution_id,
                tool_name=tool_name,
                status=ToolStatus.FAILED,
                input=input_data,
                error=str(e),
                duration_ms=(time.time() - start_time) * 1000,
            )

