use serde::Serialize;
use utoipa::ToSchema;

/// Response for successful authentication
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

/// Public user profile (safe for external consumption)
#[derive(Debug, Serialize, ToSchema)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub is_active: bool,
}