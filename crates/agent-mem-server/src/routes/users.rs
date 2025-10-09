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
use agent_mem_core::storage::factory::Repositories;
use agent_mem_core::storage::models::User;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    pub timezone: Option<String>,
}

/// User login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
    pub organization_id: String,
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
    Extension(repositories): Extension<Arc<Repositories>>,
    Json(request): Json<RegisterRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Get user repository from repositories container
    let user_repo = repositories.users.clone();

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

    // Create user object
    let new_user = User::new(
        request.organization_id.clone(),
        request.name.clone(),
        request.email.clone(),
        password_hash,
        request.timezone.clone().unwrap_or_else(|| "UTC".to_string()),
    );

    // Save user to database
    let user = user_repo
        .create(&new_user)
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
        roles: user.roles.unwrap_or_else(|| vec!["user".to_string()]),
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
    Extension(repositories): Extension<Arc<Repositories>>,
    Json(request): Json<LoginRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Get user repository from repositories container
    let user_repo = repositories.users.clone();

    // Fetch user from database
    let user = user_repo
        .find_by_email(&request.email, &request.organization_id)
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
        user.roles.clone().unwrap_or_else(|| vec!["user".to_string()]),
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
            roles: user.roles.unwrap_or_else(|| vec!["user".to_string()]),
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
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
) -> ServerResult<impl IntoResponse> {
    // Get user repository from repositories container
    let user_repo = repositories.users.clone();

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
        roles: user.roles.unwrap_or_else(|| vec!["user".to_string()]),
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
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateUserRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Get user repository from repositories container
    let user_repo = repositories.users.clone();

    // Fetch current user
    let mut user = user_repo
        .find_by_id(&auth_user.user_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| ServerError::NotFound("User not found".to_string()))?;

    // Update fields if provided
    if let Some(name) = request.name {
        user.name = name;
    }
    if let Some(email) = request.email {
        user.email = email;
    }
    user.updated_at = chrono::Utc::now();
    user.last_updated_by_id = Some(auth_user.user_id.clone());

    // Update user in database
    let user = user_repo
        .update(&user)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to update user: {e}")))?;

    let response = UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        organization_id: user.organization_id,
        roles: user.roles.unwrap_or_else(|| vec!["user".to_string()]),
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
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate request
    request
        .validate()
        .map_err(|e| ServerError::BadRequest(format!("Validation error: {e}")))?;

    // Get user repository from repositories container
    let user_repo = repositories.users.clone();

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
        .update_password(&auth_user.user_id, &new_password_hash)
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
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    // Check if user is admin
    if !auth_user.roles.contains(&"admin".to_string()) {
        return Err(ServerError::Forbidden("Admin role required".to_string()));
    }

    // Get user repository from repositories container
    let user_repo = repositories.users.clone();

    // Fetch user from database
    let user_model = user_repo
        .find_by_id(&user_id)
        .await
        .map_err(|e| ServerError::Internal(format!("Database error: {e}")))?
        .ok_or_else(|| ServerError::NotFound(format!("User with id {} not found", user_id)))?;

    let user = UserResponse {
        id: user_model.id,
        email: user_model.email,
        name: user_model.name,
        organization_id: user_model.organization_id,
        roles: user_model.roles.unwrap_or_else(|| vec!["user".to_string()]),
        created_at: user_model.created_at.timestamp(),
    };

    Ok(Json(user))
}
