"""
Type definitions for AgentMem simple API
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Any, Dict, Optional


@dataclass
class MemoryRecord:
    """A memory record."""
    
    id: str
    content: str
    agent_id: Optional[str] = None
    user_id: Optional[str] = None
    session_id: Optional[str] = None
    memory_type: Optional[str] = None
    importance: float = 0.5
    metadata: Optional[Dict[str, Any]] = None
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "content": self.content,
            "agent_id": self.agent_id,
            "user_id": self.user_id,
            "session_id": self.session_id,
            "memory_type": self.memory_type,
            "importance": self.importance,
            "metadata": self.metadata or {},
            "created_at": self.created_at.isoformat() if self.created_at else None,
            "updated_at": self.updated_at.isoformat() if self.updated_at else None,
        }


@dataclass
class SearchResult:
    """A search result with similarity score."""
    
    memory: MemoryRecord
    score: float
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        result = self.memory.to_dict()
        result["score"] = self.score
        return result

