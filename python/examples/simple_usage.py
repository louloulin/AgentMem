#!/usr/bin/env python3
"""
Simple AgentMem Usage Example

This example demonstrates the simplest way to use AgentMem,
similar to Mem0's API.
"""

import asyncio
import sys
from pathlib import Path

# Add parent directory to path to import agentmem
sys.path.insert(0, str(Path(__file__).parent.parent))

from agentmem import Memory

async def main():
    # Initialize memory with default settings (embedded mode)
    memory = Memory()
    
    # Add memories
    print("Adding memories...")
    
    # Add a simple memory
    result1 = await memory.add(
        "User prefers Python over JavaScript",
        agent_id="assistant-1",
        user_id="user-123"
    )
    print(f"Added memory: {result1['id']}")
    
    # Add another memory
    result2 = await memory.add(
        "User is working on an AI project using LLMs",
        agent_id="assistant-1",
        user_id="user-123"
    )
    print(f"Added memory: {result2['id']}")
    
    # Search memories
    print("\nSearching memories...")
    results = await memory.search(
        query="What programming language does the user prefer?",
        agent_id="assistant-1",
        user_id="user-123"
    )
    
    for result in results:
        print(f"- {result['content']} (score: {result['score']:.2f})")
    
    # Get all memories for a user
    print("\nGetting all memories...")
    all_memories = await memory.get_all(
        agent_id="assistant-1",
        user_id="user-123"
    )
    
    for mem in all_memories:
        print(f"- {mem['content']}")
    
    # Update a memory
    print("\nUpdating memory...")
    await memory.update(
        result1['id'],
        content="User strongly prefers Python over JavaScript for AI projects"
    )
    print("Memory updated!")
    
    # Delete a memory
    print("\nDeleting memory...")
    await memory.delete(result2['id'])
    print("Memory deleted!")
    
    # Final check
    print("\nFinal memories:")
    final_memories = await memory.get_all(
        agent_id="assistant-1",
        user_id="user-123"
    )
    
    for mem in final_memories:
        print(f"- {mem['content']}")

if __name__ == "__main__":
    asyncio.run(main())

