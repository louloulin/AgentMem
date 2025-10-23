//! Integration tests for database migration, pool management, and query optimization
//!
//! These tests verify:
//! - Migration version management
//! - Connection pool optimization
//! - Query performance analysis
//! - Slow query detection
//! - Index recommendations

#![cfg(feature = "postgres")]

use agent_mem_core::storage::{
    migration_manager::MigrationManager,
    pool_manager::{PoolConfig, PoolManager},
    query_analyzer::QueryAnalyzer,
};
use sqlx::PgPool;

/// Helper function to get test database URL
fn get_test_db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost:5432/agentmem_test".to_string()
    })
}

/// Test migration manager initialization and version tracking
#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_migration_manager() {
    let pool = PgPool::connect(&get_test_db_url())
        .await
        .expect("Failed to connect to database");

    let manager = MigrationManager::new(pool.clone());

    // Initialize migration tracking
    manager
        .init()
        .await
        .expect("Failed to initialize migration manager");

    // Check current version (should be None initially)
    let version = manager
        .current_version()
        .await
        .expect("Failed to get current version");
    println!("Current schema version: {:?}", version);

    // Get pending migrations
    let pending = manager
        .pending_migrations()
        .await
        .expect("Failed to get pending migrations");
    println!("Pending migrations: {:?}", pending);

    // Run all pending migrations
    if !pending.is_empty() {
        manager
            .migrate_up()
            .await
            .expect("Failed to run migrations");
        println!("✅ All migrations applied successfully");
    }

    // Verify version after migration
    let new_version = manager
        .current_version()
        .await
        .expect("Failed to get current version");
    println!("New schema version: {:?}", new_version);
    assert!(new_version.is_some());

    // Get applied migrations
    let applied = manager
        .applied_migrations()
        .await
        .expect("Failed to get applied migrations");
    println!("Applied migrations: {} total", applied.len());
    for migration in &applied {
        println!(
            "  - Version {}: {} (applied at {})",
            migration.version, migration.name, migration.applied_at
        );
    }

    // Test rollback (optional - be careful in production!)
    // manager.migrate_down(1).await.expect("Failed to rollback migration");
    // println!("✅ Rollback successful");

    pool.close().await;
}

/// Test connection pool management and statistics
#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_pool_manager() {
    let config = PoolConfig::development(get_test_db_url());

    let manager = PoolManager::new(config)
        .await
        .expect("Failed to create pool manager");

    // Check pool health
    let is_healthy = manager.health_check().await.expect("Health check failed");
    assert!(is_healthy, "Pool should be healthy");
    println!("✅ Pool health check passed");

    // Get pool statistics
    let stats = manager.stats().await;
    println!("Pool statistics:");
    println!("  - Total connections: {}", stats.total_connections);
    println!("  - Idle connections: {}", stats.idle_connections);
    println!("  - Active connections: {}", stats.active_connections);
    println!("  - Total acquired: {}", stats.total_acquired);
    println!("  - Total released: {}", stats.total_released);
    println!("  - Total timeouts: {}", stats.total_timeouts);
    println!("  - Total errors: {}", stats.total_errors);

    // Get detailed metrics
    let metrics = manager.metrics().await;
    println!("Pool metrics:");
    println!("  - Size: {}/{}", metrics.size, metrics.max_size);
    println!("  - Utilization: {:.2}%", metrics.utilization);
    println!("  - Status: {}", metrics.status());
    assert!(metrics.is_healthy(), "Pool should be healthy");

    // Test connection acquisition
    let conn = manager
        .acquire()
        .await
        .expect("Failed to acquire connection");
    println!("✅ Successfully acquired connection");
    drop(conn);

    // Test retry mechanism
    let result = manager
        .execute_with_retry(3, |pool| async move {
            sqlx::query("SELECT 1")
                .fetch_one(&pool)
                .await
                .map_err(|e| agent_mem_core::CoreError::Database(e.to_string()))
        })
        .await;
    assert!(result.is_ok(), "Retry operation should succeed");
    println!("✅ Retry mechanism works");

    manager.close().await;
}

/// Test query analyzer and performance monitoring
#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_query_analyzer() {
    let pool = PgPool::connect(&get_test_db_url())
        .await
        .expect("Failed to connect to database");

    let analyzer = QueryAnalyzer::new(pool.clone(), 50.0); // 50ms threshold

    // Test EXPLAIN ANALYZE
    let query = "SELECT * FROM organizations LIMIT 10";
    let plan = analyzer
        .explain_analyze(query)
        .await
        .expect("Failed to explain query");

    println!("Query plan:");
    println!("  - Query: {}", plan.query);
    println!("  - Execution time: {:.2}ms", plan.execution_time_ms);
    println!("  - Planning time: {:.2}ms", plan.planning_time_ms);
    println!("  - Total cost: {:.2}", plan.total_cost);
    println!("  - Estimated rows: {}", plan.rows);
    println!("  - Plan:\n{}", plan.plan);

    // Record some query executions
    analyzer
        .record_execution("SELECT * FROM users WHERE id = $1", 10.5)
        .await;
    analyzer
        .record_execution("SELECT * FROM users WHERE id = $1", 12.3)
        .await;
    analyzer
        .record_execution("SELECT * FROM agents WHERE organization_id = $1", 150.0)
        .await; // Slow query

    // Get query statistics
    let stats = analyzer.get_stats().await;
    println!("\nQuery statistics: {} unique queries", stats.len());
    for stat in &stats {
        println!("  - Query: {}", stat.query_text);
        println!("    Executions: {}", stat.execution_count);
        println!("    Avg time: {:.2}ms", stat.avg_time_ms);
        println!(
            "    Min/Max: {:.2}ms / {:.2}ms",
            stat.min_time_ms, stat.max_time_ms
        );
    }

    // Get slow queries
    let slow_queries = analyzer.get_slow_queries().await;
    println!("\nSlow queries: {} total", slow_queries.len());
    for slow in &slow_queries {
        println!("  - Query: {}", slow.query);
        println!("    Time: {:.2}ms", slow.execution_time_ms);
        println!("    Timestamp: {}", slow.timestamp);
    }

    // Get slowest queries
    let slowest = analyzer.get_slowest_queries(5).await;
    println!("\nTop 5 slowest queries:");
    for (i, query) in slowest.iter().enumerate() {
        println!(
            "  {}. {} ({:.2}ms avg)",
            i + 1,
            query.query_text,
            query.avg_time_ms
        );
    }

    // Get most frequent queries
    let frequent = analyzer.get_most_frequent_queries(5).await;
    println!("\nTop 5 most frequent queries:");
    for (i, query) in frequent.iter().enumerate() {
        println!(
            "  {}. {} ({} executions)",
            i + 1,
            query.query_text,
            query.execution_count
        );
    }

    // Get index recommendations
    let recommendations = analyzer
        .get_index_recommendations()
        .await
        .expect("Failed to get recommendations");
    println!("\nIndex recommendations: {} total", recommendations.len());
    for rec in &recommendations {
        println!("  - Table: {}", rec.table_name);
        println!("    Priority: {}", rec.priority);
        println!("    Recommendation: {}", rec.recommendation);
    }

    // Get unused indexes
    let unused = analyzer
        .get_unused_indexes()
        .await
        .expect("Failed to get unused indexes");
    println!("\nUnused indexes: {} total", unused.len());
    for idx in &unused {
        println!(
            "  - {}.{} (scans: {})",
            idx.table_name, idx.index_name, idx.scans
        );
    }

    pool.close().await;
}

/// Test pool configuration presets
#[test]
fn test_pool_config_presets() {
    let url = "postgresql://localhost/test".to_string();

    // Test production config
    let prod = PoolConfig::production(url.clone());
    assert_eq!(prod.min_connections, 5);
    assert_eq!(prod.max_connections, 20);
    assert_eq!(prod.slow_statement_threshold_ms, 50);
    println!(
        "✅ Production config: min={}, max={}",
        prod.min_connections, prod.max_connections
    );

    // Test development config
    let dev = PoolConfig::development(url.clone());
    assert_eq!(dev.min_connections, 1);
    assert_eq!(dev.max_connections, 5);
    assert!(dev.log_statements);
    println!(
        "✅ Development config: min={}, max={}",
        dev.min_connections, dev.max_connections
    );

    // Test high-performance config
    let hp = PoolConfig::high_performance(url.clone());
    assert_eq!(hp.min_connections, 10);
    assert_eq!(hp.max_connections, 50);
    assert_eq!(hp.slow_statement_threshold_ms, 20);
    println!(
        "✅ High-performance config: min={}, max={}",
        hp.min_connections, hp.max_connections
    );
}

/// Test migration checksum calculation
#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_migration_checksum() {
    let pool = PgPool::connect(&get_test_db_url())
        .await
        .expect("Failed to connect to database");

    let manager = MigrationManager::new(pool.clone());
    manager.init().await.expect("Failed to initialize");

    // Get applied migrations and verify checksums
    let applied = manager
        .applied_migrations()
        .await
        .expect("Failed to get migrations");

    for migration in &applied {
        println!(
            "Migration {}: checksum={}",
            migration.version, migration.checksum
        );
        assert!(
            !migration.checksum.is_empty(),
            "Checksum should not be empty"
        );
    }

    pool.close().await;
}
