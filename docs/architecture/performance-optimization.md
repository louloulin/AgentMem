# AgentMem Performance Optimization Guide

## Overview

This document describes the performance optimization features in AgentMem, including database migration management, connection pool optimization, and query performance analysis.

## Table of Contents

1. [Migration Management](#migration-management)
2. [Connection Pool Optimization](#connection-pool-optimization)
3. [Query Performance Analysis](#query-performance-analysis)
4. [Performance Benchmarks](#performance-benchmarks)
5. [Best Practices](#best-practices)

---

## Migration Management

### Overview

The `MigrationManager` provides production-grade database migration management with:
- Version tracking
- Up/down migrations
- Migration history
- Rollback support
- Checksum verification

### Usage

```rust
use agent_mem_core::storage::migration_manager::MigrationManager;
use sqlx::PgPool;

// Create migration manager
let pool = PgPool::connect("postgresql://localhost/agentmem").await?;
let manager = MigrationManager::new(pool);

// Initialize migration tracking
manager.init().await?;

// Check current version
let version = manager.current_version().await?;
println!("Current schema version: {:?}", version);

// Run all pending migrations
manager.migrate_up().await?;

// Rollback N migrations
manager.migrate_down(1).await?;
```

### Migration Workflow

1. **Check Current Version**
   ```rust
   let version = manager.current_version().await?;
   ```

2. **Get Pending Migrations**
   ```rust
   let pending = manager.pending_migrations().await?;
   ```

3. **Apply Migrations**
   ```rust
   manager.migrate_up().await?;
   ```

4. **Rollback (if needed)**
   ```rust
   manager.migrate_down(1).await?;
   ```

### Migration History

```rust
// Get all applied migrations
let applied = manager.applied_migrations().await?;
for migration in applied {
    println!("Version {}: {} (applied at {})", 
        migration.version, 
        migration.name, 
        migration.applied_at
    );
}
```

---

## Connection Pool Optimization

### Overview

The `PoolManager` provides advanced connection pool management with:
- Dynamic pool sizing
- Connection health checks
- Pool statistics
- Automatic reconnection
- Retry mechanisms

### Configuration Presets

#### Production Configuration
```rust
use agent_mem_core::storage::pool_manager::{PoolConfig, PoolManager};

let config = PoolConfig::production("postgresql://localhost/agentmem".to_string());
// min_connections: 5
// max_connections: 20
// slow_statement_threshold_ms: 50

let manager = PoolManager::new(config).await?;
```

#### Development Configuration
```rust
let config = PoolConfig::development("postgresql://localhost/agentmem".to_string());
// min_connections: 1
// max_connections: 5
// log_statements: true
```

#### High-Performance Configuration
```rust
let config = PoolConfig::high_performance("postgresql://localhost/agentmem".to_string());
// min_connections: 10
// max_connections: 50
// slow_statement_threshold_ms: 20
```

### Custom Configuration
```rust
let config = PoolConfig {
    url: "postgresql://localhost/agentmem".to_string(),
    min_connections: 3,
    max_connections: 15,
    connect_timeout: 30,
    idle_timeout: 600,
    max_lifetime: 1800,
    acquire_timeout: 30,
    log_statements: false,
    log_slow_statements: true,
    slow_statement_threshold_ms: 100,
};
```

### Health Monitoring

```rust
// Check pool health
let is_healthy = manager.health_check().await?;

// Get pool statistics
let stats = manager.stats().await;
println!("Total connections: {}", stats.total_connections);
println!("Idle connections: {}", stats.idle_connections);
println!("Active connections: {}", stats.active_connections);

// Get detailed metrics
let metrics = manager.metrics().await;
println!("Utilization: {:.2}%", metrics.utilization);
println!("Status: {}", metrics.status());
```

### Retry Mechanism

```rust
// Execute operation with automatic retry
let result = manager.execute_with_retry(3, |pool| async move {
    sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))
}).await?;
```

---

## Query Performance Analysis

### Overview

The `QueryAnalyzer` provides tools for analyzing and optimizing database queries:
- EXPLAIN ANALYZE support
- Slow query logging
- Query statistics
- Index recommendations

### Usage

```rust
use agent_mem_core::storage::query_analyzer::QueryAnalyzer;

let analyzer = QueryAnalyzer::new(pool, 50.0); // 50ms threshold
```

### EXPLAIN ANALYZE

```rust
let query = "SELECT * FROM agents WHERE organization_id = $1";
let plan = analyzer.explain_analyze(query).await?;

println!("Execution time: {:.2}ms", plan.execution_time_ms);
println!("Planning time: {:.2}ms", plan.planning_time_ms);
println!("Total cost: {:.2}", plan.total_cost);
println!("Estimated rows: {}", plan.rows);
println!("Plan:\n{}", plan.plan);
```

### Query Statistics

```rust
// Record query execution
analyzer.record_execution(query, execution_time_ms).await;

// Get all query statistics
let stats = analyzer.get_stats().await;
for stat in stats {
    println!("Query: {}", stat.query_text);
    println!("  Executions: {}", stat.execution_count);
    println!("  Avg time: {:.2}ms", stat.avg_time_ms);
    println!("  Min/Max: {:.2}ms / {:.2}ms", stat.min_time_ms, stat.max_time_ms);
}
```

### Slow Query Detection

```rust
// Get slow queries
let slow_queries = analyzer.get_slow_queries().await;
for slow in slow_queries {
    println!("Slow query: {}", slow.query);
    println!("  Time: {:.2}ms", slow.execution_time_ms);
    println!("  Timestamp: {}", slow.timestamp);
}

// Get top N slowest queries
let slowest = analyzer.get_slowest_queries(10).await;
```

### Index Recommendations

```rust
// Get index recommendations
let recommendations = analyzer.get_index_recommendations().await?;
for rec in recommendations {
    println!("Table: {}", rec.table_name);
    println!("  Priority: {}", rec.priority);
    println!("  Recommendation: {}", rec.recommendation);
}

// Get unused indexes
let unused = analyzer.get_unused_indexes().await?;
for idx in unused {
    println!("Unused index: {}.{}", idx.table_name, idx.index_name);
}
```

---

## Performance Benchmarks

### Running Benchmarks

```bash
# Run all benchmarks
cargo test --package agent-mem-core --test performance_benchmark -- --ignored

# Run specific benchmark
cargo test --package agent-mem-core --test performance_benchmark benchmark_crud_operations -- --ignored
```

### Performance Targets

| Operation | Target | Description |
|-----------|--------|-------------|
| CRUD Operations | < 50ms | Create, Read, Update, Delete |
| Batch Operations | < 10ms per item | Batch inserts/updates |
| Search Operations | < 100ms | Full-text and vector search |
| Concurrent Operations | < 20ms | Concurrent reads/writes |

### Benchmark Results

Run the benchmarks to see detailed results:

```bash
cargo test --package agent-mem-core --test performance_benchmark benchmark_summary -- --ignored
```

---

## Best Practices

### 1. Connection Pool Sizing

- **Development**: Use small pools (1-5 connections)
- **Production**: Use medium pools (5-20 connections)
- **High-Traffic**: Use large pools (10-50 connections)

### 2. Query Optimization

- Use EXPLAIN ANALYZE to understand query plans
- Add indexes for frequently queried columns
- Remove unused indexes
- Monitor slow queries regularly

### 3. Migration Management

- Always test migrations in development first
- Keep migrations small and focused
- Use checksums to verify migration integrity
- Have a rollback plan

### 4. Monitoring

- Monitor pool utilization (keep < 80%)
- Track slow queries (> 50ms)
- Review index recommendations weekly
- Check for connection leaks

### 5. Performance Testing

- Run benchmarks before production deployment
- Test under realistic load
- Monitor performance metrics in production
- Set up alerts for performance degradation

---

## Troubleshooting

### High Pool Utilization

```rust
let metrics = manager.metrics().await;
if metrics.is_under_pressure() {
    // Increase max_connections or optimize queries
    println!("Pool under pressure: {:.2}% utilization", metrics.utilization);
}
```

### Slow Queries

```rust
let slowest = analyzer.get_slowest_queries(10).await;
for query in slowest {
    // Analyze and optimize these queries
    let plan = analyzer.explain_analyze(&query.query_text).await?;
    println!("Query plan: {}", plan.plan);
}
```

### Connection Timeouts

```rust
// Increase acquire_timeout or max_connections
let config = PoolConfig {
    acquire_timeout: 60, // Increase from 30
    max_connections: 30, // Increase from 20
    ..Default::default()
};
```

---

## Examples

See the test files for complete examples:
- `tests/storage_optimization_test.rs` - Migration, pool, and query analyzer tests
- `tests/performance_benchmark.rs` - Performance benchmarks

---

## References

- [SQLx Documentation](https://docs.rs/sqlx/)
- [PostgreSQL Performance Tips](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [Database Connection Pooling Best Practices](https://www.postgresql.org/docs/current/runtime-config-connection.html)

