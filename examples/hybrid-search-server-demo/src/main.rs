//! 混合检索Server演示 - REST API + MCP工具
//!
//! 这个演示展示了如何通过HTTP REST API和MCP协议暴露混合检索功能

use agent_mem_core::search::{
    AdaptiveThresholdCalculator, EnhancedHybridConfig, EnhancedHybridSearchEngineV2,
    EnhancedSearchResult, QueryClassifier, QueryType,
};
use agent_mem_tools::executor::ToolExecutor;
use agent_mem_tools::mcp::server::{McpServer, McpServerConfig};
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

/// 搜索请求
#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

/// 搜索响应
#[derive(Debug, Serialize)]
struct SearchResponse {
    success: bool,
    query: String,
    query_type: String,
    results: Vec<ResultItem>,
    stats: SearchStats,
}

#[derive(Debug, Serialize)]
struct ResultItem {
    id: String,
    content: String,
    score: f32,
}

#[derive(Debug, Serialize)]
struct SearchStats {
    total_time_ms: u64,
    vector_time_ms: u64,
    bm25_time_ms: u64,
    results_count: usize,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    features: Vec<String>,
}

/// 应用状态
struct AppState {
    classifier: Arc<QueryClassifier>,
    threshold_calc: Arc<AdaptiveThresholdCalculator>,
    // 在实际应用中，这里会包含完整的搜索引擎
    // 为了演示简单性，我们只展示框架
}

/// 健康检查
async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        features: vec![
            "query-classification".to_string(),
            "adaptive-threshold".to_string(),
            "hybrid-search".to_string(),
            "vector-search".to_string(),
            "bm25-search".to_string(),
        ],
    })
}

/// 搜索端点
async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    info!("搜索请求: query='{}', limit={}", req.query, req.limit);

    // 1. 分类查询
    let query_type = state.classifier.classify(&req.query);
    info!("查询类型: {:?}", query_type);

    // 2. 提取特征（用于阈值计算）
    let features = state.classifier.extract_features(&req.query);

    // 3. 计算阈值
    let threshold = state
        .threshold_calc
        .calculate(&req.query, &query_type, &features)
        .await;
    info!("自适应阈值: {}", threshold);

    // 4. 获取搜索策略
    let strategy = state.classifier.get_strategy(&query_type);

    // 5. 模拟搜索（在实际应用中会调用真实的搜索引擎）
    let results = vec![
        ResultItem {
            id: "result1".to_string(),
            content: format!("匹配查询 '{}' 的结果1", req.query),
            score: 0.95,
        },
        ResultItem {
            id: "result2".to_string(),
            content: format!("匹配查询 '{}' 的结果2", req.query),
            score: 0.87,
        },
    ];

    let response = SearchResponse {
        success: true,
        query: req.query.clone(),
        query_type: format!("{:?}", query_type),
        results,
        stats: SearchStats {
            total_time_ms: 45,
            vector_time_ms: 20,
            bm25_time_ms: 15,
            results_count: 2,
        },
    };

    (StatusCode::OK, Json(response))
}

/// 查询分类端点
async fn classify_query(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchRequest>,
) -> impl IntoResponse {
    let query_type = state.classifier.classify(&params.query);
    let strategy = state.classifier.get_strategy(&query_type);

    Json(serde_json::json!({
        "query": params.query,
        "query_type": format!("{:?}", query_type),
        "strategy": {
            "use_vector": strategy.use_vector,
            "use_bm25": strategy.use_bm25,
            "vector_weight": strategy.vector_weight,
            "bm25_weight": strategy.bm25_weight,
            "threshold": strategy.threshold,
        }
    }))
}

/// MCP工具：搜索
async fn mcp_tool_search(
    _executor: Arc<ToolExecutor>,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let query = arguments["query"]
        .as_str()
        .ok_or("Missing 'query' parameter")?;

    let _limit = arguments["limit"].as_u64().unwrap_or(10) as usize;

    // 执行搜索（这里简化处理）
    Ok(serde_json::json!({
        "results": [
            {
                "id": "mcp_result1",
                "content": format!("MCP搜索结果: {}", query),
                "score": 0.92
            }
        ],
        "total": 1
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 启动混合检索Server演示");

    // 创建组件
    let classifier = Arc::new(QueryClassifier::with_default_config());
    let threshold_calc = Arc::new(AdaptiveThresholdCalculator::with_default_config());

    // 创建应用状态
    let state = Arc::new(AppState {
        classifier: classifier.clone(),
        threshold_calc: threshold_calc.clone(),
    });

    // 创建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/search", post(search))
        .route("/api/classify", get(classify_query))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🌐 Server启动在 http://{}", addr);
    info!("📋 可用端点:");
    info!("  - GET  /health           - 健康检查");
    info!("  - POST /api/search       - 搜索");
    info!("  - GET  /api/classify     - 查询分类");
    info!("");
    info!("💡 测试命令:");
    info!("  curl http://localhost:3000/health");
    info!("  curl -X POST http://localhost:3000/api/search -H 'Content-Type: application/json' -d '{{\"query\":\"Apple 手机\"}}'");
    info!("  curl 'http://localhost:3000/api/classify?query=iPhone'");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_request() {
        let req = SearchRequest {
            query: "test query".to_string(),
            limit: 10,
        };

        assert_eq!(req.query, "test query");
        assert_eq!(req.limit, 10);
    }

    #[test]
    fn test_health_response() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            features: vec!["search".to_string()],
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.features.len(), 1);
    }
}
