"""
AgentMem Python SDK - Type Definitions

Core data types and enums for the AgentMem Python SDK.
"""

from enum import Enum
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass
from datetime import datetime


class MemoryType(Enum):
    """Memory type enumeration."""
    EPISODIC = "episodic"
    SEMANTIC = "semantic"
    PROCEDURAL = "procedural"
    UNTYPED = "untyped"


class ImportanceLevel(Enum):
    """Memory importance level."""
    LOW = 1
    MEDIUM = 2
    HIGH = 3
    CRITICAL = 4


class MatchType(Enum):
    """Search match type."""
    EXACT_TEXT = "exact_text"
    PARTIAL_TEXT = "partial_text"
    SEMANTIC = "semantic"
    METADATA = "metadata"


@dataclass
class Memory:
    """Memory data structure."""
    id: str
    content: str
    memory_type: MemoryType
    agent_id: str
    user_id: Optional[str] = None
    session_id: Optional[str] = None
    importance: float = 0.5
    metadata: Optional[Dict[str, Any]] = None
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None
    access_count: int = 0
    last_accessed: Optional[datetime] = None
    embedding: Optional[List[float]] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert memory to dictionary."""
        return {
            "id": self.id,
            "content": self.content,
            "memory_type": self.memory_type.value,
            "agent_id": self.agent_id,
            "user_id": self.user_id,
            "session_id": self.session_id,
            "importance": self.importance,
            "metadata": self.metadata or {},
            "created_at": self.created_at.isoformat() if self.created_at else None,
            "updated_at": self.updated_at.isoformat() if self.updated_at else None,
            "access_count": self.access_count,
            "last_accessed": self.last_accessed.isoformat() if self.last_accessed else None,
            "embedding": self.embedding,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Memory":
        """Create memory from dictionary."""
        return cls(
            id=data["id"],
            content=data["content"],
            memory_type=MemoryType(data["memory_type"]),
            agent_id=data["agent_id"],
            user_id=data.get("user_id"),
            session_id=data.get("session_id"),
            importance=data.get("importance", 0.5),
            metadata=data.get("metadata"),
            created_at=datetime.fromisoformat(data["created_at"]) if data.get("created_at") else None,
            updated_at=datetime.fromisoformat(data["updated_at"]) if data.get("updated_at") else None,
            access_count=data.get("access_count", 0),
            last_accessed=datetime.fromisoformat(data["last_accessed"]) if data.get("last_accessed") else None,
            embedding=data.get("embedding"),
        )


@dataclass
class SearchQuery:
    """Search query parameters."""
    agent_id: str
    text_query: Optional[str] = None
    vector_query: Optional[List[float]] = None
    memory_type: Optional[MemoryType] = None
    user_id: Optional[str] = None
    min_importance: Optional[float] = None
    max_age_seconds: Optional[int] = None
    limit: int = 10
    metadata_filters: Optional[Dict[str, Any]] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert query to dictionary."""
        query = {
            "agent_id": self.agent_id,
            "limit": self.limit,
        }
        
        if self.text_query:
            query["text_query"] = self.text_query
        if self.vector_query:
            query["vector_query"] = self.vector_query
        if self.memory_type:
            query["memory_type"] = self.memory_type.value
        if self.user_id:
            query["user_id"] = self.user_id
        if self.min_importance is not None:
            query["min_importance"] = self.min_importance
        if self.max_age_seconds is not None:
            query["max_age_seconds"] = self.max_age_seconds
        if self.metadata_filters:
            query["metadata_filters"] = self.metadata_filters
            
        return query


@dataclass
class SearchResult:
    """Search result with score and match type."""
    memory: Memory
    score: float
    match_type: MatchType
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "SearchResult":
        """Create search result from dictionary."""
        return cls(
            memory=Memory.from_dict(data["memory"]),
            score=data["score"],
            match_type=MatchType(data["match_type"]),
        )


@dataclass
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
    def from_dict(cls, data: Dict[str, Any]) -> "MemoryStats":
        """Create stats from dictionary."""
        return cls(
            total_memories=data["total_memories"],
            memories_by_type=data["memories_by_type"],
            memories_by_agent=data["memories_by_agent"],
            average_importance=data["average_importance"],
            oldest_memory_age_days=data["oldest_memory_age_days"],
            most_accessed_memory_id=data.get("most_accessed_memory_id"),
            total_access_count=data["total_access_count"],
        )


# Exception classes
class AgentMemError(Exception):
    """Base exception for AgentMem SDK."""
    pass


class AuthenticationError(AgentMemError):
    """Authentication failed."""
    pass


class ValidationError(AgentMemError):
    """Request validation failed."""
    pass


class NetworkError(AgentMemError):
    """Network communication error."""
    pass


class NotFoundError(AgentMemError):
    """Resource not found."""
    pass


class RateLimitError(AgentMemError):
    """Rate limit exceeded."""
    pass


class ServerError(AgentMemError):
    """Server internal error."""
    pass
