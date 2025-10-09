//! Database performance benchmarks
//!
//! Compares LibSQL vs PostgreSQL performance for common operations

use agent_mem_config::DatabaseConfig;
use agent_mem_core::storage::factory::RepositoryFactory;
use agent_mem_core::storage::models::{Agent, Organization, User};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;

fn benchmark_user_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("user_creation");
    
    // LibSQL benchmark
    group.bench_function(BenchmarkId::new("libsql", "single_user"), |b| {
        b.to_async(&rt).iter(|| async {
            let config = DatabaseConfig::libsql(":memory:");
            let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
            
            // Create organization
            let org = Organization::new("Test Org".to_string());
            let org = repositories.organizations.create(&org).await.unwrap();
            
            // Create user
            let user = User::new(
                org.id.clone(),
                "Test User".to_string(),
                "test@example.com".to_string(),
                "password_hash".to_string(),
                "UTC".to_string(),
            );
            
            black_box(repositories.users.create(&user).await.unwrap());
        });
    });
    
    group.finish();
}

fn benchmark_user_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("user_read");
    
    // Setup: Create a user first
    let (repositories, user_id) = rt.block_on(async {
        let config = DatabaseConfig::libsql(":memory:");
        let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
        
        let org = Organization::new("Test Org".to_string());
        let org = repositories.organizations.create(&org).await.unwrap();
        
        let user = User::new(
            org.id.clone(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password_hash".to_string(),
            "UTC".to_string(),
        );
        let user = repositories.users.create(&user).await.unwrap();
        
        (repositories, user.id)
    });
    
    // LibSQL benchmark
    group.bench_function(BenchmarkId::new("libsql", "find_by_id"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(repositories.users.find_by_id(&user_id).await.unwrap());
        });
    });
    
    group.finish();
}

fn benchmark_agent_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("agent_creation");
    
    // LibSQL benchmark
    group.bench_function(BenchmarkId::new("libsql", "single_agent"), |b| {
        b.to_async(&rt).iter(|| async {
            let config = DatabaseConfig::libsql(":memory:");
            let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
            
            let org = Organization::new("Test Org".to_string());
            let org = repositories.organizations.create(&org).await.unwrap();
            
            let agent = Agent::new(org.id.clone(), Some("Test Agent".to_string()));
            
            black_box(repositories.agents.create(&agent).await.unwrap());
        });
    });
    
    group.finish();
}

fn benchmark_bulk_user_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("bulk_user_creation");
    
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("libsql", count), count, |b, &count| {
            b.to_async(&rt).iter(|| async move {
                let config = DatabaseConfig::libsql(":memory:");
                let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
                
                let org = Organization::new("Test Org".to_string());
                let org = repositories.organizations.create(&org).await.unwrap();
                
                for i in 0..count {
                    let user = User::new(
                        org.id.clone(),
                        format!("User {}", i),
                        format!("user{}@example.com", i),
                        "password_hash".to_string(),
                        "UTC".to_string(),
                    );
                    repositories.users.create(&user).await.unwrap();
                }
            });
        });
    }
    
    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_operations");
    
    group.bench_function(BenchmarkId::new("libsql", "10_concurrent_users"), |b| {
        b.to_async(&rt).iter(|| async {
            let config = DatabaseConfig::libsql(":memory:");
            let repositories = std::sync::Arc::new(
                RepositoryFactory::create_repositories(&config).await.unwrap()
            );
            
            let org = Organization::new("Test Org".to_string());
            let org = repositories.organizations.create(&org).await.unwrap();
            
            let mut handles = vec![];
            for i in 0..10 {
                let repos = repositories.clone();
                let org_id = org.id.clone();
                
                let handle = tokio::spawn(async move {
                    let user = User::new(
                        org_id,
                        format!("User {}", i),
                        format!("user{}@example.com", i),
                        "password_hash".to_string(),
                        "UTC".to_string(),
                    );
                    repos.users.create(&user).await
                });
                
                handles.push(handle);
            }
            
            for handle in handles {
                black_box(handle.await.unwrap().unwrap());
            }
        });
    });
    
    group.finish();
}

fn benchmark_query_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("query_operations");
    
    // Setup: Create multiple users
    let (repositories, org_id) = rt.block_on(async {
        let config = DatabaseConfig::libsql(":memory:");
        let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
        
        let org = Organization::new("Test Org".to_string());
        let org = repositories.organizations.create(&org).await.unwrap();
        
        // Create 50 users
        for i in 0..50 {
            let user = User::new(
                org.id.clone(),
                format!("User {}", i),
                format!("user{}@example.com", i),
                "password_hash".to_string(),
                "UTC".to_string(),
            );
            repositories.users.create(&user).await.unwrap();
        }
        
        (repositories, org.id)
    });
    
    // Benchmark find_by_organization_id
    group.bench_function(BenchmarkId::new("libsql", "find_by_organization_id"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(repositories.users.find_by_organization_id(&org_id).await.unwrap());
        });
    });
    
    // Benchmark find_by_email
    group.bench_function(BenchmarkId::new("libsql", "find_by_email"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(repositories.users.find_by_email("user25@example.com", &org_id).await.unwrap());
        });
    });
    
    // Benchmark email_exists
    group.bench_function(BenchmarkId::new("libsql", "email_exists"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(repositories.users.email_exists("user25@example.com", &org_id).await.unwrap());
        });
    });
    
    group.finish();
}

fn benchmark_update_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("update_operations");
    
    // Setup: Create a user
    let (repositories, user_id) = rt.block_on(async {
        let config = DatabaseConfig::libsql(":memory:");
        let repositories = RepositoryFactory::create_repositories(&config).await.unwrap();
        
        let org = Organization::new("Test Org".to_string());
        let org = repositories.organizations.create(&org).await.unwrap();
        
        let user = User::new(
            org.id.clone(),
            "Test User".to_string(),
            "test@example.com".to_string(),
            "password_hash".to_string(),
            "UTC".to_string(),
        );
        let user = repositories.users.create(&user).await.unwrap();
        
        (repositories, user.id)
    });
    
    // Benchmark update_password
    group.bench_function(BenchmarkId::new("libsql", "update_password"), |b| {
        b.to_async(&rt).iter(|| async {
            black_box(repositories.users.update_password(&user_id, "new_password_hash").await.unwrap());
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_user_creation,
    benchmark_user_read,
    benchmark_agent_creation,
    benchmark_bulk_user_creation,
    benchmark_concurrent_operations,
    benchmark_query_operations,
    benchmark_update_operations,
);

criterion_main!(benches);

