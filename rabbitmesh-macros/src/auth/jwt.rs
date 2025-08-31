//! Generic JWT Authentication Module
//! 
//! Provides universal JWT validation that works with any project's JWT format.
//! No hardcoded claims or service-specific logic.

use quote::quote;
use serde_json::Value;
use std::collections::HashMap;

/// Generate universal JWT validator function - works with ANY project type
pub fn generate_jwt_validator() -> proc_macro2::TokenStream {
    quote! {
        /// Universal JWT token validation - works with any project's JWT format
        /// 
        /// Supports any JWT structure and extracts claims generically without
        /// hardcoding specific claim names or user structures.
        fn validate_jwt_token(token: &str) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
            use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::ErrorKind};
            use serde_json::Value;
            
            // Universal JWT secret - configurable via environment
            let jwt_secret = std::env::var("JWT_SECRET")
                .or_else(|_| std::env::var("JWT_SIGNING_KEY"))
                .or_else(|_| std::env::var("AUTH_SECRET"))
                .unwrap_or_else(|_| {
                    tracing::warn!("No JWT secret found in environment. Using default for development.");
                    "development-secret-key".to_string()
                });
            
            let key = DecodingKey::from_secret(jwt_secret.as_ref());
            
            // Universal validation settings - accepts any valid JWT
            let mut validation = Validation::new(Algorithm::HS256);
            validation.validate_exp = true;  // Check expiration
            validation.validate_aud = false; // Don't validate audience - universal
            validation.validate_nbf = true;  // Check not-before
            
            // Try different algorithms if HS256 fails (generic approach)
            let algorithms = [Algorithm::HS256, Algorithm::HS384, Algorithm::HS512];
            let mut last_error = None;
            
            for alg in &algorithms {
                validation.algorithms = vec![*alg];
                match decode::<Value>(token, &key, &validation) {
                    Ok(token_data) => {
                        // Extract claims as generic HashMap - works with any user data structure
                        let mut claims = std::collections::HashMap::new();
                        if let Value::Object(claim_map) = token_data.claims {
                            for (key, value) in claim_map {
                                claims.insert(key, value);
                            }
                        }
                        
                        // Ensure we have some form of user identifier (universal check)
                        let has_user_id = claims.contains_key("sub") || 
                                         claims.contains_key("user_id") || 
                                         claims.contains_key("id") ||
                                         claims.contains_key("username") ||
                                         claims.contains_key("user") ||
                                         claims.contains_key("uid");
                        
                        if !has_user_id {
                            return Err("JWT token missing user identifier (sub, user_id, id, username, user, or uid)".to_string());
                        }
                        
                        return Ok(claims);
                    }
                    Err(err) => {
                        last_error = Some(err);
                        continue;
                    }
                }
            }
            
            // If all algorithms failed, return the error
            if let Some(err) = last_error {
                return Err(match err.kind() {
                    ErrorKind::ExpiredSignature => "JWT token has expired".to_string(),
                    ErrorKind::InvalidSignature => "Invalid JWT signature".to_string(),
                    ErrorKind::InvalidToken => "Invalid JWT token format".to_string(),
                    ErrorKind::InvalidIssuer => "Invalid JWT issuer".to_string(),
                    ErrorKind::InvalidAudience => "Invalid JWT audience".to_string(),
                    ErrorKind::InvalidSubject => "Invalid JWT subject".to_string(),
                    ErrorKind::ImmatureSignature => "JWT token not yet valid (nbf claim)".to_string(),
                    _ => format!("JWT validation error: {}", err),
                });
            }
            
            Err("Failed to validate JWT with any supported algorithm".to_string())
        }
        
        /// Extract user identifier from JWT claims (generic approach)
        fn extract_user_id(claims: &std::collections::HashMap<String, serde_json::Value>) -> Option<String> {
            // Try multiple common user ID claim names
            let id_fields = ["sub", "user_id", "id", "username", "user", "uid", "email"];
            
            for field in &id_fields {
                if let Some(value) = claims.get(*field) {
                    match value {
                        serde_json::Value::String(s) => return Some(s.clone()),
                        serde_json::Value::Number(n) => return Some(n.to_string()),
                        _ => continue,
                    }
                }
            }
            None
        }
        
        /// Extract user roles from JWT claims (generic approach)
        fn extract_user_roles(claims: &std::collections::HashMap<String, serde_json::Value>) -> Vec<String> {
            // Try multiple common role claim names
            let role_fields = ["roles", "role", "authorities", "permissions", "scopes", "groups"];
            
            for field in &role_fields {
                if let Some(value) = claims.get(*field) {
                    match value {
                        serde_json::Value::Array(arr) => {
                            return arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                        }
                        serde_json::Value::String(s) => {
                            // Handle comma-separated roles
                            return s.split(',')
                                .map(|role| role.trim().to_string())
                                .filter(|role| !role.is_empty())
                                .collect();
                        }
                        _ => continue,
                    }
                }
            }
            Vec::new()
        }
        
        /// Extract user attributes from JWT claims (for ABAC)
        fn extract_user_attributes(claims: &std::collections::HashMap<String, serde_json::Value>) -> std::collections::HashMap<String, serde_json::Value> {
            let mut attributes = std::collections::HashMap::new();
            
            // Extract common attribute fields
            let attr_fields = [
                "department", "organization", "team", "location", "country",
                "level", "clearance", "project", "client", "tenant",
                "age", "experience", "certification", "title", "rank"
            ];
            
            for field in &attr_fields {
                if let Some(value) = claims.get(*field) {
                    attributes.insert(field.to_string(), value.clone());
                }
            }
            
            // Also include any custom attributes with specific prefixes
            for (key, value) in claims {
                if key.starts_with("attr_") || key.starts_with("custom_") || key.starts_with("ext_") {
                    attributes.insert(key.clone(), value.clone());
                }
            }
            
            attributes
        }
    }
}

/// Generate JWT authentication preprocessing code
pub fn generate_jwt_auth_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🔐 Validating JWT authentication");
        
        // Extract JWT token from message payload headers (universal approach)
        let auth_token = msg.payload.as_object()
            .and_then(|obj| obj.get("_headers"))
            .and_then(|headers| headers.as_object())
            .and_then(|headers_obj| headers_obj.get("authorization"))
            .and_then(|auth_header| auth_header.as_str())
            .and_then(|auth_str| {
                if auth_str.starts_with("Bearer ") {
                    Some(&auth_str[7..]) // Remove "Bearer " prefix
                } else if auth_str.starts_with("JWT ") {
                    Some(&auth_str[4..]) // Remove "JWT " prefix
                } else {
                    Some(auth_str) // Raw token
                }
            })
            .or_else(|| {
                // Fallback: try to extract from message payload if it contains auth info
                msg.payload.as_object()
                    .and_then(|obj| obj.get("auth_token").or(obj.get("token")).or(obj.get("jwt")))
                    .and_then(|token| token.as_str())
            });
        
        let auth_token = auth_token.ok_or_else(|| {
            tracing::warn!("❌ Authentication failed: Missing or invalid Authorization header");
            rabbitmesh::error::RabbitMeshError::Handler("Authentication required: Missing Authorization header".to_string())
        })?;
        
        // Universal JWT validation - works with ANY project's JWT format
        let user_claims = match validate_jwt_token(auth_token) {
            Ok(claims) => {
                let user_id = extract_user_id(&claims).unwrap_or_else(|| "unknown".to_string());
                tracing::debug!("✅ JWT authentication successful for user: {}", user_id);
                claims
            }
            Err(auth_error) => {
                tracing::warn!("❌ JWT validation failed: {}", auth_error);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(format!("Authentication failed: {}", auth_error)));
            }
        };
        
        // Store user info in message context for use by business logic and authorization
        if let Ok(mut payload) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(msg.payload.clone()) {
            // Create structured auth context with all user information
            let auth_context = serde_json::json!({
                "user_id": extract_user_id(&user_claims).unwrap_or_else(|| "unknown".to_string()),
                "roles": extract_user_roles(&user_claims),
                "attributes": extract_user_attributes(&user_claims),
                "claims": user_claims,
                "auth_method": "jwt",
                "authenticated_at": chrono::Utc::now().timestamp()
            });
            
            payload.insert("_auth_user".to_string(), auth_context);
            
            // Update the actual message payload with enhanced auth context
            msg.payload = serde_json::Value::Object(payload);
            
            // Log successful authentication with metrics
            tracing::info!(
                user_id = extract_user_id(&user_claims).unwrap_or_else(|| "unknown".to_string()),
                roles_count = extract_user_roles(&user_claims).len(),
                attributes_count = extract_user_attributes(&user_claims).len(),
                "🔐 JWT authentication context added to message"
            );
            
            // Increment authentication success metrics
            #[cfg(feature = "metrics")]
            {
                if let Some(registry) = crate::observability::metrics::get_metrics_registry() {
                    registry.increment_counter("auth_jwt_success_total", &[]);
                }
            }
        } else {
            tracing::error!("❌ Failed to parse message payload as JSON object for auth context injection");
            
            // Increment authentication error metrics  
            #[cfg(feature = "metrics")]
            {
                if let Some(registry) = crate::observability::metrics::get_metrics_registry() {
                    registry.increment_counter("auth_jwt_payload_error_total", &[]);
                }
            }
            
            return Err(rabbitmesh::error::RabbitMeshError::Handler(
                "Failed to inject authentication context: Invalid message payload format".to_string()
            ));
        }
    }
}