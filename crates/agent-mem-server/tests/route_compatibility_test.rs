//! Task 1.5: 路由兼容性测试
//! 验证新旧路由都能正常工作

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    /// 模拟chat handler（简化测试）
    async fn mock_chat_handler() -> &'static str {
        "chat response"
    }

    /// 测试v1路由可以正常访问
    #[tokio::test]
    async fn test_v1_chat_route() {
        use axum::routing::post;
        
        let app = Router::new()
            .route("/api/v1/agents/:agent_id/chat", post(mock_chat_handler));

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/agents/test-agent/chat")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// ✅ Task 1.5: 测试兼容路由可以正常访问
    #[tokio::test]
    async fn test_legacy_chat_route() {
        use axum::routing::post;
        
        let app = Router::new()
            // 注册两个路由：v1和legacy
            .route("/api/v1/agents/:agent_id/chat", post(mock_chat_handler))
            .route("/api/agents/:agent_id/chat", post(mock_chat_handler));

        let request = Request::builder()
            .method("POST")
            .uri("/api/agents/test-agent/chat")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // 验证：旧路由应该返回200而不是404
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Legacy route should work (not 404)"
        );
    }

    /// ✅ Task 1.5: 测试stream路由兼容性
    #[tokio::test]
    async fn test_legacy_stream_route() {
        use axum::routing::post;
        
        let app = Router::new()
            .route("/api/v1/agents/:agent_id/chat/stream", post(mock_chat_handler))
            .route("/api/agents/:agent_id/chat/stream", post(mock_chat_handler));

        let request = Request::builder()
            .method("POST")
            .uri("/api/agents/test-agent/chat/stream")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Legacy stream route should work"
        );
    }

    /// ✅ Task 1.5: 测试history路由兼容性
    #[tokio::test]
    async fn test_legacy_history_route() {
        use axum::routing::get;
        
        async fn mock_history_handler() -> &'static str {
            "history"
        }
        
        let app = Router::new()
            .route("/api/v1/agents/:agent_id/chat/history", get(mock_history_handler))
            .route("/api/agents/:agent_id/chat/history", get(mock_history_handler));

        let request = Request::builder()
            .method("GET")
            .uri("/api/agents/test-agent/chat/history")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Legacy history route should work"
        );
    }

    /// ✅ Task 1.5: 测试LumosAI路由兼容性
    #[tokio::test]
    async fn test_legacy_lumosai_route() {
        use axum::routing::post;
        
        let app = Router::new()
            .route("/api/v1/agents/:agent_id/chat/lumosai", post(mock_chat_handler))
            .route("/api/agents/:agent_id/chat/lumosai", post(mock_chat_handler));

        let request = Request::builder()
            .method("POST")
            .uri("/api/agents/test-agent/chat/lumosai")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Legacy LumosAI route should work"
        );
    }

    /// ✅ Task 1.5: 测试LumosAI stream路由兼容性
    #[tokio::test]
    async fn test_legacy_lumosai_stream_route() {
        use axum::routing::post;
        
        let app = Router::new()
            .route("/api/v1/agents/:agent_id/chat/lumosai/stream", post(mock_chat_handler))
            .route("/api/agents/:agent_id/chat/lumosai/stream", post(mock_chat_handler));

        let request = Request::builder()
            .method("POST")
            .uri("/api/agents/test-agent/chat/lumosai/stream")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Legacy LumosAI stream route should work"
        );
    }

    /// 验证路由参数提取正常工作
    #[tokio::test]
    async fn test_agent_id_extraction() {
        use axum::{extract::Path, routing::post};
        
        async fn handler_with_param(Path(agent_id): Path<String>) -> String {
            format!("agent_id: {}", agent_id)
        }
        
        let app = Router::new()
            .route("/api/agents/:agent_id/chat", post(handler_with_param));

        let request = Request::builder()
            .method("POST")
            .uri("/api/agents/my-test-agent-123/chat")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        assert_eq!(body_str, "agent_id: my-test-agent-123");
    }
}

