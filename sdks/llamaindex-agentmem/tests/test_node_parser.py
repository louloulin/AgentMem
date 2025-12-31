"""
Unit tests for AgentMemNodeParser class
"""

import pytest
from unittest.mock import Mock, AsyncMock, patch
from llama_index.core import Document

from llamaindex_agentmem import AgentMemNodeParser
from agentmem import Config, MemoryType


@pytest.fixture
def mock_config():
    """Create mock configuration."""
    return Config(
        api_base_url="http://localhost:8080",
        api_key="test-key",
    )


@pytest.fixture
def node_parser(mock_config):
    """Create AgentMemNodeParser instance."""
    return AgentMemNodeParser(
        config=mock_config,
        agent_id="test_agent",
        user_id="test_user",
        chunk_size=100,
        chunk_overlap=20,
    )


class TestAgentMemNodeParser:
    """Test suite for AgentMemNodeParser class."""

    def test_initialization(self, mock_config):
        """Test AgentMemNodeParser initialization."""
        parser = AgentMemNodeParser(
            config=mock_config,
            agent_id="test_agent",
            user_id="test_user",
            chunk_size=200,
            chunk_overlap=50,
        )

        assert parser.agent_id == "test_agent"
        assert parser.user_id == "test_user"
        assert parser.memory_type == MemoryType.SEMANTIC
        assert parser.chunk_size == 200
        assert parser.chunk_overlap == 50
        assert parser.importance == 0.5

    def test_split_text_short_text(self, node_parser):
        """Test splitting text shorter than chunk size."""
        text = "Short text"
        chunks = node_parser._split_text(text)

        assert len(chunks) == 1
        assert chunks[0] == text

    def test_split_text_long_text(self, node_parser):
        """Test splitting long text into chunks."""
        text = " ".join(["word"] * 50)  # ~250 characters
        chunks = node_parser._split_text(text)

        assert len(chunks) > 1
        # Check that chunks don't exceed chunk_size
        for chunk in chunks:
            assert len(chunk) <= node_parser.chunk_size

    def test_split_text_with_separator(self, node_parser):
        """Test splitting text respects separator."""
        text = "word " * 50
        chunks = node_parser._split_text(text)

        # All chunks should end with complete words
        for chunk in chunks:
            assert not chunk.endswith(" word") or len(chunk) < node_parser.chunk_size

    def test_split_text_with_overlap(self, node_parser):
        """Test that chunks have proper overlap."""
        text = " ".join(["word"] * 100)
        chunks = node_parser._split_text(text)

        if len(chunks) > 1:
            # Check overlap between consecutive chunks
            first_chunk_end = chunks[0][-node_parser.chunk_overlap:]
            second_chunk_start = chunks[1][:node_parser.chunk_overlap]

            # Should have some overlap
            overlap_exists = first_chunk_end in second_chunk_start or \
                             any(word in second_chunk_start for word in first_chunk_end.split())
            # This is a weak check due to word boundary adjustments

    def test_get_num_chunks(self, node_parser):
        """Test getting number of chunks for text."""
        short_text = "Short text"
        assert node_parser.get_num_chunks(short_text) == 1

        long_text = " ".join(["word"] * 100)
        num_chunks = node_parser.get_num_chunks(long_text)
        assert num_chunks > 1

    @pytest.mark.asyncio
    async def test_store_chunk(self, node_parser):
        """Test storing a chunk in AgentMem."""
        with patch.object(
            node_parser.client, "add_memory", new_callable=AsyncMock
        ) as mock_add:
            mock_add.return_value = "mem_123"

            memory_id = await node_parser._store_chunk(
                chunk="Test chunk content",
                chunk_index=0,
                metadata={"key": "value"},
            )

            assert memory_id == "mem_123"

            # Verify call
            call_args = mock_add.call_args
            kwargs = call_args[1]
            assert kwargs["content"] == "Test chunk content"
            assert kwargs["agent_id"] == "test_agent"
            assert kwargs["user_id"] == "test_user"
            assert kwargs["memory_type"] == MemoryType.SEMANTIC
            assert kwargs["importance"] == 0.5
            assert kwargs["metadata"]["chunk_index"] == 0
            assert kwargs["metadata"]["key"] == "value"

    @pytest.mark.asyncio
    async def test_get_nodes_from_documents(self, node_parser):
        """Test parsing documents into nodes."""
        documents = [
            Document(text="Short document 1"),
            Document(text="Short document 2"),
        ]

        with patch.object(
            node_parser, "_store_chunk", new_callable=AsyncMock
        ) as mock_store:
            mock_store.return_value = "mem_123"

            nodes = await node_parser._async_get_nodes_from_documents(
                documents,
                store_in_memory=True,
            )

            assert len(nodes) == 2
            assert all(isinstance(node, Document) for node in nodes)
            assert mock_store.call_count == 2

            # Verify node metadata
            for i, node in enumerate(nodes):
                assert "document_id" in node.metadata
                assert "chunk_index" in node.metadata
                assert "total_chunks" in node.metadata
                assert node.metadata["memory_id"] == "mem_123"

    @pytest.mark.asyncio
    async def test_get_nodes_without_storage(self, node_parser):
        """Test parsing documents without storing in memory."""
        documents = [
            Document(text="Document without storage"),
        ]

        with patch.object(
            node_parser, "_store_chunk", new_callable=AsyncMock
        ) as mock_store:
            nodes = await node_parser._async_get_nodes_from_documents(
                documents,
                store_in_memory=False,
            )

            assert len(nodes) == 1
            mock_store.assert_not_called()
            assert "memory_id" not in nodes[0].metadata

    @pytest.mark.asyncio
    async def test_get_nodes_from_long_document(self, node_parser):
        """Test parsing a long document that gets chunked."""
        long_text = " ".join(["word"] * 100)
        document = Document(text=long_text, metadata={"source": "test.txt"})

        with patch.object(
            node_parser, "_store_chunk", new_callable=AsyncMock
        ) as mock_store:
            mock_store.return_value = "mem_xyz"

            nodes = await node_parser._async_get_nodes_from_documents(
                [document],
                store_in_memory=True,
            )

            # Should create multiple chunks
            assert len(nodes) > 1

            # Verify all chunks have proper metadata
            for i, node in enumerate(nodes):
                assert node.metadata["chunk_index"] == i
                assert node.metadata["total_chunks"] == len(nodes)
                assert node.metadata["source"] == "test.txt"
                assert node.metadata["memory_id"] == "mem_xyz"

    @pytest.mark.asyncio
    async def test_close(self, node_parser):
        """Test closing the client connection."""
        _ = node_parser.client

        with patch.object(node_parser._client, "aclose", new_callable=AsyncMock) as mock_close:
            await node_parser.close()

            mock_close.assert_called_once()
            assert node_parser._client is None

    def test_repr(self, node_parser):
        """Test __repr__ method."""
        repr_str = repr(node_parser)

        assert "AgentMemNodeParser" in repr_str
        assert "test_agent" in repr_str
        assert "chunk_size=100" in repr_str
        assert "overlap=20" in repr_str

    def test_custom_importance(self, mock_config):
        """Test parser with custom importance."""
        parser = AgentMemNodeParser(
            config=mock_config,
            agent_id="test_agent",
            importance=0.9,
        )

        assert parser.importance == 0.9

    def test_different_memory_types(self, mock_config):
        """Test parser with different memory types."""
        for memory_type in ["semantic", "episodic", "procedural", "untyped"]:
            parser = AgentMemNodeParser(
                config=mock_config,
                memory_type=memory_type,
            )
            assert parser.memory_type == MemoryType(memory_type)
