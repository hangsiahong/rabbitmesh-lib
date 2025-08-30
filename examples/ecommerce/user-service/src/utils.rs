use bcrypt::{hash, verify, DEFAULT_COST};
use anyhow::Result;

pub fn hash_password(password: &str) -> Result<String> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let is_valid = verify(password, hash)?;
    Ok(is_valid)
}

pub fn get_default_permissions(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "users:read".to_string(),
            "users:write".to_string(),
            "users:delete".to_string(),
            "orders:read".to_string(),
            "orders:write".to_string(),
            "orders:delete".to_string(),
        ],
        "manager" => vec![
            "users:read".to_string(),
            "users:write".to_string(),
            "orders:read".to_string(),
            "orders:write".to_string(),
        ],
        "customer" => vec![
            "orders:read".to_string(),
            "orders:write".to_string(),
        ],
        _ => vec!["orders:read".to_string()],
    }
}