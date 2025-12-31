"""
AgentMem Document Reader for LlamaIndex

LlamaIndex BaseReader implementation for loading documents from AgentMem.
"""

import logging
from typing import List, Dict, Any, Optional

from llama_index.core.readers.base import BaseReader
from llama_index.core import Document

try:
    from agentmem import AgentMemClient, SearchQuery, Config
except ImportError:
    raise ImportError(
        "agentmem package is required. Install it with: pip install agentmem"
    )

logger = logging.getLogger(__name__)


class AgentMemReader(BaseReader):
    """
    LlamaIndex Document Reader for AgentMem.

    This reader allows you to load documents from AgentMem into LlamaIndex
    for indexing and retrieval operations.

    Features:
    - Load documents by semantic search query
    - Filter by agent, user, session, and metadata
    - Supports importance-based filtering
    - Returns LlamaIndex Document objects

    Example:
        ```python
        from llamaindex_agentmem import AgentMemReader
        from llama_index.core import VectorStoreIndex

        # Initialize reader
        reader = AgentMemReader(
            connection_string="http://localhost:8080",
            api_key="your-api-key"
        )

        # Load documents by query
        docs = reader.load_data(
            query="machine learning papers",
            agent_id="research_agent",
            limit=10
        )

        # Create index
        index = VectorStoreIndex.from_documents(docs)
        ```
    """

    def __init__(
        self,
        config: Optional[Config] = None,
        connection_string: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        **kwargs
    ):
        """
        Initialize AgentMemReader.

        Args:
            config: AgentMem Config object (takes precedence)
            connection_string: AgentMem API base URL
            api_key: Optional API key for authentication
            **kwargs: Additional parameters for AgentMemClient
        """
        if config is None:
            config = Config(
                api_base_url=connection_string,
                api_key=api_key or "",
            )

        self.config = config
        self._client_kwargs = kwargs
        self._client: Optional[AgentMemClient] = None

        logger.info("Initialized AgentMemReader")

    @property
    def client(self) -> AgentMemClient:
        """Get or create AgentMem client (lazy initialization)."""
        if self._client is None:
            self._client = AgentMemClient(self.config, **self._client_kwargs)
        return self._client

    def load_data(
        self,
        query: str,
        agent_id: str = "default",
        user_id: Optional[str] = None,
        session_id: Optional[str] = None,
        limit: int = 10,
        min_importance: float = 0.0,
        metadata_filters: Optional[Dict[str, Any]] = None,
    ) -> List[Document]:
        """
        Load documents from AgentMem based on search query.

        Args:
            query: Search query for semantic matching
            agent_id: Agent identifier to search within
            user_id: Optional user identifier filter
            session_id: Optional session identifier filter
            limit: Maximum number of documents to return
            min_importance: Minimum importance score threshold
            metadata_filters: Optional metadata filter dictionary

        Returns:
            List of LlamaIndex Documents

        Example:
            ```python
            reader = AgentMemReader()

            # Simple search
            docs = reader.load_data(
                query="research papers",
                agent_id="my_agent",
                limit=5
            )

            # Advanced search with filters
            docs = reader.load_data(
                query="machine learning",
                agent_id="research_agent",
                min_importance=0.7,
                metadata_filters={"category": "AI"}
            )
            ```
        """
        import asyncio

        return asyncio.run(
            self._async_load_data(
                query=query,
                agent_id=agent_id,
                user_id=user_id,
                session_id=session_id,
                limit=limit,
                min_importance=min_importance,
                metadata_filters=metadata_filters,
            )
        )

    async def _async_load_data(
        self,
        query: str,
        agent_id: str = "default",
        user_id: Optional[str] = None,
        session_id: Optional[str] = None,
        limit: int = 10,
        min_importance: float = 0.0,
        metadata_filters: Optional[Dict[str, Any]] = None,
    ) -> List[Document]:
        """Async implementation of load_data()."""
        # Build search query
        search_query = SearchQuery(
            agent_id=agent_id,
            text_query=query,
            limit=limit,
            min_importance=min_importance,
            metadata_filters=metadata_filters,
        )

        # Add optional filters
        if user_id:
            search_query.user_id = user_id
        if session_id:
            search_query.session_id = session_id

        # Execute search
        results = await self.client.search_memories(search_query)

        # Convert to LlamaIndex Documents
        documents = []
        for result in results:
            metadata = {
                "memory_id": result.memory.id,
                "agent_id": result.memory.agent_id,
                "importance": result.memory.importance,
                "created_at": result.memory.created_at.isoformat(),
                "updated_at": result.memory.updated_at.isoformat(),
                "memory_type": result.memory.memory_type.value,
                "score": result.score,
            }

            # Add user_id and session_id if present
            if result.memory.user_id:
                metadata["user_id"] = result.memory.user_id
            if result.memory.session_id:
                metadata["session_id"] = result.memory.session_id

            # Merge with custom metadata
            if result.memory.metadata:
                metadata.update(result.memory.metadata)

            doc = Document(
                text=result.memory.content,
                metadata=metadata,
            )
            documents.append(doc)

        logger.info(
            f"Loaded {len(documents)} documents from AgentMem for query: {query}"
        )
        return documents

    def load_all_memories(
        self,
        agent_id: str = "default",
        user_id: Optional[str] = None,
        limit: int = 100,
    ) -> List[Document]:
        """
        Load all memories for an agent (not search-based).

        Args:
            agent_id: Agent identifier
            user_id: Optional user identifier filter
            limit: Maximum number of memories to return

        Returns:
            List of all LlamaIndex Documents
        """
        import asyncio

        return asyncio.run(
            self._async_load_all_memories(
                agent_id=agent_id,
                user_id=user_id,
                limit=limit,
            )
        )

    async def _async_load_all_memories(
        self,
        agent_id: str = "default",
        user_id: Optional[str] = None,
        limit: int = 100,
    ) -> List[Document]:
        """Async implementation of load_all_memories()."""
        memories = await self.client.get_all_memories(
            agent_id=agent_id,
            user_id=user_id,
            limit=limit,
        )

        documents = []
        for memory in memories:
            metadata = {
                "memory_id": memory.id,
                "agent_id": memory.agent_id,
                "importance": memory.importance,
                "created_at": memory.created_at.isoformat(),
                "updated_at": memory.updated_at.isoformat(),
                "memory_type": memory.memory_type.value,
            }

            if memory.user_id:
                metadata["user_id"] = memory.user_id
            if memory.session_id:
                metadata["session_id"] = memory.session_id
            if memory.metadata:
                metadata.update(memory.metadata)

            doc = Document(
                text=memory.content,
                metadata=metadata,
            )
            documents.append(doc)

        logger.info(f"Loaded {len(documents)} total memories from AgentMem")
        return documents

    async def close(self) -> None:
        """Close the AgentMem client connection."""
        if self._client is not None:
            await self._client.close()
            self._client = None

    async def __aenter__(self):
        """Async context manager entry."""
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()

    def __repr__(self) -> str:
        """String representation."""
        return f"AgentMemReader(connection='{self.config.api_base_url}')"
