# RabbitMesh Authorization Macros - Comprehensive Design

## 🎯 **Vision: Zero-Boilerplate Security**

Transform this disaster:
```rust
// ❌ 20+ lines of auth boilerplate in EVERY method!
#[service_method("POST /posts")]
pub async fn create_post(params: (String, CreatePostRequest)) -> Result<PostResponse, String> {
    let (auth_header, request) = params;
    let token = extract_token(&auth_header).ok_or("Missing auth")?;
    let auth_response = validate_token(&token).await?;
    if !auth_response.valid { return Err("Unauthorized"); }
    let user_id = auth_response.user_id.ok_or("Invalid token")?;
    let role = auth_response.role.ok_or("Invalid token")?;
    if !matches!(role, UserRole::Author | UserRole::Admin) {
        return Err("Insufficient permissions");
    }
    // Finally, business logic...
    let post = BlogPost::new(request.title, request.content, user_id, ...);
}
```

Into this elegance:
```rust
// ✅ Pure business logic - zero auth boilerplate!
#[service_method("POST /posts")]
#[require_role("Author")]
pub async fn create_post(
    user: AuthContext,  // ← Auto-injected, guaranteed valid
    request: CreatePostRequest
) -> Result<PostResponse, String> {
    // Pure business logic only!
    let post = BlogPost::new(request.title, request.content, user.user_id, user.username, ...);
}
```

---

## 🏗️ **1. RBAC (Role-Based Access Control)**

### **Basic Authentication**
```rust
#[service_method("GET /profile")]
#[require_auth] // Any authenticated user
pub async fn get_profile(
    user: AuthContext, // ← Auto-injected with user_id, username, role, email
    user_id: String
) -> Result<ProfileResponse, String> {
    // user.user_id, user.role, user.username guaranteed available
}
```

### **Role Requirements**
```rust
#[service_method("POST /posts")]
#[require_role("Author")] // Must be Author, Editor, or Admin
pub async fn create_post(
    user: AuthContext, // ← Guaranteed to have Author+ permissions
    request: CreatePostRequest
) -> Result<PostResponse, String> {
    // Business logic only - role already verified
}

#[service_method("DELETE /users/:id")]
#[require_role("Admin")] // Admin only
pub async fn delete_user(
    user: AuthContext, // ← Guaranteed Admin
    target_user_id: String
) -> Result<UserResponse, String> {
    // Admin-only operation
}
```

### **Multiple Role Options**
```rust
#[service_method("PUT /posts/:id")]
#[require_any_role("Author", "Editor", "Admin")]
pub async fn edit_post(
    user: AuthContext,
    post_id: String,
    request: UpdatePostRequest
) -> Result<PostResponse, String> {
    // Any of these roles can edit
}
```

---

## 🎯 **2. ABAC (Attribute-Based Access Control)**

### **Ownership-Based Access**
```rust
#[service_method("PUT /posts/:id")]
#[require_ownership(resource = "post", field = "author_id")]
pub async fn update_own_post(
    user: AuthContext,
    post_id: String,
    request: UpdatePostRequest
) -> Result<PostResponse, String> {
    // User can only edit their own posts
    // Macro auto-loads post and checks post.author_id == user.user_id
}
```

### **Complex Attribute Rules**
```rust
#[service_method("DELETE /comments/:id")]
#[require_attributes(
    // Allow if user owns the comment OR owns the post OR is admin
    user.user_id = resource.comment.author_id ||
    user.user_id = resource.post.author_id ||
    user.role = "Admin"
)]
pub async fn delete_comment(
    user: AuthContext,
    comment_id: String
) -> Result<CommentResponse, String> {
    // Complex authorization logic handled by macro
}
```

### **Time-Based Access**
```rust
#[service_method("POST /events/:id/register")]
#[require_attributes(
    user.role = "Member",
    event.registration_open = true,
    current_time < event.registration_deadline
)]
pub async fn register_for_event(
    user: AuthContext,
    event_id: String
) -> Result<RegistrationResponse, String> {
    // Time-sensitive operations with complex rules
}
```

### **Contextual Access**
```rust
#[service_method("GET /reports/:id")]
#[require_attributes(
    user.department = resource.report.department ||
    user.role = "Manager" ||
    user.user_id in resource.report.shared_with
)]
pub async fn view_report(
    user: AuthContext,
    report_id: String
) -> Result<ReportResponse, String> {
    // Department-based + role-based + sharing-based access
}
```

---

## 🔄 **3. Hybrid Approaches**

### **Layered Security**
```rust
#[service_method("POST /financial/transfer")]
#[require_role("AccountManager")] // Must have role
#[require_ownership(resource = "account", field = "owner_id")] // Must own account
#[require_attributes(
    transfer.amount <= user.daily_limit,
    account.status = "active"
)] // Additional business rules
pub async fn transfer_funds(
    user: AuthContext,
    account_id: String,
    request: TransferRequest
) -> Result<TransferResponse, String> {
    // Multiple layers of security - all enforced by macros
}
```

### **Progressive Authorization**
```rust
#[service_method("PUT /sensitive-data/:id")]
#[require_auth] // Basic auth first
#[require_2fa] // Then 2FA if available  
#[require_ownership] // Then ownership check
pub async fn update_sensitive_data(
    user: AuthContext,
    data_id: String,
    request: UpdateSensitiveRequest
) -> Result<DataResponse, String> {
    // Progressive security layers
}
```

---

## 🛡️ **4. Advanced Security Patterns**

### **Rate Limiting + Auth**
```rust
#[service_method("POST /api/upload")]
#[require_auth]
#[rate_limit(requests = 10, per = "minute", key = "user.user_id")]
pub async fn upload_file(
    user: AuthContext,
    file_data: FileUploadRequest
) -> Result<UploadResponse, String> {
    // Rate limited per authenticated user
}
```

### **Conditional Authorization**
```rust
#[service_method("GET /admin/logs")]
#[require_role("Admin")]
#[require_attributes(
    user.mfa_enabled = true, // Admin must have MFA
    request.ip in allowed_ips // IP whitelist for admin actions
)]
pub async fn view_admin_logs(
    user: AuthContext,
    filters: LogFilters
) -> Result<LogResponse, String> {
    // High-security admin operations
}
```

### **Dynamic Permission Loading**
```rust
#[service_method("POST /projects/:id/deploy")]
#[require_permission("project.deploy", resource = "project")]
pub async fn deploy_project(
    user: AuthContext, // ← Macro loads and verifies user has "project.deploy" permission for this project
    project_id: String,
    deployment_config: DeploymentConfig
) -> Result<DeploymentResponse, String> {
    // Permission-based access with resource context
}
```

---

## 🎨 **5. Elegant Parameter Patterns**

### **Smart Parameter Injection**
```rust
#[service_method("PUT /posts/:id")]
#[require_ownership] // Macro auto-determines ownership field
pub async fn update_post(
    user: AuthContext,    // ← Auto-injected authenticated user
    post: BlogPost,       // ← Auto-loaded and ownership-verified resource
    request: UpdatePostRequest
) -> Result<PostResponse, String> {
    // post is guaranteed to be owned by user
    post.update(request);
    // Save post...
}
```

### **Multiple Resource Loading**
```rust
#[service_method("POST /projects/:project_id/tasks")]
#[require_role("Developer")]
#[require_attributes(user.user_id in project.members)]
pub async fn create_task(
    user: AuthContext,           // ← User info
    project: Project,            // ← Auto-loaded project  
    request: CreateTaskRequest
) -> Result<TaskResponse, String> {
    // Both user and project guaranteed valid and authorized
}
```

---

## ⚙️ **6. Configuration & Customization**

### **Service-Level Configuration**
```rust
#[service_definition]
#[auth_config(
    auth_service = "auth-service",
    default_role = "Reader",
    require_2fa_for_roles = ["Admin", "Manager"],
    session_timeout = 3600
)]
pub struct BlogService;
```

### **Custom Authorization Logic**
```rust
#[service_method("POST /special-operation")]
#[require_custom_auth(validator = "validate_special_permissions")]
pub async fn special_operation(
    user: AuthContext,
    request: SpecialRequest
) -> Result<SpecialResponse, String> {
    // Custom validation function called by macro
}

// Custom validator
async fn validate_special_permissions(
    user: &AuthContext, 
    request: &SpecialRequest
) -> Result<(), String> {
    // Custom business logic
    if user.role == UserRole::Admin && request.is_emergency {
        Ok(())
    } else {
        Err("Special permissions required".to_string())
    }
}
```

---

## 🚀 **7. Implementation Architecture**

### **Macro Processing Pipeline**
```rust
// 1. Parse method signature
// 2. Extract authorization attributes  
// 3. Generate authentication code
// 4. Generate parameter injection code
// 5. Generate resource loading code
// 6. Generate authorization validation code
// 7. Wrap original method with all layers
// 8. Register enhanced handler with RabbitMQ
```

### **Generated Code Pattern**
```rust
// From this:
#[service_method("POST /posts")]
#[require_role("Author")]
pub async fn create_post(user: AuthContext, request: CreatePostRequest) -> Result<PostResponse, String> {
    let post = BlogPost::new(request.title, request.content, user.user_id, user.username, ...);
    Ok(PostResponse { success: true, post: Some(post) })
}

// Macro generates:
pub async fn create_post_impl(request: CreatePostRequest, user_context: AuthContext) -> Result<PostResponse, String> {
    let post = BlogPost::new(request.title, request.content, user_context.user_id, user_context.username, ...);
    Ok(PostResponse { success: true, post: Some(post) })
}

// And registers this handler:
service.register_function("create_post", |msg: Message| async move {
    // 1. Extract authorization header from message
    let auth_header = extract_auth_header(&msg)?;
    
    // 2. Validate token with auth service
    let user_context = AUTH_MIDDLEWARE.authenticate_and_authorize(&auth_header, UserRole::Author).await?;
    
    // 3. Deserialize request
    let request: CreatePostRequest = msg.deserialize_payload()?;
    
    // 4. Call actual method with injected context
    let result = Self::create_post_impl(request, user_context).await;
    
    // 5. Serialize and return response
    Ok(RpcResponse::success(result?, 0)?)
}).await;
```

---

## 🎯 **8. Developer Experience**

### **IDE Support**
- Full autocomplete for `user.user_id`, `user.role`, etc.
- Compile-time verification of authorization attributes
- Clear error messages for invalid configurations

### **Documentation Generation**
```rust
// Automatically generates API documentation:
// POST /posts - Requires: Author role - Creates a new blog post
// PUT /posts/:id - Requires: Author role + ownership - Updates existing post
// DELETE /posts/:id - Requires: Admin role OR ownership - Deletes post
```

### **Testing Support**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rabbitmesh_auth::testing::MockAuthContext;

    #[tokio::test]
    async fn test_create_post() {
        let user = MockAuthContext::author("john_doe");
        let request = CreatePostRequest { ... };
        
        let result = BlogService::create_post_impl(request, user).await;
        assert!(result.is_ok());
    }
}
```

---

## 🎉 **Benefits**

### ✅ **For Developers**
- **Write only business logic** - zero auth boilerplate
- **Guaranteed type safety** - user context always valid  
- **Consistent security** - same patterns across all services
- **Rapid development** - add auth with single attribute

### ✅ **For Security**
- **Centralized auth logic** - update once, apply everywhere
- **No security bugs** - impossible to forget authorization
- **Auditable** - clear authorization requirements in code
- **Testable** - isolated auth logic with mocks

### ✅ **For Architecture**  
- **Scalable** - add 100 services with consistent auth
- **Maintainable** - change auth patterns in one place
- **Observable** - automatic auth metrics and logging
- **Flexible** - supports any auth pattern (RBAC/ABAC/hybrid)

---

**This would make RabbitMesh the most developer-friendly AND secure microservices framework ever created!** ✨