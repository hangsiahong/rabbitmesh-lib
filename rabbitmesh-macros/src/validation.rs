//! Universal Validation and Input Processing Macros for RabbitMesh
//! 
//! This module provides comprehensive validation and input processing macros that work
//! across ALL project types and data formats:
//!
//! **Project Types Supported:**
//! - 🛒 E-commerce (product data, pricing, inventory, user profiles)
//! - 💰 Finance (trading rules, account validation, transaction limits)
//! - 🏥 Healthcare (patient data, medical records, HIPAA compliance)
//! - 🏭 IoT (sensor data, device configuration, telemetry validation)
//! - 🎮 Gaming (player input, game rules, anti-cheat validation)
//! - 📱 Social Media (content moderation, user input, privacy rules)
//! - 🏢 Enterprise (business rules, compliance, data governance)
//! - 🎓 Education (student data, grading rules, content validation)
//! - 📦 Logistics (address validation, shipping rules, route optimization)
//! - 🌍 GIS/Mapping (coordinate validation, spatial constraints)
//!
//! **Validation Types:**
//! - Data Type Validation (strings, numbers, dates, emails, URLs)
//! - Business Rule Validation (custom logic, domain rules)
//! - Security Validation (SQL injection, XSS, input sanitization)
//! - Format Validation (JSON, XML, CSV, binary formats)
//! - Compliance Validation (GDPR, HIPAA, PCI-DSS, SOX)

use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Attribute, LitStr, LitInt, LitBool};
use std::collections::HashMap;
use crate::universal::{MacroAttribute, MacroValue, MacroContext};

/// Universal validation and input processing processor
pub struct ValidationProcessor;

impl ValidationProcessor {
    /// Generate validation logic for any macro attribute
    pub fn generate_validation_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        match attr.name.as_str() {
            "validate" => Self::generate_validate_logic(attr, context),
            "sanitize" => Self::generate_sanitize_logic(attr, context),
            "transform" => Self::generate_transform_logic(attr, context),
            "constrain" => Self::generate_constrain_logic(attr, context),
            "custom_validate" => Self::generate_custom_validation(attr, context),
            "validate_email" => Self::generate_email_validation(attr, context),
            "validate_phone" => Self::generate_phone_validation(attr, context),
            "validate_range" => Self::generate_range_validation(attr, context),
            "validate_format" => Self::generate_format_validation(attr, context),
            "validate_business_rules" => Self::generate_business_rules_validation(attr, context),
            "validate_security" => Self::generate_security_validation(attr, context),
            "validate_compliance" => Self::generate_compliance_validation(attr, context),
            _ => quote! {}
        }
    }

    /// Generate main validation logic with universal support
    fn generate_validate_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let validation_rules = Self::extract_validation_rules(attr);
        let domain_validation = Self::generate_domain_validation(context, attr);
        let security_validation = Self::generate_security_validation_base();
        let compliance_check = Self::generate_compliance_check(context);
        
        quote! {
            // Universal input validation
            debug!("✅ Starting comprehensive validation");
            
            // Parse request payload for validation
            let validation_data = rabbitmesh_validation::parse_request_data(&msg).await?;
            
            #security_validation
            #compliance_check
            #domain_validation
            
            // Apply custom validation rules
            let validation_rules = vec![#(#validation_rules),*];
            for rule in validation_rules {
                if let Err(validation_error) = rabbitmesh_validation::apply_rule(&validation_data, &rule).await {
                    warn!("❌ Validation failed: {}", validation_error);
                    return Ok(rabbitmesh::RpcResponse::error(&format!("Validation failed: {}", validation_error)));
                }
            }
            
            debug!("✅ All validation checks passed");
        }
    }

    /// Generate sanitization logic
    fn generate_sanitize_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let sanitize_rules = Self::extract_sanitize_rules(attr);
        let domain_sanitization = Self::generate_domain_sanitization(context);
        
        quote! {
            // Universal input sanitization
            debug!("🧹 Starting input sanitization");
            
            let mut sanitized_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            #domain_sanitization
            
            // Apply sanitization rules
            let sanitize_rules = vec![#(#sanitize_rules),*];
            for rule in sanitize_rules {
                sanitized_data = rabbitmesh_validation::apply_sanitization(&sanitized_data, &rule).await?;
            }
            
            // Update message with sanitized data
            rabbitmesh_validation::update_message_data(&mut msg, sanitized_data).await?;
            
            debug!("🧹 Input sanitization completed");
        }
    }

    /// Generate transformation logic
    fn generate_transform_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let transform_rules = Self::extract_transform_rules(attr);
        let domain_transforms = Self::generate_domain_transforms(context);
        
        quote! {
            // Universal data transformation
            debug!("🔄 Starting data transformation");
            
            let mut transform_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            #domain_transforms
            
            // Apply transformation rules
            let transform_rules = vec![#(#transform_rules),*];
            for rule in transform_rules {
                transform_data = rabbitmesh_validation::apply_transformation(&transform_data, &rule).await?;
            }
            
            // Update message with transformed data
            rabbitmesh_validation::update_message_data(&mut msg, transform_data).await?;
            
            debug!("🔄 Data transformation completed");
        }
    }

    /// Generate constraint validation
    fn generate_constrain_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let constraints = Self::extract_constraints(attr);
        let domain_constraints = Self::generate_domain_constraints(context);
        
        quote! {
            // Universal constraint validation
            debug!("🔒 Validating constraints");
            
            let constraint_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            #domain_constraints
            
            // Apply constraints
            let constraints = vec![#(#constraints),*];
            for constraint in constraints {
                if let Err(constraint_error) = rabbitmesh_validation::validate_constraint(&constraint_data, &constraint).await {
                    warn!("🔒 Constraint violation: {}", constraint_error);
                    return Ok(rabbitmesh::RpcResponse::error(&format!("Constraint violation: {}", constraint_error)));
                }
            }
            
            debug!("🔒 All constraints satisfied");
        }
    }

    /// Generate custom validation logic
    fn generate_custom_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let validator_function = Self::extract_validator_function(attr);
        
        quote! {
            // Custom validation function
            debug!("🎯 Running custom validation: {}", #validator_function);
            
            let validation_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            // Call custom validator
            if let Err(custom_error) = #validator_function(&validation_data).await {
                warn!("🎯 Custom validation failed: {}", custom_error);
                return Ok(rabbitmesh::RpcResponse::error(&format!("Custom validation failed: {}", custom_error)));
            }
            
            debug!("🎯 Custom validation passed");
        }
    }

    /// Generate email validation
    fn generate_email_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let field_name = Self::extract_field_name(attr, "email");
        let allow_international = Self::extract_bool_arg(attr, "international").unwrap_or(true);
        let require_mx_check = Self::extract_bool_arg(attr, "mx_check").unwrap_or(false);
        
        quote! {
            // Email validation
            debug!("📧 Validating email field: {}", #field_name);
            
            let email_data = rabbitmesh_validation::get_field_data(&msg, #field_name).await?;
            
            // Basic email format validation
            if !rabbitmesh_validation::is_valid_email_format(&email_data) {
                return Ok(rabbitmesh::RpcResponse::error("Invalid email format"));
            }
            
            // International email support
            if #allow_international {
                if !rabbitmesh_validation::is_valid_international_email(&email_data) {
                    return Ok(rabbitmesh::RpcResponse::error("Invalid international email format"));
                }
            }
            
            // MX record check
            if #require_mx_check {
                if !rabbitmesh_validation::check_email_mx_record(&email_data).await? {
                    return Ok(rabbitmesh::RpcResponse::error("Email domain does not accept mail"));
                }
            }
            
            debug!("📧 Email validation passed");
        }
    }

    /// Generate phone validation
    fn generate_phone_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let field_name = Self::extract_field_name(attr, "phone");
        let country_code = Self::extract_string_arg(attr, "country").unwrap_or_else(|| "US".to_string());
        let allow_international = Self::extract_bool_arg(attr, "international").unwrap_or(true);
        
        quote! {
            // Phone number validation
            debug!("📞 Validating phone field: {} (country: {})", #field_name, #country_code);
            
            let phone_data = rabbitmesh_validation::get_field_data(&msg, #field_name).await?;
            
            // Format phone number
            let formatted_phone = rabbitmesh_validation::format_phone_number(&phone_data, #country_code)?;
            
            // Validate phone format
            if !rabbitmesh_validation::is_valid_phone_format(&formatted_phone, #country_code) {
                return Ok(rabbitmesh::RpcResponse::error("Invalid phone number format"));
            }
            
            // International phone support
            if #allow_international {
                if !rabbitmesh_validation::is_valid_international_phone(&formatted_phone) {
                    return Ok(rabbitmesh::RpcResponse::error("Invalid international phone format"));
                }
            }
            
            // Update message with formatted phone
            rabbitmesh_validation::update_field_data(&mut msg, #field_name, formatted_phone).await?;
            
            debug!("📞 Phone validation passed");
        }
    }

    /// Generate range validation
    fn generate_range_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let field_name = Self::extract_field_name(attr, "field");
        let min_value = Self::extract_int_arg(attr, "min");
        let max_value = Self::extract_int_arg(attr, "max");
        
        quote! {
            // Range validation
            debug!("📏 Validating range for field: {}", #field_name);
            
            let field_data = rabbitmesh_validation::get_field_data(&msg, #field_name).await?;
            let numeric_value = rabbitmesh_validation::parse_numeric(&field_data)?;
            
            // Check minimum value
            if let Some(min) = #min_value {
                if numeric_value < min as f64 {
                    return Ok(rabbitmesh::RpcResponse::error(&format!("Value {} is below minimum {}", numeric_value, min)));
                }
            }
            
            // Check maximum value
            if let Some(max) = #max_value {
                if numeric_value > max as f64 {
                    return Ok(rabbitmesh::RpcResponse::error(&format!("Value {} exceeds maximum {}", numeric_value, max)));
                }
            }
            
            debug!("📏 Range validation passed");
        }
    }

    /// Generate format validation
    fn generate_format_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let format_type = Self::extract_string_arg(attr, "format").unwrap_or_else(|| "json".to_string());
        let strict_mode = Self::extract_bool_arg(attr, "strict").unwrap_or(false);
        
        quote! {
            // Format validation
            debug!("📋 Validating format: {} (strict: {})", #format_type, #strict_mode);
            
            let format_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            match #format_type.as_str() {
                "json" => {
                    if !rabbitmesh_validation::is_valid_json(&format_data, #strict_mode)? {
                        return Ok(rabbitmesh::RpcResponse::error("Invalid JSON format"));
                    }
                },
                "xml" => {
                    if !rabbitmesh_validation::is_valid_xml(&format_data, #strict_mode)? {
                        return Ok(rabbitmesh::RpcResponse::error("Invalid XML format"));
                    }
                },
                "csv" => {
                    if !rabbitmesh_validation::is_valid_csv(&format_data, #strict_mode)? {
                        return Ok(rabbitmesh::RpcResponse::error("Invalid CSV format"));
                    }
                },
                "yaml" => {
                    if !rabbitmesh_validation::is_valid_yaml(&format_data, #strict_mode)? {
                        return Ok(rabbitmesh::RpcResponse::error("Invalid YAML format"));
                    }
                },
                _ => {
                    warn!("🤷 Unknown format type: {}", #format_type);
                }
            }
            
            debug!("📋 Format validation passed");
        }
    }

    /// Generate business rules validation
    fn generate_business_rules_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let rules = Self::extract_business_rules(attr);
        
        let domain_specific_rules = match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce business rules
                let business_validator = rabbitmesh_validation::EcommerceBusinessRules::new();
                business_validator.validate_product_data(&validation_data).await?;
                business_validator.validate_pricing_rules(&validation_data).await?;
                business_validator.validate_inventory_constraints(&validation_data).await?;
            },
            "finance" => quote! {
                // Finance business rules
                let business_validator = rabbitmesh_validation::FinanceBusinessRules::new();
                business_validator.validate_trading_limits(&validation_data).await?;
                business_validator.validate_regulatory_compliance(&validation_data).await?;
                business_validator.validate_risk_parameters(&validation_data).await?;
            },
            "healthcare" => quote! {
                // Healthcare business rules
                let business_validator = rabbitmesh_validation::HealthcareBusinessRules::new();
                business_validator.validate_patient_privacy(&validation_data).await?;
                business_validator.validate_medical_data_integrity(&validation_data).await?;
                business_validator.validate_hipaa_compliance(&validation_data).await?;
            },
            "gaming" => quote! {
                // Gaming business rules
                let business_validator = rabbitmesh_validation::GamingBusinessRules::new();
                business_validator.validate_game_rules(&validation_data).await?;
                business_validator.validate_anti_cheat(&validation_data).await?;
                business_validator.validate_player_fairness(&validation_data).await?;
            },
            _ => quote! {
                // Generic business rules
                let business_validator = rabbitmesh_validation::GenericBusinessRules::new();
                business_validator.validate_common_rules(&validation_data).await?;
            }
        };
        
        quote! {
            // Business rules validation
            debug!("💼 Validating business rules for domain: {}", #domain);
            
            let validation_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            #domain_specific_rules
            
            // Apply custom business rules
            let business_rules = vec![#(#rules),*];
            for rule in business_rules {
                if let Err(rule_error) = rabbitmesh_validation::validate_business_rule(&validation_data, &rule).await {
                    return Ok(rabbitmesh::RpcResponse::error(&format!("Business rule violation: {}", rule_error)));
                }
            }
            
            debug!("💼 Business rules validation passed");
        }
    }

    /// Generate security validation  
    fn generate_security_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let check_xss = Self::extract_bool_arg(attr, "xss").unwrap_or(true);
        let check_sql_injection = Self::extract_bool_arg(attr, "sql_injection").unwrap_or(true);
        let check_path_traversal = Self::extract_bool_arg(attr, "path_traversal").unwrap_or(true);
        let check_malicious_files = Self::extract_bool_arg(attr, "malicious_files").unwrap_or(true);
        
        quote! {
            // Security validation
            debug!("🛡️ Starting security validation");
            
            let security_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            // XSS protection
            if #check_xss {
                if rabbitmesh_validation::contains_xss_patterns(&security_data)? {
                    warn!("🚨 XSS attack attempt detected");
                    return Ok(rabbitmesh::RpcResponse::error("Security violation: XSS detected"));
                }
            }
            
            // SQL injection protection
            if #check_sql_injection {
                if rabbitmesh_validation::contains_sql_injection_patterns(&security_data)? {
                    warn!("🚨 SQL injection attempt detected");
                    return Ok(rabbitmesh::RpcResponse::error("Security violation: SQL injection detected"));
                }
            }
            
            // Path traversal protection
            if #check_path_traversal {
                if rabbitmesh_validation::contains_path_traversal_patterns(&security_data)? {
                    warn!("🚨 Path traversal attempt detected");
                    return Ok(rabbitmesh::RpcResponse::error("Security violation: Path traversal detected"));
                }
            }
            
            // Malicious file upload protection
            if #check_malicious_files {
                if rabbitmesh_validation::contains_malicious_file_patterns(&security_data)? {
                    warn!("🚨 Malicious file upload attempt detected");
                    return Ok(rabbitmesh::RpcResponse::error("Security violation: Malicious file detected"));
                }
            }
            
            debug!("🛡️ Security validation passed");
        }
    }

    /// Generate compliance validation
    fn generate_compliance_validation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let compliance_standards = Self::extract_compliance_standards(attr);
        let domain = Self::infer_domain_from_context(context);
        
        let domain_compliance = match domain.as_str() {
            "healthcare" => quote! {
                // HIPAA compliance for healthcare
                rabbitmesh_validation::validate_hipaa_compliance(&compliance_data).await?;
            },
            "finance" => quote! {
                // PCI-DSS and SOX compliance for finance
                rabbitmesh_validation::validate_pci_dss_compliance(&compliance_data).await?;
                rabbitmesh_validation::validate_sox_compliance(&compliance_data).await?;
            },
            _ => quote! {
                // Generic compliance checks
                rabbitmesh_validation::validate_gdpr_compliance(&compliance_data).await?;
            }
        };
        
        quote! {
            // Compliance validation
            debug!("📋 Validating compliance standards: {:?}", vec![#(#compliance_standards),*]);
            
            let compliance_data = rabbitmesh_validation::get_request_data(&msg).await?;
            
            #domain_compliance
            
            // Apply specific compliance standards
            let standards = vec![#(#compliance_standards),*];
            for standard in standards {
                match standard.as_str() {
                    "gdpr" => rabbitmesh_validation::validate_gdpr_compliance(&compliance_data).await?,
                    "hipaa" => rabbitmesh_validation::validate_hipaa_compliance(&compliance_data).await?,
                    "pci_dss" => rabbitmesh_validation::validate_pci_dss_compliance(&compliance_data).await?,
                    "sox" => rabbitmesh_validation::validate_sox_compliance(&compliance_data).await?,
                    "ccpa" => rabbitmesh_validation::validate_ccpa_compliance(&compliance_data).await?,
                    _ => warn!("⚠️ Unknown compliance standard: {}", standard)
                }
            }
            
            debug!("📋 Compliance validation passed");
        }
    }

    // Domain-specific validation generators

    /// Generate domain-specific validation
    fn generate_domain_validation(context: &MacroContext, attr: &MacroAttribute) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => Self::generate_ecommerce_validation(method_name, attr),
            "finance" => Self::generate_finance_validation(method_name, attr),
            "healthcare" => Self::generate_healthcare_validation(method_name, attr),
            "iot" => Self::generate_iot_validation(method_name, attr),
            "gaming" => Self::generate_gaming_validation(method_name, attr),
            "social" => Self::generate_social_validation(method_name, attr),
            "logistics" => Self::generate_logistics_validation(method_name, attr),
            _ => quote! {
                debug!("🔧 Using generic validation for domain: {}", #domain);
            }
        }
    }

    /// Generate e-commerce specific validation
    fn generate_ecommerce_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // E-commerce domain validation
            if #method_name.contains("product") {
                // Product validation
                rabbitmesh_validation::validate_product_data(&validation_data).await?;
                rabbitmesh_validation::validate_sku_format(&validation_data).await?;
                rabbitmesh_validation::validate_price_format(&validation_data).await?;
            } else if #method_name.contains("order") {
                // Order validation
                rabbitmesh_validation::validate_order_data(&validation_data).await?;
                rabbitmesh_validation::validate_payment_info(&validation_data).await?;
                rabbitmesh_validation::validate_shipping_address(&validation_data).await?;
            } else if #method_name.contains("cart") {
                // Cart validation
                rabbitmesh_validation::validate_cart_items(&validation_data).await?;
                rabbitmesh_validation::validate_quantity_limits(&validation_data).await?;
            }
        }
    }

    /// Generate finance specific validation
    fn generate_finance_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // Finance domain validation
            if #method_name.contains("trade") || #method_name.contains("order") {
                // Trading validation
                rabbitmesh_validation::validate_trading_limits(&validation_data).await?;
                rabbitmesh_validation::validate_market_hours(&validation_data).await?;
                rabbitmesh_validation::validate_account_balance(&validation_data).await?;
            } else if #method_name.contains("transfer") || #method_name.contains("payment") {
                // Payment validation
                rabbitmesh_validation::validate_transfer_limits(&validation_data).await?;
                rabbitmesh_validation::validate_account_verification(&validation_data).await?;
                rabbitmesh_validation::validate_kyc_compliance(&validation_data).await?;
            } else if #method_name.contains("portfolio") {
                // Portfolio validation
                rabbitmesh_validation::validate_portfolio_limits(&validation_data).await?;
                rabbitmesh_validation::validate_risk_tolerance(&validation_data).await?;
            }
        }
    }

    /// Generate healthcare specific validation
    fn generate_healthcare_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // Healthcare domain validation
            if #method_name.contains("patient") {
                // Patient data validation
                rabbitmesh_validation::validate_patient_privacy(&validation_data).await?;
                rabbitmesh_validation::validate_phi_compliance(&validation_data).await?;
                rabbitmesh_validation::validate_consent_requirements(&validation_data).await?;
            } else if #method_name.contains("medical") || #method_name.contains("diagnosis") {
                // Medical data validation
                rabbitmesh_validation::validate_medical_codes(&validation_data).await?;
                rabbitmesh_validation::validate_prescription_format(&validation_data).await?;
            } else if #method_name.contains("appointment") {
                // Appointment validation
                rabbitmesh_validation::validate_scheduling_rules(&validation_data).await?;
                rabbitmesh_validation::validate_provider_availability(&validation_data).await?;
            }
        }
    }

    /// Generate IoT specific validation
    fn generate_iot_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // IoT domain validation
            if #method_name.contains("sensor") || #method_name.contains("telemetry") {
                // Sensor data validation
                rabbitmesh_validation::validate_sensor_readings(&validation_data).await?;
                rabbitmesh_validation::validate_timestamp_sequence(&validation_data).await?;
                rabbitmesh_validation::validate_data_integrity(&validation_data).await?;
            } else if #method_name.contains("device") {
                // Device validation
                rabbitmesh_validation::validate_device_authorization(&validation_data).await?;
                rabbitmesh_validation::validate_firmware_version(&validation_data).await?;
            } else if #method_name.contains("config") {
                // Configuration validation
                rabbitmesh_validation::validate_config_parameters(&validation_data).await?;
                rabbitmesh_validation::validate_security_settings(&validation_data).await?;
            }
        }
    }

    /// Generate gaming specific validation
    fn generate_gaming_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // Gaming domain validation
            if #method_name.contains("player") || #method_name.contains("profile") {
                // Player validation
                rabbitmesh_validation::validate_player_data(&validation_data).await?;
                rabbitmesh_validation::validate_username_format(&validation_data).await?;
                rabbitmesh_validation::validate_age_requirements(&validation_data).await?;
            } else if #method_name.contains("match") || #method_name.contains("game") {
                // Game validation
                rabbitmesh_validation::validate_game_rules(&validation_data).await?;
                rabbitmesh_validation::validate_anti_cheat(&validation_data).await?;
                rabbitmesh_validation::validate_match_conditions(&validation_data).await?;
            } else if #method_name.contains("score") || #method_name.contains("achievement") {
                // Score validation
                rabbitmesh_validation::validate_score_integrity(&validation_data).await?;
                rabbitmesh_validation::validate_achievement_criteria(&validation_data).await?;
            }
        }
    }

    /// Generate social media specific validation
    fn generate_social_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // Social media domain validation
            if #method_name.contains("post") || #method_name.contains("content") {
                // Content validation
                rabbitmesh_validation::validate_content_moderation(&validation_data).await?;
                rabbitmesh_validation::validate_inappropriate_content(&validation_data).await?;
                rabbitmesh_validation::validate_content_length(&validation_data).await?;
            } else if #method_name.contains("user") || #method_name.contains("profile") {
                // User validation
                rabbitmesh_validation::validate_profile_completeness(&validation_data).await?;
                rabbitmesh_validation::validate_privacy_settings(&validation_data).await?;
            } else if #method_name.contains("message") || #method_name.contains("chat") {
                // Message validation
                rabbitmesh_validation::validate_message_format(&validation_data).await?;
                rabbitmesh_validation::validate_spam_detection(&validation_data).await?;
            }
        }
    }

    /// Generate logistics specific validation
    fn generate_logistics_validation(method_name: &str, attr: &MacroAttribute) -> TokenStream {
        quote! {
            // Logistics domain validation
            if #method_name.contains("shipment") || #method_name.contains("delivery") {
                // Shipment validation
                rabbitmesh_validation::validate_shipping_address(&validation_data).await?;
                rabbitmesh_validation::validate_package_dimensions(&validation_data).await?;
                rabbitmesh_validation::validate_shipping_rules(&validation_data).await?;
            } else if #method_name.contains("route") || #method_name.contains("tracking") {
                // Route validation
                rabbitmesh_validation::validate_route_optimization(&validation_data).await?;
                rabbitmesh_validation::validate_gps_coordinates(&validation_data).await?;
            } else if #method_name.contains("inventory") {
                // Inventory validation
                rabbitmesh_validation::validate_inventory_levels(&validation_data).await?;
                rabbitmesh_validation::validate_warehouse_capacity(&validation_data).await?;
            }
        }
    }

    // Domain-specific sanitization and transformation generators

    /// Generate domain-specific sanitization
    fn generate_domain_sanitization(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "healthcare" => quote! {
                // Healthcare data sanitization (PHI protection)
                sanitized_data = rabbitmesh_validation::sanitize_phi_data(sanitized_data).await?;
                sanitized_data = rabbitmesh_validation::mask_sensitive_health_data(sanitized_data).await?;
            },
            "finance" => quote! {
                // Finance data sanitization (PII/PCI protection)
                sanitized_data = rabbitmesh_validation::sanitize_financial_data(sanitized_data).await?;
                sanitized_data = rabbitmesh_validation::mask_credit_card_data(sanitized_data).await?;
            },
            "social" => quote! {
                // Social media sanitization (content filtering)
                sanitized_data = rabbitmesh_validation::sanitize_user_content(sanitized_data).await?;
                sanitized_data = rabbitmesh_validation::filter_inappropriate_language(sanitized_data).await?;
            },
            _ => quote! {
                // Generic sanitization
                sanitized_data = rabbitmesh_validation::sanitize_common_patterns(sanitized_data).await?;
            }
        }
    }

    /// Generate domain-specific transformations
    fn generate_domain_transforms(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce transformations
                transform_data = rabbitmesh_validation::normalize_product_data(transform_data).await?;
                transform_data = rabbitmesh_validation::format_pricing_data(transform_data).await?;
            },
            "finance" => quote! {
                // Finance transformations
                transform_data = rabbitmesh_validation::normalize_financial_amounts(transform_data).await?;
                transform_data = rabbitmesh_validation::format_account_numbers(transform_data).await?;
            },
            "iot" => quote! {
                // IoT transformations
                transform_data = rabbitmesh_validation::normalize_sensor_readings(transform_data).await?;
                transform_data = rabbitmesh_validation::convert_time_zones(transform_data).await?;
            },
            _ => quote! {
                // Generic transformations
                transform_data = rabbitmesh_validation::normalize_common_formats(transform_data).await?;
            }
        }
    }

    /// Generate domain-specific constraints
    fn generate_domain_constraints(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce constraints
                rabbitmesh_validation::validate_inventory_constraints(&constraint_data).await?;
                rabbitmesh_validation::validate_pricing_constraints(&constraint_data).await?;
            },
            "finance" => quote! {
                // Finance constraints
                rabbitmesh_validation::validate_trading_hours_constraints(&constraint_data).await?;
                rabbitmesh_validation::validate_regulatory_constraints(&constraint_data).await?;
            },
            "gaming" => quote! {
                // Gaming constraints
                rabbitmesh_validation::validate_game_balance_constraints(&constraint_data).await?;
                rabbitmesh_validation::validate_fair_play_constraints(&constraint_data).await?;
            },
            _ => quote! {
                // Generic constraints
                rabbitmesh_validation::validate_common_constraints(&constraint_data).await?;
            }
        }
    }

    // Base security validation
    fn generate_security_validation_base() -> TokenStream {
        quote! {
            // Base security checks for all domains
            debug!("🔒 Running base security validation");
            
            // Check for common attack patterns
            if rabbitmesh_validation::contains_malicious_patterns(&validation_data)? {
                warn!("🚨 Malicious pattern detected in input");
                return Ok(rabbitmesh::RpcResponse::error("Security violation: Malicious input detected"));
            }
            
            // Input length validation
            if rabbitmesh_validation::exceeds_size_limits(&validation_data)? {
                warn!("📏 Input size exceeds allowed limits");
                return Ok(rabbitmesh::RpcResponse::error("Input size exceeds limits"));
            }
            
            debug!("🔒 Base security validation passed");
        }
    }

    /// Generate compliance check
    fn generate_compliance_check(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "healthcare" => quote! {
                // HIPAA compliance check
                debug!("🏥 Checking HIPAA compliance");
                rabbitmesh_validation::validate_hipaa_requirements(&validation_data).await?;
            },
            "finance" => quote! {
                // Financial compliance check
                debug!("💰 Checking financial compliance (PCI-DSS, SOX)");
                rabbitmesh_validation::validate_financial_compliance(&validation_data).await?;
            },
            _ => quote! {
                // GDPR compliance (applies to most domains)
                debug!("🌍 Checking GDPR compliance");
                rabbitmesh_validation::validate_gdpr_requirements(&validation_data).await?;
            }
        }
    }

    // Helper methods for extracting configuration

    /// Extract validation rules from attribute
    fn extract_validation_rules(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut rules = Vec::new();
        
        for (key, value) in &attr.args {
            match value {
                MacroValue::String(rule_str) => {
                    rules.push(quote! { 
                        rabbitmesh_validation::ValidationRule::new(#key, #rule_str) 
                    });
                }
                MacroValue::Boolean(rule_bool) => {
                    rules.push(quote! { 
                        rabbitmesh_validation::ValidationRule::boolean(#key, #rule_bool) 
                    });
                }
                MacroValue::Integer(rule_int) => {
                    rules.push(quote! { 
                        rabbitmesh_validation::ValidationRule::integer(#key, #rule_int) 
                    });
                }
                _ => {}
            }
        }
        
        rules
    }

    /// Extract sanitization rules
    fn extract_sanitize_rules(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut rules = Vec::new();
        
        // Add common sanitization rules
        rules.push(quote! { 
            rabbitmesh_validation::SanitizeRule::html_encode() 
        });
        rules.push(quote! { 
            rabbitmesh_validation::SanitizeRule::trim_whitespace() 
        });
        
        // Add custom rules from attribute
        for (key, value) in &attr.args {
            if let MacroValue::String(rule_str) = value {
                rules.push(quote! { 
                    rabbitmesh_validation::SanitizeRule::custom(#key, #rule_str) 
                });
            }
        }
        
        rules
    }

    /// Extract transformation rules
    fn extract_transform_rules(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut rules = Vec::new();
        
        for (key, value) in &attr.args {
            if let MacroValue::String(transform_str) = value {
                rules.push(quote! { 
                    rabbitmesh_validation::TransformRule::new(#key, #transform_str) 
                });
            }
        }
        
        rules
    }

    /// Extract constraints
    fn extract_constraints(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut constraints = Vec::new();
        
        for (key, value) in &attr.args {
            match value {
                MacroValue::String(constraint_str) => {
                    constraints.push(quote! { 
                        rabbitmesh_validation::Constraint::expression(#key, #constraint_str) 
                    });
                }
                MacroValue::Integer(constraint_int) => {
                    constraints.push(quote! { 
                        rabbitmesh_validation::Constraint::numeric(#key, #constraint_int) 
                    });
                }
                _ => {}
            }
        }
        
        constraints
    }

    /// Extract business rules
    fn extract_business_rules(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut rules = Vec::new();
        
        for (key, value) in &attr.args {
            if let MacroValue::String(rule_str) = value {
                rules.push(quote! { 
                    rabbitmesh_validation::BusinessRule::new(#key, #rule_str) 
                });
            }
        }
        
        rules
    }

    /// Extract compliance standards
    fn extract_compliance_standards(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut standards = Vec::new();
        
        if let Some(MacroValue::Array(standard_array)) = attr.args.get("standards") {
            for standard in standard_array {
                if let MacroValue::String(standard_str) = standard {
                    standards.push(quote! { #standard_str.to_string() });
                }
            }
        }
        
        standards
    }

    /// Extract validator function name
    fn extract_validator_function(attr: &MacroAttribute) -> TokenStream {
        if let Some(MacroValue::String(func_name)) = attr.args.get("function") {
            let func_ident = format_ident!("{}", func_name);
            quote! { #func_ident }
        } else {
            quote! { default_validator }
        }
    }

    /// Extract field name
    fn extract_field_name(attr: &MacroAttribute, default: &str) -> TokenStream {
        if let Some(MacroValue::String(field)) = attr.args.get("field") {
            quote! { #field }
        } else {
            quote! { #default }
        }
    }

    /// Extract string argument
    fn extract_string_arg(attr: &MacroAttribute, key: &str) -> Option<String> {
        if let Some(MacroValue::String(value)) = attr.args.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    /// Extract integer argument
    fn extract_int_arg(attr: &MacroAttribute, key: &str) -> Option<i64> {
        if let Some(MacroValue::Integer(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Extract boolean argument
    fn extract_bool_arg(attr: &MacroAttribute, key: &str) -> Option<bool> {
        if let Some(MacroValue::Boolean(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Infer domain from context
    fn infer_domain_from_context(context: &MacroContext) -> String {
        let service_name = context.service_name.to_lowercase();
        let method_name = context.method_name.to_lowercase();
        
        // Check service name patterns
        if service_name.contains("ecommerce") || service_name.contains("shop") || service_name.contains("product") || service_name.contains("cart") {
            return "ecommerce".to_string();
        }
        if service_name.contains("finance") || service_name.contains("trading") || service_name.contains("bank") || service_name.contains("payment") {
            return "finance".to_string();
        }
        if service_name.contains("health") || service_name.contains("medical") || service_name.contains("patient") {
            return "healthcare".to_string();
        }
        if service_name.contains("iot") || service_name.contains("sensor") || service_name.contains("device") {
            return "iot".to_string();
        }
        if service_name.contains("game") || service_name.contains("gaming") || service_name.contains("player") {
            return "gaming".to_string();
        }
        if service_name.contains("social") || service_name.contains("media") || service_name.contains("feed") {
            return "social".to_string();
        }
        if service_name.contains("logistics") || service_name.contains("shipping") || service_name.contains("delivery") {
            return "logistics".to_string();
        }
        
        // Check method name patterns
        if method_name.contains("product") || method_name.contains("cart") || method_name.contains("order") {
            return "ecommerce".to_string();
        }
        if method_name.contains("trade") || method_name.contains("portfolio") || method_name.contains("account") {
            return "finance".to_string();
        }
        if method_name.contains("patient") || method_name.contains("medical") || method_name.contains("health") {
            return "healthcare".to_string();
        }
        if method_name.contains("sensor") || method_name.contains("device") || method_name.contains("telemetry") {
            return "iot".to_string();
        }
        if method_name.contains("player") || method_name.contains("game") || method_name.contains("score") {
            return "gaming".to_string();
        }
        if method_name.contains("post") || method_name.contains("feed") || method_name.contains("social") {
            return "social".to_string();
        }
        if method_name.contains("ship") || method_name.contains("deliver") || method_name.contains("route") {
            return "logistics".to_string();
        }
        
        "generic".to_string()
    }
}