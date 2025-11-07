//! MCP Server 真实测试
//! 
//! 使用真实工具进行集成测试，不使用Mock

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agentmem_tools::*;
    use std::sync::Arc;
    
    // 测试辅助函数：启动测试后端
    async fn start_test_backend() -> String {
        // TODO: 实现测试后端启动
        "http://127.0.0.1:8080".to_string()
    }
    
    #[tokio::test]
    async fn test_list_tools_with_real_tools() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        };
        
        let server = McpServer::new(config);
        
        // 注册真实工具
        let backend_url = start_test_backend().await;
        
        let add_memory_tool = Arc::new(AddMemoryTool { api_url: backend_url.clone() });
        let search_tool = Arc::new(SearchMemoriesTool { api_url: backend_url.clone() });
        
        server.register_tool(add_memory_tool).await.unwrap();
        server.register_tool(search_tool).await.unwrap();
        
        // 列出工具
        let response = server.list_tools().await.unwrap();
        
        assert_eq!(response.tools.len(), 2);
        assert_eq!(response.tools[0].name, "agentmem_add_memory");
        assert_eq!(response.tools[1].name, "agentmem_search_memories");
    }
    
    #[tokio::test]
    async fn test_tool_execution() {
        // TODO: 实现工具执行测试
    }
}
