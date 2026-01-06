"""
AgentMem Python SDK - Configuration

Configuration management for the AgentMem Python SDK.
"""

import os
from typing import Optional, Dict, Any
from dataclasses import dataclass


@dataclass
class Config:
    """AgentMem client configuration."""
    
    # Authentication
    api_key: str
    
    # Server settings
    base_url: str = "https://api.agentmem.dev"
    api_version: str = "v1"
    
    # Request settings
    timeout: int = 30
    max_retries: int = 3
    retry_delay: float = 1.0
    
    # Connection settings
    max_connections: int = 100
    max_keepalive_connections: int = 20
    keepalive_expiry: int = 5
    
    # Logging
    enable_logging: bool = False
    log_level: str = "INFO"
    
    # Features
    enable_compression: bool = True
    enable_caching: bool = True
    cache_ttl: int = 300  # 5 minutes
    
    @classmethod
    def from_env(cls, **overrides) -> "Config":
        """Create configuration from environment variables."""
        api_key = os.getenv("AGENTMEM_API_KEY")
        if not api_key:
            raise ValueError("AGENTMEM_API_KEY environment variable is required")
        
        config = cls(
            api_key=api_key,
            base_url=os.getenv("AGENTMEM_BASE_URL", "https://api.agentmem.dev"),
            api_version=os.getenv("AGENTMEM_API_VERSION", "v1"),
            timeout=int(os.getenv("AGENTMEM_TIMEOUT", "30")),
            max_retries=int(os.getenv("AGENTMEM_MAX_RETRIES", "3")),
            retry_delay=float(os.getenv("AGENTMEM_RETRY_DELAY", "1.0")),
            max_connections=int(os.getenv("AGENTMEM_MAX_CONNECTIONS", "100")),
            max_keepalive_connections=int(os.getenv("AGENTMEM_MAX_KEEPALIVE_CONNECTIONS", "20")),
            keepalive_expiry=int(os.getenv("AGENTMEM_KEEPALIVE_EXPIRY", "5")),
            enable_logging=os.getenv("AGENTMEM_ENABLE_LOGGING", "false").lower() == "true",
            log_level=os.getenv("AGENTMEM_LOG_LEVEL", "INFO"),
            enable_compression=os.getenv("AGENTMEM_ENABLE_COMPRESSION", "true").lower() == "true",
            enable_caching=os.getenv("AGENTMEM_ENABLE_CACHING", "true").lower() == "true",
            cache_ttl=int(os.getenv("AGENTMEM_CACHE_TTL", "300")),
        )
        
        # Apply overrides
        for key, value in overrides.items():
            if hasattr(config, key):
                setattr(config, key, value)
        
        return config
    
    @property
    def api_base_url(self) -> str:
        """Get the full API base URL."""
        return f"{self.base_url}/api/{self.api_version}"
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert config to dictionary."""
        return {
            "api_key": "***" if self.api_key else None,  # Mask API key
            "base_url": self.base_url,
            "api_version": self.api_version,
            "timeout": self.timeout,
            "max_retries": self.max_retries,
            "retry_delay": self.retry_delay,
            "max_connections": self.max_connections,
            "max_keepalive_connections": self.max_keepalive_connections,
            "keepalive_expiry": self.keepalive_expiry,
            "enable_logging": self.enable_logging,
            "log_level": self.log_level,
            "enable_compression": self.enable_compression,
            "enable_caching": self.enable_caching,
            "cache_ttl": self.cache_ttl,
        }
    
    def validate(self) -> None:
        """Validate configuration."""
        if not self.api_key:
            raise ValueError("API key is required")
        
        if not self.base_url:
            raise ValueError("Base URL is required")
        
        if self.timeout <= 0:
            raise ValueError("Timeout must be positive")
        
        if self.max_retries < 0:
            raise ValueError("Max retries must be non-negative")
        
        if self.retry_delay < 0:
            raise ValueError("Retry delay must be non-negative")
        
        if self.max_connections <= 0:
            raise ValueError("Max connections must be positive")
        
        if self.max_keepalive_connections <= 0:
            raise ValueError("Max keepalive connections must be positive")
        
        if self.keepalive_expiry <= 0:
            raise ValueError("Keepalive expiry must be positive")
        
        if self.cache_ttl <= 0:
            raise ValueError("Cache TTL must be positive")
        
        valid_log_levels = ["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"]
        if self.log_level not in valid_log_levels:
            raise ValueError(f"Log level must be one of: {valid_log_levels}")


# Default configuration
DEFAULT_CONFIG = Config(
    api_key="",  # Must be set by user
    base_url="https://api.agentmem.dev",
    api_version="v1",
    timeout=30,
    max_retries=3,
    retry_delay=1.0,
    max_connections=100,
    max_keepalive_connections=20,
    keepalive_expiry=5,
    enable_logging=False,
    log_level="INFO",
    enable_compression=True,
    enable_caching=True,
    cache_ttl=300,
)
