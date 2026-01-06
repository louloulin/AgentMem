"""
AgentMem Node Parser for LlamaIndex

Custom NodeParser that integrates with AgentMem for intelligent node splitting
and storage.
"""

import logging
from typing import List, Dict, Any, Optional

from llama_index.core.node_parser import NodeParser
from llama_index.core import Document
from llama_index.core.schema import BaseNode

try:
    from agentmem import AgentMemClient, MemoryType, Config
except ImportError:
    raise ImportError(
        "agentmem package is required. Install it with: pip install agentmem"
    )

logger = logging.getLogger(__name__)


class AgentMemNodeParser(NodeParser):
    """
    LlamaIndex Node Parser with AgentMem integration.

    This node parser extends LlamaIndex's NodeParser to automatically store
    parsed nodes in AgentMem for persistent, searchable memory.

    Features:
    - Automatic node storage in AgentMem
    - Semantic-aware chunking with metadata
    - Configurable chunk sizes and overlap
    - Agent-aware memory management

    Example:
        ```python
        from llamaindex_agentmem import AgentMemNodeParser
        from llama_index.core import Document

        # Initialize parser
        parser = AgentMemNodeParser(
            agent_id="my_agent",
            chunk_size=512,
            chunk_overlap=50
        )

        # Parse document and store nodes
        doc = Document(text="Long document text...")
        nodes = parser.get_nodes_from_documents([doc])

        # Nodes are automatically stored in AgentMem
        ```
    """

    def __init__(
        self,
        config: Optional[Config] = None,
        connection_string: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        agent_id: str = "default",
        user_id: str = "default",
        memory_type: str = "semantic",
        chunk_size: int = 512,
        chunk_overlap: int = 50,
        separator: str = " ",
        importance: float = 0.5,
        **kwargs
    ):
        """
        Initialize AgentMemNodeParser.

        Args:
            config: AgentMem Config object
            connection_string: AgentMem API base URL
            api_key: Optional API key for authentication
            agent_id: Agent identifier
            user_id: User identifier
            memory_type: Type of memory storage
            chunk_size: Maximum size of each chunk (in characters)
            chunk_overlap: Overlap between chunks
            separator: Separator string for splitting
            importance: Default importance score for chunks
            **kwargs: Additional parameters
        """
        super().__init__(**kwargs)

        if config is None:
            config = Config(
                api_base_url=connection_string,
                api_key=api_key or "",
            )

        self.config = config
        self.agent_id = agent_id
        self.user_id = user_id
        self.memory_type = MemoryType(memory_type.lower())
        self.chunk_size = chunk_size
        self.chunk_overlap = chunk_overlap
        self.separator = separator
        self.importance = importance
        self._client_kwargs = kwargs

        # Lazy initialization
        self._client: Optional[AgentMemClient] = None
        self._memory_ids: List[str] = []

        logger.info(
            f"Initialized AgentMemNodeParser for agent={agent_id}, "
            f"chunk_size={chunk_size}, overlap={chunk_overlap}"
        )

    @property
    def client(self) -> AgentMemClient:
        """Get or create AgentMem client."""
        if self._client is None:
            self._client = AgentMemClient(self.config, **self._client_kwargs)
        return self._client

    def _split_text(self, text: str) -> List[str]:
        """
        Split text into chunks.

        Args:
            text: Input text to split

        Returns:
            List of text chunks
        """
        if len(text) <= self.chunk_size:
            return [text]

        chunks = []
        start = 0
        text_length = len(text)

        while start < text_length:
            end = start + self.chunk_size

            # Adjust end position to avoid breaking words
            if end < text_length:
                # Find last separator before chunk_size
                last_sep = text.rfind(self.separator, start, end)
                if last_sep != -1 and last_sep > start:
                    end = last_sep + len(self.separator)

            chunk = text[start:end].strip()
            if chunk:
                chunks.append(chunk)

            # Move start position with overlap
            start = end - self.chunk_overlap
            if start < 0:
                start = 0

        return chunks

    async def _store_chunk(
        self,
        chunk: str,
        chunk_index: int,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> str:
        """
        Store a chunk in AgentMem.

        Args:
            chunk: Text chunk to store
            chunk_index: Index of this chunk in the document
            metadata: Optional metadata

        Returns:
            Memory ID
        """
        chunk_metadata = {
            "chunk_index": chunk_index,
            "chunk_size": len(chunk),
            "parser": "AgentMemNodeParser",
        }

        if metadata:
            chunk_metadata.update(metadata)

        memory_id = await self.client.add_memory(
            content=chunk,
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=self.memory_type,
            importance=self.importance,
            metadata=chunk_metadata,
        )

        return memory_id

    def get_nodes_from_documents(
        self,
        documents: List[Document],
        store_in_memory: bool = True,
        **kwargs
    ) -> List[BaseNode]:
        """
        Parse documents into nodes and optionally store in AgentMem.

        Args:
            documents: List of Documents to parse
            store_in_memory: Whether to store chunks in AgentMem (default: True)
            **kwargs: Additional parameters

        Returns:
            List of nodes
        """
        import asyncio

        return asyncio.run(
            self._async_get_nodes_from_documents(documents, store_in_memory, **kwargs)
        )

    async def _async_get_nodes_from_documents(
        self,
        documents: List[Document],
        store_in_memory: bool = True,
        **kwargs
    ) -> List[BaseNode]:
        """Async implementation of get_nodes_from_documents()."""
        all_nodes = []

        for doc_idx, document in enumerate(documents):
            # Split document into chunks
            chunks = self._split_text(document.text)

            logger.debug(
                f"Split document {doc_idx} into {len(chunks)} chunks"
            )

            # Create nodes for each chunk
            for chunk_idx, chunk in enumerate(chunks):
                # Build node metadata
                node_metadata = {
                    "document_id": getattr(document, "doc_id", f"doc_{doc_idx}"),
                    "chunk_index": chunk_idx,
                    "total_chunks": len(chunks),
                }

                # Merge with document metadata
                if document.metadata:
                    node_metadata.update(document.metadata)

                # Create node
                node = Document(
                    text=chunk,
                    metadata=node_metadata,
                )

                all_nodes.append(node)

                # Store in AgentMem if requested
                if store_in_memory:
                    memory_id = await self._store_chunk(
                        chunk,
                        chunk_idx,
                        node_metadata,
                    )
                    self._memory_ids.append(memory_id)

                    # Add memory_id to node metadata
                    node.metadata["memory_id"] = memory_id

        logger.info(
            f"Created {len(all_nodes)} nodes from {len(documents)} documents"
        )
        return all_nodes

    def get_num_chunks(self, text: str) -> int:
        """
        Get the number of chunks a text will be split into.

        Args:
            text: Input text

        Returns:
            Number of chunks
        """
        return len(self._split_text(text))

    async def close(self) -> None:
        """Close the AgentMem client connection."""
        if self._client is not None:
            await self._client.close()
            self._client = None

    def __repr__(self) -> str:
        """String representation."""
        return (
            f"AgentMemNodeParser(agent_id='{self.agent_id}', "
            f"chunk_size={self.chunk_size}, overlap={self.chunk_overlap})"
        )
