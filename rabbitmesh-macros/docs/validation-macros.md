# Validation Macros

Comprehensive input validation, schema validation, and data integrity macros for robust data processing and API security.

## Overview

Validation macros provide multiple layers of data validation, from simple input checking to complex schema validation and business rule enforcement. They ensure data integrity and API security with automatic error handling.

---

## Basic Validation Macros

### `#[validate]`

**Purpose**: Performs comprehensive input validation on service method parameters.

**Usage**:
```rust
#[service_method("POST /users")]
#[validate]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    // Input is automatically validated before this method executes
    let user_data: CreateUserRequest = msg.deserialize_payload()?;
    
    let user = create_new_user(user_data).await?;
    Ok(RpcResponse::success(&user, 0)?)
}
```

**Validation Rules Applied**:
- **JSON Format**: Validates payload is valid JSON
- **Schema Compliance**: Validates against struct field constraints
- **Required Fields**: Ensures all required fields are present
- **Type Safety**: Validates field types match expectations
- **Basic Constraints**: Length, range, format validations

**Generated Code**:
```rust
// Validation preprocessing
if let Err(validation_error) = validate_message_payload(&msg) {
    return Err(format!("Validation failed: {}", validation_error));
}
```

**When to Use**: On every service method that accepts user input.

---

### `#[validate_json]`

**Purpose**: Validates JSON payload structure and format.

**Usage**:
```rust
#[service_method("POST /api/data")]
#[validate_json]
pub async fn process_data(msg: Message) -> Result<RpcResponse, String> {
    // JSON structure is pre-validated
    let data: serde_json::Value = msg.get_json_payload()?;
    process_json_data(data).await?;
    Ok(RpcResponse::success(&"Processed", 0)?)
}
```

**Validation Features**:
- **Syntax Validation**: Ensures valid JSON syntax
- **Structure Validation**: Validates nested object structure
- **Array Validation**: Validates array elements
- **Null Handling**: Proper null value handling

**When to Use**: For endpoints accepting arbitrary JSON data.

---

### `#[validate_params]`

**Purpose**: Validates URL path parameters and query parameters.

**Usage**:
```rust
#[service_method("GET /users/:id")]
#[validate_params(
    id = "uuid",
    limit = "int:1..100",
    offset = "int:0.."
)]
pub async fn get_users(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?; // Guaranteed to be valid UUID
    let limit = msg.get_query_param("limit").unwrap_or("10".to_string());
    let offset = msg.get_query_param("offset").unwrap_or("0".to_string());
    
    let users = fetch_users(user_id, limit.parse()?, offset.parse()?).await?;
    Ok(RpcResponse::success(&users, 0)?)
}
```

**Parameter Validation Types**:
- **uuid**: Valid UUID format
- **int:min..max**: Integer within range
- **string:min..max**: String length constraints
- **email**: Valid email format
- **url**: Valid URL format
- **regex:pattern**: Custom regex pattern

**When to Use**: For endpoints with path or query parameters.

---

## Schema Validation Macros

### `#[validate_schema(schema)]`

**Purpose**: Validates input against JSON Schema specifications.

**Usage**:
```rust
#[service_method("POST /orders")]
#[require_auth]
#[validate_schema("order_creation")]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    // Validated against order creation schema
    let order_data: CreateOrderRequest = msg.deserialize_payload()?;
    
    let order = process_order_creation(order_data).await?;
    Ok(RpcResponse::success(&order, 0)?)
}
```

**Schema Configuration** (`schemas/order_creation.json`):
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["customer_id", "items", "shipping_address"],
  "properties": {
    "customer_id": {
      "type": "string",
      "format": "uuid"
    },
    "items": {
      "type": "array",
      "minItems": 1,
      "maxItems": 100,
      "items": {
        "type": "object",
        "required": ["product_id", "quantity", "price"],
        "properties": {
          "product_id": { "type": "string" },
          "quantity": { "type": "integer", "minimum": 1 },
          "price": { "type": "number", "minimum": 0 }
        }
      }
    },
    "shipping_address": {
      "type": "object",
      "required": ["street", "city", "postal_code", "country"],
      "properties": {
        "street": { "type": "string", "maxLength": 200 },
        "city": { "type": "string", "maxLength": 100 },
        "postal_code": { "type": "string", "pattern": "^[0-9]{5}(-[0-9]{4})?$" },
        "country": { "type": "string", "enum": ["US", "CA", "UK", "DE"] }
      }
    },
    "notes": {
      "type": "string",
      "maxLength": 500
    }
  }
}
```

**When to Use**: For complex data structures requiring comprehensive validation.

---

### `#[validate_openapi(operation)]`

**Purpose**: Validates input against OpenAPI specification.

**Usage**:
```rust
#[service_method("POST /api/v1/payments")]
#[validate_openapi("createPayment")]
pub async fn create_payment(msg: Message) -> Result<RpcResponse, String> {
    let payment_data: PaymentRequest = msg.deserialize_payload()?;
    let payment = process_payment(payment_data).await?;
    Ok(RpcResponse::success(&payment, 0)?)
}
```

**OpenAPI Specification**:
```yaml
paths:
  /api/v1/payments:
    post:
      operationId: createPayment
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/PaymentRequest'
components:
  schemas:
    PaymentRequest:
      type: object
      required: [amount, currency, payment_method]
      properties:
        amount:
          type: number
          minimum: 0.01
        currency:
          type: string
          enum: [USD, EUR, GBP]
        payment_method:
          type: object
          required: [type]
          properties:
            type:
              type: string
              enum: [card, bank_transfer, wallet]
```

**When to Use**: When you have existing OpenAPI specifications to enforce.

---

## Business Rule Validation Macros

### `#[validate_business_rules(rules)]`

**Purpose**: Enforces complex business logic validation.

**Usage**:
```rust
#[service_method("POST /loans/applications")]
#[require_auth]
#[validate]
#[validate_business_rules("loan_application")]
pub async fn apply_for_loan(msg: Message) -> Result<RpcResponse, String> {
    let application: LoanApplication = msg.deserialize_payload()?;
    
    // Business rules automatically validated:
    // - Age requirements
    // - Income thresholds
    // - Credit history requirements
    // - Debt-to-income ratios
    
    let result = process_loan_application(application).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**Business Rules Configuration**:
```json
{
  "loan_application": {
    "rules": [
      {
        "name": "minimum_age",
        "condition": "applicant.age >= 18",
        "message": "Applicant must be at least 18 years old"
      },
      {
        "name": "minimum_income",
        "condition": "applicant.annual_income >= 30000",
        "message": "Minimum annual income of $30,000 required"
      },
      {
        "name": "debt_to_income",
        "condition": "(applicant.monthly_debt / applicant.monthly_income) <= 0.43",
        "message": "Debt-to-income ratio cannot exceed 43%"
      },
      {
        "name": "loan_amount_limit",
        "condition": "loan.amount <= (applicant.annual_income * 5)",
        "message": "Loan amount cannot exceed 5x annual income"
      }
    ]
  }
}
```

**When to Use**: For domain-specific business logic that changes frequently.

---

### `#[validate_constraints(constraints)]`

**Purpose**: Enforces custom validation constraints with detailed error messages.

**Usage**:
```rust
#[service_method("PUT /users/:id/profile")]
#[require_auth]
#[validate_constraints([
    "age >= 13 && age <= 120",
    "email.contains('@') && email.len() <= 255",
    "phone_number.matches(r'^\\+?[1-9]\\d{1,14}$')",
    "password.len() >= 8 && password.contains_uppercase() && password.contains_digit()"
])]
pub async fn update_profile(msg: Message) -> Result<RpcResponse, String> {
    let profile_update: ProfileUpdate = msg.deserialize_payload()?;
    let updated_profile = update_user_profile(profile_update).await?;
    Ok(RpcResponse::success(&updated_profile, 0)?)
}
```

**Constraint Functions**:
- **String**: `len()`, `contains()`, `starts_with()`, `ends_with()`, `matches(regex)`
- **Numeric**: `>=`, `<=`, `>`, `<`, `==`, `!=`
- **Collections**: `len()`, `is_empty()`, `contains()`
- **Custom**: Define custom validation functions

**When to Use**: For specific validation logic that doesn't fit standard patterns.

---

## Advanced Validation Macros

### `#[validate_async(validator)]`

**Purpose**: Performs asynchronous validation (e.g., database lookups, external API calls).

**Usage**:
```rust
#[service_method("POST /users")]
#[validate]
#[validate_async("unique_email_validator")]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    let user_data: CreateUserRequest = msg.deserialize_payload()?;
    
    // Email uniqueness automatically validated against database
    let user = create_new_user(user_data).await?;
    Ok(RpcResponse::success(&user, 0)?)
}
```

**Async Validator Implementation**:
```rust
async fn unique_email_validator(data: &CreateUserRequest) -> Result<(), ValidationError> {
    if user_exists_by_email(&data.email).await? {
        return Err(ValidationError::new("email", "Email already exists"));
    }
    Ok(())
}
```

**Common Async Validations**:
- **Uniqueness**: Email, username, phone number uniqueness
- **Existence**: Validate foreign key references exist
- **External APIs**: Validate addresses, phone numbers, etc.
- **Rate Limits**: Check user-specific rate limits

**When to Use**: When validation requires database queries or external service calls.

---

### `#[validate_conditional(condition, validator)]`

**Purpose**: Applies validation only when certain conditions are met.

**Usage**:
```rust
#[service_method("POST /accounts")]
#[validate]
#[validate_conditional(
    condition = "account_type == 'business'",
    validator = "business_account_validator"
)]
pub async fn create_account(msg: Message) -> Result<RpcResponse, String> {
    let account_data: CreateAccountRequest = msg.deserialize_payload()?;
    
    // Business-specific validation only applied if account_type is 'business'
    let account = create_new_account(account_data).await?;
    Ok(RpcResponse::success(&account, 0)?)
}
```

**When to Use**: For type-specific or context-dependent validation.

---

### `#[validate_file_upload]`

**Purpose**: Validates file uploads including size, type, and content scanning.

**Usage**:
```rust
#[service_method("POST /documents/upload")]
#[require_auth]
#[validate_file_upload(
    max_size = "10MB",
    allowed_types = ["pdf", "docx", "txt"],
    scan_content = true
)]
pub async fn upload_document(msg: Message) -> Result<RpcResponse, String> {
    let file_data = msg.get_file_upload()?;
    
    // File automatically validated for:
    // - Size limits
    // - File type
    // - Malware scanning
    // - Content validation
    
    let document = save_uploaded_document(file_data).await?;
    Ok(RpcResponse::success(&document, 0)?)
}
```

**File Validation Features**:
- **Size Limits**: Maximum file size validation
- **Type Validation**: File extension and MIME type checking
- **Content Scanning**: Malware and virus scanning
- **Metadata Extraction**: Extract and validate file metadata

**When to Use**: For any endpoint that accepts file uploads.

---

## Data Transformation Macros

### `#[sanitize(rules)]`

**Purpose**: Sanitizes input data to prevent injection attacks and normalize data.

**Usage**:
```rust
#[service_method("POST /comments")]
#[require_auth]
#[validate]
#[sanitize([
    "content: html_escape, trim, max_length:1000",
    "author_name: trim, normalize_unicode, max_length:100"
])]
pub async fn create_comment(msg: Message) -> Result<RpcResponse, String> {
    let comment_data: CreateCommentRequest = msg.deserialize_payload()?;
    
    // Data is automatically sanitized:
    // - HTML escaped to prevent XSS
    // - Trimmed of whitespace
    // - Length limited
    
    let comment = create_new_comment(comment_data).await?;
    Ok(RpcResponse::success(&comment, 0)?)
}
```

**Sanitization Rules**:
- **html_escape**: Escape HTML entities
- **sql_escape**: Escape SQL special characters  
- **trim**: Remove leading/trailing whitespace
- **normalize_unicode**: Normalize Unicode characters
- **max_length**: Truncate to maximum length
- **remove_profanity**: Remove or mask profanity

**When to Use**: For user-generated content and any text input.

---

### `#[transform(transformers)]`

**Purpose**: Transforms input data to standard formats.

**Usage**:
```rust
#[service_method("POST /users")]
#[validate]
#[transform([
    "email: lowercase, trim",
    "phone_number: normalize_phone",
    "name: title_case, trim"
])]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    let user_data: CreateUserRequest = msg.deserialize_payload()?;
    
    // Data automatically transformed:
    // - Email to lowercase
    // - Phone number to standard format
    // - Name to title case
    
    let user = create_new_user(user_data).await?;
    Ok(RpcResponse::success(&user, 0)?)
}
```

**Transformation Functions**:
- **lowercase/uppercase**: Case conversion
- **title_case**: Convert to title case
- **normalize_phone**: Standard phone format
- **normalize_address**: Standard address format
- **currency_format**: Standard currency formatting

**When to Use**: To ensure consistent data formats across your application.

---

## Validation Error Handling Macros

### `#[validation_errors(handler)]`

**Purpose**: Custom validation error handling and response formatting.

**Usage**:
```rust
#[service_method("POST /users")]
#[validate]
#[validation_errors("detailed_error_handler")]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    let user_data: CreateUserRequest = msg.deserialize_payload()?;
    let user = create_new_user(user_data).await?;
    Ok(RpcResponse::success(&user, 0)?)
}
```

**Custom Error Handler**:
```rust
fn detailed_error_handler(errors: Vec<ValidationError>) -> RpcResponse {
    let error_details = errors.iter().map(|e| {
        json!({
            "field": e.field,
            "message": e.message,
            "code": e.error_code,
            "value": e.rejected_value
        })
    }).collect::<Vec<_>>();
    
    RpcResponse::error(
        400,
        "Validation failed",
        Some(json!({ "errors": error_details }))
    )
}
```

**When to Use**: When you need custom error response formats or detailed error information.

---

### `#[validate_and_collect_errors]`

**Purpose**: Collects all validation errors instead of failing on the first error.

**Usage**:
```rust
#[service_method("POST /complex-form")]
#[validate_and_collect_errors]
pub async fn process_complex_form(msg: Message) -> Result<RpcResponse, String> {
    let form_data: ComplexFormRequest = msg.deserialize_payload()?;
    
    // All validation errors collected and returned together
    // Better user experience for complex forms
    
    let result = process_form_submission(form_data).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**When to Use**: For complex forms where users need to see all validation issues at once.

---

## Integration Examples

### Complete Validation Stack
```rust
#[service_method("POST /api/v1/orders")]
#[require_auth]
#[rate_limit(requests = 100, per = "hour", by = "user")]
#[validate]
#[validate_schema("order_creation")]
#[validate_business_rules("order_validation")]
#[validate_async("inventory_check")]
#[sanitize(["notes: html_escape, trim, max_length:500"])]
#[audit_log(action = "order_creation")]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    let order_data: CreateOrderRequest = msg.deserialize_payload()?;
    
    // All validation layers automatically applied:
    // 1. Basic input validation
    // 2. Schema validation against JSON Schema
    // 3. Business rule validation
    // 4. Async inventory availability check
    // 5. Input sanitization
    // 6. Audit logging
    
    let order = process_order_creation(order_data).await?;
    Ok(RpcResponse::success(&order, 0)?)
}
```

### File Upload with Validation
```rust
#[service_method("POST /documents")]
#[require_auth]
#[require_permission("documents:write")]
#[validate_file_upload(
    max_size = "50MB",
    allowed_types = ["pdf", "docx", "xlsx", "pptx"],
    scan_content = true,
    extract_metadata = true
)]
#[validate_async("quota_check")]
pub async fn upload_document(msg: Message) -> Result<RpcResponse, String> {
    let file_upload = msg.get_file_upload()?;
    let metadata = msg.get_file_metadata()?;
    
    let document = save_document_with_metadata(file_upload, metadata).await?;
    Ok(RpcResponse::success(&document, 0)?)
}
```

### Multi-Step Validation
```rust
#[service_method("POST /financial/transactions")]
#[require_auth]
#[require_role("financial_user")]
#[validate]
#[validate_schema("financial_transaction")]
#[validate_business_rules("transaction_limits")]
#[validate_async("account_verification")]
#[validate_async("fraud_detection")]
#[sanitize(["description: trim, max_length:200"])]
#[audit_log(action = "financial_transaction")]
pub async fn create_transaction(msg: Message) -> Result<RpcResponse, String> {
    let transaction: TransactionRequest = msg.deserialize_payload()?;
    
    // Comprehensive validation pipeline:
    // 1. User authentication and authorization
    // 2. Input format validation
    // 3. Schema compliance validation
    // 4. Business rule validation (limits, etc.)
    // 5. Account verification
    // 6. Fraud detection screening
    // 7. Input sanitization
    // 8. Audit trail logging
    
    let result = process_financial_transaction(transaction).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

---

## Best Practices

### 1. Layer Validation Appropriately
```rust
// Basic endpoints
#[validate]

// User input endpoints  
#[validate]
#[sanitize(...)]

// Complex data endpoints
#[validate]
#[validate_schema(...)]
#[sanitize(...)]

// Business critical endpoints
#[validate]
#[validate_schema(...)]
#[validate_business_rules(...)]
#[validate_async(...)]
#[sanitize(...)]
#[audit_log]
```

### 2. Use Appropriate Error Handling
```rust
// Simple endpoints - fail fast
#[validate]

// Complex forms - collect all errors
#[validate_and_collect_errors]

// API endpoints - custom error format
#[validate]
#[validation_errors("api_error_handler")]
```

### 3. Combine with Security
```rust
#[service_method("POST /sensitive-data")]
#[require_auth]
#[require_permission("sensitive:write")]
#[rate_limit(requests = 10, per = "minute")]
#[validate]
#[validate_schema("sensitive_data")]
#[sanitize(...)]
#[audit_log]
pub async fn handle_sensitive_data(msg: Message) -> Result<RpcResponse, String> {
    // Implementation
}
```

---

This completes the Validation Macros documentation, providing comprehensive data validation and integrity features for microservice security and reliability.