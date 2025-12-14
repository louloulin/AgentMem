//! Query performance analysis and optimization
//!
//! This module provides tools for analyzing and optimizing database queries:
//! - EXPLAIN ANALYZE support
//! - Slow query logging
//! - Query statistics
//! - Index recommendations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{CoreError, CoreResult};

/// Query execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query_hash: String,
    pub query_text: String,
    pub execution_count: u64,
    pub total_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub avg_time_ms: f64,
    pub last_executed: DateTime<Utc>,
}

/// Query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub query: String,
    pub plan: String,
    pub execution_time_ms: f64,
    pub planning_time_ms: f64,
    pub total_cost: f64,
    pub rows: i64,
}

/// Slow query record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQuery {
    pub query: String,
    pub execution_time_ms: f64,
    pub timestamp: DateTime<Utc>,
    pub parameters: Option<String>,
}

/// Query analyzer
pub struct QueryAnalyzer {
    pool: PgPool,
    stats: Arc<RwLock<HashMap<String, QueryStats>>>,
    slow_queries: Arc<RwLock<Vec<SlowQuery>>>,
    slow_query_threshold_ms: f64,
}

impl QueryAnalyzer {
    pub fn new(pool: PgPool, slow_query_threshold_ms: f64) -> Self {
        Self {
            pool,
            stats: Arc::new(RwLock::new(HashMap::new())),
            slow_queries: Arc::new(RwLock::new(Vec::new())),
            slow_query_threshold_ms,
        }
    }

    /// Execute EXPLAIN ANALYZE on a query
    pub async fn explain_analyze(&self, query: &str) -> CoreResult<QueryPlan> {
        let explain_query = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {}", query);

        let row = sqlx::query(&explain_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to explain query: {}", e)))?;

        let plan_json: serde_json::Value = row
            .try_get(0)
            .map_err(|e| CoreError::Database(format!("Failed to parse explain result: {}", e)))?;

        let plan_array = plan_json
            .as_array()
            .ok_or_else(|| CoreError::Database("Invalid explain result format".to_string()))?;

        let plan_obj = &plan_array[0];
        let plan_node = &plan_obj["Plan"];

        let execution_time = plan_obj["Execution Time"].as_f64().unwrap_or(0.0);
        let planning_time = plan_obj["Planning Time"].as_f64().unwrap_or(0.0);
        let total_cost = plan_node["Total Cost"].as_f64().unwrap_or(0.0);
        let rows = plan_node["Plan Rows"].as_i64().unwrap_or(0);

        Ok(QueryPlan {
            query: query.to_string(),
            plan: serde_json::to_string_pretty(&plan_json)
                .unwrap_or_else(|_| "Failed to format plan".to_string()),
            execution_time_ms: execution_time,
            planning_time_ms: planning_time,
            total_cost,
            rows,
        })
    }

    /// Record query execution
    pub async fn record_execution(&self, query: &str, execution_time_ms: f64) {
        let query_hash = self.hash_query(query);

        let mut stats = self.stats.write().await;

        stats
            .entry(query_hash.clone())
            .and_modify(|s| {
                s.execution_count += 1;
                s.total_time_ms += execution_time_ms;
                s.min_time_ms = s.min_time_ms.min(execution_time_ms);
                s.max_time_ms = s.max_time_ms.max(execution_time_ms);
                s.avg_time_ms = s.total_time_ms / s.execution_count as f64;
                s.last_executed = Utc::now();
            })
            .or_insert_with(|| QueryStats {
                query_hash: query_hash.clone(),
                query_text: query.to_string(),
                execution_count: 1,
                total_time_ms: execution_time_ms,
                min_time_ms: execution_time_ms,
                max_time_ms: execution_time_ms,
                avg_time_ms: execution_time_ms,
                last_executed: Utc::now(),
            });

        // Record slow queries
        if execution_time_ms > self.slow_query_threshold_ms {
            let mut slow_queries = self.slow_queries.write().await;
            slow_queries.push(SlowQuery {
                query: query.to_string(),
                execution_time_ms,
                timestamp: Utc::now(),
                parameters: None,
            });

            // Keep only last 100 slow queries
            if slow_queries.len() > 100 {
                slow_queries.remove(0);
            }

            tracing::warn!("Slow query detected ({}ms): {}", execution_time_ms, query);
        }
    }

    /// Get query statistics
    pub async fn get_stats(&self) -> Vec<QueryStats> {
        let stats = self.stats.read().await;
        stats.values().cloned().collect()
    }

    /// Get slow queries
    pub async fn get_slow_queries(&self) -> Vec<SlowQuery> {
        self.slow_queries.read().await.clone()
    }

    /// Get top N slowest queries by average time
    pub async fn get_slowest_queries(&self, limit: usize) -> Vec<QueryStats> {
        let stats = self.stats.read().await;
        let mut queries: Vec<QueryStats> = stats.values().cloned().collect();
        queries.sort_by(|a, b| b.avg_time_ms.partial_cmp(&a.avg_time_ms)
            .unwrap_or(std::cmp::Ordering::Equal));
        queries.truncate(limit);
        queries
    }

    /// Get top N most frequent queries
    pub async fn get_most_frequent_queries(&self, limit: usize) -> Vec<QueryStats> {
        let stats = self.stats.read().await;
        let mut queries: Vec<QueryStats> = stats.values().cloned().collect();
        queries.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        queries.truncate(limit);
        queries
    }

    /// Clear statistics
    pub async fn clear_stats(&self) {
        self.stats.write().await.clear();
        self.slow_queries.write().await.clear();
    }

    /// Get index recommendations
    pub async fn get_index_recommendations(&self) -> CoreResult<Vec<IndexRecommendation>> {
        // Analyze pg_stat_user_tables for missing indexes
        let rows = sqlx::query(
            r#"
            SELECT
                schemaname,
                tablename,
                seq_scan,
                seq_tup_read,
                idx_scan,
                idx_tup_fetch,
                n_tup_ins,
                n_tup_upd,
                n_tup_del
            FROM pg_stat_user_tables
            WHERE schemaname = 'public'
            ORDER BY seq_scan DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get table stats: {}", e)))?;

        let mut recommendations = Vec::new();

        for row in rows {
            let table_name: String = row.get("tablename");
            let seq_scan: i64 = row.get("seq_scan");
            let idx_scan: Option<i64> = row.get("idx_scan");

            // If table has many sequential scans but few index scans, recommend index
            if seq_scan > 1000 && idx_scan.unwrap_or(0) < seq_scan / 10 {
                recommendations.push(IndexRecommendation {
                    table_name: table_name.clone(),
                    recommendation: format!(
                        "Table '{}' has {} sequential scans but only {} index scans. Consider adding indexes.",
                        table_name, seq_scan, idx_scan.unwrap_or(0)
                    ),
                    priority: if seq_scan > 10000 { "high" } else { "medium" }.to_string(),
                });
            }
        }

        Ok(recommendations)
    }

    /// Get unused indexes
    pub async fn get_unused_indexes(&self) -> CoreResult<Vec<UnusedIndex>> {
        let rows = sqlx::query(
            r#"
            SELECT
                schemaname,
                tablename,
                indexname,
                idx_scan
            FROM pg_stat_user_indexes
            WHERE schemaname = 'public'
            AND idx_scan = 0
            ORDER BY tablename, indexname
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get index stats: {}", e)))?;

        let unused: Vec<UnusedIndex> = rows
            .into_iter()
            .map(|row| UnusedIndex {
                schema_name: row.get("schemaname"),
                table_name: row.get("tablename"),
                index_name: row.get("indexname"),
                scans: row.get("idx_scan"),
            })
            .collect();

        Ok(unused)
    }

    /// Hash a query for tracking
    fn hash_query(&self, query: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Normalize query by removing extra whitespace
        let normalized = query.split_whitespace().collect::<Vec<_>>().join(" ");
        normalized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Index recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRecommendation {
    pub table_name: String,
    pub recommendation: String,
    pub priority: String,
}

/// Unused index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedIndex {
    pub schema_name: String,
    pub table_name: String,
    pub index_name: String,
    pub scans: i64,
}
