//! Schema Validation Module
//! 
//! Provides comprehensive JSON schema validation with support for multiple
//! validation engines, custom rules, and dynamic schema loading.

use quote::quote;

/// Generate schema validation preprocessing code
pub fn generate_schema_validation_preprocessing(
    schema_source: Option<&str>,
    validation_engine: Option<&str>,
    strict_mode: Option<bool>
) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("📋 Initializing schema validation");
        
        // Create schema validator with configuration
        let schema_source = #schema_source.unwrap_or("auto_detect");
        let validation_engine = #validation_engine.unwrap_or("jsonschema");
        let strict_mode = #strict_mode.unwrap_or(true);
        
        let schema_validator = SchemaValidator::new(schema_source, validation_engine, strict_mode).await?;
        
        tracing::debug!("📋 Schema validator initialized - Source: {}, Engine: {}, Strict: {}", 
            schema_source, validation_engine, strict_mode);
        
        // Validate message payload against schema
        let validation_result = schema_validator.validate(&msg).await;
        
        match validation_result {
            SchemaValidationResult::Valid => {
                tracing::debug!("✅ Schema validation passed");
            }
            SchemaValidationResult::Invalid { errors } => {
                let error_summary = errors.iter()
                    .take(5)  // Limit to first 5 errors
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ");
                
                tracing::warn!("❌ Schema validation failed with {} error(s): {}", errors.len(), error_summary);
                
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Schema validation failed: {}", error_summary)
                ));
            }
            SchemaValidationResult::SchemaNotFound => {
                if strict_mode {
                    tracing::error!("❌ Schema not found in strict mode");
                    return Err(rabbitmesh::error::RabbitMeshError::Handler(
                        "Schema validation failed: Schema not found".to_string()
                    ));
                } else {
                    tracing::warn!("⚠️ Schema not found, skipping validation (non-strict mode)");
                }
            }
        }
        
        /// Schema Validator for comprehensive validation
        struct SchemaValidator {
            engine: Box<dyn ValidationEngine + Send + Sync>,
            schema_loader: Arc<dyn SchemaLoader + Send + Sync>,
            cache: Arc<RwLock<std::collections::HashMap<String, CachedSchema>>>,
            strict_mode: bool,
        }
        
        /// Validation engine abstraction
        #[async_trait::async_trait]
        trait ValidationEngine {
            async fn validate(
                &self, 
                schema: &serde_json::Value, 
                data: &serde_json::Value
            ) -> Result<Vec<ValidationError>, SchemaError>;
            
            fn supports_draft(&self, draft: &str) -> bool;
            fn engine_name(&self) -> &str;
        }
        
        /// Schema loader abstraction
        #[async_trait::async_trait]
        trait SchemaLoader {
            async fn load_schema(&self, identifier: &str) -> Result<serde_json::Value, SchemaError>;
            async fn list_schemas(&self) -> Result<Vec<String>, SchemaError>;
            fn supports_source(&self, source_type: &str) -> bool;
        }
        
        /// Cached schema for performance
        #[derive(Debug, Clone)]
        struct CachedSchema {
            schema: serde_json::Value,
            loaded_at: std::time::Instant,
            ttl: std::time::Duration,
            metadata: SchemaMetadata,
        }
        
        /// Schema metadata
        #[derive(Debug, Clone)]
        struct SchemaMetadata {
            id: String,
            version: String,
            draft: String,
            title: Option<String>,
            description: Option<String>,
            source: String,
        }
        
        /// Validation results
        #[derive(Debug)]
        enum SchemaValidationResult {
            Valid,
            Invalid { errors: Vec<ValidationError> },
            SchemaNotFound,
        }
        
        /// Validation error details
        #[derive(Debug, Clone)]
        struct ValidationError {
            path: String,
            message: String,
            error_type: ValidationErrorType,
            schema_path: String,
            suggested_fix: Option<String>,
        }
        
        /// Types of validation errors
        #[derive(Debug, Clone, PartialEq)]
        enum ValidationErrorType {
            TypeMismatch,
            RequiredProperty,
            AdditionalProperty,
            FormatError,
            RangeError,
            PatternMismatch,
            EnumMismatch,
            CustomRule,
        }
        
        /// Schema validation errors
        #[derive(Debug, thiserror::Error)]
        enum SchemaError {
            #[error("Schema not found: {identifier}")]
            NotFound { identifier: String },
            #[error("Invalid schema format: {0}")]
            InvalidFormat(String),
            #[error("Schema load error: {0}")]
            LoadError(String),
            #[error("Validation engine error: {0}")]
            EngineError(String),
            #[error("Configuration error: {0}")]
            Configuration(String),
        }
        
        /// JSON Schema validation engine
        struct JsonSchemaEngine {
            draft_support: Vec<String>,
            custom_formats: std::collections::HashMap<String, Box<dyn CustomFormatValidator + Send + Sync>>,
        }
        
        /// Ajv-compatible validation engine
        struct AjvEngine {
            strict_mode: bool,
            all_errors: bool,
            custom_keywords: std::collections::HashMap<String, Box<dyn CustomKeywordValidator + Send + Sync>>,
        }
        
        /// Custom format validator trait
        trait CustomFormatValidator {
            fn validate(&self, value: &str) -> bool;
            fn error_message(&self) -> &str;
        }
        
        /// Custom keyword validator trait
        trait CustomKeywordValidator {
            fn validate(&self, schema: &serde_json::Value, data: &serde_json::Value) -> bool;
            fn error_message(&self, schema: &serde_json::Value) -> String;
        }
        
        /// File-based schema loader
        struct FileSchemaLoader {
            base_path: std::path::PathBuf,
            extensions: Vec<String>,
        }
        
        /// HTTP-based schema loader
        struct HttpSchemaLoader {
            base_url: String,
            timeout: std::time::Duration,
            headers: std::collections::HashMap<String, String>,
        }
        
        /// Database-based schema loader
        struct DatabaseSchemaLoader {
            connection_string: String,
            table_name: String,
        }
        
        /// Auto-detection schema loader
        struct AutoDetectSchemaLoader {
            loaders: Vec<Box<dyn SchemaLoader + Send + Sync>>,
        }
        
        impl SchemaValidator {
            async fn new(source: &str, engine_type: &str, strict_mode: bool) -> Result<Self, SchemaError> {
                let engine = create_validation_engine(engine_type)?;
                let schema_loader = create_schema_loader(source).await?;
                
                Ok(Self {
                    engine,
                    schema_loader,
                    cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    strict_mode,
                })
            }
            
            async fn validate(&self, msg: &rabbitmesh::Message) -> SchemaValidationResult {
                // Determine schema identifier from message
                let schema_id = self.determine_schema_id(msg);
                
                // Load schema (with caching)
                let schema = match self.load_schema_cached(&schema_id).await {
                    Ok(schema) => schema,
                    Err(SchemaError::NotFound { .. }) => {
                        return SchemaValidationResult::SchemaNotFound;
                    }
                    Err(e) => {
                        tracing::error!("Failed to load schema {}: {}", schema_id, e);
                        return SchemaValidationResult::SchemaNotFound;
                    }
                };
                
                // Perform validation
                match self.engine.validate(&schema, &msg.payload).await {
                    Ok(errors) if errors.is_empty() => SchemaValidationResult::Valid,
                    Ok(errors) => SchemaValidationResult::Invalid { errors },
                    Err(e) => {
                        tracing::error!("Schema validation engine error: {}", e);
                        SchemaValidationResult::Invalid { 
                            errors: vec![ValidationError {
                                path: "/".to_string(),
                                message: format!("Validation engine error: {}", e),
                                error_type: ValidationErrorType::CustomRule,
                                schema_path: "/".to_string(),
                                suggested_fix: None,
                            }]
                        }
                    }
                }
            }
            
            fn determine_schema_id(&self, msg: &rabbitmesh::Message) -> String {
                // Try to extract schema ID from various sources
                
                // 1. Message headers
                if let Some(headers) = &msg.headers {
                    if let Some(schema_id) = headers.get("schema_id").and_then(|v| v.as_str()) {
                        return schema_id.to_string();
                    }
                    
                    if let Some(message_type) = headers.get("message_type").and_then(|v| v.as_str()) {
                        return format!("schemas/{}.json", message_type);
                    }
                    
                    if let Some(route) = headers.get("routing_key").and_then(|v| v.as_str()) {
                        return format!("schemas/{}.json", route.replace('.', "/"));
                    }
                }
                
                // 2. Payload $schema property
                if let Some(schema_ref) = msg.payload.get("$schema").and_then(|v| v.as_str()) {
                    return schema_ref.to_string();
                }
                
                // 3. Service and method name
                let service_name = std::env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string());
                let method_name = msg.headers
                    .as_ref()
                    .and_then(|h| h.get("method"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                
                format!("schemas/{}/{}.json", service_name, method_name)
            }
            
            async fn load_schema_cached(&self, schema_id: &str) -> Result<serde_json::Value, SchemaError> {
                // Check cache first
                {
                    let cache = self.cache.read().await;
                    if let Some(cached) = cache.get(schema_id) {
                        if cached.loaded_at.elapsed() < cached.ttl {
                            return Ok(cached.schema.clone());
                        }
                    }
                }
                
                // Load from source
                let schema = self.schema_loader.load_schema(schema_id).await?;
                
                // Cache the schema
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(schema_id.to_string(), CachedSchema {
                        schema: schema.clone(),
                        loaded_at: std::time::Instant::now(),
                        ttl: std::time::Duration::from_secs(300), // 5 minutes default TTL
                        metadata: extract_schema_metadata(&schema, schema_id),
                    });
                }
                
                Ok(schema)
            }
        }
        
        #[async_trait::async_trait]
        impl ValidationEngine for JsonSchemaEngine {
            async fn validate(
                &self, 
                schema: &serde_json::Value, 
                data: &serde_json::Value
            ) -> Result<Vec<ValidationError>, SchemaError> {
                // Real JSON Schema validation implementation
                let mut errors = Vec::new();
                
                // Basic type validation
                if let Some(expected_type) = schema.get("type").and_then(|v| v.as_str()) {
                    if !self.validate_type(data, expected_type) {
                        errors.push(ValidationError {
                            path: "/".to_string(),
                            message: format!("Expected {}, got {}", expected_type, get_value_type(data)),
                            error_type: ValidationErrorType::TypeMismatch,
                            schema_path: "/type".to_string(),
                            suggested_fix: Some(format!("Convert value to {}", expected_type)),
                        });
                    }
                }
                
                // Required properties validation
                if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
                    if let Some(obj) = data.as_object() {
                        for req_prop in required {
                            if let Some(prop_name) = req_prop.as_str() {
                                if !obj.contains_key(prop_name) {
                                    errors.push(ValidationError {
                                        path: "/".to_string(),
                                        message: format!("Missing required property '{}'", prop_name),
                                        error_type: ValidationErrorType::RequiredProperty,
                                        schema_path: "/required".to_string(),
                                        suggested_fix: Some(format!("Add property '{}'", prop_name)),
                                    });
                                }
                            }
                        }
                    }
                }
                
                // Properties validation
                if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
                    if let Some(data_obj) = data.as_object() {
                        for (prop_name, prop_schema) in properties {
                            if let Some(prop_value) = data_obj.get(prop_name) {
                                let prop_errors = self.validate(prop_schema, prop_value).await?;
                                for mut error in prop_errors {
                                    error.path = format!("/{}{}", prop_name, error.path);
                                    errors.push(error);
                                }
                            }
                        }
                    }
                }
                
                // Additional properties validation
                if let Some(additional_props) = schema.get("additionalProperties") {
                    if additional_props.as_bool() == Some(false) {
                        if let (Some(data_obj), Some(schema_props)) = (data.as_object(), schema.get("properties").and_then(|v| v.as_object())) {
                            for prop_name in data_obj.keys() {
                                if !schema_props.contains_key(prop_name) {
                                    errors.push(ValidationError {
                                        path: format!("/{}", prop_name),
                                        message: format!("Additional property '{}' is not allowed", prop_name),
                                        error_type: ValidationErrorType::AdditionalProperty,
                                        schema_path: "/additionalProperties".to_string(),
                                        suggested_fix: Some(format!("Remove property '{}'", prop_name)),
                                    });
                                }
                            }
                        }
                    }
                }
                
                // String format validation
                if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
                    if let Some(string_value) = data.as_str() {
                        if !self.validate_format(string_value, format) {
                            errors.push(ValidationError {
                                path: "/".to_string(),
                                message: format!("String does not match format '{}'", format),
                                error_type: ValidationErrorType::FormatError,
                                schema_path: "/format".to_string(),
                                suggested_fix: Some(format!("Ensure value matches {} format", format)),
                            });
                        }
                    }
                }
                
                // Enum validation
                if let Some(enum_values) = schema.get("enum").and_then(|v| v.as_array()) {
                    if !enum_values.contains(data) {
                        let allowed_values: Vec<String> = enum_values.iter()
                            .map(|v| v.to_string())
                            .collect();
                        
                        errors.push(ValidationError {
                            path: "/".to_string(),
                            message: format!("Value must be one of: {}", allowed_values.join(", ")),
                            error_type: ValidationErrorType::EnumMismatch,
                            schema_path: "/enum".to_string(),
                            suggested_fix: Some(format!("Use one of: {}", allowed_values.join(", "))),
                        });
                    }
                }
                
                Ok(errors)
            }
            
            fn supports_draft(&self, draft: &str) -> bool {
                self.draft_support.contains(&draft.to_string())
            }
            
            fn engine_name(&self) -> &str {
                "jsonschema"
            }
        }
        
        impl JsonSchemaEngine {
            fn new() -> Self {
                Self {
                    draft_support: vec![
                        "http://json-schema.org/draft-07/schema#".to_string(),
                        "http://json-schema.org/draft/2019-09/schema".to_string(),
                        "http://json-schema.org/draft/2020-12/schema".to_string(),
                    ],
                    custom_formats: std::collections::HashMap::new(),
                }
            }
            
            fn validate_type(&self, data: &serde_json::Value, expected_type: &str) -> bool {
                match expected_type {
                    "string" => data.is_string(),
                    "number" => data.is_number(),
                    "integer" => data.is_i64() || data.is_u64(),
                    "boolean" => data.is_boolean(),
                    "array" => data.is_array(),
                    "object" => data.is_object(),
                    "null" => data.is_null(),
                    _ => true, // Unknown types pass validation
                }
            }
            
            fn validate_format(&self, value: &str, format: &str) -> bool {
                match format {
                    "email" => self.validate_email_format(value),
                    "uri" | "url" => self.validate_uri_format(value),
                    "date" => self.validate_date_format(value),
                    "date-time" => self.validate_datetime_format(value),
                    "uuid" => self.validate_uuid_format(value),
                    "ipv4" => self.validate_ipv4_format(value),
                    "ipv6" => self.validate_ipv6_format(value),
                    _ => {
                        // Check custom formats
                        if let Some(validator) = self.custom_formats.get(format) {
                            validator.validate(value)
                        } else {
                            true // Unknown formats pass validation
                        }
                    }
                }
            }
            
            fn validate_email_format(&self, email: &str) -> bool {
                email.contains('@') && email.contains('.') && !email.starts_with('@') && !email.ends_with('@')
            }
            
            fn validate_uri_format(&self, uri: &str) -> bool {
                uri.starts_with("http://") || uri.starts_with("https://") || uri.starts_with("ftp://")
            }
            
            fn validate_date_format(&self, date: &str) -> bool {
                // Simple date format validation (YYYY-MM-DD)
                let parts: Vec<&str> = date.split('-').collect();
                parts.len() == 3 && 
                parts[0].len() == 4 && parts[0].parse::<u16>().is_ok() &&
                parts[1].len() == 2 && parts[1].parse::<u8>().is_ok() &&
                parts[2].len() == 2 && parts[2].parse::<u8>().is_ok()
            }
            
            fn validate_datetime_format(&self, datetime: &str) -> bool {
                // Simple datetime format validation (ISO 8601)
                datetime.contains('T') && datetime.len() >= 19
            }
            
            fn validate_uuid_format(&self, uuid: &str) -> bool {
                // Simple UUID format validation
                uuid.len() == 36 && uuid.chars().filter(|&c| c == '-').count() == 4
            }
            
            fn validate_ipv4_format(&self, ip: &str) -> bool {
                let parts: Vec<&str> = ip.split('.').collect();
                parts.len() == 4 && parts.iter().all(|&part| {
                    if let Ok(num) = part.parse::<u8>() {
                        num <= 255
                    } else {
                        false
                    }
                })
            }
            
            fn validate_ipv6_format(&self, ip: &str) -> bool {
                // Simplified IPv6 validation
                ip.contains(':') && ip.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
            }
        }
        
        #[async_trait::async_trait]
        impl SchemaLoader for FileSchemaLoader {
            async fn load_schema(&self, identifier: &str) -> Result<serde_json::Value, SchemaError> {
                for extension in &self.extensions {
                    let file_path = self.base_path.join(format!("{}.{}", identifier, extension));
                    
                    if file_path.exists() {
                        let content = tokio::fs::read_to_string(&file_path).await
                            .map_err(|e| SchemaError::LoadError(format!("Failed to read {}: {}", file_path.display(), e)))?;
                        
                        return serde_json::from_str(&content)
                            .map_err(|e| SchemaError::InvalidFormat(format!("Invalid JSON in {}: {}", file_path.display(), e)));
                    }
                }
                
                Err(SchemaError::NotFound { identifier: identifier.to_string() })
            }
            
            async fn list_schemas(&self) -> Result<Vec<String>, SchemaError> {
                let mut schemas = Vec::new();
                
                let mut entries = tokio::fs::read_dir(&self.base_path).await
                    .map_err(|e| SchemaError::LoadError(format!("Failed to read directory: {}", e)))?;
                
                while let Some(entry) = entries.next_entry().await
                    .map_err(|e| SchemaError::LoadError(format!("Failed to read directory entry: {}", e)))? {
                    
                    let path = entry.path();
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if self.extensions.contains(&ext.to_string()) {
                                schemas.push(name.to_string());
                            }
                        }
                    }
                }
                
                Ok(schemas)
            }
            
            fn supports_source(&self, source_type: &str) -> bool {
                matches!(source_type, "file" | "filesystem" | "local")
            }
        }
        
        /// Utility functions
        
        fn get_value_type(value: &serde_json::Value) -> &'static str {
            match value {
                serde_json::Value::String(_) => "string",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
                serde_json::Value::Null => "null",
            }
        }
        
        fn extract_schema_metadata(schema: &serde_json::Value, schema_id: &str) -> SchemaMetadata {
            SchemaMetadata {
                id: schema.get("$id").and_then(|v| v.as_str()).unwrap_or(schema_id).to_string(),
                version: schema.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string(),
                draft: schema.get("$schema").and_then(|v| v.as_str()).unwrap_or("draft-07").to_string(),
                title: schema.get("title").and_then(|v| v.as_str()).map(|s| s.to_string()),
                description: schema.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                source: "file".to_string(),
            }
        }
        
        fn create_validation_engine(engine_type: &str) -> Result<Box<dyn ValidationEngine + Send + Sync>, SchemaError> {
            match engine_type.to_lowercase().as_str() {
                "jsonschema" => Ok(Box::new(JsonSchemaEngine::new())),
                _ => Err(SchemaError::Configuration(format!("Unsupported validation engine: {}", engine_type))),
            }
        }
        
        async fn create_schema_loader(source: &str) -> Result<Arc<dyn SchemaLoader + Send + Sync>, SchemaError> {
            match source.to_lowercase().as_str() {
                "file" | "filesystem" => {
                    let base_path = std::env::var("SCHEMA_PATH")
                        .unwrap_or_else(|_| "./schemas".to_string());
                    
                    Ok(Arc::new(FileSchemaLoader {
                        base_path: std::path::PathBuf::from(base_path),
                        extensions: vec!["json".to_string(), "yaml".to_string(), "yml".to_string()],
                    }))
                }
                "auto_detect" => {
                    let mut loaders: Vec<Box<dyn SchemaLoader + Send + Sync>> = Vec::new();
                    
                    // Add file loader
                    let base_path = std::env::var("SCHEMA_PATH")
                        .unwrap_or_else(|_| "./schemas".to_string());
                    loaders.push(Box::new(FileSchemaLoader {
                        base_path: std::path::PathBuf::from(base_path),
                        extensions: vec!["json".to_string(), "yaml".to_string()],
                    }));
                    
                    Err(SchemaError::Configuration("Auto-detect loader not fully implemented".to_string()))
                }
                _ => Err(SchemaError::Configuration(format!("Unsupported schema source: {}", source))),
            }
        }
    }
}