//! Generic Input Validation Module
//! 
//! Provides comprehensive input validation that works with any data structure.
//! No hardcoded field names or validation rules.

use quote::quote;

/// Generate input validation preprocessing code
pub fn generate_input_validation() -> proc_macro2::TokenStream {
    quote! {
        /// Universal input validation function
        fn validate_input(payload: &serde_json::Value) -> Result<(), String> {
            // Load validation configuration from environment
            let validation_config = std::env::var("INPUT_VALIDATION_CONFIG")
                .unwrap_or_else(|_| "strict".to_string());
            
            match validation_config.as_str() {
                "strict" => validate_strict(payload),
                "moderate" => validate_moderate(payload),
                "permissive" => validate_permissive(payload),
                custom => validate_custom(payload, custom),
            }
        }
        
        /// Strict validation - comprehensive checks
        fn validate_strict(payload: &serde_json::Value) -> Result<(), String> {
            // Must be a JSON object
            let obj = payload.as_object()
                .ok_or_else(|| "Payload must be a JSON object".to_string())?;
            
            // Check object size limits
            let max_fields = std::env::var("MAX_FIELDS")
                .unwrap_or_else(|_| "100".to_string())
                .parse::<usize>()
                .unwrap_or(100);
            
            if obj.len() > max_fields {
                return Err(format!("Too many fields: {} (max: {})", obj.len(), max_fields));
            }
            
            // Validate each field
            for (key, value) in obj {
                validate_field_strict(key, value)?;
            }
            
            // Check for required fields based on environment config
            if let Ok(required_fields) = std::env::var("REQUIRED_FIELDS") {
                for field in required_fields.split(',') {
                    let field = field.trim();
                    if !field.is_empty() && !obj.contains_key(field) {
                        return Err(format!("Required field '{}' is missing", field));
                    }
                }
            }
            
            Ok(())
        }
        
        /// Moderate validation - balanced checks
        fn validate_moderate(payload: &serde_json::Value) -> Result<(), String> {
            if let Some(obj) = payload.as_object() {
                for (key, value) in obj {
                    validate_field_moderate(key, value)?;
                }
            }
            Ok(())
        }
        
        /// Permissive validation - minimal checks
        fn validate_permissive(payload: &serde_json::Value) -> Result<(), String> {
            if let Some(obj) = payload.as_object() {
                for (key, value) in obj {
                    validate_field_permissive(key, value)?;
                }
            }
            Ok(())
        }
        
        /// Custom validation based on configuration
        fn validate_custom(payload: &serde_json::Value, config: &str) -> Result<(), String> {
            // Parse custom validation rules from config
            // Format: field_name:rule_type:rule_value,field_name:rule_type:rule_value
            if let Some(obj) = payload.as_object() {
                for rule in config.split(',') {
                    if let Some((field_rules, _)) = rule.split_once('|') {
                        for field_rule in field_rules.split('&') {
                            if let Some((field, rule_spec)) = field_rule.split_once(':') {
                                if let Some(value) = obj.get(field.trim()) {
                                    validate_field_with_rule(field.trim(), value, rule_spec.trim())?;
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        
        /// Strict field validation
        fn validate_field_strict(key: &str, value: &serde_json::Value) -> Result<(), String> {
            // Skip internal fields
            if key.starts_with('_') {
                return Ok(());
            }
            
            // Field name validation
            if key.len() > 100 {
                return Err(format!("Field name '{}' is too long (max: 100 chars)", key));
            }
            
            if !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '[' || c == ']' || c == '.') {
                return Err(format!("Field name '{}' contains invalid characters", key));
            }
            
            match value {
                serde_json::Value::String(s) => {
                    // String validation
                    if s.len() > 10000 {
                        return Err(format!("Field '{}' exceeds maximum length (10000 chars)", key));
                    }
                    
                    // Email validation
                    if key.to_lowercase().contains("email") {
                        validate_email(s, key)?;
                    }
                    
                    // URL validation
                    if key.to_lowercase().contains("url") || key.to_lowercase().contains("link") {
                        validate_url(s, key)?;
                    }
                    
                    // Phone validation
                    if key.to_lowercase().contains("phone") || key.to_lowercase().contains("mobile") {
                        validate_phone(s, key)?;
                    }
                    
                    // SQL injection prevention
                    validate_no_sql_injection(s, key)?;
                    
                    // XSS prevention
                    validate_no_xss(s, key)?;
                }
                serde_json::Value::Number(n) => {
                    // Number validation
                    if let Some(f) = n.as_f64() {
                        if f.is_infinite() || f.is_nan() {
                            return Err(format!("Field '{}' contains invalid number", key));
                        }
                    }
                }
                serde_json::Value::Array(arr) => {
                    // Array validation
                    if arr.len() > 1000 {
                        return Err(format!("Array field '{}' has too many elements (max: 1000)", key));
                    }
                    
                    // Validate array elements
                    for (i, element) in arr.iter().enumerate() {
                        validate_field_strict(&format!("{}[{}]", key, i), element)?;
                    }
                }
                serde_json::Value::Object(obj) => {
                    // Nested object validation
                    if obj.len() > 50 {
                        return Err(format!("Nested object '{}' has too many fields (max: 50)", key));
                    }
                    
                    // Validate nested fields
                    for (nested_key, nested_value) in obj {
                        validate_field_strict(&format!("{}.{}", key, nested_key), nested_value)?;
                    }
                }
                _ => {} // Bool and null are always valid
            }
            
            Ok(())
        }
        
        /// Moderate field validation
        fn validate_field_moderate(key: &str, value: &serde_json::Value) -> Result<(), String> {
            if key.starts_with('_') {
                return Ok(());
            }
            
            match value {
                serde_json::Value::String(s) => {
                    if s.len() > 50000 {
                        return Err(format!("Field '{}' exceeds maximum length", key));
                    }
                    
                    // Basic email validation
                    if key.to_lowercase().contains("email") && !s.contains('@') {
                        return Err(format!("Field '{}' must be a valid email address", key));
                    }
                    
                    // Basic SQL injection check
                    let dangerous_patterns = ["'", "--", "/*", "*/", "xp_", "sp_"];
                    for pattern in &dangerous_patterns {
                        if s.to_lowercase().contains(pattern) {
                            return Err(format!("Field '{}' contains potentially dangerous content", key));
                        }
                    }
                }
                serde_json::Value::Array(arr) => {
                    if arr.len() > 5000 {
                        return Err(format!("Array field '{}' has too many elements", key));
                    }
                }
                _ => {}
            }
            
            Ok(())
        }
        
        /// Permissive field validation
        fn validate_field_permissive(key: &str, value: &serde_json::Value) -> Result<(), String> {
            match value {
                serde_json::Value::String(s) => {
                    if s.len() > 100000 {
                        return Err(format!("Field '{}' is too large", key));
                    }
                }
                _ => {}
            }
            Ok(())
        }
        
        /// Validate field with specific rule
        fn validate_field_with_rule(field: &str, value: &serde_json::Value, rule: &str) -> Result<(), String> {
            let rule_parts: Vec<&str> = rule.split(':').collect();
            if rule_parts.len() < 2 {
                return Ok(());
            }
            
            let rule_type = rule_parts[0];
            let rule_value = rule_parts[1];
            
            match rule_type {
                "required" => {
                    if value.is_null() {
                        return Err(format!("Field '{}' is required", field));
                    }
                }
                "min_length" => {
                    if let Some(s) = value.as_str() {
                        if let Ok(min_len) = rule_value.parse::<usize>() {
                            if s.len() < min_len {
                                return Err(format!("Field '{}' must be at least {} characters", field, min_len));
                            }
                        }
                    }
                }
                "max_length" => {
                    if let Some(s) = value.as_str() {
                        if let Ok(max_len) = rule_value.parse::<usize>() {
                            if s.len() > max_len {
                                return Err(format!("Field '{}' must be at most {} characters", field, max_len));
                            }
                        }
                    }
                }
                "pattern" => {
                    if let Some(s) = value.as_str() {
                        // Simple pattern matching (not full regex for security)
                        match rule_value {
                            "email" => validate_email(s, field)?,
                            "phone" => validate_phone(s, field)?,
                            "url" => validate_url(s, field)?,
                            "alphanumeric" => {
                                if !s.chars().all(|c| c.is_alphanumeric()) {
                                    return Err(format!("Field '{}' must contain only alphanumeric characters", field));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "range" => {
                    if let Some(n) = value.as_f64() {
                        if let Some((min_str, max_str)) = rule_value.split_once('-') {
                            if let (Ok(min), Ok(max)) = (min_str.parse::<f64>(), max_str.parse::<f64>()) {
                                if n < min || n > max {
                                    return Err(format!("Field '{}' must be between {} and {}", field, min, max));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            
            Ok(())
        }
        
        /// Email validation
        fn validate_email(email: &str, field: &str) -> Result<(), String> {
            if email.is_empty() {
                return Err(format!("Field '{}' email cannot be empty", field));
            }
            
            if !email.contains('@') {
                return Err(format!("Field '{}' must contain @ symbol", field));
            }
            
            let parts: Vec<&str> = email.split('@').collect();
            if parts.len() != 2 {
                return Err(format!("Field '{}' must have exactly one @ symbol", field));
            }
            
            let (local, domain) = (parts[0], parts[1]);
            
            if local.is_empty() || domain.is_empty() {
                return Err(format!("Field '{}' email format is invalid", field));
            }
            
            if !domain.contains('.') {
                return Err(format!("Field '{}' domain must contain a dot", field));
            }
            
            if email.len() > 254 {
                return Err(format!("Field '{}' email is too long", field));
            }
            
            Ok(())
        }
        
        /// URL validation
        fn validate_url(url: &str, field: &str) -> Result<(), String> {
            if url.is_empty() {
                return Err(format!("Field '{}' URL cannot be empty", field));
            }
            
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(format!("Field '{}' must be a valid HTTP/HTTPS URL", field));
            }
            
            if url.len() > 2000 {
                return Err(format!("Field '{}' URL is too long", field));
            }
            
            Ok(())
        }
        
        /// Phone validation
        fn validate_phone(phone: &str, field: &str) -> Result<(), String> {
            if phone.is_empty() {
                return Err(format!("Field '{}' phone cannot be empty", field));
            }
            
            let digits_only: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
            
            if digits_only.len() < 7 || digits_only.len() > 15 {
                return Err(format!("Field '{}' phone number must have 7-15 digits", field));
            }
            
            Ok(())
        }
        
        /// SQL injection validation
        fn validate_no_sql_injection(input: &str, field: &str) -> Result<(), String> {
            let dangerous_patterns = [
                "union", "select", "insert", "update", "delete", "drop", 
                "create", "alter", "exec", "execute", "script", "javascript:",
                "vbscript:", "onload", "onerror", "onclick", "'", "\"",
                "--", "/*", "*/", "xp_", "sp_", "@@", "char(", "cast(",
                "convert(", "waitfor", "delay"
            ];
            
            let input_lower = input.to_lowercase();
            for pattern in &dangerous_patterns {
                if input_lower.contains(pattern) {
                    return Err(format!("Field '{}' contains potentially dangerous content: {}", field, pattern));
                }
            }
            
            Ok(())
        }
        
        /// XSS validation
        fn validate_no_xss(input: &str, field: &str) -> Result<(), String> {
            let xss_patterns = [
                "<script", "</script>", "javascript:", "vbscript:", "onload=", 
                "onerror=", "onclick=", "onmouseover=", "onfocus=", "onblur=",
                "<iframe", "<embed", "<object", "<applet", "<meta", "data:text/html"
            ];
            
            let input_lower = input.to_lowercase();
            for pattern in &xss_patterns {
                if input_lower.contains(pattern) {
                    return Err(format!("Field '{}' contains potentially dangerous HTML/JS content", field));
                }
            }
            
            Ok(())
        }
        
        tracing::debug!("✅ Validating input");
        if let Err(validation_error) = validate_input(&msg.payload) {
            tracing::warn!("❌ Input validation failed: {}", validation_error);
            return Err(rabbitmesh::error::RabbitMeshError::Handler(format!("Validation failed: {}", validation_error)));
        }
        tracing::debug!("✅ Input validation successful");
    }
}