use rabbitmesh_macros::{
    audit_log, cached, metrics, rate_limit, require_auth, require_permission,
    service_impl, service_method, transactional, validate
};
use rabbitmesh::Message;
use serde_json::Value;
use std::sync::Arc;

use crate::{
    handler::UserHandler,
    model::{CreateUserRequest, UpdateUserRequest, UserResponse},
};

pub struct UserService {
    handler: Arc<UserHandler>,
}

#[service_impl]
impl UserService {
    pub fn new(handler: UserHandler) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }


    #[service_method("POST /users")]
    #[validate]
    #[rate_limit(10, 60)]
    #[metrics]
    #[audit_log]
    pub async fn create_user(msg: Message) -> Result<Value, String> {
        let request: CreateUserRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // TODO: Get handler from service registry or global state
        // For now, implement the business logic directly
        let user_id = uuid::Uuid::new_v4().to_string();
        let role = request.role.unwrap_or_else(|| "customer".to_string());
        let permissions = crate::utils::get_default_permissions(&role);
        
        let user = UserResponse {
            id: user_id,
            email: request.email,
            name: request.name,
            role,
            permissions,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        tracing::info!("Created user: {}", user.id);
        Ok(serde_json::to_value(user).unwrap())
    }

    #[service_method("GET /users/:id")]
    #[require_auth]
    #[require_permission("users:read")]
    #[cached(300)]
    #[metrics]
    pub async fn get_user(msg: Message) -> Result<Value, String> {
        // Extract user_id from metadata (set by the RPC framework)
        let user_id = msg.metadata.get("user_id")
            .or_else(|| msg.metadata.get("id"))
            .ok_or("User ID not found in request")?;
        
        tracing::info!("Getting user: {}", user_id);
        
        // TODO: Query from real database
        // For now, return a constructed response based on the ID
        let user = UserResponse {
            id: user_id.clone(),
            email: format!("user-{}@example.com", user_id),
            name: format!("User {}", user_id),
            role: "customer".to_string(),
            permissions: crate::utils::get_default_permissions("customer"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(user).unwrap())
    }

    #[service_method("PUT /users/:id")]
    #[require_auth]
    #[require_permission("users:write")]
    #[validate]
    #[rate_limit(20, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    pub async fn update_user(msg: Message) -> Result<Value, String> {
        let request: UpdateUserRequest = serde_json::from_value(msg.payload)
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // For demo purposes, return updated user
        let user = UserResponse {
            id: "demo-user-123".to_string(),
            email: "demo@example.com".to_string(),
            name: request.name.unwrap_or_else(|| "Updated Demo User".to_string()),
            role: request.role.unwrap_or_else(|| "customer".to_string()),
            permissions: request.permissions.unwrap_or_else(|| vec!["orders:read".to_string(), "orders:write".to_string()]),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(user).unwrap())
    }

    #[service_method("DELETE /users/:id")]
    #[require_auth]
    #[require_permission("users:delete")]
    #[rate_limit(5, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    pub async fn delete_user(msg: Message) -> Result<Value, String> {
        // For demo purposes, always return success
        Ok(serde_json::json!({
            "message": "User deleted successfully",
            "deleted_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    #[service_method("GET /users")]
    #[require_auth]
    #[require_permission("users:read")]
    #[cached(60)]
    #[rate_limit(50, 60)]
    #[metrics]
    pub async fn list_users(msg: Message) -> Result<Value, String> {
        let limit = msg.metadata.get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(50);
        
        let skip = msg.metadata.get("skip")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        tracing::info!("Listing users with limit: {}, skip: {}", limit, skip);

        // TODO: Query from real database
        // For now, return sample users
        let users = vec![
            UserResponse {
                id: "1".to_string(),
                email: "admin@example.com".to_string(),
                name: "Admin User".to_string(),
                role: "admin".to_string(),
                permissions: crate::utils::get_default_permissions("admin"),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            UserResponse {
                id: "2".to_string(),
                email: "customer@example.com".to_string(),
                name: "Customer User".to_string(),
                role: "customer".to_string(),
                permissions: crate::utils::get_default_permissions("customer"),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        ];

        Ok(serde_json::to_value(users).unwrap())
    }

    #[service_method("GET /users/email/:email")]
    #[require_auth]
    #[require_permission("users:read")]
    #[cached(300)]
    #[metrics]
    pub async fn get_user_by_email(msg: Message) -> Result<Value, String> {
        let email = msg.metadata.get("email")
            .ok_or("Email not provided in request")?;

        tracing::info!("Getting user by email: {}", email);

        // TODO: Query from real database
        // For now, return constructed response
        let user = crate::model::User {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.clone(),
            name: "User from Email".to_string(),
            role: "customer".to_string(),
            permissions: crate::utils::get_default_permissions("customer"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(user).unwrap())
    }
}