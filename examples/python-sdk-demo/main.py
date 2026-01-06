#!/usr/bin/env python3
"""
AgentMem Python SDK Demo

This demo showcases the AgentMem Python SDK capabilities:
1. Client initialization and configuration
2. Memory management (CRUD operations)
3. Advanced search functionality
4. Batch operations
5. Statistics and monitoring
6. Error handling
"""

import asyncio
import json
import os
import sys
from typing import List, Dict, Any

# Add the SDK to the path (for demo purposes)
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../sdks/python'))

from agentmem import (
    AgentMemClient,
    Config,
    MemoryType,
    SearchQuery,
    AgentMemError,
    AuthenticationError,
    ValidationError,
    NetworkError,
)


class PythonSDKDemo:
    """Demo class showcasing AgentMem Python SDK functionality."""
    
    def __init__(self):
        """Initialize the demo with a mock client configuration."""
        # For demo purposes, we'll use a mock configuration
        # In real usage, you would set AGENTMEM_API_KEY environment variable
        self.config = Config(
            api_key="demo-api-key-12345",
            base_url="http://localhost:8080",  # Mock server for demo
            timeout=30,
            max_retries=3,
            enable_caching=True,
            cache_ttl=300,
            enable_logging=True,
        )
        
        self.client = AgentMemClient(self.config)
        self.demo_agent_id = "demo_agent_001"
        self.created_memory_ids: List[str] = []

    async def run_demo(self):
        """Run the complete SDK demo."""
        print("🧠 AgentMem Python SDK Demo")
        print("=" * 50)
        
        try:
            # Test 1: Basic memory operations
            await self.demo_basic_operations()
            
            # Test 2: Advanced search
            await self.demo_advanced_search()
            
            # Test 3: Batch operations
            await self.demo_batch_operations()
            
            # Test 4: Statistics and monitoring
            await self.demo_statistics()
            
            # Test 5: Error handling
            await self.demo_error_handling()
            
            print("\n✅ All demo tests completed successfully!")
            
        except Exception as e:
            print(f"\n❌ Demo failed with error: {e}")
            if self.config.enable_logging:
                import traceback
                traceback.print_exc()
        
        finally:
            # Cleanup
            await self.cleanup()
            await self.client.close()

    async def demo_basic_operations(self):
        """Demonstrate basic CRUD operations."""
        print("\n📝 Demo 1: Basic Memory Operations")
        print("-" * 30)
        
        # Add memories
        print("Adding memories...")
        
        memory_id_1 = await self.client.add_memory(
            content="The user prefers dark mode in the application",
            agent_id=self.demo_agent_id,
            memory_type=MemoryType.SEMANTIC,
            importance=0.8,
            metadata={"category": "user_preferences", "ui_theme": "dark"}
        )
        self.created_memory_ids.append(memory_id_1)
        print(f"✓ Added semantic memory: {memory_id_1}")
        
        memory_id_2 = await self.client.add_memory(
            content="User clicked the 'Export Data' button at 2024-01-15 14:30:00",
            agent_id=self.demo_agent_id,
            memory_type=MemoryType.EPISODIC,
            importance=0.6,
            metadata={"action": "export_data", "timestamp": "2024-01-15T14:30:00Z"}
        )
        self.created_memory_ids.append(memory_id_2)
        print(f"✓ Added episodic memory: {memory_id_2}")
        
        memory_id_3 = await self.client.add_memory(
            content="To export data: 1) Go to Settings, 2) Click Export, 3) Choose format",
            agent_id=self.demo_agent_id,
            memory_type=MemoryType.PROCEDURAL,
            importance=0.9,
            metadata={"procedure": "data_export", "steps": 3}
        )
        self.created_memory_ids.append(memory_id_3)
        print(f"✓ Added procedural memory: {memory_id_3}")
        
        # Retrieve memory
        print(f"\nRetrieving memory {memory_id_1}...")
        memory = await self.client.get_memory(memory_id_1)
        print(f"✓ Retrieved: {memory.content}")
        print(f"  Type: {memory.memory_type.value}")
        print(f"  Importance: {memory.importance}")
        print(f"  Metadata: {memory.metadata}")
        
        # Update memory
        print(f"\nUpdating memory {memory_id_1}...")
        updated_memory = await self.client.update_memory(
            memory_id_1,
            importance=0.9,
            metadata={"category": "user_preferences", "ui_theme": "dark", "updated": True}
        )
        print(f"✓ Updated importance to: {updated_memory.importance}")

    async def demo_advanced_search(self):
        """Demonstrate advanced search capabilities."""
        print("\n🔍 Demo 2: Advanced Search")
        print("-" * 25)
        
        # Text search
        print("Searching for 'user preferences'...")
        results = await self.client.search_memories(
            SearchQuery(
                agent_id=self.demo_agent_id,
                text_query="user preferences",
                limit=5
            )
        )
        
        print(f"✓ Found {len(results)} results:")
        for i, result in enumerate(results, 1):
            print(f"  {i}. {result.memory.content[:50]}... (score: {result.score:.3f})")
        
        # Search with filters
        print("\nSearching semantic memories with high importance...")
        results = await self.client.search_memories(
            SearchQuery(
                agent_id=self.demo_agent_id,
                text_query="preferences",
                memory_type=MemoryType.SEMANTIC,
                min_importance=0.7,
                limit=3
            )
        )
        
        print(f"✓ Found {len(results)} high-importance semantic memories")
        
        # Metadata search
        print("\nSearching by metadata...")
        results = await self.client.search_memories(
            SearchQuery(
                agent_id=self.demo_agent_id,
                metadata_filters={"category": "user_preferences"},
                limit=5
            )
        )
        
        print(f"✓ Found {len(results)} memories with user_preferences category")

    async def demo_batch_operations(self):
        """Demonstrate batch operations."""
        print("\n📦 Demo 3: Batch Operations")
        print("-" * 25)
        
        # Batch add memories
        print("Adding memories in batch...")
        batch_memories = [
            {
                "content": "User's favorite programming language is Python",
                "agent_id": self.demo_agent_id,
                "memory_type": "semantic",
                "importance": 0.7,
                "metadata": {"category": "preferences", "topic": "programming"}
            },
            {
                "content": "User completed Python tutorial on 2024-01-10",
                "agent_id": self.demo_agent_id,
                "memory_type": "episodic",
                "importance": 0.6,
                "metadata": {"achievement": "tutorial_completion", "language": "python"}
            },
            {
                "content": "User asked about machine learning libraries",
                "agent_id": self.demo_agent_id,
                "memory_type": "episodic",
                "importance": 0.5,
                "metadata": {"topic": "machine_learning", "question": True}
            }
        ]
        
        batch_ids = await self.client.batch_add_memories(batch_memories)
        self.created_memory_ids.extend(batch_ids)
        print(f"✓ Added {len(batch_ids)} memories in batch")
        
        for i, memory_id in enumerate(batch_ids, 1):
            print(f"  {i}. {memory_id}")

    async def demo_statistics(self):
        """Demonstrate statistics and monitoring."""
        print("\n📊 Demo 4: Statistics & Monitoring")
        print("-" * 32)
        
        # Get memory statistics
        print("Fetching memory statistics...")
        stats = await self.client.get_memory_stats(self.demo_agent_id)
        
        print(f"✓ Total memories: {stats.total_memories}")
        print(f"✓ Average importance: {stats.average_importance:.3f}")
        print(f"✓ Total access count: {stats.total_access_count}")
        
        print("\nMemories by type:")
        for memory_type, count in stats.memories_by_type.items():
            print(f"  {memory_type}: {count}")
        
        # Health check
        print("\nChecking API health...")
        try:
            health = await self.client.health_check()
            print(f"✓ API Status: {health.get('status', 'unknown')}")
            print(f"✓ Version: {health.get('version', 'unknown')}")
        except Exception as e:
            print(f"⚠️  Health check failed (expected in demo): {e}")
        
        # System metrics
        print("\nFetching system metrics...")
        try:
            metrics = await self.client.get_metrics()
            print(f"✓ Active connections: {metrics.get('active_connections', 'N/A')}")
            print(f"✓ Cache hit rate: {metrics.get('cache_hit_rate', 'N/A')}")
        except Exception as e:
            print(f"⚠️  Metrics fetch failed (expected in demo): {e}")

    async def demo_error_handling(self):
        """Demonstrate error handling."""
        print("\n⚠️  Demo 5: Error Handling")
        print("-" * 25)
        
        # Test invalid memory ID
        print("Testing invalid memory ID...")
        try:
            await self.client.get_memory("invalid-memory-id")
        except NotFoundError as e:
            print(f"✓ Caught NotFoundError: {e}")
        except NetworkError as e:
            print(f"✓ Caught NetworkError (expected in demo): {e}")
        except Exception as e:
            print(f"✓ Caught other error (expected in demo): {e}")
        
        # Test invalid search query
        print("\nTesting invalid search parameters...")
        try:
            await self.client.search_memories(
                SearchQuery(
                    agent_id="",  # Invalid empty agent ID
                    text_query="test",
                    limit=5
                )
            )
        except ValidationError as e:
            print(f"✓ Caught ValidationError: {e}")
        except Exception as e:
            print(f"✓ Caught other error (expected in demo): {e}")

    async def cleanup(self):
        """Clean up created memories."""
        print("\n🧹 Cleaning up demo data...")
        
        cleanup_count = 0
        for memory_id in self.created_memory_ids:
            try:
                await self.client.delete_memory(memory_id)
                cleanup_count += 1
            except Exception as e:
                print(f"⚠️  Failed to delete {memory_id}: {e}")
        
        print(f"✓ Cleaned up {cleanup_count} memories")


async def main():
    """Main demo function."""
    print("Starting AgentMem Python SDK Demo...")
    print("Note: This demo uses mock data and may show connection errors - this is expected!")
    print()
    
    demo = PythonSDKDemo()
    await demo.run_demo()


if __name__ == "__main__":
    asyncio.run(main())
