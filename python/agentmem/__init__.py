"""
AgentMem - Simple Memory API

A simplified, Mem0-compatible API for AgentMem.
"""

from .memory import Memory
from .types import MemoryRecord, SearchResult

__version__ = "2.0.0"
__all__ = ["Memory", "MemoryRecord", "SearchResult"]

