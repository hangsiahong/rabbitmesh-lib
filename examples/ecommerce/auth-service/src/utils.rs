use anyhow::{Result, anyhow};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use bcrypt::verify;

use crate::model::Claims;

pub struct JwtUtils {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_hours: i64,
}

impl JwtUtils {
    pub fn new(secret: &str, expiration_hours: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            expiration_hours: expiration_hours as i64,
        }
    }

    pub fn create_token(&self, user_id: &str, email: &str, name: &str, role: &str, permissions: Vec<String>) -> Result<String> {
        let now = Utc::now();
        let exp = (now + Duration::hours(self.expiration_hours)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            role: role.to_string(),
            permissions,
            exp,
            iat,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to create token: {}", e))?;

        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow!("Failed to validate token: {}", e))?;

        Ok(token_data.claims)
    }

    pub fn create_refresh_token(&self, user_id: &str) -> Result<String> {
        let now = Utc::now();
        let exp = (now + Duration::days(30)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = serde_json::json!({
            "sub": user_id,
            "type": "refresh",
            "exp": exp,
            "iat": iat
        });

        // Use a simple approach for refresh tokens
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to create refresh token: {}", e))?;

        Ok(token)
    }
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let is_valid = verify(password, hash)?;
    Ok(is_valid)
}

pub fn has_permission(user_permissions: &[String], required_permission: &str) -> bool {
    // Check for exact permission match
    if user_permissions.contains(&required_permission.to_string()) {
        return true;
    }

    // Check for wildcard permissions
    let parts: Vec<&str> = required_permission.split(':').collect();
    if parts.len() == 2 {
        let resource = parts[0];
        let wildcard_permission = format!("{}:*", resource);
        if user_permissions.contains(&wildcard_permission) {
            return true;
        }
    }

    // Check for admin role (has all permissions)
    user_permissions.contains(&"*".to_string())
}

pub fn has_role(user_role: &str, required_roles: &[&str]) -> bool {
    required_roles.contains(&user_role)
}