//! Documentation routes

use axum::{http::StatusCode, response::Html};

/// Serve OpenAPI documentation
pub async fn serve_docs() -> Result<Html<String>, StatusCode> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>AgentMem API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@3.52.5/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@3.52.5/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: '/api-docs/openapi.json',
            dom_id: '#swagger-ui',
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.presets.standalone
            ]
        });
    </script>
</body>
</html>
    "#;

    Ok(Html(html.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serve_docs() {
        let result = serve_docs().await;
        assert!(result.is_ok());

        let html = result.unwrap().0;
        assert!(html.contains("AgentMem API Documentation"));
    }
}
