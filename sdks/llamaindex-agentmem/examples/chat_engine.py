"""
Chat Engine Integration Example

This example demonstrates how to integrate AgentMem with LlamaIndex's
ChatEngine for conversational AI applications with persistent memory.
"""

import asyncio
from typing import List, Dict, Any

from llama_index.core import VectorStoreIndex, Document
from llama_index.core.chat_engine import ChatEngine
from llama_index.core.memory import ChatMemoryBuffer

from llamaindex_agentmem import AgentMemory
from agentmem import Config


async def basic_chat_example():
    """Basic chat engine with AgentMem for context storage."""
    config = Config(api_base_url="http://localhost:8080")

    # Create memory for chat history
    memory = AgentMemory(
        config=config,
        agent_id="chat_agent",
        user_id="user_123",
        memory_type="episodic",
    )

    # Create system context documents
    context_docs = [
        Document(
            text=(
                "You are a helpful AI assistant specializing in Python programming. "
                "You provide clear explanations and code examples."
            ),
            metadata={"type": "system_prompt"},
        ),
        Document(
            text=(
                "Python is a high-level, interpreted programming language known for "
                "its simplicity and readability. It supports multiple programming paradigms."
            ),
            metadata={"type": "knowledge"},
        ),
    ]

    await memory._async_put(context_docs)

    # Simulate chat conversation
    conversations = [
        "What is Python?",
        "Can you show me a simple example?",
        "What about list comprehensions?",
    ]

    print("=" * 70)
    print("Basic Chat Example")
    print("=" * 70)

    for user_message in conversations:
        print(f"\nUser: {user_message}")

        # Retrieve relevant context from memory
        relevant_context = await memory._async_get(user_message, limit=3)

        # Build context string
        context_str = "\n".join([doc.text for doc in relevant_context])

        # Simulate assistant response (in real scenario, use LLM)
        if "Python" in user_message:
            assistant_response = (
                f"{context_str}\n\n"
                "Based on the context, Python is a versatile programming language "
                "widely used in web development, data science, and automation."
            )
        elif "example" in user_message:
            assistant_response = (
                "Here's a simple Python example:\n\n"
                "```python\n"
                "# Hello World\n"
                "print('Hello, World!')\n"
                "\n"
                "# Simple function\n"
                "def greet(name):\n"
                "    return f'Hello, {name}!'\n"
                "```"
            )
        else:
            assistant_response = (
                "List comprehensions provide a concise way to create lists:\n\n"
                "```python\n"
                "# Traditional way\n"
                "squares = []\n"
                "for x in range(10):\n"
                "    squares.append(x**2)\n"
                "\n"
                "# List comprehension\n"
                "squares = [x**2 for x in range(10)]\n"
                "```"
            )

        print(f"Assistant: {assistant_response}")

        # Store conversation in memory
        conversation_doc = Document(
            text=f"User: {user_message}\nAssistant: {assistant_response}",
            metadata={"type": "conversation", "turn": len(conversations)},
        )
        await memory._async_put([conversation_doc], importance=0.8)

    # Cleanup
    await memory._async_clear()


async def multi_turn_chat_with_memory():
    """Multi-turn chat with persistent memory across sessions."""
    config = Config(api_base_url="http://localhost:8080")

    memory = AgentMemory(
        config=config,
        agent_id="multi_turn_agent",
        user_id="user_456",
        memory_type="episodic",
    )

    print("=" * 70)
    print("Multi-Turn Chat with Memory")
    print("=" * 70)

    # Session 1: Initial conversation
    print("\n--- Session 1 ---\n")

    session1_turns = [
        ("My name is Alice", "Nice to meet you, Alice!"),
        ("I'm learning Python", "That's great! Python is beginner-friendly."),
    ]

    for user_msg, assistant_msg in session1_turns:
        print(f"User: {user_msg}")
        print(f"Assistant: {assistant_msg}")

        # Store in memory
        await memory._async_put(
            [
                Document(
                    text=f"User said: {user_msg}",
                    metadata={"type": "user_message", "session": 1},
                )
            ]
        )

    # Session 2: Resume conversation (memory persists)
    print("\n--- Session 2 (Resumed) ---\n")

    session2_turns = [
        "What's my name?",
        "What should I learn first in Python?",
    ]

    for user_msg in session2_turns:
        print(f"User: {user_msg}")

        # Retrieve relevant context from previous sessions
        context = await memory._async_get(user_msg, limit=3)

        # Build response using context
        if "name" in user_msg and context:
            response = "Based on our conversation, your name is Alice!"
        elif "learn" in user_msg:
            response = (
                "For starting with Python, I recommend:\n"
                "1. Variables and data types\n"
                "2. Control flow (if/else, loops)\n"
                "3. Functions\n"
                "4. Data structures (lists, dictionaries)\n"
                "5. File handling"
            )
        else:
            response = "Let me check our conversation history..."

        print(f"Assistant: {response}")

        # Store new interaction
        await memory._async_put(
            [
                Document(
                    text=f"User asked: {user_msg}",
                    metadata={"type": "user_message", "session": 2},
                )
            ]
        )

    # Show all stored memories
    print("\n--- Conversation History ---\n")
    all_memories = await memory._async_get_all(limit=20)
    for i, mem in enumerate(all_memories, 1):
        print(f"{i}. {mem.text[:80]}...")

    # Cleanup
    await memory._async_clear()


async def chat_with_knowledge_base():
    """Chat engine backed by knowledge base stored in AgentMem."""
    config = Config(api_base_url="http://localhost:8080")

    # Knowledge base memory
    kb_memory = AgentMemory(
        config=config,
        agent_id="knowledge_base",
        memory_type="semantic",
    )

    # Chat history memory
    chat_memory = AgentMemory(
        config=config,
        agent_id="chat_history",
        user_id="user_789",
        memory_type="episodic",
    )

    # Load knowledge base documents
    kb_docs = [
        Document(
            text="AgentMem supports three memory types: semantic, episodic, and procedural.",
            metadata={"category": "memory_types"},
        ),
        Document(
            text="Semantic memory stores facts and general knowledge.",
            metadata={"category": "semantic_memory"},
        ),
        Document(
            text="Episodic memory records events and experiences with temporal context.",
            metadata={"category": "episodic_memory"},
        ),
        Document(
            text="Procedural memory stores skills and workflows.",
            metadata={"category": "procedural_memory"},
        ),
        Document(
            text="AgentMem provides Python SDK with async and sync APIs.",
            metadata={"category": "sdk"},
        ),
    ]

    await kb_memory._async_put(kb_docs)

    print("=" * 70)
    print("Chat with Knowledge Base")
    print("=" * 70)

    questions = [
        "What memory types are available?",
        "Tell me about episodic memory",
        "How do I use the Python SDK?",
    ]

    for question in questions:
        print(f"\nUser: {question}")

        # Retrieve relevant knowledge
        relevant_kb = await kb_memory._async_get(question, limit=2)

        # Retrieve chat history for context
        relevant_history = await chat_memory._async_get(question, limit=2)

        # Build response
        if relevant_kb:
            kb_text = "\n".join([doc.text for doc in relevant_kb])
            response = f"Based on my knowledge:\n{kb_text}\n"
        else:
            response = "I don't have specific information about that."

        print(f"Assistant: {response}")

        # Store Q&A in chat history
        qa_doc = Document(
            text=f"Q: {question}\nA: {response}",
            metadata={"type": "qa"},
        )
        await chat_memory._async_put([qa_doc])

    # Show statistics
    print("\n--- Memory Statistics ---\n")
    kb_count = len(await kb_memory._async_get_all())
    chat_count = len(await chat_memory._async_get_all())
    print(f"Knowledge Base Documents: {kb_count}")
    print(f"Chat History Entries: {chat_count}")

    # Cleanup
    await kb_memory._async_clear()
    await chat_memory._async_clear()


async def contextual_chat_example():
    """Chat with dynamic context retrieval based on conversation."""
    config = Config(api_base_url="http://localhost:8080")

    memory = AgentMemory(
        config=config,
        agent_id="contextual_agent",
        user_id="user_999",
        memory_type="semantic",
    )

    # Store user profile and preferences
    profile_docs = [
        Document(
            text="User is an experienced Python developer interested in machine learning.",
            metadata={"type": "user_profile", "domain": "expertise"},
        ),
        Document(
            text="User prefers concise, technical explanations over beginner-friendly ones.",
            metadata={"type": "user_preference", "style": "technical"},
        ),
        Document(
            text="User is currently working on a computer vision project.",
            metadata={"type": "current_project", "domain": "cv"},
        ),
    ]

    await memory._async_put(profile_docs)

    print("=" * 70)
    print("Contextual Chat with User Profile")
    print("=" * 70)

    # User asks a question
    user_question = "How should I preprocess images for deep learning?"

    print(f"\nUser: {user_question}")

    # Retrieve relevant user profile and context
    profile_context = await memory._async_get(
        "user profile expertise project",
        filters={"type": "user_profile"},
        limit=3,
    )

    # Retrieve relevant technical knowledge
    tech_context = await memory._async_get(user_question, limit=3)

    # Build contextual response
    response = f"""Based on your profile as an experienced Python developer working on computer vision:

For image preprocessing in deep learning, I recommend:

1. **Resize images**: Standardize dimensions (e.g., 224x224 for ResNet)
2. **Normalization**: Scale pixel values to [0, 1] or use mean/std normalization
3. **Data augmentation**: Apply rotations, flips, and color jittering
4. **Tensor conversion**: Use PyTorch transforms or TensorFlow preprocessing

Example with PyTorch:
```python
from torchvision import transforms

transform = transforms.Compose([
    transforms.Resize(256),
    transforms.CenterCrop(224),
    transforms.ToTensor(),
    transforms.Normalize(mean=[0.485, 0.456, 0.406],
                       std=[0.229, 0.224, 0.225])
])
```

Given your expertise, you might also want to explore advanced techniques like MixUp or CutMix for improved generalization."""

    print(f"\nAssistant:\n{response}")

    # Store interaction
    interaction = Document(
        text=f"Q: {user_question}\n\nA: {response}",
        metadata={
            "type": "interaction",
            "context_used": len(profile_context) + len(tech_context),
        },
    )
    await memory._async_put([interaction])

    # Cleanup
    await memory._async_clear()


async def main():
    """Run all chat engine examples."""
    await basic_chat_example()

    print("\n\n" + "=" * 70)
    print("NEXT EXAMPLE: Multi-Turn Chat")
    print("=" * 70)
    await asyncio.sleep(1)

    await multi_turn_chat_with_memory()

    print("\n\n" + "=" * 70)
    print("NEXT EXAMPLE: Knowledge Base Chat")
    print("=" * 70)
    await asyncio.sleep(1)

    await chat_with_knowledge_base()

    print("\n\n" + "=" * 70)
    print("NEXT EXAMPLE: Contextual Chat")
    print("=" * 70)
    await asyncio.sleep(1)

    await contextual_chat_example()

    print("\n\n" + "=" * 70)
    print("All chat engine examples completed!")
    print("=" * 70)


if __name__ == "__main__":
    asyncio.run(main())
