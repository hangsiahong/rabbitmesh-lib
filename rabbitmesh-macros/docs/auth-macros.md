# Authentication & Authorization Macros

Comprehensive security macros for JWT validation, role-based access control (RBAC), attribute-based access control (ABAC), and advanced permission management.

## Overview

Authentication macros provide enterprise-grade security features for microservices, handling token validation, user authorization, and fine-grained permission control with zero boilerplate.

---

## JWT Authentication Macros

### `#[require_auth]`

**Purpose**: Validates JWT tokens and injects authenticated user context into service methods.

**Usage**:
```rust
#[service_method("GET /profile")]
#[require_auth]
pub async fn get_profile(msg: Message) -> Result<RpcResponse, String> {
    // User context is automatically injected
    let user_id = msg.get_user_id()?; // Available after auth validation
    let user_roles = msg.get_user_roles()?;
    
    let profile = fetch_user_profile(user_id).await?;
    Ok(RpcResponse::success(&profile, 0)?)
}
```

**Features**:
- **Automatic Token Extraction**: Extracts JWT from Authorization header
- **Token Validation**: Validates signature, expiration, and claims
- **User Context Injection**: Injects user information into message context
- **Error Handling**: Returns proper HTTP 401/403 responses

**Generated Code**:
```rust
// Before method execution
let auth_header = msg.get_header("authorization")
    .ok_or("Missing Authorization header")?;
let token = extract_bearer_token(auth_header)?;
let user_context = validate_jwt_token(token).await?;
msg.set_user_context(user_context);
```

**When to Use**: On any endpoint that requires user authentication.

---

### `#[require_auth(optional)]`

**Purpose**: Optional authentication - validates tokens if present but doesn't require them.

**Usage**:
```rust
#[service_method("GET /public-content")]
#[require_auth(optional)]
pub async fn get_public_content(msg: Message) -> Result<RpcResponse, String> {
    let content = if msg.is_authenticated() {
        // Personalized content for authenticated users
        get_personalized_content(msg.get_user_id()?).await?
    } else {
        // Generic content for anonymous users
        get_public_content().await?
    };
    
    Ok(RpcResponse::success(&content, 0)?)
}
```

**When to Use**: For endpoints that provide different experiences for authenticated vs anonymous users.

---

### `#[jwt_validate(config)]`

**Purpose**: Advanced JWT validation with custom configuration.

**Usage**:
```rust
#[service_method("GET /admin")]
#[jwt_validate(
    issuer = "https://auth.company.com",
    audience = "microservice-api",
    algorithms = ["RS256", "ES256"],
    leeway = 30
)]
pub async fn admin_endpoint(msg: Message) -> Result<RpcResponse, String> {
    // Validated with strict JWT configuration
    Ok(RpcResponse::success(&"admin data", 0)?)
}
```

**Configuration Options**:
- **issuer**: Required token issuer
- **audience**: Required token audience
- **algorithms**: Allowed signing algorithms
- **leeway**: Clock skew tolerance (seconds)
- **required_claims**: List of required claims

**When to Use**: When you need strict JWT validation parameters.

---

## Role-Based Access Control (RBAC)

### `#[require_role(role)]`

**Purpose**: Restricts access to users with specific roles.

**Usage**:
```rust
#[service_method("DELETE /users/:id")]
#[require_auth]
#[require_role("admin")]
pub async fn delete_user(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    delete_user_by_id(user_id).await?;
    Ok(RpcResponse::success(&"User deleted", 0)?)
}

#[service_method("GET /reports")]
#[require_auth]
#[require_role("manager")]
pub async fn get_reports(msg: Message) -> Result<RpcResponse, String> {
    let reports = fetch_management_reports().await?;
    Ok(RpcResponse::success(&reports, 0)?)
}
```

**Common Roles**:
- `admin`: System administrators
- `manager`: Department managers
- `user`: Regular users
- `guest`: Limited access users

**When to Use**: When access should be restricted based on user roles.

---

### `#[require_any_role(roles)]`

**Purpose**: Allows access if user has any of the specified roles.

**Usage**:
```rust
#[service_method("GET /financial-data")]
#[require_auth]
#[require_any_role(["admin", "finance_manager", "auditor"])]
pub async fn get_financial_data(msg: Message) -> Result<RpcResponse, String> {
    let data = fetch_financial_data().await?;
    Ok(RpcResponse::success(&data, 0)?)
}
```

**When to Use**: When multiple roles should have access to the same resource.

---

### `#[require_all_roles(roles)]`

**Purpose**: Requires user to have all specified roles (rare but useful for compound permissions).

**Usage**:
```rust
#[service_method("POST /security-audit")]
#[require_auth]
#[require_all_roles(["security_officer", "admin"])]
pub async fn initiate_security_audit(msg: Message) -> Result<RpcResponse, String> {
    // Only users with both security_officer AND admin roles
    initiate_audit().await?;
    Ok(RpcResponse::success(&"Audit initiated", 0)?)
}
```

**When to Use**: For highly sensitive operations requiring compound authority.

---

## Attribute-Based Access Control (ABAC)

### `#[require_permission(permission)]`

**Purpose**: Checks fine-grained permissions based on user attributes and resource context.

**Usage**:
```rust
#[service_method("PUT /documents/:id")]
#[require_auth]
#[require_permission("documents:write")]
pub async fn update_document(msg: Message) -> Result<RpcResponse, String> {
    let doc_id = msg.get_path_param("id")?;
    update_document_content(doc_id, &msg.get_payload()?).await?;
    Ok(RpcResponse::success(&"Document updated", 0)?)
}

#[service_method("GET /sensitive-data")]
#[require_auth]
#[require_permission("sensitive_data:read")]
pub async fn get_sensitive_data(msg: Message) -> Result<RpcResponse, String> {
    let data = fetch_sensitive_data().await?;
    Ok(RpcResponse::success(&data, 0)?)
}
```

**Permission Format**: `resource:action` (e.g., `users:read`, `orders:write`, `reports:admin`)

**When to Use**: For fine-grained access control beyond simple roles.

---

### `#[require_ownership]`

**Purpose**: Ensures users can only access resources they own.

**Usage**:
```rust
#[service_method("GET /users/:id/private-data")]
#[require_auth]
#[require_ownership]
pub async fn get_private_data(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let current_user = msg.get_user_id()?;
    
    // Macro automatically validates user_id == current_user
    let private_data = fetch_private_data(user_id).await?;
    Ok(RpcResponse::success(&private_data, 0)?)
}

#[service_method("PUT /orders/:id")]
#[require_auth]
#[require_ownership(resource = "order", field = "owner_id")]
pub async fn update_order(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    // Macro checks that order.owner_id == current_user_id
    update_order(order_id, &msg.get_payload()?).await?;
    Ok(RpcResponse::success(&"Order updated", 0)?)
}
```

**Configuration**:
- **resource**: Resource type to check ownership for
- **field**: Field name containing the owner ID
- **allow_admin**: Whether admins bypass ownership checks

**When to Use**: For user-owned resources like profiles, orders, or personal data.

---

### `#[policy_check(policy)]`

**Purpose**: Evaluates complex authorization policies using a policy engine.

**Usage**:
```rust
#[service_method("POST /approve-expense/:id")]
#[require_auth]
#[policy_check("expense_approval")]
pub async fn approve_expense(msg: Message) -> Result<RpcResponse, String> {
    let expense_id = msg.get_path_param("id")?;
    
    // Policy evaluates:
    // - User role (manager+)
    // - Expense amount limits
    // - Department approval rules
    // - Time-based constraints
    
    approve_expense_request(expense_id).await?;
    Ok(RpcResponse::success(&"Expense approved", 0)?)
}
```

**Policy Examples**:
```json
{
  "expense_approval": {
    "rules": [
      {
        "condition": "user.role == 'manager' && expense.amount <= 1000",
        "effect": "allow"
      },
      {
        "condition": "user.role == 'director' && expense.amount <= 5000",
        "effect": "allow"
      },
      {
        "condition": "user.department == expense.department",
        "effect": "allow"
      }
    ]
  }
}
```

**When to Use**: For complex authorization logic that can't be expressed with simple roles or permissions.

---

## Advanced Authentication Macros

### `#[api_key_auth(scope)]`

**Purpose**: Validates API keys for service-to-service authentication.

**Usage**:
```rust
#[service_method("POST /internal/sync")]
#[api_key_auth(scope = "internal")]
pub async fn internal_sync(msg: Message) -> Result<RpcResponse, String> {
    // Only valid internal API keys can access this
    perform_internal_sync().await?;
    Ok(RpcResponse::success(&"Sync completed", 0)?)
}

#[service_method("GET /public-api/data")]
#[api_key_auth(scope = "public", rate_limit = "1000/hour")]
pub async fn public_api_data(msg: Message) -> Result<RpcResponse, String> {
    let data = fetch_public_data().await?;
    Ok(RpcResponse::success(&data, 0)?)
}
```

**Features**:
- **Scope Validation**: Different API key scopes for different access levels
- **Rate Limiting**: Built-in rate limiting per API key
- **Usage Tracking**: Automatic API key usage metrics

**When to Use**: For API endpoints consumed by external services or partners.

---

### `#[oauth2_validate(provider)]`

**Purpose**: Validates OAuth2 tokens from external providers.

**Usage**:
```rust
#[service_method("GET /social-data")]
#[oauth2_validate(provider = "google")]
pub async fn get_social_data(msg: Message) -> Result<RpcResponse, String> {
    let google_user = msg.get_oauth_user()?;
    let data = fetch_user_social_data(&google_user.id).await?;
    Ok(RpcResponse::success(&data, 0)?)
}

#[service_method("POST /github-webhook")]
#[oauth2_validate(provider = "github", scope = "repo")]
pub async fn github_webhook(msg: Message) -> Result<RpcResponse, String> {
    process_github_webhook(&msg.get_payload()?).await?;
    Ok(RpcResponse::success(&"Processed", 0)?)
}
```

**Supported Providers**:
- `google`: Google OAuth2
- `github`: GitHub OAuth2
- `microsoft`: Microsoft Azure AD
- `facebook`: Facebook OAuth2
- `custom`: Custom OAuth2 provider

**When to Use**: For integration with external OAuth2 providers.

---

### `#[mutual_tls]`

**Purpose**: Validates client certificates for mutual TLS authentication.

**Usage**:
```rust
#[service_method("POST /secure-transfer")]
#[mutual_tls]
pub async fn secure_transfer(msg: Message) -> Result<RpcResponse, String> {
    let client_cert = msg.get_client_certificate()?;
    let transfer_data = msg.get_payload()?;
    
    process_secure_transfer(client_cert, transfer_data).await?;
    Ok(RpcResponse::success(&"Transfer completed", 0)?)
}
```

**When to Use**: For highly secure service-to-service communication.

---

## Security Enhancement Macros

### `#[rate_limit(limit)]`

**Purpose**: Implements rate limiting based on user or API key.

**Usage**:
```rust
#[service_method("POST /login")]
#[rate_limit(requests = 5, per = "minute", by = "ip")]
pub async fn login(msg: Message) -> Result<RpcResponse, String> {
    // Limited to 5 login attempts per minute per IP
    let credentials = msg.get_payload()?;
    let token = authenticate_user(credentials).await?;
    Ok(RpcResponse::success(&token, 0)?)
}

#[service_method("GET /api/data")]
#[require_auth]
#[rate_limit(requests = 1000, per = "hour", by = "user")]
pub async fn get_data(msg: Message) -> Result<RpcResponse, String> {
    // 1000 requests per hour per authenticated user
    let data = fetch_data().await?;
    Ok(RpcResponse::success(&data, 0)?)
}
```

**Rate Limiting Options**:
- **by**: `ip`, `user`, `api_key`, `header:X-Client-Id`
- **per**: `second`, `minute`, `hour`, `day`
- **burst**: Allow burst capacity
- **distributed**: Use Redis for distributed rate limiting

**When to Use**: To prevent abuse and ensure fair resource usage.

---

### `#[security_headers]`

**Purpose**: Adds security headers to responses.

**Usage**:
```rust
#[service_method("GET /web-app")]
#[security_headers]
pub async fn web_app(msg: Message) -> Result<RpcResponse, String> {
    let content = render_web_app().await?;
    Ok(RpcResponse::success(&content, 0)?)
}
```

**Generated Headers**:
```
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
```

**When to Use**: For web-facing endpoints that serve HTML content.

---

### `#[audit_log(action)]`

**Purpose**: Logs security-sensitive actions for compliance and monitoring.

**Usage**:
```rust
#[service_method("DELETE /users/:id")]
#[require_auth]
#[require_role("admin")]
#[audit_log(action = "user_deletion")]
pub async fn delete_user(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let admin_user = msg.get_user_id()?;
    
    // Automatically logged:
    // - Action: user_deletion
    // - Admin user: admin_user
    // - Target user: user_id
    // - Timestamp, IP, etc.
    
    delete_user_by_id(user_id).await?;
    Ok(RpcResponse::success(&"User deleted", 0)?)
}
```

**Logged Information**:
- Action type and result
- User performing the action
- Resource being accessed/modified
- Timestamp and request context
- IP address and user agent

**When to Use**: For compliance, security monitoring, and forensic analysis.

---

## Integration Examples

### Complete Authentication Flow
```rust
use rabbitmesh_macros::*;

#[service_impl]
impl AuthService {
    #[service_method("POST /auth/login")]
    #[rate_limit(requests = 5, per = "minute", by = "ip")]
    #[audit_log(action = "login_attempt")]
    pub async fn login(msg: Message) -> Result<RpcResponse, String> {
        let credentials: LoginRequest = msg.deserialize_payload()?;
        
        let user = authenticate_user(&credentials).await
            .map_err(|_| "Invalid credentials")?;
        
        let token = generate_jwt_token(&user)?;
        
        Ok(RpcResponse::success(&AuthResponse {
            token,
            expires_in: 3600,
            user: user.into(),
        }, 0)?)
    }
    
    #[service_method("GET /auth/profile")]
    #[require_auth]
    #[cached(ttl = 300)]
    pub async fn get_profile(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_user_id()?;
        let profile = fetch_user_profile(user_id).await?;
        Ok(RpcResponse::success(&profile, 0)?)
    }
    
    #[service_method("PUT /auth/profile")]
    #[require_auth]
    #[require_ownership]
    #[audit_log(action = "profile_update")]
    pub async fn update_profile(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_user_id()?;
        let updates: ProfileUpdate = msg.deserialize_payload()?;
        
        let profile = update_user_profile(user_id, updates).await?;
        Ok(RpcResponse::success(&profile, 0)?)
    }
}
```

### Role-Based Service Protection
```rust
#[service_impl]
impl AdminService {
    #[service_method("GET /admin/users")]
    #[require_auth]
    #[require_role("admin")]
    #[audit_log(action = "user_list_access")]
    pub async fn list_users(msg: Message) -> Result<RpcResponse, String> {
        let users = fetch_all_users().await?;
        Ok(RpcResponse::success(&users, 0)?)
    }
    
    #[service_method("POST /admin/roles/:user_id")]
    #[require_auth]
    #[require_role("super_admin")]
    #[audit_log(action = "role_assignment")]
    pub async fn assign_role(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_path_param("user_id")?;
        let role_request: AssignRoleRequest = msg.deserialize_payload()?;
        
        assign_user_role(user_id, &role_request.role).await?;
        Ok(RpcResponse::success(&"Role assigned", 0)?)
    }
}
```

### API Key Service Integration
```rust
#[service_impl]
impl PublicApiService {
    #[service_method("GET /api/v1/data")]
    #[api_key_auth(scope = "public")]
    #[rate_limit(requests = 1000, per = "hour", by = "api_key")]
    pub async fn get_public_data(msg: Message) -> Result<RpcResponse, String> {
        let api_key_info = msg.get_api_key_info()?;
        let data = fetch_data_for_client(&api_key_info.client_id).await?;
        Ok(RpcResponse::success(&data, 0)?)
    }
    
    #[service_method("POST /api/v1/webhook")]
    #[api_key_auth(scope = "webhook")]
    #[validate]
    pub async fn webhook_endpoint(msg: Message) -> Result<RpcResponse, String> {
        let webhook_data: WebhookPayload = msg.deserialize_payload()?;
        process_webhook(webhook_data).await?;
        Ok(RpcResponse::success(&"Processed", 0)?)
    }
}
```

---

## Best Practices

### 1. Layer Security Appropriately
```rust
// Public endpoints
#[rate_limit(requests = 100, per = "minute")]

// User endpoints
#[require_auth]
#[rate_limit(requests = 1000, per = "hour", by = "user")]

// Admin endpoints
#[require_auth]
#[require_role("admin")]
#[audit_log]

// Sensitive operations
#[require_auth]
#[require_permission("sensitive_data:read")]
#[policy_check("data_access_policy")]
#[audit_log]
```

### 2. Combine with Validation
```rust
#[service_method("POST /users")]
#[require_auth]
#[require_role("admin")]
#[validate] // Always validate input
#[audit_log(action = "user_creation")]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    // Implementation
}
```

### 3. Use Ownership for User Data
```rust
#[service_method("GET /users/:id/private")]
#[require_auth]
#[require_ownership] // Users can only access their own data
pub async fn get_private_data(msg: Message) -> Result<RpcResponse, String> {
    // Implementation
}
```

---

This completes the Authentication & Authorization Macros documentation, providing comprehensive security features for microservice protection.