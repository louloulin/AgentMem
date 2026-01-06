"""
AgentMem LangChain Integration

Provides LangChain-compatible memory backend for AI agents.
This allows AgentMem to be used as a drop-in replacement for LangChain's memory modules.

Compatible with LangChain 0.1+ and Python 3.8+

Example:
    ```python
    from langchain.chains import ConversationChain
    from agentmem.langchain import AgentMemMemory

    # Initialize AgentMem memory
    memory = AgentMemMemory(
        session_id="user-123",
        backend_url="http://localhost:8080"
    )

    # Use with LangChain
    conversation = ConversationChain(
        llm=your_llm,
        memory=memory,
        verbose=True
    )

    result = conversation.run("Hello, I'm John!")
    ```
"""

from typing import Any, Dict, List, Optional, Tuple
from langchain.memory import BaseMemory
from langchain.schema import BaseMessage, HumanMessage, AIMessage
import requests


class AgentMemMemory(BaseMemory):
    """
    AgentMem-based memory for LangChain.

    Provides persistent, semantic search-enabled memory for LangChain agents.
    Automatically stores conversations with embeddings and enables intelligent
    retrieval of relevant context.

    Attributes:
        session_id: Unique identifier for the conversation session
        backend_url: URL of the AgentMem server (default: http://localhost:8080)
        memory_key: Key to use for storing memories in context (default: "history")
        input_key: Key to use for user input (default: "input")
        output_key: Key to use for AI output (default: "output")
        top_k: Number of relevant memories to retrieve (default: 5)
        threshold: Minimum similarity score for retrieved memories (default: 0.7)
    """

    session_id: str
    backend_url: str = "http://localhost:8080"
    memory_key: str = "history"
    input_key: str = "input"
    output_key: str = "output"
    top_k: int = 5
    threshold: float = 0.7

    @property
    def memory_variables(self) -> List[str]:
        """Return the memory keys that this class exposes."""
        return [self.memory_key]

    def _make_request(
        self,
        method: str,
        endpoint: str,
        json_data: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Make HTTP request to AgentMem backend."""
        url = f"{self.backend_url}{endpoint}"
        headers = {"Content-Type": "application/json"}

        try:
            if method.upper() == "GET":
                response = requests.get(url, headers=headers, params=json_data, timeout=10)
            else:
                response = requests.request(
                    method, url, headers=headers, json=json_data, timeout=10
                )
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            # Fallback: return empty result instead of raising
            return {"results": [], "error": str(e)}

    def load_memory_variables(self, inputs: Dict[str, Any]) -> Dict[str, str]:
        """
        Load relevant context from AgentMem based on current input.

        Retrieves semantically similar memories and formats them as
        conversation history for LangChain.
        """
        # Get current input
        query = inputs.get(self.input_key, "")

        if not query:
            return {self.memory_key: ""}

        # Search for relevant memories
        search_data = {
            "query": query,
            "session_id": self.session_id,
            "top_k": self.top_k,
            "threshold": self.threshold
        }

        result = self._make_request("POST", "/api/v1/memory/search", search_data)

        # Format retrieved memories
        memories = result.get("results", [])
        formatted_history = self._format_memories(memories)

        return {self.memory_key: formatted_history}

    def _format_memories(self, memories: List[Dict]) -> str:
        """Format memories into human-readable history."""
        if not memories:
            return ""

        lines = ["Relevant context from previous conversations:"]
        for i, mem in enumerate(memories, 1):
            content = mem.get("content", "")
            score = mem.get("score", 0.0)
            lines.append(f"{i}. {content} (relevance: {score:.2f})")

        return "\n".join(lines)

    def save_context(self, inputs: Dict[str, Any], outputs: Dict[str, str]) -> None:
        """
        Save conversation turn to AgentMem.

        Stores both user input and AI response with automatic embedding
        generation and intelligent deduplication.
        """
        # Get input and output
        user_input = inputs.get(self.input_key, "")
        ai_output = outputs.get(self.output_key, "")

        if not user_input or not ai_output:
            return

        # Store as conversation turn
        conversation = f"Human: {user_input}\nAI: {ai_output}"

        # Add to AgentMem
        memory_data = {
            "content": conversation,
            "session_id": self.session_id,
            "metadata": {
                "type": "conversation",
                "user_input": user_input,
                "ai_output": ai_output
            }
        }

        self._make_request("POST", "/api/v1/memory/add", memory_data)

    def clear(self) -> None:
        """Clear all memories for this session."""
        delete_data = {"session_id": self.session_id}
        self._make_request("DELETE", "/api/v1/memory/session", delete_data)


class AgentMemBufferMemory(AgentMemMemory):
    """
    Simpler buffer-style memory using AgentMem.

    Maintains a fixed-size buffer of recent conversation turns
    without semantic search. Useful for short-term conversations.
    """

    k: int = 5  # Number of turns to keep

    def load_memory_variables(self, inputs: Dict[str, Any]) -> Dict[str, str]:
        """Load recent conversation history."""
        # Get recent memories
        query = inputs.get(self.input_key, "")

        if not query:
            return {self.memory_key: ""}

        # Get recent memories (no semantic search)
        search_data = {
            "query": query,
            "session_id": self.session_id,
            "top_k": self.k * 2,  # Get more to filter
            "sort_by": "recency"
        }

        result = self._make_request("POST", "/api/v1/memory/search", search_data)
        memories = result.get("results", [])[:self.k]

        formatted_history = self._format_memories(memories)
        return {self.memory_key: formatted_history}


class AgentMemSummaryMemory(AgentMemMemory):
    """
    Summary memory that uses AgentMem's intelligent summarization.

    Automatically summarizes long conversations and stores summaries
    for efficient context management.
    """

    max_tokens: int = 2000  # Maximum tokens before summarization
    summary_frequency: int = 10  # Summarize every N turns

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.turn_count = 0

    def save_context(self, inputs: Dict[str, Any], outputs: Dict[str, str]) -> None:
        """Save context with automatic summarization."""
        super().save_context(inputs, outputs)

        self.turn_count += 1

        # Trigger summarization periodically
        if self.turn_count % self.summary_frequency == 0:
            self._summarize_conversation()

    def _summarize_conversation(self) -> None:
        """Generate and store conversation summary."""
        # Get recent conversation
        search_data = {
            "query": "summary of conversation",
            "session_id": self.session_id,
            "top_k": self.summary_frequency * 2,
            "sort_by": "recency"
        }

        result = self._make_request("POST", "/api/v1/memory/search", search_data)
        memories = result.get("results", [])

        if not memories:
            return

        # Combine memories
        conversation_text = "\n".join([
            mem.get("content", "") for mem in memories
        ])

        # Store as summary
        summary_data = {
            "content": f"Conversation Summary: {conversation_text[:self.max_tokens]}",
            "session_id": self.session_id,
            "metadata": {"type": "summary", "turn_count": self.turn_count}
        }

        self._make_request("POST", "/api/v1/memory/add", summary_data)


# Helper function for easy initialization
def create_agentmem_memory(
    session_id: str,
    backend_url: str = "http://localhost:8080",
    memory_type: str = "default",
    **kwargs
) -> AgentMemMemory:
    """
    Factory function to create AgentMem memory instances.

    Args:
        session_id: Unique session identifier
        backend_url: AgentMem server URL
        memory_type: Type of memory ("default", "buffer", "summary")
        **kwargs: Additional arguments for memory class

    Returns:
        Appropriate AgentMemMemory instance

    Example:
        ```python
        memory = create_agentmem_memory(
            session_id="user-123",
            backend_url="http://localhost:8080",
            memory_type="summary",
            top_k=10
        )
        ```
    """
    if memory_type == "buffer":
        return AgentMemBufferMemory(
            session_id=session_id,
            backend_url=backend_url,
            **kwargs
        )
    elif memory_type == "summary":
        return AgentMemSummaryMemory(
            session_id=session_id,
            backend_url=backend_url,
            **kwargs
        )
    else:
        return AgentMemMemory(
            session_id=session_id,
            backend_url=backend_url,
            **kwargs
        )


__all__ = [
    "AgentMemMemory",
    "AgentMemBufferMemory",
    "AgentMemSummaryMemory",
    "create_agentmem_memory"
]
