"""
Basic Usage Example for LlamaIndex-AgentMem Integration

This example demonstrates the basic functionality of the AgentMem integration
with LlamaIndex for memory management.
"""

import asyncio
from llama_index.core import Document

from llamaindex_agentmem import AgentMemory
from agentmem import Config


async def basic_memory_example():
    """Basic memory operations."""
    # Initialize AgentMemory with configuration
    config = Config(
        api_base_url="http://localhost:8080",
        api_key="your-api-key",  # Optional if not using authentication
    )

    memory = AgentMemory(
        config=config,
        agent_id="example_agent",
        user_id="example_user",
        memory_type="semantic",
    )

    # Create documents
    documents = [
        Document(text="Python is a high-level programming language."),
        Document(text="LlamaIndex is a data framework for LLM applications."),
        Document(text="AgentMem provides enterprise-grade memory management."),
        Document(text="Vector databases enable semantic search capabilities."),
        Document(text="Embeddings convert text to numerical representations."),
    ]

    # Add documents to memory
    print("Adding documents to memory...")
    await memory._async_put(documents, importance=0.8)
    print(f"Added {len(documents)} documents")

    # Search for relevant documents
    print("\nSearching for 'programming language'...")
    results = await memory._async_get("programming language", limit=3)

    for i, doc in enumerate(results, 1):
        print(f"\n{i}. {doc.text}")
        print(f"   Score: {doc.metadata.get('score', 'N/A')}")
        print(f"   Importance: {doc.metadata.get('importance', 'N/A')}")

    # Get all documents
    print("\n\nRetrieving all documents...")
    all_docs = await memory._async_get_all(limit=10)
    print(f"Total documents: {len(all_docs)}")

    # Update a document
    if results:
        print("\n\nUpdating first result...")
        first_doc = results[0]
        first_doc.text = "Python is a versatile high-level programming language."
        await memory._async_add_documents([first_doc], importance=0.9)
        print("Document updated")

    # Clear memory
    print("\n\nClearing memory...")
    await memory._async_clear()
    print("Memory cleared")


async def memory_with_metadata_example():
    """Example using metadata for filtering."""
    config = Config(api_base_url="http://localhost:8080")

    memory = AgentMemory(config=config, agent_id="metadata_agent")

    # Create documents with metadata
    documents = [
        Document(
            text="Machine learning models require training data.",
            metadata={"category": "ML", "difficulty": "advanced"},
        ),
        Document(
            text="Python lists are mutable sequences.",
            metadata={"category": "Python", "difficulty": "beginner"},
        ),
        Document(
            text="Neural networks learn patterns from data.",
            metadata={"category": "ML", "difficulty": "intermediate"},
        ),
        Document(
            text="Python functions are reusable code blocks.",
            metadata={"category": "Python", "difficulty": "beginner"},
        ),
    ]

    # Add documents
    print("Adding documents with metadata...")
    await memory._async_put(documents)

    # Search with metadata filter
    print("\nSearching for ML content...")
    ml_results = await memory._async_get(
        "models and networks",
        filters={"category": "ML"},
        limit=5,
    )

    print(f"Found {len(ml_results)} ML-related documents:")
    for doc in ml_results:
        print(f"- {doc.text}")
        print(f"  Category: {doc.metadata.get('category')}")
        print(f"  Difficulty: {doc.metadata.get('difficulty')}")

    # Cleanup
    await memory._async_clear()


async def multi_agent_example():
    """Example with multiple agents."""
    config = Config(api_base_url="http://localhost:8080")

    # Create separate memory for each agent
    researcher_memory = AgentMemory(
        config=config,
        agent_id="researcher",
        memory_type="semantic",
    )

    assistant_memory = AgentMemory(
        config=config,
        agent_id="assistant",
        memory_type="semantic",
    )

    # Researcher stores research papers
    research_docs = [
        Document(text="Attention mechanisms revolutionized NLP."),
        Document(text="Transformers enable parallel processing."),
    ]
    await researcher_memory._async_put(research_docs)

    # Assistant stores user interactions
    assistant_docs = [
        Document(text="User asked about attention mechanisms."),
        Document(text="User requested explanation of transformers."),
    ]
    await assistant_memory._async_put(assistant_docs)

    # Each agent has isolated memory
    print("Researcher memory:")
    researcher_results = await researcher_memory._async_get("attention")
    print(f"  Found {len(researcher_results)} documents")

    print("\nAssistant memory:")
    assistant_results = await assistant_memory._async_get("attention")
    print(f"  Found {len(assistant_results)} documents")

    # Cleanup
    await researcher_memory._async_clear()
    await assistant_memory._async_clear()


async def main():
    """Run all examples."""
    print("=" * 60)
    print("Basic Memory Example")
    print("=" * 60)
    await basic_memory_example()

    print("\n\n" + "=" * 60)
    print("Memory with Metadata Example")
    print("=" * 60)
    await memory_with_metadata_example()

    print("\n\n" + "=" * 60)
    print("Multi-Agent Example")
    print("=" * 60)
    await multi_agent_example()

    print("\n\n" + "=" * 60)
    print("All examples completed!")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())
