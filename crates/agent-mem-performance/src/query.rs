//! Query optimization and execution planning
//!
//! This module provides intelligent query optimization capabilities
//! to improve search performance and reduce resource usage.

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Query optimizer
pub struct QueryOptimizer {
    enabled: bool,
    query_cache: Arc<RwLock<HashMap<String, CachedQueryPlan>>>,
    statistics: Arc<RwLock<QueryStatistics>>,
}

/// Query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub query_id: String,
    pub estimated_cost: f64,
    pub execution_steps: Vec<ExecutionStep>,
    pub optimization_hints: Vec<OptimizationHint>,
    pub cache_strategy: CacheStrategy,
}

/// Execution step in a query plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_type: StepType,
    pub estimated_time_ms: f64,
    pub estimated_memory_mb: f64,
    pub parallelizable: bool,
    pub dependencies: Vec<String>,
}

/// Types of execution steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    VectorSearch,
    FilterApplication,
    ResultRanking,
    DataRetrieval,
    PostProcessing,
}

/// Optimization hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationHint {
    UseIndex(String),
    PrefilterResults,
    BatchOperations,
    ParallelExecution,
    CacheResults,
    SkipExpensiveOperations,
}

/// Cache strategy for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    NoCache,
    ShortTerm(Duration),
    LongTerm(Duration),
    Adaptive,
}

/// Cached query plan
#[derive(Debug, Clone)]
struct CachedQueryPlan {
    plan: QueryPlan,
    created_at: Instant,
    hit_count: u64,
    average_execution_time_ms: f64,
}

/// Query statistics
#[derive(Debug, Clone, Default)]
struct QueryStatistics {
    total_queries: u64,
    optimized_queries: u64,
    cache_hits: u64,
    average_optimization_time_ms: f64,
    performance_improvement_ratio: f64,
}

impl QueryOptimizer {
    /// Create a new query optimizer
    pub fn new(enabled: bool) -> Result<Self> {
        let optimizer = Self {
            enabled,
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(QueryStatistics::default())),
        };

        if enabled {
            info!("Query optimizer initialized");
        }

        Ok(optimizer)
    }

    /// Optimize a query and return execution plan
    pub async fn optimize_query(&self, query: &QueryRequest) -> Result<QueryPlan> {
        if !self.enabled {
            return Ok(self.create_default_plan(query));
        }

        let start_time = Instant::now();
        let query_hash = self.hash_query(query);

        // Check cache first
        if let Some(cached_plan) = self.get_cached_plan(&query_hash).await {
            self.update_cache_hit_stats().await;
            return Ok(cached_plan.plan);
        }

        // Create optimized plan
        let plan = self.create_optimized_plan(query).await?;

        // Cache the plan
        self.cache_plan(query_hash, plan.clone(), start_time.elapsed())
            .await;

        // Update statistics
        self.update_optimization_stats(start_time.elapsed()).await;

        Ok(plan)
    }

    /// Execute a query with the optimized plan
    pub async fn execute_query<F, Fut, T>(&self, plan: &QueryPlan, executor: F) -> Result<T>
    where
        F: FnOnce(&QueryPlan) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();

        debug!(
            "Executing query plan {} with {} steps",
            plan.query_id,
            plan.execution_steps.len()
        );

        let result = executor(plan).await;

        let execution_time = start_time.elapsed();
        self.update_execution_stats(&plan.query_id, execution_time, result.is_ok())
            .await;

        result
    }

    /// Get optimizer statistics
    pub async fn get_statistics(&self) -> Result<OptimizerStats> {
        let stats = self.statistics.read().await;
        Ok(OptimizerStats {
            total_queries: stats.total_queries,
            optimized_queries: stats.optimized_queries,
            cache_hits: stats.cache_hits,
            cache_hit_rate: if stats.total_queries > 0 {
                stats.cache_hits as f64 / stats.total_queries as f64
            } else {
                0.0
            },
            average_optimization_time_ms: stats.average_optimization_time_ms,
            performance_improvement_ratio: stats.performance_improvement_ratio,
        })
    }

    /// Clear query cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.query_cache.write().await.clear();
        info!("Query cache cleared");
        Ok(())
    }

    async fn create_optimized_plan(&self, query: &QueryRequest) -> Result<QueryPlan> {
        let mut steps = Vec::new();
        let mut hints = Vec::new();
        let mut estimated_cost = 0.0;

        // Analyze query characteristics
        let query_complexity = self.analyze_query_complexity(query);

        // Vector search step
        if query.has_vector_search() {
            let vector_step = ExecutionStep {
                step_type: StepType::VectorSearch,
                estimated_time_ms: self.estimate_vector_search_time(query),
                estimated_memory_mb: self.estimate_vector_search_memory(query),
                parallelizable: true,
                dependencies: vec![],
            };
            estimated_cost += vector_step.estimated_time_ms;
            steps.push(vector_step);

            // Add vector search optimizations
            if query.result_limit() > 1000 {
                hints.push(OptimizationHint::PrefilterResults);
            }
            if query.has_filters() {
                hints.push(OptimizationHint::UseIndex("filter_index".to_string()));
            }
        }

        // Filter application step
        if query.has_filters() {
            let filter_step = ExecutionStep {
                step_type: StepType::FilterApplication,
                estimated_time_ms: self.estimate_filter_time(query),
                estimated_memory_mb: 10.0,
                parallelizable: false,
                dependencies: if query.has_vector_search() {
                    vec!["vector_search".to_string()]
                } else {
                    vec![]
                },
            };
            estimated_cost += filter_step.estimated_time_ms;
            steps.push(filter_step);
        }

        // Result ranking step
        let ranking_step = ExecutionStep {
            step_type: StepType::ResultRanking,
            estimated_time_ms: query.result_limit() as f64 * 0.1,
            estimated_memory_mb: 5.0,
            parallelizable: true,
            dependencies: vec!["vector_search".to_string()],
        };
        estimated_cost += ranking_step.estimated_time_ms;
        steps.push(ranking_step);

        // Add general optimization hints
        if query_complexity > 0.7 {
            hints.push(OptimizationHint::ParallelExecution);
            hints.push(OptimizationHint::BatchOperations);
        }

        if self.should_cache_results(query) {
            hints.push(OptimizationHint::CacheResults);
        }

        // Determine cache strategy
        let cache_strategy = if query.is_frequent() {
            CacheStrategy::LongTerm(Duration::from_secs(3600))
        } else if query.is_expensive() {
            CacheStrategy::ShortTerm(Duration::from_secs(300))
        } else {
            CacheStrategy::Adaptive
        };

        Ok(QueryPlan {
            query_id: uuid::Uuid::new_v4().to_string(),
            estimated_cost,
            execution_steps: steps,
            optimization_hints: hints,
            cache_strategy,
        })
    }

    fn create_default_plan(&self, query: &QueryRequest) -> QueryPlan {
        QueryPlan {
            query_id: uuid::Uuid::new_v4().to_string(),
            estimated_cost: 100.0,
            execution_steps: vec![
                ExecutionStep {
                    step_type: StepType::VectorSearch,
                    estimated_time_ms: 50.0,
                    estimated_memory_mb: 20.0,
                    parallelizable: false,
                    dependencies: vec![],
                },
                ExecutionStep {
                    step_type: StepType::ResultRanking,
                    estimated_time_ms: 10.0,
                    estimated_memory_mb: 5.0,
                    parallelizable: false,
                    dependencies: vec!["vector_search".to_string()],
                },
            ],
            optimization_hints: vec![],
            cache_strategy: CacheStrategy::NoCache,
        }
    }

    fn analyze_query_complexity(&self, query: &QueryRequest) -> f64 {
        let mut complexity = 0.0;

        if query.has_vector_search() {
            complexity += 0.3;
        }
        if query.has_filters() {
            complexity += 0.2 * query.filter_count() as f64;
        }
        if query.result_limit() > 100 {
            complexity += 0.1;
        }
        if query.has_aggregations() {
            complexity += 0.4;
        }

        complexity.min(1.0)
    }

    fn estimate_vector_search_time(&self, query: &QueryRequest) -> f64 {
        // Simplified estimation based on result limit and vector dimensions
        let base_time = 20.0;
        let limit_factor = (query.result_limit() as f64).log10() * 5.0;
        let dimension_factor = query.vector_dimensions().unwrap_or(1536) as f64 / 1536.0;

        base_time + limit_factor + dimension_factor * 10.0
    }

    fn estimate_vector_search_memory(&self, query: &QueryRequest) -> f64 {
        // Estimate memory usage in MB
        let base_memory = 10.0;
        let result_memory = query.result_limit() as f64 * 0.001; // 1KB per result
        let vector_memory =
            query.vector_dimensions().unwrap_or(1536) as f64 * 4.0 / 1024.0 / 1024.0; // 4 bytes per dimension

        base_memory + result_memory + vector_memory
    }

    fn estimate_filter_time(&self, query: &QueryRequest) -> f64 {
        query.filter_count() as f64 * 2.0
    }

    fn should_cache_results(&self, query: &QueryRequest) -> bool {
        query.is_frequent() || query.is_expensive()
    }

    fn hash_query(&self, query: &QueryRequest) -> String {
        // Simplified query hashing
        format!("{query:?}")
    }

    async fn get_cached_plan(&self, query_hash: &str) -> Option<CachedQueryPlan> {
        let cache = self.query_cache.read().await;
        cache.get(query_hash).cloned()
    }

    async fn cache_plan(&self, query_hash: String, plan: QueryPlan, optimization_time: Duration) {
        let cached_plan = CachedQueryPlan {
            plan,
            created_at: Instant::now(),
            hit_count: 0,
            average_execution_time_ms: 0.0,
        };

        let mut cache = self.query_cache.write().await;
        cache.insert(query_hash, cached_plan);
    }

    async fn update_cache_hit_stats(&self) {
        let mut stats = self.statistics.write().await;
        stats.cache_hits += 1;
        stats.total_queries += 1;
    }

    async fn update_optimization_stats(&self, optimization_time: Duration) {
        let mut stats = self.statistics.write().await;
        stats.optimized_queries += 1;
        stats.total_queries += 1;

        let optimization_time_ms = optimization_time.as_millis() as f64;
        stats.average_optimization_time_ms = (stats.average_optimization_time_ms
            * (stats.optimized_queries - 1) as f64
            + optimization_time_ms)
            / stats.optimized_queries as f64;
    }

    async fn update_execution_stats(
        &self,
        query_id: &str,
        execution_time: Duration,
        success: bool,
    ) {
        // Update execution statistics for the query plan
        debug!(
            "Query {} executed in {:?}, success: {}",
            query_id, execution_time, success
        );
    }
}

/// Query request structure
#[derive(Debug, Clone)]
pub struct QueryRequest {
    pub vector: Option<Vec<f32>>,
    pub filters: HashMap<String, String>,
    pub limit: usize,
    pub aggregations: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl QueryRequest {
    pub fn has_vector_search(&self) -> bool {
        self.vector.is_some()
    }

    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    pub fn result_limit(&self) -> usize {
        self.limit
    }

    pub fn has_aggregations(&self) -> bool {
        !self.aggregations.is_empty()
    }

    pub fn vector_dimensions(&self) -> Option<usize> {
        self.vector.as_ref().map(|v| v.len())
    }

    pub fn is_frequent(&self) -> bool {
        // Simplified frequency detection
        self.metadata
            .get("frequency")
            .is_some_and(|f| f == "high")
    }

    pub fn is_expensive(&self) -> bool {
        self.limit > 1000 || self.has_aggregations() || self.filter_count() > 5
    }
}

/// Optimizer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerStats {
    pub total_queries: u64,
    pub optimized_queries: u64,
    pub cache_hits: u64,
    pub cache_hit_rate: f64,
    pub average_optimization_time_ms: f64,
    pub performance_improvement_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_optimizer_creation() {
        let optimizer = QueryOptimizer::new(true);
        assert!(optimizer.is_ok());
    }

    #[tokio::test]
    async fn test_query_optimization() {
        let optimizer = QueryOptimizer::new(true).unwrap();

        let query = QueryRequest {
            vector: Some(vec![0.1; 1536]),
            filters: HashMap::new(),
            limit: 10,
            aggregations: vec![],
            metadata: HashMap::new(),
        };

        let plan = optimizer.optimize_query(&query).await;
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert!(!plan.execution_steps.is_empty());
        assert!(plan.estimated_cost > 0.0);
    }

    #[tokio::test]
    async fn test_query_complexity_analysis() {
        let optimizer = QueryOptimizer::new(true).unwrap();

        let simple_query = QueryRequest {
            vector: Some(vec![0.1; 100]),
            filters: HashMap::new(),
            limit: 10,
            aggregations: vec![],
            metadata: HashMap::new(),
        };

        let complexity = optimizer.analyze_query_complexity(&simple_query);
        assert!(complexity > 0.0 && complexity <= 1.0);
    }

    #[tokio::test]
    async fn test_optimizer_statistics() {
        let optimizer = QueryOptimizer::new(true).unwrap();

        let stats = optimizer.get_statistics().await.unwrap();
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.cache_hits, 0);
    }
}
