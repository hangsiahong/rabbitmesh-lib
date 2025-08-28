use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn generate_user_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_token(user_id: &str) -> String {
    // Simple token generation (in production, use proper JWT)
    format!("token_{}_{}", user_id, Uuid::new_v4())
}

pub fn validate_token(token: &str) -> Option<String> {
    // Simple token validation (in production, use proper JWT verification)
    if token.starts_with("token_") {
        let parts: Vec<&str> = token.split('_').collect();
        if parts.len() >= 2 {
            return Some(parts[1].to_string());
        }
    }
    None
}