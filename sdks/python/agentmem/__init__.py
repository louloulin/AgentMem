"""
AgentMem Python SDK

Official Python client library for AgentMem - Enterprise-grade memory management for AI agents.
"""

from .client import AgentMemClient
from .types import (
    Memory,
    MemoryType,
    SearchQuery,
    SearchResult,
    MemoryStats,
    AgentMemError,
    AuthenticationError,
    ValidationError,
    NetworkError,
)
from .config import Config
from .tools import (
    ToolExecutor,
    ToolSchema,
    ToolParameter,
    ToolExecution,
    ToolStatus,
)
from .observability import (
    MetricsCollector,
    PerformanceTracker,
    HealthStatus,
    HealthCheckResult,
    ComponentHealth,
)

__version__ = "7.0.0"
__author__ = "AgentMem Team"
__email__ = "support@agentmem.dev"

__all__ = [
    # Client
    "AgentMemClient",
    "Config",
    # Memory types
    "Memory",
    "MemoryType",
    "SearchQuery",
    "SearchResult",
    "MemoryStats",
    # Tools
    "ToolExecutor",
    "ToolSchema",
    "ToolParameter",
    "ToolExecution",
    "ToolStatus",
    # Observability
    "MetricsCollector",
    "PerformanceTracker",
    "HealthStatus",
    "HealthCheckResult",
    "ComponentHealth",
    # Errors
    "AgentMemError",
    "AuthenticationError",
    "ValidationError",
    "NetworkError",
]
