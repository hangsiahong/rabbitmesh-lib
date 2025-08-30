use rabbitmesh_macros::{
    audit_log, cached, metrics, rate_limit, require_auth, require_permission,
    service_impl, service_method, validate
};
use rabbitmesh::Message;
use serde_json::Value;
use std::sync::Arc;

use crate::{
    handler::AuthHandler,
    model::{LoginRequest, LoginResponse, ValidateTokenRequest, TokenValidationResponse, RefreshTokenRequest, RefreshTokenResponse, CheckPermissionRequest, PermissionCheckResponse, UserInfo},
};

pub struct AuthService {
    handler: Arc<AuthHandler>,
}

#[service_impl]
impl AuthService {
    pub fn new(handler: AuthHandler) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    #[service_method("POST /auth/login")]
    #[validate]
    #[rate_limit(5, 300)]  // 5 attempts per 5 minutes
    #[metrics]
    #[audit_log]
    pub async fn login(msg: Message) -> Result<Value, String> {
        let request: LoginRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // TODO: Get handler from service registry or global state
        // For now, implement the business logic directly
        
        // Basic authentication logic (should use proper password hashing in production)
        if request.email.is_empty() || request.password.is_empty() {
            return Err("Email and password are required".to_string());
        }

        // For demo purposes, accept any non-empty credentials
        let token = uuid::Uuid::new_v4().to_string();
        let refresh_token = uuid::Uuid::new_v4().to_string();
        
        let response = crate::model::LoginResponse {
            access_token: token,
            refresh_token,
            expires_in: 3600, // 1 hour
            user: crate::model::UserInfo {
                id: uuid::Uuid::new_v4().to_string(),
                email: request.email.clone(),
                name: "Demo User".to_string(),
                role: "customer".to_string(),
                permissions: vec!["orders:read".to_string(), "orders:write".to_string()],
            },
        };

        tracing::info!("User logged in: {}", request.email);
        Ok(serde_json::to_value(response).unwrap())
    }

    #[service_method("POST /auth/validate")]
    #[validate]
    #[rate_limit(100, 60)]  // 100 requests per minute
    #[cached(60)]  // Cache for 1 minute
    #[metrics]
    pub async fn validate_token(msg: Message) -> Result<Value, String> {
        let request: ValidateTokenRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // TODO: Get handler from service registry or global state
        // For now, implement the business logic directly
        
        // Basic token validation (should use proper JWT validation in production)
        if request.token.is_empty() {
            return Err("Token is required".to_string());
        }

        // For demo purposes, accept any non-empty token as valid
        let response = crate::model::TokenValidationResponse {
            valid: true,
            user: Some(crate::model::UserInfo {
                id: "demo-user-id".to_string(),
                email: "demo@example.com".to_string(),
                name: "Demo User".to_string(),
                role: "customer".to_string(),
                permissions: vec!["orders:read".to_string(), "orders:write".to_string()],
            }),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        tracing::info!("Token validated successfully");
        Ok(serde_json::to_value(response).unwrap())
    }

    #[service_method("POST /auth/refresh")]
    #[validate]
    #[rate_limit(10, 300)]  // 10 refresh attempts per 5 minutes
    #[metrics]
    #[audit_log]
    pub async fn refresh_token(msg: Message) -> Result<Value, String> {
        let request: RefreshTokenRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // TODO: Get handler from service registry or global state
        // For now, implement the business logic directly
        
        // Basic refresh token validation (should use proper validation in production)
        if request.refresh_token.is_empty() {
            return Err("Refresh token is required".to_string());
        }

        // For demo purposes, accept any non-empty refresh token
        let new_access_token = uuid::Uuid::new_v4().to_string();
        let new_refresh_token = uuid::Uuid::new_v4().to_string();
        
        let response = crate::model::RefreshTokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            expires_in: 3600, // 1 hour
        };

        tracing::info!("Token refreshed successfully");
        Ok(serde_json::to_value(response).unwrap())
    }

    #[service_method("POST /auth/check-permission")]
    #[require_auth]
    #[validate]
    #[rate_limit(200, 60)]  // 200 requests per minute
    #[cached(120)]  // Cache for 2 minutes
    #[metrics]
    pub async fn check_permission(msg: Message) -> Result<Value, String> {
        let request: CheckPermissionRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // TODO: Get handler from service registry or global state
        // For now, implement the business logic directly
        
        // Basic permission check (should use proper RBAC in production)
        let default_user_id = "demo-user".to_string();
        let user_id = msg.metadata.get("user_id")
            .unwrap_or(&default_user_id);
            
        // For demo purposes, allow common permissions
        let allowed_permissions = vec![
            "orders:read",
            "orders:write",
            "users:read",
        ];
        
        let has_permission = allowed_permissions.contains(&request.permission.as_str());
        
        let response = crate::model::PermissionCheckResponse {
            user_id: user_id.clone(),
            permission: request.permission,
            allowed: has_permission,
        };

        tracing::info!("Permission check for user: {}, allowed: {}", user_id, has_permission);
        Ok(serde_json::to_value(response).unwrap())
    }

    #[service_method("POST /auth/logout")]
    #[require_auth]
    #[rate_limit(20, 60)]
    #[metrics]
    #[audit_log]
    pub async fn logout(msg: Message) -> Result<Value, String> {
        // In a real implementation, you might want to blacklist the token
        // For now, we'll just return a success response
        Ok(serde_json::json!({
            "message": "Logged out successfully",
            "logged_out_at": chrono::Utc::now()
        }))
    }

    #[service_method("GET /auth/me")]
    #[require_auth]
    #[rate_limit(50, 60)]
    #[cached(300)]  // Cache user info for 5 minutes
    #[metrics]
    pub async fn get_current_user(msg: Message) -> Result<Value, String> {
        // Extract user info from metadata (set by auth middleware)
        let default_user_id = "demo-user-id".to_string();
        let user_id = msg.metadata.get("user_id")
            .unwrap_or(&default_user_id);
            
        // TODO: Get handler from service registry or global state
        // For now, return user info based on metadata
        let user = crate::model::UserInfo {
            id: user_id.clone(),
            email: "demo@example.com".to_string(),
            name: "Demo User".to_string(),
            role: "customer".to_string(),
            permissions: vec!["orders:read".to_string(), "orders:write".to_string()],
        };

        tracing::info!("Getting current user info for: {}", user_id);
        Ok(serde_json::to_value(user).unwrap())
    }
}