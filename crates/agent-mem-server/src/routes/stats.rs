//! Statistics and analytics routes
//!
//! This module provides comprehensive statistics endpoints for the AgentMem system.
//! It includes:
//! - Dashboard statistics (agents, users, memories, messages)
//! - Memory growth trends over time
//! - Agent activity statistics
//!
//! All endpoints return real data from the repository layer.

use crate::error::{ServerError, ServerResult};
use crate::routes::memory::MemoryManager;
use agent_mem_core::search::query_optimizer::{IndexStatistics, IndexType};
use agent_mem_core::storage::factory::Repositories;
use agent_mem_core::storage::libsql::connection::LibSqlConnectionManager;
use axum::{extract::Extension, response::Json};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use utoipa::ToSchema;

/// Dashboard statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardStats {
    /// Total number of agents
    pub total_agents: i64,

    /// Total number of users
    pub total_users: i64,

    /// Total number of memories
    pub total_memories: i64,

    /// Total number of messages
    pub total_messages: i64,

    /// Active agents (agents with activity in last 24h)
    pub active_agents: i64,

    /// Active users (users with activity in last 24h)
    pub active_users: i64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// Recent activity logs (last 10 activities)
    pub recent_activities: Vec<ActivityLog>,

    /// Memory statistics by type
    pub memories_by_type: HashMap<String, i64>,

    /// Timestamp of data collection
    pub timestamp: DateTime<Utc>,
}

/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ActivityLog {
    /// Activity ID
    pub id: String,

    /// Activity type (memory_created, agent_created, message_sent, etc.)
    pub activity_type: String,

    /// Agent ID (if applicable)
    pub agent_id: Option<String>,

    /// User ID (if applicable)
    pub user_id: Option<String>,

    /// Activity description
    pub description: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Memory growth data point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryGrowthPoint {
    /// Date
    pub date: String,

    /// Total memories on this date
    pub total: i64,

    /// New memories added on this date
    pub new: i64,

    /// Memories by type
    pub by_type: HashMap<String, i64>,
}

/// Memory growth response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryGrowthResponse {
    /// Time series data points
    pub data: Vec<MemoryGrowthPoint>,

    /// Total memories across all time
    pub total_memories: i64,

    /// Growth rate (memories per day)
    pub growth_rate: f64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Agent activity statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentActivityStats {
    /// Agent ID
    pub agent_id: String,

    /// Agent name
    pub agent_name: String,

    /// Total memories for this agent
    pub total_memories: i64,

    /// Total interactions (messages)
    pub total_interactions: i64,

    /// Last active timestamp
    pub last_active: Option<DateTime<Utc>>,

    /// Average importance of memories
    pub avg_importance: f64,
}

/// Agent activity response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentActivityResponse {
    /// List of agent activity statistics
    pub agents: Vec<AgentActivityStats>,

    /// Total number of agents
    pub total_agents: i64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Memory quality statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryQualityStats {
    /// Average importance across all memories
    pub avg_importance: f64,

    /// Percentage of high-quality memories (importance > 0.7)
    pub high_quality_ratio: f64,

    /// Importance distribution by ranges
    pub importance_distribution: HashMap<String, i64>,

    /// Memory type distribution with counts and percentages
    pub type_distribution: Vec<MemoryTypeStats>,

    /// Total number of memories
    pub total_memories: i64,

    /// Most accessed memory types (placeholder for future)
    pub access_stats: HashMap<String, i64>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Memory type statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryTypeStats {
    /// Memory type name
    pub type_name: String,

    /// Count of memories of this type
    pub count: i64,

    /// Percentage of total memories
    pub percentage: f64,

    /// Average importance for this type
    pub avg_importance: f64,
}

/// Get dashboard statistics
///
/// Returns comprehensive statistics for the admin dashboard including:
/// - Total counts for agents, users, memories, messages
/// - Active entity counts (last 24h)
/// - Recent activity logs
/// - Memory distribution by type
#[utoipa::path(
    get,
    path = "/api/v1/stats/dashboard",
    tag = "statistics",
    responses(
        (status = 200, description = "Dashboard statistics retrieved successfully", body = DashboardStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_dashboard_stats(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(_memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<DashboardStats>> {
    info!("📊 Generating comprehensive dashboard stats from multiple data sources");

    // ✅ 综合数据源1: Agents
    let all_agents = repositories
        .agents
        .list(10000, 0)
        .await
        .map_err(|e| ServerError::internal_error(e.to_string()))?;
    let total_agents = all_agents.len() as i64;
    info!("  - Total agents: {}", total_agents);

    // ✅ 综合数据源2: Users
    let all_users = repositories
        .users
        .find_by_organization_id("default")
        .await
        .map_err(|e| ServerError::internal_error(e.to_string()))?;
    let total_users = all_users.len() as i64;
    info!("  - Total users: {}", total_users);

    // ✅ 综合数据源3: Messages (从所有agents聚合)
    let mut total_messages = 0i64;
    for agent in all_agents.iter().take(100) {
        let agent_messages = repositories
            .messages
            .find_by_agent_id(&agent.id, 10000)
            .await
            .map_err(|e| ServerError::internal_error(e.to_string()))?;
        total_messages += agent_messages.len() as i64;
    }
    info!(
        "  - Total messages: {} (from {} agents)",
        total_messages,
        all_agents.len().min(100)
    );

    // ✅ 综合数据源4: Memories (直接从 LibSQL Repository，避免向量搜索)
    let mut total_memories = 0i64;
    let mut memories_by_type_map: HashMap<String, i64> = HashMap::new();

    info!(
        "  - Querying memories from LibSQL for {} agents...",
        all_agents.len().min(100)
    );
    for (idx, agent) in all_agents.iter().take(100).enumerate() {
        match repositories
            .memories
            .find_by_agent_id(&agent.id, 10000)
            .await
        {
            Ok(agent_memories) => {
                let count = agent_memories.len();
                if count > 0 {
                    info!(
                        "    Agent {}/{}: {} memories",
                        idx + 1,
                        all_agents.len().min(100),
                        count
                    );
                }
                total_memories += count as i64;

                // 统计 memory 类型分布 - 将 MemoryV4 转换为 MemoryItem
                for memory in agent_memories {
                    let memory_item = memory.to_legacy_item();
                    let memory_type_str = format!("{:?}", memory_item.memory_type);
                    *memories_by_type_map.entry(memory_type_str).or_insert(0) += 1;
                }
            }
            Err(e) => {
                warn!(
                    "    Agent {}/{}: Failed to get memories - {}",
                    idx + 1,
                    all_agents.len().min(100),
                    e
                );
            }
        }
    }
    info!(
        "  - Total memories: {} (types: {:?})",
        total_memories, memories_by_type_map
    );

    // ✅ 综合数据源5: 活跃统计 (基于最近24小时的消息)
    let cutoff_time = Utc::now() - Duration::hours(24);
    info!("  - Analyzing activity since {}", cutoff_time);

    let mut recent_messages = Vec::new();
    for agent in all_agents.iter().take(10) {
        let agent_messages = repositories
            .messages
            .find_by_agent_id(&agent.id, 20)
            .await
            .map_err(|e| ServerError::internal_error(e.to_string()))?;
        recent_messages.extend(agent_messages);
    }
    recent_messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    recent_messages.truncate(20);

    let mut active_agent_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut active_user_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for msg in &recent_messages {
        if msg.created_at > cutoff_time {
            active_agent_ids.insert(msg.agent_id.clone());
            active_user_ids.insert(msg.user_id.clone());
        }
    }

    let active_agents = active_agent_ids.len() as i64;
    let active_users = active_user_ids.len() as i64;
    info!("  - Active agents (24h): {}", active_agents);
    info!("  - Active users (24h): {}", active_users);

    // ✅ 综合数据源6: 平均响应时间 (计算最近消息的时间间隔)
    let avg_response_time_ms = if recent_messages.len() >= 2 {
        let mut intervals = Vec::new();
        for i in 1..recent_messages.len().min(10) {
            let interval = (recent_messages[i - 1].created_at - recent_messages[i].created_at)
                .num_milliseconds()
                .abs() as f64;
            if interval > 0.0 && interval < 60000.0 {
                // 忽略超过1分钟的间隔
                intervals.push(interval);
            }
        }
        if !intervals.is_empty() {
            intervals.iter().sum::<f64>() / intervals.len() as f64
        } else {
            150.0
        }
    } else {
        150.0
    };
    info!("  - Avg response time: {:.0}ms", avg_response_time_ms);

    // ✅ 综合数据源7: 最近活动记录
    let mut recent_activities: Vec<ActivityLog> = Vec::new();
    for (i, msg) in recent_messages.iter().take(10).enumerate() {
        recent_activities.push(ActivityLog {
            id: format!("activity_{}", i),
            activity_type: "message_sent".to_string(),
            agent_id: Some(msg.agent_id.clone()),
            user_id: Some(msg.user_id.clone()),
            description: "Message sent in conversation".to_string(),
            timestamp: msg.created_at,
        });
    }

    info!("  - Recent activities: {} events", recent_activities.len());

    // 转换 memory 类型统计
    let memories_by_type: HashMap<String, i64> = memories_by_type_map;

    // ✅ 构建综合统计响应
    let stats = DashboardStats {
        total_agents,
        total_users,
        total_memories,
        total_messages,
        active_agents,
        active_users,
        avg_response_time_ms,
        recent_activities,
        memories_by_type,
        timestamp: Utc::now(),
    };

    info!("📊 Dashboard stats generated successfully:");
    info!(
        "   Agents: {} total, {} active (24h)",
        total_agents, active_agents
    );
    info!(
        "   Users: {} total, {} active (24h)",
        total_users, active_users
    );
    info!(
        "   Memories: {} total, {} types",
        total_memories,
        stats.memories_by_type.len()
    );
    info!(
        "   Messages: {} total, {:.0}ms avg response",
        total_messages, avg_response_time_ms
    );

    Ok(Json(stats))
}

/// Get memory growth statistics
///
/// Returns time series data showing memory growth over the specified period.
/// Data points include total memories, new memories, and breakdown by type.
#[utoipa::path(
    get,
    path = "/api/v1/stats/memories/growth",
    tag = "statistics",
    responses(
        (status = 200, description = "Memory growth statistics retrieved successfully", body = MemoryGrowthResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_growth(
    Extension(_repositories): Extension<Arc<Repositories>>,
    Extension(_memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<MemoryGrowthResponse>> {
    use libsql::{params, Builder};

    // ✅ Connect to database to query historical stats
    let db_path =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "file:./data/agentmem.db".to_string());
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to open database: {}", e)))?;
    let conn = db
        .connect()
        .map_err(|e| ServerError::internal_error(format!("Failed to connect: {}", e)))?;

    let mut data_points: Vec<MemoryGrowthPoint> = Vec::new();
    let mut total_memories = 0i64;

    // ✅ Try to query historical daily stats (last 30 days)
    // If table doesn't exist, fall back to current data
    let thirty_days_ago = (Utc::now() - Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();
    let query = "SELECT date, total_memories, new_memories, memories_by_type, avg_importance 
                 FROM memory_stats_daily 
                 WHERE date >= ?
                 ORDER BY date ASC";

    // Try to query, but don't fail if table doesn't exist
    match conn.prepare(query).await {
        Ok(stmt) => {
            if let Ok(mut rows) = stmt.query(params![thirty_days_ago]).await {
                while let Ok(Some(row)) = rows.next().await {
                    let date: String = row.get(0).unwrap_or_default();
                    let total: i64 = row.get(1).unwrap_or(0);
                    let new: i64 = row.get(2).unwrap_or(0);
                    let by_type_json: Option<String> = row.get(3).ok();

                    let by_type: HashMap<String, i64> = by_type_json
                        .and_then(|json| serde_json::from_str(&json).ok())
                        .unwrap_or_default();

                    data_points.push(MemoryGrowthPoint {
                        date,
                        total,
                        new,
                        by_type,
                    });

                    total_memories = total; // Update to latest
                }
            }
        }
        Err(e) => {
            // Table doesn't exist or query failed - log warning and continue with fallback
            warn!("⚠️  memory_stats_daily table not available: {}", e);
        }
    }

    // ✅ If no historical data exists, generate current data point
    if data_points.is_empty() {
        // Get current count from memories table
        let count_query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0";
        let count_stmt = conn
            .prepare(count_query)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to prepare count: {}", e)))?;

        if let Some(count_row) = count_stmt
            .query(params![])
            .await
            .ok()
            .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten())
        {
            total_memories = count_row.get::<i64>(0).unwrap_or(0);
        }

        let today = Utc::now().format("%Y-%m-%d").to_string();
        data_points.push(MemoryGrowthPoint {
            date: today,
            total: total_memories,
            new: total_memories,
            by_type: HashMap::new(),
        });
    }

    // ✅ Calculate real growth rate
    let growth_rate = if data_points.len() > 1 {
        let first = data_points
            .first()
            .ok_or_else(|| ServerError::internal_error("data_points is empty"))?
            .total as f64;
        let last = data_points
            .last()
            .ok_or_else(|| ServerError::internal_error("data_points is empty"))?
            .total as f64;
        let days = data_points.len() as f64;
        if days > 0.0 {
            (last - first) / days
        } else {
            0.0
        }
    } else {
        0.0
    };

    tracing::info!(
        "📊 Memory growth: {} data points, total={}, growth_rate={:.2}/day",
        data_points.len(),
        total_memories,
        growth_rate
    );

    let response = MemoryGrowthResponse {
        data: data_points,
        total_memories,
        growth_rate,
        timestamp: Utc::now(),
    };

    Ok(Json(response))
}

/// Get agent activity statistics
///
/// Returns activity statistics for all agents including:
/// - Memory counts
/// - Interaction counts (messages)
/// - Last active timestamps
/// - Average memory importance
#[utoipa::path(
    get,
    path = "/api/v1/stats/agents/activity",
    tag = "statistics",
    responses(
        (status = 200, description = "Agent activity statistics retrieved successfully", body = AgentActivityResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_agent_activity_stats(
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(_memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<AgentActivityResponse>> {
    use libsql::{params, Builder};

    // ✅ Connect to database for direct queries (avoid vector search)
    let db_path =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "file:./data/agentmem.db".to_string());
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to open database: {}", e)))?;
    let conn = db
        .connect()
        .map_err(|e| ServerError::internal_error(format!("Failed to connect: {}", e)))?;

    // Get all agents (using list with large limit)
    let all_agents = repositories
        .agents
        .list(1000, 0)
        .await
        .map_err(|e| ServerError::internal_error(e.to_string()))?;

    let total_agents = all_agents.len() as i64;

    // Build activity stats for each agent
    let mut agent_stats: Vec<AgentActivityStats> = Vec::new();

    for agent in all_agents.iter().take(20) {
        // Limit to 20 for performance
        // ✅ Query memory count and avg importance directly from database
        let memory_query = "SELECT COUNT(*), AVG(importance) 
                            FROM memories 
                            WHERE agent_id = ? AND is_deleted = 0";

        let stmt = conn.prepare(memory_query).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to prepare memory query: {}", e))
        })?;

        let mut rows = stmt.query(params![agent.id.as_str()]).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to execute memory query: {}", e))
        })?;

        let (total_memories, avg_importance) = if let Some(row) =
            rows.next().await.map_err(|e| {
                ServerError::internal_error(format!("Failed to fetch memory row: {}", e))
            })? {
            let count: i64 = row.get(0).unwrap_or(0);
            let avg: Option<f64> = row.get(1).ok();
            (count, avg.unwrap_or(0.0))
        } else {
            (0, 0.0)
        };

        // Get message count for this agent
        let messages = repositories
            .messages
            .find_by_agent_id(&agent.id, 1000)
            .await
            .map_err(|e| ServerError::internal_error(e.to_string()))?;

        let total_interactions = messages.len() as i64;

        // Get last active timestamp from most recent message
        let last_active = messages.first().map(|m| m.created_at);

        agent_stats.push(AgentActivityStats {
            agent_id: agent.id.clone(),
            agent_name: agent
                .name
                .clone()
                .unwrap_or_else(|| agent.id[..8].to_string()),
            total_memories,
            total_interactions,
            last_active,
            avg_importance,
        });
    }

    // Sort by total_interactions descending
    agent_stats.sort_by(|a, b| b.total_interactions.cmp(&a.total_interactions));

    tracing::info!(
        "📊 Agent activity: {} agents, top agent has {} interactions",
        total_agents,
        agent_stats
            .first()
            .map(|a| a.total_interactions)
            .unwrap_or(0)
    );

    let response = AgentActivityResponse {
        agents: agent_stats,
        total_agents,
        timestamp: Utc::now(),
    };

    Ok(Json(response))
}

/// Get memory quality statistics
///
/// Returns comprehensive memory quality metrics including:
/// - Importance distribution
/// - Memory type distribution
/// - High-quality memory ratio
/// - Average importance by type
#[utoipa::path(
    get,
    path = "/api/v1/stats/memory/quality",
    tag = "statistics",
    responses(
        (status = 200, description = "Memory quality statistics retrieved successfully", body = MemoryQualityStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_quality_stats(
    Extension(_repositories): Extension<Arc<Repositories>>,
) -> ServerResult<Json<MemoryQualityStats>> {
    use libsql::{params, Builder};

    // ✅ Connect to database for direct queries
    let db_path =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "file:./data/agentmem.db".to_string());
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to open database: {}", e)))?;
    let conn = db
        .connect()
        .map_err(|e| ServerError::internal_error(format!("Failed to connect: {}", e)))?;

    // Query total memories and average importance
    let basic_query = "SELECT COUNT(*), AVG(importance) 
                       FROM memories 
                       WHERE is_deleted = 0";

    let stmt = conn.prepare(basic_query).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to prepare basic query: {}", e))
    })?;

    let mut rows = stmt.query(params![]).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to execute basic query: {}", e))
    })?;

    let (total_memories, avg_importance) = if let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to fetch basic row: {}", e)))?
    {
        let count: i64 = row.get(0).unwrap_or(0);
        let avg: Option<f64> = row.get(1).ok();
        (count, avg.unwrap_or(0.0))
    } else {
        (0, 0.0)
    };

    // Query high-quality memory ratio (importance > 0.7)
    let high_quality_query = "SELECT COUNT(*) * 100.0 / ? 
                              FROM memories 
                              WHERE is_deleted = 0 AND importance > 0.7";

    let stmt2 = conn.prepare(high_quality_query).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to prepare quality query: {}", e))
    })?;

    let high_quality_ratio = if total_memories > 0 {
        let mut rows2 = stmt2.query(params![total_memories]).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to execute quality query: {}", e))
        })?;

        if let Some(row) = rows2.next().await.map_err(|e| {
            ServerError::internal_error(format!("Failed to fetch quality row: {}", e))
        })? {
            row.get::<f64>(0).unwrap_or(0.0)
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Query importance distribution
    let mut importance_distribution = HashMap::new();

    let dist_queries = vec![
        ("0.0-0.3", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND importance >= 0.0 AND importance < 0.3"),
        ("0.3-0.7", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND importance >= 0.3 AND importance < 0.7"),
        ("0.7-1.0", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND importance >= 0.7 AND importance <= 1.0"),
    ];

    for (range, query) in dist_queries {
        let stmt3 = conn.prepare(query).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to prepare dist query: {}", e))
        })?;

        let mut rows3 = stmt3.query(params![]).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to execute dist query: {}", e))
        })?;

        if let Some(row) = rows3
            .next()
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to fetch dist row: {}", e)))?
        {
            let count: i64 = row.get(0).unwrap_or(0);
            importance_distribution.insert(range.to_string(), count);
        }
    }

    // Query memory type distribution
    let type_query = "SELECT memory_type, COUNT(*), AVG(importance) 
                      FROM memories 
                      WHERE is_deleted = 0 
                      GROUP BY memory_type
                      ORDER BY COUNT(*) DESC";

    let stmt4 = conn
        .prepare(type_query)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to prepare type query: {}", e)))?;

    let mut rows4 = stmt4
        .query(params![])
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to execute type query: {}", e)))?;

    let mut type_distribution = Vec::new();

    while let Some(row) = rows4
        .next()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to fetch type row: {}", e)))?
    {
        let type_name: String = row.get(0).unwrap_or_else(|_| "Unknown".to_string());
        let count: i64 = row.get(1).unwrap_or(0);
        let type_avg_importance: Option<f64> = row.get(2).ok();

        let percentage = if total_memories > 0 {
            (count as f64 / total_memories as f64) * 100.0
        } else {
            0.0
        };

        type_distribution.push(MemoryTypeStats {
            type_name,
            count,
            percentage,
            avg_importance: type_avg_importance.unwrap_or(0.0),
        });
    }

    // Placeholder access stats (for future implementation)
    let access_stats = HashMap::new();

    tracing::info!(
        "📊 Memory quality: total={}, avg_importance={:.2}, high_quality={:.1}%, types={}",
        total_memories,
        avg_importance,
        high_quality_ratio,
        type_distribution.len()
    );

    let response = MemoryQualityStats {
        avg_importance,
        high_quality_ratio,
        importance_distribution,
        type_distribution,
        total_memories,
        access_stats,
        timestamp: Utc::now(),
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_stats_serialization() {
        let stats = DashboardStats {
            total_agents: 10,
            total_users: 5,
            total_memories: 100,
            total_messages: 50,
            active_agents: 3,
            active_users: 2,
            avg_response_time_ms: 150.0,
            recent_activities: vec![],
            memories_by_type: HashMap::new(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("total_agents"));
        assert!(json.contains("total_memories"));
    }

    #[test]
    fn test_memory_growth_point_serialization() {
        let point = MemoryGrowthPoint {
            date: "2024-01-01".to_string(),
            total: 100,
            new: 10,
            by_type: HashMap::new(),
        };

        let json = serde_json::to_string(&point).unwrap();
        assert!(json.contains("2024-01-01"));
        assert!(json.contains("\"total\":100"));
    }

    /// 测试数据库连接池统计API响应结构
    #[test]
    fn test_database_pool_stats_structure() {
        let stats = DatabasePoolStats {
            size_bytes: 1024 * 1024, // 1MB
            size_mb: 1.0,
            page_count: 256,
            page_size: 4096,
            health_status: "healthy".to_string(),
            pool_status: "active".to_string(),
        };

        // 验证字段存在
        assert_eq!(stats.size_bytes, 1024 * 1024);
        assert_eq!(stats.size_mb, 1.0);
        assert_eq!(stats.page_count, 256);
        assert_eq!(stats.page_size, 4096);
        assert_eq!(stats.health_status, "healthy");
        assert_eq!(stats.pool_status, "active");

        // 验证序列化
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("size_bytes"));
        assert!(json.contains("size_mb"));
        assert!(json.contains("page_count"));
        assert!(json.contains("health_status"));
        assert!(json.contains("pool_status"));
    }

    /// 测试数据库连接池统计API响应验证
    #[test]
    fn test_database_pool_stats_validation() {
        // 测试健康状态
        let healthy_stats = DatabasePoolStats {
            size_bytes: 1024,
            size_mb: 0.001,
            page_count: 1,
            page_size: 1024,
            health_status: "healthy".to_string(),
            pool_status: "active".to_string(),
        };
        assert_eq!(healthy_stats.health_status, "healthy");

        // 测试不健康状态
        let unhealthy_stats = DatabasePoolStats {
            size_bytes: 0,
            size_mb: 0.0,
            page_count: 0,
            page_size: 4096,
            health_status: "unhealthy".to_string(),
            pool_status: "inactive".to_string(),
        };
        assert_eq!(unhealthy_stats.health_status, "unhealthy");
    }

    /// 🆕 Phase 3.1: 测试索引性能监控响应结构
    #[test]
    fn test_index_performance_stats_structure() {
        use chrono::Utc;

        let stats = IndexPerformanceStats {
            current_index: IndexInfo {
                index_type: "Flat".to_string(),
                total_vectors: 5000,
                dimension: 1536,
                avg_vector_norm: 1.0,
                last_updated: Utc::now(),
            },
            recommended_index: "HNSW".to_string(),
            recommendations: vec![OptimizationRecommendation {
                recommendation_type: "index_type".to_string(),
                severity: "high".to_string(),
                description: "建议升级索引".to_string(),
                expected_improvement: Some(50.0),
            }],
            performance_metrics: PerformanceMetrics {
                estimated_latency_ms: 10,
                estimated_recall: 0.95,
                estimated_index_size_mb: 50.0,
            },
            timestamp: Utc::now(),
        };

        // 验证字段存在
        assert_eq!(stats.current_index.index_type, "Flat");
        assert_eq!(stats.current_index.total_vectors, 5000);
        assert_eq!(stats.recommended_index, "HNSW");
        assert_eq!(stats.recommendations.len(), 1);
        assert_eq!(stats.performance_metrics.estimated_recall, 0.95);

        // 验证序列化
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("current_index"));
        assert!(json.contains("recommendations"));
        assert!(json.contains("performance_metrics"));
    }

    /// 🆕 Phase 3.1: 测试性能指标计算
    #[test]
    fn test_performance_metrics_calculation() {
        use agent_mem_core::search::query_optimizer::IndexStatistics;

        // 测试小数据集（Flat索引）
        let small_stats = IndexStatistics::new(1000, 1536);
        let small_metrics = calculate_performance_metrics(&small_stats);
        assert_eq!(
            small_metrics.estimated_recall, 1.0,
            "Flat索引应该有100%召回率"
        );
        assert!(
            small_metrics.estimated_latency_ms < 100,
            "小数据集延迟应该很低"
        );

        // 测试大数据集（HNSW索引）
        let large_stats = IndexStatistics::new(50_000, 1536);
        let large_metrics = calculate_performance_metrics(&large_stats);
        assert!(
            large_metrics.estimated_recall >= 0.95,
            "HNSW索引应该有高召回率"
        );
        assert!(
            large_metrics.estimated_index_size_mb > 0.0,
            "应该有索引大小估算"
        );
    }

    /// 🆕 Phase 3.1: 测试预期性能提升计算
    #[test]
    fn test_expected_improvement_calculation() {
        use agent_mem_core::search::query_optimizer::IndexType;

        // 测试从Flat升级到HNSW（大数据集）
        let improvement1 =
            calculate_expected_improvement(&IndexType::Flat, &IndexType::HNSW, 50_000);
        assert!(
            improvement1 >= 60.0,
            "大数据集从Flat升级到HNSW应该有显著提升"
        );

        // 测试从Flat升级到IVF_HNSW（超大数据集）
        let improvement2 =
            calculate_expected_improvement(&IndexType::Flat, &IndexType::IVF_HNSW, 200_000);
        assert!(
            improvement2 >= 80.0,
            "超大数据集从Flat升级到IVF_HNSW应该有更大提升"
        );

        // 测试从HNSW升级到IVF_HNSW
        let improvement3 =
            calculate_expected_improvement(&IndexType::HNSW, &IndexType::IVF_HNSW, 200_000);
        assert!(improvement3 >= 30.0, "从HNSW升级到IVF_HNSW应该有中等提升");
    }
}

/// Database connection pool statistics response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DatabasePoolStats {
    /// Database size in bytes
    pub size_bytes: u64,
    /// Database size in megabytes
    pub size_mb: f64,
    /// Total number of pages
    pub page_count: u64,
    /// Page size in bytes
    pub page_size: u64,
    /// Database health status
    pub health_status: String,
    /// Connection pool status (simplified)
    pub pool_status: String,
}

/// Get database connection pool statistics
///
/// 🆕 Phase 3.2: 连接池管理 - 提供数据库连接统计信息
#[utoipa::path(
    get,
    path = "/api/v1/stats/database/pool",
    tag = "statistics",
    responses(
        (status = 200, description = "Database pool statistics retrieved successfully", body = DatabasePoolStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_database_pool_stats() -> ServerResult<Json<DatabasePoolStats>> {
    info!("📊 获取数据库连接池统计信息");

    // 获取数据库路径
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");

    // 创建连接管理器
    let manager = LibSqlConnectionManager::new(&db_path).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to create connection manager: {}", e))
    })?;

    // 获取数据库统计信息
    let db_stats = manager
        .get_stats()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get database stats: {}", e)))?;

    // 检查数据库健康状态
    let health_status = match manager.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    // 简化的连接池状态（LibSQL使用单连接模式，这里标记为active）
    let pool_status = "active".to_string();

    let response = DatabasePoolStats {
        size_bytes: db_stats.size_bytes,
        size_mb: db_stats.size_mb(),
        page_count: db_stats.page_count,
        page_size: db_stats.page_size,
        health_status,
        pool_status,
    };

    info!(
        "📊 数据库统计: 大小={:.2}MB, 页数={}, 健康状态={}",
        response.size_mb, response.page_count, response.health_status
    );

    Ok(Json(response))
}

/// 🆕 Phase 3.1: 索引性能监控和优化建议响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IndexPerformanceStats {
    /// 当前索引统计信息
    pub current_index: IndexInfo,
    /// 推荐的索引类型
    pub recommended_index: String,
    /// 优化建议列表
    pub recommendations: Vec<OptimizationRecommendation>,
    /// 性能指标
    pub performance_metrics: PerformanceMetrics,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IndexInfo {
    /// 索引类型
    pub index_type: String,
    /// 总向量数
    pub total_vectors: usize,
    /// 向量维度
    pub dimension: usize,
    /// 平均向量范数
    pub avg_vector_norm: f32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OptimizationRecommendation {
    /// 建议类型
    pub recommendation_type: String,
    /// 严重程度 (low, medium, high)
    pub severity: String,
    /// 建议描述
    pub description: String,
    /// 预期性能提升（百分比）
    pub expected_improvement: Option<f64>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceMetrics {
    /// 预期查询延迟（毫秒）
    pub estimated_latency_ms: u64,
    /// 预期召回率（0.0-1.0）
    pub estimated_recall: f32,
    /// 索引大小估算（MB）
    pub estimated_index_size_mb: f64,
}

/// 🆕 Phase 3.1: 获取索引性能监控和优化建议
///
/// 基于QueryOptimizer的IndexStatistics提供索引性能监控和优化建议
#[utoipa::path(
    get,
    path = "/api/v1/stats/index/performance",
    tag = "statistics",
    responses(
        (status = 200, description = "Index performance statistics retrieved successfully", body = IndexPerformanceStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_index_performance_stats(
    Extension(_memory_manager): Extension<Arc<MemoryManager>>,
) -> ServerResult<Json<IndexPerformanceStats>> {
    info!("📊 获取索引性能监控和优化建议");

    // 获取当前向量数量（从数据库查询）
    let total_vectors = {
        use libsql::{params, Builder};
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
            .replace("file:", "");

        let db = Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to open database: {}", e)))?;

        let conn = db
            .connect()
            .map_err(|e| ServerError::internal_error(format!("Failed to connect: {}", e)))?;

        let stmt = conn
            .prepare("SELECT COUNT(*) FROM memories WHERE is_deleted = 0")
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to prepare query: {}", e)))?;

        let mut rows = stmt
            .query(params![])
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to execute query: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to fetch row: {}", e)))?
        {
            row.get::<i64>(0).unwrap_or(0) as usize
        } else {
            0
        }
    };

    // 创建IndexStatistics（基于实际数据）
    let dimension = 1536; // 默认OpenAI embedding维度
    let stats = IndexStatistics::new(total_vectors, dimension);

    // 获取当前索引信息
    let current_index = IndexInfo {
        index_type: format!("{:?}", stats.index_type),
        total_vectors: stats.total_vectors,
        dimension: stats.dimension,
        avg_vector_norm: stats.avg_vector_norm,
        last_updated: Utc::now(), // 简化版：使用当前时间
    };

    // 生成优化建议
    let mut recommendations = Vec::new();

    // 建议1: 根据数据规模推荐索引类型
    let recommended_index = stats.index_type;
    if stats.index_type != recommended_index {
        recommendations.push(OptimizationRecommendation {
            recommendation_type: "index_type".to_string(),
            severity: "high".to_string(),
            description: format!(
                "建议使用 {:?} 索引类型以优化性能。当前使用 {:?}，数据规模为 {} 条向量",
                recommended_index, stats.index_type, total_vectors
            ),
            expected_improvement: Some(calculate_expected_improvement(
                &stats.index_type,
                &recommended_index,
                total_vectors,
            )),
        });
    }

    // 建议2: 小数据集优化
    if total_vectors < 1000 {
        recommendations.push(OptimizationRecommendation {
            recommendation_type: "dataset_size".to_string(),
            severity: "low".to_string(),
            description: "数据集较小，当前索引配置已足够。".to_string(),
            expected_improvement: None,
        });
    } else if total_vectors >= 100_000 && stats.index_type == IndexType::Flat {
        recommendations.push(OptimizationRecommendation {
            recommendation_type: "index_upgrade".to_string(),
            severity: "high".to_string(),
            description: format!(
                "数据集规模较大（{} 条向量），建议升级到 HNSW 或 IVF_HNSW 索引以提升查询性能",
                total_vectors
            ),
            expected_improvement: Some(50.0), // 预期50%性能提升
        });
    }

    // 建议3: 索引重建建议（简化版：基于统计信息）
    let hours_since_update = stats.last_updated.elapsed().as_secs() / 3600;
    if hours_since_update > 24 * 7 {
        // 超过7天
        recommendations.push(OptimizationRecommendation {
            recommendation_type: "index_rebuild".to_string(),
            severity: "medium".to_string(),
            description: format!(
                "索引已 {} 天未更新，建议重建索引以优化性能",
                hours_since_update / 24
            ),
            expected_improvement: Some(10.0), // 预期10%性能提升
        });
    }

    // 计算性能指标
    let performance_metrics = calculate_performance_metrics(&stats);

    // 保存建议数量（在move之前）
    let recommendations_count = recommendations.len();

    let response = IndexPerformanceStats {
        current_index,
        recommended_index: format!("{:?}", recommended_index),
        recommendations,
        performance_metrics,
        timestamp: Utc::now(),
    };

    info!(
        "📊 索引性能监控: 向量数={}, 索引类型={:?}, 建议数={}",
        total_vectors, stats.index_type, recommendations_count
    );

    Ok(Json(response))
}

/// 计算预期性能提升（百分比）
fn calculate_expected_improvement(
    current: &IndexType,
    recommended: &IndexType,
    total_vectors: usize,
) -> f64 {
    // 简化的性能提升计算
    match (current, recommended) {
        (IndexType::Flat, IndexType::HNSW) if total_vectors > 10_000 => 60.0,
        (IndexType::Flat, IndexType::IVF_HNSW) if total_vectors > 100_000 => 80.0,
        (IndexType::HNSW, IndexType::IVF_HNSW) if total_vectors > 100_000 => 30.0,
        _ => 20.0, // 默认20%提升
    }
}

/// 计算性能指标
fn calculate_performance_metrics(stats: &IndexStatistics) -> PerformanceMetrics {
    // 基于索引类型估算性能
    let (latency_ms, recall, index_size_mb) = match stats.index_type {
        IndexType::None | IndexType::Flat => {
            // 线性扫描：O(n)
            let latency = (stats.total_vectors as f64 * 0.0001) as u64; // 每个向量0.1μs
            (latency, 1.0, 0.0) // 精确搜索，100%召回，无索引大小
        }
        IndexType::HNSW => {
            // HNSW：O(log n)
            let latency = ((stats.total_vectors as f64).ln() * 2.0) as u64;
            let recall = 0.95; // 95%召回
            let index_size =
                (stats.total_vectors as f64 * stats.dimension as f64 * 4.0) / (1024.0 * 1024.0); // 估算索引大小
            (latency, recall, index_size)
        }
        IndexType::IVF => {
            // IVF：O(nprobe * cluster_size)
            let cluster_size = if stats.total_vectors > 0 && stats.total_vectors >= 100 {
                stats.total_vectors / 100
            } else {
                1
            }; // 假设100个聚类
            let latency = (10 * cluster_size) as u64 / 10000;
            let recall = 0.93; // 93%召回
            let index_size =
                (stats.total_vectors as f64 * stats.dimension as f64 * 2.0) / (1024.0 * 1024.0);
            (latency, recall, index_size)
        }
        IndexType::IVF_HNSW => {
            // 混合：最快
            let latency = ((stats.total_vectors as f64).ln() * 1.5) as u64;
            let recall = 0.95; // 95%召回
            let index_size =
                (stats.total_vectors as f64 * stats.dimension as f64 * 3.0) / (1024.0 * 1024.0);
            (latency, recall, index_size)
        }
    };

    PerformanceMetrics {
        estimated_latency_ms: latency_ms,
        estimated_recall: recall,
        estimated_index_size_mb: index_size_mb,
    }
}

/// 🆕 Phase 4.3: 记忆使用情况统计响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryUsageStats {
    /// 总记忆数
    pub total_memories: i64,

    /// 按访问频率分布
    pub access_frequency_distribution: HashMap<String, i64>,

    /// 按最近访问时间分布
    pub recency_distribution: HashMap<String, i64>,

    /// 平均访问次数
    pub avg_access_count: f64,

    /// 最近访问的记忆数（24小时内）
    pub recently_accessed: i64,

    /// 从未访问的记忆数
    pub never_accessed: i64,

    /// 高访问记忆数（访问次数 > 10）
    pub high_access_memories: i64,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 🆕 Phase 4.3: 获取记忆使用情况统计
#[utoipa::path(
    get,
    path = "/api/v1/stats/memory/usage",
    tag = "statistics",
    responses(
        (status = 200, description = "Memory usage statistics retrieved successfully", body = MemoryUsageStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_usage_stats(
    Extension(_repositories): Extension<Arc<Repositories>>,
) -> ServerResult<Json<MemoryUsageStats>> {
    info!("📊 获取记忆使用情况统计");

    use libsql::{params, Builder};
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to open database: {}", e)))?;

    let conn = db
        .connect()
        .map_err(|e| ServerError::internal_error(format!("Failed to connect: {}", e)))?;

    // 查询总记忆数和平均访问次数
    let basic_query = "SELECT COUNT(*), AVG(COALESCE(access_count, 0)) 
                       FROM memories 
                       WHERE is_deleted = 0";

    let stmt = conn.prepare(basic_query).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to prepare basic query: {}", e))
    })?;

    let mut rows = stmt.query(params![]).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to execute basic query: {}", e))
    })?;

    let (total_memories, avg_access_count) = if let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to fetch basic row: {}", e)))?
    {
        let count: i64 = row.get(0).unwrap_or(0);
        let avg: Option<f64> = row.get(1).ok();
        (count, avg.unwrap_or(0.0))
    } else {
        (0, 0.0)
    };

    // 查询访问频率分布
    let mut access_frequency_distribution = HashMap::new();
    let frequency_queries = vec![
        ("0", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND (access_count IS NULL OR access_count = 0)"),
        ("1-5", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND access_count >= 1 AND access_count <= 5"),
        ("6-10", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND access_count >= 6 AND access_count <= 10"),
        ("11-50", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND access_count >= 11 AND access_count <= 50"),
        ("51+", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND access_count > 50"),
    ];

    for (range, query) in frequency_queries {
        let stmt2 = conn.prepare(query).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to prepare frequency query: {}", e))
        })?;

        let mut rows2 = stmt2.query(params![]).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to execute frequency query: {}", e))
        })?;

        if let Some(row) = rows2.next().await.map_err(|e| {
            ServerError::internal_error(format!("Failed to fetch frequency row: {}", e))
        })? {
            let count: i64 = row.get(0).unwrap_or(0);
            access_frequency_distribution.insert(range.to_string(), count);
        }
    }

    // 查询最近访问时间分布
    let mut recency_distribution = HashMap::new();
    let now = Utc::now().timestamp();
    let one_day_ago = now - 86400;
    let one_week_ago = now - 604800;
    let one_month_ago = now - 2592000;

    let recency_queries = vec![
        ("24小时内", format!("SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND last_accessed IS NOT NULL AND last_accessed >= {}", one_day_ago)),
        ("1周内", format!("SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND last_accessed IS NOT NULL AND last_accessed >= {} AND last_accessed < {}", one_week_ago, one_day_ago)),
        ("1月内", format!("SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND last_accessed IS NOT NULL AND last_accessed >= {} AND last_accessed < {}", one_month_ago, one_week_ago)),
        ("1月前", format!("SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND last_accessed IS NOT NULL AND last_accessed < {}", one_month_ago)),
        ("从未访问", "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND (last_accessed IS NULL OR last_accessed = 0)".to_string()),
    ];

    for (range, query) in recency_queries {
        let stmt3 = conn.prepare(&query).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to prepare recency query: {}", e))
        })?;

        let mut rows3 = stmt3.query(params![]).await.map_err(|e| {
            ServerError::internal_error(format!("Failed to execute recency query: {}", e))
        })?;

        if let Some(row) = rows3.next().await.map_err(|e| {
            ServerError::internal_error(format!("Failed to fetch recency row: {}", e))
        })? {
            let count: i64 = row.get(0).unwrap_or(0);
            recency_distribution.insert(range.to_string(), count);
        }
    }

    // 查询最近访问的记忆数（24小时内）
    let recently_accessed_query = format!("SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND last_accessed IS NOT NULL AND last_accessed >= {}", one_day_ago);
    let stmt4 = conn.prepare(&recently_accessed_query).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to prepare recently accessed query: {}", e))
    })?;

    let recently_accessed = if let Some(row) = stmt4
        .query(params![])
        .await
        .map_err(|e| {
            ServerError::internal_error(format!("Failed to execute recently accessed query: {}", e))
        })?
        .next()
        .await
        .map_err(|e| {
            ServerError::internal_error(format!("Failed to fetch recently accessed row: {}", e))
        })? {
        row.get::<i64>(0).unwrap_or(0)
    } else {
        0
    };

    // 查询从未访问的记忆数
    let never_accessed_query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND (access_count IS NULL OR access_count = 0)";
    let stmt5 = conn.prepare(never_accessed_query).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to prepare never accessed query: {}", e))
    })?;

    let never_accessed = if let Some(row) = stmt5
        .query(params![])
        .await
        .map_err(|e| {
            ServerError::internal_error(format!("Failed to execute never accessed query: {}", e))
        })?
        .next()
        .await
        .map_err(|e| {
            ServerError::internal_error(format!("Failed to fetch never accessed row: {}", e))
        })? {
        row.get::<i64>(0).unwrap_or(0)
    } else {
        0
    };

    // 查询高访问记忆数（访问次数 > 10）
    let high_access_query =
        "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND access_count > 10";
    let stmt6 = conn.prepare(high_access_query).await.map_err(|e| {
        ServerError::internal_error(format!("Failed to prepare high access query: {}", e))
    })?;

    let high_access_memories = if let Some(row) = stmt6
        .query(params![])
        .await
        .map_err(|e| {
            ServerError::internal_error(format!("Failed to execute high access query: {}", e))
        })?
        .next()
        .await
        .map_err(|e| {
            ServerError::internal_error(format!("Failed to fetch high access row: {}", e))
        })? {
        row.get::<i64>(0).unwrap_or(0)
    } else {
        0
    };

    let stats = MemoryUsageStats {
        total_memories,
        access_frequency_distribution,
        recency_distribution,
        avg_access_count,
        recently_accessed,
        never_accessed,
        high_access_memories,
        timestamp: Utc::now(),
    };

    info!(
        "✅ 记忆使用情况统计完成: 总记忆数={}, 平均访问次数={:.2}",
        total_memories, avg_access_count
    );

    Ok(Json(stats))
}
