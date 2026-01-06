"""
Integration tests for LlamaIndex-AgentMem

These tests verify the integration between LlamaIndex and AgentMem.
Note: These require a running AgentMem server or mocked responses.
"""

import pytest
from unittest.mock import Mock, AsyncMock, patch, MagicMock
from llama_index.core import Document, VectorStoreIndex

from llamaindex_agentmem import AgentMemory, AgentMemReader, AgentMemNodeParser
from agentmem import Config, MemoryType


@pytest.fixture
def mock_config():
    """Create mock configuration."""
    return Config(
        api_base_url="http://localhost:8080",
        api_key="test-key",
    )


@pytest.fixture
def sample_documents():
    """Create sample documents."""
    return [
        Document(
            text="Machine learning is a subset of artificial intelligence.",
            metadata={"category": "AI", "importance": "high"},
        ),
        Document(
            text="Deep learning uses neural networks with multiple layers.",
            metadata={"category": "AI", "importance": "high"},
        ),
        Document(
            text="Python is a popular programming language for ML.",
            metadata={"category": "programming", "importance": "medium"},
        ),
    ]


class TestAgentMemoryIntegration:
    """Integration tests for AgentMemory."""

    @pytest.mark.asyncio
    async def test_full_memory_workflow(self, mock_config, sample_documents):
        """Test complete workflow: add, search, update, delete."""
        memory = AgentMemory(config=mock_config, agent_id="workflow_test")

        # Mock the client methods
        with patch.object(memory.client, "add_memory", new_callable=AsyncMock) as mock_add, \
             patch.object(memory.client, "search_memories", new_callable=AsyncMock) as mock_search, \
             patch.object(memory.client, "delete_memory", new_callable=AsyncMock) as mock_delete:

            mock_add.side_effect = ["mem_1", "mem_2", "mem_3"]

            # Add documents
            await memory._async_put(sample_documents)
            assert len(memory._memory_ids) == 3

            # Search
            mock_result = Mock(
                memory=Mock(
                    id="mem_1",
                    content=sample_documents[0].text,
                    importance=0.8,
                    created_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                    memory_type=MemoryType.SEMANTIC,
                    metadata=sample_documents[0].metadata,
                ),
                score=0.9,
            )
            mock_search.return_value = [mock_result]

            results = await memory._async_get("machine learning")
            assert len(results) == 1
            assert results[0].text == sample_documents[0].text

            # Delete all
            await memory._async_clear()
            assert mock_delete.call_count == 3

    @pytest.mark.asyncio
    async def test_metadata_filtering(self, mock_config):
        """Test filtering documents by metadata."""
        memory = AgentMemory(config=mock_config, agent_id="filter_test")

        with patch.object(memory.client, "search_memories", new_callable=AsyncMock) as mock_search:
            # Create filtered results
            mock_result = Mock(
                memory=Mock(
                    id="mem_1",
                    content="AI content",
                    importance=0.8,
                    created_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                    memory_type=MemoryType.SEMANTIC,
                    metadata={"category": "AI"},
                ),
                score=0.9,
            )
            mock_search.return_value = [mock_result]

            results = await memory._async_get(
                "query",
                filters={"category": "AI"},
            )

            assert len(results) == 1
            assert results[0].metadata["category"] == "AI"


class TestAgentMemReaderIntegration:
    """Integration tests for AgentMemReader."""

    @pytest.mark.asyncio
    async def test_reader_to_index_workflow(self, mock_config):
        """Test loading documents from AgentMemReader to LlamaIndex index."""
        reader = AgentMemReader(config=mock_config)

        with patch.object(reader.client, "search_memories", new_callable=AsyncMock) as mock_search:
            # Mock search results
            mock_results = [
                Mock(
                    memory=Mock(
                        id="mem_1",
                        content="Document 1 content",
                        agent_id="agent_1",
                        user_id=None,
                        session_id=None,
                        importance=0.8,
                        created_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                        updated_at=Mock(isoformat=Mock(return_value="2024-01-01")),
                        memory_type=Mock(value="semantic"),
                        metadata={"source": "doc1"},
                    ),
                    score=0.9,
                ),
                Mock(
                    memory=Mock(
                        id="mem_2",
                        content="Document 2 content",
                        agent_id="agent_1",
                        user_id=None,
                        session_id=None,
                        importance=0.7,
                        created_at=Mock(isoformat=Mock(return_value="2024-01-02")),
                        updated_at=Mock(isoformat=Mock(return_value="2024-01-02")),
                        memory_type=Mock(value="semantic"),
                        metadata={"source": "doc2"},
                    ),
                    score=0.8,
                ),
            ]
            mock_search.return_value = mock_results

            # Load documents
            documents = await reader._async_load_data(
                query="test query",
                agent_id="agent_1",
            )

            assert len(documents) == 2
            assert all(isinstance(doc, Document) for doc in documents)


class TestEndToEndScenarios:
    """End-to-end integration scenarios."""

    @pytest.mark.asyncio
    async def test_chatbot_memory_scenario(self, mock_config):
        """Simulate a chatbot using AgentMem for conversation history."""
        memory = AgentMemory(
            config=mock_config,
            agent_id="chatbot",
            user_id="user_123",
            memory_type="episodic",
        )

        with patch.object(memory.client, "add_memory", new_callable=AsyncMock) as mock_add, \
             patch.object(memory.client, "search_memories", new_callable=AsyncMock) as mock_search:

            # Simulate conversation
            conversation = [
                ("User: Hello", "Bot: Hi there!"),
                ("User: How are you?", "Bot: I'm doing well!"),
            ]

            # Store conversation
            for i, (user_msg, bot_msg) in enumerate(conversation):
                doc = Document(
                    text=f"{user_msg}\n{bot_msg}",
                    metadata={"turn": i, "type": "conversation"},
                )
                await memory._async_add_documents([doc])

            assert len(memory._memory_ids) == 2
