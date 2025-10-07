//! Query optimization utilities
//!
//! Provides tools for analyzing and optimizing SQL queries

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use std::time::Duration;
use tracing::{debug, warn};

/// Query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    /// Query text
    pub query: String,
    
    /// Execution plan (EXPLAIN output)
    pub plan: String,
    
    /// Estimated cost
    pub estimated_cost: Option<f64>,
    
    /// Estimated rows
    pub estimated_rows: Option<i64>,
    
    /// Actual execution time (if EXPLAIN ANALYZE was used)
    pub execution_time: Option<Duration>,
    
    /// Whether the query uses indexes
    pub uses_indexes: bool,
    
    /// Warnings or suggestions
    pub warnings: Vec<String>,
}

/// Query optimizer
pub struct QueryOptimizer {
    pool: Pool<Postgres>,
}

impl QueryOptimizer {
    /// Create a new query optimizer
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    
    /// Analyze a query and return its execution plan
    pub async fn explain_query(&self, query: &str) -> Result<QueryPlan, sqlx::Error> {
        debug!("Analyzing query: {}", query);
        
        // Run EXPLAIN
        let explain_query = format!("EXPLAIN (FORMAT JSON, VERBOSE) {}", query);
        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await?;
        
        let plan_json: serde_json::Value = row.try_get(0)?;
        let plan_str = serde_json::to_string_pretty(&plan_json).unwrap_or_default();
        
        // Parse plan to extract key information
        let (estimated_cost, estimated_rows, uses_indexes) = Self::parse_plan(&plan_json);
        
        // Generate warnings
        let warnings = Self::generate_warnings(&plan_json, query);
        
        Ok(QueryPlan {
            query: query.to_string(),
            plan: plan_str,
            estimated_cost,
            estimated_rows,
            execution_time: None,
            uses_indexes,
            warnings,
        })
    }
    
    /// Analyze a query with actual execution (EXPLAIN ANALYZE)
    pub async fn explain_analyze_query(&self, query: &str) -> Result<QueryPlan, sqlx::Error> {
        debug!("Analyzing query with execution: {}", query);
        
        // Run EXPLAIN ANALYZE
        let explain_query = format!("EXPLAIN (ANALYZE, FORMAT JSON, VERBOSE) {}", query);
        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await?;
        
        let plan_json: serde_json::Value = row.try_get(0)?;
        let plan_str = serde_json::to_string_pretty(&plan_json).unwrap_or_default();
        
        // Parse plan to extract key information
        let (estimated_cost, estimated_rows, uses_indexes) = Self::parse_plan(&plan_json);
        let execution_time = Self::parse_execution_time(&plan_json);
        
        // Generate warnings
        let warnings = Self::generate_warnings(&plan_json, query);
        
        Ok(QueryPlan {
            query: query.to_string(),
            plan: plan_str,
            estimated_cost,
            estimated_rows,
            execution_time,
            uses_indexes,
            warnings,
        })
    }
    
    /// Check if a query would benefit from an index
    pub async fn suggest_indexes(&self, query: &str) -> Result<Vec<String>, sqlx::Error> {
        let plan = self.explain_query(query).await?;
        
        let mut suggestions = Vec::new();
        
        // Check if query uses sequential scans
        if plan.plan.contains("Seq Scan") {
            suggestions.push("Consider adding an index to avoid sequential scans".to_string());
        }
        
        // Check if query has high cost
        if let Some(cost) = plan.estimated_cost {
            if cost > 1000.0 {
                suggestions.push(format!(
                    "Query has high estimated cost ({}). Consider optimizing.",
                    cost
                ));
            }
        }
        
        // Check if query scans many rows
        if let Some(rows) = plan.estimated_rows {
            if rows > 10000 {
                suggestions.push(format!(
                    "Query scans many rows ({}). Consider adding filters or indexes.",
                    rows
                ));
            }
        }
        
        Ok(suggestions)
    }
    
    /// Parse execution plan to extract key metrics
    fn parse_plan(plan_json: &serde_json::Value) -> (Option<f64>, Option<i64>, bool) {
        let plan_array = plan_json.as_array();
        if plan_array.is_none() {
            return (None, None, false);
        }
        
        let first_plan = &plan_array.unwrap()[0];
        let plan_obj = first_plan.get("Plan");
        if plan_obj.is_none() {
            return (None, None, false);
        }
        
        let plan = plan_obj.unwrap();
        
        // Extract cost
        let estimated_cost = plan
            .get("Total Cost")
            .and_then(|v| v.as_f64());
        
        // Extract rows
        let estimated_rows = plan
            .get("Plan Rows")
            .and_then(|v| v.as_i64());
        
        // Check if uses indexes
        let node_type = plan
            .get("Node Type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let uses_indexes = node_type.contains("Index");
        
        (estimated_cost, estimated_rows, uses_indexes)
    }
    
    /// Parse execution time from EXPLAIN ANALYZE output
    fn parse_execution_time(plan_json: &serde_json::Value) -> Option<Duration> {
        let plan_array = plan_json.as_array()?;
        let first_plan = &plan_array[0];
        let plan = first_plan.get("Plan")?;
        
        let execution_time_ms = plan
            .get("Actual Total Time")
            .and_then(|v| v.as_f64())?;
        
        Some(Duration::from_secs_f64(execution_time_ms / 1000.0))
    }
    
    /// Generate warnings based on execution plan
    fn generate_warnings(plan_json: &serde_json::Value, query: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        let plan_str = serde_json::to_string(plan_json).unwrap_or_default();
        
        // Check for sequential scans
        if plan_str.contains("Seq Scan") {
            warnings.push("Query uses sequential scan. Consider adding an index.".to_string());
        }
        
        // Check for missing WHERE clause
        if !query.to_uppercase().contains("WHERE") && !query.to_uppercase().contains("LIMIT") {
            warnings.push("Query has no WHERE clause or LIMIT. This may scan the entire table.".to_string());
        }
        
        // Check for SELECT *
        if query.to_uppercase().contains("SELECT *") {
            warnings.push("Query uses SELECT *. Consider selecting only needed columns.".to_string());
        }
        
        // Check for missing LIMIT on large tables
        if !query.to_uppercase().contains("LIMIT") && 
           (query.contains("messages") || query.contains("memories")) {
            warnings.push("Query on large table without LIMIT. Consider adding pagination.".to_string());
        }
        
        // Check for N+1 query pattern (multiple similar queries)
        if query.contains("IN (") {
            let in_clause_size = query.matches("IN (").count();
            if in_clause_size > 1 {
                warnings.push("Multiple IN clauses detected. Consider using JOINs instead.".to_string());
            }
        }
        
        warnings
    }
}

/// Query optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    
    /// Description
    pub description: String,
    
    /// Estimated impact (1-10, 10 being highest)
    pub impact: u8,
    
    /// Suggested action
    pub action: String,
}

/// Recommendation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Add an index
    AddIndex,
    
    /// Rewrite query
    RewriteQuery,
    
    /// Add WHERE clause
    AddFilter,
    
    /// Add LIMIT clause
    AddPagination,
    
    /// Use prepared statements
    UsePreparedStatements,
    
    /// Optimize JOIN
    OptimizeJoin,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_plan() {
        let plan_json = serde_json::json!([{
            "Plan": {
                "Node Type": "Index Scan",
                "Total Cost": 123.45,
                "Plan Rows": 100
            }
        }]);
        
        let (cost, rows, uses_indexes) = QueryOptimizer::parse_plan(&plan_json);
        
        assert_eq!(cost, Some(123.45));
        assert_eq!(rows, Some(100));
        assert!(uses_indexes);
    }
    
    #[test]
    fn test_generate_warnings_seq_scan() {
        let plan_json = serde_json::json!([{
            "Plan": {
                "Node Type": "Seq Scan"
            }
        }]);
        
        let warnings = QueryOptimizer::generate_warnings(&plan_json, "SELECT * FROM users");
        
        assert!(warnings.iter().any(|w| w.contains("sequential scan")));
        assert!(warnings.iter().any(|w| w.contains("SELECT *")));
    }
    
    #[test]
    fn test_generate_warnings_no_where() {
        let warnings = QueryOptimizer::generate_warnings(
            &serde_json::json!([{"Plan": {}}]),
            "SELECT id FROM users"
        );
        
        assert!(warnings.iter().any(|w| w.contains("no WHERE clause")));
    }
}

