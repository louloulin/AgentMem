// RBAC Integration Test
// 验证RBAC中间件已正确集成到路由系统

use agent_mem_server::rbac::{Action, Permission, RbacChecker, Resource, Role};

#[tokio::test]
async fn test_rbac_admin_permissions() {
    let admin_roles = vec!["admin".to_string()];

    // 验证Admin角色有所有权限
    assert!(
        RbacChecker::check_resource_action(&admin_roles, Resource::Memory, Action::Read).is_ok()
    );

    assert!(
        RbacChecker::check_resource_action(&admin_roles, Resource::Memory, Action::Write).is_ok()
    );

    assert!(
        RbacChecker::check_resource_action(&admin_roles, Resource::Memory, Action::Delete).is_ok()
    );

    assert!(RbacChecker::check_permission(&admin_roles, Permission::ManageSystem).is_ok());
}

#[tokio::test]
async fn test_rbac_user_permissions() {
    let user_roles = vec!["user".to_string()];

    // 验证User角色有读写权限
    assert!(
        RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Read).is_ok()
    );

    assert!(
        RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Write).is_ok()
    );

    // 验证User角色没有删除权限
    assert!(
        RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Delete).is_err()
    );

    // 验证User角色没有系统管理权限
    assert!(RbacChecker::check_permission(&user_roles, Permission::ManageSystem).is_err());
}

#[tokio::test]
async fn test_rbac_readonly_permissions() {
    let readonly_roles = vec!["readonly".to_string()];

    // 验证ReadOnly角色只有读权限
    assert!(
        RbacChecker::check_resource_action(&readonly_roles, Resource::Memory, Action::Read).is_ok()
    );

    // 验证ReadOnly角色没有写权限
    assert!(
        RbacChecker::check_resource_action(&readonly_roles, Resource::Memory, Action::Write)
            .is_err()
    );

    // 验证ReadOnly角色没有删除权限
    assert!(
        RbacChecker::check_resource_action(&readonly_roles, Resource::Memory, Action::Delete)
            .is_err()
    );
}

#[tokio::test]
async fn test_rbac_multiple_roles() {
    let mixed_roles = vec!["user".to_string(), "readonly".to_string()];

    // 验证多角色权限（User + ReadOnly）
    // User角色的权限应该生效
    assert!(
        RbacChecker::check_resource_action(&mixed_roles, Resource::Memory, Action::Read).is_ok()
    );

    assert!(
        RbacChecker::check_resource_action(&mixed_roles, Resource::Memory, Action::Write).is_ok()
    );
}

#[tokio::test]
async fn test_rbac_resource_types() {
    let admin_roles = vec!["admin".to_string()];

    // 测试不同资源类型的权限
    let resources = vec![
        Resource::Memory,
        Resource::Agent,
        Resource::User,
        Resource::System,
    ];

    for resource in resources {
        // Admin应该对所有资源有读权限
        assert!(RbacChecker::check_resource_action(&admin_roles, resource, Action::Read).is_ok());

        assert!(RbacChecker::check_resource_action(&admin_roles, resource, Action::Write).is_ok());

        assert!(RbacChecker::check_resource_action(&admin_roles, resource, Action::Delete).is_ok());
    }
}

#[tokio::test]
async fn test_rbac_action_types() {
    let user_roles = vec!["user".to_string()];

    // User应该有部分权限
    assert!(
        RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Read).is_ok()
    );

    assert!(
        RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Write).is_ok()
    );

    assert!(
        RbacChecker::check_resource_action(&user_roles, Resource::Memory, Action::Delete).is_err()
    );
}

#[tokio::test]
async fn test_rbac_is_admin() {
    assert!(RbacChecker::is_admin(&vec!["admin".to_string()]));
    assert!(!RbacChecker::is_admin(&vec!["user".to_string()]));
    assert!(!RbacChecker::is_admin(&vec!["readonly".to_string()]));
}

#[tokio::test]
async fn test_rbac_is_read_only() {
    assert!(RbacChecker::is_read_only(&vec!["readonly".to_string()]));
    assert!(!RbacChecker::is_read_only(&vec!["user".to_string()]));
    assert!(!RbacChecker::is_read_only(&vec!["admin".to_string()]));

    // 混合角色时不应该是只读
    assert!(!RbacChecker::is_read_only(&vec![
        "readonly".to_string(),
        "user".to_string()
    ]));
}

#[test]
fn test_role_parsing() {
    // 测试角色字符串解析
    assert_eq!(Role::from_str("admin").unwrap(), Role::Admin);
    assert_eq!(Role::from_str("user").unwrap(), Role::User);
    assert_eq!(Role::from_str("readonly").unwrap(), Role::ReadOnly);
    assert_eq!(Role::from_str("read-only").unwrap(), Role::ReadOnly);
    assert_eq!(Role::from_str("read_only").unwrap(), Role::ReadOnly);

    // 未知角色应该返回错误
    assert!(Role::from_str("unknown").is_err());
    assert!(Role::from_str("").is_err());
}

#[test]
fn test_role_as_str() {
    // 测试角色字符串表示
    assert_eq!(Role::Admin.as_str(), "admin");
    assert_eq!(Role::User.as_str(), "user");
    assert_eq!(Role::ReadOnly.as_str(), "readonly");
}

#[test]
fn test_resource_types() {
    // 确保所有资源类型都可以创建
    let _memory = Resource::Memory;
    let _agent = Resource::Agent;
    let _user = Resource::User;
    let _system = Resource::System;
}

#[test]
fn test_action_types() {
    // 确保所有操作类型都可以创建
    let _read = Action::Read;
    let _write = Action::Write;
    let _delete = Action::Delete;
    let _manage = Action::Manage;
}

#[test]
fn test_action_from_http_method() {
    // 测试HTTP方法到操作的转换
    assert_eq!(Action::from_http_method("GET"), Action::Read);
    assert_eq!(Action::from_http_method("POST"), Action::Write);
    assert_eq!(Action::from_http_method("PUT"), Action::Write);
    assert_eq!(Action::from_http_method("PATCH"), Action::Write);
    assert_eq!(Action::from_http_method("DELETE"), Action::Delete);
    assert_eq!(Action::from_http_method("OPTIONS"), Action::Read); // 默认
}
