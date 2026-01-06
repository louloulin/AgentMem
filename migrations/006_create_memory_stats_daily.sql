-- Migration: Create memory statistics tables
-- Created: 2024-11-03
-- Purpose: Track historical memory growth and system metrics

-- Daily memory statistics
CREATE TABLE IF NOT EXISTS memory_stats_daily (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL UNIQUE,  -- YYYY-MM-DD format
    total_memories INTEGER NOT NULL DEFAULT 0,
    new_memories INTEGER NOT NULL DEFAULT 0,
    deleted_memories INTEGER NOT NULL DEFAULT 0,
    memories_by_type TEXT,  -- JSON: {"Semantic": 100, "Episodic": 50}
    avg_importance REAL,
    max_importance REAL,
    min_importance REAL,
    total_size_bytes INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_memory_stats_date ON memory_stats_daily(date DESC);
CREATE INDEX IF NOT EXISTS idx_memory_stats_created ON memory_stats_daily(created_at DESC);

-- LLM call logs for cost tracking
CREATE TABLE IF NOT EXISTS llm_call_logs (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    user_id TEXT,
    model TEXT NOT NULL,
    provider TEXT NOT NULL,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    cost_usd REAL DEFAULT 0.0,
    response_time_ms INTEGER,
    status TEXT NOT NULL,  -- "success", "error", "timeout"
    error_message TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

CREATE INDEX IF NOT EXISTS idx_llm_logs_agent ON llm_call_logs(agent_id);
CREATE INDEX IF NOT EXISTS idx_llm_logs_created ON llm_call_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_llm_logs_model ON llm_call_logs(model);

-- System performance metrics
CREATE TABLE IF NOT EXISTS system_performance_logs (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    cpu_usage REAL,
    memory_usage REAL,
    disk_usage REAL,
    active_connections INTEGER DEFAULT 0,
    cache_hit_rate REAL,
    avg_response_time_ms REAL,
    requests_per_second REAL,
    error_rate REAL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_perf_logs_timestamp ON system_performance_logs(timestamp DESC);

-- Insert initial daily stat for today if not exists
INSERT OR IGNORE INTO memory_stats_daily (
    id,
    date,
    total_memories,
    new_memories,
    deleted_memories,
    memories_by_type,
    avg_importance,
    created_at,
    updated_at
)
SELECT 
    'stat-' || strftime('%Y%m%d', 'now'),
    strftime('%Y-%m-%d', 'now'),
    COUNT(*),
    0,
    0,
    json_object(),
    AVG(importance),
    strftime('%s', 'now'),
    strftime('%s', 'now')
FROM memories
WHERE is_deleted = 0;

