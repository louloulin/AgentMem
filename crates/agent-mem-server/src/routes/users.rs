//! User management API routes
//!
//! This module provides REST API endpoints for user management:
//! - User registration
//! - User login
//! - User profile management
//! - Password management

use crate::auth::{AuthService, PasswordService};
use crate::error::{ServerError, ServerResult};
use crate::middleware::{log_security_event, AuthUser, SecurityEvent};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use validator::Validate;

/// User registration request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 2))]
    pub name: String,
    pub organization_id: String,
}

/// User login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

/// User login response
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

/// User response
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub organization_id: String,
    pub roles: Vec<String>,
    pub created_at: i64,
}

/// Update user request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}

/// Change password request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/v1/users/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "User already exists")
    ),
    tag = "users"
)]
pub async fn register_user(
    Extension(db_pool): Extension<PgPool>,
    Json(request): Json<RegisterRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Create user repository
    let user_repo = agent_mem_core::storage::user_repository::UserRepository::new(db_pool);

    // Check if user already exists
    let exists = user_repo
        .email_exists(&request.email, &request.organization_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?;

    if exists {
        return Err(ServerError::BadRequest(format!(
            "User with email {} already exists",
            request.email
        )));
    }

    // Hash password
    let password_hash = PasswordService::hash_password(&request.password)?;

    // Save user to database
    let user = user_repo
        .create(
            &request.organization_id,
            &request.email,
            &password_hash,
            &request.name,
            vec!["user".to_string()],
            None,
        )
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to create user: {e}")))?;

    // Log security event
    log_security_event(SecurityEvent::LoginSuccess {
        user_id: user.id.clone(),
        ip_address: None,
    });

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        organization_id: user.organization_id,
        roles: user.roles,
        created_at: user.created_at.timestamp(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Login user
#[utoipa::path(
    post,
    path = "/api/v1/users/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "users"
)]
pub async fn login_user(
    Extension(db_pool): Extension<PgPool>,
    Json(request): Json<LoginRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Create user repository
    let user_repo = agent_mem_core::storage::user_repository::UserRepository::new(db_pool);

    // Fetch user from database
    let user = user_repo
        .find_by_email(&request.email)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| {
            log_security_event(SecurityEvent::LoginFailure {
                email: request.email.clone(),
                ip_address: None,
                reason: "User not found".to_string(),
            });
            ServerError::Unauthorized("Invalid email or password".to_string())
        })?;

    // Verify password
    let valid = PasswordService::verify_password(&request.password, &user.password_hash)?;
    if !valid {
        log_security_event(SecurityEvent::LoginFailure {
            email: request.email.clone(),
            ip_address: None,
            reason: "Invalid password".to_string(),
        });
        return Err(ServerError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    // Generate JWT token
    let auth_service = AuthService::new("default-secret-key-change-in-production");
    let token = auth_service.generate_token(
        &user.id,
        user.organization_id.clone(),
        user.roles.clone(),
        None,
    )?;

    // Log successful login
    log_security_event(SecurityEvent::LoginSuccess {
        user_id: user.id.clone(),
        ip_address: None,
    });

    let response = LoginResponse {
        token,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            organization_id: user.organization_id,
            roles: user.roles,
            created_at: user.created_at.timestamp(),
        },
    };

    Ok(Json(response))
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 401, description = "Not authenticated")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> ServerResult<impl IntoResponse> {
    // Create user repository
    let user_repo = agent_mem_core::storage::user_repository::UserRepository::new(db_pool);

    // Fetch user from database
    let user = user_repo
        .find_by_id(&auth_user.user_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| ServerError::NotFound("User not found".to_string()))?;

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        organization_id: user.organization_id,
        roles: user.roles,
        created_at: user.created_at.timestamp(),
    };

    Ok(Json(response))
}

/// Update user profile
#[utoipa::path(
    put,
    path = "/api/v1/users/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 400, description = "Invalid request")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_current_user(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateUserRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Create user repository
    let user_repo = agent_mem_core::storage::user_repository::UserRepository::new(db_pool);

    // Update user in database
    let user = user_repo
        .update(
            &auth_user.user_id,
            request.name.as_deref(),
            request.email.as_deref(),
            None, // Don't update roles here
            None, // Don't update status here
            Some(&auth_user.user_id),
        )
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to update user: {e}")))?;

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        organization_id: user.organization_id,
        roles: user.roles,
        created_at: user.created_at.timestamp(),
    };

    Ok(Json(response))
}

/// Change user password
#[utoipa::path(
    post,
    path = "/api/v1/users/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 401, description = "Not authenticated or invalid current password"),
        (status = 400, description = "Invalid request")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password(
    Extension(db_pool): Extension<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Create user repository
    let user_repo = agent_mem_core::storage::user_repository::UserRepository::new(db_pool);

    // Fetch user to verify current password
    let user = user_repo
        .find_by_id(&auth_user.user_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| ServerError::NotFound("User not found".to_string()))?;

    // Verify current password
    let valid = PasswordService::verify_password(&request.current_password, &user.password_hash)?;
    if !valid {
        return Err(ServerError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }

    // Hash new password
    let new_password_hash = PasswordService::hash_password(&request.new_password)?;

    // Update password in database
    user_repo
        .update_password(
            &auth_user.user_id,
            &new_password_hash,
            Some(&auth_user.user_id),
        )
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to update password: {e}")))?;

    // Log security event
    log_security_event(SecurityEvent::PasswordChanged {
        user_id: auth_user.user_id.clone(),
    });

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Password changed successfully"
        })),
    ))
}

/// Get user by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found")
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_id(
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user is admin
    if !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden("Admin role required".to_string()));
    }

    // TODO: Fetch user from database
    // For now, return mock data
    let user = UserResponse {
        id: user_id,
        email: "user@example.com".to_string(),
        name: "Test User".to_string(),
        organization_id: auth_user.org_id.clone(),
        roles: vec!["user".to_string()],
        created_at: chrono::Utc::now().timestamp(),
    };

    Ok(Json(user))
}
