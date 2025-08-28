use serde::{Deserialize, Serialize};

/// Internal User model for auth service database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,  // Internal - never exposed
    pub created_at: String,
    pub is_active: bool,
}

impl User {
    /// Convert internal User to external UserProfile (removes sensitive data)
    pub fn to_profile(&self) -> crate::models::UserProfile {
        crate::models::UserProfile {
            id: self.id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            created_at: self.created_at.clone(),
            is_active: self.is_active,
        }
    }
}