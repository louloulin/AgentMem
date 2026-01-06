"""
AgentMem Python SDK - Observability

Monitoring, metrics, and health checks for AgentMem.
"""

from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum
from datetime import datetime
import time


class HealthStatus(Enum):
    """Health status enumeration."""
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"


@dataclass
class ComponentHealth:
    """Component health status."""
    name: str
    status: HealthStatus
    message: Optional[str] = None
    last_check: Optional[datetime] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "name": self.name,
            "status": self.status.value,
            "message": self.message,
            "last_check": self.last_check.isoformat() if self.last_check else None,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ComponentHealth":
        """Create from dictionary."""
        return cls(
            name=data["name"],
            status=HealthStatus(data["status"]),
            message=data.get("message"),
            last_check=datetime.fromisoformat(data["last_check"]) if data.get("last_check") else None,
        )


@dataclass
class HealthCheckResult:
    """Health check result."""
    status: HealthStatus
    components: List[ComponentHealth]
    version: str
    uptime_seconds: float
    timestamp: datetime
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "status": self.status.value,
            "components": [c.to_dict() for c in self.components],
            "version": self.version,
            "uptime_seconds": self.uptime_seconds,
            "timestamp": self.timestamp.isoformat(),
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "HealthCheckResult":
        """Create from dictionary."""
        return cls(
            status=HealthStatus(data["status"]),
            components=[ComponentHealth.from_dict(c) for c in data["components"]],
            version=data["version"],
            uptime_seconds=data["uptime_seconds"],
            timestamp=datetime.fromisoformat(data["timestamp"]),
        )


@dataclass
class MetricValue:
    """Metric value with timestamp."""
    value: float
    timestamp: datetime
    labels: Optional[Dict[str, str]] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            "value": self.value,
            "timestamp": self.timestamp.isoformat(),
            "labels": self.labels or {},
        }


class MetricsCollector:
    """
    Metrics collector for AgentMem.
    
    Example:
        ```python
        from agentmem.observability import MetricsCollector
        
        collector = MetricsCollector()
        
        # Record metrics
        collector.increment("requests_total", labels={"method": "GET", "endpoint": "/api/memories"})
        collector.set_gauge("active_connections", 10)
        collector.record_histogram("request_duration_seconds", 0.025)
        
        # Get metrics
        metrics = collector.get_metrics()
        print(f"Total requests: {metrics['requests_total']}")
        ```
    """
    
    def __init__(self):
        """Initialize metrics collector."""
        self._counters: Dict[str, float] = {}
        self._gauges: Dict[str, float] = {}
        self._histograms: Dict[str, List[float]] = {}
        self._start_time = time.time()
    
    def increment(self, name: str, value: float = 1.0, labels: Optional[Dict[str, str]] = None) -> None:
        """
        Increment a counter.
        
        Args:
            name: Counter name
            value: Increment value
            labels: Optional labels
        """
        key = self._make_key(name, labels)
        self._counters[key] = self._counters.get(key, 0.0) + value
    
    def set_gauge(self, name: str, value: float, labels: Optional[Dict[str, str]] = None) -> None:
        """
        Set a gauge value.
        
        Args:
            name: Gauge name
            value: Gauge value
            labels: Optional labels
        """
        key = self._make_key(name, labels)
        self._gauges[key] = value
    
    def record_histogram(self, name: str, value: float, labels: Optional[Dict[str, str]] = None) -> None:
        """
        Record a histogram value.
        
        Args:
            name: Histogram name
            value: Value to record
            labels: Optional labels
        """
        key = self._make_key(name, labels)
        if key not in self._histograms:
            self._histograms[key] = []
        self._histograms[key].append(value)
    
    def get_counter(self, name: str, labels: Optional[Dict[str, str]] = None) -> float:
        """Get counter value."""
        key = self._make_key(name, labels)
        return self._counters.get(key, 0.0)
    
    def get_gauge(self, name: str, labels: Optional[Dict[str, str]] = None) -> float:
        """Get gauge value."""
        key = self._make_key(name, labels)
        return self._gauges.get(key, 0.0)
    
    def get_histogram_stats(self, name: str, labels: Optional[Dict[str, str]] = None) -> Dict[str, float]:
        """
        Get histogram statistics.
        
        Returns:
            Dictionary with count, sum, min, max, avg, p50, p95, p99
        """
        key = self._make_key(name, labels)
        values = self._histograms.get(key, [])
        
        if not values:
            return {
                "count": 0,
                "sum": 0.0,
                "min": 0.0,
                "max": 0.0,
                "avg": 0.0,
                "p50": 0.0,
                "p95": 0.0,
                "p99": 0.0,
            }
        
        sorted_values = sorted(values)
        count = len(sorted_values)
        
        return {
            "count": count,
            "sum": sum(sorted_values),
            "min": sorted_values[0],
            "max": sorted_values[-1],
            "avg": sum(sorted_values) / count,
            "p50": self._percentile(sorted_values, 0.50),
            "p95": self._percentile(sorted_values, 0.95),
            "p99": self._percentile(sorted_values, 0.99),
        }
    
    def get_metrics(self) -> Dict[str, Any]:
        """
        Get all metrics.
        
        Returns:
            Dictionary with all metrics
        """
        return {
            "counters": dict(self._counters),
            "gauges": dict(self._gauges),
            "histograms": {
                name: self.get_histogram_stats(name)
                for name in set(k.split("|")[0] for k in self._histograms.keys())
            },
            "uptime_seconds": time.time() - self._start_time,
        }
    
    def reset(self) -> None:
        """Reset all metrics."""
        self._counters.clear()
        self._gauges.clear()
        self._histograms.clear()
        self._start_time = time.time()
    
    def _make_key(self, name: str, labels: Optional[Dict[str, str]] = None) -> str:
        """Make metric key with labels."""
        if not labels:
            return name
        
        label_str = ",".join(f"{k}={v}" for k, v in sorted(labels.items()))
        return f"{name}|{label_str}"
    
    def _percentile(self, sorted_values: List[float], p: float) -> float:
        """Calculate percentile."""
        if not sorted_values:
            return 0.0
        
        k = (len(sorted_values) - 1) * p
        f = int(k)
        c = f + 1
        
        if c >= len(sorted_values):
            return sorted_values[-1]
        
        d0 = sorted_values[f] * (c - k)
        d1 = sorted_values[c] * (k - f)
        return d0 + d1


class PerformanceTracker:
    """
    Performance tracker for operations.
    
    Example:
        ```python
        from agentmem.observability import PerformanceTracker
        
        tracker = PerformanceTracker()
        
        # Track operation
        with tracker.track("database_query"):
            # Your code here
            result = await db.query(...)
        
        # Get statistics
        stats = tracker.get_stats("database_query")
        print(f"Average duration: {stats['avg']}ms")
        ```
    """
    
    def __init__(self):
        """Initialize performance tracker."""
        self._operations: Dict[str, List[float]] = {}
    
    def track(self, operation_name: str):
        """
        Context manager for tracking operation duration.
        
        Args:
            operation_name: Operation name
        """
        return _OperationContext(self, operation_name)
    
    def record(self, operation_name: str, duration_ms: float) -> None:
        """
        Record operation duration.
        
        Args:
            operation_name: Operation name
            duration_ms: Duration in milliseconds
        """
        if operation_name not in self._operations:
            self._operations[operation_name] = []
        self._operations[operation_name].append(duration_ms)
    
    def get_stats(self, operation_name: str) -> Dict[str, float]:
        """Get operation statistics."""
        durations = self._operations.get(operation_name, [])
        
        if not durations:
            return {
                "count": 0,
                "avg": 0.0,
                "min": 0.0,
                "max": 0.0,
                "p50": 0.0,
                "p95": 0.0,
                "p99": 0.0,
            }
        
        sorted_durations = sorted(durations)
        count = len(sorted_durations)
        
        return {
            "count": count,
            "avg": sum(sorted_durations) / count,
            "min": sorted_durations[0],
            "max": sorted_durations[-1],
            "p50": self._percentile(sorted_durations, 0.50),
            "p95": self._percentile(sorted_durations, 0.95),
            "p99": self._percentile(sorted_durations, 0.99),
        }
    
    def _percentile(self, sorted_values: List[float], p: float) -> float:
        """Calculate percentile."""
        if not sorted_values:
            return 0.0
        
        k = (len(sorted_values) - 1) * p
        f = int(k)
        c = f + 1
        
        if c >= len(sorted_values):
            return sorted_values[-1]
        
        d0 = sorted_values[f] * (c - k)
        d1 = sorted_values[c] * (k - f)
        return d0 + d1


class _OperationContext:
    """Context manager for operation tracking."""
    
    def __init__(self, tracker: PerformanceTracker, operation_name: str):
        self.tracker = tracker
        self.operation_name = operation_name
        self.start_time = None
    
    def __enter__(self):
        self.start_time = time.time()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        duration_ms = (time.time() - self.start_time) * 1000
        self.tracker.record(self.operation_name, duration_ms)

