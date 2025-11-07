//! MCP 安全认证功能演示
//!
//! 演示多种认证方式和权限控制

use agent_mem_tools::mcp::{
    AuditEvent, AuditEventType, AuditLogger, AuthManager, AuthMethod, Credentials, JwtConfig,
    OAuth2Config, Permission, Role,
};
use tracing::error;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🔐 MCP 安全认证功能演示");
    println!("============================================================\n");

    // 演示 1: API 密钥认证
    demo_api_key_auth().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 2: JWT 令牌认证
    demo_jwt_auth().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 3: OAuth 2.0 认证
    demo_oauth2_auth().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 4: 权限控制
    demo_permission_control().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 5: 审计日志
    demo_audit_logging().await;

    println!("\n============================================================");
    println!("✅ 所有演示完成！");
}

/// 演示 API 密钥认证
async fn demo_api_key_auth() {
    println!("📋 演示 1: API 密钥认证");
    println!("------------------------------------------------------------");

    let manager = AuthManager::new(JwtConfig::default(), OAuth2Config::default(), true);

    // 1. 注册 API 密钥
    println!("\n1️⃣ 注册 API 密钥:");
    manager
        .register_api_key("test-api-key-123".to_string(), "user1".to_string())
        .await
        .unwrap();
    println!("  ✅ API 密钥注册成功");
    println!("  用户 ID: user1");
    println!("  API 密钥: test-api-key-123");

    // 2. 验证 API 密钥
    println!("\n2️⃣ 验证 API 密钥:");
    let credentials = Credentials {
        method: AuthMethod::ApiKey,
        api_key: Some("test-api-key-123".to_string()),
        access_token: None,
        jwt_token: None,
    };

    match manager.authenticate(&credentials).await {
        Ok(context) => {
            println!("  ✅ 认证成功");
            println!("  用户 ID: {}", context.user_id);
            println!("  角色: {:?}", context.role);
            println!("  权限数量: {}", context.permissions.len());
        }
        Err(e) => {
            error!("  ❌ 认证失败: {}", e);
        }
    }

    // 3. 验证无效的 API 密钥
    println!("\n3️⃣ 验证无效的 API 密钥:");
    let invalid_credentials = Credentials {
        method: AuthMethod::ApiKey,
        api_key: Some("invalid-key".to_string()),
        access_token: None,
        jwt_token: None,
    };

    match manager.authenticate(&invalid_credentials).await {
        Ok(_) => {
            println!("  ⚠️  认证成功（不应该发生）");
        }
        Err(e) => {
            println!("  ✅ 认证失败（预期行为）");
            println!("  错误: {e}");
        }
    }

    // 4. 撤销 API 密钥
    println!("\n4️⃣ 撤销 API 密钥:");
    manager.revoke_api_key("test-api-key-123").await.unwrap();
    println!("  ✅ API 密钥已撤销");

    match manager.authenticate(&credentials).await {
        Ok(_) => {
            println!("  ⚠️  认证成功（不应该发生）");
        }
        Err(_) => {
            println!("  ✅ 认证失败（密钥已撤销）");
        }
    }
}

/// 演示 JWT 令牌认证
async fn demo_jwt_auth() {
    println!("📋 演示 2: JWT 令牌认证");
    println!("------------------------------------------------------------");

    let jwt_config = JwtConfig {
        secret: "my-secret-key".to_string(),
        expiry_seconds: 3600,
        issuer: "agentmem-demo".to_string(),
        audience: "demo-client".to_string(),
    };

    let manager = AuthManager::new(jwt_config, OAuth2Config::default(), true);

    // 1. 使用 JWT 令牌认证
    println!("\n1️⃣ 使用 JWT 令牌认证:");
    let jwt_token = "user2:1234567890:signature"; // 简化的 JWT 格式
    let credentials = Credentials {
        method: AuthMethod::Jwt,
        api_key: None,
        access_token: None,
        jwt_token: Some(jwt_token.to_string()),
    };

    match manager.authenticate(&credentials).await {
        Ok(context) => {
            println!("  ✅ JWT 认证成功");
            println!("  用户 ID: {}", context.user_id);
            println!("  角色: {:?}", context.role);
            println!("  认证时间: {}", context.authenticated_at);
            if let Some(expires_at) = context.expires_at {
                println!("  过期时间: {expires_at}");
            }
        }
        Err(e) => {
            error!("  ❌ 认证失败: {}", e);
        }
    }

    // 2. 使用无效格式的 JWT
    println!("\n2️⃣ 使用无效格式的 JWT:");
    let invalid_jwt = "invalid-format";
    let invalid_credentials = Credentials {
        method: AuthMethod::Jwt,
        api_key: None,
        access_token: None,
        jwt_token: Some(invalid_jwt.to_string()),
    };

    match manager.authenticate(&invalid_credentials).await {
        Ok(_) => {
            println!("  ⚠️  认证成功（不应该发生）");
        }
        Err(e) => {
            println!("  ✅ 认证失败（预期行为）");
            println!("  错误: {e}");
        }
    }
}

/// 演示 OAuth 2.0 认证
async fn demo_oauth2_auth() {
    println!("📋 演示 3: OAuth 2.0 认证");
    println!("------------------------------------------------------------");

    let oauth2_config = OAuth2Config {
        client_id: "demo-client-id".to_string(),
        client_secret: "demo-client-secret".to_string(),
        auth_url: "https://auth.example.com/oauth/authorize".to_string(),
        token_url: "https://auth.example.com/oauth/token".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    let manager = AuthManager::new(JwtConfig::default(), oauth2_config, true);

    // 1. 使用 OAuth 2.0 访问令牌认证
    println!("\n1️⃣ 使用 OAuth 2.0 访问令牌认证:");
    let access_token = "oauth2-access-token-abc123";
    let credentials = Credentials {
        method: AuthMethod::OAuth2,
        api_key: None,
        access_token: Some(access_token.to_string()),
        jwt_token: None,
    };

    match manager.authenticate(&credentials).await {
        Ok(context) => {
            println!("  ✅ OAuth 2.0 认证成功");
            println!("  用户 ID: {}", context.user_id);
            println!("  角色: {:?}", context.role);
            println!("  权限数量: {}", context.permissions.len());
        }
        Err(e) => {
            error!("  ❌ 认证失败: {}", e);
        }
    }

    // 2. 显示 OAuth 2.0 配置
    println!("\n2️⃣ OAuth 2.0 配置:");
    println!("  客户端 ID: demo-client-id");
    println!("  授权 URL: https://auth.example.com/oauth/authorize");
    println!("  令牌 URL: https://auth.example.com/oauth/token");
    println!("  重定向 URI: http://localhost:8080/callback");
    println!("  作用域: {:?}", vec!["read", "write"]);
}

/// 演示权限控制
async fn demo_permission_control() {
    println!("📋 演示 4: 权限控制");
    println!("------------------------------------------------------------");

    let manager = AuthManager::new(JwtConfig::default(), OAuth2Config::default(), true);

    // 1. 创建不同角色的用户
    println!("\n1️⃣ 创建不同角色的用户:");

    // 管理员
    manager
        .register_api_key("admin-key".to_string(), "admin".to_string())
        .await
        .unwrap();
    manager.update_role("admin", Role::Admin).await.unwrap();
    println!("  ✅ 创建管理员用户: admin");

    // 开发者
    manager
        .register_api_key("dev-key".to_string(), "developer".to_string())
        .await
        .unwrap();
    manager
        .update_role("developer", Role::Developer)
        .await
        .unwrap();
    println!("  ✅ 创建开发者用户: developer");

    // 普通用户
    manager
        .register_api_key("user-key".to_string(), "user".to_string())
        .await
        .unwrap();
    manager.update_role("user", Role::User).await.unwrap();
    println!("  ✅ 创建普通用户: user");

    // 只读用户
    manager
        .register_api_key("readonly-key".to_string(), "readonly".to_string())
        .await
        .unwrap();
    manager
        .update_role("readonly", Role::ReadOnly)
        .await
        .unwrap();
    println!("  ✅ 创建只读用户: readonly");

    // 2. 检查权限
    println!("\n2️⃣ 检查权限:");

    let permissions_to_check = vec![
        Permission::Admin,
        Permission::ListTools,
        Permission::CallTool("test_tool".to_string()),
        Permission::ListResources,
    ];

    for user_id in &["admin", "developer", "user", "readonly"] {
        println!("\n  用户: {user_id}");
        for permission in &permissions_to_check {
            let has_perm = manager.check_permission(user_id, permission).await.unwrap();
            println!(
                "    {:?}: {}",
                permission,
                if has_perm { "✅ 允许" } else { "❌ 拒绝" }
            );
        }
    }

    // 3. 授予和撤销权限
    println!("\n3️⃣ 授予和撤销权限:");
    println!("  为普通用户授予调用工具权限:");
    manager
        .grant_permission("user", Permission::CallTool("special_tool".to_string()))
        .await
        .unwrap();
    let has_perm = manager
        .check_permission("user", &Permission::CallTool("special_tool".to_string()))
        .await
        .unwrap();
    println!("    ✅ 权限已授予: {has_perm}");

    println!("\n  撤销权限:");
    manager
        .revoke_permission("user", &Permission::CallTool("special_tool".to_string()))
        .await
        .unwrap();
    let has_perm = manager
        .check_permission("user", &Permission::CallTool("special_tool".to_string()))
        .await
        .unwrap();
    println!("    ✅ 权限已撤销: {}", !has_perm);
}

/// 演示审计日志
async fn demo_audit_logging() {
    println!("📋 演示 5: 审计日志");
    println!("------------------------------------------------------------");

    let logger = AuditLogger::new(1000, true);

    // 1. 记录各种审计事件
    println!("\n1️⃣ 记录审计事件:");

    // 认证成功
    logger
        .log(
            AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                "user1".to_string(),
                "login".to_string(),
                true,
            )
            .with_ip("192.168.1.100".to_string()),
        )
        .await;
    println!("  ✅ 记录认证成功事件");

    // 认证失败
    logger
        .log(
            AuditEvent::new(
                AuditEventType::AuthenticationFailure,
                "unknown".to_string(),
                "login".to_string(),
                false,
            )
            .with_ip("192.168.1.101".to_string())
            .with_detail("reason".to_string(), "Invalid credentials".to_string()),
        )
        .await;
    println!("  ✅ 记录认证失败事件");

    // 工具调用
    logger
        .log(
            AuditEvent::new(
                AuditEventType::ToolCall,
                "user1".to_string(),
                "call_tool".to_string(),
                true,
            )
            .with_resource("weather_api".to_string()),
        )
        .await;
    println!("  ✅ 记录工具调用事件");

    // 资源访问
    logger
        .log(
            AuditEvent::new(
                AuditEventType::ResourceAccess,
                "user1".to_string(),
                "read_resource".to_string(),
                true,
            )
            .with_resource("agentmem://memory/core".to_string()),
        )
        .await;
    println!("  ✅ 记录资源访问事件");

    // 权限授予
    logger
        .log(AuditEvent::new(
            AuditEventType::PermissionGranted,
            "admin".to_string(),
            "grant_permission".to_string(),
            true,
        ))
        .await;
    println!("  ✅ 记录权限授予事件");

    // 2. 查询审计事件
    println!("\n2️⃣ 查询审计事件:");

    let all_events = logger.get_all().await;
    println!("  总事件数: {}", all_events.len());

    // 按用户查询
    let user1_events = logger.query(Some("user1"), None, None, None).await;
    println!("  user1 的事件数: {}", user1_events.len());

    // 按事件类型查询
    let auth_events = logger
        .query(
            None,
            Some(&AuditEventType::AuthenticationSuccess),
            None,
            None,
        )
        .await;
    println!("  认证成功事件数: {}", auth_events.len());

    // 3. 显示审计事件详情
    println!("\n3️⃣ 审计事件详情:");
    for (i, event) in all_events.iter().enumerate() {
        println!("\n  事件 {}:", i + 1);
        println!("    ID: {}", event.id);
        println!("    类型: {:?}", event.event_type);
        println!("    用户: {}", event.user_id);
        println!("    操作: {}", event.action);
        println!("    时间: {}", event.timestamp);
        println!("    结果: {}", if event.success { "成功" } else { "失败" });
        if let Some(resource) = &event.resource {
            println!("    资源: {resource}");
        }
        if let Some(ip) = &event.ip_address {
            println!("    IP: {ip}");
        }
        if !event.details.is_empty() {
            println!("    详情: {:?}", event.details);
        }
    }
}
