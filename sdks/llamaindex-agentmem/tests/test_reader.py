"""
Unit tests for AgentMemReader class
"""

import pytest
from unittest.mock import Mock, AsyncMock, patch
from llama_index.core import Document

from llamaindex_agentmem import AgentMemReader
from agentmem import Config


@pytest.fixture
def mock_config():
    """Create mock configuration."""
    return Config(
        api_base_url="http://localhost:8080",
        api_key="test-key",
    )


@pytest.fixture
def agent_mem_reader(mock_config):
    """Create AgentMemReader instance."""
    return AgentMemReader(config=mock_config)


@pytest.fixture
def mock_search_results():
    """Create mock search results."""
    return [
        Mock(
            memory=Mock(
                id="mem_1",
                content="First result",
                agent_id="agent_1",
                user_id="user_1",
                session_id="session_1",
                importance=0.8,
                created_at=Mock(isoformat=Mock(return_value="2024-01-01T00:00:00")),
                updated_at=Mock(isoformat=Mock(return_value="2024-01-01T01:00:00")),
                memory_type=Mock(value="semantic"),
                metadata={"category": "test"},
            ),
            score=0.95,
        ),
        Mock(
            memory=Mock(
                id="mem_2",
                content="Second result",
                agent_id="agent_1",
                user_id="user_1",
                session_id="session_1",
                importance=0.6,
                created_at=Mock(isoformat=Mock(return_value="2024-01-02T00:00:00")),
                updated_at=Mock(isoformat=Mock(return_value="2024-01-02T01:00:00")),
                memory_type=Mock(value="episodic"),
                metadata=None,
            ),
            score=0.85,
        ),
    ]


@pytest.fixture
def mock_memories():
    """Create mock memories."""
    return [
        Mock(
            id="mem_1",
            content="Memory 1",
            agent_id="agent_1",
            user_id="user_1",
            session_id="session_1",
            importance=0.7,
            created_at=Mock(isoformat=Mock(return_value="2024-01-01T00:00:00")),
            updated_at=Mock(isoformat=Mock(return_value="2024-01-01T01:00:00")),
            memory_type=Mock(value="semantic"),
            metadata={"key": "value"},
        ),
        Mock(
            id="mem_2",
            content="Memory 2",
            agent_id="agent_1",
            user_id=None,
            session_id=None,
            importance=0.5,
            created_at=Mock(isoformat=Mock(return_value="2024-01-02T00:00:00")),
            updated_at=Mock(isoformat=Mock(return_value="2024-01-02T01:00:00")),
            memory_type=Mock(value="episodic"),
            metadata=None,
        ),
    ]


class TestAgentMemReader:
    """Test suite for AgentMemReader class."""

    def test_initialization(self, mock_config):
        """Test AgentMemReader initialization."""
        reader = AgentMemReader(config=mock_config)

        assert reader.config == mock_config
        assert reader._client is None

    def test_initialization_with_connection_string(self):
        """Test initialization with connection string parameter."""
        reader = AgentMemReader(
            connection_string="http://custom-url:9000",
            api_key="custom-key",
        )

        assert reader.config.api_base_url == "http://custom-url:9000"
        assert reader.config.api_key == "custom-key"

    @pytest.mark.asyncio
    async def test_load_data(
        self, agent_mem_reader, mock_search_results
    ):
        """Test loading documents by search query."""
        with patch.object(
            agent_mem_reader.client, "search_memories", new_callable=AsyncMock
        ) as mock_search:
            mock_search.return_value = mock_search_results

            documents = await agent_mem_reader._async_load_data(
                query="test query",
                agent_id="agent_1",
                user_id="user_1",
                limit=10,
            )

            assert len(documents) == 2
            assert documents[0].text == "First result"
            assert documents[1].text == "Second result"

            # Verify metadata
            assert documents[0].metadata["memory_id"] == "mem_1"
            assert documents[0].metadata["score"] == 0.95
            assert documents[0].metadata["agent_id"] == "agent_1"
            assert documents[0].metadata["user_id"] == "user_1"
            assert documents[0].metadata["session_id"] == "session_1"
            assert documents[0].metadata["category"] == "test"

            assert documents[1].metadata["memory_id"] == "mem_2"
            assert documents[1].metadata["score"] == 0.85

    @pytest.mark.asyncio
    async def test_load_data_with_filters(self, agent_mem_reader, mock_search_results):
        """Test loading documents with metadata filters."""
        with patch.object(
            agent_mem_reader.client, "search_memories", new_callable=AsyncMock
        ) as mock_search:
            mock_search.return_value = mock_search_results

            documents = await agent_mem_reader._async_load_data(
                query="test query",
                agent_id="agent_1",
                min_importance=0.7,
                metadata_filters={"category": "test"},
            )

            assert len(documents) == 2

            # Verify search query parameters
            call_args = mock_search.call_args
            search_query = call_args[0][0]

            assert search_query.min_importance == 0.7
            assert search_query.metadata_filters == {"category": "test"}

    @pytest.mark.asyncio
    async def test_load_all_memories(self, agent_mem_reader, mock_memories):
        """Test loading all memories."""
        with patch.object(
            agent_mem_reader.client, "get_all_memories", new_callable=AsyncMock
        ) as mock_get_all:
            mock_get_all.return_value = mock_memories

            documents = await agent_mem_reader._async_load_all_memories(
                agent_id="agent_1",
                user_id="user_1",
                limit=100,
            )

            assert len(documents) == 2
            assert documents[0].text == "Memory 1"
            assert documents[1].text == "Memory 2"

            # Verify metadata
            assert documents[0].metadata["memory_id"] == "mem_1"
            assert documents[0].metadata["agent_id"] == "agent_1"
            assert documents[0].metadata["user_id"] == "user_1"
            assert documents[0].metadata["key"] == "value"

            # Second memory has no user_id or session_id
            assert "user_id" not in documents[1].metadata
            assert "session_id" not in documents[1].metadata

    @pytest.mark.asyncio
    async def test_close(self, agent_mem_reader):
        """Test closing the client connection."""
        # Initialize client
        _ = agent_mem_reader.client

        with patch.object(agent_mem_reader._client, "aclose", new_callable=AsyncMock) as mock_close:
            await agent_mem_reader.close()

            mock_close.assert_called_once()
            assert agent_mem_reader._client is None

    @pytest.mark.asyncio
    async def test_async_context_manager(self, agent_mem_reader):
        """Test using reader as async context manager."""
        with patch.object(agent_mem_reader, "close", new_callable=AsyncMock) as mock_close:
            async with agent_mem_reader:
                pass

            mock_close.assert_called_once()

    def test_repr(self, agent_mem_reader):
        """Test __repr__ method."""
        repr_str = repr(agent_mem_reader)

        assert "AgentMemReader" in repr_str
        assert "http://localhost:8080" in repr_str

    @pytest.mark.asyncio
    async def test_load_data_without_user_id(self, agent_mem_reader, mock_search_results):
        """Test loading documents without optional user_id."""
        # Create results without user_id
        modified_results = [Mock(
            memory=Mock(
                id="mem_1",
                content="Result",
                agent_id="agent_1",
                user_id=None,
                session_id=None,
                importance=0.8,
                created_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                updated_at=Mock(isoformat=Mock(return_value="2024-01-02")),
                memory_type=Mock(value="semantic"),
                metadata=None,
            ),
            score=0.9,
        )]

        with patch.object(
            agent_mem_reader.client, "search_memories", new_callable=AsyncMock
        ) as mock_search:
            mock_search.return_value = modified_results

            documents = await agent_mem_reader._async_load_data(
                query="test",
                agent_id="agent_1",
            )

            assert len(documents) == 1
            # user_id should not be in metadata if None
            assert "user_id" not in documents[0].metadata
            assert "session_id" not in documents[0].metadata
