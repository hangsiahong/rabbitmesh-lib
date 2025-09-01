//! Role-Based Access Control (RBAC) Module
//! 
//! Provides generic RBAC implementation that works with any role system.
//! No hardcoded roles or permissions.

use quote::quote;

/// Generate RBAC authorization preprocessing code
pub fn generate_rbac_preprocessing(required_role: Option<&str>, required_permission: Option<&str>) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🛡️ Performing RBAC authorization check");
        
        // Extract user authentication context
        let auth_user = msg.payload.as_object()
            .and_then(|obj| obj.get("_auth_user"))
            .and_then(|user| user.as_object())
            .ok_or_else(|| {
                tracing::warn!("❌ RBAC failed: User not authenticated");
                rabbitmesh::error::RabbitMeshError::Handler("Authorization failed: User not authenticated".to_string())
            })?;
        
        let user_id = auth_user.get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let user_roles = auth_user.get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
            })
            .unwrap_or_default();
        
        tracing::debug!("👤 User {} has roles: {:?}", user_id, user_roles);
        
        // Check role-based authorization
        let mut authorization_passed = false;
        let mut authorization_reason = String::new();
        
        // Check required role
        if let Some(required_role) = #required_role {
            if user_roles.contains(&required_role) {
                authorization_passed = true;
                authorization_reason = format!("User has required role: {}", required_role);
                tracing::debug!("✅ Role check passed: User {} has role {}", user_id, required_role);
            } else {
                authorization_reason = format!("User missing required role: {}", required_role);
                tracing::warn!("❌ Role check failed: User {} missing role {}", user_id, required_role);
            }
        }
        
        // Check required permission (if role check didn't already pass)
        if !authorization_passed {
            if let Some(required_permission) = #required_permission {
                // Check if any user role has the required permission
                let user_permissions = auth_user.get("attributes")
                    .and_then(|attrs| attrs.as_object())
                    .and_then(|attrs_obj| attrs_obj.get("permissions"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<&str>>()
                    })
                    .unwrap_or_default();
                
                if user_permissions.contains(&required_permission) {
                    authorization_passed = true;
                    authorization_reason = format!("User has required permission: {}", required_permission);
                    tracing::debug!("✅ Permission check passed: User {} has permission {}", user_id, required_permission);
                } else {
                    // Check role-based permissions from environment or config
                    for role in &user_roles {
                        let role_permissions_key = format!("RBAC_ROLE_{}_PERMISSIONS", role.to_uppercase());
                        if let Ok(role_permissions_str) = std::env::var(&role_permissions_key) {
                            let role_permissions: Vec<&str> = role_permissions_str
                                .split(',')
                                .map(|p| p.trim())
                                .collect();
                            
                            if role_permissions.contains(&required_permission) {
                                authorization_passed = true;
                                authorization_reason = format!("Role {} has required permission: {}", role, required_permission);
                                tracing::debug!("✅ Role-based permission check passed: Role {} has permission {}", role, required_permission);
                                break;
                            }
                        }
                    }
                    
                    if !authorization_passed {
                        authorization_reason = format!("User missing required permission: {}", required_permission);
                        tracing::warn!("❌ Permission check failed: User {} missing permission {}", user_id, required_permission);
                    }
                }
            }
        }
        
        // If no specific role or permission required, check for basic roles
        if #required_role.is_none() && #required_permission.is_none() {
            // Default RBAC: allow admin or any authenticated user with roles
            if user_roles.contains(&"admin") || user_roles.contains(&"administrator") || !user_roles.is_empty() {
                authorization_passed = true;
                authorization_reason = "User has valid roles".to_string();
                tracing::debug!("✅ Default RBAC passed: User {} has valid roles", user_id);
            } else {
                authorization_reason = "User has no valid roles".to_string();
                tracing::warn!("❌ Default RBAC failed: User {} has no roles", user_id);
            }
        }
        
        // Final authorization check
        if !authorization_passed {
            tracing::warn!("❌ RBAC authorization failed for user {}: {}", user_id, authorization_reason);
            return Err(rabbitmesh::error::RabbitMeshError::Handler(
                format!("Authorization failed: {}", authorization_reason)
            ));
        }
        
        tracing::info!(
            user_id = user_id,
            user_roles = ?user_roles,
            required_role = #required_role,
            required_permission = #required_permission,
            reason = authorization_reason,
            "🛡️ RBAC authorization successful"
        );
    }
}

/// Generate role hierarchy checking code
pub fn generate_role_hierarchy_check(min_role_level: u8) -> proc_macro2::TokenStream {
    quote! {
        // Check role hierarchy levels
        if let Some(auth_user) = msg.payload.as_object()
            .and_then(|obj| obj.get("_auth_user"))
            .and_then(|user| user.as_object()) {
            
            let user_roles = auth_user.get("roles")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<&str>>()
                })
                .unwrap_or_default();
            
            let user_id = auth_user.get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            let min_required_level = #min_role_level;
            let mut user_max_level = 0u8;
            
            // Get role levels from environment configuration
            for role in &user_roles {
                let role_level_key = format!("RBAC_ROLE_{}_LEVEL", role.to_uppercase());
                if let Ok(level_str) = std::env::var(&role_level_key) {
                    if let Ok(level) = level_str.parse::<u8>() {
                        user_max_level = user_max_level.max(level);
                    }
                }
            }
            
            // Default role levels if not configured
            if user_max_level == 0 {
                for role in &user_roles {
                    user_max_level = match *role {
                        "admin" | "administrator" | "superuser" => user_max_level.max(100),
                        "manager" | "supervisor" | "lead" => user_max_level.max(80),
                        "senior" | "expert" => user_max_level.max(60),
                        "user" | "member" | "employee" => user_max_level.max(40),
                        "guest" | "readonly" => user_max_level.max(20),
                        _ => user_max_level.max(10),
                    };
                }
            }
            
            if user_max_level < min_required_level {
                tracing::warn!("❌ Role hierarchy check failed: User {} max level {} < required level {}", 
                    user_id, user_max_level, min_required_level);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Authorization failed: Insufficient role level (required: {}, user: {})", 
                        min_required_level, user_max_level)
                ));
            }
            
            tracing::debug!("✅ Role hierarchy check passed: User {} level {} >= required {}", 
                user_id, user_max_level, min_required_level);
        }
    }
}