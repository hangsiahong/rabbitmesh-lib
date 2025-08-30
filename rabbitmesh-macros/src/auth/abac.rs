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
        let user_claims = msg.payload.as_object()
            .and_then(|obj| obj.get("_auth_user"))
            .and_then(|user| user.as_object())
            .ok_or_else(|| {
                tracing::warn!("❌ ABAC failed: User not authenticated");
                rabbitmesh::error::RabbitMeshError::Handler("Authorization failed: User not authenticated".to_string())
            })?;
        
        // Extract user attributes for ABAC evaluation
        let user_attributes = extract_user_attributes(&user_claims.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::HashMap<String, serde_json::Value>>());
        
        let user_id = extract_user_id(&user_claims.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<std::collections::HashMap<String, serde_json::Value>>())
            .unwrap_or_else(|| "unknown".to_string());
        
        // Extract resource attributes from request context
        let resource_attributes = extract_resource_attributes(&msg);
        
        // Extract environment attributes
        let environment_attributes = extract_environment_attributes();
        
        tracing::debug!("👤 User {} attributes: {:?}", user_id, user_attributes);
        tracing::debug!("📦 Resource attributes: {:?}", resource_attributes);
        tracing::debug!("🌍 Environment attributes: {:?}", environment_attributes);
        
        // Evaluate ABAC policy
        let policy_result = if let Some(policy_name) = #policy_name {
            evaluate_named_policy(policy_name, &user_attributes, &resource_attributes, &environment_attributes)
        } else {
            evaluate_default_abac_policy(&user_attributes, &resource_attributes, &environment_attributes)
        };
        
        match policy_result {
            ABACResult::Permit => {
                tracing::debug!("✅ ABAC authorization successful for user {}", user_id);
            }
            ABACResult::Deny(reason) => {
                tracing::warn!("❌ ABAC failed: User {} - {}", user_id, reason);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Authorization failed: {}", reason)
                ));
            }
            ABACResult::NotApplicable => {
                tracing::warn!("❌ ABAC policy not applicable for user {}", user_id);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    "Authorization failed: No applicable policy found".to_string()
                ));
            }
        }
        
        // ABAC evaluation result
        #[derive(Debug)]
        enum ABACResult {
            Permit,
            Deny(String),
            NotApplicable,
        }
        
        /// Extract resource attributes from the request message
        fn extract_resource_attributes(msg: &rabbitmesh::Message) -> std::collections::HashMap<String, serde_json::Value> {
            let mut attributes = std::collections::HashMap::new();
            
            // Extract resource ID from common patterns
            if let Some(obj) = msg.payload.as_object() {
                // Common resource identifier patterns
                let id_fields = ["id", "resource_id", "item_id", "entity_id", "object_id"];
                for field in &id_fields {
                    if let Some(value) = obj.get(*field) {
                        attributes.insert("resource.id".to_string(), value.clone());
                        break;
                    }
                }
                
                // Resource type from service context
                if let Some(service_name) = std::env::var("SERVICE_NAME").ok() {
                    attributes.insert("resource.type".to_string(), serde_json::Value::String(service_name));
                }
                
                // Owner information
                let owner_fields = ["owner_id", "created_by", "user_id", "owner"];
                for field in &owner_fields {
                    if let Some(value) = obj.get(*field) {
                        attributes.insert("resource.owner".to_string(), value.clone());
                        break;
                    }
                }
                
                // Classification/sensitivity
                let classification_fields = ["classification", "sensitivity", "level", "security_level"];
                for field in &classification_fields {
                    if let Some(value) = obj.get(*field) {
                        attributes.insert("resource.classification".to_string(), value.clone());
                        break;
                    }
                }
                
                // Project/tenant/organization context
                let context_fields = ["project", "tenant", "organization", "department"];
                for field in &context_fields {
                    if let Some(value) = obj.get(*field) {
                        attributes.insert(format!("resource.{}", field).as_str().to_string(), value.clone());
                    }
                }
            }
            
            attributes
        }
        
        /// Extract environment attributes
        fn extract_environment_attributes() -> std::collections::HashMap<String, serde_json::Value> {
            let mut attributes = std::collections::HashMap::new();
            
            // Current time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            attributes.insert("environment.current_time".to_string(), serde_json::Value::Number(now.into()));
            
            // Time of day (hour)
            let hour = (now / 3600) % 24;
            attributes.insert("environment.hour".to_string(), serde_json::Value::Number(hour.into()));
            
            // Day of week (0 = Sunday)
            let day_of_week = ((now / 86400) + 4) % 7; // Unix epoch was Thursday
            attributes.insert("environment.day_of_week".to_string(), serde_json::Value::Number(day_of_week.into()));
            
            // Network/IP information (if available from headers)
            if let Ok(ip) = std::env::var("CLIENT_IP") {
                attributes.insert("environment.client_ip".to_string(), serde_json::Value::String(ip));
            }
            
            // Service environment
            if let Ok(env) = std::env::var("ENVIRONMENT") {
                attributes.insert("environment.deployment".to_string(), serde_json::Value::String(env));
            }
            
            attributes
        }
        
        /// Evaluate a named ABAC policy
        fn evaluate_named_policy(
            policy_name: &str,
            user_attrs: &std::collections::HashMap<String, serde_json::Value>,
            resource_attrs: &std::collections::HashMap<String, serde_json::Value>,
            env_attrs: &std::collections::HashMap<String, serde_json::Value>
        ) -> ABACResult {
            // Load policy configuration from environment
            let policy_config_key = format!("ABAC_POLICY_{}", policy_name.to_uppercase());
            let policy_config = match std::env::var(&policy_config_key) {
                Ok(config) => config,
                Err(_) => {
                    return ABACResult::NotApplicable;
                }
            };
            
            // Parse and evaluate policy rules
            evaluate_policy_rules(&policy_config, user_attrs, resource_attrs, env_attrs)
        }
        
        /// Evaluate default ABAC policy with common patterns
        fn evaluate_default_abac_policy(
            user_attrs: &std::collections::HashMap<String, serde_json::Value>,
            resource_attrs: &std::collections::HashMap<String, serde_json::Value>,
            env_attrs: &std::collections::HashMap<String, serde_json::Value>
        ) -> ABACResult {
            // Default policy rules (configurable via environment)
            let default_policy = std::env::var("ABAC_DEFAULT_POLICY")
                .unwrap_or_else(|_| {
                    // Basic ownership and admin access policy
                    "user.id == resource.owner OR user.role == 'admin' OR (user.department == resource.department AND user.level >= 3)".to_string()
                });
            
            evaluate_policy_rules(&default_policy, user_attrs, resource_attrs, env_attrs)
        }
        
        /// Evaluate policy rules using a simple expression evaluator
        fn evaluate_policy_rules(
            policy_rules: &str,
            user_attrs: &std::collections::HashMap<String, serde_json::Value>,
            resource_attrs: &std::collections::HashMap<String, serde_json::Value>,
            env_attrs: &std::collections::HashMap<String, serde_json::Value>
        ) -> ABACResult {
            // Simple policy rule evaluator
            // In production, you'd use a proper policy engine like OPA
            // What about https://docs.rs/rusty-rules/latest/rusty_rules ?
            // https://crates.io/crates/evaluator_rs
            
            // Combine all attributes into one namespace
            let mut all_attrs = std::collections::HashMap::new();
            
            // Add user attributes with user. prefix
            for (key, value) in user_attrs {
                all_attrs.insert(format!("user.{}", key), value);
            }
            
            // Add resource attributes with resource. prefix
            for (key, value) in resource_attrs {
                all_attrs.insert(key.clone(), value);
            }
            
            // Add environment attributes
            for (key, value) in env_attrs {
                all_attrs.insert(key.clone(), value);
            }
            
            // Simple expression evaluation
            if evaluate_simple_expression(policy_rules, &all_attrs) {
                ABACResult::Permit
            } else {
                ABACResult::Deny("Policy evaluation failed".to_string())
            }
        }
        
        /// Simple expression evaluator for ABAC policies
        fn evaluate_simple_expression(
            expression: &str,
            attributes: &std::collections::HashMap<String, &serde_json::Value>
        ) -> bool {
            // Handle OR conditions
            for or_clause in expression.split(" OR ") {
                if evaluate_and_clause(or_clause.trim(), attributes) {
                    return true;
                }
            }
            false
        }
        
        /// Evaluate AND clause
        fn evaluate_and_clause(
            clause: &str,
            attributes: &std::collections::HashMap<String, &serde_json::Value>
        ) -> bool {
            for and_condition in clause.split(" AND ") {
                if !evaluate_condition(and_condition.trim(), attributes) {
                    return false;
                }
            }
            true
        }
        
        /// Evaluate individual condition
        fn evaluate_condition(
            condition: &str,
            attributes: &std::collections::HashMap<String, &serde_json::Value>
        ) -> bool {
            // Handle different comparison operators
            if let Some((left, right)) = condition.split_once(" == ") {
                return evaluate_equality(left.trim(), right.trim(), attributes);
            }
            if let Some((left, right)) = condition.split_once(" >= ") {
                return evaluate_greater_equal(left.trim(), right.trim(), attributes);
            }
            if let Some((left, right)) = condition.split_once(" <= ") {
                return evaluate_less_equal(left.trim(), right.trim(), attributes);
            }
            if let Some((left, right)) = condition.split_once(" > ") {
                return evaluate_greater(left.trim(), right.trim(), attributes);
            }
            if let Some((left, right)) = condition.split_once(" < ") {
                return evaluate_less(left.trim(), right.trim(), attributes);
            }
            
            false
        }
        
        /// Evaluate equality condition
        fn evaluate_equality(left: &str, right: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> bool {
            let left_val = get_attribute_value(left, attributes);
            let right_val = get_attribute_value(right, attributes);
            left_val == right_val
        }
        
        /// Evaluate greater than or equal condition
        fn evaluate_greater_equal(left: &str, right: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> bool {
            if let (Some(left_num), Some(right_num)) = (
                get_numeric_value(left, attributes),
                get_numeric_value(right, attributes)
            ) {
                left_num >= right_num
            } else {
                false
            }
        }
        
        /// Evaluate less than or equal condition
        fn evaluate_less_equal(left: &str, right: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> bool {
            if let (Some(left_num), Some(right_num)) = (
                get_numeric_value(left, attributes),
                get_numeric_value(right, attributes)
            ) {
                left_num <= right_num
            } else {
                false
            }
        }
        
        /// Evaluate greater than condition
        fn evaluate_greater(left: &str, right: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> bool {
            if let (Some(left_num), Some(right_num)) = (
                get_numeric_value(left, attributes),
                get_numeric_value(right, attributes)
            ) {
                left_num > right_num
            } else {
                false
            }
        }
        
        /// Evaluate less than condition
        fn evaluate_less(left: &str, right: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> bool {
            if let (Some(left_num), Some(right_num)) = (
                get_numeric_value(left, attributes),
                get_numeric_value(right, attributes)
            ) {
                left_num < right_num
            } else {
                false
            }
        }
        
        /// Get attribute value as string
        fn get_attribute_value(key: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> String {
            if key.starts_with("'") && key.ends_with("'") {
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
        
        /// Get numeric value for comparisons
        fn get_numeric_value(key: &str, attributes: &std::collections::HashMap<String, &serde_json::Value>) -> Option<f64> {
            if let Ok(num) = key.parse::<f64>() {
                // Numeric literal
                Some(num)
            } else if let Some(value) = attributes.get(key) {
                // Attribute value
                match value {
                    serde_json::Value::Number(n) => n.as_f64(),
                    serde_json::Value::String(s) => s.parse().ok(),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}