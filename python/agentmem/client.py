"""
AgentMem Python Client

Native Python client for AgentMem with async support.
"""

from typing import Any, Dict, List, Optional, Union
import requests
import asyncio
import aiohttp


class MemoryClient:
    """
    Synchronous client for AgentMem backend.

    Provides a Pythonic interface to the AgentMem memory system.

    Attributes:
        base_url: Base URL of the AgentMem server
        session_id: Optional session identifier for multi-tenant scenarios
        timeout: Request timeout in seconds
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        session_id: Optional[str] = None,
        timeout: int = 30
    ):
        self.base_url = base_url.rstrip("/")
        self.session_id = session_id
        self.timeout = timeout

    def _request(
        self,
        method: str,
        endpoint: str,
        json_data: Optional[Dict] = None,
        params: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Make HTTP request."""
        url = f"{self.base_url}{endpoint}"
        headers = {"Content-Type": "application/json"}

        if self.session_id:
            headers["X-Session-ID"] = self.session_id

        response = requests.request(
            method,
            url,
            headers=headers,
            json=json_data,
            params=params,
            timeout=self.timeout
        )
        response.raise_for_status()
        return response.json()

    def add(
        self,
        content: str,
        metadata: Optional[Dict[str, Any]] = None,
        scope: str = "session"
    ) -> Dict[str, Any]:
        """
        Add a memory.

        Args:
            content: Memory content
            metadata: Optional metadata dictionary
            scope: Memory scope ("session", "user", "global")

        Returns:
            Result dictionary with memory ID and status
        """
        data = {
            "content": content,
            "scope": scope
        }
        if metadata:
            data["metadata"] = metadata

        return self._request("POST", "/api/v1/memory/add", data)

    def search(
        self,
        query: str,
        top_k: int = 10,
        threshold: float = 0.0,
        scope: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """
        Search memories.

        Args:
            query: Search query
            top_k: Maximum number of results
            threshold: Minimum similarity score
            scope: Filter by scope

        Returns:
            List of memory results with scores
        """
        params = {
            "query": query,
            "top_k": top_k,
            "threshold": threshold
        }
        if scope:
            params["scope"] = scope

        result = self._request("GET", "/api/v1/memory/search", params=params)
        return result.get("results", [])

    def get_all(self, scope: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get all memories."""
        params = {}
        if scope:
            params["scope"] = scope

        result = self._request("GET", "/api/v1/memory/all", params=params)
        return result.get("memories", [])

    def delete(self, memory_id: str) -> bool:
        """Delete a memory by ID."""
        self._request("DELETE", f"/api/v1/memory/{memory_id}")
        return True

    def clear(self, scope: str = "session") -> bool:
        """Clear all memories in scope."""
        self._request("DELETE", "/api/v1/memory/clear", params={"scope": scope})
        return True


class AsyncMemoryClient:
    """
    Async client for AgentMem backend.

    Provides async interface using aiohttp for high-performance scenarios.
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        session_id: Optional[str] = None,
        timeout: int = 30
    ):
        self.base_url = base_url.rstrip("/")
        self.session_id = session_id
        self.timeout = timeout
        self._session: Optional[aiohttp.ClientSession] = None

    async def _get_session(self) -> aiohttp.ClientSession:
        """Get or create aiohttp session."""
        if self._session is None or self._session.closed:
            timeout = aiohttp.ClientTimeout(total=self.timeout)
            self._session = aiohttp.ClientSession(timeout=timeout)
        return self._session

    async def close(self) -> None:
        """Close the aiohttp session."""
        if self._session and not self._session.closed:
            await self._session.close()

    async def _request(
        self,
        method: str,
        endpoint: str,
        json_data: Optional[Dict] = None,
        params: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Make async HTTP request."""
        session = await self._get_session()
        url = f"{self.base_url}{endpoint}"
        headers = {"Content-Type": "application/json"}

        if self.session_id:
            headers["X-Session-ID"] = self.session_id

        async with session.request(
            method,
            url,
            headers=headers,
            json=json_data,
            params=params
        ) as response:
            response.raise_for_status()
            return await response.json()

    async def add(
        self,
        content: str,
        metadata: Optional[Dict[str, Any]] = None,
        scope: str = "session"
    ) -> Dict[str, Any]:
        """Add a memory."""
        data = {
            "content": content,
            "scope": scope
        }
        if metadata:
            data["metadata"] = metadata

        return await self._request("POST", "/api/v1/memory/add", data)

    async def search(
        self,
        query: str,
        top_k: int = 10,
        threshold: float = 0.0,
        scope: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Search memories."""
        params = {
            "query": query,
            "top_k": top_k,
            "threshold": threshold
        }
        if scope:
            params["scope"] = scope

        result = await self._request("GET", "/api/v1/memory/search", params=params)
        return result.get("results", [])

    async def get_all(self, scope: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get all memories."""
        params = {}
        if scope:
            params["scope"] = scope

        result = await self._request("GET", "/api/v1/memory/all", params=params)
        return result.get("memories", [])

    async def delete(self, memory_id: str) -> bool:
        """Delete a memory by ID."""
        await self._request("DELETE", f"/api/v1/memory/{memory_id}")
        return True

    async def clear(self, scope: str = "session") -> bool:
        """Clear all memories in scope."""
        await self._request("DELETE", "/api/v1/memory/clear", params={"scope": scope})
        return True


# Convenience class for simple usage
class Memory:
    """
    High-level Memory interface with auto-configuration.

    Automatically detects environment and provides sensible defaults.
    """

    def __init__(
        self,
        backend_url: Optional[str] = None,
        session_id: Optional[str] = None
    ):
        """
        Initialize Memory.

        Args:
            backend_url: AgentMem server URL (default: http://localhost:8080)
            session_id: Optional session identifier
        """
        if backend_url is None:
            backend_url = "http://localhost:8080"

        self.client = MemoryClient(base_url=backend_url, session_id=session_id)

    def add(self, content: str, **kwargs) -> Dict[str, Any]:
        """Add a memory."""
        return self.client.add(content, **kwargs)

    def search(self, query: str, **kwargs) -> List[Dict[str, Any]]:
        """Search memories."""
        return self.client.search(query, **kwargs)

    def get_all(self, **kwargs) -> List[Dict[str, Any]]:
        """Get all memories."""
        return self.client.get_all(**kwargs)

    def delete(self, memory_id: str) -> bool:
        """Delete a memory."""
        return self.client.delete(memory_id)

    def clear(self, **kwargs) -> bool:
        """Clear all memories."""
        return self.client.clear(**kwargs)


__all__ = ["MemoryClient", "AsyncMemoryClient", "Memory"]
