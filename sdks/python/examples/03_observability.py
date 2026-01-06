"""
Example 3: Observability and Monitoring

This example demonstrates observability features:
- Metrics collection
- Performance tracking
- Health checks
"""

import asyncio
import random
from agentmem.observability import MetricsCollector, PerformanceTracker


async def simulate_api_request(endpoint: str, method: str = "GET"):
    """Simulate an API request."""
    # Simulate processing time
    await asyncio.sleep(random.uniform(0.01, 0.1))
    
    # Simulate occasional errors
    if random.random() < 0.1:  # 10% error rate
        raise Exception("Simulated API error")
    
    return {"status": "success", "endpoint": endpoint}


async def simulate_database_query(query_type: str):
    """Simulate a database query."""
    # Simulate query time
    await asyncio.sleep(random.uniform(0.005, 0.05))
    
    return {"rows": random.randint(1, 100)}


async def main():
    """Main example function."""
    print("🚀 AgentMem Python SDK - Observability Example\n")
    
    # Create metrics collector and performance tracker
    metrics = MetricsCollector()
    tracker = PerformanceTracker()
    
    # 1. Basic metrics collection
    print("1️⃣  Collecting basic metrics...")
    
    # Increment counters
    metrics.increment("requests_total", labels={"method": "GET", "endpoint": "/api/memories"})
    metrics.increment("requests_total", labels={"method": "POST", "endpoint": "/api/memories"})
    metrics.increment("requests_total", labels={"method": "GET", "endpoint": "/api/memories"})
    
    # Set gauges
    metrics.set_gauge("active_connections", 15)
    metrics.set_gauge("memory_usage_bytes", 104857600)  # 100 MB
    
    # Record histograms
    metrics.record_histogram("request_duration_seconds", 0.025)
    metrics.record_histogram("request_duration_seconds", 0.032)
    metrics.record_histogram("request_duration_seconds", 0.018)
    
    print("   ✅ Metrics recorded\n")
    
    # 2. Simulate API requests with metrics
    print("2️⃣  Simulating API requests...")
    
    endpoints = ["/api/memories", "/api/search", "/api/stats"]
    methods = ["GET", "POST", "PUT"]
    
    for _ in range(20):
        endpoint = random.choice(endpoints)
        method = random.choice(methods)
        
        try:
            with tracker.track(f"api_request_{endpoint}"):
                await simulate_api_request(endpoint, method)
            
            # Record success
            metrics.increment("requests_total", labels={"method": method, "endpoint": endpoint, "status": "200"})
            metrics.record_histogram("request_duration_seconds", random.uniform(0.01, 0.1))
        
        except Exception as e:
            # Record error
            metrics.increment("errors_total", labels={"type": "api_error", "endpoint": endpoint})
    
    print(f"   ✅ Simulated 20 API requests\n")
    
    # 3. Simulate database queries with tracking
    print("3️⃣  Simulating database queries...")
    
    query_types = ["SELECT", "INSERT", "UPDATE", "DELETE"]
    
    for _ in range(30):
        query_type = random.choice(query_types)
        
        with tracker.track(f"db_query_{query_type}"):
            await simulate_database_query(query_type)
        
        metrics.increment("db_queries_total", labels={"type": query_type})
    
    print(f"   ✅ Simulated 30 database queries\n")
    
    # 4. Get and display metrics
    print("4️⃣  Displaying collected metrics...")
    
    all_metrics = metrics.get_metrics()
    
    print("   📊 Counters:")
    for name, value in all_metrics["counters"].items():
        print(f"      {name}: {value}")
    
    print("\n   📊 Gauges:")
    for name, value in all_metrics["gauges"].items():
        print(f"      {name}: {value}")
    
    print("\n   📊 Histograms:")
    for name, stats in all_metrics["histograms"].items():
        print(f"      {name}:")
        print(f"         Count: {stats['count']}")
        print(f"         Avg: {stats['avg']:.4f}")
        print(f"         Min: {stats['min']:.4f}")
        print(f"         Max: {stats['max']:.4f}")
        print(f"         P50: {stats['p50']:.4f}")
        print(f"         P95: {stats['p95']:.4f}")
        print(f"         P99: {stats['p99']:.4f}")
    
    print(f"\n   ⏱️  Uptime: {all_metrics['uptime_seconds']:.2f}s\n")
    
    # 5. Display performance statistics
    print("5️⃣  Displaying performance statistics...")
    
    # API request stats
    print("   📈 API Request Performance:")
    for endpoint in endpoints:
        stats = tracker.get_stats(f"api_request_{endpoint}")
        if stats["count"] > 0:
            print(f"      {endpoint}:")
            print(f"         Count: {stats['count']}")
            print(f"         Avg: {stats['avg']:.2f}ms")
            print(f"         Min: {stats['min']:.2f}ms")
            print(f"         Max: {stats['max']:.2f}ms")
            print(f"         P95: {stats['p95']:.2f}ms")
    
    # Database query stats
    print("\n   📈 Database Query Performance:")
    for query_type in query_types:
        stats = tracker.get_stats(f"db_query_{query_type}")
        if stats["count"] > 0:
            print(f"      {query_type}:")
            print(f"         Count: {stats['count']}")
            print(f"         Avg: {stats['avg']:.2f}ms")
            print(f"         P95: {stats['p95']:.2f}ms")
    
    print()
    
    # 6. Demonstrate metric queries
    print("6️⃣  Querying specific metrics...")
    
    # Get specific counter
    get_requests = metrics.get_counter("requests_total", labels={"method": "GET", "endpoint": "/api/memories", "status": "200"})
    print(f"   📊 GET /api/memories requests: {get_requests}")
    
    # Get specific gauge
    active_conns = metrics.get_gauge("active_connections")
    print(f"   📊 Active connections: {active_conns}")
    
    # Get histogram stats
    duration_stats = metrics.get_histogram_stats("request_duration_seconds")
    print(f"   📊 Request duration P95: {duration_stats['p95']:.4f}s\n")
    
    # 7. Demonstrate context manager for tracking
    print("7️⃣  Using context manager for tracking...")
    
    async def complex_operation():
        """A complex operation to track."""
        await asyncio.sleep(0.05)
        # Simulate some work
        result = sum(range(1000))
        return result
    
    with tracker.track("complex_operation"):
        result = await complex_operation()
    
    stats = tracker.get_stats("complex_operation")
    print(f"   ✅ Complex operation completed in {stats['avg']:.2f}ms\n")
    
    # 8. Summary
    print("8️⃣  Summary...")
    
    total_requests = sum(v for k, v in all_metrics["counters"].items() if "requests_total" in k)
    total_errors = sum(v for k, v in all_metrics["counters"].items() if "errors_total" in k)
    error_rate = (total_errors / total_requests * 100) if total_requests > 0 else 0
    
    print(f"   📊 Total requests: {total_requests}")
    print(f"   ❌ Total errors: {total_errors}")
    print(f"   📉 Error rate: {error_rate:.2f}%")
    print(f"   ⏱️  Uptime: {all_metrics['uptime_seconds']:.2f}s\n")
    
    print("✨ Example completed successfully!")


if __name__ == "__main__":
    asyncio.run(main())

