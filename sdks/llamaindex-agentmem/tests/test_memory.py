"""
Unit tests for AgentMemory class
"""

import pytest
from unittest.mock import Mock, AsyncMock, patch
from llama_index.core import Document

from llamaindex_agentmem import AgentMemory
from agentmem import Config, MemoryType


@pytest.fixture
def mock_config():
    """Create mock configuration."""
    return Config(
        api_base_url="http://localhost:8080",
        api_key="test-key",
    )


@pytest.fixture
def agent_memory(mock_config):
    """Create AgentMemory instance."""
    return AgentMemory(
        config=mock_config,
        agent_id="test_agent",
        user_id="test_user",
        memory_type="semantic",
    )


@pytest.fixture
def sample_documents():
    """Create sample documents for testing."""
    return [
        Document(text="First document", metadata={"index": 1}),
        Document(text="Second document", metadata={"index": 2}),
        Document(text="Third document", metadata={"index": 3}),
    ]


class TestAgentMemory:
    """Test suite for AgentMemory class."""

    def test_initialization(self, mock_config):
        """Test AgentMemory initialization."""
        memory = AgentMemory(
            config=mock_config,
            agent_id="test_agent",
            user_id="test_user",
            memory_type="semantic",
        )

        assert memory.agent_id == "test_agent"
        assert memory.user_id == "test_user"
        assert memory.memory_type == MemoryType.SEMANTIC
        assert memory.top_k == 10
        assert len(memory._memory_ids) == 0

    def test_initialization_with_defaults(self, mock_config):
        """Test initialization with default values."""
        memory = AgentMemory(config=mock_config)

        assert memory.user_id == "default"
        assert memory.memory_type == MemoryType.SEMANTIC
        assert memory.agent_id.startswith("llamaindex_agent_")

    @pytest.mark.asyncio
    async def test_add_documents(self, agent_memory, sample_documents):
        """Test adding documents to memory."""
        with patch.object(agent_memory.client, "add_memory", new_callable=AsyncMock) as mock_add:
            mock_add.return_value = "mem_123"

            memory_ids = await agent_memory._async_add_documents(
                sample_documents,
                importance=0.8,
            )

            assert len(memory_ids) == 3
            assert all(isinstance(mid, str) for mid in memory_ids)
            assert mock_add.call_count == 3

            # Verify calls
            for i, call in enumerate(mock_add.call_args_list):
                args, kwargs = call
                assert kwargs["agent_id"] == "test_agent"
                assert kwargs["user_id"] == "test_user"
                assert kwargs["importance"] == 0.8
                assert "metadata" in kwargs

    @pytest.mark.asyncio
    async def test_get_documents(self, agent_memory):
        """Test retrieving documents from memory."""
        mock_results = [
            Mock(
                memory=Mock(
                    id="mem_1",
                    content="Test content",
                    importance=0.8,
                    created_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                    memory_type=MemoryType.SEMANTIC,
                    metadata={"key": "value"},
                ),
                score=0.9,
            )
        ]

        with patch.object(
            agent_memory.client, "search_memories", new_callable=AsyncMock
        ) as mock_search:
            mock_search.return_value = mock_results

            documents = await agent_memory._async_get("test query", limit=5)

            assert len(documents) == 1
            assert documents[0].text == "Test content"
            assert documents[0].metadata["memory_id"] == "mem_1"
            assert documents[0].metadata["score"] == 0.9

    @pytest.mark.asyncio
    async def test_get_all_documents(self, agent_memory):
        """Test getting all documents from memory."""
        mock_memories = [
            Mock(
                id="mem_1",
                content="Content 1",
                importance=0.5,
                created_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                memory_type=MemoryType.SEMANTIC,
                agent_id="test_agent",
                user_id="test_user",
                session_id=None,
                metadata=None,
            ),
            Mock(
                id="mem_2",
                content="Content 2",
                importance=0.7,
                created_at=Mock(isoformat=Mock(return_value="2024-01-02")),
                memory_type=MemoryType.EPISODIC,
                agent_id="test_agent",
                user_id="test_user",
                session_id=None,
                metadata=None,
            ),
        ]

        with patch.object(
            agent_memory.client, "get_all_memories", new_callable=AsyncMock
        ) as mock_get_all:
            mock_get_all.return_value = mock_memories

            documents = await agent_memory._async_get_all(limit=100)

            assert len(documents) == 2
            assert documents[0].text == "Content 1"
            assert documents[1].text == "Content 2"

    @pytest.mark.asyncio
    async def test_put_documents(self, agent_memory, sample_documents):
        """Test put method (add documents)."""
        with patch.object(
            agent_memory, "_async_add_documents", new_callable=AsyncMock
        ) as mock_add:
            await agent_memory._async_put(sample_documents, importance=0.6)

            mock_add.assert_called_once_with(sample_documents, importance=0.6)

    @pytest.mark.asyncio
    async def test_set_documents(self, agent_memory, sample_documents):
        """Test set method (replace documents)."""
        with patch.object(agent_memory, "_async_clear", new_callable=AsyncMock), patch.object(
            agent_memory, "_async_add_documents", new_callable=AsyncMock
        ) as mock_add:
            await agent_memory._async_set(sample_documents, importance=0.7)

            # Should clear first, then add
            mock_add.assert_called_once_with(sample_documents, importance=0.7)

    @pytest.mark.asyncio
    async def test_delete_document(self, agent_memory):
        """Test deleting a specific document."""
        doc = Document(
            text="Test",
            metadata={"memory_id": "mem_123"},
        )

        with patch.object(
            agent_memory.client, "delete_memory", new_callable=AsyncMock
        ) as mock_delete:
            await agent_memory._async_delete(doc)

            mock_delete.assert_called_once_with("mem_123")

    @pytest.mark.asyncio
    async def test_delete_document_without_memory_id(self, agent_memory):
        """Test deleting document without memory_id raises error."""
        doc = Document(text="Test", metadata={})

        with pytest.raises(ValueError, match="must have 'memory_id'"):
            await agent_memory._async_delete(doc)

    @pytest.mark.asyncio
    async def test_clear_memory(self, agent_memory):
        """Test clearing all documents."""
        agent_memory._memory_ids = ["mem_1", "mem_2", "mem_3"]

        with patch.object(
            agent_memory.client, "delete_memory", new_callable=AsyncMock
        ) as mock_delete:
            await agent_memory._async_clear()

            assert mock_delete.call_count == 3
            assert len(agent_memory._memory_ids) == 0

    def test_to_dict(self, agent_memory):
        """Test serialization to dictionary."""
        agent_memory._memory_ids = ["mem_1", "mem_2"]

        data = agent_memory.to_dict()

        assert data["agent_id"] == "test_agent"
        assert data["user_id"] == "test_user"
        assert data["memory_type"] == "semantic"
        assert data["top_k"] == 10
        assert data["memory_count"] == 2

    def test_from_dict(self):
        """Test deserialization from dictionary."""
        data = {
            "agent_id": "test_agent",
            "user_id": "test_user",
            "memory_type": "semantic",
            "top_k": 10,
        }

        memory = AgentMemory.from_dict(data)

        assert memory.agent_id == "test_agent"
        assert memory.user_id == "test_user"
        assert memory.memory_type == MemoryType.SEMANTIC

    def test_len(self, agent_memory):
        """Test __len__ method."""
        agent_memory._memory_ids = ["mem_1", "mem_2", "mem_3"]
        assert len(agent_memory) == 3

    def test_repr(self, agent_memory):
        """Test __repr__ method."""
        agent_memory._memory_ids = ["mem_1", "mem_2"]

        repr_str = repr(agent_memory)

        assert "AgentMemory" in repr_str
        assert "test_agent" in repr_str
        assert "test_user" in repr_str
        assert "count=2" in repr_str
