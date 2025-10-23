//! Performance benchmarks for database operations
//!
//! These benchmarks measure the performance of various database operations
//! to ensure they meet production requirements.

use agent_mem_core::storage::{
    agent_repository::AgentRepository,
    batch::BatchOperations,
    memory_repository::MemoryRepository,
    models::*,
    postgres::PostgresStorage,
    repository::{OrganizationRepository, Repository, UserRepository},
    PostgresConfig,
};
use std::time::Instant;

/// Helper function to get test database URL
fn get_test_db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://agentmem:password@localhost:5432/agentmem_test".to_string()
    })
}

/// Helper function to create test PostgreSQL storage
async fn create_test_storage() -> PostgresStorage {
    let config = PostgresConfig {
        url: get_test_db_url(),
        max_connections: 20,
        connection_timeout: 30,
        query_timeout: 30,
        ssl: false,
    };

    let storage = PostgresStorage::new(config)
        .await
        .expect("Failed to create PostgreSQL storage");

    storage.migrate().await.expect("Failed to run migrations");

    storage
}

/// Benchmark result
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    operations: usize,
    total_time_ms: u128,
    avg_time_ms: f64,
    ops_per_sec: f64,
}

impl BenchmarkResult {
    fn new(name: &str, operations: usize, total_time_ms: u128) -> Self {
        let avg_time_ms = total_time_ms as f64 / operations as f64;
        let ops_per_sec = 1000.0 / avg_time_ms;

        Self {
            name: name.to_string(),
            operations,
            total_time_ms,
            avg_time_ms,
            ops_per_sec,
        }
    }

    fn print(&self) {
        println!("📊 {}", self.name);
        println!("   Operations: {}", self.operations);
        println!("   Total time: {}ms", self.total_time_ms);
        println!("   Avg time: {:.2}ms", self.avg_time_ms);
        println!("   Ops/sec: {:.2}", self.ops_per_sec);
        println!();
    }

    fn check_threshold(&self, max_avg_ms: f64) -> bool {
        if self.avg_time_ms > max_avg_ms {
            println!(
                "❌ FAILED: {} exceeded threshold ({}ms > {}ms)",
                self.name, self.avg_time_ms, max_avg_ms
            );
            false
        } else {
            println!(
                "✅ PASSED: {} within threshold ({}ms <= {}ms)",
                self.name, self.avg_time_ms, max_avg_ms
            );
            true
        }
    }
}

#[tokio::test]
#[ignore] // Requires database
async fn benchmark_crud_operations() {
    println!("\n🚀 Running CRUD Operations Benchmark\n");

    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let agent_repo = AgentRepository::new(pool);

    // Create test organization and user
    let org = Organization::new("Benchmark Org".to_string());
    let org = org_repo.create(&org).await.expect("Failed to create org");

    let user = User::new(
        org.id.clone(),
        "Benchmark User".to_string(),
        "benchmark@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Benchmark: Create agents
    let num_agents = 100;
    let start = Instant::now();

    for i in 0..num_agents {
        let agent = Agent::new(org.id.clone(), Some(format!("Agent {}", i)));
        agent_repo
            .create(&agent)
            .await
            .expect("Failed to create agent");
    }

    let result = BenchmarkResult::new("Create Agent", num_agents, start.elapsed().as_millis());
    result.print();
    assert!(result.check_threshold(50.0), "Create operation too slow");

    // Benchmark: Read agents
    let start = Instant::now();

    for _ in 0..num_agents {
        agent_repo
            .list_by_organization(&org.id, Some(10), Some(0))
            .await
            .expect("Failed to list agents");
    }

    let result = BenchmarkResult::new("List Agents", num_agents, start.elapsed().as_millis());
    result.print();
    assert!(result.check_threshold(30.0), "List operation too slow");

    // Clean up
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn benchmark_batch_operations() {
    println!("\n🚀 Running Batch Operations Benchmark\n");

    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let batch_ops = BatchOperations::new(pool);

    // Create test organization and user
    let org = Organization::new("Benchmark Org".to_string());
    let org = org_repo.create(&org).await.expect("Failed to create org");

    let user = User::new(
        org.id.clone(),
        "Benchmark User".to_string(),
        "benchmark@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Benchmark: Batch insert agents
    let batch_size = 100;
    let agents: Vec<Agent> = (0..batch_size)
        .map(|i| Agent::new(org.id.clone(), Some(format!("Batch Agent {}", i))))
        .collect();

    let start = Instant::now();
    batch_ops
        .batch_insert_agents(&agents)
        .await
        .expect("Failed to batch insert");

    let result = BenchmarkResult::new(
        "Batch Insert Agents",
        batch_size,
        start.elapsed().as_millis(),
    );
    result.print();
    assert!(result.check_threshold(10.0), "Batch insert too slow");

    // Clean up
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn benchmark_memory_operations() {
    println!("\n🚀 Running Memory Operations Benchmark\n");

    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let agent_repo = AgentRepository::new(pool.clone());
    let memory_repo = MemoryRepository::new(pool);

    // Create test data
    let org = Organization::new("Benchmark Org".to_string());
    let org = org_repo.create(&org).await.expect("Failed to create org");

    let user = User::new(
        org.id.clone(),
        "Benchmark User".to_string(),
        "benchmark@example.com".to_string(),
        "hashed_password".to_string(),
        "UTC".to_string(),
    );
    let user = user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    let agent = Agent::new(org.id.clone(), Some("Benchmark Agent".to_string()));
    let agent = agent_repo
        .create(&agent)
        .await
        .expect("Failed to create agent");

    // Benchmark: Create memories
    let num_memories = 100;
    let start = Instant::now();

    for i in 0..num_memories {
        let memory = Memory {
            id: generate_id("memory"),
            organization_id: org.id.clone(),
            user_id: user.id.clone(),
            agent_id: agent.id.clone(),
            content: format!("Memory content {}", i),
            hash: None,
            metadata: serde_json::json!({}),
            score: Some(0.5),
            memory_type: "episodic".to_string(),
            scope: "agent".to_string(),
            level: "short_term".to_string(),
            importance: 0.5,
            access_count: 0,
            last_accessed: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        };

        memory_repo
            .create(&memory)
            .await
            .expect("Failed to create memory");
    }

    let result = BenchmarkResult::new("Create Memory", num_memories, start.elapsed().as_millis());
    result.print();
    assert!(result.check_threshold(50.0), "Memory create too slow");

    // Benchmark: Full-text search
    let num_searches = 50;
    let start = Instant::now();

    for _ in 0..num_searches {
        memory_repo
            .search_fulltext(&agent.id, "content", Some(10))
            .await
            .expect("Failed to search");
    }

    let result = BenchmarkResult::new(
        "Full-text Search",
        num_searches,
        start.elapsed().as_millis(),
    );
    result.print();
    assert!(result.check_threshold(100.0), "Search too slow");

    // Clean up
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn benchmark_concurrent_operations() {
    println!("\n🚀 Running Concurrent Operations Benchmark\n");

    let storage = create_test_storage().await;
    let pool = storage.pool().clone();
    let org_repo = OrganizationRepository::new(pool.clone());

    // Create test organization
    let org = Organization::new("Benchmark Org".to_string());
    let org = org_repo.create(&org).await.expect("Failed to create org");

    // Benchmark: Concurrent reads
    let num_concurrent = 50;
    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..num_concurrent {
        let org_id = org.id.clone();
        let repo = org_repo.clone();

        let handle = tokio::spawn(async move {
            repo.read(&org_id).await.expect("Failed to read org");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    let result = BenchmarkResult::new(
        "Concurrent Reads",
        num_concurrent,
        start.elapsed().as_millis(),
    );
    result.print();
    assert!(result.check_threshold(20.0), "Concurrent reads too slow");

    // Clean up
    org_repo.hard_delete(&org.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires database
async fn benchmark_summary() {
    println!("\n📊 Performance Benchmark Summary\n");
    println!("Target Performance Requirements:");
    println!("  - CRUD operations: < 50ms average");
    println!("  - Batch operations: < 10ms per item");
    println!("  - Search operations: < 100ms");
    println!("  - Concurrent operations: < 20ms per operation");
    println!("\nRun individual benchmarks to see detailed results.");
}
