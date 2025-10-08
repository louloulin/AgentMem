"""
Simple Memory API - Mem0-compatible interface for AgentMem
"""

import asyncio
import json
import os
import subprocess
import tempfile
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from .types import MemoryRecord, SearchResult


class Memory:
    """
    Simple Memory API for AgentMem.
    
    This class provides a Mem0-compatible interface for AgentMem,
    making it easy to add, search, update, and delete memories.
    
    Example:
        ```python
        import asyncio
        from agentmem import Memory
        
        async def main():
            # Initialize memory (embedded mode by default)
            memory = Memory()
            
            # Add a memory
            result = await memory.add(
                "User prefers Python over JavaScript",
                agent_id="assistant-1",
                user_id="user-123"
            )
            print(f"Added memory: {result['id']}")
            
            # Search memories
            results = await memory.search(
                query="What programming language does the user prefer?",
                agent_id="assistant-1",
                user_id="user-123"
            )
            
            for result in results:
                print(f"- {result['content']} (score: {result['score']:.2f})")
        
        asyncio.run(main())
        ```
    """
    
    def __init__(
        self,
        storage_path: Optional[str] = None,
        embedding_model: Optional[str] = None,
        llm_model: Optional[str] = None,
    ):
        """
        Initialize Memory instance.
        
        Args:
            storage_path: Path to storage directory (default: ./agentmem_data)
            embedding_model: Embedding model to use (default: all-MiniLM-L6-v2)
            llm_model: LLM model to use (default: gpt-3.5-turbo)
        """
        self.storage_path = storage_path or "./agentmem_data"
        self.embedding_model = embedding_model or "all-MiniLM-L6-v2"
        self.llm_model = llm_model or "gpt-3.5-turbo"
        
        # Create storage directory if it doesn't exist
        Path(self.storage_path).mkdir(parents=True, exist_ok=True)
        
        # In-memory storage for now (will be replaced with actual backend)
        self._memories: Dict[str, MemoryRecord] = {}
    
    async def add(
        self,
        content: str,
        agent_id: Optional[str] = None,
        user_id: Optional[str] = None,
        session_id: Optional[str] = None,
        memory_type: Optional[str] = None,
        importance: float = 0.5,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """
        Add a new memory.
        
        Args:
            content: The memory content
            agent_id: Agent identifier
            user_id: User identifier
            session_id: Session identifier
            memory_type: Type of memory (episodic, semantic, procedural)
            importance: Importance score (0.0 to 1.0)
            metadata: Additional metadata
        
        Returns:
            Dictionary with memory ID and status
        """
        memory_id = str(uuid.uuid4())
        now = datetime.now(timezone.utc)

        memory = MemoryRecord(
            id=memory_id,
            content=content,
            agent_id=agent_id,
            user_id=user_id,
            session_id=session_id,
            memory_type=memory_type,
            importance=importance,
            metadata=metadata or {},
            created_at=now,
            updated_at=now,
        )
        
        self._memories[memory_id] = memory
        
        return {
            "id": memory_id,
            "status": "success",
            "message": "Memory added successfully",
        }
    
    async def search(
        self,
        query: str,
        agent_id: Optional[str] = None,
        user_id: Optional[str] = None,
        session_id: Optional[str] = None,
        memory_type: Optional[str] = None,
        limit: int = 10,
        threshold: float = 0.0,
    ) -> List[Dict[str, Any]]:
        """
        Search for memories.
        
        Args:
            query: Search query
            agent_id: Filter by agent ID
            user_id: Filter by user ID
            session_id: Filter by session ID
            memory_type: Filter by memory type
            limit: Maximum number of results
            threshold: Minimum similarity threshold
        
        Returns:
            List of search results with scores
        """
        results = []
        
        for memory in self._memories.values():
            # Apply filters
            if agent_id and memory.agent_id != agent_id:
                continue
            if user_id and memory.user_id != user_id:
                continue
            if session_id and memory.session_id != session_id:
                continue
            if memory_type and memory.memory_type != memory_type:
                continue
            
            # Simple text matching for now (will be replaced with vector search)
            score = self._calculate_similarity(query, memory.content)
            
            if score >= threshold:
                result = memory.to_dict()
                result["score"] = score
                results.append(result)
        
        # Sort by score descending
        results.sort(key=lambda x: x["score"], reverse=True)
        
        return results[:limit]
    
    async def get(self, memory_id: str) -> Optional[Dict[str, Any]]:
        """
        Get a memory by ID.
        
        Args:
            memory_id: Memory identifier
        
        Returns:
            Memory record or None if not found
        """
        memory = self._memories.get(memory_id)
        return memory.to_dict() if memory else None
    
    async def get_all(
        self,
        agent_id: Optional[str] = None,
        user_id: Optional[str] = None,
        session_id: Optional[str] = None,
        memory_type: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> List[Dict[str, Any]]:
        """
        Get all memories matching filters.
        
        Args:
            agent_id: Filter by agent ID
            user_id: Filter by user ID
            session_id: Filter by session ID
            memory_type: Filter by memory type
            limit: Maximum number of results
        
        Returns:
            List of memory records
        """
        results = []
        
        for memory in self._memories.values():
            # Apply filters
            if agent_id and memory.agent_id != agent_id:
                continue
            if user_id and memory.user_id != user_id:
                continue
            if session_id and memory.session_id != session_id:
                continue
            if memory_type and memory.memory_type != memory_type:
                continue
            
            results.append(memory.to_dict())
        
        # Sort by created_at descending
        results.sort(key=lambda x: x["created_at"] or "", reverse=True)
        
        if limit:
            results = results[:limit]
        
        return results
    
    async def update(
        self,
        memory_id: str,
        content: Optional[str] = None,
        importance: Optional[float] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """
        Update a memory.
        
        Args:
            memory_id: Memory identifier
            content: New content (optional)
            importance: New importance score (optional)
            metadata: New metadata (optional)
        
        Returns:
            Updated memory record
        """
        memory = self._memories.get(memory_id)
        
        if not memory:
            raise ValueError(f"Memory not found: {memory_id}")
        
        if content is not None:
            memory.content = content
        if importance is not None:
            memory.importance = importance
        if metadata is not None:
            memory.metadata = metadata

        memory.updated_at = datetime.now(timezone.utc)

        return memory.to_dict()
    
    async def delete(self, memory_id: str) -> Dict[str, Any]:
        """
        Delete a memory.
        
        Args:
            memory_id: Memory identifier
        
        Returns:
            Status dictionary
        """
        if memory_id in self._memories:
            del self._memories[memory_id]
            return {
                "status": "success",
                "message": f"Memory {memory_id} deleted successfully",
            }
        else:
            raise ValueError(f"Memory not found: {memory_id}")
    
    async def clear(
        self,
        agent_id: Optional[str] = None,
        user_id: Optional[str] = None,
        session_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Clear all memories matching filters.
        
        Args:
            agent_id: Filter by agent ID
            user_id: Filter by user ID
            session_id: Filter by session ID
        
        Returns:
            Status dictionary with count of deleted memories
        """
        to_delete = []
        
        for memory_id, memory in self._memories.items():
            # Apply filters
            if agent_id and memory.agent_id != agent_id:
                continue
            if user_id and memory.user_id != user_id:
                continue
            if session_id and memory.session_id != session_id:
                continue
            
            to_delete.append(memory_id)
        
        for memory_id in to_delete:
            del self._memories[memory_id]
        
        return {
            "status": "success",
            "message": f"Deleted {len(to_delete)} memories",
            "count": len(to_delete),
        }
    
    def _calculate_similarity(self, query: str, content: str) -> float:
        """
        Calculate simple text similarity.
        
        This is a placeholder implementation. In production, this would use
        vector embeddings and cosine similarity.
        """
        query_lower = query.lower()
        content_lower = content.lower()
        
        # Simple word overlap scoring
        query_words = set(query_lower.split())
        content_words = set(content_lower.split())
        
        if not query_words or not content_words:
            return 0.0
        
        overlap = len(query_words & content_words)
        union = len(query_words | content_words)
        
        return overlap / union if union > 0 else 0.0

