"""
AgentMem Python SDK - Type Stubs

Type hints for the AgentMem Python SDK.
"""

from typing import Dict, List, Optional, Any, Union, Callable
from datetime import datetime
from enum import Enum

# Version
__version__: str
__author__: str
__email__: str

# ===== Client =====

class Config:
    """Configuration for AgentMem client."""
    api_key: str
    api_base_url: str
    timeout: int
    max_retries: int
    retry_delay: float
    enable_caching: bool
    cache_ttl: int
    max_connections: int
    max_keepalive_connections: int
    keepalive_expiry: float
    enable_compression: bool
    enable_logging: bool
    log_level: str
    
    def __init__(
        self,
        api_key: str,
        api_base_url: str = ...,
        timeout: int = ...,
        max_retries: int = ...,
        retry_delay: float = ...,
        enable_caching: bool = ...,
        cache_ttl: int = ...,
        max_connections: int = ...,
        max_keepalive_connections: int = ...,
        keepalive_expiry: float = ...,
        enable_compression: bool = ...,
        enable_logging: bool = ...,
        log_level: str = ...,
    ) -> None: ...
    
    @classmethod
    def from_env(cls) -> Config: ...
    
    def validate(self) -> None: ...

class AgentMemClient:
    """AgentMem Python client."""
    config: Config
    
    def __init__(self, config: Config) -> None: ...
    
    async def close(self) -> None: ...
    
    async def __aenter__(self) -> AgentMemClient: ...
    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None: ...
    
    async def add_memory(
        self,
        content: str,
        agent_id: str,
        memory_type: MemoryType = ...,
        user_id: Optional[str] = ...,
        session_id: Optional[str] = ...,
        importance: float = ...,
        metadata: Optional[Dict[str, Any]] = ...,
    ) -> str: ...
    
    async def get_memory(self, memory_id: str) -> Memory: ...
    
    async def update_memory(
        self,
        memory_id: str,
        content: Optional[str] = ...,
        importance: Optional[float] = ...,
        metadata: Optional[Dict[str, Any]] = ...,
    ) -> Memory: ...
    
    async def delete_memory(self, memory_id: str) -> bool: ...
    
    async def search_memories(self, query: SearchQuery) -> List[SearchResult]: ...
    
    async def batch_add_memories(self, memories: List[Dict[str, Any]]) -> List[str]: ...
    
    async def get_memory_stats(self, agent_id: str) -> MemoryStats: ...
    
    async def health_check(self) -> Dict[str, Any]: ...
    
    async def get_metrics(self) -> Dict[str, Any]: ...

# ===== Types =====

class MemoryType(Enum):
    """Memory type enumeration."""
    EPISODIC: str
    SEMANTIC: str
    PROCEDURAL: str
    UNTYPED: str

class ImportanceLevel(Enum):
    """Memory importance level."""
    LOW: int
    MEDIUM: int
    HIGH: int
    CRITICAL: int

class MatchType(Enum):
    """Search match type."""
    EXACT_TEXT: str
    PARTIAL_TEXT: str
    SEMANTIC: str
    METADATA: str

class Memory:
    """Memory data structure."""
    id: str
    content: str
    memory_type: MemoryType
    agent_id: str
    user_id: Optional[str]
    session_id: Optional[str]
    importance: float
    metadata: Optional[Dict[str, Any]]
    created_at: Optional[datetime]
    updated_at: Optional[datetime]
    access_count: int
    last_accessed: Optional[datetime]
    embedding: Optional[List[float]]
    
    def to_dict(self) -> Dict[str, Any]: ...
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> Memory: ...

class SearchQuery:
    """Search query parameters."""
    agent_id: str
    text_query: Optional[str]
    vector_query: Optional[List[float]]
    memory_type: Optional[MemoryType]
    user_id: Optional[str]
    min_importance: Optional[float]
    max_age_seconds: Optional[int]
    limit: int
    metadata_filters: Optional[Dict[str, Any]]
    
    def to_dict(self) -> Dict[str, Any]: ...

class SearchResult:
    """Search result with score and match type."""
    memory: Memory
    score: float
    match_type: MatchType
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> SearchResult: ...

class MemoryStats:
    """Memory statistics."""
    total_memories: int
    memories_by_type: Dict[str, int]
    memories_by_agent: Dict[str, int]
    average_importance: float
    oldest_memory_age_days: float
    most_accessed_memory_id: Optional[str]
    total_access_count: int
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> MemoryStats: ...

# ===== Tools =====

class ToolStatus(Enum):
    """Tool execution status."""
    PENDING: str
    RUNNING: str
    SUCCESS: str
    FAILED: str
    CANCELLED: str

class ToolParameter:
    """Tool parameter definition."""
    name: str
    type: str
    description: str
    required: bool
    default: Optional[Any]
    
    def to_dict(self) -> Dict[str, Any]: ...

class ToolSchema:
    """Tool schema definition."""
    name: str
    description: str
    parameters: List[ToolParameter]
    returns: str
    
    def to_dict(self) -> Dict[str, Any]: ...
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> ToolSchema: ...

class ToolExecution:
    """Tool execution result."""
    id: str
    tool_name: str
    status: ToolStatus
    input: Dict[str, Any]
    output: Optional[Any]
    error: Optional[str]
    duration_ms: Optional[float]
    
    def to_dict(self) -> Dict[str, Any]: ...
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> ToolExecution: ...

class ToolExecutor:
    """Tool executor for AgentMem."""
    
    def __init__(self) -> None: ...
    
    def register_tool(self, schema: ToolSchema, handler: Callable) -> None: ...
    
    def unregister_tool(self, name: str) -> bool: ...
    
    def list_tools(self) -> List[ToolSchema]: ...
    
    def get_tool(self, name: str) -> Optional[ToolSchema]: ...
    
    async def execute(
        self,
        tool_name: str,
        input_data: Dict[str, Any],
        timeout: Optional[float] = ...,
    ) -> ToolExecution: ...

# ===== Observability =====

class HealthStatus(Enum):
    """Health status enumeration."""
    HEALTHY: str
    DEGRADED: str
    UNHEALTHY: str

class ComponentHealth:
    """Component health status."""
    name: str
    status: HealthStatus
    message: Optional[str]
    last_check: Optional[datetime]
    
    def to_dict(self) -> Dict[str, Any]: ...
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> ComponentHealth: ...

class HealthCheckResult:
    """Health check result."""
    status: HealthStatus
    components: List[ComponentHealth]
    version: str
    uptime_seconds: float
    timestamp: datetime
    
    def to_dict(self) -> Dict[str, Any]: ...
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> HealthCheckResult: ...

class MetricsCollector:
    """Metrics collector for AgentMem."""
    
    def __init__(self) -> None: ...
    
    def increment(self, name: str, value: float = ..., labels: Optional[Dict[str, str]] = ...) -> None: ...
    
    def set_gauge(self, name: str, value: float, labels: Optional[Dict[str, str]] = ...) -> None: ...
    
    def record_histogram(self, name: str, value: float, labels: Optional[Dict[str, str]] = ...) -> None: ...
    
    def get_counter(self, name: str, labels: Optional[Dict[str, str]] = ...) -> float: ...
    
    def get_gauge(self, name: str, labels: Optional[Dict[str, str]] = ...) -> float: ...
    
    def get_histogram_stats(self, name: str, labels: Optional[Dict[str, str]] = ...) -> Dict[str, float]: ...
    
    def get_metrics(self) -> Dict[str, Any]: ...
    
    def reset(self) -> None: ...

class PerformanceTracker:
    """Performance tracker for operations."""
    
    def __init__(self) -> None: ...
    
    def track(self, operation_name: str): ...
    
    def record(self, operation_name: str, duration_ms: float) -> None: ...
    
    def get_stats(self, operation_name: str) -> Dict[str, float]: ...

# ===== Errors =====

class AgentMemError(Exception):
    """Base exception for AgentMem SDK."""
    ...

class AuthenticationError(AgentMemError):
    """Authentication failed."""
    ...

class ValidationError(AgentMemError):
    """Request validation failed."""
    ...

class NetworkError(AgentMemError):
    """Network communication error."""
    ...

class NotFoundError(AgentMemError):
    """Resource not found."""
    ...

class RateLimitError(AgentMemError):
    """Rate limit exceeded."""
    ...

class ServerError(AgentMemError):
    """Server internal error."""
    ...

