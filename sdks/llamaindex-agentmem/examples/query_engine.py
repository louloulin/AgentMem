"""
Query Engine Integration Example

This example demonstrates how to integrate AgentMem with LlamaIndex's
QueryEngine for advanced question-answering capabilities.
"""

import asyncio
from typing import List

from llama_index.core import VectorStoreIndex, Document, Settings
from llama_index.core.query_engine import QueryEngine
from llama_index.core.response.schema import Response

from llamaindex_agentmem import AgentMemory
from agentmem import Config


async def create_query_engine_with_agentmem():
    """Create a QueryEngine backed by AgentMem."""
    # Initialize AgentMemory
    config = Config(api_base_url="http://localhost:8080")
    memory = AgentMemory(
        config=config,
        agent_id="qa_agent",
        top_k=5,  # Retrieve top 5 relevant documents
    )

    # Create sample documents
    documents = [
        Document(
            text=(
                "AgentMem is a distributed memory system designed for AI agents. "
                "It provides semantic memory, episodic memory, and procedural memory types. "
                "The system uses vector embeddings for efficient semantic search."
            ),
            metadata={"source": "docs", "topic": "overview"},
        ),
        Document(
            text=(
                "Semantic memory stores general knowledge and facts. "
                "It is ideal for storing domain-specific information, "
                "technical documentation, and reference material."
            ),
            metadata={"source": "docs", "topic": "semantic"},
        ),
        Document(
            text=(
                "Episodic memory records events and experiences. "
                "It captures temporal context and is useful for "
                "conversation history and user interactions."
            ),
            metadata={"source": "docs", "topic": "episodic"},
        ),
        Document(
            text=(
                "Procedural memory stores skills, workflows, and processes. "
                "It enables agents to remember how to perform tasks "
                "and execute complex procedures."
            ),
            metadata={"source": "docs", "topic": "procedural"},
        ),
        Document(
            text=(
                "AgentMem supports importance-based memory ranking from 0.0 to 1.0. "
                "Higher importance memories are prioritized during retrieval. "
                "Importance can be manually set or automatically calculated."
            ),
            metadata={"source": "docs", "topic": "features"},
        ),
        Document(
            text=(
                "The AgentMem Python SDK provides async and sync APIs. "
                "It supports batch operations, metadata filtering, "
                "and comprehensive error handling."
            ),
            metadata={"source": "docs", "topic": "sdk"},
        ),
    ]

    # Add documents to memory
    print("Loading documents into AgentMem...")
    await memory._async_put(documents)
    print(f"Loaded {len(documents)} documents\n")

    # Create index from retrieved documents
    def retrieve_and_query(query: str) -> Response:
        """Retrieve documents and create query engine."""
        # Retrieve relevant documents
        relevant_docs = memory.get(query, limit=3)

        if not relevant_docs:
            return Response(response="No relevant information found.")

        # Create index from retrieved documents
        index = VectorStoreIndex.from_documents(relevant_docs)

        # Create query engine
        query_engine = index.as_query_engine(
            similarity_threshold=0.7,
            verbose=True,
        )

        # Execute query
        response = query_engine.query(query)
        return response

    # Example queries
    queries = [
        "What types of memory does AgentMem support?",
        "How does importance-based ranking work?",
        "What is semantic memory used for?",
        "Tell me about the Python SDK features",
    ]

    print("=" * 70)
    print("Query Engine Example")
    print("=" * 70)

    for query in queries:
        print(f"\n{'=' * 70}")
        print(f"Query: {query}")
        print(f"{'=' * 70}")

        response = retrieve_and_query(query)
        print(f"\nAnswer:\n{response}")

    # Cleanup
    print("\n\nCleaning up...")
    await memory._async_clear()


async def query_with_metadata_filter():
    """Example of querying with metadata filters."""
    config = Config(api_base_url="http://localhost:8080")
    memory = AgentMemory(config=config, agent_id="filter_agent")

    # Add documents with different metadata
    documents = [
        Document(
            text="Python 3.10 introduced pattern matching with match statements.",
            metadata={"language": "Python", "version": "3.10", "category": "features"},
        ),
        Document(
            text="Python 3.11 added improved error messages and performance.",
            metadata={"language": "Python", "version": "3.11", "category": "features"},
        ),
        Document(
            text="Python 3.12 includes better type hints and syntax improvements.",
            metadata={"language": "Python", "version": "3.12", "category": "features"},
        ),
        Document(
            text="JavaScript ES2023 added array sorting stability.",
            metadata={"language": "JavaScript", "version": "ES2023", "category": "features"},
        ),
        Document(
            text="TypeScript 5.0 introduced decorator support.",
            metadata={"language": "TypeScript", "version": "5.0", "category": "features"},
        ),
    ]

    await memory._async_put(documents)

    print("=" * 70)
    print("Metadata Filter Example")
    print("=" * 70)

    # Query with metadata filter
    query = "new features and improvements"
    filters = {"language": "Python", "category": "features"}

    print(f"\nQuery: {query}")
    print(f"Filters: {filters}")
    print(f"{'-' * 70}")

    # Retrieve filtered results
    results = await memory._async_get(query, filters=filters, limit=5)

    print(f"\nFound {len(results)} Python-related results:\n")
    for i, doc in enumerate(results, 1):
        print(f"{i}. {doc.text}")
        print(f"   Metadata: {doc.metadata}")
        print()

    # Create query engine from filtered results
    if results:
        index = VectorStoreIndex.from_documents(results)
        query_engine = index.as_query_engine()

        response = query_engine.query("Summarize the Python features mentioned")
        print(f"\nSummary:\n{response}")

    # Cleanup
    await memory._async_clear()


async def streaming_query_example():
    """Example of streaming query responses."""
    config = Config(api_base_url="http://localhost:8080")
    memory = AgentMemory(config=config, agent_id="streaming_agent")

    # Add technical documentation
    documents = [
        Document(
            text=(
                "LlamaIndex is a data framework for building LLM applications. "
                "It provides connectors to load data from various sources, "
                "indices to structure data, and engines to query data efficiently."
            ),
            metadata={"topic": "llamaindex", "type": "overview"},
        ),
        Document(
            text=(
                "VectorStoreIndex is the most common index type in LlamaIndex. "
                "It uses embeddings to represent documents and enables "
                "semantic search through similarity scoring."
            ),
            metadata={"topic": "llamaindex", "type": "indices"},
        ),
        Document(
            text=(
                "Query engines in LlamaIndex retrieve relevant context and "
                "synthesize responses using LLMs. Different query engines "
                "support various modes like retrieval, synthesis, and refinement."
            ),
            metadata={"topic": "llamaindex", "type": "querying"},
        ),
    ]

    await memory._async_put(documents)

    print("=" * 70)
    print("Streaming Query Example")
    print("=" * 70)

    query = "How does LlamaIndex handle data querying?"

    # Retrieve relevant documents
    results = await memory._async_get(query, limit=3)

    if results:
        # Create streaming query engine
        index = VectorStoreIndex.from_documents(results)
        query_engine = index.as_query_engine(streaming=True)

        print(f"\nQuery: {query}\n")
        print("Response (streaming):\n")

        # Stream response
        streaming_response = query_engine.query(query)
        for chunk in streaming_response.response_gen:
            print(chunk, end="", flush=True)
        print("\n")

    # Cleanup
    await memory._async_clear()


async def main():
    """Run all query engine examples."""
    await create_query_engine_with_agentmem()

    print("\n\n" + "=" * 70)
    print("NEXT EXAMPLE: Metadata Filtering")
    print("=" * 70)
    await asyncio.sleep(1)

    await query_with_metadata_filter()

    print("\n\n" + "=" * 70)
    print("NEXT EXAMPLE: Streaming Queries")
    print("=" * 70)
    await asyncio.sleep(1)

    await streaming_query_example()

    print("\n\n" + "=" * 70)
    print("All query engine examples completed!")
    print("=" * 70)


if __name__ == "__main__":
    asyncio.run(main())
