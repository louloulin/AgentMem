"""
AgentMem Memory Integration for LlamaIndex

Core memory class that integrates AgentMem with LlamaIndex's BaseMemory interface.
"""

import logging
from typing import List, Dict, Any, Optional

from llama_index.core import Document
from llama_index.core.memory import BaseMemory
from llama_index.core.schema import BaseNode

try:
    from agentmem import AgentMemClient, MemoryType, SearchQuery, Config
except ImportError:
    raise ImportError(
        "agentmem package is required. Install it with: pip install agentmem"
    )

logger = logging.getLogger(__name__)


class AgentMemory(BaseMemory):
    """
    AgentMem integration for LlamaIndex memory management.

    This class provides seamless integration between LlamaIndex and AgentMem,
    allowing you to use AgentMem's enterprise-grade memory capabilities as
    the backend for LlamaIndex's memory operations.

    Features:
    - Persistent memory storage with semantic search
    - Multi-agent memory management
    - Importance-based memory ranking
    - Metadata filtering and tagging
    - Automatic memory deduplication

    Example:
        ```python
        from llama_index.core import VectorStoreIndex, Document
        from llamaindex_agentmem import AgentMemory
        from agentmem import Config

        # Initialize with AgentMem config
        config = Config(
            api_base_url="http://localhost:8080",
            api_key="your-api-key"
        )
        memory = AgentMemory(config=config, agent_id="my_agent")

        # Add documents
        docs = [Document(text="Hello world"), Document(text="Another document")]
        memory.put(docs)

        # Retrieve relevant documents
        results = memory.get("search query")
        ```
    """

    def __init__(
        self,
        config: Optional[Config] = None,
        connection_string: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        user_id: str = "default",
        agent_id: Optional[str] = None,
        memory_type: str = "semantic",
        top_k: int = 10,
        **kwargs
    ):
        """
        Initialize AgentMemory.

        Args:
            config: AgentMem Config object (takes precedence)
            connection_string: AgentMem API base URL
            api_key: Optional API key for authentication
            user_id: User identifier for memory isolation
            agent_id: Agent identifier (auto-generated if not provided)
            memory_type: Type of memory storage ("semantic", "episodic", "procedural", "untyped")
            top_k: Default number of results to retrieve
            **kwargs: Additional parameters for AgentMemClient
        """
        if config is None:
            # Create config from parameters
            config = Config(
                api_base_url=connection_string,
                api_key=api_key or "",
            )

        self.config = config
        self.user_id = user_id
        self.agent_id = agent_id or f"llamaindex_agent_{id(self)}"
        self.memory_type = MemoryType(memory_type.lower())
        self.top_k = top_k
        self._client_kwargs = kwargs

        # Lazy initialization of client
        self._client: Optional[AgentMemClient] = None
        self._memory_ids: List[str] = []

        logger.info(
            f"Initialized AgentMemory for agent={self.agent_id}, "
            f"user={user_id}, type={memory_type}"
        )

    @property
    def client(self) -> AgentMemClient:
        """Get or create AgentMem client (lazy initialization)."""
        if self._client is None:
            self._client = AgentMemClient(self.config, **self._client_kwargs)
        return self._client

    async def _async_add_documents(
        self,
        documents: List[Document],
        importance: float = 0.5,
        **metadata
    ) -> List[str]:
        """
        Add documents to memory asynchronously.

        Args:
            documents: List of LlamaIndex Documents
            importance: Memory importance score (0.0 to 1.0)
            **metadata: Additional metadata to attach

        Returns:
            List of memory IDs
        """
        memory_ids = []

        for doc in documents:
            # Merge metadata
            doc_metadata = doc.metadata or {}
            doc_metadata.update(metadata)

            # Add document to AgentMem
            memory_id = await self.client.add_memory(
                content=doc.text,
                agent_id=self.agent_id,
                user_id=self.user_id,
                memory_type=self.memory_type,
                importance=importance,
                metadata=doc_metadata,
            )
            memory_ids.append(memory_id)

        self._memory_ids.extend(memory_ids)
        logger.debug(f"Added {len(memory_ids)} documents to AgentMem")
        return memory_ids

    def get(self, input_str: str, **kwargs) -> List[Document]:
        """
        Retrieve relevant documents based on input string.

        Args:
            input_str: Query string for semantic search
            **kwargs: Additional search parameters
                - limit: Maximum number of results (default: self.top_k)
                - min_importance: Minimum importance score (default: 0.0)
                - filters: Metadata filters (default: None)

        Returns:
            List of relevant Documents sorted by relevance
        """
        import asyncio

        return asyncio.run(self._async_get(input_str, **kwargs))

    async def _async_get(self, input_str: str, **kwargs) -> List[Document]:
        """Async implementation of get()."""
        limit = kwargs.get("limit", self.top_k)
        min_importance = kwargs.get("min_importance", 0.0)
        filters = kwargs.get("filters")

        # Build search query
        query = SearchQuery(
            agent_id=self.agent_id,
            text_query=input_str,
            limit=limit,
            min_importance=min_importance,
            metadata_filters=filters,
        )

        # Execute search
        results = await self.client.search_memories(query)

        # Convert to LlamaIndex Documents
        documents = []
        for result in results:
            doc = Document(
                text=result.memory.content,
                metadata={
                    "memory_id": result.memory.id,
                    "importance": result.memory.importance,
                    "created_at": result.memory.created_at.isoformat(),
                    "memory_type": result.memory.memory_type.value,
                    "score": result.score,
                    **(result.memory.metadata or {}),
                },
            )
            documents.append(doc)

        logger.debug(f"Retrieved {len(documents)} documents for query: {input_str}")
        return documents

    def get_all(self, **kwargs) -> List[Document]:
        """
        Get all documents from memory.

        Args:
            **kwargs: Optional filters
                - limit: Maximum number of results (default: 100)
                - agent_id: Override agent ID filter

        Returns:
            List of all Documents
        """
        import asyncio

        return asyncio.run(self._async_get_all(**kwargs))

    async def _async_get_all(self, **kwargs) -> List[Document]:
        """Async implementation of get_all()."""
        limit = kwargs.get("limit", 100)
        agent_id = kwargs.get("agent_id", self.agent_id)

        memories = await self.client.get_all_memories(
            agent_id=agent_id,
            user_id=self.user_id,
            limit=limit,
        )

        documents = []
        for memory in memories:
            doc = Document(
                text=memory.content,
                metadata={
                    "memory_id": memory.id,
                    "importance": memory.importance,
                    "created_at": memory.created_at.isoformat(),
                    "memory_type": memory.memory_type.value,
                    **(memory.metadata or {}),
                },
            )
            documents.append(doc)

        logger.debug(f"Retrieved {len(documents)} total documents")
        return documents

    def put(self, documents: List[Document], **kwargs) -> None:
        """
        Add documents to memory.

        Args:
            documents: List of Documents to add
            **kwargs: Additional parameters
                - importance: Memory importance score (default: 0.5)
        """
        import asyncio

        asyncio.run(self._async_put(documents, **kwargs))

    async def _async_put(self, documents: List[Document], **kwargs) -> None:
        """Async implementation of put()."""
        importance = kwargs.get("importance", 0.5)
        await self._async_add_documents(documents, importance=importance)

    def set(self, documents: List[Document], **kwargs) -> None:
        """
        Replace all documents in memory with new documents.

        This clears existing memory and adds new documents.

        Args:
            documents: List of Documents to set
            **kwargs: Additional parameters
                - importance: Memory importance score (default: 0.5)
        """
        import asyncio

        asyncio.run(self._async_set(documents, **kwargs))

    async def _async_set(self, documents: List[Document], **kwargs) -> None:
        """Async implementation of set()."""
        # Clear existing memories
        await self._async_clear()

        # Add new documents
        importance = kwargs.get("importance", 0.5)
        await self._async_add_documents(documents, importance=importance)

    def delete(self, document: Document) -> None:
        """
        Delete a specific document from memory.

        Args:
            document: Document to delete (must have memory_id in metadata)
        """
        import asyncio

        asyncio.run(self._async_delete(document))

    async def _async_delete(self, document: Document) -> None:
        """Async implementation of delete()."""
        memory_id = document.metadata.get("memory_id")
        if not memory_id:
            raise ValueError("Document must have 'memory_id' in metadata to delete")

        await self.client.delete_memory(memory_id)
        self._memory_ids = [mid for mid in self._memory_ids if mid != memory_id]
        logger.debug(f"Deleted document with memory_id: {memory_id}")

    def clear(self) -> None:
        """Clear all documents from memory for this agent."""
        import asyncio

        asyncio.run(self._async_clear())

    async def _async_clear(self) -> None:
        """Async implementation of clear()."""
        # Delete all tracked memory IDs
        for memory_id in self._memory_ids:
            try:
                await self.client.delete_memory(memory_id)
            except Exception as e:
                logger.warning(f"Failed to delete memory {memory_id}: {e}")

        self._memory_ids.clear()
        logger.info(f"Cleared all documents for agent {self.agent_id}")

    def to_dict(self) -> Dict[str, Any]:
        """Serialize memory to dictionary."""
        return {
            "agent_id": self.agent_id,
            "user_id": self.user_id,
            "memory_type": self.memory_type.value,
            "top_k": self.top_k,
            "memory_count": len(self._memory_ids),
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AgentMemory":
        """Deserialize memory from dictionary."""
        return cls(
            agent_id=data["agent_id"],
            user_id=data["user_id"],
            memory_type=data["memory_type"],
            top_k=data["top_k"],
        )

    def __len__(self) -> int:
        """Return number of documents in memory."""
        return len(self._memory_ids)

    def __repr__(self) -> str:
        """String representation."""
        return (
            f"AgentMemory(agent_id='{self.agent_id}', "
            f"user_id='{self.user_id}', "
            f"type='{self.memory_type.value}', "
            f"count={len(self._memory_ids)})"
        )
