//! Attribute-Based Access Control (ABAC) Module
//! 
//! Provides generic ABAC implementation supporting complex policy evaluation.
//! Works with any attribute system - no hardcoded attributes or policies.

use quote::quote;

/// Generate ABAC authorization preprocessing code
pub fn generate_abac_preprocessing(policy_name: Option<&str>) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🔐 Performing ABAC authorization check");
        
        // Extract user authentication context
        let auth_user = msg.payload.as_object()
            .and_then(|obj| obj.get("_auth_user"))
            .and_then(|user| user.as_object())
            .ok_or_else(|| {
                tracing::warn!("❌ ABAC failed: User not authenticated");
                rabbitmesh::error::RabbitMeshError::Handler("Authorization failed: User not authenticated".to_string())
            })?;
        
        let user_id = auth_user.get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        // Extract user attributes
        let user_attributes = {
            let mut attributes = std::collections::HashMap::new();
            
            // Add basic user info
            if let Some(id) = auth_user.get("user_id").and_then(|v| v.as_str()) {
                attributes.insert("user.id".to_string(), serde_json::Value::String(id.to_string()));
            }
            
            // Add roles as attributes
            if let Some(roles) = auth_user.get("roles").and_then(|v| v.as_array()) {
                for role in roles {
                    if let Some(role_str) = role.as_str() {
                        attributes.insert(format!("user.role.{}", role_str), serde_json::Value::Bool(true));
                    }
                }
                attributes.insert("user.roles".to_string(), serde_json::Value::Array(roles.clone()));
            }
            
            // Add custom user attributes
            if let Some(user_attrs) = auth_user.get("attributes").and_then(|v| v.as_object()) {
                for (key, value) in user_attrs {
                    attributes.insert(format!("user.{}", key), value.clone());
                }
            }
            
            // Add JWT claims as attributes
            if let Some(claims) = auth_user.get("claims").and_then(|v| v.as_object()) {
                for (key, value) in claims {
                    if !["iat", "exp", "nbf", "aud", "iss"].contains(&key.as_str()) {
                        attributes.insert(format!("user.{}", key), value.clone());
                    }
                }
            }
            
            attributes
        };
        
        // Extract resource attributes from request
        let resource_attributes = {
            let mut attributes = std::collections::HashMap::new();
            
            if let Some(payload_obj) = msg.payload.as_object() {
                // Common resource identifier patterns
                let id_fields = ["id", "resource_id", "item_id", "entity_id", "object_id"];
                for field in &id_fields {
                    if let Some(value) = payload_obj.get(*field) {
                        attributes.insert("resource.id".to_string(), value.clone());
                        break;
                    }
                }
                
                // Resource type from service context or payload
                if let Ok(service_name) = std::env::var("SERVICE_NAME") {
                    attributes.insert("resource.type".to_string(), serde_json::Value::String(service_name));
                }
                
                // Owner information
                let owner_fields = ["owner_id", "created_by", "user_id", "owner"];
                for field in &owner_fields {
                    if let Some(value) = payload_obj.get(*field) {
                        attributes.insert("resource.owner".to_string(), value.clone());
                        break;
                    }
                }
                
                // Classification/sensitivity
                let classification_fields = ["classification", "sensitivity", "level", "security_level"];
                for field in &classification_fields {
                    if let Some(value) = payload_obj.get(*field) {
                        attributes.insert("resource.classification".to_string(), value.clone());
                        break;
                    }
                }
                
                // Project/tenant/organization context
                let context_fields = ["project", "tenant", "organization", "department", "team"];
                for field in &context_fields {
                    if let Some(value) = payload_obj.get(*field) {
                        attributes.insert(format!("resource.{}", field), value.clone());
                    }
                }
            }
            
            attributes
        };
        
        // Extract environment attributes
        let environment_attributes = {
            let mut attributes = std::collections::HashMap::new();
            
            // Current time attributes
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            attributes.insert("environment.current_time".to_string(), serde_json::Value::Number(now.into()));
            
            let hour = (now / 3600) % 24;
            attributes.insert("environment.hour".to_string(), serde_json::Value::Number(hour.into()));
            
            let day_of_week = ((now / 86400) + 4) % 7;
            attributes.insert("environment.day_of_week".to_string(), serde_json::Value::Number(day_of_week.into()));
            
            // Network/deployment context
            if let Ok(client_ip) = std::env::var("CLIENT_IP") {
                attributes.insert("environment.client_ip".to_string(), serde_json::Value::String(client_ip));
            }
            
            if let Ok(environment) = std::env::var("ENVIRONMENT") {
                attributes.insert("environment.deployment".to_string(), serde_json::Value::String(environment));
            }
            
            // Service context
            if let Ok(service_name) = std::env::var("SERVICE_NAME") {
                attributes.insert("environment.service".to_string(), serde_json::Value::String(service_name));
            }
            
            attributes
        };
        
        tracing::debug!("👤 User {} attributes: {} items", user_id, user_attributes.len());
        tracing::debug!("📦 Resource attributes: {} items", resource_attributes.len());
        tracing::debug!("🌍 Environment attributes: {} items", environment_attributes.len());
        
        // Evaluate ABAC policy
        let policy_result = if let Some(policy_name) = #policy_name {
            // Load named policy from environment
            let policy_config_key = format!("ABAC_POLICY_{}", policy_name.to_uppercase());
            match std::env::var(&policy_config_key) {
                Ok(policy_config) => {
                    evaluate_abac_policy(&policy_config, &user_attributes, &resource_attributes, &environment_attributes)
                }
                Err(_) => {
                    tracing::warn!("❌ ABAC policy '{}' not found in environment", policy_name);
                    false
                }
            }
        } else {
            // Evaluate default ABAC policy
            let default_policy = std::env::var("ABAC_DEFAULT_POLICY")
                .unwrap_or_else(|_| {
                    // Basic ownership and admin access policy
                    "user.id == resource.owner OR user.role.admin == true OR (user.department == resource.department AND user.level >= 3)".to_string()
                });
            
            evaluate_abac_policy(&default_policy, &user_attributes, &resource_attributes, &environment_attributes)
        };
        
        if !policy_result {
            tracing::warn!("❌ ABAC authorization failed for user {}", user_id);
            return Err(rabbitmesh::error::RabbitMeshError::Handler(
                "Authorization failed: ABAC policy evaluation failed".to_string()
            ));
        }
        
        tracing::info!(
            user_id = user_id,
            policy = #policy_name.unwrap_or("default"),
            user_attrs_count = user_attributes.len(),
            resource_attrs_count = resource_attributes.len(),
            env_attrs_count = environment_attributes.len(),
            "🔐 ABAC authorization successful"
        );
        
        // Function to evaluate ABAC policy expressions
        fn evaluate_abac_policy(
            policy_expression: &str,
            user_attrs: &std::collections::HashMap<String, serde_json::Value>,
            resource_attrs: &std::collections::HashMap<String, serde_json::Value>,
            env_attrs: &std::collections::HashMap<String, serde_json::Value>
        ) -> bool {
            // Combine all attributes into one namespace for evaluation
            let mut all_attrs = std::collections::HashMap::new();
            
            // Add all attribute sets
            for (key, value) in user_attrs {
                all_attrs.insert(key.clone(), value.clone());
            }
            for (key, value) in resource_attrs {
                all_attrs.insert(key.clone(), value.clone());
            }
            for (key, value) in env_attrs {
                all_attrs.insert(key.clone(), value.clone());
            }
            
            // Simple policy expression evaluator
            evaluate_policy_expression(policy_expression, &all_attrs)
        }
        
        fn evaluate_policy_expression(
            expression: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>
        ) -> bool {
            // Handle OR conditions
            for or_clause in expression.split(" OR ") {
                if evaluate_and_clause(or_clause.trim(), attributes) {
                    return true;
                }
            }
            false
        }
        
        fn evaluate_and_clause(
            clause: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>
        ) -> bool {
            for and_condition in clause.split(" AND ") {
                if !evaluate_condition(and_condition.trim(), attributes) {
                    return false;
                }
            }
            true
        }
        
        fn evaluate_condition(
            condition: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>
        ) -> bool {
            // Handle different comparison operators
            if let Some((left, right)) = condition.split_once(" == ") {
                return evaluate_equality(left.trim(), right.trim(), attributes);
            }
            if let Some((left, right)) = condition.split_once(" != ") {
                return !evaluate_equality(left.trim(), right.trim(), attributes);
            }
            if let Some((left, right)) = condition.split_once(" >= ") {
                return evaluate_comparison(left.trim(), right.trim(), attributes, |a, b| a >= b);
            }
            if let Some((left, right)) = condition.split_once(" <= ") {
                return evaluate_comparison(left.trim(), right.trim(), attributes, |a, b| a <= b);
            }
            if let Some((left, right)) = condition.split_once(" > ") {
                return evaluate_comparison(left.trim(), right.trim(), attributes, |a, b| a > b);
            }
            if let Some((left, right)) = condition.split_once(" < ") {
                return evaluate_comparison(left.trim(), right.trim(), attributes, |a, b| a < b);
            }
            
            false
        }
        
        fn evaluate_equality(
            left: &str,
            right: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>
        ) -> bool {
            let left_val = get_attribute_value(left, attributes);
            let right_val = get_attribute_value(right, attributes);
            left_val == right_val
        }
        
        fn evaluate_comparison(
            left: &str,
            right: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>,
            op: fn(f64, f64) -> bool
        ) -> bool {
            if let (Some(left_num), Some(right_num)) = (
                get_numeric_value(left, attributes),
                get_numeric_value(right, attributes)
            ) {
                op(left_num, right_num)
            } else {
                false
            }
        }
        
        fn get_attribute_value(
            key: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>
        ) -> String {
            if key.starts_with('"') && key.ends_with('"') {
                // String literal
                key[1..key.len()-1].to_string()
            } else if let Some(value) = attributes.get(key) {
                // Attribute value
                match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        }
        
        fn get_numeric_value(
            key: &str,
            attributes: &std::collections::HashMap<String, serde_json::Value>
        ) -> Option<f64> {
            if let Ok(num) = key.parse::<f64>() {
                // Numeric literal
                Some(num)
            } else if let Some(value) = attributes.get(key) {
                // Attribute value
                match value {
                    serde_json::Value::Number(n) => n.as_f64(),
                    serde_json::Value::String(s) => s.parse().ok(),
                    serde_json::Value::Bool(true) => Some(1.0),
                    serde_json::Value::Bool(false) => Some(0.0),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}