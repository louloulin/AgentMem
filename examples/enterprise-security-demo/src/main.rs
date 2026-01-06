//! Enterprise Security Demo
//!
//! Comprehensive demonstration of enterprise-grade security features including:
//! - RBAC (Role-Based Access Control)
//! - AES-256 End-to-End Encryption
//! - JWT Authentication
//! - Audit Logging
//! - Data Masking and PII Protection

use agent_mem_compat::{Mem0Client, Permission};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔒 启动企业级安全管理演示");

    // Create Mem0Client with enterprise security
    let client = Mem0Client::new().await?;

    // Demo 1: User Authentication
    println!("\n🎯 演示 1: 用户认证和会话管理");
    demo_authentication(&client).await?;

    // Demo 2: Permission-based Access Control
    println!("\n🎯 演示 2: 基于权限的访问控制");
    demo_rbac(&client).await?;

    // Demo 3: Data Encryption and Decryption
    println!("\n🎯 演示 3: 数据加密和解密");
    demo_encryption(&client).await?;

    // Demo 4: Data Masking for PII Protection
    println!("\n🎯 演示 4: 敏感数据脱敏保护");
    demo_data_masking(&client).await?;

    // Demo 5: Audit Logging
    println!("\n🎯 演示 5: 审计日志记录");
    demo_audit_logging(&client).await?;

    // Demo 6: User Management
    println!("\n🎯 演示 6: 用户管理");
    demo_user_management(&client).await?;

    println!("\n✅ 所有企业级安全演示完成！");

    println!("\n🎉 企业级安全功能特点:");
    println!("  - 🔐 RBAC 基于角色的访问控制");
    println!("  - 🛡️ AES-256 端到端加密");
    println!("  - 🎫 JWT + OAuth2 认证");
    println!("  - 📋 完整的审计日志追踪");
    println!("  - 🎭 敏感数据自动脱敏");
    println!("  - 👥 企业级用户管理");
    println!("  - 🚨 安全事件监控");
    println!("  - 🔒 IP 白名单控制");

    Ok(())
}

async fn demo_authentication(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔑 尝试用户认证...");

    // Try to authenticate with default admin user
    match client
        .authenticate(
            "admin",
            "admin123",
            "127.0.0.1",
            "Enterprise-Security-Demo/1.0",
        )
        .await
    {
        Ok(session) => {
            println!("  ✅ 认证成功!");
            println!("    👤 用户ID: {}", session.user_id);
            println!("    🎫 会话ID: {}", session.id);
            println!("    ⏰ 创建时间: {}", session.created_at);
            println!("    🌐 IP地址: {}", session.ip_address);

            // Validate the token
            match client.validate_token(&session.token).await {
                Ok(claims) => {
                    println!("  ✅ Token 验证成功!");
                    println!("    👤 用户名: {}", claims.username);
                    println!("    🎭 角色: {:?}", claims.roles);
                    println!(
                        "    ⏰ 过期时间: {}",
                        chrono::DateTime::from_timestamp(claims.exp, 0).unwrap_or_default()
                    );
                }
                Err(e) => {
                    println!("  ❌ Token 验证失败: {e}");
                }
            }

            // Logout
            match client.logout(&session.id).await {
                Ok(_) => println!("  ✅ 用户登出成功"),
                Err(e) => println!("  ❌ 登出失败: {e}"),
            }
        }
        Err(e) => {
            println!("  ❌ 认证失败: {e}");
        }
    }

    Ok(())
}

async fn demo_rbac(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🛡️ 测试基于角色的访问控制...");

    // Test different permissions
    let permissions_to_test = vec![
        Permission::ReadMemory,
        Permission::WriteMemory,
        Permission::DeleteMemory,
        Permission::SystemAdmin,
        Permission::ViewAuditLogs,
        Permission::ManageSecurity,
    ];

    for permission in permissions_to_test {
        match client.check_permission("admin", &permission).await {
            Ok(has_permission) => {
                if has_permission {
                    println!("  ✅ 权限检查通过: {permission:?}");
                } else {
                    println!("  ❌ 权限检查失败: {permission:?}");
                }
            }
            Err(e) => {
                println!("  ⚠️ 权限检查错误: {permission:?} - {e}");
            }
        }
    }

    Ok(())
}

async fn demo_encryption(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔐 测试数据加密和解密...");

    let sensitive_data = "这是一些敏感数据：用户密码 = secret123, API密钥 = sk-1234567890abcdef";
    println!("    📝 原始数据: {sensitive_data}");

    // Encrypt data
    match client.encrypt_data(sensitive_data).await {
        Ok(encrypted_data) => {
            println!("  ✅ 数据加密成功");
            println!(
                "    🔒 加密数据: {}...",
                &encrypted_data[..50.min(encrypted_data.len())]
            );

            // Decrypt data
            match client.decrypt_data(&encrypted_data).await {
                Ok(decrypted_data) => {
                    println!("  ✅ 数据解密成功");
                    println!("    🔓 解密数据: {decrypted_data}");

                    if decrypted_data == sensitive_data {
                        println!("  ✅ 加密解密验证成功！");
                    } else {
                        println!("  ❌ 加密解密验证失败！");
                    }
                }
                Err(e) => {
                    println!("  ❌ 数据解密失败: {e}");
                }
            }
        }
        Err(e) => {
            println!("  ❌ 数据加密失败: {e}");
        }
    }

    Ok(())
}

async fn demo_data_masking(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🎭 测试敏感数据脱敏...");

    let pii_data = vec![
        "我的邮箱是 john.doe@example.com，请联系我",
        "我的电话号码是 123-456-7890",
        "我的信用卡号是 1234 5678 9012 3456",
        "我的社会安全号是 123-45-6789",
        "联系信息：email: jane@company.com, phone: 987-654-3210",
    ];

    for data in pii_data {
        println!("    📝 原始数据: {data}");

        match client.mask_sensitive_data(data).await {
            Ok(masked_data) => {
                println!("    🎭 脱敏数据: {masked_data}");
            }
            Err(e) => {
                println!("    ❌ 数据脱敏失败: {e}");
            }
        }
        println!();
    }

    Ok(())
}

async fn demo_audit_logging(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 查看审计日志...");

    match client.get_audit_logs(Some(10)).await {
        Ok(logs) => {
            println!("  ✅ 获取到 {} 条审计日志", logs.len());

            for (i, log) in logs.iter().enumerate().take(5) {
                println!("    {}. 事件类型: {:?}", i + 1, log.event_type);
                println!("       用户ID: {:?}", log.user_id);
                println!("       操作: {}", log.action);
                println!("       结果: {}", if log.success { "成功" } else { "失败" });
                println!("       风险评分: {}", log.risk_score);
                println!("       时间: {}", log.timestamp);
                println!();
            }
        }
        Err(e) => {
            println!("  ❌ 获取审计日志失败: {e}");
        }
    }

    Ok(())
}

async fn demo_user_management(client: &Mem0Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("  👥 测试用户管理...");

    // Create a new user
    let username = format!("test_user_{}", &Uuid::new_v4().to_string()[..8]);
    let email = format!("{username}@example.com");
    let password = "test_password_123";
    let roles = vec!["user".to_string()];

    println!("    👤 创建新用户: {username}");

    match client.create_user(&username, &email, password, roles).await {
        Ok(user_id) => {
            println!("  ✅ 用户创建成功!");
            println!("    🆔 用户ID: {user_id}");
            println!("    📧 邮箱: {email}");
            println!("    🎭 角色: [\"user\"]");

            // Try to authenticate with the new user
            println!("    🔑 尝试新用户认证...");
            match client
                .authenticate(
                    &username,
                    password,
                    "127.0.0.1",
                    "Enterprise-Security-Demo/1.0",
                )
                .await
            {
                Ok(session) => {
                    println!("  ✅ 新用户认证成功!");
                    println!("    🎫 会话ID: {}", session.id);

                    // Test permission for regular user
                    match client
                        .check_permission(&user_id, &Permission::SystemAdmin)
                        .await
                    {
                        Ok(has_permission) => {
                            if has_permission {
                                println!("  ⚠️ 警告: 普通用户不应该有系统管理员权限!");
                            } else {
                                println!("  ✅ 权限控制正常: 普通用户没有系统管理员权限");
                            }
                        }
                        Err(e) => {
                            println!("  ❌ 权限检查失败: {e}");
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ 新用户认证失败: {e}");
                }
            }
        }
        Err(e) => {
            println!("  ❌ 用户创建失败: {e}");
        }
    }

    Ok(())
}
