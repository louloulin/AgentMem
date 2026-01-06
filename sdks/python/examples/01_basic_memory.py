"""
Example 1: Basic Memory Management

This example demonstrates basic memory operations:
- Adding memories
- Retrieving memories
- Searching memories
- Updating memories
- Deleting memories
"""

import asyncio
import os
from agentmem import AgentMemClient, Config, MemoryType, SearchQuery


async def main():
    """Main example function."""
    print("🚀 AgentMem Python SDK - Basic Memory Management Example\n")
    
    # Initialize client
    config = Config(
        api_key=os.getenv("AGENTMEM_API_KEY", "demo-api-key"),
        api_base_url=os.getenv("AGENTMEM_BASE_URL", "http://localhost:8080"),
        timeout=30,
        enable_caching=True,
    )
    
    async with AgentMemClient(config) as client:
        agent_id = "example_agent_1"
        
        # 1. Add memories
        print("1️⃣  Adding memories...")
        
        memory_id_1 = await client.add_memory(
            content="The user prefers dark mode in the UI",
            agent_id=agent_id,
            memory_type=MemoryType.SEMANTIC,
            importance=0.8,
            metadata={"category": "user_preferences", "ui": "theme"}
        )
        print(f"   ✅ Added semantic memory: {memory_id_1}")
        
        memory_id_2 = await client.add_memory(
            content="User logged in at 2024-01-15 10:30:00",
            agent_id=agent_id,
            memory_type=MemoryType.EPISODIC,
            importance=0.5,
            metadata={"category": "user_activity", "action": "login"}
        )
        print(f"   ✅ Added episodic memory: {memory_id_2}")
        
        memory_id_3 = await client.add_memory(
            content="To reset password, click 'Forgot Password' and follow the email instructions",
            agent_id=agent_id,
            memory_type=MemoryType.PROCEDURAL,
            importance=0.9,
            metadata={"category": "procedures", "topic": "authentication"}
        )
        print(f"   ✅ Added procedural memory: {memory_id_3}\n")
        
        # 2. Retrieve a memory
        print("2️⃣  Retrieving memory...")
        memory = await client.get_memory(memory_id_1)
        print(f"   📝 Content: {memory.content}")
        print(f"   🏷️  Type: {memory.memory_type.value}")
        print(f"   ⭐ Importance: {memory.importance}")
        print(f"   📊 Metadata: {memory.metadata}\n")
        
        # 3. Search memories
        print("3️⃣  Searching memories...")
        
        # Text search
        results = await client.search_memories(
            SearchQuery(
                agent_id=agent_id,
                text_query="user preferences",
                limit=5
            )
        )
        print(f"   🔍 Found {len(results)} results for 'user preferences':")
        for i, result in enumerate(results, 1):
            print(f"      {i}. {result.memory.content[:50]}... (score: {result.score:.3f})")
        print()
        
        # Search with filters
        results = await client.search_memories(
            SearchQuery(
                agent_id=agent_id,
                memory_type=MemoryType.SEMANTIC,
                min_importance=0.7,
                limit=10
            )
        )
        print(f"   🔍 Found {len(results)} semantic memories with importance >= 0.7\n")
        
        # 4. Update a memory
        print("4️⃣  Updating memory...")
        updated_memory = await client.update_memory(
            memory_id=memory_id_1,
            importance=0.95,
            metadata={"category": "user_preferences", "ui": "theme", "updated": True}
        )
        print(f"   ✅ Updated memory importance to {updated_memory.importance}\n")
        
        # 5. Batch add memories
        print("5️⃣  Batch adding memories...")
        batch_memories = [
            {
                "content": f"User viewed page {i}",
                "agent_id": agent_id,
                "memory_type": "episodic",
                "importance": 0.3
            }
            for i in range(1, 6)
        ]
        batch_ids = await client.batch_add_memories(batch_memories)
        print(f"   ✅ Added {len(batch_ids)} memories in batch\n")
        
        # 6. Get statistics
        print("6️⃣  Getting memory statistics...")
        stats = await client.get_memory_stats(agent_id)
        print(f"   📊 Total memories: {stats.total_memories}")
        print(f"   📊 Memories by type:")
        for mem_type, count in stats.memories_by_type.items():
            print(f"      - {mem_type}: {count}")
        print(f"   📊 Average importance: {stats.average_importance:.2f}\n")
        
        # 7. Delete a memory
        print("7️⃣  Deleting memory...")
        success = await client.delete_memory(memory_id_2)
        if success:
            print(f"   ✅ Deleted memory: {memory_id_2}\n")
        
        # 8. Health check
        print("8️⃣  Checking API health...")
        health = await client.health_check()
        print(f"   ✅ API Status: {health.get('status', 'unknown')}\n")
    
    print("✨ Example completed successfully!")


if __name__ == "__main__":
    asyncio.run(main())

