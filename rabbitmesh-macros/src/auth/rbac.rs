//! Role-Based Access Control (RBAC) Module
//! 
//! Provides generic RBAC implementation that works with any role system.
//! No hardcoded roles or permissions - fully configurable.

use quote::quote;

/// Generate RBAC authorization preprocessing code
pub fn generate_rbac_preprocessing(required_role: Option<&str>, required_permission: Option<&str>) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("👮 Performing RBAC authorization check");
        
        // Extract user authentication context
        let user_claims = msg.payload.as_object()
            .and_then(|obj| obj.get("_auth_user"))
            .and_then(|user| user.as_object())
            .ok_or_else(|| {
                tracing::warn!("❌ RBAC failed: User not authenticated");
                rabbitmesh::error::RabbitMeshError::Handler("Authorization failed: User not authenticated".to_string())
            })?;
        
        // Extract user roles using generic approach
        let user_roles = extract_user_roles(&user_claims.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::HashMap<String, serde_json::Value>>());
        
        let user_id = extract_user_id(&user_claims.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::HashMap<String, serde_json::Value>>())
            .unwrap_or_else(|| "unknown".to_string());
        
        tracing::debug!("👤 User {} has roles: {:?}", user_id, user_roles);
        
        // Check role-based authorization
        let mut authorized = false;
        
        // If specific role is required
        if let Some(required_role) = #required_role {
            authorized = user_roles.iter().any(|role| {
                // Support exact match and hierarchical roles
                role == required_role || 
                role.starts_with(&format!("{}:", required_role)) || // hierarchical roles like "admin:superuser"
                check_role_hierarchy(role, required_role)
            });
            
            if !authorized {
                tracing::warn!("❌ RBAC failed: User {} with roles {:?} lacks required role '{}'", 
                    user_id, user_roles, required_role);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Authorization failed: Required role '{}' not found", required_role)
                ));
            }
        }
        
        // If specific permission is required  
        if let Some(required_permission) = #required_permission {
            // Check if user has permission directly or through role-based permissions
            authorized = check_user_permission(&user_claims, required_permission, &user_roles);
            
            if !authorized {
                tracing::warn!("❌ RBAC failed: User {} lacks required permission '{}'", 
                    user_id, required_permission);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Authorization failed: Required permission '{}' not found", required_permission)
                ));
            }
        }
        
        // If no specific requirements, check for any valid role
        if #required_role.is_none() && #required_permission.is_none() {
            authorized = !user_roles.is_empty();
            
            if !authorized {
                tracing::warn!("❌ RBAC failed: User {} has no roles assigned", user_id);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    "Authorization failed: No roles assigned to user".to_string()
                ));
            }
        }
        
        tracing::debug!("✅ RBAC authorization successful for user {}", user_id);
        
        /// Check hierarchical role permissions (e.g., admin > manager > user)
        fn check_role_hierarchy(user_role: &str, required_role: &str) -> bool {
            // Define common role hierarchies - this can be configured via environment
            let hierarchy_config = std::env::var("RBAC_ROLE_HIERARCHY")
                .unwrap_or_else(|_| "admin>manager>supervisor>user,owner>member>guest".to_string());
            
            for hierarchy_chain in hierarchy_config.split(',') {
                let roles: Vec<&str> = hierarchy_chain.split('>').map(|s| s.trim()).collect();
                
                if let (Some(user_pos), Some(req_pos)) = (
                    roles.iter().position(|&r| r == user_role),
                    roles.iter().position(|&r| r == required_role)
                ) {
                    // Higher position in hierarchy (lower index) grants access
                    return user_pos <= req_pos;
                }
            }
            
            false
        }
        
        /// Check if user has specific permission
        fn check_user_permission(
            user_claims: &serde_json::Map<String, serde_json::Value>, 
            required_permission: &str, 
            user_roles: &[String]
        ) -> bool {
            // Direct permission check in user claims
            if let Some(permissions) = user_claims.get("permissions")
                .or_else(|| user_claims.get("perms"))
                .or_else(|| user_claims.get("authorities"))
                .or_else(|| user_claims.get("scopes")) {
                
                match permissions {
                    serde_json::Value::Array(arr) => {
                        if arr.iter().any(|p| p.as_str() == Some(required_permission)) {
                            return true;
                        }
                    }
                    serde_json::Value::String(s) => {
                        if s.split(',').any(|p| p.trim() == required_permission) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            
            // Role-based permission check (load from environment or config)
            let role_permissions_config = std::env::var("RBAC_ROLE_PERMISSIONS")
                .unwrap_or_else(|_| "admin:*,manager:read,write,delete,user:read".to_string());
            
            for role_perm_mapping in role_permissions_config.split(',') {
                if let Some((role, permissions)) = role_perm_mapping.split_once(':') {
                    if user_roles.contains(&role.trim().to_string()) {
                        let role_permissions: Vec<&str> = permissions.split(',').map(|s| s.trim()).collect();
                        if role_permissions.contains(&"*") || role_permissions.contains(&required_permission) {
                            return true;
                        }
                    }
                }
            }
            
            false
        }
    }
}

/// Generate role requirement preprocessing
pub fn generate_role_requirement_preprocessing(role: &str) -> proc_macro2::TokenStream {
    generate_rbac_preprocessing(Some(role), None)
}

/// Generate permission requirement preprocessing  
pub fn generate_permission_requirement_preprocessing(permission: &str) -> proc_macro2::TokenStream {
    generate_rbac_preprocessing(None, Some(permission))
}