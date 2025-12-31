"""
LlamaIndex-AgentMem Integration

Official integration adapter between LlamaIndex and AgentMem for enterprise-grade
memory management in AI applications.
"""

from .memory import AgentMemory
from .reader import AgentMemReader
from .node_parser import AgentMemNodeParser

__version__ = "1.0.0"
__author__ = "AgentMem Team"
__email__ = "support@agentmem.dev"

__all__ = [
    "AgentMemory",
    "AgentMemReader",
    "AgentMemNodeParser",
]
