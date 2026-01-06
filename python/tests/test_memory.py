"""
Tests for the simple Memory API
"""

import asyncio
import pytest
from agentmem import Memory


@pytest.mark.asyncio
async def test_memory_add():
    """Test adding a memory."""
    memory = Memory()
    
    result = await memory.add(
        "User prefers Python over JavaScript",
        agent_id="assistant-1",
        user_id="user-123"
    )
    
    assert result["status"] == "success"
    assert "id" in result
    assert len(result["id"]) > 0


@pytest.mark.asyncio
async def test_memory_get():
    """Test getting a memory by ID."""
    memory = Memory()
    
    # Add a memory
    result = await memory.add(
        "User likes pizza",
        agent_id="assistant-1",
        user_id="user-123"
    )
    memory_id = result["id"]
    
    # Get the memory
    retrieved = await memory.get(memory_id)
    
    assert retrieved is not None
    assert retrieved["id"] == memory_id
    assert retrieved["content"] == "User likes pizza"
    assert retrieved["agent_id"] == "assistant-1"
    assert retrieved["user_id"] == "user-123"


@pytest.mark.asyncio
async def test_memory_get_all():
    """Test getting all memories with filters."""
    memory = Memory()
    
    # Add multiple memories
    await memory.add("Memory 1", agent_id="agent-1", user_id="user-1")
    await memory.add("Memory 2", agent_id="agent-1", user_id="user-2")
    await memory.add("Memory 3", agent_id="agent-2", user_id="user-1")
    
    # Get all memories for agent-1
    results = await memory.get_all(agent_id="agent-1")
    assert len(results) == 2
    
    # Get all memories for user-1
    results = await memory.get_all(user_id="user-1")
    assert len(results) == 2
    
    # Get all memories for agent-1 and user-1
    results = await memory.get_all(agent_id="agent-1", user_id="user-1")
    assert len(results) == 1
    assert results[0]["content"] == "Memory 1"


@pytest.mark.asyncio
async def test_memory_search():
    """Test searching memories."""
    memory = Memory()
    
    # Add memories
    await memory.add(
        "User prefers Python programming",
        agent_id="assistant-1",
        user_id="user-123"
    )
    await memory.add(
        "User likes JavaScript for web development",
        agent_id="assistant-1",
        user_id="user-123"
    )
    await memory.add(
        "User enjoys cooking Italian food",
        agent_id="assistant-1",
        user_id="user-123"
    )
    
    # Search for programming-related memories
    results = await memory.search(
        query="programming language",
        agent_id="assistant-1",
        user_id="user-123"
    )
    
    assert len(results) > 0
    # The Python memory should have higher score due to word overlap
    assert "Python" in results[0]["content"] or "JavaScript" in results[0]["content"]


@pytest.mark.asyncio
async def test_memory_update():
    """Test updating a memory."""
    memory = Memory()
    
    # Add a memory
    result = await memory.add(
        "Original content",
        agent_id="assistant-1",
        importance=0.5
    )
    memory_id = result["id"]
    
    # Update the memory
    updated = await memory.update(
        memory_id,
        content="Updated content",
        importance=0.9
    )
    
    assert updated["content"] == "Updated content"
    assert updated["importance"] == 0.9
    
    # Verify the update persisted
    retrieved = await memory.get(memory_id)
    assert retrieved["content"] == "Updated content"
    assert retrieved["importance"] == 0.9


@pytest.mark.asyncio
async def test_memory_delete():
    """Test deleting a memory."""
    memory = Memory()
    
    # Add a memory
    result = await memory.add("To be deleted", agent_id="assistant-1")
    memory_id = result["id"]
    
    # Delete the memory
    delete_result = await memory.delete(memory_id)
    assert delete_result["status"] == "success"
    
    # Verify it's deleted
    retrieved = await memory.get(memory_id)
    assert retrieved is None


@pytest.mark.asyncio
async def test_memory_clear():
    """Test clearing memories."""
    memory = Memory()
    
    # Add multiple memories
    await memory.add("Memory 1", agent_id="agent-1", user_id="user-1")
    await memory.add("Memory 2", agent_id="agent-1", user_id="user-2")
    await memory.add("Memory 3", agent_id="agent-2", user_id="user-1")
    
    # Clear memories for agent-1
    result = await memory.clear(agent_id="agent-1")
    assert result["status"] == "success"
    assert result["count"] == 2
    
    # Verify only agent-2 memories remain
    all_memories = await memory.get_all()
    assert len(all_memories) == 1
    assert all_memories[0]["agent_id"] == "agent-2"


@pytest.mark.asyncio
async def test_memory_with_metadata():
    """Test memories with metadata."""
    memory = Memory()
    
    # Add memory with metadata
    result = await memory.add(
        "User's favorite color is blue",
        agent_id="assistant-1",
        user_id="user-123",
        metadata={"category": "preference", "confidence": 0.95}
    )
    memory_id = result["id"]
    
    # Retrieve and verify metadata
    retrieved = await memory.get(memory_id)
    assert retrieved["metadata"]["category"] == "preference"
    assert retrieved["metadata"]["confidence"] == 0.95


@pytest.mark.asyncio
async def test_memory_importance_scoring():
    """Test importance scoring."""
    memory = Memory()
    
    # Add memories with different importance
    await memory.add("Low importance", importance=0.2)
    await memory.add("Medium importance", importance=0.5)
    await memory.add("High importance", importance=0.9)
    
    # Get all memories
    all_memories = await memory.get_all()
    assert len(all_memories) == 3
    
    # Verify importance values
    importances = [m["importance"] for m in all_memories]
    assert 0.2 in importances
    assert 0.5 in importances
    assert 0.9 in importances


@pytest.mark.asyncio
async def test_memory_search_with_threshold():
    """Test search with similarity threshold."""
    memory = Memory()
    
    # Add memories
    await memory.add("Python programming language", agent_id="agent-1")
    await memory.add("JavaScript web development", agent_id="agent-1")
    await memory.add("Cooking recipes", agent_id="agent-1")
    
    # Search with threshold
    results = await memory.search(
        query="programming",
        agent_id="agent-1",
        threshold=0.1  # Only return results with score >= 0.1
    )
    
    # Should return programming-related memories
    assert len(results) > 0
    for result in results:
        assert result["score"] >= 0.1


@pytest.mark.asyncio
async def test_memory_search_limit():
    """Test search result limit."""
    memory = Memory()
    
    # Add many memories
    for i in range(10):
        await memory.add(f"Memory {i}", agent_id="agent-1")
    
    # Search with limit
    results = await memory.search(
        query="Memory",
        agent_id="agent-1",
        limit=5
    )
    
    assert len(results) == 5


@pytest.mark.asyncio
async def test_memory_types():
    """Test different memory types."""
    memory = Memory()
    
    # Add different types of memories
    await memory.add("User said hello", memory_type="episodic")
    await memory.add("User prefers dark mode", memory_type="semantic")
    await memory.add("How to reset password", memory_type="procedural")
    
    # Filter by memory type
    episodic = await memory.get_all(memory_type="episodic")
    assert len(episodic) == 1
    assert episodic[0]["memory_type"] == "episodic"
    
    semantic = await memory.get_all(memory_type="semantic")
    assert len(semantic) == 1
    assert semantic[0]["memory_type"] == "semantic"


if __name__ == "__main__":
    # Run tests
    asyncio.run(test_memory_add())
    asyncio.run(test_memory_get())
    asyncio.run(test_memory_get_all())
    asyncio.run(test_memory_search())
    asyncio.run(test_memory_update())
    asyncio.run(test_memory_delete())
    asyncio.run(test_memory_clear())
    asyncio.run(test_memory_with_metadata())
    asyncio.run(test_memory_importance_scoring())
    asyncio.run(test_memory_search_with_threshold())
    asyncio.run(test_memory_search_limit())
    asyncio.run(test_memory_types())
    print("All tests passed!")

