"""
Tests for AgentMem Observability Module
"""

import pytest
import time
from agentmem.observability import (
    MetricsCollector,
    PerformanceTracker,
    HealthStatus,
    ComponentHealth,
    HealthCheckResult,
)
from datetime import datetime


# Test fixtures
@pytest.fixture
def metrics_collector():
    """Create a metrics collector instance."""
    return MetricsCollector()


@pytest.fixture
def performance_tracker():
    """Create a performance tracker instance."""
    return PerformanceTracker()


# Tests
class TestMetricsCollector:
    """Tests for MetricsCollector."""
    
    def test_create_collector(self, metrics_collector):
        """Test creating a metrics collector."""
        assert metrics_collector is not None
        metrics = metrics_collector.get_metrics()
        assert "counters" in metrics
        assert "gauges" in metrics
        assert "histograms" in metrics
    
    def test_increment_counter(self, metrics_collector):
        """Test incrementing a counter."""
        metrics_collector.increment("test_counter")
        value = metrics_collector.get_counter("test_counter")
        assert value == 1.0
        
        metrics_collector.increment("test_counter", value=5.0)
        value = metrics_collector.get_counter("test_counter")
        assert value == 6.0
    
    def test_counter_with_labels(self, metrics_collector):
        """Test counter with labels."""
        metrics_collector.increment("requests", labels={"method": "GET", "endpoint": "/api"})
        metrics_collector.increment("requests", labels={"method": "POST", "endpoint": "/api"})
        metrics_collector.increment("requests", labels={"method": "GET", "endpoint": "/api"})
        
        get_count = metrics_collector.get_counter("requests", labels={"method": "GET", "endpoint": "/api"})
        post_count = metrics_collector.get_counter("requests", labels={"method": "POST", "endpoint": "/api"})
        
        assert get_count == 2.0
        assert post_count == 1.0
    
    def test_set_gauge(self, metrics_collector):
        """Test setting a gauge."""
        metrics_collector.set_gauge("memory_usage", 100.0)
        value = metrics_collector.get_gauge("memory_usage")
        assert value == 100.0
        
        metrics_collector.set_gauge("memory_usage", 150.0)
        value = metrics_collector.get_gauge("memory_usage")
        assert value == 150.0
    
    def test_gauge_with_labels(self, metrics_collector):
        """Test gauge with labels."""
        metrics_collector.set_gauge("connections", 10, labels={"type": "active"})
        metrics_collector.set_gauge("connections", 5, labels={"type": "idle"})
        
        active = metrics_collector.get_gauge("connections", labels={"type": "active"})
        idle = metrics_collector.get_gauge("connections", labels={"type": "idle"})
        
        assert active == 10
        assert idle == 5
    
    def test_record_histogram(self, metrics_collector):
        """Test recording histogram values."""
        values = [0.1, 0.2, 0.15, 0.3, 0.25]
        for value in values:
            metrics_collector.record_histogram("duration", value)
        
        stats = metrics_collector.get_histogram_stats("duration")
        assert stats["count"] == 5
        assert stats["min"] == 0.1
        assert stats["max"] == 0.3
        assert abs(stats["avg"] - 0.2) < 0.01
    
    def test_histogram_percentiles(self, metrics_collector):
        """Test histogram percentile calculations."""
        # Record 100 values from 0 to 99
        for i in range(100):
            metrics_collector.record_histogram("test", float(i))
        
        stats = metrics_collector.get_histogram_stats("test")
        
        # P50 should be around 49.5
        assert 48 <= stats["p50"] <= 51
        
        # P95 should be around 94.05
        assert 93 <= stats["p95"] <= 96
        
        # P99 should be around 98.01
        assert 97 <= stats["p99"] <= 99
    
    def test_histogram_with_labels(self, metrics_collector):
        """Test histogram with labels."""
        metrics_collector.record_histogram("latency", 0.1, labels={"endpoint": "/api/v1"})
        metrics_collector.record_histogram("latency", 0.2, labels={"endpoint": "/api/v1"})
        metrics_collector.record_histogram("latency", 0.5, labels={"endpoint": "/api/v2"})
        
        v1_stats = metrics_collector.get_histogram_stats("latency", labels={"endpoint": "/api/v1"})
        v2_stats = metrics_collector.get_histogram_stats("latency", labels={"endpoint": "/api/v2"})
        
        assert v1_stats["count"] == 2
        assert v2_stats["count"] == 1
        assert v2_stats["avg"] == 0.5
    
    def test_get_all_metrics(self, metrics_collector):
        """Test getting all metrics."""
        metrics_collector.increment("counter1")
        metrics_collector.set_gauge("gauge1", 100)
        metrics_collector.record_histogram("hist1", 0.5)
        
        all_metrics = metrics_collector.get_metrics()
        
        assert "counters" in all_metrics
        assert "gauges" in all_metrics
        assert "histograms" in all_metrics
        assert "uptime_seconds" in all_metrics
        
        assert "counter1" in all_metrics["counters"]
        assert "gauge1" in all_metrics["gauges"]
        assert "hist1" in all_metrics["histograms"]
    
    def test_reset_metrics(self, metrics_collector):
        """Test resetting metrics."""
        metrics_collector.increment("counter")
        metrics_collector.set_gauge("gauge", 100)
        metrics_collector.record_histogram("hist", 0.5)
        
        metrics_collector.reset()
        
        all_metrics = metrics_collector.get_metrics()
        assert len(all_metrics["counters"]) == 0
        assert len(all_metrics["gauges"]) == 0
        assert len(all_metrics["histograms"]) == 0


class TestPerformanceTracker:
    """Tests for PerformanceTracker."""
    
    def test_create_tracker(self, performance_tracker):
        """Test creating a performance tracker."""
        assert performance_tracker is not None
    
    def test_record_operation(self, performance_tracker):
        """Test recording an operation."""
        performance_tracker.record("test_op", 10.5)
        performance_tracker.record("test_op", 15.2)
        performance_tracker.record("test_op", 12.8)
        
        stats = performance_tracker.get_stats("test_op")
        assert stats["count"] == 3
        assert abs(stats["avg"] - 12.83) < 0.1
        assert stats["min"] == 10.5
        assert stats["max"] == 15.2
    
    def test_track_context_manager(self, performance_tracker):
        """Test tracking with context manager."""
        with performance_tracker.track("operation"):
            time.sleep(0.01)  # Simulate work
        
        stats = performance_tracker.get_stats("operation")
        assert stats["count"] == 1
        assert stats["avg"] >= 10  # At least 10ms
    
    def test_multiple_operations(self, performance_tracker):
        """Test tracking multiple operations."""
        for _ in range(5):
            with performance_tracker.track("op1"):
                time.sleep(0.005)
        
        for _ in range(3):
            with performance_tracker.track("op2"):
                time.sleep(0.01)
        
        stats1 = performance_tracker.get_stats("op1")
        stats2 = performance_tracker.get_stats("op2")
        
        assert stats1["count"] == 5
        assert stats2["count"] == 3
        assert stats2["avg"] > stats1["avg"]  # op2 should be slower
    
    def test_stats_for_nonexistent_operation(self, performance_tracker):
        """Test getting stats for non-existent operation."""
        stats = performance_tracker.get_stats("nonexistent")
        
        assert stats["count"] == 0
        assert stats["avg"] == 0.0
        assert stats["min"] == 0.0
        assert stats["max"] == 0.0


class TestHealthStatus:
    """Tests for HealthStatus."""
    
    def test_health_status_values(self):
        """Test health status enum values."""
        assert HealthStatus.HEALTHY.value == "healthy"
        assert HealthStatus.DEGRADED.value == "degraded"
        assert HealthStatus.UNHEALTHY.value == "unhealthy"


class TestComponentHealth:
    """Tests for ComponentHealth."""
    
    def test_create_component_health(self):
        """Test creating component health."""
        health = ComponentHealth(
            name="database",
            status=HealthStatus.HEALTHY,
            message="All connections active",
            last_check=datetime.now()
        )
        
        assert health.name == "database"
        assert health.status == HealthStatus.HEALTHY
        assert health.message == "All connections active"
        assert health.last_check is not None
    
    def test_component_health_to_dict(self):
        """Test converting component health to dictionary."""
        now = datetime.now()
        health = ComponentHealth(
            name="cache",
            status=HealthStatus.DEGRADED,
            message="High latency",
            last_check=now
        )
        
        data = health.to_dict()
        assert data["name"] == "cache"
        assert data["status"] == "degraded"
        assert data["message"] == "High latency"
        assert data["last_check"] == now.isoformat()
    
    def test_component_health_from_dict(self):
        """Test creating component health from dictionary."""
        now = datetime.now()
        data = {
            "name": "api",
            "status": "healthy",
            "message": "OK",
            "last_check": now.isoformat()
        }
        
        health = ComponentHealth.from_dict(data)
        assert health.name == "api"
        assert health.status == HealthStatus.HEALTHY
        assert health.message == "OK"


class TestHealthCheckResult:
    """Tests for HealthCheckResult."""
    
    def test_create_health_check_result(self):
        """Test creating health check result."""
        components = [
            ComponentHealth("db", HealthStatus.HEALTHY),
            ComponentHealth("cache", HealthStatus.HEALTHY),
        ]
        
        result = HealthCheckResult(
            status=HealthStatus.HEALTHY,
            components=components,
            version="1.0.0",
            uptime_seconds=3600.0,
            timestamp=datetime.now()
        )
        
        assert result.status == HealthStatus.HEALTHY
        assert len(result.components) == 2
        assert result.version == "1.0.0"
        assert result.uptime_seconds == 3600.0
    
    def test_health_check_result_to_dict(self):
        """Test converting health check result to dictionary."""
        now = datetime.now()
        components = [ComponentHealth("db", HealthStatus.HEALTHY)]
        
        result = HealthCheckResult(
            status=HealthStatus.HEALTHY,
            components=components,
            version="1.0.0",
            uptime_seconds=100.0,
            timestamp=now
        )
        
        data = result.to_dict()
        assert data["status"] == "healthy"
        assert len(data["components"]) == 1
        assert data["version"] == "1.0.0"
        assert data["uptime_seconds"] == 100.0
    
    def test_health_check_result_from_dict(self):
        """Test creating health check result from dictionary."""
        now = datetime.now()
        data = {
            "status": "degraded",
            "components": [
                {"name": "db", "status": "healthy", "message": None, "last_check": None},
                {"name": "cache", "status": "degraded", "message": "Slow", "last_check": None},
            ],
            "version": "2.0.0",
            "uptime_seconds": 7200.0,
            "timestamp": now.isoformat()
        }
        
        result = HealthCheckResult.from_dict(data)
        assert result.status == HealthStatus.DEGRADED
        assert len(result.components) == 2
        assert result.components[1].status == HealthStatus.DEGRADED


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

