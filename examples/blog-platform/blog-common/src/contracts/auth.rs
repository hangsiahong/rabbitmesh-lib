use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to validate a user token
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidateTokenRequest {
    pub token: String,
}

/// Response from token validation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidateTokenResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
}

/// Public user profile for cross-service communication
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub is_active: bool,
}

/// Request to get user by ID (for service-to-service calls)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetUserRequest {
    pub user_id: String,
}